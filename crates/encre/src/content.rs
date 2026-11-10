use crate::schema::{Schema, schema};
use std::sync::OnceLock;

struct Term {
    names: Vec<String>,
    min: usize,
    max: usize,
}

fn split_terms(expr: &str) -> Vec<String> {
    let mut terms = vec![String::new()];
    let mut depth = 0;
    for c in expr.chars() {
        depth += i32::from(c == '(') - i32::from(c == ')');
        match c {
            ' ' if depth == 0 => terms.push(String::new()),
            _ => terms.last_mut().expect("terms is never empty").push(c),
        }
    }
    terms.retain(|term| !term.is_empty());
    terms
}

fn range(term: &str) -> Option<(&str, usize, usize)> {
    let (body, bounds) = term.strip_suffix('}')?.split_once('{')?;
    let (min, max) = bounds.split_once(',').unwrap_or((bounds, bounds));
    Some((body, min.trim().parse().ok()?, max.trim().parse().unwrap_or(usize::MAX)))
}

fn quantified(term: &str) -> (&str, usize, usize) {
    match term.chars().last() {
        Some('*') => (&term[..term.len() - 1], 0, usize::MAX),
        Some('+') => (&term[..term.len() - 1], 1, usize::MAX),
        Some('?') => (&term[..term.len() - 1], 0, 1),
        Some('}') => range(term).unwrap_or((term, 1, 1)),
        _ => (term, 1, 1),
    }
}

fn parse(expr: &str) -> Vec<Term> {
    split_terms(expr)
        .into_iter()
        .map(|term| {
            let (body, min, max) = quantified(&term);
            let names = body.trim_matches(['(', ')']).split('|').map(|name| name.trim().to_owned()).collect();
            Term { names, min, max }
        })
        .collect()
}

fn accepts(schema: &Schema, term: &Term, kind: &str) -> bool {
    term.names.iter().any(|name| name == kind || schema.node(kind).is_some_and(|node| node.in_group(name)))
}

fn matches_from(schema: &Schema, terms: &[Term], kinds: &[&str]) -> bool {
    let Some((term, rest)) = terms.split_first() else {
        return kinds.is_empty();
    };
    let mut taken = 0;
    while taken < term.min {
        if !kinds.get(taken).is_some_and(|kind| accepts(schema, term, kind)) {
            return false;
        }
        taken += 1;
    }
    loop {
        if matches_from(schema, rest, &kinds[taken..]) {
            return true;
        }
        if taken >= term.max || !kinds.get(taken).is_some_and(|kind| accepts(schema, term, kind)) {
            return false;
        }
        taken += 1;
    }
}

fn compiled(expr: &str) -> Option<&'static [Term]> {
    static TERMS: OnceLock<Vec<(&'static str, Vec<Term>)>> = OnceLock::new();
    let terms = TERMS.get_or_init(|| schema().nodes.iter().map(|n| (n.content.as_str(), parse(&n.content))).collect());
    let same = |e: &&str| std::ptr::eq(*e, expr);
    terms.iter().find(|(e, _)| same(e)).or_else(|| terms.iter().find(|(e, _)| *e == expr)).map(|(_, t)| &t[..])
}

/// Whether child node types `kinds` satisfy the ProseMirror content expression `expr`.
pub fn matches(schema: &Schema, expr: &str, kinds: &[&str]) -> bool {
    match compiled(expr) {
        Some(terms) => matches_from(schema, terms, kinds),
        None => matches_from(schema, &parse(expr), kinds),
    }
}

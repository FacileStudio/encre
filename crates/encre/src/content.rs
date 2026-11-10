use crate::schema::Schema;

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

fn parse(expr: &str) -> Vec<Term> {
    split_terms(expr)
        .into_iter()
        .map(|term| {
            let (body, min, max) = match term.chars().last() {
                Some('*') => (&term[..term.len() - 1], 0, usize::MAX),
                Some('+') => (&term[..term.len() - 1], 1, usize::MAX),
                Some('?') => (&term[..term.len() - 1], 0, 1),
                _ => (term.as_str(), 1, 1),
            };
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

/// Whether child node types `kinds` satisfy the ProseMirror content expression `expr`.
pub fn matches(schema: &Schema, expr: &str, kinds: &[&str]) -> bool {
    matches_from(schema, &parse(expr), kinds)
}

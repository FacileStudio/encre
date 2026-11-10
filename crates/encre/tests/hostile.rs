//! Crafted worst cases for every path a server runs on untrusted markdown. Each one runs at 1 MiB and 8 MiB
//! and must scale linearly: a quadratic path costs 64 times more at 8 MiB, a linear one 8 times. Run them
//! with `cargo test --release -p encre --test hostile -- --ignored --test-threads=1 --nocapture`.

use encre::markdown::{parse, plain_text, serialize, validate};
use std::time::Duration;

const MIB: usize = 1 << 20;

fn timed<T>(f: impl FnOnce() -> T) -> Duration {
    let cpu = || {
        let stat = std::fs::read_to_string("/proc/thread-self/schedstat").unwrap_or_default();
        match stat.split_whitespace().next().and_then(|n| n.parse().ok()) {
            Some(nanos) => Duration::from_nanos(nanos),
            None => std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default(),
        }
    };
    let start = cpu();
    std::hint::black_box(f());
    cpu().saturating_sub(start)
}

fn repeat(unit: &str, size: usize) -> String {
    let mut out = unit.repeat(size / unit.len() + 1);
    let mut cut = size;
    while !out.is_char_boundary(cut) {
        cut -= 1;
    }
    out.truncate(cut);
    out
}

fn costs(md: &str) -> [Duration; 4] {
    let mut doc = None;
    let parsed = timed(|| doc = parse(md).ok());
    let validated = timed(|| validate(md).map(|_| ()));
    let doc = doc.unwrap_or_else(|| parse("").expect("empty markdown parses"));
    [parsed, validated, timed(|| serialize(&doc)), timed(|| plain_text(&doc))]
}

fn within(name: &str, step: &str, one: Duration, eight: Duration) {
    println!("{name:18} {step:10} 1 MiB {one:>10.1?}  8 MiB {eight:>10.1?}");
    let budget = one.max(Duration::from_millis(20)) * 20;
    assert!(eight <= budget, "{name}: {step} took {eight:?} at 8 MiB and {one:?} at 1 MiB");
}

const UNITS: &[(&str, &str)] = &[
    ("open brackets", "["),
    ("bracket text", "[a"),
    ("unclosed badges", "[badge:info x "),
    ("badges", "[badge:info x] [status:done] "),
    ("backslashes", "\\"),
    ("equals", "="),
    ("highlight openers", "==a "),
    ("alerts", "> [!NOTE]\n"),
    ("deep quote", ">"),
    ("deep list", "- "),
    ("links", "[a](b) "),
    ("paragraphs", "a\n\n"),
    ("marks", "*a*b"),
    ("flanking", "**a**b*c* "),
];

type Maker = fn(usize) -> String;

type Input = Box<dyn Fn(usize) -> String>;

fn shaped() -> Vec<(&'static str, Maker)> {
    vec![
        ("backslash runs", |n| repeat(&format!("{}[", "\\".repeat(64)), n)),
        ("deep emphasis", |n| format!("{}a{}", repeat("*", n / 2), repeat("*", n / 2))),
        ("fence ladder", |n| {
            let open: String = (0..8).rev().map(|d| format!("{}callout\n", "`".repeat(d + 3))).collect();
            let close: String = (0..8).map(|d| format!("\n{}", "`".repeat(d + 3))).collect();
            format!("{open}{}{close}", repeat("x\n", n))
        }),
        ("compare", |n| format!("```compare\n{}```", repeat("### pro\nx\n", n))),
        ("columns", |n| format!("````columns\n{}````", repeat("```column\nx\n```\n", n))),
        ("digits then dots", |n| format!("{}{}", repeat("1", n / 2), repeat(".", n / 2))),
        ("table padding", |n| format!("|{}\n|{}\n{}", repeat("a|", 2048), repeat("-|", 2048), repeat("x\n", n))),
        ("backtick runs", |size| {
            let runs = (1..).scan(0, |len, n| {
                *len += n + 1;
                (*len < size).then(|| "`".repeat(n) + " ")
            });
            runs.collect()
        }),
    ]
}

#[test]
#[ignore = "benchmark"]
fn every_markdown_path_is_linear() {
    let only = std::env::var("HOSTILE_CASE").unwrap_or_default();
    let units = UNITS.iter().map(|(name, unit)| (*name, Box::new(move |n| repeat(unit, n)) as Input));
    let shaped = shaped().into_iter().map(|(name, f)| (name, Box::new(f) as Input));
    let steps = ["parse", "validate", "serialize", "plain_text"];
    for (name, input) in units.chain(shaped).filter(|(name, _)| only.is_empty() || *name == only) {
        let (small, large) = (costs(&input(MIB)), costs(&input(8 * MIB)));
        for ((step, one), eight) in steps.iter().zip(small).zip(large) {
            within(name, step, one, eight);
        }
    }
}

fn patch(size: usize) -> Duration {
    let mut doc = parse(&repeat("a", size)).expect("parses");
    encre::assign_ids(&mut doc);
    let op = encre::ops::Op::Patch { find: "a".to_owned(), replace: "b".to_owned(), expect: 1 };
    let error = encre::ops::apply(&mut doc, 1, &op).expect_err("matches every byte");
    assert!(error.to_string().contains(&format!("found {size}")), "{error}");
    timed(|| encre::ops::apply(&mut doc, 1, &op).map_err(|e| e.to_string().len()))
}

#[test]
#[ignore = "benchmark"]
fn patch_matching_is_linear() {
    within("patch matches", "apply", patch(MIB), patch(8 * MIB));
}

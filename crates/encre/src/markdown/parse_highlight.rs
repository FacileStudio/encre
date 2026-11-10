use super::parse_span::{Ch, Unit};

fn is_eq(unit: Option<&Unit>) -> bool {
    matches!(unit, Some(Unit::Ch(ch)) if ch.c == '=' && ch.syntax)
}

fn solid(unit: Option<&Unit>) -> bool {
    !matches!(unit, None | Some(Unit::Ch(Ch { space: true, .. })))
}

/// The indexes of the `==` runs that open and close a highlight, in pairs.
pub fn delimiters(units: &[Unit]) -> Vec<usize> {
    let (mut pairs, mut open) = (Vec::new(), None);
    let mut i = 0;
    while i < units.len() {
        let run = units[i..].iter().take_while(|u| is_eq(Some(u))).count();
        if run == 0 {
            i += 1;
            continue;
        }
        match open {
            Some(start) if run == 2 && solid(i.checked_sub(1).and_then(|p| units.get(p))) => {
                pairs.extend([start, i]);
                open = None;
            }
            _ if run == 2 && solid(units.get(i + 2)) => open = Some(i),
            _ => {}
        }
        i += run;
    }
    pairs
}

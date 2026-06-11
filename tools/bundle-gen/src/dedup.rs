//! Name-based gap-fill: SRD 5.2 records win; SRD 5.1 records whose
//! normalized name is absent from the 5.2 set are appended.

use std::collections::{BTreeMap, BTreeSet};

/// Lowercase + strip everything non-alphanumeric, then apply the curated
/// alias map (5.1 name -> 5.2 name) so renamed records dedup correctly.
pub fn normalize(name: &str, aliases: &BTreeMap<String, String>) -> String {
    let squash = |s: &str| -> String {
        s.chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|c| c.to_ascii_lowercase())
            .collect()
    };
    let squashed = squash(name);
    aliases
        .iter()
        .find(|(from, _)| squash(from) == squashed)
        .map(|(_, to)| squash(to))
        .unwrap_or(squashed)
}

/// Returns the 5.2 base records plus 5.1 records that fill name gaps,
/// along with the gap-filled names for the review report.
pub fn gap_fill<T>(
    base: Vec<T>,
    legacy: Vec<T>,
    name_of: impl Fn(&T) -> String,
    aliases: &BTreeMap<String, String>,
) -> (Vec<T>, Vec<String>) {
    let mut seen: BTreeSet<String> =
        base.iter().map(|r| normalize(&name_of(r), aliases)).collect();
    let mut out = base;
    let mut filled = Vec::new();
    for rec in legacy {
        // `insert` also guards against duplicate names *within* the
        // legacy set (upstream 5.1 data has a few).
        if seen.insert(normalize(&name_of(&rec), aliases)) {
            filled.push(name_of(&rec));
            out.push(rec);
        }
    }
    (out, filled)
}

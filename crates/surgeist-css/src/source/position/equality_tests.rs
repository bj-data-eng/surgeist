//! Equality of source-bound values compares their source text only when the
//! two sides do not already share one immutable snapshot. Sharing a snapshot
//! means sharing its text, so identity answers the source part of the
//! comparison without rescanning it; distinct snapshots keep text equality.

use super::SNAPSHOT_TEXT_BYTES_COMPARED;
use crate::source::CssSourceSnapshot;

fn compared_bytes(compare: impl FnOnce() -> bool) -> (bool, usize) {
    SNAPSHOT_TEXT_BYTES_COMPARED.with(|compared| compared.set(0));
    let equal = compare();
    (
        equal,
        SNAPSHOT_TEXT_BYTES_COMPARED.with(std::cell::Cell::get),
    )
}

fn large_sheet(rules: usize) -> String {
    (0..rules)
        .map(|index| format!(".r{index} {{ color: red; margin: {index}px 2px; }}\n"))
        .collect()
}

#[test]
fn values_sharing_one_snapshot_compare_without_reading_its_text() {
    let source = large_sheet(400);
    let report = crate::CssParser::default().parse_sheet(&source);
    let sheet = report.syntax();
    let copy = sheet.clone();
    let (equal, bytes) = compared_bytes(|| *sheet == copy);
    assert!(equal);
    assert_eq!(bytes, 0, "shared-snapshot equality rescanned source text");

    let values = crate::component::values::parse_component_values(&source).unwrap();
    let copy = values.clone();
    let (equal, bytes) = compared_bytes(|| values == copy);
    assert!(equal);
    assert_eq!(bytes, 0, "shared-snapshot equality rescanned source text");

    let snapshot = CssSourceSnapshot::new(&source);
    let (equal, bytes) = compared_bytes(|| snapshot == snapshot.clone());
    assert!(equal);
    assert_eq!(bytes, 0);
}

#[test]
fn distinct_snapshots_keep_text_equality() {
    let source = large_sheet(20);
    let first = crate::CssParser::default().parse_sheet(&source);
    let second = crate::CssParser::default().parse_sheet(&source);
    assert_eq!(first.syntax(), second.syntax());
    assert_eq!(
        CssSourceSnapshot::new(&source),
        CssSourceSnapshot::new(&source)
    );

    let changed = source.replacen(".r0 ", ".r0  ", 1);
    assert_ne!(
        CssSourceSnapshot::new(&source),
        CssSourceSnapshot::new(&changed)
    );
    let values = crate::component::values::parse_component_values("a /* x */ b").unwrap();
    let other = crate::component::values::parse_component_values("a /* y */ b").unwrap();
    assert_ne!(values, other);
    let same = crate::component::values::parse_component_values("a /* x */ b").unwrap();
    assert_eq!(values, same);
}

//! `border`, `outline` and `column-rule` share the `<line-width> ||
//! <line-style> || <color>` family. When every member equals its initial
//! value, CSSOM "serialize a CSS value" step 2 omits all but one member and
//! prefers the shortest representation; the grammar requires one member, and
//! `none` (the initial style) is shorter than `medium` or `currentcolor`.
//! The family therefore keeps the one fallback token `none`.

use surgeist_css::*;

fn cssom(source: &str) -> String {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax())
        .unwrap()
        .serialize_cssom()
        .unwrap()
}

#[test]
fn all_initial_line_shorthands_share_the_none_fallback() {
    for (source, expected) in [
        ("border: none", "border: none;"),
        ("outline: none", "outline: none;"),
        ("column-rule: none", "column-rule: none;"),
        ("column-rule: medium", "column-rule: none;"),
        (
            "column-rule: medium none currentcolor",
            "column-rule: none;",
        ),
        (
            "column-rule-width: medium; column-rule-style: none; column-rule-color: currentcolor",
            "column-rule: none;",
        ),
    ] {
        assert_eq!(cssom(source), expected, "{source}");
    }
}

#[test]
fn column_rule_keeps_non_initial_members_without_the_fallback() {
    for (source, expected) in [
        ("column-rule: thin", "column-rule: thin;"),
        ("column-rule: medium solid", "column-rule: solid;"),
        ("column-rule: red", "column-rule: red;"),
        ("column-rule: thin none red", "column-rule: thin red;"),
    ] {
        assert_eq!(cssom(source), expected, "{source}");
    }
}

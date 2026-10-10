#![forbid(unsafe_code)]
//! Literal expectations from CSSOM's selected 2026 CSSKeyframesRule branch.

use surgeist_css::{
    CssRule, CssRuleCssomSerializationErrorKind, CssSpecifiedValueSerializationErrorKind,
    CssSpecifiedValueSerializationLimits, parse_sheet,
};

fn rule(source: &str) -> CssRule {
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [rule] = report.syntax().rules() else {
        panic!("one keyframes rule");
    };
    rule.clone()
}

#[test]
fn decoded_nonreserved_quoted_name_uses_identifier_output() {
    let value = rule("@keyframes \"fade in\" {}");
    assert_eq!(
        value.serialize_cssom().unwrap(),
        "@keyframes fade\\ in { \n}"
    );
    assert_eq!(
        value.to_specified_css().unwrap(),
        "@keyframes \"fade in\" { }"
    );
}

#[test]
fn selected_wrapper_keeps_space_before_first_indented_child_and_terminal_lf() {
    for (source, expected) in [
        ("@keyframes k{}", "@keyframes k { \n}"),
        ("@keyframes k{from{}}", "@keyframes k {   0% { }\n}"),
        (
            "@keyframes k{from{opacity:0}to{opacity:1}}",
            "@keyframes k {   0% { opacity: 0; }\n  100% { opacity: 1; }\n}",
        ),
    ] {
        let value = rule(source);
        assert_eq!(value.serialize_cssom().unwrap(), expected, "{source}");
    }
}

#[test]
fn reserved_decoded_names_keep_string_output() {
    for name in [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "default",
        "none",
        "INITIAL",
    ] {
        let value = rule(&format!("@keyframes \"{name}\" {{}}"));
        assert_eq!(
            value.serialize_cssom().unwrap(),
            format!("@keyframes \"{name}\" {{ \n}}")
        );
    }
}

#[test]
fn selected_output_byte_failure_is_atomic_and_retry_keeps_original_occurrences() {
    let report = parse_sheet("/*😀*/ @keyframes \"fade in\" { from { opacity:0 } }");
    assert!(report.is_clean(), "{report:?}");
    let before = report.clone();
    let CssRule::Keyframes(frames) = &report.syntax().rules()[0] else {
        panic!("keyframes");
    };
    let declaration = &frames.blocks()[0].declarations()[0];
    let saved = declaration.clone();
    let value = &report.syntax().rules()[0];
    let expected = "@keyframes fade\\ in {   0% { opacity: 0; }\n}";
    assert_eq!(value.serialize_cssom().unwrap(), expected);
    let adequate = CssSpecifiedValueSerializationLimits::new(100_000, 100_000, expected.len());
    assert_eq!(
        value.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
    let failure = value
        .serialize_cssom_with_limits(CssSpecifiedValueSerializationLimits::new(
            100_000,
            100_000,
            expected.len() - 1,
        ))
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        CssRuleCssomSerializationErrorKind::Resource(
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        )
    );
    assert_eq!(report, before);
    assert!(declaration.source().same_occurrence(saved.source()));
    assert_eq!(
        value.serialize_cssom_with_limits(adequate).unwrap(),
        expected
    );
}

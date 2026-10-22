//! A misplaced at-rule is dropped with one placement diagnostic that names the
//! rule and the context it requires, whichever rule list rejects it.
//!
//! - Nesting 1 §3.2: beneath a style rule only conditional group rules,
//!   `@layer` and `@scope` are admitted, so global definitions are rejected.
//!   <https://www.w3.org/TR/2026/WD-css-nesting-1-20260122/#nesting-other-at-rules>
//! - Cascade 6 §2.5: a `@scope` body admits what a style rule body admits
//!   besides declarations, so `@page` is rejected there too.
//!   <https://www.w3.org/TR/2024/WD-css-cascade-6-20240906/#scope-syntax>
//! - Cascade 5 §6.1: `@import` is valid only at the stylesheet top level.
use surgeist_css::*;

const WITHOUT_STYLE_ANCESTOR: &str = "a rule list without a style-rule ancestor";
const PAGE: &str = "a rule list without a style-rule ancestor that is not itself a @scope body";
const TOP_LEVEL: &str = "the stylesheet top level";

#[test]
fn each_misplaced_rule_reports_one_placement_diagnostic_naming_its_context() {
    let cases = [
        // A definition in a style rule body, in a group nested in one, and in
        // a scope nested in one shares one requirement.
        (
            ".a { @font-face { font-family: x; } color: red; }",
            "@font-face",
            "font-face",
            WITHOUT_STYLE_ANCESTOR,
        ),
        (
            ".a { @media screen { @keyframes k { from { opacity: 0 } } } }",
            "@keyframes",
            "keyframes",
            WITHOUT_STYLE_ANCESTOR,
        ),
        (
            ".a { @scope (.b) { @font-face { font-family: x; } } }",
            "@font-face",
            "font-face",
            WITHOUT_STYLE_ANCESTOR,
        ),
        // `@page` is excluded beneath a style rule and directly in `@scope`.
        (".a { @page { margin: 0 } }", "@page", "page", PAGE),
        ("@scope (.a) { @page { margin: 0 } }", "@page", "page", PAGE),
        // A root-only statement inside a group.
        (
            "@media screen { @import \"x.css\"; }",
            "@import",
            "import",
            TOP_LEVEL,
        ),
    ];
    for (source, rule, name, context) in cases {
        let report = parse_sheet(source);
        let [diagnostic] = report.diagnostics() else {
            panic!("{source:?}: one placement diagnostic: {report:?}");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidAtRulePlacement,
            "{source:?}"
        );
        let ErrorKind::InvalidAtRulePlacement(detail) = diagnostic.error().kind() else {
            panic!("{source:?}: placement owns the rejection");
        };
        assert_eq!(detail.name().as_str(), name, "{source:?}");
        assert_eq!(detail.expected_context().as_str(), context, "{source:?}");
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            source.find(rule).unwrap(),
            "{source:?}"
        );
        assert_eq!(
            validate_sheet(source).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

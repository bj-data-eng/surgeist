#![forbid(unsafe_code)]
//! Supplemental public An+B controls from CSS Syntax 3 CRD 2021-12-24 §6.2.
//! Literal pairs use the full signless integer before applying the separate '-'.
//! Canonical text follows §10.1. The final test separately characterizes the
//! established bounded-provider behavior outside the public i32 coefficient model.
//! <https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/>

use surgeist_css::{
    CssNamespaceContext, CssNthPattern, CssPseudoClass, CssPseudoSelectorListItem,
    CssRecoveryAction, CssSelector, parse_selector,
};

fn parsed(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.syntax().as_ref().expect("admitted selector").clone()
}

fn coefficients(pattern: CssNthPattern) -> (i32, i32) {
    match pattern {
        CssNthPattern::Odd => (2, 1),
        CssNthPattern::Even => (2, 0),
        CssNthPattern::Integer(b) => (0, b),
        CssNthPattern::AnPlusB(value) => (value.a(), value.b()),
        _ => panic!("unexpected nth pattern: {pattern:?}"),
    }
}

fn nth_pair(selector: &CssSelector) -> (i32, i32) {
    match selector {
        CssSelector::PseudoClass(CssPseudoClass::NthChild(value))
        | CssSelector::PseudoClass(CssPseudoClass::NthLastChild(value)) => {
            coefficients(value.pattern())
        }
        CssSelector::PseudoClass(CssPseudoClass::NthOfType(value))
        | CssSelector::PseudoClass(CssPseudoClass::NthLastOfType(value)) => coefficients(*value),
        _ => panic!("one nth pseudo-class: {selector:?}"),
    }
}

fn assert_argument(argument: &str, expected: (i32, i32), canonical: &str) {
    let source = format!(":nth-child({argument})");
    let selector = parsed(&source);
    assert_eq!(nth_pair(&selector), expected, "{source}");
    let before = selector.clone();
    let output = selector.to_specified_css().unwrap();
    assert_eq!(output, format!(":nth-child({canonical})"), "{source}");
    assert_eq!(selector, before);
    assert_eq!(nth_pair(&parsed(&output)), expected, "{source}");
}

#[test]
fn leading_zero_signless_offsets_use_the_full_value_across_ordinary_and_boundary_magnitudes() {
    for (argument, a, b, canonical) in [
        ("n - 0000", 1, 0, "n"),
        ("2n- 0002", 2, -2, "2n-2"),
        ("-n - 00017", -1, -17, "-n-17"),
        ("n - 0002147483646", 1, -2147483646, "n-2147483646"),
        ("n- 0002147483647", 1, -2147483647, "n-2147483647"),
        ("n - 0002147483648", 1, i32::MIN, "n-2147483648"),
        (
            "-2n- 0000000000000000000002147483648",
            -2,
            i32::MIN,
            "-2n-2147483648",
        ),
    ] {
        assert_argument(argument, (a, b), canonical);
    }
}

#[test]
fn comments_and_each_css_whitespace_form_preserve_separate_minus_offset_roles() {
    for (argument, a, b, canonical) in [
        ("n/**/-/**/0002", 1, -2, "n-2"),
        ("n-/**/0002", 1, -2, "n-2"),
        ("n \t-\r\n 0002147483648", 1, i32::MIN, "n-2147483648"),
        (
            "2n/**/-/**/\u{c}0002147483648/**/",
            2,
            i32::MIN,
            "2n-2147483648",
        ),
        ("+/**/n- /**/0002147483648", 1, i32::MIN, "n-2147483648"),
        ("-n-\r/**/0002147483648", -1, i32::MIN, "-n-2147483648"),
    ] {
        assert_argument(argument, (a, b), canonical);
    }
}

#[test]
fn decoded_n_ident_and_dimension_units_keep_the_signless_offset_mapping() {
    for (argument, a, b, canonical) in [
        (r"\6e /**/-/**/0002", 1, -2, "n-2"),
        (r"2\4e /**/-/**/0002147483648", 2, i32::MIN, "2n-2147483648"),
        (r"2\6e - /**/0002147483648", 2, i32::MIN, "2n-2147483648"),
        (r"-3\4e -/**/0002147483648", -3, i32::MIN, "-3n-2147483648"),
        (r"+/**/\6e -/**/0002147483648", 1, i32::MIN, "n-2147483648"),
    ] {
        assert_argument(argument, (a, b), canonical);
    }
}

#[test]
fn every_nth_wrapper_preserves_the_corrected_pair_and_argument_end() {
    for name in [
        "nth-child",
        "nth-last-child",
        "nth-of-type",
        "nth-last-of-type",
    ] {
        let source = format!(":{name}(-2n-/**/0002147483648)");
        let selector = parsed(&source);
        assert_eq!(nth_pair(&selector), (-2, i32::MIN), "{source}");
        let output = selector.to_specified_css().unwrap();
        assert_eq!(output, format!(":{name}(-2n-2147483648)"));
        assert_eq!(nth_pair(&parsed(&output)), (-2, i32::MIN));
    }
}

#[test]
fn consumed_offset_leaves_of_members_and_following_pseudo_class_intact() {
    for name in ["nth-child", "nth-last-child"] {
        let source = format!(":{name}(2n/**/-/**/0002147483648/**/ of .first,#last):hover");
        let selector = parsed(&source);
        let CssSelector::Compound(compound) = &selector else {
            panic!("nth and following hover share the compound: {selector:?}");
        };
        let [nth, CssPseudoClass::Hover] = compound.pseudo_classes() else {
            panic!("both pseudo-classes remain in order: {selector:?}");
        };
        let child = match nth {
            CssPseudoClass::NthChild(value) | CssPseudoClass::NthLastChild(value) => value,
            _ => panic!("child-indexed nth wrapper"),
        };
        assert_eq!(coefficients(child.pattern()), (2, i32::MIN));
        let [
            CssPseudoSelectorListItem::Selector(first),
            CssPseudoSelectorListItem::Selector(last),
        ] = child
            .selector_list()
            .expect("of-list remains pending after the number")
            .items()
        else {
            panic!("two of-list members survive");
        };
        assert_eq!(first, &CssSelector::Class("first".into()));
        assert_eq!(last, &CssSelector::Key("last".into()));
        assert_eq!(
            selector.to_specified_css().unwrap(),
            format!(":{name}(2n-2147483648 of .first, #last):hover")
        );
    }
}

#[test]
fn trivia_and_escaped_units_do_not_admit_signed_or_number_flag_offsets_after_a_minus() {
    for argument in [
        "n -/**/+0002",
        "n-/**/-0002",
        "n -/**/0002.0",
        "n-/**/0002e0",
        r"2\6e -/**/+0002",
        r"2\6e -/**/0002%",
    ] {
        let source = format!(":nth-child({argument})");
        let report = parse_selector(&source, &CssNamespaceContext::default());
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean(), "{source}: {report:?}");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput }),
            "{source}: {report:?}"
        );
    }
}

#[test]
fn out_of_model_offsets_preserve_the_existing_delegated_bounded_result() {
    // Source-backed characterization of unchanged provider behavior only.
    // These final mathematical offsets do not fit i32; this does not assert
    // unbounded CSS arithmetic or prescribe a new rejection/clamping policy.
    for (argument, b, canonical) in [
        ("n - 2147483649", -2147483647, "n-2147483647"),
        (
            "n- /**/00099999999999999999999999",
            -2147483647,
            "n-2147483647",
        ),
        ("n + 2147483648", 2147483647, "n+2147483647"),
    ] {
        assert_argument(argument, (1, b), canonical);
    }
}

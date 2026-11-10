use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::CssSpecifiedDeclarationBlock;
use surgeist_css::rule::CssSheet;
use surgeist_css::selector::{CssNamespaceContext, CssSelector};
use surgeist_css::source::same_provenance;
use surgeist_css::values::common::{CssIdent, CssUrlModifierFunction};

fn components(source: &str) -> CssComponentValues {
    CssComponentValues::parse(&CssParser::default(), source).unwrap()
}

fn sheet(source: &str) -> CssSheet {
    let report = CssParser::default().parse_sheet(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().clone()
}

#[test]
fn declarations_and_supports_tests_equal_across_the_spelling_of_one_value() {
    for (left, right) in [
        ("a { width: 1.0px }", "a { width: 1.00px }"),
        ("a { margin: 1e1px }", "a { margin: 1E1px }"),
        (
            "@supports (width: 1.0px) {}",
            "@supports (width: 1.00px) {}",
        ),
    ] {
        let (left, right) = (sheet(left), sheet(right));
        assert_eq!(left, right);
        assert!(!same_provenance(&left, &right));
    }
    // A typed value compares by exact value, so `1px` equals `1.0px` there; the values differ.
    assert_eq!(sheet("a { width: 1.0px }"), sheet("a { width: 1px }"));
    assert_ne!(sheet("a { width: 1.0px }"), sheet("a { width: 2.0px }"));
}

#[test]
fn url_modifiers_equal_across_the_spelling_of_their_arguments() {
    let modifier = |arguments: &str| {
        CssUrlModifierFunction::try_new(
            CssIdent::try_new("integrity").unwrap(),
            components(arguments),
        )
        .unwrap()
    };
    let (double, single) = (modifier(r#""x""#), modifier("'x'"));
    assert_eq!(double, single);
    assert!(!same_provenance(&double, &single));
    assert_ne!(double, modifier(r#""y""#));
}

#[test]
fn invalid_selector_members_equal_across_whitespace_text() {
    let parse = |source: &str| {
        CssSelector::parse(
            &CssParser::default(),
            source,
            &CssNamespaceContext::default(),
        )
        .syntax()
        .clone()
        .unwrap()
    };
    let tab = parse(":is(.A,\t:NoPe(inner(&)))");
    let spaces = parse(":is(.A,  :NoPe(inner(&)))");
    assert_eq!(tab, spaces);
    assert!(!same_provenance(&tab, &spaces));
    assert_ne!(tab, parse(":is(.A,\t:NoPe(other(&)))"));
}

// Side shortening compares retained math by its authored form, so spelling that `==`
// ignores still keeps two equal-valued sides apart; ordinary literals shorten by value.
#[test]
fn shortening_compares_the_authored_form_of_math_that_equality_ignores() {
    let cssom = |source: &str| {
        let report = CssParser::default().parse_block_contents(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        CssSpecifiedDeclarationBlock::try_from_declarations(report.syntax(), Default::default())
            .unwrap()
            .serialize_cssom(Default::default())
            .unwrap()
    };
    assert_eq!(cssom("margin: 1.0px 1.00px"), "margin: 1px;");
    assert_eq!(
        cssom("margin: calc(1.0px + 2px) calc(1.0px + 2px)"),
        "margin: calc(3px);"
    );
    assert_eq!(
        cssom("margin: calc(1.0px + 2px) calc(1.00px + 2px)"),
        "margin: calc(3px) calc(3px);"
    );
    assert_eq!(
        cssom("margin: calc(1px + 2px) calc((1px + 2px))"),
        "margin: calc(3px) calc(3px);"
    );
}

#[test]
fn hex_colors_equal_across_digit_case_and_differ_in_provenance() {
    use surgeist_css::color::CssHexColor;
    let hex = |digits: &str| CssHexColor::try_new(digits).unwrap();
    for (left, right) in [
        ("ABC", "abc"),
        ("AbCdEf", "abcdef"),
        ("ABCD", "abcd"),
        ("AABBCCDD", "aabbccdd"),
    ] {
        let (left, right) = (hex(left), hex(right));
        assert_eq!(left, right);
        assert!(!same_provenance(&left, &right));
        assert!(same_provenance(&left, &left.clone()));
    }
    // The digit count is part of the specified value.
    assert_ne!(hex("abc"), hex("aabbcc"));
    assert_ne!(hex("abc"), hex("abd"));
}

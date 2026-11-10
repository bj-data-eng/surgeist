use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues};
use surgeist_css::numeric::{CssCalculation, CssLength};
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::same_provenance;

fn calculation(source: &str) -> CssCalculation<CssLength> {
    CssCalculation::try_from_components(
        CssComponentValues::parse(&CssParser::default(), source).unwrap(),
        Default::default(),
    )
    .unwrap()
}

#[test]
fn a_parsed_media_query_list_equals_the_list_built_from_its_members() {
    let report = CssMediaQueryList::parse(&CssParser::default(), "a, b");
    assert!(report.is_clean());
    let parsed = report.syntax();
    let built = CssMediaQueryList::new(parsed.queries().to_vec());
    assert_eq!(parsed, &built);
    assert!(parsed.parse_source().is_some() && built.parse_source().is_none());
    assert!(!same_provenance(parsed, &built));
    assert!(same_provenance(parsed, &parsed.clone()));
}

#[test]
fn media_query_lists_parsed_from_different_text_are_equal_when_their_members_are() {
    let compact = CssMediaQueryList::parse(&CssParser::default(), "a,b");
    let spaced = CssMediaQueryList::parse(&CssParser::default(), "a ,  b");
    assert_eq!(compact.syntax(), spaced.syntax());
    assert!(!same_provenance(compact.syntax(), spaced.syntax()));
    let other = CssMediaQueryList::parse(&CssParser::default(), "a, c");
    assert_ne!(compact.syntax(), other.syntax());
    assert!(!same_provenance(compact.syntax(), other.syntax()));
}

#[test]
fn sheets_equal_across_spans_and_differ_in_provenance() {
    let plain = CssParser::default().parse_sheet("a { width: 2px }");
    let shifted = CssParser::default().parse_sheet("/* leading */\n\n  a { width: 2px }");
    assert!(plain.is_clean() && shifted.is_clean());
    assert_eq!(plain.syntax(), shifted.syntax());
    assert!(!same_provenance(plain.syntax(), shifted.syntax()));
    let again = CssParser::default().parse_sheet("a { width: 2px }");
    assert!(same_provenance(plain.syntax(), again.syntax()));
    let other = CssParser::default().parse_sheet("a { width: 3px }");
    assert_ne!(plain.syntax(), other.syntax());
}

#[test]
fn declarations_equal_across_positions_and_differ_by_occurrence() {
    let first = CssParser::default().parse_block_contents("width: 2px");
    let second = CssParser::default().parse_block_contents("color: red; width: 2px");
    let (first, second) = (&first.syntax()[0], &second.syntax()[1]);
    assert_ne!(first.position(), second.position());
    assert_eq!(first, second);
    assert!(!same_provenance(first, second));
    assert!(!first.same_occurrence(second));
    assert!(first.same_occurrence(&first.clone()));
    let important = CssParser::default().parse_block_contents("width: 2px !important");
    assert_ne!(first, &important.syntax()[0]);
}

#[test]
fn parsed_components_equal_the_programmatic_ones_of_the_same_spelling() {
    let parsed = CssComponentValues::parse(&CssParser::default(), "2px").unwrap();
    let built = CssComponentValue::try_token("2px").unwrap();
    assert_eq!(&parsed.items()[0], &built);
    assert!(!same_provenance(&parsed.items()[0], &built));
    // The type flag is token data: an integer and a number with the same value differ.
    let respelled = CssComponentValue::try_token("2.0px").unwrap();
    assert_ne!(&parsed.items()[0], &respelled);
}

fn components(source: &str) -> CssComponentValues {
    CssComponentValues::parse(&CssParser::default(), source).unwrap()
}

#[test]
fn component_values_equal_across_spelling_and_differ_in_provenance() {
    for (left, right) in [
        ("1.0px", "1.00px"),
        ("1e1px", "1E1px"),
        ("5.0%", "5.00%"),
        (r"a\62", "ab"),
        (r#""x""#, "'x'"),
        (r"'\78'", r#""x""#),
        ("url(a)", "url( a )"),
        ("#a", r"#\61"),
        (r"f\75nc(1)", "func(1)"),
        ("(1.0)", "(1.00)"),
        (r"[a\62]", "[ab]"),
        (" ", "\n\t"),
    ] {
        let (left, right) = (components(left), components(right));
        assert_eq!(left, right, "{left:?} {right:?}");
        assert!(!same_provenance(&left, &right), "{left:?} {right:?}");
        assert!(same_provenance(&left, &left.clone()));
    }
}

#[test]
fn component_values_keep_decoded_data_and_numeric_type_in_equality() {
    for (left, right) in [
        ("1px", "1.0px"),
        ("+1", "1"),
        ("-1", "1"),
        ("1px", "2px"),
        ("1PX", "1px"),
        ("1px", "1em"),
        ("1%", "1"),
        ("1.5", "1.5px"),
        ("a", "b"),
        (r#""x""#, r#""y""#),
        ("#a", "#b"),
        ("func(1)", "func(2)"),
        ("(1)", "[1]"),
        ("/*a*/", "/*b*/"),
    ] {
        let (left, right) = (components(left), components(right));
        assert_ne!(left, right, "{left:?} {right:?}");
    }
}

#[test]
fn numeric_values_equal_across_spelling_and_differ_in_provenance() {
    let value = calculation("calc(1px + 2px)");
    let respelled = calculation("CALC(1.0px + 2px)");
    assert_eq!(value, respelled);
    assert!(!same_provenance(&value, &respelled));
    let shifted = calculation("  calc(1px + 2px)");
    assert_eq!(value, shifted);
    assert!(!same_provenance(&value, &shifted));
    assert!(same_provenance(&value, &calculation("calc(1px + 2px)")));
    assert_ne!(value, calculation("calc(2px + 1px)"));
}

#[test]
fn diagnostics_equal_across_positions_and_differ_in_provenance() {
    let first = CssParser::default().parse_block_contents("width: red");
    let second = CssParser::default().parse_block_contents("color: red; width: red");
    let (first, second) = (&first.diagnostics()[0], &second.diagnostics()[0]);
    assert_ne!(first.span(), second.span());
    assert_ne!(first.error().position(), second.error().position());
    assert_eq!(first, second);
    assert!(!same_provenance(first, second));
    assert!(same_provenance(first, &first.clone()));
    let other = CssParser::default().parse_block_contents("width: 1 2 3");
    assert_ne!(first, &other.diagnostics()[0]);
}

#[test]
fn declaration_lists_compare_through_the_same_rule() {
    let first = CssParser::default().parse_block_contents("width: 2px; height: 3px");
    let second = CssParser::default().parse_block_contents("width:2px;/* c */height:3px");
    assert_eq!(first.syntax().as_slice(), second.syntax().as_slice());
    assert!(!same_provenance(
        first.syntax().as_slice(),
        second.syntax().as_slice()
    ));
}

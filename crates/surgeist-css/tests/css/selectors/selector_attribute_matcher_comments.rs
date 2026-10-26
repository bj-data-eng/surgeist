//! Selected Selectors 4 <attr-matcher> consists of optional ~ | ^ $ * then =.
//! Its components forbid whitespace; Syntax 3 section 9 makes comments inert.

use crate::css::support::color_and_selector_fixtures::assert_selector_input_rejected as assert_whole_input_rejected;
use crate::css::support::color_and_selector_fixtures::assert_strict_validation_rejects;
use surgeist_css::{
    CssAttributeCaseSensitivity as Case, CssAttributeMatcher as Matcher,
    CssNamespaceConstraint as Namespace, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssParsedStyleSelectors, CssParser, CssQualifiedNamePrefix as Prefix, CssRecoveryAction,
    CssRelativeSelectorList, CssRule, CssSelector, CssSelectorCombinator, CssStyleSelectorContext,
};

type AttributeOperator = (&'static str, fn(String) -> Matcher);

const OPERATORS: [AttributeOperator; 5] = [
    ("~", Matcher::Includes),
    ("|", Matcher::DashMatch),
    ("^", Matcher::Prefix),
    ("$", Matcher::Suffix),
    ("*", Matcher::Substring),
];

#[derive(Clone, Copy, Debug)]
enum Front {
    Selector,
    List,
    Relative,
    Sheet,
}

const FRONTS: [Front; 4] = [Front::Selector, Front::List, Front::Relative, Front::Sheet];

// Clean positive syntax must reach the real front before model/output assertions.
// The surrounding members also prove that list boundaries survive retention.
fn retained(
    front: Front,
    source: &str,
    context: &CssNamespaceContext,
) -> Result<CssSelector, String> {
    let rejected = || format!("{front:?}: {source:?}");
    match front {
        Front::Selector => {
            let report = CssSelector::parse(&CssParser::default(), source, context);
            if !report.is_clean() {
                return Err(rejected());
            }
            Ok(report.into_validation_result().unwrap().unwrap())
        }
        Front::List => {
            let report = CssParsedStyleSelectors::parse(
                &CssParser::default(),
                format!(".Lead, {source}"),
                context,
                CssStyleSelectorContext::Ordinary,
            );
            if !report.is_clean() {
                return Err(rejected());
            }
            let list = report.into_validation_result().unwrap().unwrap();
            assert_eq!(list.selectors().selectors().len(), 2);
            assert_eq!(
                list.selectors().selectors()[0].selector(),
                &CssSelector::Class("Lead".into())
            );
            Ok(list.selectors().selectors()[1].selector().clone())
        }
        Front::Relative => {
            let report = CssRelativeSelectorList::parse(
                &CssParser::default(),
                format!("> .Lead, + {source}"),
                context,
            );
            if !report.is_clean() {
                return Err(rejected());
            }
            let list = report.into_validation_result().unwrap().unwrap();
            assert_eq!(list.selectors().len(), 2);
            assert_eq!(
                list.selectors()[0].combinator(),
                CssSelectorCombinator::Child
            );
            assert_eq!(
                list.selectors()[0].selector(),
                &CssSelector::Class("Lead".into())
            );
            assert_eq!(
                list.selectors()[1].combinator(),
                CssSelectorCombinator::NextSibling
            );
            Ok(list.selectors()[1].selector().clone())
        }
        Front::Sheet => {
            let report = CssParser::default().parse_sheet(format!(".Lead, {source} {{}}"));
            if !report.is_clean() {
                return Err(rejected());
            }
            let sheet = report.into_validation_result().unwrap();
            let [CssRule::Style(style)] = sheet.rules() else {
                panic!("one complete style rule");
            };
            assert_eq!(style.selectors().selectors().len(), 2);
            assert_eq!(
                style.selectors().selectors()[0].selector(),
                &CssSelector::Class("Lead".into())
            );
            Ok(style.selectors().selectors()[1].selector().clone())
        }
    }
}

fn check(
    selector: &CssSelector,
    expected_matcher: Matcher,
    case: Case,
    namespace: &Namespace,
    prefix: &Prefix,
    literal: &str,
) {
    let CssSelector::Compound(compound) = selector else {
        panic!("attribute compound");
    };
    assert_eq!(compound.scope_anchors(), 0);
    assert_eq!(compound.nesting_selectors(), 0);
    assert_eq!(compound.type_selector(), None);
    assert!(compound.ids().is_empty() && compound.classes().is_empty());
    assert!(compound.pseudo_classes().is_empty() && compound.pseudo_elements().is_none());
    let [attribute] = compound.attributes() else {
        panic!("one retained attribute");
    };
    assert_eq!(attribute.name().as_str(), "Data");
    assert_eq!(attribute.matcher(), &expected_matcher);
    assert_eq!(attribute.case_sensitivity(), case);
    assert_eq!(attribute.namespace(), namespace);
    assert_eq!(attribute.qualified_name().prefix(), prefix);
    assert_eq!(
        selector.serialize_specified(Default::default()).unwrap(),
        literal
    );
}

fn comments(front: Front) {
    let context = CssNamespaceContext::default();
    let mut failures = Vec::new();
    for (symbol, constructor) in OPERATORS {
        for gap in ["/**/", "/*first*//*second*/"] {
            for (operand, modifier, case, output_modifier) in [
                (r#""M\69 X""#, "I", Case::AsciiCaseInsensitive, "i"),
                (r"M\69 X", "s", Case::ExplicitSensitive, "s"),
            ] {
                let source = format!("[Data{symbol}{gap}={operand} {modifier}]");
                let literal = format!("[Data{symbol}=\"MiX\" {output_modifier}]");
                match retained(front, &source, &context) {
                    Ok(selector) => {
                        check(
                            &selector,
                            constructor("MiX".into()),
                            case,
                            &Namespace::ExplicitNone,
                            &Prefix::Unqualified,
                            &literal,
                        );
                        assert_eq!(retained(front, &literal, &context).unwrap(), selector);
                    }
                    Err(failure) => failures.push(failure),
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "valid comment-separated matchers rejected: {failures:?}"
    );
}

#[test]
fn singular_front_admits_all_comment_separated_matchers() {
    comments(Front::Selector);
}

#[test]
fn ordinary_list_front_admits_all_comment_separated_matchers() {
    comments(Front::List);
}

#[test]
fn relative_list_front_admits_all_comment_separated_matchers() {
    comments(Front::Relative);
}

#[test]
fn stylesheet_front_admits_all_comment_separated_matchers() {
    comments(Front::Sheet);
}

#[test]
fn adjacent_matchers_keep_typed_operands_case_and_canonical_output_on_all_fronts() {
    let context = CssNamespaceContext::default();
    for front in FRONTS {
        for (symbol, constructor) in OPERATORS {
            for (operand, suffix, case, canonical_suffix) in [
                (r#""M\69 X""#, " I", Case::AsciiCaseInsensitive, " i"),
                (r"M\69 X", " s", Case::ExplicitSensitive, " s"),
                ("MiX", "", Case::DocumentDefault, ""),
            ] {
                let source = format!("[Data{symbol}={operand}{suffix}]");
                let literal = format!("[Data{symbol}=\"MiX\"{canonical_suffix}]");
                let selector = retained(front, &source, &context).unwrap();
                check(
                    &selector,
                    constructor("MiX".into()),
                    case,
                    &Namespace::ExplicitNone,
                    &Prefix::Unqualified,
                    &literal,
                );
                assert_eq!(retained(front, &literal, &context).unwrap(), selector);
            }
        }
    }
}

fn reject(front: Front, attribute: &str) {
    let context = CssNamespaceContext::default();
    match front {
        Front::Selector => {
            let report = CssSelector::parse(&CssParser::default(), attribute, &context);
            assert_whole_input_rejected(attribute, report);
        }
        Front::List => {
            let source = format!(".Lead, {attribute}, .Tail");
            let report = CssParsedStyleSelectors::parse(
                &CssParser::default(),
                &source,
                &context,
                CssStyleSelectorContext::Ordinary,
            );
            assert_whole_input_rejected(&source, report);
        }
        Front::Relative => {
            let source = format!("> .Lead, + {attribute}, ~ .Tail");
            let report = CssRelativeSelectorList::parse(&CssParser::default(), &source, &context);
            assert_whole_input_rejected(&source, report);
        }
        Front::Sheet => {
            let source = format!(
                ".Before {{}} .Lead, {attribute} {{ color:red; .Never {{ color:blue }} }} .After {{}}"
            );
            let report = CssParser::default().parse_sheet(&source);
            let [CssRule::Style(before), CssRule::Style(after)] = report.syntax().rules() else {
                panic!("only surrounding complete rules survive: {attribute:?}");
            };
            assert_eq!(
                before.selectors().selectors()[0].selector(),
                &CssSelector::Class("Before".into())
            );
            assert_eq!(
                after.selectors().selectors()[0].selector(),
                &CssSelector::Class("After".into())
            );
            assert!(before.rules().is_empty() && after.rules().is_empty());
            let [diagnostic] = report.diagnostics() else {
                panic!("one complete rule rejection");
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropQualifiedRule);
            assert_strict_validation_rejects(report);
        }
    }
}

#[test]
fn all_five_css_whitespace_characters_split_and_reject_matchers() {
    for (symbol, _) in OPERATORS {
        for whitespace in [" ", "\t", "\n", "\r", "\u{c}"] {
            for gap in [whitespace.to_owned(), format!("/**/{whitespace}/**/")] {
                let source = format!("[Data{symbol}{gap}=MiX]");
                for front in FRONTS {
                    reject(front, &source);
                }
            }
        }
    }
}

#[test]
fn malformed_operator_pairs_reject_complete_lists_and_containing_rule_contents() {
    for pair in [
        "~==", "|==", "^==", "$==", "*==", "!=", "=~", "||=", "^$=", "$*=", "~/**/~=",
    ] {
        let source = format!("[Data{pair}MiX]");
        for front in FRONTS {
            reject(front, &source);
        }
    }
}

#[test]
fn namespace_and_case_controls_remain_valid_with_adjacent_matchers() {
    namespace_cases("");
}

#[test]
fn namespace_and_case_retention_also_admits_comment_separated_matchers() {
    namespace_cases("/*one*//*two*/");
}

fn namespace_cases(gap: &str) {
    let svg = CssNamespacePrefix::try_new("Svg").unwrap();
    let context = CssNamespaceContext::from_bindings([
        (None, CssNamespaceName::new("urn:default")),
        (Some(svg.clone()), CssNamespaceName::new("urn:svg")),
    ]);
    let mut failures = Vec::new();
    for (name, namespace, prefix) in [
        ("Data", Namespace::ExplicitNone, Prefix::Unqualified),
        (
            "Svg|Data",
            Namespace::Named(svg.clone()),
            Prefix::Named(svg),
        ),
        ("*|Data", Namespace::Any, Prefix::Any),
        ("|Data", Namespace::ExplicitNone, Prefix::ExplicitNone),
    ] {
        let source = format!("[{name}|{gap}=MiX I]");
        let literal = format!("[{name}|=\"MiX\" i]");
        for front in [Front::Selector, Front::List, Front::Relative] {
            match retained(front, &source, &context) {
                Ok(selector) => {
                    check(
                        &selector,
                        Matcher::DashMatch("MiX".into()),
                        Case::AsciiCaseInsensitive,
                        &namespace,
                        &prefix,
                        &literal,
                    );
                    assert_eq!(retained(front, &literal, &context).unwrap(), selector);
                }
                Err(failure) => failures.push(failure),
            }
        }
        let sheet_source =
            format!("@namespace \"urn:default\"; @namespace Svg \"urn:svg\"; {source} {{}}");
        let report = CssParser::default().parse_sheet(&sheet_source);
        if !report.is_clean() {
            failures.push(format!("Sheet namespace: {source:?}"));
            continue;
        }
        let sheet = report.into_validation_result().unwrap();
        let [
            CssRule::Namespace(_),
            CssRule::Namespace(_),
            CssRule::Style(style),
        ] = sheet.rules()
        else {
            panic!("namespace declarations and complete style rule");
        };
        let selector = style.selectors().selectors()[0].selector();
        check(
            selector,
            Matcher::DashMatch("MiX".into()),
            Case::AsciiCaseInsensitive,
            &namespace,
            &prefix,
            &literal,
        );
        let canonical = CssParser::default().parse_sheet(format!(
            "@namespace \"urn:default\"; @namespace Svg \"urn:svg\"; {literal} {{}}"
        ));
        assert!(canonical.is_clean());
        let CssRule::Style(reparsed) = &canonical.syntax().rules()[2] else {
            panic!("canonical style rule");
        };
        assert_eq!(reparsed.selectors().selectors()[0].selector(), selector);
    }
    assert!(
        failures.is_empty(),
        "valid namespace matcher syntax rejected: {failures:?}"
    );
    for front in [Front::Selector, Front::List, Front::Relative] {
        assert!(retained(front, "[svg|Data|=MiX]", &context).is_err());
    }
}

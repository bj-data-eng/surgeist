//! Variables 1 §3 retains everything after the first var fallback comma.
//! It permits var only in property-value contexts and defers whole valid values.
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
//! The public pending transition preserves original occurrence, priority and origins.

use crate::css::support::value_assertions::assert_first_diagnostic;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValueRef, CssComponentValues, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::declaration::{CssCustomPropertyName, CssDeclaration, CssImportance};
use surgeist_css::diagnostic::CssErrorCode;
use surgeist_css::property::{
    CssContributions, CssExpansion, CssKnownProperty, CssLonghandValueRef, CssPropertyNameRef,
    CssPropertyTarget,
};
use surgeist_css::rule::CssRule;
use surgeist_css::selector::{CssNamespaceContext, CssSelector};

#[test]
fn later_var_commas_are_retained_fallback_tokens_in_known_and_custom_values() {
    for (text, arguments, comma_count) in [
        ("var(--Theme, red,/**/ BLUE,)", "--Theme, red,/**/ BLUE,", 3),
        (
            "var(--Theme, nested(red, blue), var(--other,))",
            "--Theme, nested(red, blue), var(--other,)",
            2,
        ),
    ] {
        let source = format!("display:{text}!important");
        let report = CssParser::default().parse_block_contents(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let parsed = &report.syntax()[0];
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let original = components.clone();
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        for declaration in [parsed, &checked] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            assert_eq!(
                declaration
                    .known()
                    .unwrap()
                    .substitution_dependent()
                    .unwrap()
                    .as_css(),
                text,
            );
            let [function] = declaration.value_components().items() else {
                panic!("one complete var function")
            };
            let CssComponentValueRef::Function(function) = function.view() else {
                panic!("retained var function")
            };
            assert_eq!(function.name(), "var");
            assert_eq!(
                function
                    .values()
                    .serialize_tokens(Default::default())
                    .unwrap()
                    .as_css(),
                arguments
            );
            assert_eq!(
                function
                    .values()
                    .items()
                    .iter()
                    .filter(|value| matches!(
                        value.view(),
                        CssComponentValueRef::Token(CssValueTokenRef::Comma)
                    ))
                    .count(),
                comma_count,
            );
            assert!(matches!(declaration.expand(), Ok(CssExpansion::Pending(_))));
        }
        let name = CssCustomPropertyName::try_new("--kept").unwrap();
        let custom = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Custom(&name)),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(
            custom.custom().unwrap().value().value().unwrap().as_css(),
            text
        );
        assert_eq!(custom.value_components(), &original);
        assert!(matches!(
            custom.expand(),
            Ok(CssExpansion::Contributions(_))
        ));
        assert_eq!(components, original);
    }
}

#[test]
fn valid_var_calls_cannot_form_property_names_or_selectors() {
    for source in ["var(--side):20px", "var(--side, margin-top):20px"] {
        let report = CssDeclaration::parse(&CssParser::default(), source);
        assert_first_diagnostic(
            report.diagnostics(),
            source,
            CssErrorCode::UnexpectedToken,
            "var(",
        );
        assert!(
            report.syntax().is_none(),
            "variable property name: {source}"
        );
    }
    let context = CssNamespaceContext::default();
    for selector in ["var(--selector)", ".item var(--selector, .chosen)"] {
        let report = CssSelector::parse(&CssParser::default(), selector, &context);
        assert_first_diagnostic(
            report.diagnostics(),
            selector,
            CssErrorCode::InvalidSelector,
            "var(",
        );
        assert!(report.syntax().is_none(), "variable selector: {selector}");
        let rule = CssRule::parse(
            &CssParser::default(),
            format!("{selector}{{color:red}}"),
            &context,
        );
        assert_first_diagnostic(
            rule.diagnostics(),
            &format!("{selector}{{color:red}}"),
            CssErrorCode::InvalidSelector,
            "var(",
        );
        assert!(
            rule.syntax().is_none(),
            "variable rule selector: {selector}"
        );
    }
    let control = CssDeclaration::parse(&CssParser::default(), "display:var(--mode)");
    assert!(control.is_clean());
    assert!(
        control
            .syntax()
            .as_ref()
            .unwrap()
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    assert!(CssSelector::parse(&CssParser::default(), ".item", &context).is_clean());
}

#[test]
fn strict_pending_reentry_retains_original_priority_source_after_failed_replacements() {
    const SOURCE: &str = "/*😀*/display:bogus VAR(--layout, red, blue)!/**/ImPoRtAnT";
    const VALUE: &str = "bogus VAR(--layout, red, blue)";
    let report = CssParser::default().parse_block_contents(SOURCE);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one complete pending declaration")
    };
    assert_eq!(source.importance(), CssImportance::Important);
    assert_eq!(
        source
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        VALUE
    );
    assert_eq!(
        source
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        VALUE
    );
    let origin = source.parsed_value().unwrap();
    assert_eq!(origin.source().as_str(), SOURCE);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "/*😀*/display:".len()
    );
    assert_eq!(
        origin.span().end().byte_offset().value(),
        SOURCE.find('!').unwrap()
    );
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("whole value awaits replacement")
    };
    for invalid in ["block!important", "block;inline"] {
        let error = pending
            .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
            .unwrap_err();
        assert!(error.replacement_error().is_some());
        assert!(pending.source().same_occurrence(source));
        assert_eq!(
            pending.source().parsed_value().unwrap().source().as_str(),
            SOURCE
        );
        assert_eq!(pending.source().importance(), CssImportance::Important);
    }
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(x)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "block").unwrap();
    let original = replacement.clone();
    for _ in 0..2 {
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("one completed display contribution")
        };
        let [value] = values.items() else {
            panic!("one terminal display")
        };
        assert_eq!(value.property(), CssKnownProperty::Display);
        let CssLonghandValueRef::Display(display) = value.ordinary_value().unwrap().view() else {
            panic!("actual replacement display")
        };
        assert_eq!(
            display.serialize_specified(Default::default()).unwrap(),
            "block"
        );
        assert!(value.source().same_occurrence(source));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert_eq!(
            value.source().parsed_value().unwrap().source().as_str(),
            SOURCE
        );
        assert_eq!(
            value
                .source()
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            VALUE
        );
        assert_eq!(value.replacement_components(), Some(&replacement));
        let CssValueOrigin::Parsed(retained) =
            value.replacement_components().unwrap().items()[0].origin()
        else {
            panic!("original caller replacement token")
        };
        assert_eq!(retained.source().as_str(), "block");
        assert_eq!(retained.span().start().byte_offset().value(), 0);
        assert_eq!(retained.span().end().byte_offset().value(), 5);
    }
    assert_eq!(replacement, original);
}

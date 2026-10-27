//! CSS Variables 1 §2.1 (2022-06-16): custom properties admit declaration-value
//! tokens; a CSS-wide keyword is special only when it is the whole value.
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#syntax

use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{
    CssCustomPropertyName, CssDeclaration, CssGlobalKeyword, CssImportance,
};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::property::{CssKnownProperty, CssPropertyNameRef, CssPropertyTarget};
use surgeist_css::rule::CssRule;

fn custom_tokens(declaration: &CssDeclaration) -> &str {
    let value = declaration.custom().expect("custom declaration").value();
    assert_eq!(value.global(), None);
    value.value().expect("ordinary custom tokens").as_css()
}

fn style_custom(source: &str) -> CssDeclaration {
    let report = CssParser::default().parse_block_contents(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one custom declaration: {source}")
    };
    declaration.clone()
}

#[test]
fn every_global_prefix_with_a_valid_tail_is_ordinary_custom_data_in_both_entrypoints() {
    for keyword in ["inherit", "initial", "unset", "revert", "revert-layer"] {
        let value = format!("{keyword} 1px");
        let attribute = format!("color:red;--x:{value}!important;height:2px");
        let report = CssParser::default().parse_block_contents(&attribute);
        assert!(report.is_clean(), "{attribute}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 3);
        assert_eq!(
            report.syntax()[0].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Color)
        );
        assert_eq!(
            report.syntax()[2].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Height)
        );
        let custom = &report.syntax()[1];
        assert_eq!(custom_tokens(custom), value);
        assert_eq!(custom.importance(), CssImportance::Important);
        assert_eq!(
            custom.position().unwrap().byte_offset().value(),
            attribute.find("--x").unwrap()
        );
        let name = custom.parsed_name().expect("authored name origin");
        let value_origin = custom.parsed_value().expect("authored value origin");
        assert!(name.source().same_snapshot(value_origin.source()));
        assert!(matches!(
            custom.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        assert!(
            CssParser::default()
                .parse_block_contents(&attribute)
                .into_validation_result()
                .is_ok(),
            "{attribute}"
        );

        let sheet = format!(".x{{{attribute}}}");
        let report = CssParser::default().parse_sheet(&sheet);
        assert!(report.is_clean(), "{sheet}: {:?}", report.diagnostics());
        let [CssRule::Style(rule)] = report.syntax().rules() else {
            panic!("one style rule: {sheet}")
        };
        assert_eq!(rule.declarations().len(), 3);
        assert_eq!(
            rule.declarations()[0].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Color)
        );
        assert_eq!(
            rule.declarations()[2].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Height)
        );
        let custom = &rule.declarations()[1];
        assert_eq!(custom_tokens(custom), value);
        assert_eq!(custom.importance(), CssImportance::Important);
        assert_eq!(
            custom.position().unwrap().byte_offset().value(),
            sheet.find("--x").unwrap()
        );
        assert!(
            CssParser::default()
                .parse_sheet(&sheet)
                .into_validation_result()
                .is_ok(),
            "{sheet}"
        );
    }
}

#[test]
fn checked_construction_distinguishes_global_only_from_prefixed_custom_tokens() {
    let name = CssCustomPropertyName::try_new("--x").unwrap();
    for (spelling, keyword) in [
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let standalone = style_custom(&format!("--x:{spelling}"));
        assert_eq!(standalone.custom().unwrap().value().global(), Some(keyword));
        let spaced = style_custom(&format!("--x: /*leading*/ {spelling} /*trailing*/"));
        assert_eq!(spaced.custom().unwrap().value().global(), Some(keyword));
        let constructed = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Custom(&name)),
            CssComponentValues::parse(&CssParser::default(), spelling).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(
            constructed.custom().unwrap().value().global(),
            Some(keyword)
        );

        let prefixed = format!("{spelling} 1px");
        let constructed = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Custom(&name)),
            CssComponentValues::parse(&CssParser::default(), &prefixed).unwrap(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(custom_tokens(&constructed), prefixed);
        assert_eq!(constructed.importance(), CssImportance::Important);
        assert!(constructed.parsed_name().is_none());
    }
}

#[test]
fn global_looking_prefixes_keep_case_escape_and_valid_var_tail() {
    for (authored, expected) in [
        ("INHERIT 1px", "INHERIT 1px"),
        ("\\69nherit 1px", "\\69nherit 1px"),
        ("inherit var(--gap)", "inherit var(--gap)"),
    ] {
        let source = format!("--x:{authored}");
        let declaration = style_custom(&source);
        assert_eq!(custom_tokens(&declaration), expected);
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_ok()
        );
    }
    assert_eq!(
        style_custom("--x:\\69nherit")
            .custom()
            .unwrap()
            .value()
            .global(),
        Some(CssGlobalKeyword::Inherit)
    );
}

#[test]
fn malformed_custom_tails_and_known_property_globals_drop_atomically() {
    for invalid in [
        "--x:inherit var()",
        "--x:inherit var(foo)",
        "--x:inherit 1px !oops",
        "width:inherit 1px",
    ] {
        let source = format!("color:red;{invalid};height:2px");
        let report = CssParser::default().parse_block_contents(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Color)
        );
        assert_eq!(
            report.syntax()[1].property_name(),
            CssPropertyNameRef::Known(CssKnownProperty::Height)
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid declaration: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
    }
}

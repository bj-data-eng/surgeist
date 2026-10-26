//! Additional authored `attr()` boundary cases from pinned CSS Values 5 §§3.1.1, 3.3, 7.7.

use crate::css::support::value_assertions::property_rejection;
use surgeist_css::*;

fn pending_display(text: &str) -> CssDeclaration {
    let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
    let declaration = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &components);
    assert_eq!(
        declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        text
    );
    declaration
}

#[test]
fn decoded_type_names_and_comments_keep_valid_syntax_distinct_from_whitespace() {
    for value in [
        "attr(foo <LENGTH>)",
        "attr(foo <TrAnSfOrM-LiSt>)",
        r"attr(foo <l\65 ngth>)",
        "attr(foo </**/length/**/>)",
        "attr(foo <length>/**/+)",
        "attr(foo \"<LENGTH>\")",
        "attr(foo/**/bar)",
    ] {
        let declaration = pending_display(value);
        assert!(matches!(
            declaration.expand().unwrap(),
            CssExpansion::Pending(_)
        ));
    }
    for value in [
        "attr(foo < length>)",
        "attr(foo <length >)",
        "attr(foo <length> +)",
        "attr(foo <transform-list>/**/+)",
        "attr(foo </**/length/**/ >)",
    ] {
        property_rejection(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal,
            ),
            CssKnownProperty::Display,
        );
    }
}

#[test]
fn valid_attr_family_independently_qualifies_invalid_var_or_env_family() {
    for value in [
        "attr(foo) var(x)",
        "env() attr(foo)",
        "attr(foo <length>) env(foo -1)",
    ] {
        pending_display(value);
    }
    for value in ["attr(foo) attr()", "attr(foo) attr(foo,)"] {
        property_rejection(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal,
            ),
            CssKnownProperty::Display,
        );
    }
}

#[test]
fn strict_fallback_separators_apply_to_outer_stream_not_nested_functions() {
    for value in ["attr(foo,rgb(1,2,3))", "attr(foo,f({a,b}))"] {
        pending_display(value);
    }
    for value in ["attr(foo,a,b)", "attr(foo,a {})"] {
        property_rejection(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal,
            ),
            CssKnownProperty::Display,
        );
    }
}

#[test]
fn attribute_functions_keep_property_and_query_contexts_separate() {
    let valid_style = CssCondition::<CssContainerLeaf>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "style(--x:attr(foo))").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        valid_style.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));
    let raw_style = CssCondition::<CssContainerLeaf>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "style(--x:attr())").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        raw_style.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::Style(_))
    ));

    let width = CssCondition::<CssContainerLeaf>::try_from_components(
        CssComponentValues::parse(&CssParser::default(), "(width:attr(foo))").unwrap(),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        width.kind(),
        CssConditionKind::Leaf(CssContainerLeaf::GeneralEnclosed(_))
    ));
    let descriptor = CssAuthoredFontFaceDescriptorValue::parse(
        &CssParser::default(),
        "attr(foo)",
        CssFontFaceDescriptorKind::FontStyle,
    );
    assert!(descriptor.syntax().is_none());
}

#[test]
fn nested_decoded_syntax_reports_the_original_outer_string_origin() {
    let decoded = format!("{}x{}", "(".repeat(257), ")".repeat(257));
    assert!(
        CssComponentValues::parse(&CssParser::default(), &decoded)
            .unwrap_err()
            .is_nesting_limit()
    );
    let source = format!("display:attr(foo \"{decoded}\");color:red");
    let report = CssParser::default().parse_block_contents(&source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0]
            .error()
            .position()
            .byte_offset()
            .value(),
        source.find('"').unwrap()
    );
    assert_eq!(report.syntax().len(), 1);

    let outer_string = CssComponentValues::parse(&CssParser::default(), format!("\"{decoded}\""))
        .unwrap()
        .items()[0]
        .clone();
    let outer_origin = outer_string.origin().clone();
    let arguments = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_ident("foo").unwrap(),
            CssComponentValue::try_token(" ").unwrap(),
            outer_string,
        ],
        Default::default(),
    )
    .unwrap();
    let function = CssComponentValue::try_function("attr", arguments).unwrap();
    let value = CssComponentValues::try_new(vec![function], Default::default()).unwrap();
    assert_eq!(
        value.serialize_tokens(Default::default()).unwrap().as_css(),
        format!("attr(foo \"{decoded}\")")
    );
    let error = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Display)),
        value,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert!(
        error
            .component_error()
            .is_some_and(|error| error.is_nesting_limit()),
        "{error:?}"
    );
    assert_eq!(error.origin(), CssSerializedOrigin::Token(outer_origin));
}

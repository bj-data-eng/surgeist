#![forbid(unsafe_code)]
//! Additional authored `attr()` boundary cases from pinned CSS Values 5 §§3.1.1, 3.3, 7.7.

use surgeist_css::*;

fn pending_display(text: &str) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
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
            expand_declaration(&declaration).unwrap(),
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
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "{value}"
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
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn strict_fallback_separators_apply_to_outer_stream_not_nested_functions() {
    for value in ["attr(foo,rgb(1,2,3))", "attr(foo,f({a,b}))"] {
        pending_display(value);
    }
    for value in ["attr(foo,a,b)", "attr(foo,a {})"] {
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Display),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn attribute_functions_keep_property_and_query_contexts_separate() {
    let valid_style = CssContainerCondition::try_from_components(
        parse_component_values("style(--x:attr(foo))").unwrap(),
    )
    .unwrap();
    assert!(matches!(
        valid_style.kind(),
        CssContainerConditionKind::Style(_)
    ));
    let raw_style = CssContainerCondition::try_from_components(
        parse_component_values("style(--x:attr())").unwrap(),
    )
    .unwrap();
    assert!(matches!(
        raw_style.kind(),
        CssContainerConditionKind::Style(_)
    ));

    let width = CssContainerCondition::try_from_components(
        parse_component_values("(width:attr(foo))").unwrap(),
    )
    .unwrap();
    assert!(matches!(
        width.kind(),
        CssContainerConditionKind::GeneralEnclosed(_)
    ));
    let descriptor =
        parse_font_face_descriptor_value("attr(foo)", CssFontFaceDescriptorKind::FontStyle);
    assert!(descriptor.syntax().is_none());
}

#[test]
fn nested_decoded_syntax_reports_the_original_outer_string_origin() {
    let decoded = format!("{}x{}", "(".repeat(257), ")".repeat(257));
    assert_eq!(
        parse_component_values(&decoded).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    let source = format!("display:attr(foo \"{decoded}\");color:red");
    let report = parse_style_attribute(&source);
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

    let outer_string = parse_component_values(&format!("\"{decoded}\""))
        .unwrap()
        .items()[0]
        .clone();
    let outer_origin = outer_string.origin().clone();
    let arguments = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("foo").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
        outer_string,
    ])
    .unwrap();
    let function = CssComponentValue::try_function("attr", arguments).unwrap();
    let value = CssComponentValues::try_new(vec![function]).unwrap();
    assert_eq!(
        value.serialize().unwrap().as_css(),
        format!("attr(foo \"{decoded}\")")
    );
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        value,
        CssImportance::Normal,
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
    );
    assert_eq!(error.origin(), &CssSerializedOrigin::Token(outer_origin));
}

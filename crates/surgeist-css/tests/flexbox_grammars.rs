use surgeist_css::{
    CssErrorCode, CssFlexDirection, CssFlexWrap, CssGlobalKeyword, CssKnownDeclaredValueRef,
    CssKnownProperty, CssKnownPropertyValueRef, CssRecoveryAction, ErrorKind,
    parse_style_attribute,
};

#[test]
fn c14_flex_flow_retain_typed_structure() {
    let report = parse_style_attribute(concat!(
        "flex-flow: wrap-reverse column; ",
        "flex-flow: row-reverse wrap; ",
        "flex-flow: column; ",
        "flex-flow: nowrap",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let expected = [
        (CssFlexDirection::Column, CssFlexWrap::WrapReverse),
        (CssFlexDirection::RowReverse, CssFlexWrap::Wrap),
        (CssFlexDirection::Column, CssFlexWrap::NoWrap),
        (CssFlexDirection::Row, CssFlexWrap::NoWrap),
    ];
    assert_eq!(report.syntax().len(), expected.len());
    for (declaration, (direction, wrap)) in report.syntax().iter().zip(expected) {
        assert_eq!(
            declaration.known().unwrap().property(),
            CssKnownProperty::FlexFlow
        );
        let CssKnownPropertyValueRef::FlexFlow(value) = declaration
            .known()
            .unwrap()
            .property_value()
            .expect("ordinary flex-flow")
        else {
            panic!("expected flex-flow wrapper");
        };
        assert_eq!(value.flow().direction(), direction);
        assert_eq!(value.flow().wrap(), wrap);
    }
}

#[test]
fn flex_flow_preserves_authored_text_and_symbolic_branches() {
    let report = parse_style_attribute(concat!(
        "flex-flow:  ROW   wrap-reverse ; ",
        "flex-flow: inherit; ",
        "flex-flow: var(--flow, column wrap)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::FlexFlow(value)) =
        report.syntax()[0].known().unwrap().declared_value()
    else {
        panic!("expected ordinary flex-flow");
    };
    assert_eq!(value.as_css(), "ROW   wrap-reverse");
    assert_eq!(value.flow().direction(), CssFlexDirection::Row);
    assert_eq!(value.flow().wrap(), CssFlexWrap::WrapReverse);
    assert!(matches!(
        report.syntax()[1].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
    ));
    assert!(matches!(
        report.syntax()[2].known().unwrap().declared_value(),
        CssKnownDeclaredValueRef::SubstitutionDependent(value)
            if value.as_css() == "var(--flow, column wrap)"
    ));
}

#[test]
fn flex_flow_mutations_drop_exact_declaration_and_retain_siblings() {
    for (invalid, responsible_text) in [
        ("row column", "column"),
        ("wrap nowrap", "nowrap"),
        ("row wrap nowrap", "nowrap"),
        ("row, wrap", ","),
        ("row / wrap", "/"),
        ("row 1", "1"),
        ("flex-start", "flex-start"),
    ] {
        let declaration = format!("flex-flow: {invalid};");
        let source = format!("color: red; {declaration} width: 1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Width,
            "{source}",
        );

        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected exactly one diagnostic");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let declaration_start = source.find(&declaration).unwrap();
        let declaration_end = declaration_start + declaration.len();
        let value_start = declaration_start + "flex-flow: ".len();
        let responsible = value_start + invalid.find(responsible_text).unwrap();
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            declaration_start,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            declaration_end,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{source}",
        );
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            source[..responsible].encode_utf16().count(),
            "{source}",
        );
        assert!(matches!(
            diagnostic.error().kind(),
            ErrorKind::InvalidPropertyValue(detail)
                if detail.property() == CssKnownProperty::FlexFlow
        ));

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects invalid flex-flow")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

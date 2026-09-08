use surgeist_css::{
    CssColumnCount, CssColumnFill, CssColumnSpan, CssColumnWidth, CssErrorCode, CssGlobalKeyword,
    CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssLengthUnit,
    CssLineStyle, CssLineWidth, CssPositiveIntegerValue, CssRecoveryAction, ErrorKind,
    parse_style_attribute,
};

#[test]
fn c14_multicolumn_properties_retain_typed_structure() {
    let report = parse_style_attribute(concat!(
        "column-count: 3; ",
        "column-fill: balance-all; ",
        "column-rule: thick dashed rebeccapurple; ",
        "column-rule-color: currentcolor; ",
        "column-rule-style: double; ",
        "column-rule-width: 2px; ",
        "column-span: all; ",
        "column-width: 12em; ",
        "columns: 4 10rem",
    ));

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 9);

    let CssKnownPropertyValueRef::ColumnCount(count) = ordinary(&report.syntax()[0]) else {
        panic!("expected column-count");
    };
    assert!(matches!(
        count.count(),
        CssColumnCount::Count(CssPositiveIntegerValue::Literal(value)) if value.value() == 3
    ));

    let CssKnownPropertyValueRef::ColumnFill(fill) = ordinary(&report.syntax()[1]) else {
        panic!("expected column-fill");
    };
    assert_eq!(*fill.fill(), CssColumnFill::BalanceAll);

    let CssKnownPropertyValueRef::ColumnRule(rule) = ordinary(&report.syntax()[2]) else {
        panic!("expected column-rule");
    };
    assert!(matches!(rule.rule().width(), Some(CssLineWidth::Thick)));
    assert_eq!(rule.rule().style(), Some(CssLineStyle::Dashed));
    assert_eq!(
        rule.rule()
            .current_color()
            .and_then(|color| color.named())
            .map(|color| color.name()),
        Some("rebeccapurple"),
    );

    let CssKnownPropertyValueRef::ColumnRuleColor(color) = ordinary(&report.syntax()[3]) else {
        panic!("expected column-rule-color");
    };
    assert!(color.current().is_current_color());

    let CssKnownPropertyValueRef::ColumnRuleStyle(style) = ordinary(&report.syntax()[4]) else {
        panic!("expected column-rule-style");
    };
    assert_eq!(*style.style(), CssLineStyle::Double);

    let CssKnownPropertyValueRef::ColumnRuleWidth(width) = ordinary(&report.syntax()[5]) else {
        panic!("expected column-rule-width");
    };
    assert!(matches!(
        width.width(),
        CssLineWidth::Length(value)
            if matches!(value.value(), CssLength::Px(value) if value.value() == 2.0)
    ));

    let CssKnownPropertyValueRef::ColumnSpan(span) = ordinary(&report.syntax()[6]) else {
        panic!("expected column-span");
    };
    assert_eq!(*span.span(), CssColumnSpan::All);

    let CssKnownPropertyValueRef::ColumnWidth(width) = ordinary(&report.syntax()[7]) else {
        panic!("expected column-width");
    };
    assert!(matches!(
        width.width(),
        CssColumnWidth::Length(value)
            if matches!(value.value(), CssLength::Dimension(value)
                if value.value() == 12.0 && value.unit() == CssLengthUnit::Em)
    ));

    let CssKnownPropertyValueRef::Columns(columns) = ordinary(&report.syntax()[8]) else {
        panic!("expected columns");
    };
    assert!(matches!(
        columns.columns().count(),
        CssColumnCount::Count(CssPositiveIntegerValue::Literal(value)) if value.value() == 4
    ));
    assert!(matches!(
        columns.columns().width(),
        CssColumnWidth::Length(value)
            if matches!(value.value(), CssLength::Dimension(value)
                if value.value() == 10.0 && value.unit() == CssLengthUnit::Rem)
    ));
}

#[test]
fn multicolumn_shorthands_accept_permutations_and_preserve_omitted_components() {
    let report = parse_style_attribute(concat!(
        "columns: 12em; columns: 2; columns: auto 12em; ",
        "columns: 2 auto; columns: auto auto; columns: 12em 2; ",
        "column-rule: red solid 2px; column-rule: dashed; column-rule: thin blue",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 9);

    for index in [0, 2] {
        let CssKnownPropertyValueRef::Columns(value) = ordinary(&report.syntax()[index]) else {
            panic!("expected columns");
        };
        assert!(matches!(value.columns().count(), CssColumnCount::Auto));
    }
    for index in [1, 3] {
        let CssKnownPropertyValueRef::Columns(value) = ordinary(&report.syntax()[index]) else {
            panic!("expected columns");
        };
        assert!(matches!(value.columns().width(), CssColumnWidth::Auto));
    }
    for index in [4, 5] {
        let CssKnownPropertyValueRef::Columns(value) = ordinary(&report.syntax()[index]) else {
            panic!("expected columns");
        };
        assert_eq!(
            value.as_css(),
            if index == 4 { "auto auto" } else { "12em 2" }
        );
    }

    let CssKnownPropertyValueRef::ColumnRule(complete) = ordinary(&report.syntax()[6]) else {
        panic!("expected column-rule");
    };
    assert!(complete.rule().width().is_some());
    assert!(complete.rule().style().is_some());
    assert!(complete.rule().current_color().is_some());

    let CssKnownPropertyValueRef::ColumnRule(style_only) = ordinary(&report.syntax()[7]) else {
        panic!("expected column-rule");
    };
    assert!(style_only.rule().width().is_none());
    assert_eq!(style_only.rule().style(), Some(CssLineStyle::Dashed));
    assert!(style_only.rule().current_color().is_none());

    let CssKnownPropertyValueRef::ColumnRule(width_color) = ordinary(&report.syntax()[8]) else {
        panic!("expected column-rule");
    };
    assert!(matches!(
        width_color.rule().width(),
        Some(CssLineWidth::Thin)
    ));
    assert!(width_color.rule().style().is_none());
    assert!(width_color.rule().current_color().is_some());
}

#[test]
fn multicolumn_calculations_remain_symbolic_and_checked_constructors_reject_literals() {
    let report = parse_style_attribute(concat!(
        "column-count: calc(1 + 2); ",
        "column-width: calc(10px - 20px); ",
        "column-rule-width: calc(1px * 2)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());

    let CssKnownPropertyValueRef::ColumnCount(count) = ordinary(&report.syntax()[0]) else {
        panic!("expected column-count");
    };
    assert!(matches!(
        count.count(),
        CssColumnCount::Count(CssPositiveIntegerValue::Calculation(_))
    ));
    let CssKnownPropertyValueRef::ColumnWidth(width) = ordinary(&report.syntax()[1]) else {
        panic!("expected column-width");
    };
    assert!(matches!(
        width.width(),
        CssColumnWidth::Length(value) if matches!(value.value(), CssLength::Calc(_))
    ));
    let CssKnownPropertyValueRef::ColumnRuleWidth(width) = ordinary(&report.syntax()[2]) else {
        panic!("expected column-rule-width");
    };
    assert!(matches!(
        width.width(),
        CssLineWidth::Length(value) if matches!(value.value(), CssLength::Calc(_))
    ));

    assert!(surgeist_css::CssPositiveInteger::try_new(0).is_none());
    assert!(surgeist_css::CssPositiveInteger::try_new(-1).is_none());
    assert!(
        surgeist_css::CssNonNegativeLength::try_new(
            CssLength::try_dimension(-1.0, CssLengthUnit::Px).unwrap(),
        )
        .is_none()
    );
    assert!(surgeist_css::CssNonNegativeLength::try_new(CssLength::Auto).is_none());
}

#[test]
fn every_multicolumn_property_keeps_globals_and_substitutions_distinct() {
    for (name, property) in [
        ("column-count", CssKnownProperty::ColumnCount),
        ("column-fill", CssKnownProperty::ColumnFill),
        ("column-rule", CssKnownProperty::ColumnRule),
        ("column-rule-color", CssKnownProperty::ColumnRuleColor),
        ("column-rule-style", CssKnownProperty::ColumnRuleStyle),
        ("column-rule-width", CssKnownProperty::ColumnRuleWidth),
        ("column-span", CssKnownProperty::ColumnSpan),
        ("column-width", CssKnownProperty::ColumnWidth),
        ("columns", CssKnownProperty::Columns),
    ] {
        let source = format!("{name}: inherit; {name}: var(--multicol, initial)");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(report.syntax()[0].known().unwrap().property(), property);
        assert_eq!(report.syntax()[1].known().unwrap().property(), property);
        assert!(matches!(
            report.syntax()[0].known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(CssGlobalKeyword::Inherit)
        ));
        assert!(matches!(
            report.syntax()[1].known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(value)
                if value.as_css() == "var(--multicol, initial)"
        ));
    }
}

#[test]
fn multicolumn_mutations_drop_exact_declaration_and_retain_siblings() {
    for (name, invalid, responsible_text) in [
        ("column-count", "0", "0"),
        ("column-count", "1.5", "1.5"),
        ("column-fill", "balance auto", "auto"),
        ("column-rule", "thin thick", "thick"),
        ("column-rule", "solid dashed", "dashed"),
        ("column-rule", "red blue", "blue"),
        ("column-rule", "solid / red", "/"),
        ("column-rule-color", "12px", "12px"),
        ("column-rule-style", "auto", "auto"),
        ("column-rule-width", "-1px", "-1px"),
        ("column-rule-width", "10%", "10%"),
        ("column-span", "auto", "auto"),
        ("column-width", "-1px", "-1px"),
        ("column-width", "10%", "10%"),
        ("column-width", "thin", "thin"),
        ("columns", "2 3", "3"),
        ("columns", "10px 20px", "20px"),
        ("columns", "2 / 10px", "/"),
        ("columns", "2, 10px", ","),
    ] {
        let declaration = format!("{name}: {invalid};");
        let source = format!("--emoji: \"😀\"; color: red; {declaration} width: 1px");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 3, "{source}");
        assert_eq!(
            report.syntax()[1].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        assert_eq!(
            report.syntax()[2].known().unwrap().property(),
            CssKnownProperty::Width,
            "{source}",
        );

        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: expected exactly one diagnostic");
        };
        let expected_code = if name == "column-rule-color" {
            CssErrorCode::InvalidColorSyntax
        } else {
            CssErrorCode::InvalidPropertyValue
        };
        assert_eq!(diagnostic.error().code(), expected_code, "{source}");
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let declaration_start = source.find(&declaration).unwrap();
        let declaration_end = declaration_start + declaration.len();
        let value_start = declaration_start + name.len() + 2;
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
        assert!(
            matches!(
                diagnostic.error().kind(),
                ErrorKind::InvalidPropertyValue(detail)
                    if detail.property().canonical_name() == name
            ) || matches!(
                (name, diagnostic.error().kind()),
                ("column-rule-color", ErrorKind::InvalidColorSyntax(_))
            ),
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects invalid multicolumn syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

fn ordinary(declaration: &surgeist_css::CssDeclaration) -> CssKnownPropertyValueRef<'_> {
    declaration
        .known()
        .expect("known declaration")
        .property_value()
        .expect("ordinary value")
}

use surgeist_css::{
    CssBorderCollapse, CssBoxEdgeKeyword, CssCaptionSide, CssClip, CssClipEdge, CssEmptyCells,
    CssErrorCode, CssGlobalKeyword, CssKnownDeclaredValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLength, CssPageBreak, CssPageBreakInside, CssQuotes,
    CssRecoveryAction, CssTableLayout, CssWordSpacing, ErrorKind, parse_style_attribute,
};

#[test]
fn css2_residual_properties_retain_typed_values() {
    let report = parse_style_attribute(concat!(
        "border-collapse: collapse; ",
        "border-spacing: 2px 3px; ",
        "caption-side: bottom; ",
        "clip: rect(auto, 10px, 20px, -1px); ",
        "empty-cells: hide; ",
        "orphans: 3; ",
        "page-break-after: right; ",
        "page-break-before: always; ",
        "page-break-inside: avoid; ",
        "quotes: \"«\" \"»\" \"‹\" \"›\"; ",
        "table-layout: fixed; ",
        "widows: 4; ",
        "word-spacing: -0.25em; ",
        "color: red",
    ));

    if !report.is_clean() {
        assert_eq!(report.diagnostics().len(), 13);
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.error().code() == CssErrorCode::UnknownProperty)
        );
        assert_eq!(report.syntax().len(), 1);
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
        );
    }

    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 14);
    assert_eq!(
        report
            .syntax()
            .iter()
            .map(|declaration| declaration.known().unwrap().property().canonical_name())
            .collect::<Vec<_>>(),
        [
            "border-collapse",
            "border-spacing",
            "caption-side",
            "clip",
            "empty-cells",
            "orphans",
            "page-break-after",
            "page-break-before",
            "page-break-inside",
            "quotes",
            "table-layout",
            "widows",
            "word-spacing",
            "color",
        ],
    );

    let CssKnownPropertyValueRef::BorderCollapse(border_collapse) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected border-collapse");
    };
    assert_eq!(border_collapse.collapse(), &CssBorderCollapse::Collapse);
    assert_eq!(border_collapse.as_css(), "collapse");

    let CssKnownPropertyValueRef::BorderSpacing(border_spacing) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected border-spacing");
    };
    assert!(matches!(
        border_spacing.spacing().horizontal().value(),
        CssLength::Px(value) if value.value() == 2.0
    ));
    assert!(matches!(
        border_spacing.spacing().vertical().value(),
        CssLength::Px(value) if value.value() == 3.0
    ));

    let CssKnownPropertyValueRef::CaptionSide(caption_side) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected caption-side");
    };
    assert_eq!(caption_side.side(), &CssCaptionSide::Bottom);

    let CssKnownPropertyValueRef::Clip(clip) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected clip");
    };
    let CssClip::Rect(rect) = clip.clip() else {
        panic!("expected clipping rectangle");
    };
    assert!(matches!(rect.top(), CssClipEdge::Auto));
    assert!(matches!(
        rect.left(),
        CssClipEdge::Length(length)
            if matches!(length.value(), CssLength::Px(value) if value.value() == -1.0)
    ));

    let CssKnownPropertyValueRef::EmptyCells(empty_cells) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected empty-cells");
    };
    assert_eq!(empty_cells.cells(), &CssEmptyCells::Hide);

    for index in [5, 11] {
        let value = report.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap();
        match value {
            CssKnownPropertyValueRef::Orphans(value) => {
                assert_eq!(value.minimum().literal(), Some(3));
            }
            CssKnownPropertyValueRef::Widows(value) => {
                assert_eq!(value.minimum().literal(), Some(4));
            }
            _ => panic!("expected a page-line minimum"),
        }
    }

    for index in [6, 7] {
        let value = report.syntax()[index]
            .known()
            .unwrap()
            .property_value()
            .unwrap();
        match value {
            CssKnownPropertyValueRef::PageBreakAfter(value) => {
                assert_eq!(value.page_break(), &CssPageBreak::Right);
            }
            CssKnownPropertyValueRef::PageBreakBefore(value) => {
                assert_eq!(value.page_break(), &CssPageBreak::Always);
            }
            _ => panic!("expected an outside page-break value"),
        }
    }

    let CssKnownPropertyValueRef::PageBreakInside(page_break_inside) = report.syntax()[8]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected page-break-inside");
    };
    assert_eq!(page_break_inside.page_break(), &CssPageBreakInside::Avoid);

    let CssKnownPropertyValueRef::Quotes(quotes) = report.syntax()[9]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected quotes");
    };
    let CssQuotes::Pairs(pairs) = quotes.quotes() else {
        panic!("expected quotation pairs");
    };
    assert_eq!(pairs.pairs().len(), 2);
    assert_eq!(pairs.pairs()[0].open().as_str(), "«");
    assert_eq!(pairs.pairs()[1].close().as_str(), "›");

    let CssKnownPropertyValueRef::TableLayout(table_layout) = report.syntax()[10]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected table-layout");
    };
    assert_eq!(table_layout.layout(), &CssTableLayout::Fixed);

    let CssKnownPropertyValueRef::WordSpacing(word_spacing) = report.syntax()[12]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected word-spacing");
    };
    assert!(matches!(
        word_spacing.spacing(),
        CssWordSpacing::Length(length)
            if matches!(length.value(), CssLength::Dimension(value) if value.value() == -0.25)
    ));
}

#[test]
fn css2_residual_keyword_numeric_list_and_separator_domains_are_complete() {
    for (property, values) in [
        ("border-collapse", &["collapse", "separate"][..]),
        ("caption-side", &["top", "bottom"]),
        ("empty-cells", &["show", "hide"]),
        (
            "page-break-after",
            &["auto", "always", "avoid", "left", "right"],
        ),
        (
            "page-break-before",
            &["auto", "always", "avoid", "left", "right"],
        ),
        ("page-break-inside", &["auto", "avoid"]),
        ("table-layout", &["auto", "fixed"]),
    ] {
        for value in values {
            let source = format!("{property}: {value}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert_eq!(report.syntax().len(), 1, "{source}");
        }
    }

    for source in [
        "border-spacing: 0",
        "border-spacing: 1px 2em",
        "border-spacing: calc(1px + 2em)",
        "clip: auto",
        "clip: rect(auto, -1px, 2em, 0)",
        "clip: rect(auto -1px 2em 0)",
        "orphans: 1",
        "orphans: calc(2 + 1)",
        "widows: 2147483647",
        "widows: calc(4 - 1)",
        "quotes: none",
        "quotes: \"\" \"\"",
        "quotes: \"[\" \"]\" \"«\" \"»\"",
        "word-spacing: normal",
        "word-spacing: -2em",
        "word-spacing: calc(1em - 2px)",
    ] {
        let report = parse_style_attribute(source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert_eq!(report.syntax().len(), 1, "{source}");
    }
}

#[test]
fn css2_residual_globals_and_substitutions_remain_whole_property_branches() {
    const PROPERTIES: &[&str] = &[
        "border-collapse",
        "border-spacing",
        "caption-side",
        "clip",
        "empty-cells",
        "orphans",
        "page-break-after",
        "page-break-before",
        "page-break-inside",
        "quotes",
        "table-layout",
        "widows",
        "word-spacing",
    ];
    const GLOBALS: &[(&str, CssGlobalKeyword)] = &[
        ("inherit", CssGlobalKeyword::Inherit),
        ("initial", CssGlobalKeyword::Initial),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ];

    for property in PROPERTIES {
        for (keyword, expected) in GLOBALS {
            let source = format!("{property}: {keyword}");
            let report = parse_style_attribute(&source);
            assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
            assert!(matches!(
                report.syntax()[0].known().unwrap().declared_value(),
                CssKnownDeclaredValueRef::Global(actual) if actual == *expected
            ));
        }

        let source = format!("{property}: var(--residual, initial)");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        assert!(matches!(
            report.syntax()[0].known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(value)
                if value.as_css() == "var(--residual, initial)"
        ));
    }
}

#[test]
fn css2_residual_invalid_values_drop_exact_declaration_and_keep_sibling() {
    for (property, value) in [
        ("border-collapse", "collapse separate"),
        ("border-spacing", "-1px"),
        ("border-spacing", "1%"),
        ("border-spacing", "1px 2px 3px"),
        ("caption-side", "top bottom"),
        ("clip", "circle(1px)"),
        ("clip", "rect(1px, 2px 3px, 4px)"),
        ("clip", "rect(1px, 2px, 3px)"),
        ("clip", "rect(1%, 2px, 3px, 4px)"),
        ("empty-cells", "show hide"),
        ("orphans", "0"),
        ("orphans", "-1"),
        ("orphans", "1.5"),
        ("page-break-after", "always avoid"),
        ("page-break-before", "page"),
        ("page-break-inside", "always"),
        ("quotes", "\"open\""),
        ("quotes", "none \"open\" \"close\""),
        ("table-layout", "auto fixed"),
        ("widows", "0"),
        ("word-spacing", "10%"),
        ("word-spacing", "normal 1px"),
        ("word-spacing", "auto"),
    ] {
        let invalid = format!("{property}: {value}");
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0].known().unwrap().property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            invalid.len() + 1,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected property-specific error");
        };
        assert_eq!(detail.property().canonical_name(), property, "{source}");
        assert!(
            diagnostic.error().position().byte_offset().value() < invalid.len() + 1,
            "{source}: responsible position must be inside the dropped declaration",
        );

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects recovered residual syntax")
                .diagnostics(),
            report.diagnostics(),
            "{source}",
        );
    }
}

#[test]
fn css2_residual_eof_and_non_bmp_recovery_preserve_exact_coordinates() {
    let clean_eof = parse_style_attribute("table-layout: fixed");
    assert!(clean_eof.is_clean(), "{:?}", clean_eof.diagnostics());
    assert_eq!(clean_eof.syntax().len(), 1);

    let invalid_eof_source = "quotes: \"open\"";
    let invalid_eof = parse_style_attribute(invalid_eof_source);
    let [diagnostic] = invalid_eof.diagnostics() else {
        panic!("odd quote at EOF must recover exactly once");
    };
    assert!(invalid_eof.syntax().is_empty());
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        invalid_eof_source.len()
    );
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::Quotes
    ));

    let source = "--😀: 1; word-spacing: 10%; color: red";
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 2);
    let [diagnostic] = report.diagnostics() else {
        panic!("percentage word spacing must recover exactly once");
    };
    let responsible = source.find("10%").unwrap();
    let declaration_start = source.find("word-spacing").unwrap();
    let declaration_end = declaration_start + source[declaration_start..].find(';').unwrap() + 1;
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..responsible].encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        declaration_start
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        declaration_end
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert!(matches!(
        diagnostic.error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::WordSpacing
    ));
    assert_eq!(
        report.syntax()[1].known().unwrap().property(),
        CssKnownProperty::Color
    );
}

#[test]
fn box_edge_keyword_value_exposes_every_o_box3_branch_and_rejects_adjacent_grammar() {
    for (authored, expected) in [
        ("content-box", CssBoxEdgeKeyword::ContentBox),
        ("padding-box", CssBoxEdgeKeyword::PaddingBox),
        ("border-box", CssBoxEdgeKeyword::BorderBox),
        ("margin-box", CssBoxEdgeKeyword::MarginBox),
        ("fill-box", CssBoxEdgeKeyword::FillBox),
        ("stroke-box", CssBoxEdgeKeyword::StrokeBox),
        ("view-box", CssBoxEdgeKeyword::ViewBox),
    ] {
        let value = CssBoxEdgeKeyword::from_keyword(authored).expect("known box edge keyword");
        assert_eq!(value, expected);
        assert_eq!(value.as_css_str(), authored);
        assert_eq!(
            CssBoxEdgeKeyword::from_keyword(&authored.to_ascii_uppercase()),
            Some(expected)
        );
    }
    assert_eq!(
        CssBoxEdgeKeyword::from_keyword("content-box padding-box"),
        None
    );
    assert_eq!(CssBoxEdgeKeyword::from_keyword("content"), None);
    assert_eq!(CssBoxEdgeKeyword::from_keyword(""), None);
}

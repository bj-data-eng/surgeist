use surgeist_css::{
    CssBlendMode, CssBoxEdgeKeyword, CssErrorCode, CssKnownProperty, CssPropertyNameRef,
    CssRecoveryAction, CssRule, ErrorKind, parse_sheet, parse_style_attribute,
};

#[derive(Clone, Copy)]
struct InvalidDeclaration {
    authored_name: &'static str,
    invalid_value: &'static str,
    responsible: &'static str,
    property: CssKnownProperty,
}

const C12_INVALID_DECLARATIONS: &[InvalidDeclaration] = &[
    InvalidDeclaration {
        authored_name: "border-collapse",
        invalid_value: "collapse separate",
        responsible: "separate",
        property: CssKnownProperty::BorderCollapse,
    },
    InvalidDeclaration {
        authored_name: "border-spacing",
        invalid_value: "1%",
        responsible: "1%",
        property: CssKnownProperty::BorderSpacing,
    },
    InvalidDeclaration {
        authored_name: "caption-side",
        invalid_value: "top bottom",
        responsible: "bottom",
        property: CssKnownProperty::CaptionSide,
    },
    InvalidDeclaration {
        authored_name: "clip",
        invalid_value: "rect(1%, 2px, 3px, 4px)",
        responsible: "1%",
        property: CssKnownProperty::Clip,
    },
    InvalidDeclaration {
        authored_name: "empty-cells",
        invalid_value: "show hide",
        responsible: "hide",
        property: CssKnownProperty::EmptyCells,
    },
    InvalidDeclaration {
        authored_name: "orphans",
        invalid_value: "0",
        responsible: "0",
        property: CssKnownProperty::Orphans,
    },
    InvalidDeclaration {
        authored_name: "page-break-after",
        invalid_value: "always avoid",
        responsible: "avoid",
        property: CssKnownProperty::PageBreakAfter,
    },
    InvalidDeclaration {
        authored_name: "page-break-before",
        invalid_value: "page",
        responsible: "page",
        property: CssKnownProperty::PageBreakBefore,
    },
    InvalidDeclaration {
        authored_name: "page-break-inside",
        invalid_value: "always",
        responsible: "always",
        property: CssKnownProperty::PageBreakInside,
    },
    InvalidDeclaration {
        authored_name: "quotes",
        invalid_value: "\"open\"",
        responsible: "\"open\"",
        property: CssKnownProperty::Quotes,
    },
    InvalidDeclaration {
        authored_name: "table-layout",
        invalid_value: "auto fixed",
        responsible: "fixed",
        property: CssKnownProperty::TableLayout,
    },
    InvalidDeclaration {
        authored_name: "widows",
        invalid_value: "0",
        responsible: "0",
        property: CssKnownProperty::Widows,
    },
    InvalidDeclaration {
        authored_name: "word-spacing",
        invalid_value: "10%",
        responsible: "10%",
        property: CssKnownProperty::WordSpacing,
    },
    InvalidDeclaration {
        authored_name: "text-combine-upright",
        invalid_value: "none all",
        responsible: "all",
        property: CssKnownProperty::TextCombineUpright,
    },
    InvalidDeclaration {
        authored_name: "text-orientation",
        invalid_value: "mixed upright",
        responsible: "upright",
        property: CssKnownProperty::TextOrientation,
    },
    InvalidDeclaration {
        authored_name: "unicode-bidi",
        invalid_value: "isolate embed",
        responsible: "embed",
        property: CssKnownProperty::UnicodeBidi,
    },
    InvalidDeclaration {
        authored_name: "glyph-orientation-vertical",
        invalid_value: "45deg",
        responsible: "45deg",
        property: CssKnownProperty::TextOrientation,
    },
    InvalidDeclaration {
        authored_name: "caret-color",
        invalid_value: "auto red",
        responsible: "red",
        property: CssKnownProperty::CaretColor,
    },
    InvalidDeclaration {
        authored_name: "outline-offset",
        invalid_value: "10%",
        responsible: "10%",
        property: CssKnownProperty::OutlineOffset,
    },
    InvalidDeclaration {
        authored_name: "resize",
        invalid_value: "horizontal vertical",
        responsible: "vertical",
        property: CssKnownProperty::Resize,
    },
    InvalidDeclaration {
        authored_name: "contain",
        invalid_value: "size size",
        responsible: "size",
        property: CssKnownProperty::Contain,
    },
    InvalidDeclaration {
        authored_name: "transform-box",
        invalid_value: "padding-box",
        responsible: "padding-box",
        property: CssKnownProperty::TransformBox,
    },
    InvalidDeclaration {
        authored_name: "background-blend-mode",
        invalid_value: "normal multiply",
        responsible: "multiply",
        property: CssKnownProperty::BackgroundBlendMode,
    },
    InvalidDeclaration {
        authored_name: "isolation",
        invalid_value: "auto isolate",
        responsible: "isolate",
        property: CssKnownProperty::Isolation,
    },
    InvalidDeclaration {
        authored_name: "mix-blend-mode",
        invalid_value: "normal, multiply",
        responsible: ",",
        property: CssKnownProperty::MixBlendMode,
    },
];

fn property_names(declarations: &surgeist_css::CssDeclarationList) -> Vec<&str> {
    declarations
        .iter()
        .map(|declaration| match declaration.property_name() {
            CssPropertyNameRef::Known(property) => property.canonical_name(),
            CssPropertyNameRef::Custom(name) => name.as_str(),
            _ => panic!("unexpected future property-name kind"),
        })
        .collect()
}

fn assert_one_invalid_declaration(
    source: &str,
    unit_start: usize,
    unit_end: usize,
    responsible: usize,
    expected_property: CssKnownProperty,
) {
    let report = parse_style_attribute(source);
    assert_eq!(
        property_names(report.syntax()),
        ["--😀", "color"],
        "{source}"
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("{source}: expected exactly one declaration diagnostic");
    };
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
        unit_start,
        "{source}",
    );
    assert_eq!(
        diagnostic.span().end().byte_offset().value(),
        unit_end,
        "{source}",
    );
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        responsible,
        "{source}",
    );
    assert_eq!(
        diagnostic.error().position().column().value() as usize,
        source[..diagnostic.error().position().byte_offset().value()]
            .encode_utf16()
            .count(),
        "{source}",
    );
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("{source}: expected a property-specific value diagnostic");
    };
    assert_eq!(detail.property(), expected_property, "{source}");

    #[cfg(feature = "app-strict")]
    assert_eq!(
        surgeist_css::validate_style_attribute(source)
            .expect_err("strict validation rejects recovered C12 syntax")
            .diagnostics(),
        report.diagnostics(),
        "{source}",
    );
}

#[test]
fn c12_residual_recovery_preserves_siblings_and_boundaries() {
    for case in C12_INVALID_DECLARATIONS {
        let invalid = format!("{}: {};", case.authored_name, case.invalid_value);
        let source = format!("--😀: kept; {invalid} color: red");
        let unit_start = source.find(&invalid).expect("invalid declaration");
        let value_start = invalid.find(": ").expect("declaration colon") + 2;
        let responsible = if case.responsible == ";" {
            unit_start + invalid.find(';').expect("declaration terminator")
        } else if case.authored_name == "contain" {
            unit_start
                + value_start
                + case
                    .invalid_value
                    .rfind(case.responsible)
                    .expect("responsible token")
        } else {
            unit_start
                + value_start
                + case
                    .invalid_value
                    .find(case.responsible)
                    .expect("responsible token")
        };
        assert_one_invalid_declaration(
            &source,
            unit_start,
            unit_start + invalid.len(),
            responsible,
            case.property,
        );
    }

    assert_eq!(
        CssBoxEdgeKeyword::from_keyword("content-box padding-box"),
        None,
    );
    assert_eq!(CssBlendMode::from_keyword("normal multiply"), None);

    let repeated = concat!(
        "border-collapse: collapse separate; ",
        "text-orientation: mixed upright; ",
        "background-blend-mode: normal multiply; ",
        "color: red",
    );
    let repeated_report = parse_style_attribute(repeated);
    assert_eq!(property_names(repeated_report.syntax()), ["color"]);
    assert_eq!(repeated_report.diagnostics().len(), 3);
    for (diagnostic, property) in repeated_report.diagnostics().iter().zip([
        CssKnownProperty::BorderCollapse,
        CssKnownProperty::TextOrientation,
        CssKnownProperty::BackgroundBlendMode,
    ]) {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert!(matches!(
            diagnostic.error().kind(),
            ErrorKind::InvalidPropertyValue(detail) if detail.property() == property
        ));
    }

    let nested = concat!(
        ".parent { border-spacing: 1%; color: red; ",
        "& .child { contain: size size; width: 1px; } word-spacing: 2px; } ",
        ".after { height: 3px; }",
    );
    let nested_report = parse_sheet(nested);
    assert_eq!(nested_report.diagnostics().len(), 2);
    assert!(nested_report.diagnostics().iter().all(|diagnostic| {
        diagnostic.error().code() == CssErrorCode::InvalidPropertyValue
            && diagnostic.action() == CssRecoveryAction::DropDeclaration
    }));
    let retained = nested_report
        .syntax()
        .rules()
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Style(style) => Some(property_names(style.declarations())),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(retained, ["color", "width", "word-spacing", "height"]);

    let eof_source = "quotes: \"open\"";
    let eof_report = parse_style_attribute(eof_source);
    let [eof] = eof_report.diagnostics() else {
        panic!("odd quotes value at EOF must recover exactly once");
    };
    assert_eq!(eof.span().start().byte_offset().value(), 0);
    assert_eq!(eof.span().end().byte_offset().value(), eof_source.len());
    assert!(matches!(
        eof.error().kind(),
        ErrorKind::InvalidPropertyValue(detail)
            if detail.property() == CssKnownProperty::Quotes
    ));

    for depth in [255_usize, 256] {
        let value = format!(
            "var(--fallback, {}size{})",
            "f(".repeat(depth - 1),
            ")".repeat(depth - 1),
        );
        let source = format!("contain: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics(),
        );
        assert_eq!(property_names(report.syntax()), ["contain", "color"]);
    }

    let depth = 257_usize;
    let value = format!(
        "var(--fallback, {}size{})",
        "f(".repeat(depth - 1),
        ")".repeat(depth - 1),
    );
    let invalid = format!("contain: {value};");
    let source = format!("{invalid} color: red");
    let report = parse_style_attribute(&source);
    assert_eq!(property_names(report.syntax()), ["color"]);
    let [diagnostic] = report.diagnostics() else {
        panic!("depth 257 must produce exactly one bounded diagnostic");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::NestingLimit);
    assert_eq!(diagnostic.action(), CssRecoveryAction::StopAtNestingLimit,);
    assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
    assert_eq!(diagnostic.span().end().byte_offset().value(), invalid.len());

    for depth in [255_usize, 256, 257] {
        let invalid_value = format!("{}x{}", "f(".repeat(depth), ")".repeat(depth));
        let invalid = format!("contain:{invalid_value};");
        let nested_source = format!("@page{{{invalid}margin-left:1cm;}}.after{{height:1px;}}");
        let nested_report = parse_sheet(&nested_source);
        assert!(matches!(
            nested_report.syntax().rules(),
            [CssRule::Page(_), CssRule::Style(_)]
        ));
        let [nested_diagnostic] = nested_report.diagnostics() else {
            panic!("page-context depth {depth} must recover one C12 declaration");
        };
        let invalid_start = nested_source.find(&invalid).expect("nested declaration");
        assert_eq!(
            nested_diagnostic.span().start().byte_offset().value(),
            invalid_start,
            "depth {depth}",
        );
        assert_eq!(
            nested_diagnostic.span().end().byte_offset().value(),
            invalid_start + invalid.len(),
            "depth {depth}",
        );
        assert_eq!(
            nested_diagnostic.error().code(),
            if depth >= 256 {
                CssErrorCode::NestingLimit
            } else {
                CssErrorCode::InvalidPropertyValue
            },
            "depth {depth}: {nested_diagnostic:#?}",
        );
        assert_eq!(
            nested_diagnostic.action(),
            if depth >= 256 {
                CssRecoveryAction::StopAtNestingLimit
            } else {
                CssRecoveryAction::DropDeclaration
            },
            "depth {depth}",
        );
        if depth >= 256 {
            let first_over_limit = nested_source
                .match_indices("f(")
                .nth(255)
                .expect("256th nested function")
                .0;
            assert_eq!(
                nested_diagnostic.error().position().byte_offset().value(),
                first_over_limit,
                "depth {depth}",
            );
        } else {
            assert!(matches!(
                nested_diagnostic.error().kind(),
                ErrorKind::InvalidPropertyValue(detail)
                    if detail.property() == CssKnownProperty::Contain
            ));
        }

        #[cfg(feature = "app-strict")]
        assert_eq!(
            surgeist_css::validate_sheet(&nested_source)
                .expect_err("strict validation rejects nested C12 recovery")
                .diagnostics(),
            nested_report.diagnostics(),
            "depth {depth}",
        );
    }
}

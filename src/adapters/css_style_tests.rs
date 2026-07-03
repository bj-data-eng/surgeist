use super::{AdapterErrorKind, lower_css_sheet_to_style};
use crate::{css, style};

fn lower(input: &str) -> style::Sheet {
    let css = css::parse_sheet(input).expect("CSS should parse");
    lower_css_sheet_to_style(&css).expect("CSS should lower to style")
}

#[test]
fn css_sheet_lowers_to_style_sheet() {
    let css = css::parse_sheet(".panel { width: 12px; margin: auto; }").unwrap();
    let sheet = lower_css_sheet_to_style(&css).unwrap();

    assert_eq!(sheet.rules().len(), 1);
}

#[test]
fn parses_calc_width_as_style_calc_length() {
    let sheet = lower(".panel { width: calc(20px + 10%); }");
    let value = sheet.rules()[0]
        .declarations()
        .get(style::Property::Width)
        .expect("width should be present");

    match value {
        style::Value::Length(style::Length::Calc(calc)) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(20px + 10%)");
        }
        other => panic!("expected calc width, got {other:?}"),
    }
}

#[test]
fn parses_nested_calc_width_with_subtraction() {
    let sheet = lower(".panel { width: calc(100% - calc(12px + 3%)); }");
    let value = sheet.rules()[0]
        .declarations()
        .get(style::Property::Width)
        .expect("width should be present");

    match value {
        style::Value::Length(style::Length::Calc(calc)) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(100% - calc(12px + 3%))");
        }
        other => panic!("expected nested calc width, got {other:?}"),
    }
}

#[test]
fn style_validation_error_uses_declaration_location() {
    let css = css::parse_sheet(".panel {\n  opacity: 2;\n}").unwrap();
    let error = lower_css_sheet_to_style(&css).unwrap_err();

    match error.kind() {
        AdapterErrorKind::CssStyleValidation { property, reason } => {
            assert_eq!(property, "Opacity");
            assert!(reason.contains("line 1, column 3"), "{reason}");
            assert!(
                reason.contains("Opacity must be between 0 and 1"),
                "{reason}"
            );
        }
        other => panic!("expected style validation error, got {other:?}"),
    }
}

#[test]
fn css_global_keywords_lower_to_style_keywords() {
    let sheet = lower(".panel { color: inherit; padding: initial; margin: unset; }");
    let declarations = sheet.rules()[0].declarations();

    assert_eq!(
        declarations.get(style::Property::Color),
        Some(&style::Value::Keyword(style::Keyword::Inherit))
    );
    assert_eq!(
        declarations.get(style::Property::Padding),
        Some(&style::Value::Keyword(style::Keyword::Initial))
    );
    assert_eq!(
        declarations.get(style::Property::Margin),
        Some(&style::Value::Keyword(style::Keyword::Unset))
    );
}

#[test]
fn unsupported_css_global_keywords_are_adapter_errors() {
    for keyword in ["revert", "revert-layer"] {
        let css = css::parse_sheet(&format!(".panel {{ width: {keyword}; }}")).unwrap();
        let error = lower_css_sheet_to_style(&css).unwrap_err();

        match error.kind() {
            AdapterErrorKind::CssValueUnsupported { property, reason } => {
                assert_eq!(property, "Width");
                assert!(
                    reason.contains(&format!("unsupported CSS global keyword `{keyword}`")),
                    "{reason}"
                );
            }
            other => panic!("expected CSS value unsupported error for {keyword}, got {other:?}"),
        }
    }
}

#[test]
fn parses_calc_in_edge_shorthands() {
    let sheet = lower(".panel { margin: calc(4px + 1%) 2px; }");
    let edges = match sheet.rules()[0].declarations().get(style::Property::Margin) {
        Some(style::Value::Edges(edges)) => edges,
        other => panic!("expected margin edges, got {other:?}"),
    };

    match &edges.top {
        style::Length::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(4px + 1%)");
        }
        other => panic!("expected calc top edge, got {other:?}"),
    }
    assert_eq!(edges.right, style::Length::px(2.0));
    match &edges.bottom {
        style::Length::Calc(calc) => {
            assert!(calc.uses_percentage());
            assert_eq!(calc.to_css_string(), "calc(4px + 1%)");
        }
        other => panic!("expected calc bottom edge, got {other:?}"),
    }
    assert_eq!(edges.left, style::Length::px(2.0));
}

#[test]
fn parses_normal_gap_without_treating_it_as_calc() {
    let sheet = lower(".panel { gap: normal; }");
    let declarations = sheet.rules()[0].declarations();

    assert_eq!(
        declarations.get(style::Property::RowGap),
        Some(&style::Value::Length(style::Length::NORMAL))
    );
    assert_eq!(
        declarations.get(style::Property::ColumnGap),
        Some(&style::Value::Length(style::Length::NORMAL))
    );
}

#[test]
fn parses_calc_gap() {
    let sheet = lower(".panel { gap: calc(8px + 2%); }");
    let declarations = sheet.rules()[0].declarations();

    for property in [style::Property::RowGap, style::Property::ColumnGap] {
        match declarations.get(property) {
            Some(style::Value::Length(style::Length::Calc(calc))) => {
                assert!(calc.uses_percentage());
                assert_eq!(calc.to_css_string(), "calc(8px + 2%)");
            }
            other => panic!("expected calc gap for {property:?}, got {other:?}"),
        }
    }
}

#[test]
fn grid_flow_tolerance_calc_reaches_style_validation() {
    let css = css::parse_sheet(".panel { grid-flow-tolerance: calc(8px + 2%); }").unwrap();
    let error = lower_css_sheet_to_style(&css).unwrap_err();

    match error.kind() {
        AdapterErrorKind::CssStyleValidation { property, reason } => {
            assert_eq!(property, "GridFlowTolerance");
            assert!(reason.contains("line 0, column 10"), "{reason}");
            assert!(
                reason.contains("grid flow tolerance length must be a concrete px length"),
                "{reason}"
            );
        }
        other => panic!("expected style validation error, got {other:?}"),
    }
}

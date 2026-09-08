use surgeist_css::{
    CssAuthoredGridAutoRepeatKind, CssAuthoredGridAutoTrackComponent,
    CssAuthoredGridGeneralTrackComponent, CssErrorCode, CssGridRepeatCount, CssKnownProperty,
    CssKnownPropertyValueRef, CssRecoveryAction, CssTokenKind, ErrorKind, parse_style_attribute,
};

#[test]
fn grid_repeat_models_reject_invalid_cross_products() {
    for invalid in [
        "grid-template-columns: repeat(2, repeat(3, 10px))",
        "grid-template-columns: repeat(auto-fit, 1fr)",
    ] {
        let source = format!("{invalid}; color: red");
        let report = parse_style_attribute(&source);

        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained sibling")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: invalid repetition must recover once");
        };
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    }
}

#[test]
fn grid_repeat_models_accept_each_structural_language_and_project_exact_i01_values() {
    let report = parse_style_attribute(concat!(
        "grid-template-columns: [a] repeat(2, minmax(10px, 1fr) [b]) 1fr; ",
        "grid-template-rows: 20px repeat(auto-fill, minmax(auto, 10px)) repeat(2, 5px); ",
        "grid-auto-rows: minmax(10px, auto) fit-content(20%); ",
        "grid-template: repeat(2, 10px 1fr) / repeat(auto-fit, minmax(10px, 1fr)); ",
        "grid: auto-flow dense 12px / repeat(auto-fit, 10px)",
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 5);

    let CssKnownPropertyValueRef::GridTemplateColumns(columns) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-template-columns");
    };
    let general = columns
        .current()
        .general_list()
        .expect("general track list");
    assert!(matches!(
        general.components()[1],
        CssAuthoredGridGeneralTrackComponent::Repeat(_)
    ));
    let old = columns
        .i01_subset()
        .expect("legacy-compatible general list");
    let surgeist_css::CssGridTrackComponent::Repeat(repeat) = &old.components()[1] else {
        panic!("expected projected repeat");
    };
    assert!(matches!(repeat.count(), CssGridRepeatCount::Integer(value) if value.value() == 2));

    let CssKnownPropertyValueRef::GridTemplateRows(rows) = report.syntax()[1]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-template-rows");
    };
    let auto = rows.current().auto_list().expect("auto track list");
    let auto_repeat = auto
        .components()
        .iter()
        .find_map(|component| match component {
            CssAuthoredGridAutoTrackComponent::AutoRepeat(value) => Some(value),
            _ => None,
        })
        .expect("single automatic repetition");
    assert_eq!(auto_repeat.kind(), CssAuthoredGridAutoRepeatKind::AutoFill);
    assert!(rows.i01_subset().is_some());

    let CssKnownPropertyValueRef::GridAutoRows(auto_rows) = report.syntax()[2]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-auto-rows");
    };
    assert_eq!(auto_rows.current().sizes().len(), 2);
    assert!(auto_rows.i01_subset().is_some());

    let CssKnownPropertyValueRef::GridTemplate(template) = report.syntax()[3]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-template");
    };
    assert!(template.current().rows().unwrap().general_list().is_some());
    assert!(template.current().columns().unwrap().auto_list().is_some());
    assert!(template.i01_subset().is_some());

    let CssKnownPropertyValueRef::Grid(grid) = report.syntax()[4]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid");
    };
    assert!(grid.current().auto_flow().unwrap().dense());
    assert!(
        grid.current()
            .explicit_tracks()
            .unwrap()
            .auto_list()
            .is_some()
    );
    assert!(grid.i01_subset().is_some());
}

#[test]
fn grid_repeat_models_reject_each_invalid_structural_cross_product() {
    for (property, value) in [
        ("grid-template-columns", "repeat(2, [only])"),
        ("grid-template-columns", "repeat(auto-fit, auto)"),
        (
            "grid-template-columns",
            "repeat(auto-fit, fit-content(10px))",
        ),
        (
            "grid-template-columns",
            "repeat(auto-fit, minmax(auto, 1fr))",
        ),
        ("grid-template-columns", "1fr repeat(auto-fit, 10px)"),
        ("grid-template-columns", "repeat(auto-fit, 10px) 1fr"),
        (
            "grid-template-columns",
            "repeat(auto-fit, 10px) repeat(auto-fill, 20px)",
        ),
        (
            "grid-template-columns",
            "repeat(auto-fit, 10px) repeat(2, 1fr)",
        ),
        ("grid-auto-rows", "repeat(2, 10px)"),
        ("grid-auto-columns", "repeat(auto-fill, 10px)"),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].error().code(),
            CssErrorCode::InvalidPropertyValue,
            "{source}",
        );
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{source}",
        );
    }
}

#[test]
fn flexible_minmax_minimums_are_rejected_at_the_first_responsible_token() {
    for (property, value) in [
        ("grid-template-columns", "minmax(1fr, 10px)"),
        ("grid-template-columns", "repeat(2, minmax(1fr, 10px))"),
        ("grid-auto-rows", "minmax(1fr, 10px)"),
        ("grid-auto-columns", "minmax(1fr, 10px)"),
    ] {
        let source = format!("{property}: {value}; color: red");
        let report = parse_style_attribute(&source);

        assert_eq!(report.syntax().len(), 1, "{source}");
        assert_eq!(
            report.syntax()[0]
                .known()
                .expect("retained sibling")
                .property(),
            CssKnownProperty::Color,
            "{source}",
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("{source}: flexible minimum must recover once");
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
        let responsible = source.find("1fr").expect("flexible minimum");
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            responsible,
            "{source}",
        );
        assert_eq!(diagnostic.error().position().line().value(), 0, "{source}");
        assert_eq!(
            diagnostic.error().position().column().value() as usize,
            responsible,
            "{source}",
        );
        assert_eq!(
            diagnostic.span().start().byte_offset().value(),
            0,
            "{source}"
        );
        assert_eq!(
            diagnostic.span().end().byte_offset().value(),
            source.find(';').expect("declaration terminator") + 1,
            "{source}",
        );
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("{source}: expected Grid property error");
        };
        assert_eq!(
            detail.encountered().expect("responsible fraction").kind(),
            CssTokenKind::Dimension,
            "{source}",
        );
        assert_eq!(
            detail
                .encountered()
                .expect("responsible fraction")
                .authored(),
            "1fr",
            "{source}",
        );

        #[cfg(feature = "app-strict")]
        {
            let failure = surgeist_css::validate_style_attribute(&source)
                .expect_err("strict validation rejects a flexible minmax minimum");
            assert_eq!(failure.diagnostics(), report.diagnostics(), "{source}");
        }
    }
}

#[test]
fn grid_repeat_typed_calculation_stays_symbolic_and_outside_i01_projection() {
    let report = parse_style_attribute(
        "grid-template-columns: repeat(auto-fit, calc((10px + 5%) * 2)); color: red",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::GridTemplateColumns(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("expected grid-template-columns");
    };
    assert!(value.current().auto_list().is_some());
    assert!(value.i01_subset().is_none());
}

#[test]
fn grid_repeat_failures_report_the_first_responsible_token_and_recover_progressively() {
    let source = concat!(
        "grid-template-columns: repeat(auto-fit, 1fr); ",
        "grid-auto-rows: repeat(2, 10px); ",
        "color: red",
    );
    let report = parse_style_attribute(source);
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(report.diagnostics().len(), 2);
    for (diagnostic, responsible) in report.diagnostics().iter().zip(["1fr", "repeat("]) {
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("expected property value diagnostic");
        };
        let encountered = detail.encountered().expect("responsible token");
        assert_eq!(encountered.authored(), responsible);
        assert!(matches!(
            encountered.kind(),
            CssTokenKind::Dimension | CssTokenKind::Function
        ));
    }
}

#[test]
fn grid_repeat_calculations_preserve_the_exact_depth_boundary() {
    for depth in [254_usize, 255] {
        let value = format!("{}10px{}", "calc(".repeat(depth), ")".repeat(depth));
        let source = format!("grid-template-columns: repeat(auto-fill, {value}); color: red");
        let report = parse_style_attribute(&source);
        assert!(
            report.is_clean(),
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        assert_eq!(report.syntax().len(), 2, "depth {depth}");
    }

    for depth in [256_usize, 257] {
        let value = format!("{}10px{}", "calc(".repeat(depth), ")".repeat(depth));
        let source = format!("grid-template-columns: repeat(auto-fill, {value}); color: red");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 1, "depth {depth}");
        assert_eq!(report.diagnostics().len(), 1, "depth {depth}");
    }
}

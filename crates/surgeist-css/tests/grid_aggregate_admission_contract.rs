#![forbid(unsafe_code)]
//! Existing admission boundaries, before aggregate output APIs.
//! Independent authority: selected Grid2 2025-03-26 §7.4 (required row/column
//! slash), §7.8 (both auto-flow orientations and unordered dense), and §7.2
//! (auto/min-content track breadths). Grid3 2026-01-21 §3.1 retains these track
//! properties/shorthands. No area-string or none-as-track-child model assumption.
use surgeist_css::*;

fn name(property: CssKnownProperty) -> &'static str {
    match property {
        CssKnownProperty::GridTemplate => "grid-template",
        CssKnownProperty::Grid => "grid",
        _ => panic!("aggregate property"),
    }
}

fn occurrence(declaration: &CssDeclaration, property: CssKnownProperty) {
    assert_eq!(declaration.importance(), CssImportance::Important);
    let known = declaration.known().expect("known aggregate occurrence");
    assert_eq!(known.property(), property);
    assert_eq!(known.grammar(), property.grammar());
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    match (property, known.property_value()) {
        (CssKnownProperty::GridTemplate, Some(CssKnownPropertyValueRef::GridTemplate(_)))
        | (CssKnownProperty::Grid, Some(CssKnownPropertyValueRef::Grid(_))) => {}
        _ => panic!("ordinary typed aggregate"),
    }
}

fn parsed(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let source = format!("{}:{value}!important", name(property));
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1, "{source}");
    let declaration = report.syntax()[0].clone();
    occurrence(&declaration, property);
    declaration
}

fn checked(property: CssKnownProperty, value: &str, by_grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).expect("clean components");
    let result = if by_grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    };
    let declaration =
        result.unwrap_or_else(|error| panic!("{}:{value}: {error:?}", name(property)));
    occurrence(&declaration, property);
    declaration
}

fn grid(declaration: &CssDeclaration) -> &CssGrid {
    let Some(CssKnownPropertyValueRef::Grid(value)) = declaration.known().unwrap().property_value()
    else {
        panic!("grid");
    };
    value.value()
}

fn template(declaration: &CssDeclaration) -> &CssGridTemplate {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplate(value) => value.value(),
        CssKnownPropertyValueRef::Grid(value) => {
            value.value().template_value().expect("template branch")
        }
        _ => panic!("template aggregate"),
    }
}

fn assert_flow(
    declaration: &CssDeclaration,
    axis: CssGridAutoFlowAxis,
    dense: bool,
    auto_size: Option<&str>,
    explicit_size: &str,
) {
    let value = grid(declaration);
    assert!(value.template_value().is_none());
    let flow = value.auto_flow().expect("auto-flow branch");
    assert_eq!(flow.axis(), axis);
    assert_eq!(flow.dense(), dense);
    match (value.auto_tracks(), auto_size) {
        (None, None) => {}
        (Some(tracks), Some(expected)) => {
            assert_eq!(tracks.serialize_specified().unwrap(), expected);
        }
        _ => panic!("retain optional auto sizing"),
    }
    assert_eq!(
        value
            .explicit_tracks()
            .unwrap()
            .serialize_specified()
            .unwrap(),
        explicit_size
    );
}

fn reject_rows_style(property: CssKnownProperty) {
    let source = format!("{}:10px;color:red", name(property));
    let report = parse_style_attribute(&source);
    let [retained] = report.syntax().as_slice() else {
        panic!("only the valid color sibling survives");
    };
    assert_eq!(
        retained.known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid aggregate diagnostic");
    };
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

fn reject_rows_validator(property: CssKnownProperty) {
    let source = format!("{}:10px", name(property));
    assert!(
        validate_style_attribute(&source).is_err(),
        "{source}: required column slash missing"
    );
}

fn reject_rows_checked(property: CssKnownProperty, by_grammar: bool) {
    let components = parse_component_values("10px").unwrap();
    let result = if by_grammar {
        parse_property_value_for_grammar(property.grammar(), components, CssImportance::Important)
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
    };
    assert!(
        result.is_err(),
        "{}:10px is not the rows/columns shorthand form",
        name(property)
    );
}

macro_rules! rows_only_contract {
    ($style:ident, $validator:ident, $property:ident, $grammar:ident, $kind:ident) => {
        #[test]
        fn $style() {
            reject_rows_style(CssKnownProperty::$kind);
        }
        #[test]
        fn $validator() {
            reject_rows_validator(CssKnownProperty::$kind);
        }
        #[test]
        fn $property() {
            reject_rows_checked(CssKnownProperty::$kind, false);
        }
        #[test]
        fn $grammar() {
            reject_rows_checked(CssKnownProperty::$kind, true);
        }
    };
}
rows_only_contract!(
    template_rows_only_style_drops_invalid_declaration,
    template_rows_only_validator_rejects,
    template_rows_only_checked_property_rejects,
    template_rows_only_checked_grammar_rejects,
    GridTemplate
);
rows_only_contract!(
    grid_rows_only_style_drops_invalid_declaration,
    grid_rows_only_validator_rejects,
    grid_rows_only_checked_property_rejects,
    grid_rows_only_checked_grammar_rejects,
    Grid
);

macro_rules! flow_contract {
    ($style:ident, $property:ident, $grammar:ident, $input:literal, $axis:ident, $dense:literal, $auto:expr) => {
        #[test]
        fn $style() {
            assert_flow(
                &parsed(CssKnownProperty::Grid, $input),
                CssGridAutoFlowAxis::$axis,
                $dense,
                $auto,
                "10px",
            );
        }
        #[test]
        fn $property() {
            assert_flow(
                &checked(CssKnownProperty::Grid, $input, false),
                CssGridAutoFlowAxis::$axis,
                $dense,
                $auto,
                "10px",
            );
        }
        #[test]
        fn $grammar() {
            assert_flow(
                &checked(CssKnownProperty::Grid, $input, true),
                CssGridAutoFlowAxis::$axis,
                $dense,
                $auto,
                "10px",
            );
        }
    };
}
flow_contract!(
    column_without_auto_size_parses,
    column_without_auto_size_checked_property,
    column_without_auto_size_checked_grammar,
    "10px / auto-flow",
    Column,
    false,
    None
);
flow_contract!(
    column_with_dense_auto_size_parses,
    column_with_dense_auto_size_checked_property,
    column_with_dense_auto_size_checked_grammar,
    "10px / auto-flow dense 20px",
    Column,
    true,
    Some("20px")
);
flow_contract!(
    row_dense_before_auto_flow_parses,
    row_dense_before_auto_flow_checked_property,
    row_dense_before_auto_flow_checked_grammar,
    "dense auto-flow 20px / 10px",
    Row,
    true,
    Some("20px")
);
flow_contract!(
    column_dense_before_auto_flow_parses,
    column_dense_before_auto_flow_checked_property,
    column_dense_before_auto_flow_checked_grammar,
    "10px / dense auto-flow 20px",
    Column,
    true,
    Some("20px")
);

fn keyword_tracks(declaration: &CssDeclaration) {
    let value = template(declaration);
    assert!(!value.is_none());
    assert_eq!(value.rows().unwrap().serialize_specified().unwrap(), "auto");
    assert_eq!(
        value.columns().unwrap().serialize_specified().unwrap(),
        "min-content"
    );
}

macro_rules! keyword_contract {
    ($style:ident, $property:ident, $grammar:ident, $kind:ident) => {
        #[test]
        fn $style() {
            keyword_tracks(&parsed(CssKnownProperty::$kind, "auto / min-content"));
        }
        #[test]
        fn $property() {
            keyword_tracks(&checked(
                CssKnownProperty::$kind,
                "auto / min-content",
                false,
            ));
        }
        #[test]
        fn $grammar() {
            keyword_tracks(&checked(
                CssKnownProperty::$kind,
                "auto / min-content",
                true,
            ));
        }
    };
}
keyword_contract!(
    template_keyword_start_tracks_parse,
    template_keyword_start_tracks_checked_property,
    template_keyword_start_tracks_checked_grammar,
    GridTemplate
);
keyword_contract!(
    grid_keyword_start_tracks_parse,
    grid_keyword_start_tracks_checked_property,
    grid_keyword_start_tracks_checked_grammar,
    Grid
);

#[test]
fn none_and_explicit_rows_columns_controls_pass_all_existing_front_doors() {
    for property in [CssKnownProperty::GridTemplate, CssKnownProperty::Grid] {
        for value in ["none", "10px / 20px"] {
            for declaration in [
                parsed(property, value),
                checked(property, value, false),
                checked(property, value, true),
            ] {
                let aggregate = template(&declaration);
                if value == "none" {
                    assert!(aggregate.is_none());
                    assert!(aggregate.rows().is_none());
                    assert!(aggregate.columns().is_none());
                } else {
                    assert!(!aggregate.is_none());
                    assert_eq!(
                        aggregate.rows().unwrap().serialize_specified().unwrap(),
                        "10px"
                    );
                    assert_eq!(
                        aggregate.columns().unwrap().serialize_specified().unwrap(),
                        "20px"
                    );
                }
            }
        }
    }
}

#[test]
fn row_flow_with_and_without_auto_sizes_controls_pass_all_existing_front_doors() {
    for (input, dense, auto) in [
        ("auto-flow / 10px", false, None),
        ("auto-flow dense / 10px", true, None),
        ("auto-flow 20px / 10px", false, Some("20px")),
        ("auto-flow dense 20px / 10px", true, Some("20px")),
    ] {
        for declaration in [
            parsed(CssKnownProperty::Grid, input),
            checked(CssKnownProperty::Grid, input, false),
            checked(CssKnownProperty::Grid, input, true),
        ] {
            assert_flow(&declaration, CssGridAutoFlowAxis::Row, dense, auto, "10px");
        }
    }
}

#[test]
fn malformed_duplicate_prefixes_and_slashes_are_rejected_wholly() {
    for input in [
        "auto-flow auto-flow / 10px",
        "10px / auto-flow auto-flow",
        "auto-flow dense dense / 10px",
        "10px / auto-flow dense dense",
        "auto-flow / / 10px",
        "10px // auto-flow",
        "10px / auto-flow / 20px",
        "10px / auto-flow dense 20px extra",
    ] {
        let source = format!("grid:{input};color:red");
        let report = parse_style_attribute(&source);
        assert!(!report.is_clean(), "{input}");
        let [retained] = report.syntax().as_slice() else {
            panic!("invalid grid is wholly dropped: {input}");
        };
        assert_eq!(
            retained.known().unwrap().property(),
            CssKnownProperty::Color
        );
        for by_grammar in [false, true] {
            let components = parse_component_values(input).unwrap();
            let result = if by_grammar {
                parse_property_value_for_grammar(
                    CssKnownProperty::Grid.grammar(),
                    components,
                    CssImportance::Important,
                )
            } else {
                parse_property_value(
                    CssPropertyNameRef::Known(CssKnownProperty::Grid),
                    components,
                    CssImportance::Important,
                )
            };
            assert!(result.is_err(), "{input}");
        }
    }
}

#[test]
fn valid_row_flow_preserves_authored_case_importance_order_and_utf8_operand_origins() {
    let source = "/*😀*/color:red;GRID:AUTO-FLOW dense 20px / 10px!important;width:1px";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [before, declaration, after] = report.syntax().as_slice() else {
        panic!("three original occurrences");
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    occurrence(declaration, CssKnownProperty::Grid);
    assert_flow(
        declaration,
        CssGridAutoFlowAxis::Row,
        true,
        Some("20px"),
        "10px",
    );
    let Some(CssKnownPropertyValueRef::Grid(wrapper)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("grid wrapper");
    };
    assert_eq!(wrapper.as_css(), "AUTO-FLOW dense 20px / 10px");
    let aggregate = grid(declaration);
    let auto = aggregate.auto_tracks().unwrap().sizes()[0]
        .breadth()
        .unwrap()
        .length_percentage()
        .unwrap();
    let CssGridGeneralTrackComponent::TrackSize(explicit) = &aggregate
        .explicit_tracks()
        .unwrap()
        .general_list()
        .unwrap()
        .components()[0]
    else {
        panic!("one explicit size");
    };
    let explicit = explicit.breadth().unwrap().length_percentage().unwrap();
    for (operand, token) in [(auto, "20px"), (explicit, "10px")] {
        let CssValueOrigin::Parsed(origin) = operand.origin() else {
            panic!("parsed operand");
        };
        let start = source.find(token).unwrap();
        assert_eq!(origin.span().start().byte_offset().value(), start);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            start + token.len()
        );
        assert_eq!(origin.source().as_str(), source);
    }
}

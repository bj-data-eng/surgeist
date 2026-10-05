#![forbid(unsafe_code)]
//! Authored Grid contract through existing public APIs.
//!
//! Independent oracles: pinned Grid2 2025-03-26 reference lines 1520-1524,
//! 1783-1794, 2198-2201, 2227, 2253, 2821-2843, 2868-2886 and 3259-3317;
//! Grid3 2026-01-21 lines 428/465 preserves the accepted automatic-repeat body;
//! Values4 2024-03-12 lines 3835/4625 admits number-result math in integer slots.
//! Accepted product policy retains normal/bare dense, exact numeric math,
//! decoded checked line names, empty groups, mandatory shorthand slash, and
//! both auto-flow orientations. Cascade5 supplies all-member global propagation.
//!
//! Full specification membership is asserted without prescribing a terminal
//! order not specified by Grid. Once chosen, metadata order must govern every
//! ordinary/global/reentry result and remain stable across normalization.
//! New none/subgrid/area borrowed views and checked constructors need functional
//! tests with implementation. This draft never names absent variants or stubs.
//! Explicit row/column projection values and their new child origins cannot be
//! fully inspected through today's borrowed longhand API; identity/provenance
//! is covered here, while existing area/implicit borrowed views cover defaults.
//! Catalog source completeness and downstream execution are not test oracles.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const ROLES: [(P, &str); 4] = [
    (P::GridTemplateRows, "10px"),
    (P::GridTemplateColumns, "20px"),
    (P::GridTemplate, "10px / 20px"),
    (P::Grid, "10px / 20px"),
];
const EXPLICIT: [P; 3] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
];
const GRID: [P; 6] = [
    P::GridTemplateRows,
    P::GridTemplateColumns,
    P::GridTemplateAreas,
    P::GridAutoRows,
    P::GridAutoColumns,
    P::GridAutoFlow,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn parsed(property: P, value: &str) -> CssDeclaration {
    let css = format!("/*😀*/{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), property);
    assert_eq!(source.known().unwrap().grammar(), property.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    source.clone()
}
fn checked(property: P, value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    let source = if grammar {
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{}:{value}: {error:?}", property.canonical_name()));
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    assert_eq!(source.known().unwrap().grammar(), property.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    source
}
fn fronts(property: P, value: &str) -> [CssDeclaration; 3] {
    [
        parsed(property, value),
        checked(property, value, false),
        checked(property, value, true),
    ]
}
fn typed_output(
    source: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::GridTemplateRows(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridTemplateColumns(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::GridTemplate(v) => {
            v.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Grid(v) => v.value().serialize_specified_with_limits(limits),
        _ => panic!("owning typed Grid wrapper"),
    }
}
fn accepted(property: P, value: &str, expected: &str) {
    for source in fronts(property, value) {
        assert!(source.known().unwrap().global().is_none());
        assert!(source.known().unwrap().substitution_dependent().is_none());
        let before = source.clone();
        assert_eq!(
            typed_output(&source, CssSpecifiedValueSerializationLimits::default()).unwrap(),
            expected
        );
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{}: {expected} !important;", property.canonical_name())
        );
        assert_eq!(source, before);
        checked(property, expected, false);
        checked(property, expected, true);
    }
}
fn invalid(property: P, value: &str) {
    let css = format!("color:red;{}:{value};color:blue", property.canonical_name());
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{css}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|d| d.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic declaration failure: {css}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed property failure")
    };
    assert_eq!(detail.property(), property);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(value).unwrap();
    let before = components.clone();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(property),
            components.clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    assert!(
        parse_property_value_for_grammar(
            property.grammar(),
            components.clone(),
            CssImportance::Normal
        )
        .is_err()
    );
    assert_eq!(components, before);
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("completed intrinsic Grid contributions")
    };
    values
}
fn names(values: &CssLonghandContributions) -> Vec<P> {
    values
        .items()
        .iter()
        .map(CssLonghandContribution::property)
        .collect()
}
fn members(property: P) -> Vec<P> {
    let metadata = property.metadata().unwrap();
    match metadata.kind() {
        CssPropertyKindRef::Longhand(v) => vec![v.property().known_property()],
        CssPropertyKindRef::Shorthand(v) => {
            v.members().iter().map(|p| p.known_property()).collect()
        }
        _ => panic!("Grid longhand or ordinary shorthand"),
    }
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
}
fn ordinary(source: &CssDeclaration) -> CssLonghandContributions {
    let values = completed(source);
    assert_eq!(names(&values), members(source.known().unwrap().property()));
    for item in values.items() {
        assert_source(item, source);
        assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
        assert_eq!(
            item.ordinary_value().unwrap().property().known_property(),
            item.property()
        );
        assert!(item.replacement_components().is_none());
    }
    values
}
fn existing_terminal_output(values: &CssLonghandContributions, property: P) -> String {
    let item = values
        .items()
        .iter()
        .find(|v| v.property() == property)
        .unwrap();
    match item.ordinary_value().unwrap().view() {
        CssLonghandValueRef::GridTemplateAreas(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::GridAutoRows(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::GridAutoColumns(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::GridAutoFlow(v) => v.serialize_specified().unwrap(),
        _ => panic!("currently callable borrowed dependency"),
    }
}

#[test]
fn none_subgrid_and_name_repeat_admit_the_selected_axis_language() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        for (value, expected) in [
            ("NONE", "none"),
            ("SuBgRiD", "subgrid"),
            ("subgrid [] [a] [A]", "subgrid [] [a] [A]"),
            (
                "subgrid repeat(2, [a] []) [z]",
                "subgrid repeat(2, [a] []) [z]",
            ),
            (
                "subgrid repeat(auto-fill, [a])",
                "subgrid repeat(auto-fill, [a])",
            ),
            (r"subgrid [a\ b] [\41 ]", r"subgrid [a\ b] [A]"),
        ] {
            accepted(property, value, expected);
        }
        for value in [
            "subgrid 10px",
            "subgrid repeat(auto-fit, [a])",
            "subgrid repeat(0, [a])",
            "subgrid repeat(2, 10px)",
            "subgrid repeat(2, repeat(2, [a]))",
            "subgrid [auto]",
            "subgrid [span]",
            "subgrid [inherit]",
            "subgrid [default]",
            r"subgrid [\0 ]",
            "subgrid repeat(auto-fill, [a]) repeat(auto-fill, [b])",
        ] {
            invalid(property, value);
        }
    }
}
#[test]
fn calculated_repeat_counts_are_authored_integer_math_not_used_repetition() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        for value in [
            "repeat(calc(2), 10px)",
            "repeat(calc(2), 10px) repeat(auto-fill, minmax(auto, 1fr))",
            "subgrid repeat(calc(2), [a])",
        ] {
            accepted(property, value, value);
        }
        for value in [
            "repeat(calc(2px), 10px)",
            "repeat(0, 10px)",
            "repeat(-1, 10px)",
            "repeat(2.5, 10px)",
            "repeat(2, repeat(2, 10px))",
        ] {
            invalid(property, value);
        }
    }
}
#[test]
fn axis_children_compose_without_confusing_none_with_the_whole_shorthand() {
    for property in [P::GridTemplate, P::Grid] {
        for value in [
            "none / 10px",
            "10px / none",
            "none / none",
            "subgrid [a] / 10px",
            "10px / subgrid repeat(2, [b])",
        ] {
            accepted(property, value, value);
        }
    }
    for (value, expected) in [
        ("auto-flow / none", "auto-flow / none"),
        ("none / auto-flow", "none / auto-flow"),
        ("auto-flow / subgrid [a]", "auto-flow / subgrid [a]"),
        (
            "subgrid [a] / dense auto-flow",
            "subgrid [a] / auto-flow dense",
        ),
    ] {
        accepted(P::Grid, value, expected);
    }
}
#[test]
fn complete_area_string_alternatives_have_repeat_free_explicit_columns() {
    for property in [P::GridTemplate, P::Grid] {
        for value in [
            r#""a a""#,
            r#""a a" "b b" 1fr"#,
            r#"[top] "a a" [end] [start] "b b" 1fr [bottom] / 10px 20px"#,
            r#"[] "auto span" [] / minmax(10px, 1fr) fit-content(20%)"#,
        ] {
            accepted(property, value, value);
        }
        for value in [
            r#""a a" / repeat(2, 10px)"#,
            r#""a a" / none"#,
            r#""a a" / subgrid"#,
            r#""a a" / repeat(auto-fill, 10px)"#,
            r#""a a" repeat(2, 10px)"#,
            r#""a a" /"#,
            r#""""#,
            r#""a a" "b""#,
            r#""a ." ". a""#,
            r#""a" [end] junk"#,
        ] {
            invalid(property, value);
        }
    }
}
#[test]
fn existing_track_auto_flow_precision_and_empty_group_policies_are_controls() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        for value in [
            "[] 10px []",
            "[a] repeat(2, [b] 1fr []) [z]",
            "10px repeat(auto-fill, minmax(auto, 1fr)) 20px",
        ] {
            accepted(property, value, value);
        }
        for value in [
            "[]",
            "[a] [b] 10px",
            "repeat(auto-fill, 1fr) repeat(auto-fit, 10px)",
            "minmax(1fr, 10px)",
            "fit-content(1fr)",
        ] {
            invalid(property, value);
        }
    }
    for property in [P::GridTemplate, P::Grid] {
        accepted(property, "auto / MIN-CONTENT", "auto / min-content");
        accepted(
            property,
            "calc(10px + 5%) / 1e-50fr",
            "calc(5% + 10px) / 0fr",
        );
        invalid(property, "10px");
        invalid(property, "10px /");
    }
    for (value, expected) in [
        ("dense auto-flow / 10px", "auto-flow dense / 10px"),
        ("10px / dense auto-flow", "10px / auto-flow dense"),
        ("auto-flow 20px / 10px", "auto-flow 20px / 10px"),
        ("10px / auto-flow 20px", "10px / auto-flow 20px"),
    ] {
        accepted(P::Grid, value, expected);
    }
    for value in [
        "auto-flow",
        "auto-flow / auto-flow",
        "auto-flow dense dense / 10px",
    ] {
        invalid(P::Grid, value);
    }
    for (value, expected) in [
        ("NORMAL", "normal"),
        ("dense", "dense"),
        ("dense row", "row dense"),
        ("dense column", "column dense"),
    ] {
        let source = checked(P::GridAutoFlow, value, true);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("grid-auto-flow: {expected} !important;")
        );
    }
}

#[test]
fn metadata_has_noninherited_none_initials_and_three_or_six_settable_members() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        let metadata = property.metadata().unwrap();
        assert_eq!(metadata.grammar(), property.grammar());
        let CssPropertyKindRef::Longhand(v) = metadata.kind() else {
            panic!("longhand")
        };
        assert_eq!(v.property().known_property(), property);
        assert!(!v.inherited_by_default());
        let initial = v.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary none initial")
        };
        let values = ordinary(&checked(property, "none", true));
        assert_eq!(Some(initial), values.items()[0].ordinary_value());
    }
    for (property, expected) in [
        (P::GridTemplate, EXPLICIT.as_slice()),
        (P::Grid, GRID.as_slice()),
    ] {
        let metadata = property.metadata().unwrap();
        let CssPropertyKindRef::Shorthand(v) = metadata.kind() else {
            panic!("ordinary shorthand")
        };
        let actual: Vec<_> = v.members().iter().map(|p| p.known_property()).collect();
        assert_eq!(actual.len(), expected.len());
        assert!(
            expected
                .iter()
                .all(|p| actual.iter().filter(|v| *v == p).count() == 1)
        );
        assert_eq!(v.members(), v.settable_members());
        assert!(v.reset_only_members().is_empty());
        assert!(!actual.contains(&P::RowGap));
        assert!(!actual.contains(&P::ColumnGap));
        assert!(!actual.contains(&P::GridRowStart));
        assert!(!actual.contains(&P::FlowTolerance));
    }
}
#[test]
fn ordinary_projection_preserves_occurrences_and_source_defined_omission_defaults() {
    for (property, value) in ROLES {
        for source in fronts(property, value) {
            ordinary(&source);
        }
    }
    for property in [P::GridTemplate, P::Grid] {
        for (value, expected_areas) in [
            ("none", "none"),
            ("10px / 20px", "none"),
            (r#""a a" "b b" 1fr / 10px 20px"#, r#""a a" "b b""#),
        ] {
            for source in fronts(property, value) {
                let values = ordinary(&source);
                assert_eq!(
                    existing_terminal_output(&values, P::GridTemplateAreas),
                    expected_areas
                );
                if property == P::Grid {
                    assert_eq!(existing_terminal_output(&values, P::GridAutoRows), "auto");
                    assert_eq!(
                        existing_terminal_output(&values, P::GridAutoColumns),
                        "auto"
                    );
                    assert_eq!(existing_terminal_output(&values, P::GridAutoFlow), "normal");
                }
            }
        }
    }
    for (value, rows, columns, flow) in [
        ("auto-flow / 10px", "auto", "auto", "row"),
        ("dense auto-flow 20px / 10px", "20px", "auto", "row dense"),
        ("10px / auto-flow", "auto", "auto", "column"),
        (
            "10px / dense auto-flow 20px",
            "auto",
            "20px",
            "column dense",
        ),
    ] {
        for source in fronts(P::Grid, value) {
            let values = ordinary(&source);
            assert_eq!(
                existing_terminal_output(&values, P::GridTemplateAreas),
                "none"
            );
            assert_eq!(existing_terminal_output(&values, P::GridAutoRows), rows);
            assert_eq!(
                existing_terminal_output(&values, P::GridAutoColumns),
                columns
            );
            assert_eq!(existing_terminal_output(&values, P::GridAutoFlow), flow);
        }
    }
}
#[test]
fn every_css_wide_keyword_projects_to_all_one_three_or_six_terminals() {
    for (property, _) in ROLES {
        for (text, keyword) in GLOBALS {
            for source in fronts(property, text) {
                let values = completed(&source);
                assert_eq!(names(&values), members(property));
                for item in values.items() {
                    assert_source(item, &source);
                    assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                    assert!(item.ordinary_value().is_none());
                    assert!(item.replacement_components().is_none());
                }
            }
        }
    }
}
#[test]
fn whole_value_pending_reentry_reuses_original_grammar_order_importance_and_origins() {
    for (property, ordinary_text) in ROLES {
        for pending_text in ["var(--grid)", "env(grid)", "attr(data-grid *)"] {
            for source in fronts(property, pending_text) {
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("one whole pending value")
                };
                assert!(handle.source().same_occurrence(&source));
                for invalid_text in ["not-a-grid-value", "var(--still-pending)"] {
                    for _ in 0..2 {
                        let error = handle
                            .reenter(parse_component_values(invalid_text).unwrap())
                            .unwrap_err();
                        if invalid_text.starts_with("var") {
                            assert!(matches!(
                                error.kind(),
                                CssExpansionErrorKind::ResidualSubstitution
                            ));
                        } else {
                            assert!(matches!(
                                error.kind(),
                                CssExpansionErrorKind::InvalidReplacement(_)
                            ));
                        }
                        assert!(handle.source().same_occurrence(&source));
                    }
                }
                let new_branch = match property {
                    P::GridTemplateRows | P::GridTemplateColumns => "subgrid repeat(calc(2), [a])",
                    _ => r#""a a" "b b" 1fr / 10px 20px"#,
                };
                for replacement_text in [ordinary_text, new_branch, "none"] {
                    let replacement = parse_component_values(replacement_text).unwrap();
                    let before = replacement.clone();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("completed retry")
                    };
                    assert_eq!(names(&values), members(property));
                    for item in values.items() {
                        assert_source(item, &source);
                        assert!(item.ordinary_value().is_some());
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                    assert_eq!(replacement, before);
                }
                for (text, keyword) in GLOBALS {
                    let replacement = parse_component_values(text).unwrap();
                    let CssContributions::Longhands(values) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("completed global retry")
                    };
                    assert_eq!(names(&values), members(property));
                    for item in values.items() {
                        assert_source(item, &source);
                        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
            }
        }
    }
}

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    // Public origin mappings cover comments; public closing_origin covers
    // structural delimiters. Never inspect private first_implicit_origin.
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("retained implicit closing origin")
}
fn assert_closure_error(error: &CssPropertyValueParseError, origin: &CssValueOrigin) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
}
#[test]
fn all_four_checked_roles_reject_original_implicit_comments_functions_and_blocks() {
    for (property, value) in ROLES {
        let function = match property {
            P::GridTemplateRows | P::GridTemplateColumns => "minmax(10px, 1fr",
            _ => "10px / minmax(20px, 1fr",
        };
        let block = match property {
            P::GridTemplateRows | P::GridTemplateColumns => "10px [last",
            _ => "10px / 20px [last",
        };
        for text in [
            format!("{value}/*"),
            "initial/*".into(),
            "var(--grid)/*".into(),
            function.into(),
            block.into(),
        ] {
            let components = parse_component_values(&text).unwrap();
            let before = components.clone();
            let origin = implicit_origin(&components);
            for result in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                ),
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                ),
            ] {
                assert_closure_error(&result.unwrap_err(), &origin);
            }
            assert_eq!(components, before);
            let source = checked(property, "var(--grid)", true);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            // Residual substitution has documented priority over reentry grammar.
            if !text.starts_with("var") {
                for _ in 0..2 {
                    let error = handle.reenter(components.clone()).unwrap_err();
                    let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                        panic!("original closure rejection")
                    };
                    assert_closure_error(error, &origin);
                }
                assert!(
                    handle
                        .reenter(parse_component_values(value).unwrap())
                        .is_ok()
                );
                assert!(handle.source().same_occurrence(&source));
            }
        }
        for text in [
            format!("{value}/**/"),
            "initial/**/".into(),
            "var(--grid)/**/".into(),
            format!("{function})"),
            format!("{block}]"),
        ] {
            checked(property, &text, false);
            checked(property, &text, true);
        }
    }
}
#[test]
fn checked_invalid_repeat_count_maps_to_the_original_responsible_token() {
    for property in [P::GridTemplateRows, P::GridTemplateColumns] {
        let components = parse_component_values("/*😀*/repeat(0, 10px)").unwrap();
        let serialized = components.serialize().unwrap();
        let expected = serialized
            .origin_at(serialized.as_css().find("0,").unwrap())
            .unwrap()
            .clone();
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Normal,
            ),
        ] {
            assert_eq!(result.unwrap_err().origin(), &expected);
        }
    }
}

#[test]
fn normalization_preserves_duplicate_order_contexts_and_exact_one_three_six_accounting() {
    let css = "@media screen{.a{grid:none!important;grid-template:10px / 20px;grid-template-rows:10px;grid-template-columns:20px;grid:var(--grid);grid:unset}}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    // Four ordinary occurrences: 6+3+1+1; one whole pending handle costs one;
    // six global terminals. The public normalization contract totals 18.
    let exact = CssNormalizationLimits::try_new(1, 2, 6, 18).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|v| match v {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (index, (item, property)) in declarations
        .iter()
        .zip([
            P::Grid,
            P::GridTemplate,
            P::GridTemplateRows,
            P::GridTemplateColumns,
            P::Grid,
            P::Grid,
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property);
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(names(values), members(property));
                for terminal in values.items() {
                    assert_source(terminal, item.source());
                }
            }
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 4);
                assert!(handle.source().same_occurrence(item.source()));
            }
            _ => panic!("Grid expansion"),
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(1, 2, 6, 17).unwrap(),
            CssNormalizationResource::Contributions,
            17,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 5, 18).unwrap(),
            CssNormalizationResource::Declarations,
            5,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(5));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[5].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn recoverable_browser_comments_retain_diagnostics_and_source_in_normalized_reports() {
    for (property, value) in ROLES {
        let report = parse_sheet(&format!(".a{{{}:{value}/*", property.canonical_name()));
        assert!(!report.is_clean());
        let before = report.clone();
        let count = match property {
            P::Grid => 6,
            P::GridTemplate => 3,
            _ => 1,
        };
        let exact = CssNormalizationLimits::try_new(0, 1, 1, count).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|v| match v {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("retained recovered occurrence")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("retained ordinary value")
        };
        assert_eq!(names(values), members(property));
        for terminal in values.items() {
            assert_source(terminal, item.source());
        }
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, count - 1).unwrap(),
        )
        .unwrap_err();
        assert!(error.declaration().unwrap().same_occurrence(item.source()));
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}

#[test]
fn every_new_typed_branch_obeys_atomic_byte_limits_and_reusable_output() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for (property, value) in [
        (P::GridTemplateRows, "none"),
        (P::GridTemplateColumns, "subgrid [] repeat(calc(2), [a])"),
        (P::GridTemplate, r#""a a" "b b" 1fr / 10px 20px"#),
        (P::Grid, "none / auto-flow dense"),
    ] {
        let source = checked(property, value, true);
        let before = source.clone();
        let exact = Limits::new(usize::MAX, usize::MAX, value.len());
        assert_eq!(typed_output(&source, exact).unwrap(), value);
        for _ in 0..2 {
            assert_eq!(
                typed_output(
                    &source,
                    Limits::new(usize::MAX, usize::MAX, value.len() - 1)
                )
                .unwrap_err()
                .kind(),
                Kind::ByteLimit
            );
            assert_eq!(source, before);
        }
        assert_eq!(typed_output(&source, exact).unwrap(), value);
    }
}
#[test]
fn existing_aggregate_work_costs_and_sheet_sibling_byte_limits_remain_atomic() {
    use CssSpecifiedRuleSerializationErrorKind as RuleKind;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    // Accepted grid_aggregate_serialization.rs:124-158 independently fixes five
    // semantic visits for the represented aggregate. The public declaration
    // contract (declaration_serialization.rs:205-210) adds declaration + name.
    // No CssDeclarationList public specified serializer exists at this basis.
    let declaration = checked(P::Grid, "10px / 20px", true);
    let declaration_before = declaration.clone();
    let value_text = "10px / 20px";
    let declaration_text = "grid: 10px / 20px !important;";
    assert_eq!(
        typed_output(&declaration, Limits::new(5, 5, value_text.len())).unwrap(),
        value_text
    );
    assert_eq!(
        declaration
            .to_specified_css_with_limits(Limits::new(7, 7, declaration_text.len()))
            .unwrap(),
        declaration_text
    );
    for (value_limits, declaration_limits, kind) in [
        (
            Limits::new(4, 5, value_text.len()),
            Limits::new(6, 7, declaration_text.len()),
            Kind::InputNodeLimit,
        ),
        (
            Limits::new(5, 4, value_text.len()),
            Limits::new(7, 6, declaration_text.len()),
            Kind::ProjectionNodeLimit,
        ),
    ] {
        for _ in 0..2 {
            assert_eq!(
                typed_output(&declaration, value_limits).unwrap_err().kind(),
                kind
            );
            assert_eq!(
                declaration
                    .to_specified_css_with_limits(declaration_limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(declaration, declaration_before);
        }
    }
    assert_eq!(declaration.to_specified_css().unwrap(), declaration_text);

    // Public CssSheet composition supplies one shared final byte budget and a
    // newline between top-level rules (specified_rule_serialization.rs:296-299).
    // The fixture's canonical text fixes that exact budget independently. Do
    // not trial-fit whole-sheet work counts: selector and rule graph costs stay
    // with existing public graph owners and functional implementation coverage.
    let report = parse_sheet(".a{grid:10px / 20px}.b{grid:10px / 20px}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = ".a { grid: 10px / 20px; }\n.b { grid: 10px / 20px; }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(error.kind(), RuleKind::Resource(Kind::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn new_subgrid_and_area_siblings_share_final_sheet_output_budget_without_mutation() {
    use CssSpecifiedRuleSerializationErrorKind as RuleKind;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        r#".a{grid-template-rows:subgrid repeat(calc(2), [a])!important}.b{grid-template:"a a" / 10px 20px}"#,
    );
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected = r#".a { grid-template-rows: subgrid repeat(calc(2), [a]) !important; }
.b { grid-template: "a a" / 10px 20px; }"#;
    assert_eq!(sheet.to_specified_css().unwrap(), expected);
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|r| r.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(error.kind(), RuleKind::Resource(Kind::ByteLimit));
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(
        sheet
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
}
#[test]
fn independent_supported_color_and_implicit_grid_controls_keep_intrinsic_contracts() {
    for (property, value, expected) in [
        (P::Color, "currentcolor", "currentcolor"),
        (P::GridAutoRows, "auto", "auto"),
        (P::GridAutoColumns, "auto", "auto"),
        (P::GridAutoFlow, "normal", "normal"),
    ] {
        let source = checked(property, value, true);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{}: {expected} !important;", property.canonical_name())
        );
        let values = completed(&source);
        assert_eq!(names(&values), [property]);
        assert_source(&values.items()[0], &source);
    }
}

#![forbid(unsafe_code)]

//! Logical 1 WD 2025-12-04 §4.7 supplies logical side assignments; Box 3,
//! Backgrounds 3, Position 3 and Scroll Snap 1 supply physical assignments.
//! Complementary reset membership remains source-limited. Surgeist's bounded
//! policy selects four sides in the authored mode, with no complementary resets;
//! CSS-wide values select physical sides. Frozen WebKit 73aa6c89 corroborates
//! physical membership, CSS-wide dispatch and quad repetition, not logical mode.

use surgeist_css::*;

struct Family {
    name: &'static str,
    physical: [&'static str; 4],
    logical: [&'static str; 4],
    tokens: [&'static str; 4],
    initial: &'static str,
    invalid: &'static str,
}

const FAMILIES: [Family; 8] = [
    Family {
        name: "border-color",
        physical: [
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ],
        logical: [
            "border-block-start-color",
            "border-inline-start-color",
            "border-block-end-color",
            "border-inline-end-color",
        ],
        tokens: ["red", "green", "blue", "black"],
        initial: "currentcolor",
        invalid: "1px",
    },
    Family {
        name: "border-style",
        physical: [
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ],
        logical: [
            "border-block-start-style",
            "border-inline-start-style",
            "border-block-end-style",
            "border-inline-end-style",
        ],
        tokens: ["solid", "dashed", "dotted", "double"],
        initial: "none",
        invalid: "1px",
    },
    Family {
        name: "border-width",
        physical: [
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ],
        logical: [
            "border-block-start-width",
            "border-inline-start-width",
            "border-block-end-width",
            "border-inline-end-width",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "medium",
        invalid: "-1px",
    },
    Family {
        name: "inset",
        physical: ["top", "right", "bottom", "left"],
        logical: [
            "inset-block-start",
            "inset-inline-start",
            "inset-block-end",
            "inset-inline-end",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "auto",
        invalid: "1fr",
    },
    Family {
        name: "margin",
        physical: ["margin-top", "margin-right", "margin-bottom", "margin-left"],
        logical: [
            "margin-block-start",
            "margin-inline-start",
            "margin-block-end",
            "margin-inline-end",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "0",
        invalid: "1fr",
    },
    Family {
        name: "padding",
        physical: [
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ],
        logical: [
            "padding-block-start",
            "padding-inline-start",
            "padding-block-end",
            "padding-inline-end",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "0",
        invalid: "-1px",
    },
    Family {
        name: "scroll-margin",
        physical: [
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ],
        logical: [
            "scroll-margin-block-start",
            "scroll-margin-inline-start",
            "scroll-margin-block-end",
            "scroll-margin-inline-end",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "0",
        invalid: "1%",
    },
    Family {
        name: "scroll-padding",
        physical: [
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ],
        logical: [
            "scroll-padding-block-start",
            "scroll-padding-inline-start",
            "scroll-padding-block-end",
            "scroll-padding-inline-end",
        ],
        tokens: ["1px", "2px", "3px", "4px"],
        initial: "auto",
        invalid: "-1px",
    },
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap()
}
fn declaration(name: &str, value: &str) -> CssDeclaration {
    let text = format!("{name}:{value}!important");
    let report = parse_style_attribute(&text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration: {text}")
    };
    source.clone()
}
fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}
fn terminals(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap_or_else(|error| {
            panic!(
                "{} expansion: {error:?}",
                source.known().unwrap().property().canonical_name()
            )
        })
    else {
        panic!("ordinary longhands")
    };
    values
}
fn one_value(name: &str, value: &str) -> CssLonghandValue {
    let values = terminals(&declaration(name, value));
    assert_eq!(values.items().len(), 1);
    values.items()[0].ordinary_value().unwrap().clone()
}
// Scalar equality deliberately includes diagnostic provenance. Compare the
// selected length's exact literal structure separately from its real origin.
fn scroll_margin_length(value: &CssLonghandValue) -> Option<&CssSpecifiedLength> {
    match value.view() {
        CssLonghandValueRef::ScrollMarginTop(value)
        | CssLonghandValueRef::ScrollMarginRight(value)
        | CssLonghandValueRef::ScrollMarginBottom(value)
        | CssLonghandValueRef::ScrollMarginLeft(value)
        | CssLonghandValueRef::ScrollMarginBlockStart(value)
        | CssLonghandValueRef::ScrollMarginInlineStart(value)
        | CssLonghandValueRef::ScrollMarginBlockEnd(value)
        | CssLonghandValueRef::ScrollMarginInlineEnd(value) => Some(value),
        _ => None,
    }
}
fn scroll_padding(value: &CssLonghandValue) -> Option<&CssScrollPaddingValue> {
    match value.view() {
        CssLonghandValueRef::ScrollPaddingTop(value)
        | CssLonghandValueRef::ScrollPaddingRight(value)
        | CssLonghandValueRef::ScrollPaddingBottom(value)
        | CssLonghandValueRef::ScrollPaddingLeft(value)
        | CssLonghandValueRef::ScrollPaddingBlockStart(value)
        | CssLonghandValueRef::ScrollPaddingInlineStart(value)
        | CssLonghandValueRef::ScrollPaddingBlockEnd(value)
        | CssLonghandValueRef::ScrollPaddingInlineEnd(value) => Some(value),
        _ => None,
    }
}
fn assert_literal_payload(actual: &CssComponentValue, expected: &CssComponentValue) {
    match (actual.view(), expected.view()) {
        (
            CssComponentValueRef::Token(CssValueTokenRef::Number(actual)),
            CssComponentValueRef::Token(CssValueTokenRef::Number(expected)),
        )
        | (
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(actual)),
            CssComponentValueRef::Token(CssValueTokenRef::Percentage(expected)),
        ) => {
            assert_eq!(actual.representation(), expected.representation());
            assert_eq!(actual.kind(), expected.kind());
        }
        (
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: actual,
                unit: actual_unit,
            }),
            CssComponentValueRef::Token(CssValueTokenRef::Dimension {
                number: expected,
                unit: expected_unit,
            }),
        ) => {
            assert_eq!(actual.representation(), expected.representation());
            assert_eq!(actual.kind(), expected.kind());
            assert_eq!(actual_unit, expected_unit);
        }
        _ => panic!("selected literal token/unit shape differs"),
    }
}
fn assert_semantic_payload(actual: &CssLonghandValue, expected: &CssLonghandValue) {
    assert_eq!(actual.property(), expected.property());
    if let Some(actual_padding) = scroll_padding(actual) {
        match (
            actual_padding,
            scroll_padding(expected).expect("scroll-padding domain"),
        ) {
            (CssScrollPaddingValue::Auto, CssScrollPaddingValue::Auto) => {}
            (
                CssScrollPaddingValue::LengthPercentage(actual),
                CssScrollPaddingValue::LengthPercentage(expected),
            ) => {
                assert!(actual.calculation().is_none());
                assert!(expected.calculation().is_none());
                assert_literal_payload(
                    actual.literal_component().expect("ordinary exact padding"),
                    expected
                        .literal_component()
                        .expect("ordinary exact padding"),
                );
            }
            _ => panic!("selected Auto/LengthPercentage branch differs"),
        }
        return;
    }
    match (scroll_margin_length(actual), scroll_margin_length(expected)) {
        (Some(actual), Some(expected)) => {
            assert!(actual.calculation().is_none());
            assert!(expected.calculation().is_none());
            assert_literal_payload(
                actual.literal_component().expect("ordinary exact length"),
                expected.literal_component().expect("ordinary exact length"),
            );
        }
        (None, None) => assert_eq!(actual, expected),
        _ => panic!("selected longhand domain differs"),
    }
}
fn scroll_scalar_origin(value: &CssLonghandValue) -> Option<&CssValueOrigin> {
    scroll_margin_length(value)
        .map(CssSpecifiedLength::origin)
        .or_else(|| match scroll_padding(value) {
            Some(CssScrollPaddingValue::LengthPercentage(value)) => Some(value.origin()),
            Some(CssScrollPaddingValue::Auto) | None => None,
            Some(_) => panic!("unexpected scroll-padding branch"),
        })
}
fn assigned(arity: usize) -> [usize; 4] {
    match arity {
        1 => [0, 0, 0, 0],
        2 => [0, 1, 0, 1],
        3 => [0, 1, 2, 1],
        4 => [0, 1, 2, 3],
        _ => panic!("quad arity"),
    }
}
fn authored(family: &Family, logical: bool, arity: usize) -> String {
    format!(
        "{}{}",
        if logical { "logical " } else { "" },
        family.tokens[..arity].join(" ")
    )
}
fn assert_values(
    family: &Family,
    logical: bool,
    arity: usize,
    source: &CssDeclaration,
    values: &CssLonghandContributions,
    replacement: Option<&CssComponentValues>,
) {
    let names = if logical {
        family.logical
    } else {
        family.physical
    };
    assert_eq!(
        values.items().len(),
        4,
        "{} has no complementary resets",
        family.name
    );
    for ((item, name), token) in values.items().iter().zip(names).zip(assigned(arity)) {
        assert_eq!(item.property(), grammar(name).target_property());
        let actual = item.ordinary_value().expect("ordinary selected side");
        assert_semantic_payload(actual, &one_value(name, family.tokens[token]));
        if let Some(origin) = scroll_scalar_origin(actual) {
            let components = replacement.unwrap_or_else(|| source.value_components());
            let leaves = components
                .items()
                .iter()
                .filter(|component| {
                    matches!(
                        component.view(),
                        CssComponentValueRef::Token(
                            CssValueTokenRef::Number(_)
                                | CssValueTokenRef::Dimension { .. }
                                | CssValueTokenRef::Percentage(_)
                        )
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(leaves.len(), arity);
            assert_eq!(origin, leaves[token].origin());
        }
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().position(), source.position());
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert_eq!(item.replacement_components(), replacement);
    }
}

fn expand_family(family: &Family) {
    for logical in [false, true] {
        for arity in 1..=4 {
            let value = authored(family, logical, arity);
            for source in [
                declaration(family.name, &value),
                checked(family.name, &value),
            ] {
                assert_eq!(
                    source.value_components().serialize().unwrap().as_css(),
                    value
                );
                assert_values(family, logical, arity, &source, &terminals(&source), None);
            }
        }
    }
}

macro_rules! family_expansion_test {
    ($test:ident, $index:expr) => {
        #[test]
        fn $test() {
            expand_family(&FAMILIES[$index]);
        }
    };
}
family_expansion_test!(
    border_color_expands_all_arities_in_only_the_selected_mode,
    0
);
family_expansion_test!(
    border_style_expands_all_arities_in_only_the_selected_mode,
    1
);
family_expansion_test!(
    border_width_expands_all_arities_in_only_the_selected_mode,
    2
);
family_expansion_test!(inset_expands_all_arities_in_only_the_selected_mode, 3);
family_expansion_test!(margin_expands_all_arities_in_only_the_selected_mode, 4);
family_expansion_test!(padding_expands_all_arities_in_only_the_selected_mode, 5);
family_expansion_test!(
    scroll_margin_expands_all_arities_in_only_the_selected_mode,
    6
);
family_expansion_test!(
    scroll_padding_expands_all_arities_in_only_the_selected_mode,
    7
);

#[test]
fn shorthand_metadata_becomes_available_without_using_new_mode_metadata_api() {
    for family in &FAMILIES {
        let metadata = grammar(family.name).metadata().unwrap();
        assert_eq!(metadata.grammar(), grammar(family.name));
    }
}

#[test]
fn all_selected_longhands_keep_exact_intrinsic_initial_and_noninheritance() {
    for family in &FAMILIES {
        for name in family.physical.into_iter().chain(family.logical) {
            let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
            else {
                panic!("{name} terminal metadata")
            };
            assert_eq!(
                metadata.property().known_property(),
                grammar(name).target_property()
            );
            assert!(!metadata.inherited_by_default());
            let initial = metadata.initial_value();
            let CssInitialValueRef::Value(value) = initial.view() else {
                panic!("fixed {name} initial")
            };
            assert_semantic_payload(value, &one_value(name, family.initial));
            if let Some(length) = scroll_margin_length(value) {
                assert!(matches!(length.origin(), CssValueOrigin::Programmatic));
            }
        }
    }
}

#[test]
fn all_css_wide_keywords_target_physical_sides_without_complementary_resets() {
    for family in &FAMILIES {
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            for source in [declaration(family.name, text), checked(family.name, text)] {
                let values = terminals(&source);
                assert_eq!(values.items().len(), 4);
                for (item, name) in values.items().iter().zip(family.physical) {
                    assert_eq!(item.property(), grammar(name).target_property());
                    assert!(
                        matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                    );
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert!(item.replacement_components().is_none());
                }
            }
        }
    }
}

#[test]
fn reusable_pending_handle_selects_replacement_mode_and_retains_both_origins() {
    for family in &FAMILIES {
        let source = declaration(family.name, "var(--edges)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending shorthand")
        };
        assert!(handle.source().same_occurrence(&source));
        for logical in [true, false, true] {
            for arity in 1..=4 {
                let replacement =
                    parse_component_values(&authored(family, logical, arity)).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("reentered terminals")
                };
                assert_values(family, logical, arity, &source, &values, Some(&replacement));
                assert_eq!(
                    values.items()[0]
                        .source()
                        .value_components()
                        .serialize()
                        .unwrap()
                        .as_css(),
                    "var(--edges)"
                );
            }
        }
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("global replacement terminals")
            };
            assert_eq!(values.items().len(), 4);
            for (item, name) in values.items().iter().zip(family.physical) {
                assert_eq!(item.property(), grammar(name).target_property());
                assert!(
                    matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn invalid_and_residual_reentry_are_atomic_and_keep_pending_source_usable() {
    for family in &FAMILIES {
        let source = declaration(family.name, "var(--edges)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending shorthand")
        };
        for invalid in [
            family.invalid,
            "logical",
            "initial 1px",
            "logical logical 1px",
        ] {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        assert_eq!(
            handle
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        assert!(handle.source().same_occurrence(&source));
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            "var(--edges)"
        );
    }
}

#[test]
fn parser_drops_invalid_quad_atomically_between_valid_neighbors() {
    for family in &FAMILIES {
        let invalids = [
            family.invalid.to_owned(),
            "logical".to_owned(),
            family.tokens.repeat(2).join(" "),
            format!("{} logical", family.tokens[0]),
        ];
        for invalid in invalids {
            let text = format!("color:red;{}:{invalid};color:blue", family.name);
            let report = parse_style_attribute(&text);
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid declaration: {text}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
            assert_eq!(report.syntax().len(), 2);
            assert!(validate_style_attribute(&text).is_err());
            assert!(
                parse_property_value_for_grammar(
                    grammar(family.name),
                    parse_component_values(&invalid).unwrap(),
                    CssImportance::Normal
                )
                .is_err()
            );
            let syntax = report.syntax().as_slice();
            assert_eq!(
                syntax[0].value_components().serialize().unwrap().as_css(),
                "red"
            );
            assert_eq!(
                syntax[1].value_components().serialize().unwrap().as_css(),
                "blue"
            );
        }
    }
}

#[test]
fn normalization_preserves_mode_interleaving_importance_sources_and_cumulative_limits() {
    for family in &FAMILIES {
        let text = format!(
            ".a{{{}:{};{}:{}!important;{}:{};{}:logical {};{}:var(--edges)}}",
            family.logical[0],
            family.tokens[3],
            family.name,
            family.tokens[0],
            family.physical[0],
            family.tokens[3],
            family.name,
            family.tokens[1],
            family.name
        );
        let report = parse_sheet(&text);
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let preserved = report.syntax().clone();
        let normalized = normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(1, 1, 5, 11).unwrap(),
        )
        .unwrap();
        let declarations = normalized
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(declarations.len(), 5);
        let properties = [
            family.logical[0],
            family.name,
            family.physical[0],
            family.name,
            family.name,
        ];
        for (order, (item, name)) in declarations.iter().zip(properties).enumerate() {
            assert_eq!(item.order(), order);
            assert_eq!(
                item.source().known().unwrap().property(),
                grammar(name).target_property()
            );
            assert_eq!(
                item.source().importance(),
                if order == 1 {
                    CssImportance::Important
                } else {
                    CssImportance::Normal
                }
            );
        }
        for (index, names) in [(1, family.physical), (3, family.logical)] {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                declarations[index].expansion()
            else {
                panic!("quad group")
            };
            assert_eq!(values.items().len(), 4);
            for (item, name) in values.items().iter().zip(names) {
                assert_eq!(item.property(), grammar(name).target_property());
                assert!(item.source().same_occurrence(declarations[index].source()));
            }
        }
        assert!(matches!(
            declarations[4].expansion(),
            CssExpansion::Pending(_)
        ));
        for (declarations_limit, contributions_limit, resource, limit, order) in [
            (5, 10, CssNormalizationResource::Contributions, 10, 4),
            (4, 11, CssNormalizationResource::Declarations, 4, 4),
        ] {
            let error = normalize_sheet_with_limits(
                report.syntax(),
                CssNormalizationLimits::try_new(1, 1, declarations_limit, contributions_limit)
                    .unwrap(),
            )
            .unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit }
            );
            assert_eq!(error.declaration_order(), Some(order));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[order].source())
            );
            assert_eq!(
                report.syntax(),
                &preserved,
                "normalization failure is atomic"
            );
        }
    }
}

#[test]
fn existing_standalone_quad_writers_keep_mode_output_and_atomic_byte_limits() {
    // Distinct components keep the same spelling in either retaining or
    // coalescing writers. Existing scroll writers also expose quad coalescing.
    for family in &FAMILIES {
        for logical in [false, true] {
            for arity in 1..=4 {
                let text = authored(family, logical, arity);
                let source = declaration(family.name, &text);
                assert_eq!(canonical(&source), text);
                let preserved = source.value_components().clone();
                assert_eq!(
                    canonical_with_limits(
                        &source,
                        CssSpecifiedValueSerializationLimits::new(256, usize::MAX, text.len())
                    )
                    .unwrap(),
                    text
                );
                assert!(
                    canonical_with_limits(
                        &source,
                        CssSpecifiedValueSerializationLimits::new(256, usize::MAX, text.len() - 1)
                    )
                    .is_err()
                );
                assert_eq!(source.value_components(), &preserved);
            }
            let text = format!(
                "{}{}",
                if logical { "logical " } else { "" },
                [family.tokens[0]; 4].join(" ")
            );
            if family.name.starts_with("scroll-") {
                let expected = format!(
                    "{}{}",
                    if logical { "logical " } else { "" },
                    family.tokens[0]
                );
                assert_eq!(canonical(&declaration(family.name, &text)), expected);
            }
        }
    }
}
fn canonical(source: &CssDeclaration) -> String {
    canonical_with_limits(source, CssSpecifiedValueSerializationLimits::default()).unwrap()
}
fn canonical_with_limits(
    source: &CssDeclaration,
    limits: CssSpecifiedValueSerializationLimits,
) -> Result<String, CssSpecifiedValueSerializationError> {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::BorderColor(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::BorderStyle(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::BorderWidth(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Inset(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Margin(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::Padding(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::ScrollMargin(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        CssKnownPropertyValueRef::ScrollPadding(value) => {
            value.value().serialize_specified_with_limits(limits)
        }
        _ => panic!("quad provider"),
    }
}

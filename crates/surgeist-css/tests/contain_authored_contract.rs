#![forbid(unsafe_code)]
//! Containment2 WD20220917 §1.2/§2, Values3 §2.2, and the accepted
//! represented contain provider contract supply this independent oracle.
//! Specified strict/content remain authored keywords; computed containment,
//! counters, layout, painting and content-visibility execution stay downstream.
//! A component list costs one aggregate plus each keyword; enum carriers are
//! transparent. Declaration/name add two nodes, separators charge UTF-8 bytes.
//! Adopt by copying into tests/: the helper below is existing test-only support.
//! No future Style variant or generated Contain longhand variant is named.

#[path = "common/authored_property.rs"]
mod authored_property;
use authored_property::{ParserFront, assert_source, checked_components, invalid};
use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::Contain;
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const STATES: [&str; 18] = [
    "none",
    "strict",
    "content",
    "size",
    "layout",
    "style",
    "paint",
    "size layout",
    "size style",
    "size paint",
    "layout style",
    "layout paint",
    "style paint",
    "size layout style",
    "size layout paint",
    "size style paint",
    "layout style paint",
    "size layout style paint",
];
fn permutations(words: &[&str]) -> Vec<String> {
    if words.is_empty() {
        return vec![String::new()];
    }
    let mut values = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let remaining: Vec<_> = words
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (i != index).then_some(*v))
            .collect();
        for tail in permutations(&remaining) {
            values.push(if tail.is_empty() {
                (*word).to_owned()
            } else {
                format!("{word} {tail}")
            });
        }
    }
    values
}
fn containment(source: &CssDeclaration) -> &CssContain {
    let CssKnownPropertyValueRef::Contain(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("existing typed contain wrapper")
    };
    value.containment()
}
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one ordinary longhand")
    };
    assert_terminals(&values, source);
    values
}
fn assert_terminals(values: &CssLonghandContributions, source: &CssDeclaration) {
    let [item] = values.items() else {
        panic!("contain has exactly one terminal and no resets")
    };
    assert_eq!(item.property(), PROPERTY);
    assert_source(item, source);
}
fn closure_origin(values: &CssComponentValues) -> CssValueOrigin {
    for component in values.items() {
        let origin = match component.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = origin {
            return origin.clone();
        }
    }
    let serialized = values.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original implicit EOF mapping")
}
macro_rules! style_case {
    ($name:ident,$canonical:literal) => {
        #[test]
        fn $name() {
            let words: Vec<_> = $canonical.split_whitespace().collect();
            for input in permutations(&words) {
                for front in FRONTS {
                    let source = front.valid(PROPERTY, &input, $canonical);
                    let CssContain::Components(values) = containment(&source) else {
                        panic!("authored group")
                    };
                    assert_eq!(values.components().len(), words.len());
                }
            }
        }
    };
}
style_case!(style_is_an_ordinary_containment_component, "style");
style_case!(size_style_all_orders_preserve_full_group, "size style");
style_case!(layout_style_all_orders_preserve_full_group, "layout style");
style_case!(style_paint_all_orders_preserve_full_group, "style paint");
style_case!(
    size_layout_style_all_orders_preserve_full_group,
    "size layout style"
);
style_case!(
    size_style_paint_all_orders_preserve_full_group,
    "size style paint"
);
style_case!(
    layout_style_paint_all_orders_preserve_full_group,
    "layout style paint"
);
style_case!(
    all_four_containment_roles_admit_every_permutation,
    "size layout style paint"
);

#[test]
fn represented_keyword_and_component_controls_preserve_authored_order_and_alias_identity() {
    for (text, value) in [
        ("none", CssContain::None),
        ("strict", CssContain::Strict),
        ("content", CssContain::Content),
    ] {
        for front in FRONTS {
            assert_eq!(containment(&front.valid(PROPERTY, text, text)), &value);
        }
    }
    for canonical in [
        "size",
        "layout",
        "paint",
        "size layout",
        "size paint",
        "layout paint",
        "size layout paint",
    ] {
        let words: Vec<_> = canonical.split_whitespace().collect();
        for input in permutations(&words) {
            for front in FRONTS {
                let source = front.valid(PROPERTY, &input, canonical);
                let CssContain::Components(values) = containment(&source) else {
                    panic!("represented group")
                };
                let expected: Vec<_> = input
                    .split_whitespace()
                    .map(|word| match word {
                        "size" => CssContainComponent::Size,
                        "layout" => CssContainComponent::Layout,
                        "paint" => CssContainComponent::Paint,
                        _ => unreachable!(),
                    })
                    .collect();
                assert_eq!(values.components(), expected);
            }
        }
    }
    assert!(CssContainComponentList::try_new(vec![]).is_none());
    assert!(
        CssContainComponentList::try_new(vec![
            CssContainComponent::Size,
            CssContainComponent::Size
        ])
        .is_none()
    );
}

#[test]
fn grammar_rejects_duplicates_exclusive_mixtures_and_values_outside_the_selected_pin() {
    for input in [
        "",
        "none size",
        "strict paint",
        "content style",
        "none none",
        "size size",
        "style style",
        "size layout style paint size",
        "size, paint",
        "inline-size",
        "inline-size style",
        "auto",
        "normal",
        "inherit paint",
        "style initial",
        "1",
        "\"style\"",
        "var()",
    ] {
        invalid(PROPERTY, input);
    }
}

#[test]
fn escaped_style_tokens_preserve_original_components_source_coordinates_and_importance() {
    for (input, canonical) in [
        (r"\73 tyle", "style"),
        (r"PAINT/**/\73 tyle SIZE", "size style paint"),
    ] {
        for front in FRONTS {
            let source = front.valid(PROPERTY, input, canonical);
            let before = source.clone();
            assert_eq!(source.importance(), CssImportance::Important);
            let values = contributions(&source);
            assert!(values.items()[0].ordinary_value().is_some());
            assert_eq!(source, before);
        }
    }
    let input = r"/*😀*/\73 tyle";
    let components = parse_component_values(input).unwrap();
    let token=components.items().iter().find(|v|matches!(v.view(),CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) if name=="style")).unwrap();
    let CssValueOrigin::Parsed(origin) = token.origin() else {
        panic!("original escaped style origin")
    };
    assert_eq!(origin.source().as_str(), input);
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    let range =
        origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value();
    assert_eq!(&origin.source().as_str()[range], r"\73 tyle");
    for grammar in [false, true] {
        let source = checked_components(
            PROPERTY,
            components.clone(),
            grammar,
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(source.value_components(), &components);
        assert_eq!(containment(&source).serialize_specified().unwrap(), "style");
    }
}

#[test]
fn intrinsic_metadata_is_noninherited_with_an_ordinary_initial_and_one_terminal() {
    let metadata = PROPERTY.metadata().expect("intrinsic contain metadata");
    let CssPropertyKindRef::Longhand(meta) = metadata.kind() else {
        panic!("contain is a longhand")
    };
    assert!(!meta.inherited_by_default());
    assert_eq!(meta.property().known_property(), PROPERTY);
    let initial = meta.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("intrinsic ordinary initial")
    };
    assert_eq!(value.property().known_property(), PROPERTY);
    // Exact typed None initial requires the new generated borrowed longhand
    // variant and belongs to functional implementation-time evidence.
    for input in STATES {
        contributions(&ParserFront::StyleAttribute.valid(PROPERTY, input, input));
    }
}

#[test]
fn whole_globals_expand_once_and_reject_ordinary_value_mixtures() {
    for (text, keyword) in GLOBALS {
        for front in FRONTS {
            let source = front.valid(PROPERTY, text, text);
            let values = contributions(&source);
            assert!(
                matches!(values.items()[0].value(),CssContributionValueRef::Global(value) if value==keyword)
            );
        }
        invalid(PROPERTY, &format!("{text} style"));
        invalid(PROPERTY, &format!("paint {text}"));
    }
}

#[test]
fn checked_fronts_reject_original_implicit_closures_for_ordinary_global_and_pending_values() {
    let mut inputs: Vec<_> = STATES
        .into_iter()
        .chain(GLOBALS.map(|(text, _)| text))
        .chain(["var(--contain)", "env(contain)", "attr(data-contain *)"])
        .map(|input| format!("{input}/*"))
        .collect();
    inputs.extend(["var(--contain", "env(contain", "attr(data-contain *"].map(str::to_owned));
    for text in inputs {
        let components = parse_component_values(&text).unwrap();
        let before = components.clone();
        let origin = closure_origin(&components);
        for grammar in [false, true] {
            let error = checked_components(
                PROPERTY,
                components.clone(),
                grammar,
                CssImportance::Important,
            )
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, before);
        let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
            panic!("original EOF")
        };
        assert!(opening.source().same_snapshot(at.source()));
        assert_eq!(at.source().as_str(), text);
        assert_eq!(at.span().start().byte_offset().value(), text.len());
        assert_eq!(at.span().start(), at.span().end());
    }
}

#[test]
fn browser_recovery_retains_original_occurrence_and_diagnostics_without_cleaning_source() {
    for input in ["paint size/*", "style/*", "initial/*", "var(--contain"] {
        let css = format!(".a{{contain:{input}");
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        assert!(validate_sheet(&css).is_err());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let before = report.clone();
        let normalized = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap(),
        )
        .unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("one recovered declaration")
        };
        assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
        assert_eq!(item.source().known().unwrap().property(), PROPERTY);
        assert_eq!(item.order(), 0);
        match item.expansion() {
            CssExpansion::Pending(handle) => {
                assert!(handle.source().same_occurrence(item.source()))
            }
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_terminals(values, item.source())
            }
            _ => panic!("contain lifecycle"),
        }
        assert_eq!(report, before);
    }
}

#[test]
fn pending_reentry_is_residual_first_retryable_and_preserves_replacement_and_occurrence_origins() {
    for input in [
        "var(--contain)",
        "env(contain)",
        "attr(data-contain *)",
        "style var(--contain)",
    ] {
        let source = ParserFront::StyleAttribute.parse(PROPERTY, input);
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("whole pending occurrence")
        };
        for bad in [
            "var(--again)",
            "env(again)",
            "attr(data-again *)",
            "style var(--again",
        ] {
            let error = handle
                .reenter(parse_component_values(bad).unwrap())
                .unwrap_err();
            assert_eq!(error.kind(), &CssExpansionErrorKind::ResidualSubstitution);
        }
        for bad in ["style style", "none paint", "inline-size", "size/*"] {
            let replacement = parse_component_values(bad).unwrap();
            let expected = checked_components(
                PROPERTY,
                replacement.clone(),
                true,
                CssImportance::Important,
            )
            .unwrap_err();
            for _ in 0..2 {
                let error = handle.reenter(replacement.clone()).unwrap_err();
                assert!(
                    matches!(error.kind(),CssExpansionErrorKind::InvalidReplacement(value) if value==&expected)
                );
            }
        }
        for text in STATES {
            let replacement = parse_component_values(text).unwrap();
            let before_replacement = replacement.clone();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed ordinary replacement")
            };
            assert_terminals(&values, &source);
            assert!(values.items()[0].ordinary_value().is_some());
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
            for (actual, original) in values.items()[0]
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(replacement.items())
            {
                let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) =
                    (actual.origin(), original.origin())
                else {
                    panic!("parsed replacement token")
                };
                assert!(actual.source().same_snapshot(original.source()));
                assert_eq!(actual.span(), original.span());
            }
            assert_eq!(replacement, before_replacement);
        }
        for (text, keyword) in GLOBALS {
            let replacement = parse_component_values(text).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("global replacement")
            };
            assert_terminals(&values, &source);
            assert!(
                matches!(values.items()[0].value(),CssContributionValueRef::Global(value) if value==keyword)
            );
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
        assert!(handle.source().same_occurrence(&source));
        assert_eq!(source, before);
    }
}

#[test]
fn normalization_counts_one_terminal_per_occurrence_and_preserves_order_importance_and_pending_state()
 {
    let css = ".a{contain:paint size!important;contain:style;contain:inherit!important;contain:var(--contain)}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 4, 4).unwrap();
    let normalized = normalize_report_with_limits(&report, exact).unwrap();
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 4);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), PROPERTY);
        assert_eq!(
            item.source().importance(),
            if index % 2 == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        if index == 3 {
            assert!(matches!(item.expansion(), CssExpansion::Pending(_)));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("ordinary/global terminal")
            };
            assert_terminals(values, item.source());
        }
    }
    let error = normalize_report_with_limits(
        &report,
        CssNormalizationLimits::try_new(0, 1, 4, 3).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3
        }
    );
    assert_eq!(error.declaration_order(), Some(3));
    assert_eq!(report, before);
    assert!(normalize_report_with_limits(&report, exact).is_ok());
}

#[test]
fn every_specified_state_keeps_exact_aggregate_leaf_and_declaration_byte_costs_with_atomic_reuse() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for canonical in STATES {
        let source = ParserFront::CheckedGrammar.valid(PROPERTY, canonical, canonical);
        let before = source.clone();
        let expected = format!("contain: {canonical} !important;");
        let primitive_nodes = if ["none", "strict", "content"].contains(&canonical) {
            1
        } else {
            1 + canonical.split_whitespace().count()
        };
        let value = containment(&source);
        let exact = Limits::new(primitive_nodes, primitive_nodes, canonical.len());
        assert_eq!(
            value.serialize_specified_with_limits(exact).unwrap(),
            canonical
        );
        for (limits, kind) in [
            (
                Limits::new(primitive_nodes - 1, primitive_nodes, canonical.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(primitive_nodes, primitive_nodes - 1, canonical.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(primitive_nodes, primitive_nodes, canonical.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    value
                        .serialize_specified_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
            }
        }
        let nodes = primitive_nodes + 2;
        let exact = Limits::new(nodes, nodes, expected.len());
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                Limits::new(nodes - 1, nodes, expected.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(nodes, nodes - 1, expected.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(nodes, nodes, expected.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    source
                        .to_specified_css_with_limits(limits)
                        .unwrap_err()
                        .kind(),
                    kind
                );
            }
        }
        assert_eq!(
            source.to_specified_css_with_limits(exact).unwrap(),
            expected
        );
        assert_eq!(source, before);
    }
}

#[test]
fn sibling_sheet_output_shares_a_cumulative_byte_budget_and_remains_reusable_after_failure() {
    let input = ".a{contain:paint style size!important;contain:strict;contain:content}";
    let expected =
        ".a { contain: size style paint !important; contain: strict; contain: content; }";
    let report = parse_sheet(input);
    assert!(report.is_clean());
    let before = report.clone();
    let limits = CssSpecifiedValueSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(limits)
            .unwrap(),
        expected
    );
    for _ in 0..2 {
        let error = report
            .syntax()
            .to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1,
            ))
            .unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(0));
        assert_eq!(report, before);
    }
    assert_eq!(
        report
            .syntax()
            .to_specified_css_with_limits(limits)
            .unwrap(),
        expected
    );
}

//! Containment2 WD20220917 §1.2/§2, Values3 §2.2, and the accepted
//! represented contain provider contract supply this independent oracle.
//! Specified strict/content remain authored keywords; computed containment,
//! counters, layout, painting and content-visibility execution stay downstream.
//! A component list costs one aggregate plus each keyword; enum carriers are
//! transparent. Declaration/name add two nodes, separators charge UTF-8 bytes.
//! Adopt by copying into tests/: the helper below is existing test-only support.
//! No future Style variant or generated Contain longhand variant is named.

use crate::css::support::authored_property;
use crate::css::support::failure_classes::serialization_failure;
use authored_property::{ParserFront, assert_source, checked_components};
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValueRef, CssComponentValues, CssValueOrigin, CssValueTokenRef,
};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssLonghandContributions, CssPropertyKindRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::box_model::{CssContain, CssContainComponent, CssContainComponentList};

const PROPERTY: CssKnownProperty = CssKnownProperty::Contain;
const FRONTS: [ParserFront; 5] = [
    ParserFront::StyleAttribute,
    ParserFront::CheckedName,
    ParserFront::CheckedGrammar,
    ParserFront::TextName,
    ParserFront::TextGrammar,
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
    value.value()
}
fn contributions(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
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
    let components = CssComponentValues::parse(&CssParser::default(), input).unwrap();
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
        assert_eq!(
            containment(&source)
                .serialize_specified(Default::default())
                .unwrap(),
            "style"
        );
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
fn normalization_counts_one_terminal_per_occurrence_and_preserves_order_importance_and_pending_state()
 {
    let css = ".a{contain:paint size!important;contain:style;contain:inherit!important;contain:var(--contain)}";
    let report = CssParser::default().parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 4, 4).unwrap();
    let normalized = report.normalize(exact).unwrap();
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
    let error = report
        .normalize(CssNormalizationLimits::try_new(0, 1, 4, 3).unwrap())
        .unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 3))
    );
    assert_eq!(error.declaration_order(), Some(3));
    assert_eq!(report, before);
    assert!(report.normalize(exact).is_ok());
}

#[test]
fn every_specified_state_keeps_exact_aggregate_leaf_and_declaration_byte_costs_with_atomic_reuse() {
    use crate::css::support::failure_classes::SerializationFailure as Kind;
    use CssSerializationLimits as Limits;
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
        assert_eq!(value.serialize_specified(exact).unwrap(), canonical);
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
                    serialization_failure(&value.serialize_specified(limits).unwrap_err()),
                    kind
                );
            }
        }
        let nodes = primitive_nodes + 2;
        let exact = Limits::new(nodes, nodes, expected.len());
        assert_eq!(source.serialize_specified(exact).unwrap(), expected);
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
                    serialization_failure(&source.serialize_specified(limits).unwrap_err()),
                    kind
                );
            }
        }
        assert_eq!(source.serialize_specified(exact).unwrap(), expected);
        assert_eq!(source, before);
    }
}

#[test]
fn sibling_sheet_output_shares_a_cumulative_byte_budget_and_remains_reusable_after_failure() {
    let input = ".a{contain:paint style size!important;contain:strict;contain:content}";
    let expected =
        ".a { contain: size style paint !important; contain: strict; contain: content; }";
    let report = CssParser::default().parse_sheet(input);
    assert!(report.is_clean());
    let before = report.clone();
    let limits = CssSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().serialize_specified(limits).unwrap(),
        expected
    );
    for _ in 0..2 {
        let error = report
            .syntax()
            .serialize_specified(CssSerializationLimits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1,
            ))
            .unwrap_err();
        assert!(error.is_byte_limit(), "{error:?}");
        assert_eq!(error.rule_index(), Some(0));
        assert_eq!(report, before);
    }
    assert_eq!(
        report.syntax().serialize_specified(limits).unwrap(),
        expected
    );
}

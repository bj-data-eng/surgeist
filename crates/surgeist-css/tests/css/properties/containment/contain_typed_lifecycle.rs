//! Functional coverage of the new Containment2 Style component and generated
//! borrowed Contain longhand. The published contain_authored_contract suite
//! owns existing-callable RED; these typed symbols did not exist there.
//! Containment2 WD20220917 §2 selects a nonempty unique unordered group and
//! grammar-order specified output. Accepted aggregate-plus-keyword costs remain.

use crate::css::support::property_admission::assert_implicit_closure;
use crate::css::support::serialization_limits::LimitLadder;
use CssContainComponent::{Layout, Paint, Size, Style};
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValue, CssComponentValues, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty,
    CssKnownPropertyValueRef, CssLonghandContributions, CssLonghandValueRef, CssPropertyKindRef,
    CssPropertyNameRef, CssPropertyTarget,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::box_model::{
    CssContain, CssContainComponent, CssContainComponentList, CssContentVisibility,
};

const ROLES: [(CssContainComponent, &str); 4] = [
    (Size, "size"),
    (Layout, "layout"),
    (Style, "style"),
    (Paint, "paint"),
];

fn permutations(values: &[CssContainComponent]) -> Vec<Vec<CssContainComponent>> {
    if values.is_empty() {
        return vec![Vec::new()];
    }
    let mut result = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let rest: Vec<_> = values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| (i != index).then_some(*v))
            .collect();
        for mut tail in permutations(&rest) {
            tail.insert(0, *value);
            result.push(tail);
        }
    }
    result
}

fn word(value: CssContainComponent) -> &'static str {
    match value {
        Size => "size",
        Layout => "layout",
        Style => "style",
        Paint => "paint",
        _ => unreachable!("selected finite domain"),
    }
}

fn primitive(source: &CssDeclaration) -> &CssContain {
    let CssKnownPropertyValueRef::Contain(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed contain wrapper")
    };
    wrapper.value()
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("ordinary contain contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal")
    };
    assert_eq!(item.property(), CssKnownProperty::Contain);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().value_components(), source.value_components());
    let CssLonghandValueRef::Contain(value) = item.ordinary_value().unwrap().view() else {
        panic!("new borrowed Contain variant")
    };
    assert_eq!(value, primitive(source));
    values
}

fn primitive_limits(value: &CssContain, expected: &str, nodes: usize) {
    LimitLadder::new(expected, nodes, nodes)
        .assert_unchanged(value, CssContain::serialize_specified);
}

#[test]
fn every_unique_typed_component_state_retains_authored_order_and_emits_grammar_order() {
    for subset in 1..16 {
        let selected: Vec<_> = ROLES
            .iter()
            .enumerate()
            .filter_map(|(index, (role, _))| (subset & (1 << index) != 0).then_some(*role))
            .collect();
        let expected = ROLES
            .iter()
            .enumerate()
            .filter_map(|(index, (_, text))| (subset & (1 << index) != 0).then_some(*text))
            .collect::<Vec<_>>()
            .join(" ");
        for authored in permutations(&selected) {
            let list = CssContainComponentList::try_new(authored.clone()).unwrap();
            assert_eq!(list.components(), authored);
            let value = CssContain::Components(list);
            primitive_limits(&value, &expected, 1 + selected.len());
            let CssContain::Components(retained) = value else {
                panic!("retained components")
            };
            assert_eq!(retained.components(), authored);
        }
    }
    for (value, text) in [
        (CssContain::None, "none"),
        (CssContain::Strict, "strict"),
        (CssContain::Content, "content"),
    ] {
        primitive_limits(&value, text, 1);
    }
}

#[test]
fn component_constructor_rejects_empty_and_every_duplicate_position_including_style() {
    assert!(CssContainComponentList::try_new(Vec::new()).is_none());
    for (duplicate, _) in ROLES {
        assert!(CssContainComponentList::try_new(vec![duplicate, duplicate]).is_none());
        for order in permutations(&[Size, Layout, Style, Paint]) {
            for index in 0..=order.len() {
                let mut invalid = order.clone();
                invalid.insert(index, duplicate);
                assert!(CssContainComponentList::try_new(invalid).is_none());
            }
        }
    }
}

#[test]
fn borrowed_contain_payloads_preserve_all_typed_states_and_exact_parsed_authored_order() {
    for subset in 1..16 {
        let selected: Vec<_> = ROLES
            .iter()
            .enumerate()
            .filter_map(|(index, (role, _))| (subset & (1 << index) != 0).then_some(*role))
            .collect();
        for authored in permutations(&selected) {
            let text = authored
                .iter()
                .map(|role| word(*role))
                .collect::<Vec<_>>()
                .join(" ");
            let expected =
                CssContain::Components(CssContainComponentList::try_new(authored.clone()).unwrap());
            let report = CssParser::default()
                .parse_block_contents(format!("/*😀*/contain:{text}!important"));
            assert!(report.is_clean());
            let source = &report.syntax()[0];
            let before = source.clone();
            assert_eq!(primitive(source), &expected);
            completed(source);
            assert_eq!(source, &before);
            for grammar in [false, true] {
                let input = CssComponentValues::parse(&CssParser::default(), &text).unwrap();
                let source = if grammar {
                    CssDeclaration::try_from_components(
                        &CssParser::default(),
                        CssPropertyTarget::Grammar(CssKnownProperty::Contain.grammar()),
                        input,
                        CssImportance::Important,
                    )
                } else {
                    CssDeclaration::try_from_components(
                        &CssParser::default(),
                        CssPropertyTarget::Property(CssPropertyNameRef::Known(
                            CssKnownProperty::Contain,
                        )),
                        input,
                        CssImportance::Important,
                    )
                }
                .unwrap();
                assert_eq!(primitive(&source), &expected);
                completed(&source);
            }
        }
    }
    for (value, text) in [
        (CssContain::None, "none"),
        (CssContain::Strict, "strict"),
        (CssContain::Content, "content"),
    ] {
        let source = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Contain)),
            CssComponentValues::parse(&CssParser::default(), text).unwrap(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(primitive(&source), &value);
        completed(&source);
    }
}

#[test]
fn intrinsic_none_is_a_typed_noninherited_contain_initial_without_authored_source() {
    let metadata = CssKnownProperty::Contain.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("contain longhand")
    };
    assert!(!longhand.inherited_by_default());
    assert_eq!(
        longhand.property().known_property(),
        CssKnownProperty::Contain
    );
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::Contain
    );
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary intrinsic initial")
    };
    let CssLonghandValueRef::Contain(value) = value.view() else {
        panic!("typed borrowed initial")
    };
    assert_eq!(value, &CssContain::None);
    primitive_limits(value, "none", 1);
}

#[test]
fn mixed_token_origins_survive_checked_contain_and_pending_replacement_without_reparsing() {
    let style = CssComponentValues::parse(&CssParser::default(), r"\73 tyle")
        .unwrap()
        .items()[0]
        .clone();
    let size = CssComponentValues::parse(&CssParser::default(), "SIZE")
        .unwrap()
        .items()[0]
        .clone();
    let paint = CssComponentValue::try_ident("paint").unwrap();
    let input = CssComponentValues::try_new(
        vec![style.clone(), paint.clone(), size.clone()],
        Default::default(),
    )
    .unwrap();
    let expected =
        CssContain::Components(CssContainComponentList::try_new(vec![Style, Paint, Size]).unwrap());
    for grammar in [false, true] {
        let source = if grammar {
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(CssKnownProperty::Contain.grammar()),
                input.clone(),
                CssImportance::Important,
            )
        } else {
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Contain)),
                input.clone(),
                CssImportance::Important,
            )
        }
        .unwrap();
        assert_eq!(primitive(&source), &expected);
        assert_eq!(source.value_components(), &input);
        for (actual, original) in source.value_components().items().iter().zip(input.items()) {
            assert_eq!(actual.origin(), original.origin());
            if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) =
                (actual.origin(), original.origin())
            {
                assert!(actual.source().same_snapshot(original.source()));
                assert_eq!(actual.span(), original.span());
            }
        }
        assert_eq!(
            source.value_components().items()[1].origin(),
            &CssValueOrigin::Programmatic
        );
        completed(&source);
    }
    let pending = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Property(CssPropertyNameRef::Known(CssKnownProperty::Contain)),
        CssComponentValues::parse(&CssParser::default(), "var(--contain)").unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    let CssExpansion::Pending(handle) = pending.expand().unwrap() else {
        panic!("pending")
    };
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(input.clone()).unwrap() else {
            panic!("one replacement terminal")
        };
        let [item] = values.items() else {
            panic!("one replacement terminal")
        };
        assert!(item.source().same_occurrence(&pending));
        assert_eq!(item.source().importance(), CssImportance::Important);
        let CssLonghandValueRef::Contain(value) = item.ordinary_value().unwrap().view() else {
            panic!("typed replacement")
        };
        assert_eq!(value, &expected);
        let replacement = item.replacement_components().unwrap();
        assert_eq!(replacement, &input);
        for (actual, original) in replacement.items().iter().zip(input.items()) {
            assert_eq!(actual.origin(), original.origin());
            if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) =
                (actual.origin(), original.origin())
            {
                assert!(actual.source().same_snapshot(original.source()));
            }
        }
    }
    assert!(handle.source().same_occurrence(&pending));
}

#[test]
fn recovered_style_keeps_its_typed_borrowed_payload_and_original_diagnostics() {
    let css = ".a{contain:style/*";
    let report = CssParser::default().parse_sheet(css);
    assert_implicit_closure(css, &report);
    let before = report.clone();
    let normalized = report
        .normalize(CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap())
        .unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let item = normalized
        .syntax()
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
    let values = completed(item.source());
    let CssLonghandValueRef::Contain(value) = values.items()[0].ordinary_value().unwrap().view()
    else {
        panic!("recovered borrowed payload")
    };
    assert_eq!(
        value,
        &CssContain::Components(CssContainComponentList::try_new(vec![Style]).unwrap())
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "style"
    );
    assert_eq!(report, before);
}

#[test]
fn authored_contain_and_content_visibility_keep_independent_typed_longhand_values() {
    for (contain_text, contain_value, visibility_text, visibility_value) in [
        (
            "none",
            CssContain::None,
            "hidden",
            CssContentVisibility::Hidden,
        ),
        (
            "style paint",
            CssContain::Components(CssContainComponentList::try_new(vec![Style, Paint]).unwrap()),
            "auto",
            CssContentVisibility::Auto,
        ),
        (
            "strict",
            CssContain::Strict,
            "visible",
            CssContentVisibility::Visible,
        ),
    ] {
        let report = CssParser::default().parse_block_contents(format!(
            "contain:{contain_text};content-visibility:{visibility_text}"
        ));
        assert!(report.is_clean());
        let [contain, visibility] = report.syntax().as_slice() else {
            panic!("two authored occurrences")
        };
        let contain_values = completed(contain);
        let CssLonghandValueRef::Contain(value) =
            contain_values.items()[0].ordinary_value().unwrap().view()
        else {
            panic!("contain payload")
        };
        assert_eq!(value, &contain_value);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            visibility.expand().unwrap()
        else {
            panic!("content visibility terminal")
        };
        let [item] = values.items() else {
            panic!("one visibility terminal")
        };
        assert_eq!(item.property(), CssKnownProperty::ContentVisibility);
        let CssLonghandValueRef::ContentVisibility(value) = item.ordinary_value().unwrap().view()
        else {
            panic!("visibility payload")
        };
        assert_eq!(value, &visibility_value);
        assert_eq!(
            contain.serialize_specified(Default::default()).unwrap(),
            format!("contain: {contain_text};")
        );
        assert_eq!(
            visibility.serialize_specified(Default::default()).unwrap(),
            format!("content-visibility: {visibility_text};")
        );
    }
}

#[test]
fn mixed_containment_normalization_and_pending_retry_preserve_each_independent_occurrence() {
    let css = ".a{contain:paint style!important;content-visibility:hidden;contain:var(--contain)!important;content-visibility:env(vis);contain:inherit;content-visibility:revert!important;contain:strict;contain:content}";
    let expected = ".a { contain: style paint !important; content-visibility: hidden; contain: var(--contain) !important; content-visibility: env(vis); contain: inherit; content-visibility: revert !important; contain: strict; contain: content; }";
    let report = CssParser::default().parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(0, 1, 8, 8).unwrap();
    let normalized = report.normalize(exact).unwrap();
    let declarations: Vec<_> = normalized
        .syntax()
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 8);
    for (index, item) in declarations.iter().enumerate() {
        let property = if [1, 3, 5].contains(&index) {
            CssKnownProperty::ContentVisibility
        } else {
            CssKnownProperty::Contain
        };
        let importance = if [0, 2, 5].contains(&index) {
            CssImportance::Important
        } else {
            CssImportance::Normal
        };
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property);
        assert_eq!(item.source().importance(), importance);
        assert_eq!(item.source().parsed_value().unwrap().source().as_str(), css);
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        if index == 2 || index == 3 {
            let CssExpansion::Pending(handle) = item.expansion() else {
                panic!("pending whole occurrence")
            };
            assert!(handle.source().same_occurrence(item.source()));
            let invalid = if index == 2 { "style style" } else { "none" };
            for _ in 0..2 {
                let error = handle
                    .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                    .unwrap_err();
                assert!(error.replacement_error().is_some());
            }
            let error = handle
                .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again").unwrap())
                .unwrap_err();
            assert!(error.is_residual_substitution(), "{error:?}");
            let replacement = CssComponentValues::parse(
                &CssParser::default(),
                if index == 2 {
                    "paint style size"
                } else {
                    "auto"
                },
            )
            .unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one replacement terminal")
                };
                let [contribution] = values.items() else {
                    panic!("one terminal")
                };
                assert_eq!(contribution.property(), property);
                assert!(contribution.source().same_occurrence(item.source()));
                assert_eq!(contribution.source().importance(), importance);
                assert_eq!(
                    contribution.source().value_components(),
                    item.source().value_components()
                );
                assert_eq!(contribution.replacement_components(), Some(&replacement));
                for (actual, original) in contribution
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(original)) =
                        (actual.origin(), original.origin())
                    else {
                        panic!("replacement origin")
                    };
                    assert!(actual.source().same_snapshot(original.source()));
                    assert_eq!(actual.span(), original.span());
                }
                let value = contribution.ordinary_value().unwrap();
                match (index, value.view()) {
                    (2, CssLonghandValueRef::Contain(value)) => assert_eq!(
                        value,
                        &CssContain::Components(
                            CssContainComponentList::try_new(vec![Paint, Style, Size]).unwrap()
                        )
                    ),
                    (3, CssLonghandValueRef::ContentVisibility(value)) => {
                        assert_eq!(value, &CssContentVisibility::Auto)
                    }
                    _ => panic!("independent replacement payload"),
                }
            }
            let replacement =
                CssComponentValues::parse(&CssParser::default(), "revert-layer").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("whole global replacement")
            };
            let [contribution] = values.items() else {
                panic!("one global terminal")
            };
            assert_eq!(contribution.property(), property);
            assert!(matches!(
                contribution.value(),
                CssContributionValueRef::Global(CssGlobalKeyword::RevertLayer)
            ));
            assert!(contribution.source().same_occurrence(item.source()));
            assert_eq!(contribution.source().importance(), importance);
            assert_eq!(contribution.replacement_components(), Some(&replacement));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("completed occurrence")
            };
            let [contribution] = values.items() else {
                panic!("one completed terminal")
            };
            assert_eq!(contribution.property(), property);
            assert!(contribution.source().same_occurrence(item.source()));
            assert_eq!(contribution.source().importance(), importance);
            assert_eq!(
                contribution.source().value_components(),
                item.source().value_components()
            );
            match (index, contribution.value()) {
                (0, CssContributionValueRef::Ordinary(CssLonghandValueRef::Contain(value))) => {
                    assert_eq!(
                        value,
                        &CssContain::Components(
                            CssContainComponentList::try_new(vec![Paint, Style]).unwrap()
                        )
                    )
                }
                (
                    1,
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::ContentVisibility(
                        CssContentVisibility::Hidden,
                    )),
                ) => {}
                (4, CssContributionValueRef::Global(CssGlobalKeyword::Inherit))
                | (5, CssContributionValueRef::Global(CssGlobalKeyword::Revert)) => {}
                (
                    6,
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::Contain(
                        CssContain::Strict,
                    )),
                )
                | (
                    7,
                    CssContributionValueRef::Ordinary(CssLonghandValueRef::Contain(
                        CssContain::Content,
                    )),
                ) => {}
                _ => panic!("independent original payload"),
            }
        }
    }
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 1, 7, 8).unwrap(),
            CssNormalizationResource::Declarations,
            7,
        ),
        (
            CssNormalizationLimits::try_new(0, 1, 8, 7).unwrap(),
            CssNormalizationResource::Contributions,
            7,
        ),
    ] {
        for _ in 0..2 {
            let error = report.normalize(limits).unwrap_err();
            assert_eq!(error.exhausted_limit(), Some((resource, limit)));
            assert_eq!(error.declaration_order(), Some(7));
            assert_eq!(report, before);
        }
    }
    assert!(report.normalize(exact).is_ok());
    let exact = CssSerializationLimits::new(usize::MAX, usize::MAX, expected.len());
    assert_eq!(
        report.syntax().serialize_specified(exact).unwrap(),
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
        assert_eq!(report, before);
    }
    assert_eq!(
        report.syntax().serialize_specified(exact).unwrap(),
        expected
    );
}

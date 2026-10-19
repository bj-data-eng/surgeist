#![forbid(unsafe_code)]

//! Flexbox 1 §§5.1–5.3: direction and wrapping are noninherited terminals;
//! `flex-flow` sets both, including the initial of an omitted component.

use surgeist_css::*;

fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("ordinary flex direction/wrap contributions")
    };
    items.items().to_vec()
}

fn ordinary(name: &str, value: &str) -> CssLonghandValue {
    let source = declaration(&format!("{name}:{value}"));
    let items = expanded(&source);
    let [item] = items.as_slice() else {
        panic!("one terminal: {name}:{value}")
    };
    item.ordinary_value().unwrap().clone()
}

#[test]
fn direction_and_wrap_metadata_have_specified_noninherited_initials() {
    for (property, name, initial) in [
        (CssKnownProperty::FlexDirection, "flex-direction", "row"),
        (CssKnownProperty::FlexWrap, "flex-wrap", "nowrap"),
    ] {
        let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
            panic!("{name} must be a terminal longhand")
        };
        assert_eq!(metadata.property().known_property(), property);
        assert!(!metadata.inherited_by_default(), "{name}");
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("{name} has a fixed intrinsic initial")
        };
        assert_eq!(value.property().known_property(), property);
        assert_eq!(value, &ordinary(name, initial));
    }
}

#[test]
fn every_direction_and_wrap_keyword_expands_to_its_exact_terminal() {
    for (property, name, keywords) in [
        (
            CssKnownProperty::FlexDirection,
            "flex-direction",
            &["row", "row-reverse", "column", "column-reverse"][..],
        ),
        (
            CssKnownProperty::FlexWrap,
            "flex-wrap",
            &["nowrap", "wrap", "wrap-reverse"][..],
        ),
    ] {
        for keyword in keywords {
            let source = declaration(&format!("{name}:{keyword}!important"));
            let items = expanded(&source);
            let [item] = items.as_slice() else {
                panic!("one contribution for {name}:{keyword}")
            };
            assert_eq!(item.property(), property);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn flow_metadata_has_two_ordered_settable_terminals_and_no_reset_members() {
    let CssPropertyKindRef::Shorthand(metadata) =
        CssKnownProperty::FlexFlow.metadata().unwrap().kind()
    else {
        panic!("flex-flow must be a shorthand")
    };
    let expected = [CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap];
    assert_eq!(
        metadata
            .members()
            .iter()
            .map(|member| member.known_property())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(metadata.settable_members().len(), 2);
    assert!(metadata.reset_only_members().is_empty());
    assert!(!metadata.is_legacy());
}

#[test]
fn unordered_flow_components_and_omissions_contribute_both_specified_values() {
    for (authored, direction, wrap) in [
        ("wrap-reverse column", "column", "wrap-reverse"),
        ("column wrap-reverse", "column", "wrap-reverse"),
        ("wrap-reverse", "row", "wrap-reverse"),
        ("column", "column", "nowrap"),
        ("nowrap", "row", "nowrap"),
    ] {
        let source = declaration(&format!("flex-flow:{authored}!important"));
        let items = expanded(&source);
        let [direction_item, wrap_item] = items.as_slice() else {
            panic!("two flow contributions: {authored}")
        };
        assert_eq!(direction_item.property(), CssKnownProperty::FlexDirection);
        assert_eq!(wrap_item.property(), CssKnownProperty::FlexWrap);
        assert_eq!(
            direction_item.ordinary_value(),
            Some(&ordinary("flex-direction", direction))
        );
        assert_eq!(
            wrap_item.ordinary_value(),
            Some(&ordinary("flex-wrap", wrap))
        );
        for item in items {
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn every_css_wide_keyword_reaches_each_flow_terminal_without_resolution() {
    for (text, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for (name, expected) in [
            ("flex-direction", &[CssKnownProperty::FlexDirection][..]),
            ("flex-wrap", &[CssKnownProperty::FlexWrap][..]),
            (
                "flex-flow",
                &[CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap][..],
            ),
        ] {
            let source = declaration(&format!("{name}:{text}!important"));
            let items = expanded(&source);
            assert_eq!(items.len(), expected.len(), "{name}:{text}");
            for (item, property) in items.iter().zip(expected) {
                assert_eq!(item.property(), *property);
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_flow_reentry_checks_complete_grammar_and_preserves_original_source() {
    let source = declaration("flex-flow:var(--flow)!important");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("substitution remains pending")
    };
    for invalid in ["row column", "wrap nowrap", "row wrap nowrap", "row / wrap"] {
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    let replacement = parse_component_values("wrap-reverse column").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("complete flow reentry")
        };
        let [direction, wrap] = items.items() else {
            panic!("two reentered terminals")
        };
        assert_eq!(
            direction.ordinary_value(),
            Some(&ordinary("flex-direction", "column"))
        );
        assert_eq!(
            wrap.ordinary_value(),
            Some(&ordinary("flex-wrap", "wrap-reverse"))
        );
        for item in items.items() {
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalized_flow_keeps_mixed_order_and_rejects_a_missing_member_budget() {
    let report = parse_sheet(
        ".a{flex-direction:column;flex-flow:row column;flex-flow:wrap-reverse;flex-wrap:wrap}",
    );
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDeclaration
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 3);
    for (index, expected) in [
        &[CssKnownProperty::FlexDirection][..],
        &[CssKnownProperty::FlexDirection, CssKnownProperty::FlexWrap][..],
        &[CssKnownProperty::FlexWrap][..],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[index].expansion()
        else {
            panic!("normalized flex flow members")
        };
        assert_eq!(
            items
                .items()
                .iter()
                .map(|item| item.property())
                .collect::<Vec<_>>(),
            expected.to_vec()
        );
        assert!(
            items
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(declarations[index].source()))
        );
    }
    let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, 2).unwrap();
    let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2
        }
    ));
    assert_eq!(
        error.declaration().unwrap().known().unwrap().property(),
        CssKnownProperty::FlexFlow
    );
}

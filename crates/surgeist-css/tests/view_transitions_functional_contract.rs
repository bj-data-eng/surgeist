#![forbid(unsafe_code)]
//! Independent new-API functional contract. VT1 CRD20240328 §§2.1,3.1,3.2,
//! Values4 custom-ident and selected Selectors4 attachment/context contracts.
//! New typed APIs establish functional behavior without absent-symbol RED.
//! Limits tariffs follow the adopted shared writer convention, not spec mandates.
use CssPseudoElementSegment::{PseudoClass as P, PseudoElement as E};
use surgeist_css::*;
type Limits = CssSpecifiedValueSerializationLimits;
type LimitKind = CssSpecifiedValueSerializationErrorKind;

fn ident(text: &str) -> CssViewTransitionIdent {
    CssViewTransitionIdent::try_new(CssCustomIdent::try_new(text).unwrap()).unwrap()
}
fn name(text: &str) -> CssViewTransitionName {
    CssViewTransitionName::Custom(ident(text))
}
fn argument(text: &str) -> CssViewTransitionNameSelector {
    CssViewTransitionNameSelector::Name(CssCustomIdent::try_new(text).unwrap())
}
fn elements(arg: CssViewTransitionNameSelector) -> [CssPseudoElement; 4] {
    [
        CssPseudoElement::ViewTransitionGroup(arg.clone()),
        CssPseudoElement::ViewTransitionImagePair(arg.clone()),
        CssPseudoElement::ViewTransitionOld(arg.clone()),
        CssPseudoElement::ViewTransitionNew(arg),
    ]
}
fn declaration(source: &str) -> CssDeclaration {
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{source}: {report:?}");
    let [value] = report.syntax().as_slice() else {
        panic!("one authored declaration")
    };
    value.clone()
}
fn value(declaration: &CssDeclaration) -> &CssViewTransitionName {
    let CssKnownPropertyValueRef::ViewTransitionName(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed name wrapper")
    };
    wrapper.name()
}
fn selector(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.is_clean(), "{source}: {report:?}");
    report.into_validation_result().unwrap().unwrap()
}
fn segments(value: &CssSelector) -> &[CssPseudoElementSegment] {
    let CssSelector::Compound(value) = value else {
        panic!("originating compound")
    };
    value.pseudo_elements().unwrap().segments()
}
fn rejected(source: &str) {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(report.syntax().is_none(), "{source}: {report:?}");
    assert!(report.into_validation_result().is_err());
}
fn suffix_list(value: &CssSelector) -> &CssPseudoSelectorList {
    let [_, P(CssPseudoClass::Is(list))] = segments(value) else {
        panic!("one Is suffix")
    };
    list
}
fn retained(source: &str) -> CssSelector {
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert!(!report.is_clean(), "{source}: {report:?}");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::PreserveInvalidSelectorListItem)
    );
    report.syntax().as_ref().unwrap().clone()
}
fn attach(
    element: CssPseudoElement,
    list: CssPseudoSelectorList,
) -> Option<CssPseudoElementSequence> {
    CssPseudoElementSequence::try_from_segments(vec![E(element), P(CssPseudoClass::Is(list))])
}
fn parsed_range(origin: &CssValueOrigin, source: &str, bytes: (usize, usize), columns: (u32, u32)) {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("original parsed token")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), bytes.0);
    assert_eq!(origin.span().end().byte_offset().value(), bytes.1);
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(origin.span().end().line().value(), 0);
    assert_eq!(origin.span().start().column().value(), columns.0);
    assert_eq!(origin.span().end().column().value(), columns.1);
}

#[test]
fn checked_property_ident_excludes_its_keywords_without_changing_generic_ident() {
    for forbidden in ["none", "NONE", "NoNe", "auto", "AUTO", "AuTo"] {
        let generic = CssCustomIdent::try_new(forbidden).unwrap();
        assert!(CssViewTransitionIdent::try_new(generic.clone()).is_none());
        assert_eq!(
            argument(forbidden),
            CssViewTransitionNameSelector::Name(generic)
        );
    }
    for forbidden in [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "default",
        "DeFaUlT",
    ] {
        assert!(CssCustomIdent::try_new(forbidden).is_none());
    }
    for (decoded, specified) in [
        ("Card", "Card"),
        ("card", "card"),
        ("A B", r"A\ B"),
        ("Δa", "Δa"),
        ("auto-card", "auto-card"),
    ] {
        let domain = ident(decoded);
        assert_eq!(domain.as_str(), decoded);
        assert_eq!(
            CssViewTransitionName::Custom(domain)
                .serialize_specified()
                .unwrap(),
            specified
        );
    }
    assert_ne!(ident("Card"), ident("card"));
    assert_eq!(
        CssViewTransitionName::None.serialize_specified().unwrap(),
        "none"
    );
}

#[test]
fn ordinary_wrapper_global_alternatives_and_intrinsic_metadata_share_one_property() {
    let property = CssKnownProperty::ViewTransitionName;
    assert_eq!(property.canonical_name(), "view-transition-name");
    assert_eq!(
        CssKnownProperty::from_name("VIEW-TRANSITION-NAME"),
        Some(property)
    );
    for (text, expected) in [
        ("none", CssViewTransitionName::None),
        ("NONE", CssViewTransitionName::None),
        ("Card", name("Card")),
        (r"\43 ard", name("Card")),
        (r"A\ B", name("A B")),
    ] {
        let source = declaration(&format!("view-transition-name:{text}"));
        assert_eq!(source.known().unwrap().property(), property);
        assert_eq!(value(&source), &expected);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!(
                "view-transition-name: {};",
                expected.serialize_specified().unwrap()
            )
        );
    }
    for (text, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration(&format!("view-transition-name:{text}"));
        assert_eq!(source.known().unwrap().global(), Some(expected));
        assert!(source.known().unwrap().property_value().is_none());
    }
    let CssPropertyKindRef::Longhand(metadata) = property.metadata().unwrap().kind() else {
        panic!("terminal property")
    };
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("intrinsic initial")
    };
    let CssLonghandValueRef::ViewTransitionName(actual) = initial.view() else {
        panic!("typed initial name")
    };
    assert_eq!(actual, &CssViewTransitionName::None);
}

#[test]
fn parsed_value_window_and_checked_origins_remain_truthful_with_unicode() {
    let source = "/*😀*/view-transition-name:Δcard!important";
    let authored = declaration(source);
    assert_eq!(value(&authored), &name("Δcard"));
    let parsed_name = authored.parsed_name().unwrap();
    assert_eq!(
        (
            parsed_name.span().start().byte_offset().value(),
            parsed_name.span().end().byte_offset().value()
        ),
        (8, 28)
    );
    assert_eq!(
        (
            parsed_name.span().start().column().value(),
            parsed_name.span().end().column().value()
        ),
        (6, 26)
    );
    let region = authored.parsed_value().unwrap();
    assert_eq!(region.source().as_str(), source);
    assert_eq!(
        (
            region.span().start().byte_offset().value(),
            region.span().end().byte_offset().value()
        ),
        (29, 35)
    );
    assert_eq!(
        (
            region.span().start().column().value(),
            region.span().end().column().value()
        ),
        (27, 32)
    );
    parsed_range(
        authored.value_components().items()[0].origin(),
        source,
        (29, 35),
        (27, 32),
    );
    assert!(region.source().same_snapshot(parsed_name.source()));
    let parsed = parse_component_values(r"\43 ard").unwrap();
    let checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ViewTransitionName),
        parsed.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(value(&checked), &name("Card"));
    assert!(checked.parsed_name().is_none() && checked.parsed_value().is_none());
    assert_eq!(checked.value_components(), &parsed);
    parsed_range(
        checked.value_components().items()[0].origin(),
        r"\43 ard",
        (0, 7),
        (0, 7),
    );
    let space = parse_component_values(" ").unwrap().items()[0].clone();
    let mixed =
        CssComponentValues::try_new(vec![space, CssComponentValue::try_ident("Card").unwrap()])
            .unwrap();
    let mixed_checked = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ViewTransitionName),
        mixed.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(value(&mixed_checked), &name("Card"));
    assert_eq!(mixed_checked.value_components(), &mixed);
    assert!(mixed_checked.parsed_name().is_none() && mixed_checked.parsed_value().is_none());
    assert!(matches!(
        mixed_checked.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        mixed_checked.value_components().items()[1].origin(),
        &CssValueOrigin::Programmatic
    );
}

#[test]
fn invalid_property_values_drop_only_their_occurrence_with_original_responsible_token() {
    let source = "/*😀*/view-transition-name:AUTO;color:red";
    let report = parse_style_attribute(source);
    let [sibling] = report.syntax().as_slice() else {
        panic!("valid later declaration survives")
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let diagnostic = report
        .diagnostics()
        .iter()
        .find(|d| d.error().code() == CssErrorCode::InvalidPropertyValue)
        .unwrap();
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 29);
    assert_eq!(diagnostic.error().position().column().value(), 27);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("typed grammar error")
    };
    assert_eq!(detail.property(), CssKnownProperty::ViewTransitionName);
    assert_eq!(detail.encountered().unwrap().authored(), "AUTO");
    assert!(report.into_validation_result().is_err());
    for invalid in [
        "",
        "default",
        "card other",
        "card,other",
        "\"card\"",
        "5",
        "50%",
        "f(card)",
        "initial card",
    ] {
        let components = parse_component_values(invalid).unwrap();
        let before = components.clone();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::ViewTransitionName),
                components.clone(),
                CssImportance::Normal
            )
            .is_err(),
            "{invalid}"
        );
        assert_eq!(components, before);
    }
}

#[test]
fn pending_property_reentry_is_strict_repeatable_and_preserves_source_and_replacement() {
    for pending_css in ["var(--tag)", "env(tag)", "attr(data-tag)"] {
        let source = declaration(&format!("view-transition-name:{pending_css}!important"));
        assert!(source.known().unwrap().substitution_dependent().is_some());
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending name")
        };
        for invalid in [
            "auto",
            "default",
            "Card other",
            "var(--again)",
            "env(again)",
        ] {
            let replacement = parse_component_values(invalid).unwrap();
            let before = replacement.clone();
            assert!(pending.reenter(replacement.clone()).is_err(), "{invalid}");
            assert_eq!(replacement, before);
            assert!(pending.source().same_occurrence(&source));
        }
        let replacement = parse_component_values(r"\43 ard").unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("one terminal name")
            };
            let [item] = items.items() else {
                panic!("one contribution")
            };
            assert_eq!(item.property(), CssKnownProperty::ViewTransitionName);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssLonghandValueRef::ViewTransitionName(actual) =
                item.ordinary_value().unwrap().view()
            else {
                panic!("ordinary replacement name")
            };
            assert_eq!(actual, &name("Card"));
        }
        let recovered = parse_component_values("var(--tag").unwrap();
        assert!(pending.reenter(recovered).is_err());
        assert!(
            pending
                .reenter(parse_component_values("none").unwrap())
                .is_ok()
        );
    }
}

#[test]
fn every_typed_pseudo_argument_branch_is_distinct_and_checked_readmission_retains_it() {
    for (argument_css, arg) in [
        ("*", CssViewTransitionNameSelector::Wildcard),
        ("Card", argument("Card")),
        ("none", argument("none")),
        ("auto", argument("auto")),
        (r"A\ B", argument("A B")),
        ("Δa", argument("Δa")),
    ] {
        for (function, element) in ["group", "image-pair", "old", "new"]
            .into_iter()
            .zip(elements(arg))
        {
            let css = format!("::view-transition-{function}({argument_css})");
            let parsed = selector(&css);
            assert_eq!(segments(&parsed), &[E(element.clone())]);
            let checked_sequence = CssPseudoElementSequence::try_new(vec![element]).unwrap();
            assert_eq!(checked_sequence.segments(), segments(&parsed));
            let checked_list = CssSelectorList::try_new(vec![parsed.clone()]).unwrap();
            assert_eq!(checked_list.selectors(), &[parsed]);
            assert_eq!(checked_list.to_specified_css().unwrap(), css);
        }
    }
    let root = selector("::VIEW-TRANSITION");
    assert_eq!(segments(&root), &[E(CssPseudoElement::ViewTransition)]);
    assert_eq!(root.to_specified_css().unwrap(), "::view-transition");
    assert_ne!(
        selector("::view-transition-old(Card)"),
        selector("::view-transition-old(card)")
    );
}

#[test]
fn malformed_names_reject_and_implicit_function_eof_is_retained_but_not_clean() {
    for source in [
        "::view-transition()",
        ":view-transition",
        "::view-transition-group",
        "::view-transition-group()",
        "::view-transition-group(A B)",
        "::view-transition-group(A,B)",
        "::view-transition-group(default)",
        "::view-transition-group(revert)",
        "::view-transition-group(\"A\")",
        "::view-transition-group(1)",
        "::view-transition-group(**)",
        "::view-transition-group(A.x)",
        "::view-transition-group(.x)",
        "::view-transition-group (A)",
        ":: view-transition-group(A)",
    ] {
        rejected(source);
    }
    let source = "::view-transition-old(Card";
    let report = parse_selector(source, &CssNamespaceContext::default());
    assert_eq!(
        report
            .syntax()
            .as_ref()
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "::view-transition-old(Card)"
    );
    assert!(!report.is_clean());
    assert!(report.diagnostics().iter().any(|d| matches!(
        d.error().kind(),
        ErrorKind::UnexpectedEnd(_)
    ) && d.error().position().byte_offset().value()
        == source.len()));
    assert!(report.into_validation_result().is_err());
}

#[test]
fn named_only_child_and_logical_suffix_permission_do_not_escape_to_root_or_unrelated_pseudos() {
    for element in elements(argument("Card")) {
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                E(element.clone()),
                P(CssPseudoClass::OnlyChild)
            ])
            .is_some()
        );
        for forbidden in [CssPseudoClass::Current, CssPseudoClass::FirstChild] {
            assert!(
                CssPseudoElementSequence::try_from_segments(vec![E(element.clone()), P(forbidden)])
                    .is_none()
            );
        }
    }
    for element in [
        CssPseudoElement::ViewTransition,
        CssPseudoElement::Before,
        CssPseudoElement::Highlight(CssCustomIdent::try_new("Card").unwrap()),
    ] {
        assert!(
            CssPseudoElementSequence::try_from_segments(vec![
                E(element),
                P(CssPseudoClass::OnlyChild)
            ])
            .is_none()
        );
    }
    for source in [
        "::view-transition-old(Card):only-child",
        "::view-transition-old(Card):not(:only-child)",
        "::view-transition-old(Card):is(:only-child,:hover)",
        "::view-transition-old(Card):where(:only-child)",
        "::part(label)::view-transition-old(Card):only-child",
    ] {
        selector(source);
    }
    for source in [
        "::view-transition:only-child",
        "::before:only-child",
        "::view-transition-old(Card):not(:current)",
        "::part(label)::view-transition-old(Card):first-child",
        "::view-transition-old(Card) > .x",
        "::view-transition .x",
        "::view-transition-group(Card) + .x",
        "::view-transition-group(Card) > ::view-transition-image-pair(Card)",
    ] {
        rejected(source);
    }
}

#[test]
fn typed_parent_child_edges_and_tree_abiding_receivers_keep_names_symbolic() {
    let root = CssPseudoElement::ViewTransition;
    let group = CssPseudoElement::ViewTransitionGroup(argument("A"));
    let pair = CssPseudoElement::ViewTransitionImagePair(argument("B"));
    let old = CssPseudoElement::ViewTransitionOld(argument("C"));
    let new = CssPseudoElement::ViewTransitionNew(argument("D"));
    for edge in [
        vec![root.clone(), group.clone()],
        vec![group.clone(), pair.clone()],
        vec![pair.clone(), old.clone()],
        vec![pair.clone(), new.clone()],
    ] {
        assert!(CssPseudoElementSequence::try_new(edge).is_some());
    }
    assert!(
        CssPseudoElementSequence::try_new(vec![
            root.clone(),
            group.clone(),
            pair.clone(),
            old.clone()
        ])
        .is_some()
    );
    for edge in [
        vec![root.clone(), pair.clone()],
        vec![group.clone(), old.clone()],
        vec![pair.clone(), group],
        vec![old.clone(), new],
        vec![old.clone(), root.clone()],
    ] {
        assert!(CssPseudoElementSequence::try_new(edge).is_none());
    }
    let part = CssPseudoElement::Part(
        CssPartNameList::try_new(vec![CssPartName::try_new("label").unwrap()]).unwrap(),
    );
    let slotted = CssPseudoElement::Slotted(
        CssCompoundSelectorArgument::try_new(CssSelector::Class("item".into())).unwrap(),
    );
    for child in std::iter::once(root).chain(elements(argument("Name"))) {
        assert!(CssPseudoElementSequence::try_new(vec![part.clone(), child.clone()]).is_some());
        assert!(CssPseudoElementSequence::try_new(vec![slotted.clone(), child]).is_some());
    }
    assert_eq!(
        selector("::view-transition-group(A)::view-transition-image-pair(B)")
            .to_specified_css()
            .unwrap(),
        "::view-transition-group(A)::view-transition-image-pair(B)"
    );
}

#[test]
fn retained_invalid_logical_items_require_actual_suffix_permission_inclusion() {
    let generic = retained("::before:is(:NoPe(&))");
    let named = retained("::view-transition-old(Card):is(:NoPe(&))");
    let search = retained("::search-text:is(:NoPe(&))");
    let backed = retained("::details-content:is(:NoPe(&))");
    let receiver = CssPseudoElement::ViewTransitionOld(argument("Card"));
    assert!(attach(receiver.clone(), suffix_list(&named).clone()).is_some());
    assert!(attach(CssPseudoElement::Before, suffix_list(&named).clone()).is_some());
    assert!(attach(receiver.clone(), suffix_list(&generic).clone()).is_none());
    assert!(attach(CssPseudoElement::SearchText, suffix_list(&named).clone()).is_none());
    assert!(attach(receiver.clone(), suffix_list(&search).clone()).is_none());
    assert!(
        attach(
            CssPseudoElement::DetailsContent,
            suffix_list(&named).clone()
        )
        .is_none()
    );
    assert!(attach(receiver, suffix_list(&backed).clone()).is_some());
    let [CssPseudoSelectorListItem::InvalidNesting(item)] = suffix_list(&named).items() else {
        panic!("retained invalid nesting member")
    };
    assert_eq!(item.authored(), ":NoPe(&)");
    assert_eq!(
        item.origin().source().as_str(),
        "::view-transition-old(Card):is(:NoPe(&))"
    );
    assert_eq!(
        named.to_specified_css().unwrap(),
        "::view-transition-old(Card):is(:NoPe(&))"
    );
    let detached = CssSelector::PseudoClass(CssPseudoClass::Is(suffix_list(&named).clone()));
    assert_eq!(
        detached.to_specified_css().unwrap_err().kind(),
        LimitKind::UnrepresentableValue
    );
}

#[test]
fn supports_lexical_snapshot_keeps_function_opener_closer_and_identifier_spans_separate() {
    let source =
        "/*😀*/@supports selector(::view-transition-old(Δa)){.x{view-transition-name:Card}}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("one actual Supports rule")
    };
    let condition = rule.condition();
    let CssSupportsConditionKind::Selector(semantic) = condition.kind() else {
        panic!("typed selector probe")
    };
    assert_eq!(
        segments(semantic),
        &[E(CssPseudoElement::ViewTransitionOld(argument("Δa")))]
    );
    let outer = condition
        .components()
        .iter()
        .find_map(|c| match c.view() {
            CssComponentValueRef::Function(f) => Some((c, f)),
            _ => None,
        })
        .unwrap();
    parsed_range(outer.0.origin(), source, (18, 27), (16, 25));
    let inner = outer
        .1
        .values()
        .items()
        .iter()
        .find_map(|c| match c.view() {
            CssComponentValueRef::Function(f) => Some((c, f)),
            _ => None,
        })
        .unwrap();
    parsed_range(inner.0.origin(), source, (29, 49), (27, 47));
    parsed_range(
        inner.1.values().items()[0].origin(),
        source,
        (49, 52),
        (47, 49),
    );
    parsed_range(inner.1.closing_origin(), source, (52, 53), (49, 50));
    let reused = CssSupportsCondition::try_from_components(
        CssComponentValues::try_new(condition.components().to_vec()).unwrap(),
        &CssNamespaceContext::default(),
    )
    .unwrap();
    assert_eq!(&reused, condition);
    let sheet = CssSheet::try_from_rules(report.syntax().rules().to_vec()).unwrap();
    assert_eq!(sheet.rules(), report.syntax().rules());
    assert_eq!(
        sheet.to_specified_css().unwrap(),
        "@supports selector(::view-transition-old(Δa)) { .x { view-transition-name: Card; } }"
    );
}

#[test]
fn real_nested_scoped_rules_normalize_and_recover_whole_invalid_selector_units() {
    let source = ".p{view-transition-name:Card;@scope(.s){&::view-transition-new(Card):only-child{view-transition-name:none}}}";
    let expected = ".p { view-transition-name: Card; @scope (.s) { &::view-transition-new(Card):only-child { view-transition-name: none; } } }";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{report:?}");
    assert_eq!(report.syntax().to_specified_css().unwrap(), expected);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(item) => Some(value(item.source()).clone()),
            _ => None,
        })
        .collect();
    assert_eq!(values, vec![name("Card"), CssViewTransitionName::None]);
    let bad_source = "::view-transition-group(Card) > .bad{view-transition-name:Card}.after{view-transition-name:none}";
    let bad = parse_sheet(bad_source);
    let [CssRule::Style(after)] = bad.syntax().rules() else {
        panic!("only later rule survives")
    };
    assert_eq!(
        after.selectors().selectors()[0].selector(),
        &CssSelector::Class("after".into())
    );
    assert_eq!(
        value(&after.declarations().as_slice()[0]),
        &CssViewTransitionName::None
    );
    assert!(
        bad.diagnostics()
            .iter()
            .any(|d| d.error().code() == CssErrorCode::InvalidSelector
                && d.action() == CssRecoveryAction::DropQualifiedRule)
    );
    assert!(bad.into_validation_result().is_err());
}

#[test]
fn proposed_scalar_and_selector_list_tariffs_are_cumulative_atomic_and_repeatable() {
    let scalar = name("A B");
    let expected = r"A\ B";
    assert_eq!(
        scalar
            .serialize_specified_with_limits(Limits::new(1, 1, 4))
            .unwrap(),
        expected
    );
    for (limits, cause) in [
        (Limits::new(0, 1, 4), LimitKind::InputNodeLimit),
        (Limits::new(1, 0, 4), LimitKind::ProjectionNodeLimit),
        (Limits::new(1, 1, 3), LimitKind::ByteLimit),
    ] {
        assert_eq!(
            scalar
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(scalar, name("A B"));
    }
    assert_eq!(
        CssViewTransitionName::None
            .serialize_specified_with_limits(Limits::new(1, 1, 4))
            .unwrap(),
        "none"
    );
    let members = vec![
        selector("::view-transition-old(Card)"),
        selector("::view-transition"),
    ];
    let css = "::view-transition-old(Card), ::view-transition";
    let fit = Limits::new(6, 6, css.len());
    let list = CssSelectorList::try_new_with_limits(members.clone(), fit).unwrap();
    let before = list.clone();
    assert_eq!(list.to_specified_css_with_limits(fit).unwrap(), css);
    for (limits, cause) in [
        (Limits::new(5, 6, css.len()), LimitKind::InputNodeLimit),
        (Limits::new(6, 5, css.len()), LimitKind::ProjectionNodeLimit),
        (Limits::new(6, 6, css.len() - 1), LimitKind::ByteLimit),
    ] {
        for member in &members {
            assert!(member.to_specified_css_with_limits(limits).is_ok());
        }
        let error = CssSelectorList::try_new_with_limits(members.clone(), limits).unwrap_err();
        let CssSelectorConstructionErrorKind::Specified(error) = error.kind() else {
            panic!("shared specified cause")
        };
        assert_eq!(error.kind(), cause);
        assert_eq!(
            list.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            cause
        );
        assert_eq!(list, before);
    }
    assert_eq!(list.to_specified_css_with_limits(fit).unwrap(), css);
}

#[test]
fn checked_pending_components_obey_the_real_256_ceiling_and_keep_leaf_origins() {
    let accepted = format!("{}var(--tag){}", "f(".repeat(255), ")".repeat(255));
    let components = parse_component_values(&accepted).unwrap();
    let before = components.clone();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::ViewTransitionName),
        components.clone(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(
        declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .is_some()
    );
    assert_eq!(declaration.value_components(), &before);
    let rejected = format!("{}var(--tag){}", "f(".repeat(256), ")".repeat(256));
    assert_eq!(
        parse_component_values(&rejected).unwrap_err().kind(),
        CssComponentValueErrorKind::NestingLimit
    );
    let CssExpansion::Pending(pending) = expand_declaration(&declaration).unwrap() else {
        panic!("deep authored pending name")
    };
    let replacement =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("Card").unwrap()]).unwrap();
    let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("valid bounded replacement")
    };
    assert_eq!(
        items.items()[0].replacement_components(),
        Some(&replacement)
    );
    assert_eq!(
        replacement.items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
}

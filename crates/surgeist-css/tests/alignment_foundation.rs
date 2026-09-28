#![forbid(unsafe_code)]

//! CSS Box Alignment 3, 2026-01-30, §§4.1–4.4 and 5–7:
//! https://www.w3.org/TR/2026/WD-css-align-3-20260130/
//! The nine authored properties retain their distinct grammars and expand to
//! the six specified longhands before any layout-dependent interpretation.

use surgeist_css::*;

const CONTENT: &str = "align-content";
const JUSTIFY_CONTENT: &str = "justify-content";
const ITEMS: &str = "align-items";
const JUSTIFY_ITEMS: &str = "justify-items";
const SELF: &str = "align-self";
const JUSTIFY_SELF: &str = "justify-self";

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known property {name}"))
}

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn accepted(name: &str, value: &str) {
    let declaration = parsed(name, value);
    let checked = parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"));
    assert_eq!(declaration.known().unwrap().grammar(), grammar(name));
    assert_eq!(checked.known().unwrap().grammar(), grammar(name));
}

fn rejected(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid-value diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(source).unwrap()
    else {
        panic!("intrinsic terminal alignment contributions")
    };
    items.items().to_vec()
}

fn ordinary(name: &str, value: &str) -> CssLonghandValue {
    let items = expanded(&parsed(name, value));
    let [item] = items.as_slice() else {
        panic!("one terminal: {name}:{value}")
    };
    item.ordinary_value().unwrap().clone()
}

fn pair(name: &str, value: &str, first: (&str, &str), second: (&str, &str)) {
    let source = parsed(name, value);
    let items = expanded(&source);
    let [a, b] = items.as_slice() else {
        panic!("two shorthand terminals: {name}:{value}")
    };
    for (item, (terminal, keyword)) in [(a, first), (b, second)] {
        assert_eq!(item.property(), grammar(terminal).target_property());
        assert_eq!(item.ordinary_value(), Some(&ordinary(terminal, keyword)));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
        assert!(item.replacement_components().is_none());
    }
}

#[test]
fn six_longhands_have_distinct_noninherited_initial_values() {
    for (name, initial) in [
        (CONTENT, "normal"),
        (JUSTIFY_CONTENT, "normal"),
        (ITEMS, "normal"),
        (JUSTIFY_ITEMS, "legacy"),
        (SELF, "auto"),
        (JUSTIFY_SELF, "auto"),
    ] {
        let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal longhand")
        };
        assert!(!metadata.inherited_by_default(), "{name}");
        let initial_value = metadata.initial_value();
        let CssInitialValueRef::Value(value) = initial_value.view() else {
            panic!("{name} has a fixed specified initial")
        };
        assert_eq!(value, &ordinary(name, initial));
    }
}

#[test]
fn content_axes_accept_their_own_positions_and_baseline_domain() {
    for value in [
        "baseline",
        "first baseline",
        "baseline first",
        "last baseline",
        "baseline last",
        "space-evenly",
        "safe start",
        "unsafe flex-start",
    ] {
        accepted(CONTENT, value);
    }
    for value in [
        "left",
        "right",
        "self-start",
        "safe normal",
        "unsafe stretch",
    ] {
        rejected(CONTENT, value);
    }
    for value in ["left", "right", "safe left", "unsafe right", "space-around"] {
        accepted(JUSTIFY_CONTENT, value);
    }
    for value in ["baseline", "first baseline", "self-end", "safe normal"] {
        rejected(JUSTIFY_CONTENT, value);
    }
}

#[test]
fn item_and_self_axes_admit_only_their_own_prefixed_positions() {
    for value in [
        "self-start",
        "self-end",
        "safe start",
        "safe flex-start",
        "unsafe self-end",
        "baseline first",
    ] {
        accepted(ITEMS, value);
    }
    for value in [
        "auto",
        "left",
        "safe normal",
        "unsafe stretch",
        "space-between",
    ] {
        rejected(ITEMS, value);
    }
    for name in [SELF, JUSTIFY_SELF] {
        for value in ["auto", "safe normal", "unsafe normal", "safe self-start"] {
            accepted(name, value);
        }
        for value in [
            "safe auto",
            "unsafe baseline",
            "safe stretch",
            "space-around",
        ] {
            rejected(name, value);
        }
    }
    for value in ["left", "right", "unsafe left"] {
        accepted(JUSTIFY_SELF, value);
        rejected(SELF, value);
    }
}

#[test]
fn justify_items_legacy_is_unordered_but_confined_to_its_own_grammar() {
    for value in [
        "legacy",
        "legacy left",
        "left legacy",
        "legacy center",
        "center legacy",
        "safe right",
        "unsafe self-start",
    ] {
        accepted(JUSTIFY_ITEMS, value);
    }
    for value in [
        "legacy start",
        "safe legacy",
        "legacy safe left",
        "auto",
        "safe normal",
    ] {
        rejected(JUSTIFY_ITEMS, value);
    }
    for name in [ITEMS, SELF, JUSTIFY_SELF] {
        rejected(name, "legacy");
    }
}

#[test]
fn three_place_shorthands_have_ordered_settable_pairs_without_reset_members() {
    for (name, first, second) in [
        ("place-content", CONTENT, JUSTIFY_CONTENT),
        ("place-items", ITEMS, JUSTIFY_ITEMS),
        ("place-self", SELF, JUSTIFY_SELF),
    ] {
        let CssPropertyKindRef::Shorthand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a shorthand")
        };
        assert_eq!(
            metadata
                .members()
                .iter()
                .map(|member| member.known_property())
                .collect::<Vec<_>>(),
            [
                grammar(first).target_property(),
                grammar(second).target_property()
            ]
        );
        assert_eq!(metadata.settable_members().len(), 2);
        assert!(metadata.reset_only_members().is_empty());
        assert!(!metadata.is_legacy());
    }
}

#[test]
fn place_content_baseline_defaults_justify_to_start_and_pairs_restrict_each_axis() {
    pair(
        "place-content",
        "baseline",
        (CONTENT, "baseline"),
        (JUSTIFY_CONTENT, "start"),
    );
    pair(
        "place-content",
        "last baseline",
        (CONTENT, "last baseline"),
        (JUSTIFY_CONTENT, "start"),
    );
    pair(
        "place-content",
        "safe center",
        (CONTENT, "safe center"),
        (JUSTIFY_CONTENT, "safe center"),
    );
    pair(
        "place-content",
        "baseline right",
        (CONTENT, "baseline"),
        (JUSTIFY_CONTENT, "right"),
    );
    for value in [
        "baseline baseline",
        "left",
        "left center",
        "normal self-start",
    ] {
        rejected("place-content", value);
    }
}

#[test]
fn place_items_and_self_copy_valid_omissions_and_enforce_component_domains() {
    pair(
        "place-items",
        "stretch",
        (ITEMS, "stretch"),
        (JUSTIFY_ITEMS, "stretch"),
    );
    pair(
        "place-items",
        "normal legacy",
        (ITEMS, "normal"),
        (JUSTIFY_ITEMS, "legacy"),
    );
    pair(
        "place-items",
        "self-end right",
        (ITEMS, "self-end"),
        (JUSTIFY_ITEMS, "right"),
    );
    for value in ["legacy", "legacy normal", "auto", "left right"] {
        rejected("place-items", value);
    }
    pair("place-self", "auto", (SELF, "auto"), (JUSTIFY_SELF, "auto"));
    pair(
        "place-self",
        "safe normal",
        (SELF, "safe normal"),
        (JUSTIFY_SELF, "safe normal"),
    );
    pair(
        "place-self",
        "auto right",
        (SELF, "auto"),
        (JUSTIFY_SELF, "right"),
    );
    for value in ["right auto", "legacy", "auto legacy"] {
        rejected("place-self", value);
    }
}

#[test]
fn css_wide_values_expand_symbolically_for_all_nine_properties() {
    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        for (name, members) in [
            (CONTENT, &[CONTENT][..]),
            (JUSTIFY_CONTENT, &[JUSTIFY_CONTENT][..]),
            (ITEMS, &[ITEMS][..]),
            (JUSTIFY_ITEMS, &[JUSTIFY_ITEMS][..]),
            (SELF, &[SELF][..]),
            (JUSTIFY_SELF, &[JUSTIFY_SELF][..]),
            ("place-content", &[CONTENT, JUSTIFY_CONTENT][..]),
            ("place-items", &[ITEMS, JUSTIFY_ITEMS][..]),
            ("place-self", &[SELF, JUSTIFY_SELF][..]),
        ] {
            let source = parsed(name, spelling);
            let items = expanded(&source);
            assert_eq!(items.len(), members.len(), "{name}:{spelling}");
            for (item, member) in items.iter().zip(members) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn pending_shorthands_reenter_strictly_and_keep_authored_occurrence() {
    for (name, valid, invalid, members) in [
        (
            "place-content",
            "baseline",
            "baseline baseline",
            &[CONTENT, JUSTIFY_CONTENT][..],
        ),
        (
            "place-items",
            "normal legacy",
            "legacy normal",
            &[ITEMS, JUSTIFY_ITEMS][..],
        ),
        (
            "place-self",
            "auto right",
            "right auto",
            &[SELF, JUSTIFY_SELF][..],
        ),
    ] {
        let source = parsed(name, "var(--alignment)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("{name} remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        assert!(matches!(
            pending
                .reenter(parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let replacement = parse_component_values(valid).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to longhands")
            };
            assert_eq!(items.items().len(), 2);
            for (item, member) in items.items().iter().zip(members) {
                assert_eq!(item.property(), grammar(member).target_property());
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
        }
    }
}

#[test]
fn normalization_preserves_mixed_alignment_order_and_fails_at_pair_budget() {
    let report = parse_sheet(
        ".a{align-content:normal;place-content:baseline;justify-content:right;place-items:normal legacy;place-self:auto right}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 5);
    for (index, members) in [
        &[CONTENT][..],
        &[CONTENT, JUSTIFY_CONTENT][..],
        &[JUSTIFY_CONTENT][..],
        &[ITEMS, JUSTIFY_ITEMS][..],
        &[SELF, JUSTIFY_SELF][..],
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(declarations[index].order(), index);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            declarations[index].expansion()
        else {
            panic!("normalized alignment terminals")
        };
        assert_eq!(
            items
                .items()
                .iter()
                .map(|item| item.property())
                .collect::<Vec<_>>(),
            members
                .iter()
                .map(|name| grammar(name).target_property())
                .collect::<Vec<_>>()
        );
        assert!(
            items
                .items()
                .iter()
                .all(|item| item.source().same_occurrence(declarations[index].source()))
        );
    }
    let one = parse_sheet(".a{place-content:baseline}");
    assert!(one.is_clean());
    let limits = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
    let error = normalize_sheet_with_limits(one.syntax(), limits).unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 1,
        }
    );
    assert_eq!(error.declaration_order(), Some(0));
    assert_eq!(
        error
            .declaration()
            .unwrap()
            .known()
            .unwrap()
            .grammar()
            .name(),
        "place-content"
    );
}

//! CSS Box Alignment 3, 2026-01-30, §§4.1–4.4 and 5–7:
//! https://www.w3.org/TR/2026/WD-css-align-3-20260130/
//! The nine authored properties retain their distinct grammars and expand to
//! the six specified longhands before any layout-dependent interpretation.

use crate::css::support::property_admission::Responsible::At;
use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{important_declaration as parsed, selected_grammar as grammar};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::normalize::{
    CssNormalizationLimits, CssNormalizationResource, CssNormalizedItem,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssLonghandContribution, CssLonghandValue,
    CssPropertyKindRef, CssPropertyTarget,
};

const CONTENT: &str = "align-content";
const JUSTIFY_CONTENT: &str = "justify-content";
const ITEMS: &str = "align-items";
const JUSTIFY_ITEMS: &str = "justify-items";
const SELF: &str = "align-self";
const JUSTIFY_SELF: &str = "justify-self";

fn accepted(name: &str, value: &str) {
    let declaration = parsed(name, value);
    let checked = CssDeclaration::try_from_components(
        &CssParser::default(),
        CssPropertyTarget::Grammar(grammar(name)),
        CssComponentValues::parse(&CssParser::default(), value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"));
    assert_eq!(declaration.known().unwrap().grammar(), grammar(name));
    assert_eq!(checked.known().unwrap().grammar(), grammar(name));
}

fn expanded(source: &CssDeclaration) -> Vec<CssLonghandContribution> {
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = source.expand().unwrap()
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
    for (value, token) in [
        ("left", At("left")),
        ("right", At("right")),
        ("self-start", At("self-start")),
        ("safe normal", At("normal")),
        ("unsafe stretch", At("stretch")),
    ] {
        rejected(CONTENT, value, token);
    }
    for value in ["left", "right", "safe left", "unsafe right", "space-around"] {
        accepted(JUSTIFY_CONTENT, value);
    }
    for (value, token) in [
        ("baseline", At("baseline")),
        ("first baseline", At("first")),
        ("self-end", At("self-end")),
        ("safe normal", At("normal")),
    ] {
        rejected(JUSTIFY_CONTENT, value, token);
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
    for (value, token) in [
        ("auto", At("auto")),
        ("left", At("left")),
        ("safe normal", At("normal")),
        ("unsafe stretch", At("stretch")),
        ("space-between", At("space-between")),
    ] {
        rejected(ITEMS, value, token);
    }
    for name in [SELF, JUSTIFY_SELF] {
        for value in ["auto", "safe normal", "unsafe normal", "safe self-start"] {
            accepted(name, value);
        }
        for (value, token) in [
            ("safe auto", At("auto")),
            ("unsafe baseline", At("baseline")),
            ("safe stretch", At("stretch")),
            ("space-around", At("space-around")),
        ] {
            rejected(name, value, token);
        }
    }
    for (value, token) in [
        ("left", At("left")),
        ("right", At("right")),
        ("unsafe left", At("left")),
    ] {
        accepted(JUSTIFY_SELF, value);
        rejected(SELF, value, token);
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
    for (value, token) in [
        ("legacy start", At("start")),
        ("safe legacy", At("legacy")),
        ("legacy safe left", At("safe")),
        ("auto", At("auto")),
        ("safe normal", At("normal")),
    ] {
        rejected(JUSTIFY_ITEMS, value, token);
    }
    for name in [ITEMS, SELF, JUSTIFY_SELF] {
        rejected(name, "legacy", Some("legacy"));
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
    for (value, token) in [
        ("baseline baseline", Some("baseline")),
        ("left", Some("left")),
        ("left center", Some("left")),
        ("normal self-start", Some("self-start")),
    ] {
        rejected("place-content", value, token);
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
    for (value, token) in [
        ("legacy", At("legacy")),
        ("legacy normal", At("legacy")),
        ("auto", At("auto")),
        ("left right", At("left")),
    ] {
        rejected("place-items", value, token);
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
    for (value, token) in [
        ("right auto", At("right")),
        ("legacy", At("legacy")),
        ("auto legacy", At("legacy")),
    ] {
        rejected("place-self", value, token);
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
        let CssExpansion::Pending(pending) = source.expand().unwrap() else {
            panic!("{name} remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        assert!(
            pending
                .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some()
        );
        assert!(
            pending
                .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
                .unwrap_err()
                .is_residual_substitution()
        );
        let replacement = CssComponentValues::parse(&CssParser::default(), valid).unwrap();
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
    let report = CssParser::default().parse_sheet(".a{align-content:normal;place-content:baseline;justify-content:right;place-items:normal legacy;place-self:auto right}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
    let one = CssParser::default().parse_sheet(".a{place-content:baseline}");
    assert!(one.is_clean());
    let limits = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
    let error = one.syntax().normalize(limits).unwrap_err();
    assert_eq!(
        error.exhausted_limit(),
        Some((CssNormalizationResource::Contributions, 1))
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

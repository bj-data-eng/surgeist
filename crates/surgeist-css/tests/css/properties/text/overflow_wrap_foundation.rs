//! CSS Text 4 Working Draft (2026-08-14) §6.4 behavior and the selected
//! CSS Text 3 Candidate Recommendation Draft (2026-08-14) support source.
//! Exact typed payloads belong to later API tests.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::important_declaration as declaration;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::*;

fn one_ordinary(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        source.expand().expect("overflow-wrap expands")
    else {
        panic!("one ordinary overflow-wrap contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::OverflowWrap);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    item.ordinary_value().expect("ordinary keyword").clone()
}

#[test]
fn word_wrap_is_a_case_insensitive_legacy_name_alias_with_selected_support() {
    let canonical = grammar("overflow-wrap");
    let alias = grammar("word-wrap");
    assert_eq!(
        CssKnownProperty::from_name("word-wrap"),
        Some(CssKnownProperty::OverflowWrap)
    );
    assert_eq!(
        CssKnownProperty::from_name("WORD-WRAP"),
        Some(CssKnownProperty::OverflowWrap)
    );
    assert_eq!(alias, canonical);
    assert_eq!(alias.name(), "overflow-wrap");
    assert_eq!(alias.target_property(), CssKnownProperty::OverflowWrap);
    assert_eq!(alias.feature_id(), canonical.feature_id());
    assert_eq!(CssKnownProperty::OverflowWrap.aliases(), &["word-wrap"]);

    for name in ["overflow-wrap", "word-wrap"] {
        let support = property_support_metadata(name).expect("property support");
        assert_eq!(support.property(), CssKnownProperty::OverflowWrap);
        assert_eq!(support.canonical_name(), "overflow-wrap");
        assert_eq!(support.aliases(), &["word-wrap"]);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "S-TEXT3");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/CRD-css-text-3-20260814/")
        );
        assert_eq!(support.feature().production(), "#propdef-overflow-wrap");
    }

    let authored = declaration("WORD-WRAP", "anywhere");
    assert_eq!(
        authored.known().unwrap().property(),
        CssKnownProperty::OverflowWrap
    );
    assert!(
        authored.parsed_name().is_some(),
        "alias spelling retains provenance"
    );
    assert_eq!(
        checked("word-wrap", "anywhere").known().unwrap().property(),
        CssKnownProperty::OverflowWrap
    );
}

#[test]
fn overflow_wrap_is_an_inherited_longhand_with_normal_initial() {
    let CssPropertyKindRef::Longhand(longhand) = grammar("overflow-wrap")
        .metadata()
        .expect("overflow-wrap metadata")
        .kind()
    else {
        panic!("overflow-wrap is not a shorthand")
    };
    assert_eq!(
        longhand.property().known_property(),
        CssKnownProperty::OverflowWrap
    );
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::OverflowWrap
    );
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("normal is a fixed initial")
    };
    assert_eq!(
        value,
        &one_ordinary(&declaration("overflow-wrap", "normal"))
    );
}

#[test]
fn canonical_grammar_accepts_three_keywords_and_recovers_from_other_values() {
    let name = "overflow-wrap";
    for value in ["normal", "break-word", "anywhere", "ANYWHERE"] {
        for source in [declaration(name, value), checked(name, value)] {
            assert_eq!(
                source.known().unwrap().property(),
                CssKnownProperty::OverflowWrap
            );
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
    for value in ["none", "break-all", "normal anywhere", "1px"] {
        let source = format!("color:red;{name}:{value};color:blue");
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid-value diagnostic: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
        assert!(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(grammar(name)),
                CssComponentValues::parse(&CssParser::default(), value).unwrap(),
                CssImportance::Normal
            )
            .is_err(),
            "checked construction accepted {name}:{value}"
        );
    }
}

#[test]
fn alias_and_canonical_keywords_expand_once_with_original_source() {
    for (name, value) in [("overflow-wrap", "break-word"), ("word-wrap", "anywhere")] {
        for source in [declaration(name, value), checked(name, value)] {
            assert_eq!(
                one_ordinary(&source),
                one_ordinary(&declaration("overflow-wrap", value))
            );
        }
    }
    let alias = declaration("word-wrap", "anywhere");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = alias.expand().unwrap()
    else {
        panic!("alias contributes to canonical longhand")
    };
    let [item] = values.items() else {
        panic!("alias makes one contribution")
    };
    assert!(item.source().same_occurrence(&alias));
    assert!(item.source().parsed_name().is_some());
    assert!(item.replacement_components().is_none());

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration("word-wrap", spelling);
        checked("word-wrap", spelling);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            source.expand().unwrap()
        else {
            panic!("one symbolic global contribution")
        };
        let [item] = values.items() else {
            panic!("one global contribution")
        };
        assert_eq!(item.property(), CssKnownProperty::OverflowWrap);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn pending_alias_reentry_completes_once_and_rejects_invalid_replacements() {
    let source = declaration("word-wrap", "var(--wrap)");
    let CssExpansion::Pending(pending) = source.expand().unwrap() else {
        panic!("substitution remains pending")
    };
    assert!(pending.source().same_occurrence(&source));
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    assert!(
        pending
            .reenter(CssComponentValues::parse(&CssParser::default(), "none").unwrap())
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
    let replacement = CssComponentValues::parse(&CssParser::default(), "normal").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("replacement completes canonical longhand")
    };
    let [item] = values.items() else {
        panic!("one reentered contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::OverflowWrap);
    assert_eq!(
        item.ordinary_value(),
        Some(&one_ordinary(&declaration("overflow-wrap", "normal")))
    );
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert_eq!(item.replacement_components(), Some(&replacement));
}

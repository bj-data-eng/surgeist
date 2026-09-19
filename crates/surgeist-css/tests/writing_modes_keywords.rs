#![forbid(unsafe_code)]
//! Canonical Writing Modes keyword contracts.
//! WM3 REC2019-12-10 §§2/3/5 and WM4 CR2019-07-30 §3.2 require these
//! 16 keywords. No glyph alias grammar or math-substitution assertion is made.
use CssKnownProperty as P;
use surgeist_css::*;

#[derive(Clone, Copy)]
enum Expected {
    Direction(CssDirection),
    Bidi(CssUnicodeBidi),
    Writing(CssWritingMode),
    Orientation(CssTextOrientation),
}

const KEYWORDS: &[(P, &str, Expected)] = &[
    (P::Direction, "ltr", Expected::Direction(CssDirection::Ltr)),
    (P::Direction, "rtl", Expected::Direction(CssDirection::Rtl)),
    (
        P::UnicodeBidi,
        "normal",
        Expected::Bidi(CssUnicodeBidi::Normal),
    ),
    (
        P::UnicodeBidi,
        "embed",
        Expected::Bidi(CssUnicodeBidi::Embed),
    ),
    (
        P::UnicodeBidi,
        "isolate",
        Expected::Bidi(CssUnicodeBidi::Isolate),
    ),
    (
        P::UnicodeBidi,
        "bidi-override",
        Expected::Bidi(CssUnicodeBidi::BidiOverride),
    ),
    (
        P::UnicodeBidi,
        "isolate-override",
        Expected::Bidi(CssUnicodeBidi::IsolateOverride),
    ),
    (
        P::UnicodeBidi,
        "plaintext",
        Expected::Bidi(CssUnicodeBidi::Plaintext),
    ),
    (
        P::WritingMode,
        "horizontal-tb",
        Expected::Writing(CssWritingMode::HorizontalTb),
    ),
    (
        P::WritingMode,
        "vertical-rl",
        Expected::Writing(CssWritingMode::VerticalRl),
    ),
    (
        P::WritingMode,
        "vertical-lr",
        Expected::Writing(CssWritingMode::VerticalLr),
    ),
    (
        P::WritingMode,
        "sideways-rl",
        Expected::Writing(CssWritingMode::SidewaysRl),
    ),
    (
        P::WritingMode,
        "sideways-lr",
        Expected::Writing(CssWritingMode::SidewaysLr),
    ),
    (
        P::TextOrientation,
        "mixed",
        Expected::Orientation(CssTextOrientation::Mixed),
    ),
    (
        P::TextOrientation,
        "upright",
        Expected::Orientation(CssTextOrientation::Upright),
    ),
    (
        P::TextOrientation,
        "sideways",
        Expected::Orientation(CssTextOrientation::Sideways),
    ),
];

fn declaration(property: P, text: &str, importance: CssImportance) -> CssDeclaration {
    let suffix = if importance == CssImportance::Important {
        "!important"
    } else {
        ""
    };
    let report = parse_style_attribute(&format!("{}:{text}{suffix}", property.canonical_name()));
    assert!(
        report.is_clean(),
        "{property:?}:{text}: {:?}",
        report.diagnostics()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}

fn assert_keyword(declaration: &CssDeclaration, text: &str, expected: Expected) {
    let raw = match (
        declaration.known().unwrap().property_value().unwrap(),
        expected,
    ) {
        (CssKnownPropertyValueRef::Direction(value), Expected::Direction(expected)) => {
            assert_eq!(value.i01_subset(), Some(&expected));
            value.as_css()
        }
        (CssKnownPropertyValueRef::UnicodeBidi(value), Expected::Bidi(expected)) => {
            assert_eq!(value.bidi(), &expected);
            value.as_css()
        }
        (CssKnownPropertyValueRef::WritingMode(value), Expected::Writing(expected)) => {
            assert_eq!(value.i01_subset(), Some(&expected));
            value.as_css()
        }
        (CssKnownPropertyValueRef::TextOrientation(value), Expected::Orientation(expected)) => {
            assert_eq!(value.orientation(), &expected);
            value.as_css()
        }
        _ => panic!("independently expected typed keyword"),
    };
    assert_eq!(raw, text);
}

fn metadata(property: P, inherited: bool) {
    let metadata = property
        .metadata()
        .expect("canonical keyword longhand metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("one longhand")
    };
    assert_eq!(longhand.inherited_by_default(), inherited);
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), property);
    assert!(matches!(initial.view(), CssInitialValueRef::Value(_)));
    // Exact Ltr/Normal/HorizontalTb initial payload proof requires the real new
    // longhand branch and belongs to functional tests, not a parser oracle here.
}

#[test]
fn direction_metadata_is_inherited_with_ordinary_initial() {
    metadata(P::Direction, true);
}
#[test]
fn unicode_bidi_metadata_is_noninherited_with_ordinary_initial() {
    metadata(P::UnicodeBidi, false);
}
#[test]
fn writing_mode_metadata_is_inherited_with_ordinary_initial() {
    metadata(P::WritingMode, true);
}

fn expansion_states(property: P, reentry_keyword: &str) {
    for importance in [CssImportance::Normal, CssImportance::Important] {
        for &(owner, text, _) in KEYWORDS {
            if owner != property {
                continue;
            }
            let parsed = declaration(property, text, importance);
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                importance,
            )
            .unwrap();
            assert_eq!(checked.value_components(), &components);
            for source in [parsed, checked] {
                let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                    expand_declaration(&source).expect("ordinary keyword expansion")
                else {
                    panic!("longhand expansion")
                };
                let [value] = values.items() else {
                    panic!("one contribution")
                };
                assert_eq!(value.property(), property);
                assert_eq!(
                    value.ordinary_value().unwrap().property().known_property(),
                    property
                );
                assert!(value.source().same_occurrence(&source));
                assert_eq!(value.source().importance(), importance);
                assert!(value.replacement_components().is_none());
            }
        }
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(property, text, importance);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("global expansion")
            };
            let [value] = values.items() else {
                panic!("one contribution")
            };
            assert_eq!(value.property(), property);
            assert_eq!(value.value(), CssContributionValueRef::Global(keyword));
            assert!(value.source().same_occurrence(&source));
            assert_eq!(value.source().importance(), importance);
        }
        let source = declaration(property, "var(--mode)", importance);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending property")
        };
        assert!(pending.source().same_occurrence(&source));
        let replacement = parse_component_values(reentry_keyword).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("replacement contribution")
        };
        let [value] = values.items() else {
            panic!("one replacement")
        };
        assert_eq!(value.property(), property);
        assert_eq!(
            value.ordinary_value().unwrap().property().known_property(),
            property
        );
        assert!(value.source().same_occurrence(&source));
        assert_eq!(value.source().importance(), importance);
        assert_eq!(value.replacement_components(), Some(&replacement));
        let CssContributions::Longhands(globals) = pending
            .reenter(parse_component_values("inherit").unwrap())
            .unwrap()
        else {
            panic!("global reentry")
        };
        assert_eq!(globals.items().len(), 1);
        assert_eq!(globals.items()[0].property(), property);
        assert_eq!(
            globals.items()[0].value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
        );
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        for invalid in ["unknown", "inherit extra", "rtl; color:red"] {
            assert!(matches!(
                pending
                    .reenter(parse_component_values(invalid).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
    }
}

#[test]
fn direction_ordinary_global_and_pending_states_expand() {
    expansion_states(P::Direction, "rtl");
}
#[test]
fn unicode_bidi_ordinary_global_and_pending_states_expand() {
    expansion_states(P::UnicodeBidi, "isolate-override");
}
#[test]
fn writing_mode_ordinary_global_and_pending_states_expand() {
    expansion_states(P::WritingMode, "sideways-lr");
}

#[test]
fn sixteen_existing_keywords_have_exact_parsed_and_checked_views() {
    assert_eq!(KEYWORDS.len(), 16);
    for &(property, text, expected) in KEYWORDS {
        for importance in [CssImportance::Normal, CssImportance::Important] {
            let parsed = declaration(property, text, importance);
            assert_keyword(&parsed, text, expected);
            assert_eq!(parsed.importance(), importance);
            assert!(matches!(
                parsed.value_components().items()[0].origin(),
                CssValueOrigin::Parsed(_)
            ));
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()])
                    .unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                importance,
            )
            .unwrap();
            assert_keyword(&checked, text, expected);
            assert_eq!(checked.value_components(), &components);
            assert!(matches!(
                components.items()[0].origin(),
                CssValueOrigin::Programmatic
            ));
        }
    }
    for (property, text, expected) in [
        (
            P::WritingMode,
            "SIDEWAYS-RL",
            Expected::Writing(CssWritingMode::SidewaysRl),
        ),
        (
            P::WritingMode,
            r"sideways-\6c r",
            Expected::Writing(CssWritingMode::SidewaysLr),
        ),
    ] {
        let source = declaration(property, text, CssImportance::Normal);
        assert_keyword(&source, text, expected);
    }
}

#[test]
fn existing_text_orientation_metadata_and_transport_remain_complete() {
    metadata(P::TextOrientation, true);
    let metadata = P::TextOrientation.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("orientation longhand")
    };
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary initial")
    };
    let CssLonghandValueRef::TextOrientation(value) = initial.view() else {
        panic!("orientation initial")
    };
    assert_eq!(value, &CssTextOrientation::Mixed);
    expansion_states(P::TextOrientation, "upright");
}

#[test]
fn all_reset_exclusions_do_not_change_with_keyword_metadata() {
    let source = declaration(P::All, "initial", CssImportance::Important);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    let metadata = P::All.metadata().unwrap();
    let CssPropertyKindRef::UniversalReset(metadata) = metadata.kind() else {
        panic!("reset metadata")
    };
    for (property, excluded) in [
        (P::Direction, true),
        (P::UnicodeBidi, true),
        (P::WritingMode, false),
        (P::TextOrientation, false),
    ] {
        assert_eq!(
            reset.excludes(CssPropertyNameRef::Known(property)),
            excluded
        );
        assert_eq!(
            metadata.excludes(CssPropertyNameRef::Known(property)),
            excluded
        );
    }
    let custom = CssCustomPropertyName::try_new("--writing-mode").unwrap();
    assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));
    assert!(metadata.excludes(CssPropertyNameRef::Custom(&custom)));
    assert!(reset.source().same_occurrence(&source));
    assert_eq!(reset.source().importance(), CssImportance::Important);
}

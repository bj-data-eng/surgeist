//! New authored models and specified serialization from Speech 1 §§7.1–7.2.
//! These tests accompany functional new APIs; preimplementation parser RED is
//! retained separately in speech_keywords_lifecycle.rs.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use surgeist_css::CssParser;
use surgeist_css::conformance::{
    CssExclusionReason, CssSpecificationTier, CssSupportStatus, conformance_exclusion,
    feature_metadata,
};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssLonghandValueRef, CssPropertyKindRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::speech::{CssSpeak, CssSpeakAs, CssSpeakAsPunctuation};

fn speak_as(text: &str) -> CssSpeakAs {
    let report = CssParser::default().parse_block_contents(format!("speak-as:{text}"));
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let CssKnownPropertyValueRef::SpeakAs(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("speak-as wrapper")
    };
    *value.value()
}

#[test]
fn checked_speak_as_rejects_empty_modifiers_and_preserves_every_authored_choice() {
    assert!(CssSpeakAs::try_new(false, false, None).is_none());
    let normal = CssSpeakAs::normal();
    assert!(normal.is_normal());
    assert!(!normal.spell_out());
    assert!(!normal.digits());
    assert_eq!(normal.punctuation(), None);
    assert_eq!(
        normal.serialize_specified(Default::default()).unwrap(),
        "normal"
    );
    assert_eq!(normal, speak_as("normal"));
    for (spell_out, digits, punctuation, expected) in [
        (true, false, None, "spell-out"),
        (false, true, None, "digits"),
        (true, true, None, "spell-out digits"),
        (
            false,
            false,
            Some(CssSpeakAsPunctuation::Literal),
            "literal-punctuation",
        ),
        (
            true,
            false,
            Some(CssSpeakAsPunctuation::Literal),
            "spell-out literal-punctuation",
        ),
        (
            false,
            true,
            Some(CssSpeakAsPunctuation::Literal),
            "digits literal-punctuation",
        ),
        (
            true,
            true,
            Some(CssSpeakAsPunctuation::Literal),
            "spell-out digits literal-punctuation",
        ),
        (
            false,
            false,
            Some(CssSpeakAsPunctuation::None),
            "no-punctuation",
        ),
        (
            true,
            false,
            Some(CssSpeakAsPunctuation::None),
            "spell-out no-punctuation",
        ),
        (
            false,
            true,
            Some(CssSpeakAsPunctuation::None),
            "digits no-punctuation",
        ),
        (
            true,
            true,
            Some(CssSpeakAsPunctuation::None),
            "spell-out digits no-punctuation",
        ),
    ] {
        let value = CssSpeakAs::try_new(spell_out, digits, punctuation).unwrap();
        assert!(!value.is_normal());
        assert_eq!(value.spell_out(), spell_out);
        assert_eq!(value.digits(), digits);
        assert_eq!(value.punctuation(), punctuation);
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(speak_as(expected), value);
    }
}

#[test]
fn specified_speech_keywords_canonicalize_order_case_and_escapes_without_rewriting_authored_tokens()
{
    let authored = r"NO-PUNCTUATION d\69 gits SPELL-OUT";
    let report =
        CssParser::default().parse_block_contents(format!("speak-as:{authored}!important"));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let source = &report.syntax()[0];
    let CssKnownPropertyValueRef::SpeakAs(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("speak-as")
    };
    let expected = "spell-out digits no-punctuation";
    assert_eq!(value.as_css(), authored);
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        expected
    );
    assert_eq!(*value.value(), speak_as(expected));
    let CssExpansion::Contributions(CssContributions::Longhands(items)) = source.expand().unwrap()
    else {
        panic!("ordinary terminal")
    };
    let [item] = items.items() else {
        panic!("one terminal")
    };
    let CssLonghandValueRef::SpeakAs(expanded) = item.ordinary_value().unwrap().view() else {
        panic!("speak-as terminal")
    };
    assert_eq!(expanded, value.value());
    assert_eq!(
        expanded.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert!(item.source().same_occurrence(source));
    assert_eq!(value.as_css(), authored);
    assert_eq!(
        source
            .value_components()
            .serialize_tokens(Default::default())
            .unwrap()
            .as_css(),
        authored
    );
    for (text, expected_value, expected_css) in [
        ("AUTO", CssSpeak::Auto, "auto"),
        (r"n\65 ver", CssSpeak::Never, "never"),
        ("ALWAYS", CssSpeak::Always, "always"),
    ] {
        let report = CssParser::default().parse_block_contents(format!("speak:{text}"));
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::Speak(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("speak")
        };
        assert_eq!(*value.value(), expected_value);
        assert_eq!(
            value
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            expected_css
        );
        assert_eq!(value.as_css(), text);
    }
}

#[test]
fn speech_initials_are_intrinsic_auto_and_normal_without_authored_occurrences() {
    for property in [CssKnownProperty::Speak, CssKnownProperty::SpeakAs] {
        let metadata = property.metadata().unwrap();
        let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
            panic!("terminal")
        };
        assert!(longhand.inherited_by_default());
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("intrinsic initial")
        };
        match (property, value.view()) {
            (CssKnownProperty::Speak, CssLonghandValueRef::Speak(value)) => {
                assert_eq!(*value, CssSpeak::Auto)
            }
            (CssKnownProperty::SpeakAs, CssLonghandValueRef::SpeakAs(value)) => {
                assert_eq!(*value, CssSpeakAs::normal())
            }
            _ => panic!("initial property/value coupling"),
        }
    }
}

#[test]
fn speech_specified_serialization_obeys_exact_leaf_and_byte_limits_atomically() {
    let compound = CssSpeakAs::try_new(true, true, Some(CssSpeakAsPunctuation::Literal)).unwrap();
    let expected = "spell-out digits literal-punctuation";
    assert_eq!(
        compound
            .serialize_specified(CssSerializationLimits::new(1, 1, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(0, 1, expected.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 0, expected.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(1, 1, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&compound.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    assert_eq!(
        compound.serialize_specified(Default::default()).unwrap(),
        expected
    );
    for (value, css) in [
        (CssSpeak::Auto, "auto"),
        (CssSpeak::Never, "never"),
        (CssSpeak::Always, "always"),
    ] {
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, css.len()))
                .unwrap(),
            css
        );
        assert!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, css.len() - 1))
                .unwrap_err()
                .is_byte_limit()
        );
    }
}

#[test]
fn speech_property_support_reports_its_exact_normative_source_and_keeps_counter_style_distinct() {
    for (property, id, production, text) in [
        (
            CssKnownProperty::Speak,
            "official.property.speak",
            "#propdef-speak",
            "always",
        ),
        (
            CssKnownProperty::SpeakAs,
            "official.property.speak-as",
            "#propdef-speak-as",
            "digits",
        ),
    ] {
        let feature = feature_metadata(id).unwrap();
        assert_eq!(feature.source().id().as_str(), "S-SPEECH1");
        assert_eq!(
            feature.source().url(),
            Some("https://www.w3.org/TR/2023/CRD-css-speech-1-20230214/")
        );
        assert_eq!(
            feature.source().tier(),
            CssSpecificationTier::Snapshot2026Stable
        );
        assert_eq!(feature.production(), production);
        assert_eq!(feature.status(), CssSupportStatus::Complete);
        let report = CssParser::default()
            .parse_block_contents(format!("{}:{text}", property.canonical_name()));
        assert!(report.is_clean());
        assert_eq!(report.syntax()[0].known().unwrap().property(), property);
    }
    let descriptor = feature_metadata("official.descriptor.counter-style.speak-as").unwrap();
    assert_eq!(descriptor.source().id().as_str(), "O-COUNTERSTYLES3");
    assert!(
        CssParser::default()
            .parse_sheet("@counter-style spoken{system:cyclic;symbols:x;speak-as:words}")
            .is_clean()
    );
    assert!(
        !CssParser::default()
            .parse_block_contents("speak-as:words")
            .is_clean()
    );
    let historical = conformance_exclusion("excluded.O-CSS2.informative-property.speak").unwrap();
    assert_eq!(historical.reason(), CssExclusionReason::InformativeOnly);
}

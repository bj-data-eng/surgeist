#![forbid(unsafe_code)]

//! Functional coverage of the shared descriptor providers consumed by loading.
//! Fonts 4 (2026-09-07) owns the concrete grammars and section 13.2 range
//! shortening. CSSOM owns strings; shared specified providers own work/byte
//! limits. No face lookup, initial-value resolution or resource load is performed.

use surgeist_css::*;

type Kind = CssFontFaceDescriptorKind;
type Value = CssFontFaceDescriptorValue;
type Authored = CssAuthoredFontFaceDescriptorValue;
type Limits = CssSpecifiedValueSerializationLimits;
type ErrorKind = CssSpecifiedValueSerializationErrorKind;

fn cases() -> [(Kind, &'static str, &'static str); 14] {
    [
        (Kind::FontFamily, "'Demo Font'", "Demo Font"),
        (
            Kind::Src,
            "local(Demo), url(face.woff2)",
            "local(\"Demo\"), url(\"face.woff2\")",
        ),
        (Kind::FontWeight, "400 4e2", "400"),
        (Kind::FontStyle, "oblique 10deg 1e1deg", "oblique 10deg"),
        (Kind::FontWidth, "75% 75.0%", "75%"),
        (Kind::FontDisplay, "SWAP", "swap"),
        (Kind::UnicodeRange, "u+4??, u+a0", "U+400-4FF, U+A0"),
        (
            Kind::FontFeatureSettings,
            "'liga' off, 'kern' on",
            "\"liga\" off, \"kern\" on",
        ),
        (Kind::FontVariationSettings, "'wght' 625.0", "\"wght\" 625"),
        (Kind::FontNamedInstance, "'Regular'", "\"Regular\""),
        (Kind::FontLanguageOverride, "'ENG'", "\"ENG\""),
        (Kind::AscentOverride, "125.0%", "125%"),
        (Kind::DescentOverride, "NORMAL", "normal"),
        (Kind::LineGapOverride, "0%", "0%"),
    ]
}

fn ordinary(source: &str, kind: Kind) -> Value {
    let value = parse_font_face_descriptor_value(source, kind)
        .into_validation_result()
        .unwrap()
        .unwrap();
    let Authored::Ordinary(value) = value else {
        panic!("ordinary descriptor: {source}")
    };
    value
}

fn assert_atomic_limits(value: &Value, expected: &str, nodes: usize) {
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(nodes, nodes, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, error) in [
        (
            Limits::new(nodes - 1, nodes, expected.len()),
            ErrorKind::InputNodeLimit,
        ),
        (
            Limits::new(nodes, nodes - 1, expected.len()),
            ErrorKind::ProjectionNodeLimit,
        ),
        (
            Limits::new(nodes, nodes, expected.len() - 1),
            ErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            error
        );
        assert_eq!(value, &before);
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

#[test]
fn every_descriptor_emits_the_same_canonical_value_after_raw_and_checked_admission() {
    for (kind, source, expected) in cases() {
        let value = ordinary(source, kind);
        assert_eq!(value.kind(), kind);
        assert_eq!(value.serialize_specified().unwrap(), expected, "{kind:?}");
        let checked =
            Authored::try_from_components(kind, parse_component_values(source).unwrap()).unwrap();
        let Authored::Ordinary(checked) = checked else {
            panic!("checked ordinary descriptor")
        };
        assert_eq!(checked.serialize_specified().unwrap(), expected);
        let before = value.clone();
        for (limits, error) in [
            (
                Limits::new(0, usize::MAX, usize::MAX),
                ErrorKind::InputNodeLimit,
            ),
            (
                Limits::new(usize::MAX, 0, usize::MAX),
                ErrorKind::ProjectionNodeLimit,
            ),
            (
                Limits::new(usize::MAX, usize::MAX, expected.len() - 1),
                ErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
            assert_eq!(value, before);
        }
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn environment_pending_values_reenter_each_concrete_grammar_without_mutating_authored_input() {
    for (kind, replacement, expected) in cases() {
        let source = "/*😀*/env(face-value)";
        let raw = parse_font_face_descriptor_value(source, kind)
            .into_validation_result()
            .unwrap()
            .unwrap();
        let checked =
            Authored::try_from_components(kind, parse_component_values(source).unwrap()).unwrap();
        for authored in [raw, checked] {
            let Authored::Pending(pending) = authored else {
                panic!("whole descriptor pending")
            };
            assert_eq!(pending.kind(), kind);
            let before = pending.components().clone();
            let value = pending
                .reparse_after_substitution(parse_component_values(replacement).unwrap())
                .unwrap();
            assert_eq!(value.kind(), kind);
            assert_eq!(value.serialize_specified().unwrap(), expected);
            for residual in ["env(face-value)", "var(--face-value)"] {
                assert!(
                    pending
                        .reparse_after_substitution(parse_component_values(residual).unwrap())
                        .is_err()
                );
            }
            assert_eq!(pending.components(), &before);
            let function = pending
                .components()
                .items()
                .iter()
                .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
                .unwrap();
            let CssValueOrigin::Parsed(origin) = function.origin() else {
                panic!("original pending function origin")
            };
            assert_eq!(origin.source().as_str(), source);
            assert_eq!(
                origin.span().start().byte_offset().value(),
                source.find("env").unwrap()
            );
        }
    }
}

#[test]
fn descriptor_boundaries_and_stylesheet_only_substitution_do_not_become_ordinary_values() {
    for (kind, source, _) in cases() {
        for invalid in [
            "initial".to_owned(),
            "var(--face-value)".to_owned(),
            format!("{source}!important"),
            format!("{source};"),
        ] {
            let report = parse_font_face_descriptor_value(&invalid, kind);
            assert!(report.syntax().is_none(), "{kind:?}: {invalid}: {report:?}");
            assert!(report.into_validation_result().is_err());
            assert!(
                Authored::try_from_components(kind, parse_component_values(&invalid).unwrap())
                    .is_err()
            );
        }
    }
}

#[test]
fn list_members_and_suppressed_equal_endpoints_share_cumulative_visit_limits() {
    // A Unicode list charges its list and two retained ranges. Range providers
    // charge their wrapper and both endpoints, even when Fonts 4 shortens output.
    for (kind, source, expected, nodes) in [
        (Kind::UnicodeRange, "U+0-7F,U+A0", "U+0-7F, U+A0", 3),
        (Kind::FontWeight, "400 4e2", "400", 3),
        (Kind::FontWidth, "75% 75.0%", "75%", 3),
        (Kind::FontStyle, "oblique 10deg 1e1deg", "oblique 10deg", 3),
        // Two source wrappers and their strings, plus the containing list.
        (
            Kind::Src,
            "local(First),local(Second)",
            "local(\"First\"), local(\"Second\")",
            5,
        ),
        // Each setting retains its tag and its explicit on/off value.
        (
            Kind::FontFeatureSettings,
            "'liga' off,'kern' on",
            "\"liga\" off, \"kern\" on",
            4,
        ),
        (
            Kind::FontVariationSettings,
            "'wght' 400,'wdth' 75",
            "\"wght\" 400, \"wdth\" 75",
            4,
        ),
    ] {
        assert_atomic_limits(&ordinary(source, kind), expected, nodes);
    }
}

#[test]
fn shared_strings_use_escaped_utf8_byte_limits_and_retain_original_string_origins() {
    let source = "/*😀*/'é\\\"Face'";
    let expected = "\"é\\\"Face\"";
    let components = parse_component_values(source).unwrap();
    let Authored::Ordinary(value) =
        Authored::try_from_components(Kind::FontNamedInstance, components).unwrap()
    else {
        panic!("named instance")
    };
    let Value::FontNamedInstance(CssFontNamedInstance::String(name)) = &value else {
        panic!("string payload")
    };
    assert_eq!(name.as_str(), "é\"Face");
    let before = name.component().clone();
    assert_atomic_limits(&value, expected, 1);
    assert_eq!(name.component(), &before);
    let CssValueOrigin::Parsed(origin) = name.origin() else {
        panic!("parsed string origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find('\'').unwrap()
    );
    let language = Value::FontLanguageOverride(CssFontLanguageOverride::String(
        CssFontLanguageString::try_new("é\"Face").unwrap(),
    ));
    assert_atomic_limits(&language, expected, 1);
}

#[test]
fn metric_calculations_remain_authored_and_original_after_aggregate_failure() {
    for kind in [
        Kind::AscentOverride,
        Kind::DescentOverride,
        Kind::LineGapOverride,
    ] {
        for (source, expected) in [
            ("calc(50% + 25%)", "calc(75%)"),
            ("calc(-10%)", "calc(-10%)"),
        ] {
            let value = ordinary(source, kind);
            let percentage = match &value {
                Value::AscentOverride(CssFontMetricOverride::Percentage(value))
                | Value::DescentOverride(CssFontMetricOverride::Percentage(value))
                | Value::LineGapOverride(CssFontMetricOverride::Percentage(value)) => value,
                _ => panic!("metric calculation"),
            };
            let calculation = percentage.calculation().unwrap();
            assert_eq!(calculation.result_type(), CssCalculationType::Percentage);
            let before = calculation.components().clone();
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_eq!(
                value
                    .serialize_specified_with_limits(Limits::new(
                        usize::MAX,
                        usize::MAX,
                        expected.len() - 1
                    ))
                    .unwrap_err()
                    .kind(),
                ErrorKind::ByteLimit
            );
            assert_eq!(calculation.components(), &before);
            let CssValueOrigin::Parsed(origin) = percentage.origin() else {
                panic!("original calculation origin")
            };
            assert_eq!(origin.source().as_str(), source);
        }
    }
}

#[test]
fn aggregate_source_emission_preserves_url_phase_modifiers_and_original_argument_origins() {
    let source = "SRC(\"relative.face\" integrity(sha256)) format(WOFF2), url()";
    let value = ordinary(source, Kind::Src);
    let Value::Src(list) = &value else {
        panic!("source list")
    };
    let CssFontFaceSource::Url(first) = &list.sources()[0] else {
        panic!("authored src URL")
    };
    assert_eq!(first.url().function(), CssUrlFunction::Src);
    assert_eq!(first.url().as_str(), "relative.face");
    let CssUrlModifier::Function(modifier) = &first.url().modifiers()[0] else {
        panic!("integrity modifier")
    };
    let before = modifier.argument_components().clone();
    let expected = "src(\"relative.face\" integrity(sha256)) format(woff2), url(\"\")";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(
                usize::MAX,
                usize::MAX,
                expected.len() - 1
            ))
            .unwrap_err()
            .kind(),
        ErrorKind::ByteLimit
    );
    assert_eq!(modifier.argument_components(), &before);
    let CssValueOrigin::Parsed(origin) = modifier.argument_components().items()[0].origin() else {
        panic!("parsed modifier argument")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("sha256").unwrap()
    );
    let CssFontFaceSource::Url(last) = &list.sources()[1] else {
        panic!("empty authored URL")
    };
    assert_eq!(last.url().function(), CssUrlFunction::Url);
    assert_eq!(last.url().as_str(), "");
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

#[test]
fn eof_string_recovery_stays_visible_and_checked_descriptor_reentry_rejects_it() {
    for (kind, source) in [
        (Kind::FontFamily, "'Demo"),
        (Kind::FontNamedInstance, "'Regular"),
    ] {
        let report = parse_font_face_descriptor_value(source, kind);
        assert!(matches!(report.syntax(), Some(Authored::Ordinary(_))));
        let [diagnostic] = report.diagnostics() else {
            panic!("one retained EOF string diagnostic")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert!(report.into_validation_result().is_err());
        let components = parse_component_values(source).unwrap();
        assert!(Authored::try_from_components(kind, components.clone()).is_err());
        let Authored::Pending(pending) = parse_font_face_descriptor_value("env(face-value)", kind)
            .into_validation_result()
            .unwrap()
            .unwrap()
        else {
            panic!("pending descriptor")
        };
        assert!(pending.reparse_after_substitution(components).is_err());
    }
}

#[test]
fn stretch_alias_uses_width_grammar_while_removed_variant_has_no_descriptor_provider() {
    let source = "@font-face{font-stretch:75%;font-variant:normal;font-display:swap}";
    let report = parse_sheet(source);
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("retained face rule")
    };
    let width = face.descriptors().effective(Kind::FontWidth).unwrap();
    let Authored::Ordinary(value) = width.value() else {
        panic!("ordinary width under legacy descriptor name")
    };
    assert_eq!(value.serialize_specified().unwrap(), "75%");
    assert_eq!(face.descriptors().occurrences().len(), 2);
    assert!(face.descriptors().effective(Kind::FontDisplay).is_some());
    let [diagnostic] = report.diagnostics() else {
        panic!("removed descriptor recovery")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        source.find("font-variant").unwrap()
    );
}

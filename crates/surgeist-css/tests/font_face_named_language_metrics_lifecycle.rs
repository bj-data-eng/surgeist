#![forbid(unsafe_code)]

//! Fonts 4 WD 2026-09-07 §§4.7, 4.10, 4.11 descriptor grammars.
//! Env 1 §3 supplies whole-descriptor deferral and strict substituted reentry.
//! These tests use existing public parsing/lifecycle APIs; a descriptor kind is
//! obtained from an admitted occurrence, without referring to future enum variants.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssComponentValueErrorKind,
    CssComponentValueLimits, CssErrorCode, CssFontFaceDescriptor, CssFontFaceDescriptorKind,
    CssFontFaceDescriptors, CssFontFaceValueErrorKind, CssNormalizedItem, CssRecoveryAction,
    CssRule, CssRuleContextKindRef, CssSerializedOrigin, CssValueOrigin, normalize_report,
    parse_component_values, parse_font_face_descriptor_value, parse_sheet,
};

fn kind_from_sheet(name: &str, value: &str) -> CssFontFaceDescriptorKind {
    let source = format!("/*😀*/\r\n@font-face{{{name}:{value};font-display:swap}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{name}: {value}: {report:?}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("one retained font-face rule")
    };
    let records: Vec<_> = face.descriptors().occurrences().collect();
    assert_eq!(records.len(), 2, "{name}: {value}");
    assert!(matches!(records[0].value(), Authored::Ordinary(_)));
    let kind = records[0].value().kind();
    assert_eq!(kind.css_name(), name);
    assert_eq!(
        records[0].position().unwrap().byte_offset().value(),
        source.find(name).unwrap()
    );
    assert_eq!(records[1].value().kind().css_name(), "font-display");
    kind
}

fn ordinary_grammar(name: &str, valid: &[&str], invalid: &[&str]) {
    let kind = kind_from_sheet(name, valid[0]);
    for value in valid {
        assert_eq!(kind_from_sheet(name, value), kind);
        let raw = parse_font_face_descriptor_value(value, kind);
        assert!(raw.is_clean(), "{name}: {value}: {raw:?}");
        let Some(Authored::Ordinary(ordinary)) = raw.syntax() else {
            panic!("{name}: {value} must have an ordinary typed value")
        };
        assert_eq!(ordinary.kind(), kind);
        let checked =
            Authored::try_from_components(kind, parse_component_values(value).unwrap()).unwrap();
        assert!(matches!(checked, Authored::Ordinary(_)));
        assert_eq!(checked.kind(), kind);
        let record = CssFontFaceDescriptor::new(checked);
        assert_eq!(record.position(), None);
        let descriptors = CssFontFaceDescriptors::new(vec![record]);
        assert_eq!(descriptors.effective(kind).unwrap().value().kind(), kind);
    }
    for value in invalid {
        let raw = parse_font_face_descriptor_value(value, kind);
        assert!(raw.syntax().is_none(), "{name}: {value}: {raw:?}");
        assert!(
            raw.diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.action() == CssRecoveryAction::RejectInput })
        );
        assert!(
            Authored::try_from_components(kind, parse_component_values(value).unwrap()).is_err(),
            "{name}: {value}"
        );
    }
}

fn pending_recovery_and_provenance(name: &str, first: &str, last: &str, invalid: &str) {
    let source = format!(
        "/*😀*/\r\n@font-face{{font-family:Demo;{name}:{first};{name}:{last};\
         {name}:{invalid};{name}:env(selection, fallback);font-display:swap}}"
    );
    let report = parse_sheet(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid descriptor must recover locally: {report:?}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidDescriptorValue
    );
    let invalid_offset = source.find(&format!("{name}:{invalid}")).unwrap();
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        invalid_offset
    );
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("one retained font-face rule")
    };
    let records: Vec<_> = face.descriptors().occurrences().collect();
    assert_eq!(records.len(), 5);
    assert_eq!(records[0].value().kind().css_name(), "font-family");
    assert_eq!(records[4].value().kind().css_name(), "font-display");
    for (record, marker) in [
        (records[1], format!("{name}:{first}")),
        (records[2], format!("{name}:{last}")),
        (records[3], format!("{name}:env")),
    ] {
        assert_eq!(record.value().kind().css_name(), name);
        assert_eq!(
            record.position().unwrap().byte_offset().value(),
            source.find(&marker).unwrap()
        );
    }
    assert!(matches!(records[1].value(), Authored::Ordinary(_)));
    assert!(matches!(records[2].value(), Authored::Ordinary(_)));
    let Authored::Pending(pending) = records[3].value() else {
        panic!("env() defers the complete descriptor")
    };
    let kind = pending.kind();
    assert!(std::ptr::eq(
        face.descriptors().effective(kind).unwrap(),
        records[3]
    ));
    let CssValueOrigin::Parsed(origin) = pending.components().items()[0].origin() else {
        panic!("pending function retains its original parsed origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        source.find("env(selection").unwrap()
    );

    let ordinary = pending
        .reparse_after_substitution(parse_component_values(last).unwrap())
        .unwrap();
    assert_eq!(ordinary.kind(), kind);
    let invalid_components = parse_component_values(invalid).unwrap();
    let error = pending
        .reparse_after_substitution(invalid_components)
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
    let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
        panic!("strict failure retains replacement provenance")
    };
    assert_eq!(origin.source().as_str(), invalid);
    for residual in ["env(selection)", "var(--selection)"] {
        assert_eq!(
            pending
                .reparse_after_substitution(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssFontFaceValueErrorKind::ResidualSubstitution
        );
    }
    for text in [
        "env(selection)",
        "env(selection) arbitrary trailing grammar",
    ] {
        let raw = parse_font_face_descriptor_value(text, kind);
        assert!(raw.is_clean(), "{name}: {text}: {raw:?}");
        assert!(matches!(raw.syntax(), Some(Authored::Pending(_))));
        let components = parse_component_values(text).unwrap();
        let checked = Authored::try_from_components(kind, components.clone()).unwrap();
        let Authored::Pending(value) = checked else {
            panic!("checked env() defers the whole grammar")
        };
        assert_eq!(value.components(), &components);
    }
    for text in ["env(selection)!important", "env(selection);", "env()"] {
        let raw = parse_font_face_descriptor_value(text, kind);
        assert!(raw.syntax().is_none(), "{name}: {text}: {raw:?}");
        assert!(
            Authored::try_from_components(kind, parse_component_values(text).unwrap()).is_err()
        );
    }
    let components = parse_component_values("env(selection)").unwrap();
    let original = components.clone();
    let error = Authored::try_from_components_with_limits(
        kind,
        components.clone(),
        CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssFontFaceValueErrorKind::Component(CssComponentValueErrorKind::ComponentLimit)
    );
    assert_eq!(components, original);

    let normalized = normalize_report(&report).unwrap();
    assert_eq!(normalized.diagnostics(), report.diagnostics());
    let [CssNormalizedItem::Rule(rule)] = normalized.syntax().items() else {
        panic!("normalization retains the enclosing font-face rule")
    };
    let CssRuleContextKindRef::FontFace(normalized_face) = rule.kind() else {
        panic!("normalized font-face payload")
    };
    assert_eq!(normalized_face.descriptors(), face.descriptors());

    let reverse = parse_sheet(&format!(
        "@font-face{{{name}:env(selection);{name}:{last}}}"
    ));
    assert!(reverse.is_clean(), "{reverse:?}");
    let [CssRule::FontFace(reverse_face)] = reverse.syntax().rules() else {
        panic!("reverse occurrence rule")
    };
    assert!(matches!(
        reverse_face.descriptors().effective(kind).unwrap().value(),
        Authored::Ordinary(_)
    ));
}

#[test]
fn named_instance_admits_auto_and_strings_and_preserves_descriptor_lifecycle() {
    ordinary_grammar(
        "font-named-instance",
        &[
            "auto",
            "AuTo",
            "\"Grotesque\"",
            "\"\"",
            "\"é😀\"",
            "\"auto\"",
        ],
        &[
            "",
            "normal",
            "none",
            "Grotesque",
            "1",
            "auto auto",
            "inherit",
            "initial",
            "unset",
            "revert",
            "var(--instance)",
            "auto!important",
        ],
    );
    pending_recovery_and_provenance("font-named-instance", "auto", "\"Grotesque\"", "normal");
}

#[test]
fn language_override_admits_normal_and_strings_and_preserves_descriptor_lifecycle() {
    ordinary_grammar(
        "font-language-override",
        &[
            "normal",
            "NoRmAl",
            "\"TRK\"",
            "\"LongTag\"",
            "\"\"",
            "\"é😀\"",
        ],
        &[
            "",
            "auto",
            "none",
            "TRK",
            "1",
            "normal normal",
            "inherit",
            "initial",
            "unset",
            "revert",
            "var(--language)",
            "normal!important",
        ],
    );
    pending_recovery_and_provenance("font-language-override", "normal", "\"TRK\"", "auto");
}

fn metric_lifecycle(name: &str) {
    ordinary_grammar(
        name,
        &[
            "normal",
            "NoRmAl",
            "0%",
            "125.000000000000000000001%",
            "500%",
            "calc(50% + 25%)",
            "calc(-10%)",
        ],
        &[
            "",
            "auto",
            "-1%",
            "0",
            "1px",
            "50% 75%",
            "calc(1px)",
            "calc(50% + 1)",
            "inherit",
            "initial",
            "unset",
            "revert",
            "var(--metric)",
            "50%!important",
        ],
    );
    pending_recovery_and_provenance(name, "normal", "125%", "-1%");
}

#[test]
fn ascent_override_admits_nonnegative_percentage_calculations_and_preserves_lifecycle() {
    metric_lifecycle("ascent-override");
}

#[test]
fn descent_override_admits_nonnegative_percentage_calculations_and_preserves_lifecycle() {
    metric_lifecycle("descent-override");
}

#[test]
fn line_gap_override_admits_nonnegative_percentage_calculations_and_preserves_lifecycle() {
    metric_lifecycle("line-gap-override");
}

#[test]
fn existing_width_descriptor_retains_calculations_pending_values_and_local_recovery() {
    ordinary_grammar(
        "font-width",
        &["normal", "75%", "calc(50% + 25%)"],
        &["bad", "-1%"],
    );
    pending_recovery_and_provenance("font-width", "normal", "75%", "bad");
}

#[test]
fn unknown_descriptor_recovers_locally_and_does_not_discard_historical_siblings() {
    let source =
        "@font-face{font-family:Demo;unknown-face-field:normal;src:local(Demo);font-display:swap}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].action(),
        CssRecoveryAction::DropDescriptor
    );
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font-face siblings survive")
    };
    let names: Vec<_> = face
        .descriptors()
        .occurrences()
        .map(|record| record.value().kind().css_name())
        .collect();
    assert_eq!(names, ["font-family", "src", "font-display"]);
}

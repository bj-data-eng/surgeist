#![forbid(unsafe_code)]

//! Shared authored providers for the selected Font Loading adapter boundary.
//! Loading 3 (2023-04-06) requires whole font grammar and CSS-wide rejection.
//! Ordinary-only and clean-report loading admission are explicit integration
//! policies; shared CSS retains pending syntax and valid recovery separately.
//! Initial-value resolution, system font selection and loading stay downstream.

use surgeist_css::*;

fn query(source: &str) -> CssParseReport<Option<CssDeclaration>> {
    parse_property_value_text(
        source,
        CssPropertyNameRef::Known(CssKnownProperty::Font),
        CssImportance::Normal,
    )
}

fn font(declaration: &CssDeclaration) -> &CssFontValue {
    let CssKnownDeclaredValueRef::Property(CssKnownPropertyValueRef::Font(value)) =
        declaration.known().unwrap().declared_value()
    else {
        panic!("ordinary authored font")
    };
    value.font()
}

#[test]
fn raw_queries_preserve_relative_values_and_the_original_unwrapped_source() {
    for (source, expected_weight, expected_size) in [
        (
            "/*😀*/ bolder 120%/1.25 Demo",
            CssFontWeight::Bolder,
            "120%",
        ),
        ("lighter 2em Demo", CssFontWeight::Lighter, "2em"),
        ("bolder larger Demo", CssFontWeight::Bolder, "larger"),
    ] {
        let declaration = query(source).into_validation_result().unwrap().unwrap();
        assert!(declaration.parsed_name().is_none());
        assert!(declaration.position().is_none());
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
        let CssFontValue::Explicit(value) = font(&declaration) else {
            panic!("explicit font query")
        };
        assert_eq!(value.weight(), Some(&expected_weight));
        assert_eq!(value.size().serialize_specified().unwrap(), expected_size);
        assert_eq!(value.families().families()[0].as_str(), "Demo");
        assert_eq!(
            value.weight().unwrap().serialize_specified().unwrap(),
            match expected_weight {
                CssFontWeight::Bolder => "bolder",
                CssFontWeight::Lighter => "lighter",
                _ => unreachable!(),
            }
        );
    }
}

#[test]
fn system_queries_retain_the_selected_system_branch_without_contextual_resolution() {
    for (source, expected) in [
        ("caption", CssSystemFont::Caption),
        ("icon", CssSystemFont::Icon),
        ("menu", CssSystemFont::Menu),
        ("message-box", CssSystemFont::MessageBox),
        ("small-caption", CssSystemFont::SmallCaption),
        ("status-bar", CssSystemFont::StatusBar),
    ] {
        let declaration = query(source).into_validation_result().unwrap().unwrap();
        assert_eq!(font(&declaration), &CssFontValue::System(expected));
        assert_eq!(font(&declaration).serialize_specified().unwrap(), source);
    }
}

#[test]
fn globals_and_pending_queries_are_distinct_clean_authored_branches() {
    for (source, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let declaration = query(source).into_validation_result().unwrap().unwrap();
        assert_eq!(
            declaration.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(expected)
        );
    }
    for source in ["var(--query)", "env(query)", "attr(data-font)"] {
        let declaration = query(source).into_validation_result().unwrap().unwrap();
        assert!(matches!(
            declaration.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(_)
        ));
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
    }
}

#[test]
fn malformed_or_annotated_queries_reject_the_entire_original_value() {
    for source in [
        "",
        "16px",
        "bolder Demo",
        "16px Demo!important",
        "16px Demo;",
        "16px Demo}",
        "16px Demo{color:red}",
    ] {
        let report = query(source);
        assert!(report.syntax().is_none(), "{source}: {report:?}");
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.action() == CssRecoveryAction::RejectInput)
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn recovered_query_syntax_remains_available_but_clean_admission_rejects_it() {
    let complete = query("16px \"Demo\"");
    assert!(complete.is_clean());
    assert!(complete.syntax().is_some());
    for (source, expected_family) in [
        ("16px \"Demo", "Demo"),
        ("16px 'Demo", "Demo"),
        (r#"16px "Demo\""#, "Demo\""),
        ("16px \"Demo\\", "Demo"),
    ] {
        let report = query(source);
        let declaration = report.syntax().as_ref().expect("retained EOF-ended family");
        let CssFontValue::Explicit(value) = font(declaration) else {
            panic!("explicit recovered font")
        };
        assert_eq!(value.families().families()[0].as_str(), expected_family);
        assert!(!report.is_clean(), "{source}");
        let [diagnostic] = report.diagnostics() else {
            panic!("exactly one EOF-ended string recovery: {source}")
        };
        assert_eq!(
            diagnostic.action(),
            CssRecoveryAction::RetainWithImplicitClosure
        );
        assert_eq!(
            diagnostic.error().position().byte_offset().value(),
            source.len()
        );
        assert_eq!(
            declaration.parsed_value().unwrap().source().as_str(),
            source
        );
        assert!(report.into_validation_result().is_err());
    }
}

#[test]
fn source_descriptor_recovery_and_strict_components_have_separate_admission() {
    let source = "url(font.woff2), bad, local(Demo)";
    let report = parse_font_face_descriptor_value(source, CssFontFaceDescriptorKind::Src);
    assert!(report.syntax().is_some());
    let [diagnostic] = report.diagnostics() else {
        panic!("exactly one discarded source member")
    };
    assert_eq!(
        diagnostic.action(),
        CssRecoveryAction::DropFontSourceListItem
    );
    let offset = diagnostic.error().position().byte_offset().value();
    assert_eq!(offset, source.find("bad").unwrap());
    assert!(report.into_validation_result().is_err());
    assert!(
        CssAuthoredFontFaceDescriptorValue::try_from_components(
            CssFontFaceDescriptorKind::Src,
            parse_component_values(source).unwrap()
        )
        .is_err()
    );
    for valid in [
        "url(font.woff2)",
        "local(Demo)",
        "url(font.woff2), local(Demo)",
    ] {
        let value = parse_font_face_descriptor_value(valid, CssFontFaceDescriptorKind::Src)
            .into_validation_result()
            .unwrap()
            .unwrap();
        assert!(matches!(
            value,
            CssAuthoredFontFaceDescriptorValue::Ordinary(_)
        ));
    }
    for invalid in ["font.woff2", "url(font.woff2)!important", "local(Demo);"] {
        let report = parse_font_face_descriptor_value(invalid, CssFontFaceDescriptorKind::Src);
        assert!(report.syntax().is_none(), "{invalid}: {report:?}");
        assert!(report.into_validation_result().is_err());
    }
}

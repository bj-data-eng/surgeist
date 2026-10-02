#![forbid(unsafe_code)]

//! Color 5 WD 2026-09-08 §5.3 and Env 1 WD 2025-09-23 §3.
//! Construction checks the same authored grammar; callers supply replacements.
use surgeist_css::{
    CssColorProfileDescriptor, CssColorProfileDescriptorKind as Kind,
    CssColorProfileDescriptorValue as Value, CssColorProfileDescriptorValueRef as View,
    CssColorProfileName, CssColorProfileRenderingIntent as Intent, CssColorProfileRule,
    CssColorProfileRuleName as Name, CssColorProfileValueErrorKind as Failure, CssComponentValue,
    CssComponentValueErrorKind, CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssNormalizedItem, CssRecoveryAction, CssRule, CssRuleContextKindRef, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind as EmitFailure,
    CssSpecifiedValueSerializationLimits as EmitLimits, CssSupportStatus, CssUrlFunction,
    CssValueOrigin, CssValueTokenRef, feature_metadata, normalize_report,
    parse_color_profile_descriptor_value as fragment, parse_component_values, parse_sheet,
    validate_sheet,
};

fn value(kind: Kind, source: &str) -> Value {
    Value::try_new(kind, parse_component_values(source).unwrap()).unwrap()
}
fn pending(kind: Kind) -> Value {
    value(kind, "env(selection)")
}

#[test]
fn typed_name_default_and_absence_are_independent_of_authored_occurrences() {
    assert_eq!(Intent::default(), Intent::RelativeColorimetric);
    for name in [
        Name::Custom(CssColorProfileName::try_new("--").unwrap()),
        Name::DeviceCmyk,
    ] {
        let rule = CssColorProfileRule::new(name.clone(), vec![]);
        assert_eq!(rule.name(), &name);
        assert_eq!(rule.position(), None);
        assert!(rule.descriptors().is_empty());
        for kind in [Kind::Src, Kind::RenderingIntent, Kind::Components] {
            assert!(rule.effective(kind).is_none());
        }
    }
    let parsed = parse_sheet(
        r"@color-profile --Case {} @color-profile --case {} @color-profile d\65 vice-cmyk {}",
    );
    assert!(parsed.is_clean());
    let [
        CssRule::ColorProfile(first),
        CssRule::ColorProfile(second),
        CssRule::ColorProfile(device),
    ] = parsed.syntax().rules()
    else {
        panic!("typed profiles")
    };
    assert_ne!(first.name(), second.name());
    assert_eq!(device.name(), &Name::DeviceCmyk);
}

#[test]
fn parsed_fragment_and_construction_agree_on_every_ordinary_descriptor_grammar() {
    for (kind, source, expected) in [
        (Kind::Src, "url()", "url(\"\")"),
        (Kind::Src, "url(#x)", "url(\"#x\")"),
        (
            Kind::Src,
            "src(\"relative.icc\" cors m(a/**/b))",
            "src(\"relative.icc\" cors m(a/**/b))",
        ),
        (
            Kind::RenderingIntent,
            "RELATIVE-COLORIMETRIC",
            "relative-colorimetric",
        ),
        (
            Kind::RenderingIntent,
            "ABSOLUTE-COLORIMETRIC",
            "absolute-colorimetric",
        ),
        (Kind::RenderingIntent, "PERCEPTUAL", "perceptual"),
        (Kind::RenderingIntent, "SATURATION", "saturation"),
        (
            Kind::Components,
            r"initial, default, alpha, pi, A, a, A, 日本, a\+b",
            r"initial, default, alpha, pi, A, a, A, 日本, a\+b",
        ),
    ] {
        let report = fragment(source, kind);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let parsed = report.syntax().as_ref().unwrap();
        let constructed = value(kind, source);
        assert_eq!(parsed.kind(), kind);
        assert_eq!(parsed.view(), constructed.view());
        assert_eq!(parsed.to_specified_css().unwrap(), expected);
        assert_eq!(constructed.to_specified_css().unwrap(), expected);
    }
    let source_url = value(Kind::Src, "src(\"a\" cors)");
    let View::Src(url) = source_url.view() else {
        panic!("URL")
    };
    assert_eq!(url.function(), CssUrlFunction::Src);
}

#[test]
fn ordinary_components_exclude_only_none_and_keep_order_and_duplicates() {
    let checked = value(
        Kind::Components,
        "inherit, unset, revert, revert-layer, default, alpha, pi, A, a, A",
    );
    let View::Components(names) = checked.view() else {
        panic!("components")
    };
    assert_eq!(
        names.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
        [
            "inherit",
            "unset",
            "revert",
            "revert-layer",
            "default",
            "alpha",
            "pi",
            "A",
            "a",
            "A"
        ]
    );
    for invalid in [
        "", "none", "NONE", r"n\6f ne", "a b", "a,", "a,,b", "\"a\"", "1",
    ] {
        assert!(
            fragment(invalid, Kind::Components).syntax().is_none(),
            "{invalid}"
        );
        assert!(matches!(
            Value::try_new(Kind::Components, parse_component_values(invalid).unwrap())
                .unwrap_err()
                .kind(),
            Failure::Grammar(_)
        ));
    }
}

#[test]
fn effective_lookup_retains_last_pending_occurrence_without_selecting_profiles() {
    let report = parse_sheet(
        "@color-profile --x { rendering-intent: perceptual; rendering-intent: env(intent); src: url(a); src: url(b); } @color-profile --x {}",
    );
    assert!(report.is_clean());
    let [CssRule::ColorProfile(first), CssRule::ColorProfile(second)] = report.syntax().rules()
    else {
        panic!("profiles")
    };
    assert_eq!(first.descriptors().len(), 4);
    assert!(matches!(
        first.effective(Kind::RenderingIntent).unwrap().view(),
        View::Pending(_)
    ));
    let View::Src(last) = first.effective(Kind::Src).unwrap().view() else {
        panic!("last src")
    };
    assert_eq!(last.as_str(), "b");
    assert!(second.effective(Kind::RenderingIntent).is_none());
    let descriptor = CssColorProfileDescriptor::new(value(Kind::RenderingIntent, "saturation"));
    assert_eq!(descriptor.position(), None);
    assert_eq!(
        descriptor.to_specified_css().unwrap(),
        "rendering-intent: saturation"
    );
}

#[test]
fn environment_only_admission_defers_whole_grammar_without_a_var_veto() {
    for kind in [Kind::Src, Kind::RenderingIntent, Kind::Components] {
        for source in [
            "env(selection)",
            "env(selection, var(--x))",
            "env(selection) var(--x)",
            "env(selection) var()",
            r"e\6ev(selection 0 1, a, b)",
        ] {
            let parsed = fragment(source, kind);
            assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics());
            assert!(matches!(
                parsed.syntax().as_ref().unwrap().view(),
                View::Pending(_)
            ));
            assert!(matches!(value(kind, source).view(), View::Pending(_)));
        }
        for invalid in [
            "var(--x)",
            "env()",
            "env(selection -1)",
            "env(selection) env()",
        ] {
            assert!(
                fragment(invalid, kind).syntax().is_none(),
                "{kind:?}: {invalid}"
            );
            assert!(Value::try_new(kind, parse_component_values(invalid).unwrap()).is_err());
        }
    }
}

#[test]
fn url_strings_and_escaped_names_do_not_confuse_data_with_substitution() {
    for source in ["url(\"env(x) var(--x)\")", "src(\"var(--x)\")"] {
        assert!(matches!(value(Kind::Src, source).view(), View::Src(_)));
    }
    for source in [r"v\61 r(--x)", r"ENV(x)"] {
        let error = pending(Kind::Src)
            .reparse_after_substitution(parse_component_values(source).unwrap())
            .unwrap_err();
        assert_eq!(error.kind(), &Failure::ResidualSubstitution);
    }
}

#[test]
fn replacement_reentry_returns_ordinary_values_or_typed_errors_never_pending() {
    for (kind, replacement) in [
        (Kind::Src, "url(relative.icc)"),
        (Kind::RenderingIntent, "PERCEPTUAL"),
        (Kind::Components, "red, green"),
    ] {
        let checked = pending(kind)
            .reparse_after_substitution(parse_component_values(replacement).unwrap())
            .unwrap();
        assert!(!matches!(checked.view(), View::Pending(_)));
        assert_eq!(checked.kind(), kind);
        assert_eq!(
            checked.to_specified_css().unwrap(),
            value(kind, replacement).to_specified_css().unwrap()
        );
        assert_eq!(
            checked
                .reparse_after_substitution(parse_component_values(replacement).unwrap())
                .unwrap_err()
                .kind(),
            &Failure::NotPending
        );
    }
    for invalid in [
        "auto",
        "perceptual saturation",
        "var(--x)",
        "env(x)",
        "nested(var(--x))",
        "{env(x)}",
    ] {
        assert!(
            pending(Kind::RenderingIntent)
                .reparse_after_substitution(parse_component_values(invalid).unwrap())
                .is_err()
        );
    }
}

#[test]
fn strict_construction_and_reentry_reject_recovered_closures_with_original_origin() {
    for (kind, source) in [
        (Kind::Src, "url(\"a\""),
        (Kind::Components, "env(channels"),
        (Kind::RenderingIntent, "env(intent"),
    ] {
        let report = fragment(source, kind);
        assert!(!report.is_clean());
        let parsed = report
            .syntax()
            .as_ref()
            .expect("browser recovery retains valid grammar");
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.action()
                    == CssRecoveryAction::RetainWithImplicitClosure)
        );
        for error in [
            Value::try_new(kind, parsed.components().clone()).unwrap_err(),
            pending(kind)
                .reparse_after_substitution(parsed.components().clone())
                .unwrap_err(),
        ] {
            assert_eq!(error.kind(), &Failure::RecoveredComponent);
            let CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { at, opening }) =
                error.origin()
            else {
                panic!("original implicit closing origin")
            };
            assert_eq!(at.span().start().byte_offset().value(), source.len());
            assert_eq!(opening.source().as_str(), source);
        }
    }
}

#[test]
fn mixed_components_and_replacement_errors_keep_original_token_identity() {
    let mut items = parse_component_values("red,").unwrap().items().to_vec();
    items.push(CssComponentValue::try_ident("Green").unwrap());
    let components = CssComponentValues::try_new(items).unwrap();
    let checked = pending(Kind::Components)
        .reparse_after_substitution(components.clone())
        .unwrap();
    assert_eq!(checked.components(), &components);
    assert!(matches!(
        checked.components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert!(matches!(
        checked.components().items()[2].origin(),
        CssValueOrigin::Programmatic
    ));
    let mut invalid = parse_component_values("red,").unwrap().items().to_vec();
    invalid.push(CssComponentValue::try_token("1").unwrap());
    let error = Value::try_new(
        Kind::Components,
        CssComponentValues::try_new(invalid).unwrap(),
    )
    .unwrap_err();
    assert!(matches!(error.kind(), Failure::Grammar(_)));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn separately_constructed_adjacent_tokens_do_not_merge_into_an_identifier() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("red").unwrap(),
        CssComponentValue::try_ident("green").unwrap(),
    ])
    .unwrap();
    assert!(Value::try_new(Kind::Components, components).is_err());
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("red").unwrap(),
        CssComponentValue::try_token(",").unwrap(),
        CssComponentValue::try_ident("green").unwrap(),
    ])
    .unwrap();
    assert_eq!(
        Value::try_new(Kind::Components, values)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        "red, green"
    );
}

#[test]
fn root_annotations_fail_before_environment_deferral() {
    for kind in [Kind::Src, Kind::RenderingIntent, Kind::Components] {
        for source in ["env(x) !important", "env(x) !other", "env(x) ;", "env(x) }"] {
            assert!(fragment(source, kind).syntax().is_none(), "{source}");
            if let Ok(components) = parse_component_values(source) {
                assert!(Value::try_new(kind, components).is_err());
            }
        }
        assert!(matches!(
            value(kind, "env(x, {nested})").view(),
            View::Pending(_)
        ));
    }
}

#[test]
fn matched_root_blocks_remain_data_under_environment_whole_grammar_deferral() {
    // Syntax 3 CRD 2021-12-24 #declaration-value admits matched blocks;
    // Env 1 WD 2025-09-23 §3 assumes the entire descriptor grammar valid.
    for source in ["env(intent) { x }", "{ x } env(intent)"] {
        let parsed = fragment(source, Kind::RenderingIntent);
        assert!(parsed.is_clean(), "{source}: {:?}", parsed.diagnostics());
        assert!(matches!(
            parsed.syntax().as_ref().unwrap().view(),
            View::Pending(_)
        ));
        assert!(matches!(
            value(Kind::RenderingIntent, source).view(),
            View::Pending(_)
        ));
        let sheet = parse_sheet(&format!(
            "@color-profile --x {{ rendering-intent: {source}; }} .after {{ color: red; }}"
        ));
        assert!(sheet.is_clean(), "{:?}", sheet.diagnostics());
        assert!(matches!(
            sheet.syntax().rules(),
            [CssRule::ColorProfile(_), CssRule::Style(_)]
        ));
    }
    for ordinary in ["{}", "perceptual {}"] {
        assert!(fragment(ordinary, Kind::RenderingIntent).syntax().is_none());
    }
}

#[test]
fn normalization_borrows_typed_profile_payload_and_keeps_descriptor_origins() {
    let source = "@media all { @color-profile --x { src: url(a); rendering-intent: perceptual; } }";
    let report = parse_sheet(source);
    let normalized = normalize_report(&report).unwrap();
    let [
        CssNormalizedItem::Rule(parent),
        CssNormalizedItem::Rule(terminal),
    ] = normalized.syntax().items()
    else {
        panic!("two intact contexts")
    };
    let CssRuleContextKindRef::ColorProfile(profile) = terminal.kind() else {
        panic!("typed profile")
    };
    assert!(terminal.parent().unwrap().same_context(parent));
    assert_eq!(
        profile.position().unwrap().byte_offset().value(),
        source.find("@color-profile").unwrap()
    );
    assert_eq!(
        profile.descriptors()[0]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.find("src:").unwrap()
    );
    assert_eq!(
        profile.descriptors()[1]
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        source.find("rendering-intent:").unwrap()
    );
    assert!(
        profile.descriptors()[0]
            .value()
            .components()
            .items()
            .iter()
            .any(|component| matches!(
                component.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Url("a"))
            ))
    );
}

#[test]
fn descriptor_and_component_limits_are_checked_at_construction_and_emission() {
    let components = parse_component_values("red, green").unwrap();
    let error = Value::try_new_with_limits(
        Kind::Components,
        components.clone(),
        CssComponentValueLimits::try_new(10, 0, 100).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &Failure::Component(CssComponentValueErrorKind::ComponentLimit)
    );
    let error = pending(Kind::Components)
        .reparse_after_substitution_with_limits(
            components,
            CssComponentValueLimits::try_new(10, 100, 1).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &Failure::Component(CssComponentValueErrorKind::ByteLimit)
    );
    let checked = value(Kind::Components, "red, green");
    for (limits, expected) in [
        (EmitLimits::new(1, 100, 100), EmitFailure::InputNodeLimit),
        (
            EmitLimits::new(100, 1, 100),
            EmitFailure::ProjectionNodeLimit,
        ),
        (EmitLimits::new(100, 100, 9), EmitFailure::ByteLimit),
    ] {
        assert_eq!(
            checked
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
    assert_eq!(
        checked
            .to_specified_css_with_limits(EmitLimits::new(2, 2, 10))
            .unwrap(),
        "red, green"
    );
}

#[test]
fn descriptor_and_keyframe_bodies_reject_profile_children_without_losing_following_rule() {
    for source in [
        "@font-face { @color-profile --x {} font-family: Demo; src: url(a); } .after { color: red; }",
        "@keyframes k { from { @color-profile --x {} color: red; } } .after { color: red; }",
    ] {
        let report = parse_sheet(source);
        assert!(!report.is_clean());
        assert!(validate_sheet(source).is_err());
        assert!(matches!(
            report.syntax().rules().last().unwrap(),
            CssRule::Style(_)
        ));
    }
}

#[test]
fn support_metadata_registers_only_the_authored_rule_and_three_descriptors() {
    for id in [
        "interop.rule.color-profile",
        "interop.descriptor.color-profile.src",
        "interop.descriptor.color-profile.rendering-intent",
        "interop.descriptor.color-profile.components",
    ] {
        assert_eq!(
            feature_metadata(id).unwrap().status(),
            CssSupportStatus::Complete
        );
    }
}

#[test]
fn programmatic_profile_composes_checked_descriptors_and_generic_writer() {
    let rule = CssColorProfileRule::new(
        Name::Custom(CssColorProfileName::try_new("--Programmatic").unwrap()),
        vec![
            CssColorProfileDescriptor::new(value(Kind::Src, "src(\"a.icc\" cors)")),
            CssColorProfileDescriptor::new(value(Kind::RenderingIntent, "PERCEPTUAL")),
            CssColorProfileDescriptor::new(value(Kind::Components, "R, G, B")),
        ],
    );
    let expected = "@color-profile --Programmatic { src: src(\"a.icc\" cors); rendering-intent: perceptual; components: R, G, B; }";
    assert_eq!(rule.to_specified_css().unwrap(), expected);
    assert_eq!(
        CssRule::ColorProfile(rule).to_specified_css().unwrap(),
        expected
    );
    assert_eq!(
        validate_sheet(expected)
            .unwrap()
            .to_specified_css()
            .unwrap(),
        expected
    );
}

#[test]
fn descriptor_recovery_reports_actual_token_offsets_and_preserves_valid_neighbors() {
    let source = "@color-profile --x { src: url(a); components: none; rendering-intent: saturation; } .after { color: red; }";
    let report = parse_sheet(source);
    assert!(!report.is_clean());
    assert_eq!(report.diagnostics().len(), 1);
    let diagnostic = &report.diagnostics()[0];
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDescriptor);
    assert_eq!(
        diagnostic.error().position().byte_offset().value(),
        source.find("none").unwrap()
    );
    let [CssRule::ColorProfile(profile), CssRule::Style(_)] = report.syntax().rules() else {
        panic!("retained neighbors")
    };
    assert_eq!(profile.descriptors().len(), 2);
    assert_eq!(
        profile.effective(Kind::RenderingIntent).unwrap().view(),
        View::RenderingIntent(Intent::Saturation)
    );
    assert!(profile.effective(Kind::Components).is_none());
    assert_eq!(
        normalize_report(&report).unwrap().diagnostics(),
        report.diagnostics()
    );
}

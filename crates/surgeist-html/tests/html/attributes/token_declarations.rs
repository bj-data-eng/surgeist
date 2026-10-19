use crate::html::support::{
    known_attribute as attr, known_element as element, ordered_attributes as attrs,
    value_interpreter as interpreter,
};
use surgeist_html::*;

fn token(value: impl Into<DocumentString>) -> HtmlSpaceToken {
    HtmlSpaceToken::new(value).unwrap()
}
fn nn(value: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&value.into()).unwrap()
}
fn keys(value: impl Into<DocumentString>) -> HtmlAccessKeyReport {
    interpreter()
        .accesskey(
            &element(KnownHtmlElement::Button),
            Some(&attr(KnownHtmlAttribute::Accesskey, value)),
        )
        .unwrap()
}
fn blocking(value: &str) -> HtmlBlockingReport {
    interpreter()
        .blocking(
            &element(KnownHtmlElement::Style),
            Some(&attr(KnownHtmlAttribute::Blocking, value)),
        )
        .unwrap()
}
fn sizes(value: impl Into<DocumentString>) -> HtmlIconSizesReport {
    interpreter()
        .icon_sizes(
            &element(KnownHtmlElement::Link),
            Some(&attr(KnownHtmlAttribute::Sizes, value)),
        )
        .unwrap()
}
fn candidate_units(report: &HtmlAccessKeyReport) -> Vec<Vec<u16>> {
    report
        .candidate_tokens()
        .map(|t| t.value().code_units().to_vec())
        .collect()
}

#[test]
fn zero_token_values_conform_but_missing_is_not_specified_empty() {
    let i = HtmlValueInterpreter::new(0);
    let missing = i
        .accesskey(
            &element(KnownHtmlElement::Button),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(
        missing.input().author(),
        &HtmlTokenDeclarationAuthorState::Missing
    );
    assert_eq!(
        missing.input().common().unwrap().recovery(),
        &HtmlValueRecoveryState::Missing
    );
    assert_eq!(
        missing.input().observation(),
        HtmlAttributeObservation::Missing
    );
    for value in ["", " \t\r\n\u{c}"] {
        assert_eq!(
            keys(value).input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert!(candidate_units(&keys(value)).is_empty());
        assert_eq!(
            blocking(value).input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert_eq!(
            blocking(value).explicit_tokens(),
            Some(HtmlBlockingTokens::Empty)
        );
        assert_eq!(
            sizes(value).input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert_eq!(sizes(value).keywords(), Some([].as_slice()));
    }
    let missing = i
        .blocking(
            &element(KnownHtmlElement::Style),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(missing.explicit_tokens(), Some(HtmlBlockingTokens::Empty));
    assert_eq!(
        missing.input().author(),
        &HtmlTokenDeclarationAuthorState::Missing
    );
    let missing = i
        .icon_sizes(
            &element(KnownHtmlElement::Link),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(missing.keywords(), None);
    assert_eq!(
        missing.resource_conformance(),
        HtmlAttributeContextAuthorState::NotRequired
    );
}

#[test]
fn accesskey_author_uniqueness_is_exact_and_processing_keeps_repeated_candidates() {
    assert_eq!(
        keys("A a").input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
    let repeated = keys("ab a a");
    assert_eq!(
        repeated.input().author(),
        &HtmlTokenDeclarationAuthorState::Nonconforming(
            HtmlTokenDeclarationError::AccessKeyLength { token_index: 0 }
        )
    );
    assert_eq!(candidate_units(&repeated), vec![vec![97], vec![97]]);
    let duplicate = repeated.input().duplicate().unwrap();
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (1, 2)
    );
    assert_eq!(duplicate.comparison(), HtmlTokenComparison::CaseSensitive);
    assert_eq!(duplicate.token(), &DocumentString::from("a"));
    assert!(matches!(
        keys("a a").input().author(),
        HtmlTokenDeclarationAuthorState::Nonconforming(HtmlTokenDeclarationError::Duplicate(_))
    ));
    assert_eq!(
        keys("a").input().common().unwrap().author(),
        &HtmlValueAuthorState::NotDefined
    );
}

#[test]
fn accesskey_counts_code_points_without_replacing_isolated_surrogates() {
    for units in [
        vec![0xd83d, 0xde42],
        vec![0xd800],
        vec![0xdc00],
        vec![0],
        vec![0xa0],
    ] {
        let report = keys(DocumentString::from_utf16(units.clone()));
        assert_eq!(
            report.input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert_eq!(candidate_units(&report), vec![units.clone()]);
        assert_eq!(report.input().raw().unwrap().code_units(), units);
    }
    for units in [
        vec![0xd800, 0xd800],
        vec![0xd83d, 0xde42, 0x301],
        vec![97, 0x301],
    ] {
        let report = keys(DocumentString::from_utf16(units));
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(
                HtmlTokenDeclarationError::AccessKeyLength { token_index: 0 }
            )
        ));
        assert!(candidate_units(&report).is_empty());
    }
}

#[test]
fn native_accesskeys_check_cardinality_uniqueness_and_emit_exact_order() {
    let value = HtmlAccessKeys::new([
        token("A"),
        token(DocumentString::from_utf16(vec![0xd800])),
        token("a"),
    ])
    .unwrap();
    assert_eq!(value.tokens().len(), 3);
    let emitted = value.into_attribute();
    assert_eq!(
        emitted.name(),
        &KnownHtmlAttribute::Accesskey.qualified_name()
    );
    assert_eq!(emitted.value().code_units(), [65, 32, 0xd800, 32, 97]);
    assert_eq!(
        HtmlAccessKeys::new([])
            .unwrap()
            .into_attribute()
            .value()
            .code_units(),
        []
    );
    assert_eq!(
        HtmlAccessKeys::new([token("ab")]),
        Err(HtmlTokenDeclarationError::AccessKeyLength { token_index: 0 })
    );
    assert!(matches!(
        HtmlAccessKeys::new([token("a"), token("a")]),
        Err(HtmlTokenDeclarationError::Duplicate(_))
    ));
}

#[test]
fn accesskey_applies_to_unknown_html_names_and_preserves_prefixes() {
    let e = QualifiedName::new(Namespace::html(), Some("h".into()), "custom-control");
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "accesskey"),
        "a",
    );
    let report = interpreter().accesskey(&e, Some(&raw)).unwrap();
    assert_eq!(report.input().element(), &e);
    assert_eq!(report.input().raw_attribute(), Some(&raw));
    assert_eq!(
        report.input().applicability(),
        HtmlAttributeApplicability::Applicable
    );
    assert_eq!(
        report.input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
    assert_eq!(report.input().inventory_rows(), [4]);
    assert!(
        report
            .input()
            .source_clause()
            .ends_with("#the-accesskey-attribute")
    );
    assert!(
        report
            .processing_source_clause()
            .ends_with("#keyboard-shortcuts-processing-model")
    );
}

#[test]
fn blocking_author_exactness_does_not_adopt_processing_case_folding_or_deduplication() {
    assert_eq!(
        blocking("render").input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
    for value in ["RENDER", "render unknown", "unknown RENDER", "render\u{a0}"] {
        let report = blocking(value);
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(
                HtmlTokenDeclarationError::InvalidBlockingToken { .. }
            )
        ));
        assert_eq!(
            report.explicit_tokens(),
            Some(if value == "render\u{a0}" {
                HtmlBlockingTokens::Empty
            } else {
                HtmlBlockingTokens::Render
            })
        );
    }
    let repeated = blocking("render render");
    assert!(matches!(
        repeated.input().author(),
        HtmlTokenDeclarationAuthorState::Nonconforming(HtmlTokenDeclarationError::Duplicate(_))
    ));
    assert_eq!(repeated.explicit_tokens(), Some(HtmlBlockingTokens::Render));
    assert_eq!(
        blocking("unknown").explicit_tokens(),
        Some(HtmlBlockingTokens::Empty)
    );
    assert_eq!(
        blocking("RENDER render unknown").explicit_tokens(),
        Some(HtmlBlockingTokens::Render)
    );
    assert_eq!(
        blocking("\t\r\n\u{c}render ").input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
}

#[test]
fn blocking_native_emission_is_the_explicit_set_without_context_certification() {
    for (value, literal) in [
        (HtmlBlockingTokens::Empty, ""),
        (HtmlBlockingTokens::Render, "render"),
    ] {
        let raw = value.into_attribute();
        assert_eq!(raw.name(), &KnownHtmlAttribute::Blocking.qualified_name());
        assert_eq!(raw.value(), &DocumentString::from(literal));
        let report = interpreter()
            .blocking(&element(KnownHtmlElement::Link), Some(&raw))
            .unwrap();
        assert_eq!(
            report.input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert_eq!(
            report.input().context_author(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert_eq!(report.explicit_tokens(), Some(value));
    }
}

#[test]
fn complete_script_context_matrix_allows_only_two_external_kinds() {
    use HtmlScriptAttributeContext::*;
    let raw = attr(KnownHtmlAttribute::Blocking, "RENDER");
    for (context, allowed) in [
        (ExternalClassic, true),
        (InlineClassic, false),
        (ExternalModule, true),
        (InlineModule, false),
        (ImportMap, false),
        (SpeculationRules, false),
        (DataBlock, false),
    ] {
        let report = interpreter()
            .blocking_for_script(&element(KnownHtmlElement::Script), Some(&raw), context)
            .unwrap();
        assert_eq!(report.input().script_context(), Some(context));
        assert_eq!(
            report.input().context_author(),
            if allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::ScriptContextForbidsAttribute,
                )
            }
        );
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(
                HtmlTokenDeclarationError::InvalidBlockingToken { token_index: 0 }
            )
        ));
        assert_eq!(report.explicit_tokens(), Some(HtmlBlockingTokens::Render));
        assert!(
            report
                .input()
                .source_clause()
                .ends_with("#attr-script-blocking")
        );
        assert!(
            report
                .input()
                .context_source_clause()
                .unwrap()
                .ends_with("#the-script-element")
        );
        assert_eq!(
            interpreter()
                .blocking_for_script(
                    &element(KnownHtmlElement::Script),
                    None::<&surgeist_html::Attribute>,
                    context
                )
                .unwrap()
                .input()
                .context_author(),
            HtmlAttributeContextAuthorState::NotRequired
        );
    }
}

#[test]
fn link_blocking_checks_actual_rel_membership_independently_of_token_errors() {
    for (rel, allowed) in [
        ("stylesheet", true),
        ("EXPECT", true),
        ("expect stylesheet", true),
        ("", false),
        ("icon", false),
        ("stylesheetish", false),
        ("stylesheet\u{a0}", false),
    ] {
        let raw = attr(KnownHtmlAttribute::Blocking, "bad");
        let rel = attr(KnownHtmlAttribute::Rel, rel);
        let report = interpreter()
            .blocking_for_link(
                &element(KnownHtmlElement::Link),
                &attrs([raw.clone(), rel.clone()]),
            )
            .unwrap();
        assert_eq!(report.input().raw_attribute(), Some(&raw));
        assert_eq!(report.input().rel_attribute(), Some(&rel));
        assert!(report.input().rel_observed());
        assert_eq!(
            report.input().context_author(),
            if allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::BlockingRequiresStylesheetOrExpectRel,
                )
            }
        );
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(
                HtmlTokenDeclarationError::InvalidBlockingToken { .. }
            )
        ));
    }
    let report = interpreter()
        .blocking_for_link(
            &element(KnownHtmlElement::Link),
            &attrs([attr(KnownHtmlAttribute::Blocking, "render")]),
        )
        .unwrap();
    assert_eq!(report.input().rel_attribute(), None);
    assert_eq!(
        report.input().context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::BlockingRequiresStylesheetOrExpectRel
        )
    );
}

#[test]
fn icon_sizes_case_insensitive_author_uniqueness_does_not_remove_processing_repetitions() {
    for value in ["any ANY", "16x16 16X16"] {
        let report = sizes(value);
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(HtmlTokenDeclarationError::Duplicate(_))
        ));
        let duplicate = report.input().duplicate().unwrap();
        assert_eq!(
            duplicate.comparison(),
            HtmlTokenComparison::AsciiCaseInsensitive
        );
        assert_eq!(
            (duplicate.first_index(), duplicate.duplicate_index()),
            (0, 1)
        );
        let keywords = report.keywords().unwrap();
        assert_eq!(keywords.len(), 2);
        assert_eq!(keywords[0].result(), keywords[1].result());
        assert_eq!(
            (keywords[0].token_index(), keywords[1].token_index()),
            (0, 1)
        );
    }
    assert_eq!(
        sizes("ANY").keywords().unwrap()[0].result(),
        Ok(&HtmlIconSize::Scalable)
    );
    assert_eq!(
        sizes("any 16X32").input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
}

#[test]
fn invalid_icon_keywords_are_rejected_individually_and_valid_neighbors_survive() {
    for value in [
        "0x1", "01x1", "1x0", "1xx2", "+1x2", "1x2tail", "1.0x2", "x1", "1x", "-0x2", "١x2",
    ] {
        let report = sizes(format!("any {value} 2x3"));
        assert!(matches!(
            report.input().author(),
            HtmlTokenDeclarationAuthorState::Nonconforming(
                HtmlTokenDeclarationError::InvalidIconSize { token_index: 1, .. }
            )
        ));
        let keywords = report.keywords().unwrap();
        assert_eq!(keywords[0].result(), Ok(&HtmlIconSize::Scalable));
        assert!(keywords[1].result().is_err());
        match keywords[2].result().unwrap() {
            HtmlIconSize::Bitmap(size) => {
                assert_eq!(size.width().decimal_digits(), "2");
                assert_eq!(size.height().decimal_digits(), "3");
            }
            _ => panic!("expected bitmap"),
        }
    }
}

#[test]
fn icon_dimensions_beyond_u128_are_exact_in_reports_and_checked_native_values() {
    let huge = "340282366920938463463374607431768211456";
    let report = sizes(format!("{huge}X2"));
    assert_eq!(
        report.input().author(),
        &HtmlTokenDeclarationAuthorState::Conforming
    );
    match report.keywords().unwrap()[0].result().unwrap() {
        HtmlIconSize::Bitmap(size) => {
            assert_eq!(size.width().decimal_digits(), huge);
            assert_eq!(size.height().decimal_digits(), "2");
        }
        _ => panic!("expected bitmap"),
    }
    let bitmap = HtmlBitmapIconSize::new(nn(huge), nn("2")).unwrap();
    let native =
        HtmlIconSizes::new([HtmlIconSize::Scalable, HtmlIconSize::Bitmap(bitmap)]).unwrap();
    assert_eq!(
        native.into_attribute().value(),
        &DocumentString::from(format!("any {huge}x2"))
    );
    assert_eq!(
        HtmlBitmapIconSize::new(nn("0"), nn("1")),
        Err(HtmlIconSizeError::ZeroDimension)
    );
    assert_eq!(
        HtmlBitmapIconSize::new(nn("1"), nn("0")),
        Err(HtmlIconSizeError::ZeroDimension)
    );
}

#[test]
fn native_icon_sets_reject_duplicates_ignore_order_and_emit_canonical_tokens() {
    let bitmap = HtmlIconSize::Bitmap(HtmlBitmapIconSize::new(nn("0016"), nn("0032")).unwrap());
    let first = HtmlIconSizes::new([HtmlIconSize::Scalable, bitmap.clone()]).unwrap();
    let second = HtmlIconSizes::new([bitmap.clone(), HtmlIconSize::Scalable]).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.tokens()[1].value(), &DocumentString::from("16x32"));
    let raw = first.into_attribute();
    assert_eq!(raw.name(), &KnownHtmlAttribute::Sizes.qualified_name());
    assert_eq!(raw.value(), &DocumentString::from("any 16x32"));
    assert!(HtmlIconSizes::new([bitmap.clone(), bitmap]).is_err());
    assert!(HtmlIconSizes::new([HtmlIconSize::Scalable, HtmlIconSize::Scalable]).is_err());
    assert!(
        HtmlIconSizes::new([])
            .unwrap()
            .into_attribute()
            .value()
            .is_empty()
    );
}

#[test]
fn icon_rel_author_presence_and_resource_availability_are_independent() {
    for (rel, allowed) in [
        ("ICON", true),
        ("apple-touch-icon", true),
        ("other icon", true),
        ("stylesheet", false),
        ("iconish", false),
        ("", false),
    ] {
        let report = interpreter()
            .icon_sizes_for_link(
                &element(KnownHtmlElement::Link),
                &attrs([
                    attr(KnownHtmlAttribute::Sizes, "16x32"),
                    attr(KnownHtmlAttribute::Rel, rel),
                ]),
            )
            .unwrap();
        assert_eq!(
            report.input().author(),
            &HtmlTokenDeclarationAuthorState::Conforming
        );
        assert_eq!(
            report.input().context_author(),
            if allowed {
                HtmlAttributeContextAuthorState::Conforming
            } else {
                HtmlAttributeContextAuthorState::Nonconforming(
                    HtmlAttributeContextConformanceError::IconSizesRequireIconRel,
                )
            }
        );
        assert_eq!(
            report.resource_conformance(),
            HtmlAttributeContextAuthorState::Unchecked
        );
        assert!(report.input().source_clause().ends_with("#attr-link-sizes"));
        assert!(report.processing_source_clause().ends_with("#rel-icon"));
        assert!(
            report
                .input()
                .context_source_clause()
                .unwrap()
                .ends_with("#attr-link-rel")
        );
    }
    let report = interpreter()
        .icon_sizes_for_link(
            &element(KnownHtmlElement::Link),
            &attrs([attr(KnownHtmlAttribute::Sizes, "any")]),
        )
        .unwrap();
    assert_eq!(
        report.input().context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::IconSizesRequireIconRel
        )
    );
}

#[test]
fn namespaced_rel_is_actual_missing_and_missing_declarations_need_no_presence_check() {
    let rel = Attribute::new(
        QualifiedName::new(Namespace::from_uri("urn:other"), None, "rel"),
        "icon stylesheet",
    );
    let report = interpreter()
        .icon_sizes_for_link(
            &element(KnownHtmlElement::Link),
            &attrs([attr(KnownHtmlAttribute::Sizes, "any"), rel]),
        )
        .unwrap();
    assert_eq!(report.input().rel_attribute(), None);
    assert_eq!(
        report.input().context_author(),
        HtmlAttributeContextAuthorState::Nonconforming(
            HtmlAttributeContextConformanceError::IconSizesRequireIconRel
        )
    );
    let absent = interpreter()
        .blocking_for_link(
            &element(KnownHtmlElement::Link),
            &attrs([attr(KnownHtmlAttribute::Rel, "icon")]),
        )
        .unwrap();
    assert_eq!(
        absent.input().context_author(),
        HtmlAttributeContextAuthorState::NotRequired
    );
    assert_eq!(
        absent.input().author(),
        &HtmlTokenDeclarationAuthorState::Missing
    );
}

#[test]
fn wrong_companion_context_has_no_token_or_descriptor_projection() {
    let raw = attr(KnownHtmlAttribute::Blocking, "bad");
    let report = interpreter()
        .blocking_for_script(
            &element(KnownHtmlElement::Style),
            Some(&raw),
            HtmlScriptAttributeContext::ExternalClassic,
        )
        .unwrap();
    assert_eq!(
        report.input().applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(report.input().common(), None);
    assert_eq!(report.explicit_tokens(), None);
    assert_eq!(report.input().raw_attribute(), Some(&raw));
    let report = interpreter()
        .blocking_for_link(&element(KnownHtmlElement::Script), &attrs([raw]))
        .unwrap();
    assert_eq!(
        report.input().applicability(),
        HtmlAttributeApplicability::DifferentContext
    );
    assert_eq!(report.explicit_tokens(), None);
    assert_eq!(report.input().common(), None);
}

#[test]
fn exact_expanded_names_control_applicability_without_changing_raw_units() {
    let raw = attr(
        KnownHtmlAttribute::Sizes,
        DocumentString::from_utf16(vec![0xd800, 0, 0x61]),
    );
    for e in [
        element(KnownHtmlElement::Img),
        QualifiedName::new(Namespace::html(), None, "LINK"),
        QualifiedName::new(Namespace::svg(), None, "link"),
    ] {
        let report = interpreter().icon_sizes(&e, Some(&raw)).unwrap();
        assert_ne!(
            report.input().applicability(),
            HtmlAttributeApplicability::Applicable
        );
        assert_eq!(report.input().raw_attribute(), Some(&raw));
        assert_eq!(report.input().common(), None);
        assert_eq!(report.keywords(), None);
        assert_eq!(
            report.input().author(),
            &HtmlTokenDeclarationAuthorState::Unapplied
        );
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "ACCESSKEY"),
        QualifiedName::new(Namespace::from_uri("urn:x"), None, "accesskey"),
    ] {
        let raw = Attribute::new(name, "a");
        let report = interpreter()
            .accesskey(&element(KnownHtmlElement::Button), Some(&raw))
            .unwrap();
        assert_eq!(
            report.input().applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.input().raw_attribute(), Some(&raw));
    }
}

#[test]
fn original_utf16_caps_precede_wrong_roles_and_token_filtering() {
    let raw = attr(
        KnownHtmlAttribute::Accesskey,
        DocumentString::from_utf16(vec![0xd83d, 0xde42]),
    );
    assert!(
        HtmlValueInterpreter::new(2)
            .accesskey(&element(KnownHtmlElement::Button), Some(&raw))
            .is_ok()
    );
    let error = HtmlValueInterpreter::new(1)
        .accesskey(
            &QualifiedName::new(Namespace::svg(), None, "button"),
            Some(&raw),
        )
        .unwrap_err();
    assert_eq!((error.capacity(), error.required()), (1, 2));
    let raw = attr(KnownHtmlAttribute::Blocking, "unknown");
    assert_eq!(
        HtmlValueInterpreter::new(6)
            .blocking(&element(KnownHtmlElement::Img), Some(&raw))
            .unwrap_err()
            .required(),
        7
    );
    let raw = attr(KnownHtmlAttribute::Sizes, "0x1");
    assert_eq!(
        HtmlValueInterpreter::new(2)
            .icon_sizes(&element(KnownHtmlElement::Img), Some(&raw))
            .unwrap_err()
            .required(),
        3
    );
}

#[test]
fn companion_caps_admit_both_selected_original_slots_even_for_wrong_context() {
    for e in [
        element(KnownHtmlElement::Link),
        element(KnownHtmlElement::Script),
    ] {
        let attributes = attrs([
            attr(KnownHtmlAttribute::Blocking, ""),
            attr(KnownHtmlAttribute::Rel, "expect "),
        ]);
        let error = HtmlValueInterpreter::new(6)
            .blocking_for_link(&e, &attributes)
            .unwrap_err();
        assert_eq!((error.capacity(), error.required()), (6, 7));
        let attributes = attrs([
            attr(KnownHtmlAttribute::Sizes, ""),
            attr(KnownHtmlAttribute::Rel, "icon "),
        ]);
        let error = HtmlValueInterpreter::new(4)
            .icon_sizes_for_link(&e, &attributes)
            .unwrap_err();
        assert_eq!((error.capacity(), error.required()), (4, 5));
    }
    let attributes = attrs([
        attr(KnownHtmlAttribute::Sizes, "16x32"),
        attr(KnownHtmlAttribute::Rel, "icon"),
    ]);
    assert!(
        HtmlValueInterpreter::new(5)
            .icon_sizes_for_link(&element(KnownHtmlElement::Link), &attributes)
            .is_ok()
    );
}

#[test]
fn icon_non_ascii_units_remain_exact_and_do_not_become_ascii_dimensions() {
    for units in [
        vec![49, 120, 0xd800],
        vec![49, 120, 0],
        vec![49, 120, 0xff12],
    ] {
        let report = sizes(DocumentString::from_utf16(units.clone()));
        assert_eq!(report.input().raw().unwrap().code_units(), units);
        assert_eq!(
            report.keywords().unwrap()[0].result(),
            Err(HtmlIconSizeError::DimensionDigits)
        );
    }
}

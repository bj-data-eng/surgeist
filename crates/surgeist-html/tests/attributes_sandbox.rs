use std::collections::HashSet;
use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAttributeObservation,
    HtmlSandboxAuthorDiagnostics, HtmlSandboxAuthorState, HtmlSandboxDirectiveState,
    HtmlSandboxFlag as F, HtmlSandboxFlags, HtmlSandboxKeyword as K, HtmlSandboxRedundancy as R,
    HtmlSandboxReport, HtmlSandboxValue, HtmlSandboxValueError, HtmlSpaceToken,
    HtmlTokenComparison, HtmlValueInterpreter, HtmlValueRecoveryState, KnownHtmlAttribute as A,
    KnownHtmlElement as E, Namespace, QualifiedName,
};

// Independent named expectations from HTML #parse-a-sandboxing-directive,
// raw complete pinned source L65697–65746. Set order is not a semantic claim.
const ALL_RESTRICTIONS: [F; 16] = [
    F::Navigation,
    F::AuxiliaryNavigation,
    F::TopNavigationWithoutUserActivation,
    F::TopNavigationWithUserActivation,
    F::Origin,
    F::Forms,
    F::PointerLock,
    F::Scripts,
    F::AutomaticFeatures,
    F::DocumentDomain,
    F::PropagatesToAuxiliary,
    F::Modals,
    F::OrientationLock,
    F::Presentation,
    F::Downloads,
    F::CustomProtocolsNavigation,
];

fn raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(A::Sandbox.qualified_name(), value)
}

fn interpret(value: &Attribute) -> HtmlSandboxReport {
    HtmlValueInterpreter::new(2048)
        .iframe_sandbox(&E::Iframe.qualified_name(), Some(value))
        .unwrap()
}

fn flags(report: &HtmlSandboxReport) -> &HtmlSandboxFlags {
    let Some(HtmlSandboxDirectiveState::Present(flags)) = report.intrinsic() else {
        panic!("expected a present local directive")
    };
    flags
}

fn diagnostics(report: &HtmlSandboxReport) -> &HtmlSandboxAuthorDiagnostics {
    let HtmlSandboxAuthorState::Nonconforming(diagnostics) = report.author() else {
        panic!("expected independent author diagnostics")
    };
    diagnostics
}

fn expect_restrictions_except(flags: &HtmlSandboxFlags, removed: &[F]) {
    let expected: HashSet<_> = ALL_RESTRICTIONS
        .into_iter()
        .filter(|f| !removed.contains(f))
        .collect();
    let actual: HashSet<_> = flags.iter().collect();
    assert_eq!(actual, expected);
    for restriction in ALL_RESTRICTIONS {
        assert_eq!(flags.contains(restriction), !removed.contains(&restriction));
    }
    assert_eq!(flags.len(), expected.len());
    assert_eq!(flags.is_empty(), expected.is_empty());
}

#[test]
fn missing_is_absent_while_empty_and_whitespace_parse_to_all_restrictions() {
    let interpreter = HtmlValueInterpreter::new(16);
    let missing = interpreter
        .iframe_sandbox(&E::Iframe.qualified_name(), None)
        .unwrap();
    assert_eq!(
        missing.intrinsic(),
        Some(&HtmlSandboxDirectiveState::Absent)
    );
    assert_eq!(missing.author(), &HtmlSandboxAuthorState::Missing);
    assert_eq!(missing.observation(), HtmlAttributeObservation::Missing);
    assert_eq!(missing.tokens(), None);
    assert!(matches!(
        missing.common().unwrap().recovery(),
        HtmlValueRecoveryState::Missing
    ));
    for value in ["", " \t\n\r\u{c}"] {
        let attribute = raw(value);
        let report = interpreter
            .iframe_sandbox(&E::Iframe.qualified_name(), Some(&attribute))
            .unwrap();
        expect_restrictions_except(flags(&report), &[]);
        assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
        assert!(report.tokens().unwrap().tokens().is_empty());
        assert_eq!(report.raw_attribute(), Some(&attribute));
        assert_eq!(
            report.observation(),
            if value.is_empty() {
                HtmlAttributeObservation::PresentEmpty
            } else {
                HtmlAttributeObservation::PresentNonempty
            }
        );
    }
}

#[test]
fn each_source_keyword_relaxes_exactly_its_named_restrictions() {
    // These literal relationships are the normative source clauses, not a
    // producer vocabulary enumeration used to manufacture expected results.
    let cases: &[(K, &str, &[F])] = &[
        (K::AllowDownloads, "allow-downloads", &[F::Downloads]),
        (K::AllowForms, "allow-forms", &[F::Forms]),
        (K::AllowModals, "allow-modals", &[F::Modals]),
        (
            K::AllowOrientationLock,
            "allow-orientation-lock",
            &[F::OrientationLock],
        ),
        (K::AllowPointerLock, "allow-pointer-lock", &[F::PointerLock]),
        (
            K::AllowPopups,
            "allow-popups",
            &[F::AuxiliaryNavigation, F::CustomProtocolsNavigation],
        ),
        (
            K::AllowPopupsToEscapeSandbox,
            "allow-popups-to-escape-sandbox",
            &[F::PropagatesToAuxiliary],
        ),
        (
            K::AllowPresentation,
            "allow-presentation",
            &[F::Presentation],
        ),
        (K::AllowSameOrigin, "allow-same-origin", &[F::Origin]),
        (
            K::AllowScripts,
            "allow-scripts",
            &[F::Scripts, F::AutomaticFeatures],
        ),
        (
            K::AllowTopNavigation,
            "allow-top-navigation",
            &[
                F::TopNavigationWithoutUserActivation,
                F::TopNavigationWithUserActivation,
                F::CustomProtocolsNavigation,
            ],
        ),
        (
            K::AllowTopNavigationByUserActivation,
            "allow-top-navigation-by-user-activation",
            &[F::TopNavigationWithUserActivation],
        ),
        (
            K::AllowTopNavigationToCustomProtocols,
            "allow-top-navigation-to-custom-protocols",
            &[F::CustomProtocolsNavigation],
        ),
    ];
    for &(keyword, spelling, removed) in cases {
        for text in [spelling.to_owned(), spelling.to_ascii_uppercase()] {
            let attribute = raw(text.as_str());
            let report = interpret(&attribute);
            expect_restrictions_except(flags(&report), removed);
            assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
            assert_eq!(report.raw_attribute(), Some(&attribute));
            assert_eq!(
                report.tokens().unwrap().tokens()[0].value(),
                &DocumentString::from(text.as_str())
            );
            assert_eq!(
                K::from_token(&HtmlSpaceToken::new(text).unwrap()),
                Some(keyword)
            );
        }
        let native = HtmlSandboxValue::new([keyword]).unwrap();
        expect_restrictions_except(&native.flags(), removed);
        assert_eq!(native.into_attribute(), raw(spelling));
        assert_eq!(keyword.keyword(), spelling);
        assert!(
            keyword
                .source_clause()
                .ends_with(&format!("#attr-iframe-sandbox-{spelling}"))
        );
    }
}

#[test]
fn navigation_redundancy_is_author_invalid_without_changing_flag_processing() {
    let report = interpret(&raw(
        "allow-top-navigation allow-top-navigation-by-user-activation",
    ));
    assert_eq!(
        diagnostics(&report).redundancies().collect::<Vec<_>>(),
        vec![R::TopNavigationAndByUserActivation]
    );
    expect_restrictions_except(
        flags(&report),
        &[
            F::TopNavigationWithoutUserActivation,
            F::TopNavigationWithUserActivation,
            F::CustomProtocolsNavigation,
        ],
    );
    assert_eq!(
        flags(&report),
        flags(&interpret(&raw("allow-top-navigation")))
    );
}

#[test]
fn custom_protocols_redundancy_is_independent_for_popups_and_top_navigation() {
    for (value, redundant, removed) in [
        (
            "allow-top-navigation-to-custom-protocols allow-popups",
            R::CustomProtocolsAndPopups,
            vec![F::AuxiliaryNavigation, F::CustomProtocolsNavigation],
        ),
        (
            "allow-top-navigation-to-custom-protocols allow-top-navigation",
            R::CustomProtocolsAndTopNavigation,
            vec![
                F::TopNavigationWithoutUserActivation,
                F::TopNavigationWithUserActivation,
                F::CustomProtocolsNavigation,
            ],
        ),
    ] {
        let report = interpret(&raw(value));
        assert_eq!(
            diagnostics(&report).redundancies().collect::<Vec<_>>(),
            vec![redundant]
        );
        expect_restrictions_except(flags(&report), &removed);
    }
}

#[test]
fn duplicate_unknown_and_all_redundancies_coexist_without_author_gating() {
    let attribute = raw(
        "ALLOW-SCRIPTS unknown ALLOW-TOP-NAVIGATION allow-top-navigation-by-user-activation allow-top-navigation-to-custom-protocols allow-popups allow-scripts unknown",
    );
    let report = interpret(&attribute);
    let errors = diagnostics(&report);
    let duplicate = errors.duplicate().unwrap();
    assert_eq!(duplicate.first_index(), 0);
    assert_eq!(duplicate.duplicate_index(), 6);
    assert_eq!(duplicate.token(), &DocumentString::from("allow-scripts"));
    assert_eq!(
        duplicate.comparison(),
        HtmlTokenComparison::AsciiCaseInsensitive
    );
    assert_eq!(
        errors
            .unknown_tokens()
            .iter()
            .map(|token| token.token_index())
            .collect::<Vec<_>>(),
        vec![1, 7]
    );
    assert!(
        errors
            .unknown_tokens()
            .iter()
            .all(|token| token.value() == &DocumentString::from("unknown"))
    );
    assert_eq!(
        errors.redundancies().collect::<Vec<_>>(),
        vec![
            R::TopNavigationAndByUserActivation,
            R::CustomProtocolsAndTopNavigation,
            R::CustomProtocolsAndPopups
        ]
    );
    expect_restrictions_except(
        flags(&report),
        &[
            F::Scripts,
            F::AutomaticFeatures,
            F::TopNavigationWithoutUserActivation,
            F::TopNavigationWithUserActivation,
            F::CustomProtocolsNavigation,
            F::AuxiliaryNavigation,
        ],
    );
    assert_eq!(report.tokens().unwrap().tokens().len(), 8);
    assert_eq!(report.raw_attribute(), Some(&attribute));
}

#[test]
fn duplicate_unknown_spelling_does_not_hide_later_recognized_keywords() {
    let report = interpret(&raw("UNKNOWN unknown allow-forms"));
    let duplicate = diagnostics(&report).duplicate().unwrap();
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (0, 1)
    );
    assert_eq!(diagnostics(&report).unknown_tokens().len(), 2);
    expect_restrictions_except(flags(&report), &[F::Forms]);
}

#[test]
fn scripts_and_same_origin_are_not_an_author_prohibition_or_missing_origin_context() {
    let native = HtmlSandboxValue::new([K::AllowScripts, K::AllowSameOrigin]).unwrap();
    let report = interpret(&native.into_attribute());
    assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
    expect_restrictions_except(
        flags(&report),
        &[F::Scripts, F::AutomaticFeatures, F::Origin],
    );
}

#[test]
fn present_unknown_directive_is_restricted_instead_of_becoming_absent_or_invalid_default() {
    for value in [
        "unknown",
        "allow-storage-access-by-user-activation",
        "allow-scripts,allow-forms",
        "allow-script",
        "0",
    ] {
        let report = interpret(&raw(value));
        assert_eq!(
            diagnostics(&report).unknown_tokens()[0].value(),
            &DocumentString::from(value)
        );
        expect_restrictions_except(flags(&report), &[]);
    }
}

#[test]
fn only_ascii_whitespace_splits_and_only_ascii_case_matches() {
    let attribute = raw(" \tALLOW-SCRIPTS\n\r\u{c}allow-forms ");
    let report = interpret(&attribute);
    expect_restrictions_except(
        flags(&report),
        &[F::Scripts, F::AutomaticFeatures, F::Forms],
    );
    assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
    assert_eq!(report.input_code_units(), attribute.value().len());
    for value in [
        "allow-scripts\u{a0}allow-forms",
        "allow-ſcripts",
        "allow-scrıpts",
        "allow-scripts\u{b}allow-forms",
    ] {
        let report = interpret(&raw(value));
        expect_restrictions_except(flags(&report), &[]);
        assert_eq!(
            diagnostics(&report).unknown_tokens()[0].value(),
            &DocumentString::from(value)
        );
    }
}

#[test]
fn null_and_isolated_surrogates_are_exact_unknown_tokens_not_replacement_or_trimming() {
    let units = vec![b'a' as u16, 0, 0xd800, b' ' as u16, b'a' as u16, 0xdc00];
    let attribute = raw(DocumentString::from_utf16(units.clone()));
    let report = interpret(&attribute);
    assert_eq!(report.raw().unwrap().code_units(), units);
    let errors = diagnostics(&report);
    assert_eq!(
        errors.unknown_tokens()[0].value().code_units(),
        &[b'a' as u16, 0, 0xd800]
    );
    assert_eq!(
        errors.unknown_tokens()[1].value().code_units(),
        &[b'a' as u16, 0xdc00]
    );
    expect_restrictions_except(flags(&report), &[]);
}

#[test]
fn strict_expanded_identity_preserves_prefixes_and_rejects_foreign_or_mismatched_roles() {
    let interpreter = HtmlValueInterpreter::new(64);
    let iframe = QualifiedName::new(Namespace::html(), Some(DocumentString::from("p")), "iframe");
    let attribute = Attribute::new(
        QualifiedName::new(
            Namespace::none(),
            Some(DocumentString::from("q")),
            "sandbox",
        ),
        "allow-forms",
    );
    let report = interpreter
        .iframe_sandbox(&iframe, Some(&attribute))
        .unwrap();
    assert_eq!(report.element(), &iframe);
    assert_eq!(report.raw_attribute(), Some(&attribute));
    expect_restrictions_except(flags(&report), &[F::Forms]);
    for (element, expected) in [
        (
            QualifiedName::new(Namespace::svg(), None, "iframe"),
            HtmlAttributeApplicability::ForeignElementNamespace,
        ),
        (
            QualifiedName::new(Namespace::html(), None, "Iframe"),
            HtmlAttributeApplicability::DifferentElement,
        ),
        (
            E::Div.qualified_name(),
            HtmlAttributeApplicability::DifferentElement,
        ),
    ] {
        let report = interpreter
            .iframe_sandbox(&element, Some(&attribute))
            .unwrap();
        assert_eq!(report.applicability(), expected);
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.author(), &HtmlSandboxAuthorState::Unapplied);
        assert_eq!(report.common(), None);
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
    for name in [
        QualifiedName::new(Namespace::none(), None, "Sandbox"),
        QualifiedName::new(Namespace::html(), None, "sandbox"),
        A::Src.qualified_name(),
    ] {
        let attribute = Attribute::new(name, "allow-forms");
        let report = interpreter
            .iframe_sandbox(&iframe, Some(&attribute))
            .unwrap();
        assert_eq!(
            report.applicability(),
            HtmlAttributeApplicability::DifferentAttribute
        );
        assert_eq!(report.intrinsic(), None);
        assert_eq!(report.raw_attribute(), Some(&attribute));
    }
}

#[test]
fn raw_unit_admission_precedes_wrong_names_and_zero_admits_missing_or_empty() {
    let interpreter = HtmlValueInterpreter::new(3);
    let attribute = Attribute::new(
        A::Src.qualified_name(),
        DocumentString::from_utf16(vec![0xd800, 0xdc00, 0xd800, 0]),
    );
    let error = interpreter
        .iframe_sandbox(&E::Div.qualified_name(), Some(&attribute))
        .unwrap_err();
    assert_eq!(error.capacity(), 3);
    assert_eq!(error.required(), 4);
    let zero = HtmlValueInterpreter::new(0);
    assert_eq!(
        zero.iframe_sandbox(&E::Iframe.qualified_name(), None)
            .unwrap()
            .intrinsic(),
        Some(&HtmlSandboxDirectiveState::Absent)
    );
    expect_restrictions_except(
        flags(
            &zero
                .iframe_sandbox(&E::Iframe.qualified_name(), Some(&raw("")))
                .unwrap(),
        ),
        &[],
    );
    assert_eq!(
        zero.iframe_sandbox(&E::Iframe.qualified_name(), Some(&raw(" ")))
            .unwrap_err()
            .required(),
        1
    );
}

#[test]
fn native_empty_value_means_present_restrictions_and_order_does_not_change_equality() {
    let empty = HtmlSandboxValue::new([]).unwrap();
    expect_restrictions_except(&empty.flags(), &[]);
    assert_eq!(empty.into_attribute(), raw(""));
    let forward = HtmlSandboxValue::new([K::AllowScripts, K::AllowForms]).unwrap();
    let backward = HtmlSandboxValue::new([K::AllowForms, K::AllowScripts]).unwrap();
    assert_eq!(forward, backward);
    assert_eq!(
        forward.tokens().comparison(),
        HtmlTokenComparison::AsciiCaseInsensitive
    );
    assert_eq!(forward.into_attribute(), raw("allow-scripts allow-forms"));
    assert_eq!(backward.into_attribute(), raw("allow-forms allow-scripts"));
}

#[test]
fn native_known_value_rejects_duplicate_without_silently_deduplicating() {
    let error = HtmlSandboxValue::new([K::AllowForms, K::AllowScripts, K::AllowForms]).unwrap_err();
    let HtmlSandboxValueError::Duplicate(duplicate) = error else {
        panic!("expected duplicate rejection")
    };
    assert_eq!(
        (duplicate.first_index(), duplicate.duplicate_index()),
        (0, 2)
    );
    assert_eq!(duplicate.token(), &DocumentString::from("allow-forms"));
}

#[test]
fn native_known_value_rejects_each_source_redundancy_pair_in_either_order() {
    for (left, right, expected) in [
        (
            K::AllowTopNavigation,
            K::AllowTopNavigationByUserActivation,
            R::TopNavigationAndByUserActivation,
        ),
        (
            K::AllowTopNavigationToCustomProtocols,
            K::AllowTopNavigation,
            R::CustomProtocolsAndTopNavigation,
        ),
        (
            K::AllowTopNavigationToCustomProtocols,
            K::AllowPopups,
            R::CustomProtocolsAndPopups,
        ),
    ] {
        assert_eq!(
            HtmlSandboxValue::new([left, right]),
            Err(HtmlSandboxValueError::Redundant(expected))
        );
        assert_eq!(
            HtmlSandboxValue::new([right, left]),
            Err(HtmlSandboxValueError::Redundant(expected))
        );
    }
}

#[test]
fn even_every_raw_relaxation_keeps_the_two_unconditional_restrictions() {
    let report = interpret(&raw(
        "allow-downloads allow-forms allow-modals allow-orientation-lock allow-pointer-lock allow-popups allow-popups-to-escape-sandbox allow-presentation allow-same-origin allow-scripts allow-top-navigation allow-top-navigation-by-user-activation allow-top-navigation-to-custom-protocols",
    ));
    assert_eq!(
        flags(&report).iter().collect::<HashSet<_>>(),
        HashSet::from([F::Navigation, F::DocumentDomain])
    );
    assert_eq!(diagnostics(&report).redundancies().count(), 3);
}

#[test]
fn correct_native_full_relaxations_keep_only_navigation_and_document_domain() {
    let native = HtmlSandboxValue::new([
        K::AllowDownloads,
        K::AllowForms,
        K::AllowModals,
        K::AllowOrientationLock,
        K::AllowPointerLock,
        K::AllowPopups,
        K::AllowPopupsToEscapeSandbox,
        K::AllowPresentation,
        K::AllowSameOrigin,
        K::AllowScripts,
        K::AllowTopNavigation,
    ])
    .unwrap();
    assert_eq!(
        native.flags().iter().collect::<HashSet<_>>(),
        HashSet::from([F::Navigation, F::DocumentDomain])
    );
    let report = interpret(&native.into_attribute());
    assert_eq!(report.author(), &HtmlSandboxAuthorState::Conforming);
    assert_eq!(
        flags(&report).iter().collect::<HashSet<_>>(),
        HashSet::from([F::Navigation, F::DocumentDomain])
    );
}

#[test]
fn report_metadata_names_the_definition_processing_and_existing_row() {
    let report = interpret(&raw("allow-forms"));
    assert_eq!(report.attribute(), A::Sandbox);
    assert_eq!(report.inventory_rows(), &[133]);
    assert!(report.source_clause().ends_with("#attr-iframe-sandbox"));
    assert!(
        report
            .processing_source_clause()
            .ends_with("#parse-a-sandboxing-directive")
    );
    assert!(
        report
            .flag_set_source_clause()
            .ends_with("#iframe-sandboxing-flag-set")
    );
    assert!(
        F::DocumentDomain
            .source_clause()
            .ends_with("#sandboxed-document.domain-browsing-context-flag")
    );
}

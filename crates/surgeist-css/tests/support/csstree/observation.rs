//! Live public-parser observations. Immutable artifact and oracle validation
//! remain in the parent; expected classes are supplied independently.

use super::{
    Adapter, BaselineFailure, BaselineFailureKind, CssErrorCodeName, CssRecoveryActionName,
    EXPECTED_SOURCE_REPOSITORY, EXPECTED_SOURCE_REVISION, EXPECTED_SOURCE_TREE, EntryPoint,
    ExpectedClass, ExpectedDiagnostic, ExpectedRetainedSyntax, ExpectedUnsupportedPolicy,
    Extractor, NeutralInventory, NullableObservation, NullableString, ORACLE_PATH, Observation,
    Outcome, Payload, PayloadRelation, Probe, PropertyOrDescriptor, REGISTRY, RawDiagnostic,
    RawOracle, RawOracleRecord, RegistryEntry, RegistryExtractor, UnsupportedPolicy, ValidatedCase,
    adapters, canonicalize_raw_oracle, observation_matches_expected, payload_relation_holds,
    raw_extractor, resolve_case_registry,
};
use std::panic::{AssertUnwindSafe, catch_unwind};
use surgeist_css::{
    CssErrorCode, CssImportance, CssNamespaceContext, CssNamespaceName, CssNamespacePrefix,
    CssPropertyNameRef, CssRecoveryAction, CssRecoveryDiagnostic, parse_declaration,
    parse_font_face_descriptor_value, parse_media_query, parse_media_query_list,
    parse_property_value_text, parse_rule, parse_selector, parse_selector_list, parse_sheet,
    parse_style_attribute, parse_style_block, validate_sheet, validate_style_attribute,
};

pub(super) fn observe_csstree_oracle(
    inventory: &NeutralInventory,
) -> Result<Vec<u8>, Vec<BaselineFailure>> {
    let mut records = Vec::with_capacity(inventory.cases.len());
    let mut failures = Vec::new();

    for case in &inventory.cases {
        match observe_csstree_record(case) {
            Ok(record) => records.push(record),
            Err(failure) => failures.push(failure),
        }
    }
    failures.sort();
    if !failures.is_empty() {
        return Err(failures);
    }

    let oracle = RawOracle {
        schema_version: 1,
        provider_repository: EXPECTED_SOURCE_REPOSITORY.into(),
        source_revision: EXPECTED_SOURCE_REVISION.into(),
        source_tree: EXPECTED_SOURCE_TREE.into(),
        expectation_schema_version: 1,
        generation_report_sha256: inventory.report_digest.clone(),
        expected_class_registry_sha256: inventory.expected_classes_digest.clone(),
        records,
    };
    canonicalize_raw_oracle(&oracle)
        .map(String::into_bytes)
        .map_err(|error| {
            vec![BaselineFailure {
                case_id: "<oracle>".into(),
                path: ORACLE_PATH.into(),
                context: "<oracle>".into(),
                kind: BaselineFailureKind::AdapterMismatch,
                detail: error,
            }]
        })
}

pub(super) fn observe_csstree_record(
    case: &ValidatedCase,
) -> Result<RawOracleRecord, BaselineFailure> {
    let registry = REGISTRY
        .iter()
        .find(|entry| entry.fixture_path() == case.expectation_path)
        .copied()
        .ok_or_else(|| {
            BaselineFailure::new(
                case,
                BaselineFailureKind::RegistryMissing,
                "neutral expectation path has no explicit registry entry",
            )
        })?;
    let registry = resolve_case_registry(case, registry)
        .map_err(|error| BaselineFailure::new(case, BaselineFailureKind::AdapterMismatch, error))?;
    let complete = registry
        .adapter()
        .wrap(&case.input, registry.property_or_descriptor())
        .map_err(|mismatch| {
            BaselineFailure::new(
                case,
                BaselineFailureKind::AdapterMismatch,
                format!("complete-input adapter failed: {mismatch:?}"),
            )
        })?;
    let span = complete.payload_span();
    let payload = Payload {
        prefix: complete.source()[..span.start].to_owned(),
        suffix: complete.source()[span.end..].to_owned(),
        input_byte_length: case.input.len(),
    };

    let expected_class = case.expected_class.as_ref().ok_or_else(|| {
        BaselineFailure::new(
            case,
            BaselineFailureKind::ExpectedClassMismatch,
            "neutral case reached observation without a prevalidated expected class",
        )
    })?;

    let (probe, outcome, observation) = if matches!(
        expected_class,
        ExpectedClass::Unsupported {
            policy: ExpectedUnsupportedPolicy::PanicFreedomOnly,
            ..
        }
    ) {
        let parsed = catch_unwind(AssertUnwindSafe(|| {
            let _report = parse_style_attribute(complete.source());
            validate_strict_parity(
                _report.is_clean(),
                _report.diagnostics(),
                validate_style_attribute(complete.source()),
            )?;
            Ok::<(), String>(())
        }));
        match parsed {
            Err(_) => {
                return Err(BaselineFailure::new(
                    case,
                    BaselineFailureKind::ParserPanicked,
                    "public parse_style_attribute unwound for adapterless panic-freedom probe",
                ));
            }
            Ok(Err(error)) => {
                return Err(BaselineFailure::new(
                    case,
                    BaselineFailureKind::ExpectedClassMismatch,
                    error,
                ));
            }
            Ok(Ok(())) => {}
        }
        let ExpectedClass::Unsupported { reason, .. } = expected_class else {
            unreachable!("matched panic-freedom expected class")
        };
        (
            Probe::PanicFreedom {
                entry_point: EntryPoint::StyleAttribute,
                adapter: Adapter::CustomPropertyContainment,
                payload,
            },
            Outcome::Unsupported {
                reason: *reason,
                policy: UnsupportedPolicy::PanicFreedomOnly,
            },
            NullableObservation(None),
        )
    } else {
        if !registry.has_truthful_complete_input() {
            return Err(BaselineFailure::new(
                case,
                BaselineFailureKind::AdapterMismatch,
                "active expected class lacks a truthful complete-input registry entry",
            ));
        }
        let observation = observe_public_parser(case, registry, &complete)?;
        let outcome = outcome_for_expected_class(case, expected_class, &observation)?;
        (
            Probe::Active {
                entry_point: registry.entry_point(),
                adapter: registry.adapter(),
                extractor: raw_extractor(registry.extractor()),
                property_or_descriptor: NullableString(
                    registry
                        .property_or_descriptor()
                        .map(PropertyOrDescriptor::name)
                        .map(str::to_owned),
                ),
                options: case.options.clone(),
                payload,
            },
            outcome,
            NullableObservation(Some(observation)),
        )
    };

    Ok(RawOracleRecord {
        id: case.id.clone(),
        path: case.expectation_path.clone(),
        expectation_sha256: case.expectation_sha256.clone(),
        source: case.source.clone(),
        context: case.context,
        input: case.input.clone(),
        options: case.options.clone(),
        probe,
        outcome,
        observation,
    })
}

fn outcome_for_expected_class(
    case: &ValidatedCase,
    expected: &ExpectedClass,
    observation: &Observation,
) -> Result<Outcome, BaselineFailure> {
    let mismatch = |detail: String| {
        BaselineFailure::new(case, BaselineFailureKind::ExpectedClassMismatch, detail)
    };
    match expected {
        ExpectedClass::Clean { retained_syntax } => {
            require_observation_predicates(case, observation, retained_syntax, true, &[])?;
            Ok(Outcome::Clean)
        }
        ExpectedClass::Recovered {
            retained_syntax,
            diagnostics,
        } => {
            require_observation_predicates(case, observation, retained_syntax, false, diagnostics)?;
            Ok(Outcome::Recovered)
        }
        ExpectedClass::StrictRejected {
            retained_syntax,
            diagnostics,
        } => {
            require_observation_predicates(case, observation, retained_syntax, false, diagnostics)?;
            Ok(Outcome::StrictRejected)
        }
        ExpectedClass::Unsupported { reason, policy } => {
            let ExpectedUnsupportedPolicy::FullObservation {
                retained_syntax,
                is_clean,
                diagnostics,
            } = policy
            else {
                return Err(mismatch(
                    "panic-freedom expected class reached full observation".into(),
                ));
            };
            require_observation_predicates(
                case,
                observation,
                retained_syntax,
                *is_clean,
                diagnostics,
            )?;
            Ok(Outcome::Unsupported {
                reason: *reason,
                policy: UnsupportedPolicy::FullObservation,
            })
        }
    }
}

fn require_observation_predicates(
    case: &ValidatedCase,
    observation: &Observation,
    retained_syntax: &ExpectedRetainedSyntax,
    is_clean: bool,
    diagnostics: &[ExpectedDiagnostic],
) -> Result<(), BaselineFailure> {
    if observation_matches_expected(observation, retained_syntax, is_clean, diagnostics) {
        return Ok(());
    }
    Err(BaselineFailure::new(
        case,
        BaselineFailureKind::ExpectedClassMismatch,
        format!(
            "expected retained_syntax={retained_syntax:?} is_clean={is_clean} diagnostics={diagnostics:?}; observed syntax_count={} is_clean={} diagnostics={:?}",
            observation.syntax_count, observation.is_clean, observation.diagnostics
        ),
    ))
}

pub(super) fn observe_public_parser(
    case: &ValidatedCase,
    registry: RegistryEntry,
    complete: &adapters::CompleteInput,
) -> Result<Observation, BaselineFailure> {
    let observed = catch_unwind(AssertUnwindSafe(|| match registry.entry_point() {
        EntryPoint::Sheet => {
            let report = parse_sheet(complete.source());
            let count = registry
                .extractor()
                .extract_sheet(report.syntax())
                .map_err(|mismatch| format!("public sheet extractor failed: {mismatch:?}"))?;
            let observation = observation_from_report(
                raw_extractor(registry.extractor()),
                count,
                report.is_clean(),
                report.diagnostics(),
                complete,
            )?;
            validate_strict_parity(
                report.is_clean(),
                report.diagnostics(),
                validate_sheet(complete.source()),
            )?;
            Ok::<Observation, String>(observation)
        }
        EntryPoint::StyleBlock => {
            let report = parse_style_block(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::StyleBlock,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::Rule => {
            let report = parse_rule(complete.source(), &corpus_namespace_context());
            match registry.adapter() {
                Adapter::TopLevelAtRule => {
                    let count = registry
                        .extractor()
                        .extract_at_rule(report.syntax())
                        .map_err(|mismatch| {
                            format!("public at-rule extractor failed: {mismatch:?}")
                        })?;
                    fragment_observation(report, registry, registry.extractor(), complete, |_| {
                        count
                    })
                }
                Adapter::TopLevelRule => fragment_observation(
                    report,
                    registry,
                    RegistryExtractor::SheetRules,
                    complete,
                    |syntax| usize::from(syntax.is_some()),
                ),
                adapter => Err(format!("public rule adapter mismatch: {adapter:?}")),
            }
        }
        EntryPoint::Declaration => {
            let report = parse_declaration(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::StyleDeclarations,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::PropertyValueText => {
            let Some(PropertyOrDescriptor::Property(property)) = registry.property_or_descriptor()
            else {
                return Err("raw property value entry requires property context".into());
            };
            let report = parse_property_value_text(
                complete.source(),
                CssPropertyNameRef::Known(property),
                CssImportance::Normal,
            );
            if report
                .syntax()
                .as_ref()
                .is_some_and(|value| value.property_name() != CssPropertyNameRef::Known(property))
            {
                return Err("raw value does not match its property context".into());
            }
            fragment_observation(
                report,
                registry,
                RegistryExtractor::KnownDeclaration { index: 0, property },
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::Selector => {
            let report = parse_selector(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::Selector,
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::SelectorList => {
            let report = parse_selector_list(complete.source(), &corpus_namespace_context());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::SelectorList,
                complete,
                |syntax| syntax.as_ref().map_or(0, |list| list.selectors().len()),
            )
        }
        EntryPoint::MediaQuery => {
            let report = parse_media_query(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::MediaQuery,
                complete,
                |_| 1,
            )
        }
        EntryPoint::MediaQueryList => {
            let report = parse_media_query_list(complete.source());
            fragment_observation(
                report,
                registry,
                RegistryExtractor::MediaQueries,
                complete,
                |syntax| syntax.queries().len(),
            )
        }
        EntryPoint::FontFaceDescriptorValue => {
            let Some(PropertyOrDescriptor::FontFaceDescriptor(kind)) =
                registry.property_or_descriptor()
            else {
                return Err("font-face value entry point requires descriptor context".into());
            };
            let report = parse_font_face_descriptor_value(complete.source(), kind.css_kind());
            if report
                .syntax()
                .as_ref()
                .is_some_and(|value| value.kind() != kind.css_kind())
            {
                return Err("font-face value does not match its descriptor context".into());
            }
            fragment_observation(
                report,
                registry,
                RegistryExtractor::FontFaceDescriptor(kind),
                complete,
                |syntax| usize::from(syntax.is_some()),
            )
        }
        EntryPoint::StyleAttribute => {
            let report = parse_style_attribute(complete.source());
            let count = registry
                .extractor()
                .extract_declaration_list(report.syntax())
                .map_err(|mismatch| {
                    format!("public style-attribute extractor failed: {mismatch:?}")
                })?;
            let observation = observation_from_report(
                raw_extractor(registry.extractor()),
                count,
                report.is_clean(),
                report.diagnostics(),
                complete,
            )?;
            validate_strict_parity(
                report.is_clean(),
                report.diagnostics(),
                validate_style_attribute(complete.source()),
            )?;
            Ok::<Observation, String>(observation)
        }
    }));

    match observed {
        Ok(Ok(observation)) => Ok(observation),
        Ok(Err(error)) => Err(BaselineFailure::new(
            case,
            if error.contains("payload relation") {
                BaselineFailureKind::DiagnosticOutsidePayload
            } else {
                BaselineFailureKind::ExtractorMismatch
            },
            error,
        )),
        Err(_) => Err(BaselineFailure::new(
            case,
            BaselineFailureKind::ParserPanicked,
            "public parser or extractor unwound",
        )),
    }
}

pub(super) fn corpus_namespace_context() -> CssNamespaceContext {
    CssNamespaceContext::from_bindings([(
        Some(CssNamespacePrefix::try_new("ns").expect("fixed corpus namespace prefix")),
        CssNamespaceName::new("surgeist-corpus-probe"),
    )])
}

fn fragment_observation<T>(
    report: surgeist_css::CssParseReport<T>,
    registry: RegistryEntry,
    expected_extractor: RegistryExtractor,
    complete: &adapters::CompleteInput,
    count: impl FnOnce(&T) -> usize,
) -> Result<Observation, String> {
    if registry.extractor() != expected_extractor {
        return Err(format!(
            "public fragment extractor mismatch: {:?}",
            registry.extractor()
        ));
    }
    let observation = observation_from_report(
        raw_extractor(expected_extractor),
        count(report.syntax()),
        report.is_clean(),
        report.diagnostics(),
        complete,
    )?;
    let clean = report.is_clean();
    let diagnostics = report.diagnostics().to_vec();
    validate_strict_parity(clean, &diagnostics, report.into_validation_result())?;
    Ok(observation)
}

fn validate_strict_parity<T>(
    is_clean: bool,
    diagnostics: &[CssRecoveryDiagnostic],
    strict: Result<T, surgeist_css::CssValidationFailure>,
) -> Result<(), String> {
    match (is_clean, strict) {
        (true, Ok(_)) => Ok(()),
        (false, Err(failure)) if failure.diagnostics() == diagnostics => Ok(()),
        (true, Err(failure)) => Err(format!(
            "app-strict rejected a clean ordinary report with diagnostics {:?}",
            failure.diagnostics()
        )),
        (false, Ok(_)) => Err("app-strict accepted a recovered ordinary report".into()),
        (false, Err(failure)) => Err(format!(
            "app-strict diagnostics diverged from ordinary report: strict={:?} ordinary={diagnostics:?}",
            failure.diagnostics()
        )),
    }
}

pub(super) fn observation_from_report(
    extractor: Extractor,
    syntax_count: usize,
    is_clean: bool,
    diagnostics: &[CssRecoveryDiagnostic],
    complete: &adapters::CompleteInput,
) -> Result<Observation, String> {
    let payload_span = complete.payload_span();
    let diagnostics = diagnostics
        .iter()
        .map(|diagnostic| {
            let byte_offset = diagnostic.error().position().byte_offset().value();
            let span_start = diagnostic.span().start().byte_offset().value();
            let span_end = diagnostic.span().end().byte_offset().value();
            let intersects = payload_relation_holds(
                PayloadRelation::Intersects,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let ends_at = payload_relation_holds(
                PayloadRelation::EndsAt,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let recovery_ends_at = payload_relation_holds(
                PayloadRelation::RecoveryEndsAt,
                payload_span.clone(),
                byte_offset,
                span_start,
                span_end,
            );
            let payload_relation = if intersects {
                PayloadRelation::Intersects
            } else if ends_at {
                PayloadRelation::EndsAt
            } else if recovery_ends_at {
                PayloadRelation::RecoveryEndsAt
            } else {
                return Err(format!(
                    "no closed payload relation holds for payload span {payload_span:?}: \
                     diagnostic offset={byte_offset} span={span_start}..{span_end}"
                ));
            };
            Ok::<RawDiagnostic, String>(RawDiagnostic {
                code: css_error_code_name(diagnostic.error().code())?,
                action: css_recovery_action_name(diagnostic.action())?,
                byte_offset,
                span_start,
                span_end,
                multiplicity: 1,
                payload_relation,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Observation {
        extractor,
        syntax_count,
        is_clean,
        diagnostics,
    })
}

fn css_error_code_name(code: CssErrorCode) -> Result<CssErrorCodeName, String> {
    match code {
        CssErrorCode::UnexpectedEnd => Ok(CssErrorCodeName::UnexpectedEnd),
        CssErrorCode::UnexpectedToken => Ok(CssErrorCodeName::UnexpectedToken),
        CssErrorCode::InvalidAtRulePlacement => Ok(CssErrorCodeName::InvalidAtRulePlacement),
        CssErrorCode::InvalidAtRulePrelude => Ok(CssErrorCodeName::InvalidAtRulePrelude),
        CssErrorCode::InvalidAtRuleBody => Ok(CssErrorCodeName::InvalidAtRuleBody),
        CssErrorCode::UnknownAtRule => Ok(CssErrorCodeName::UnknownAtRule),
        CssErrorCode::UnsupportedAtRule => Ok(CssErrorCodeName::UnsupportedAtRule),
        CssErrorCode::InvalidQualifiedRule => Ok(CssErrorCodeName::InvalidQualifiedRule),
        CssErrorCode::InvalidSelector => Ok(CssErrorCodeName::InvalidSelector),
        CssErrorCode::InvalidMediaQuery => Ok(CssErrorCodeName::InvalidMediaQuery),
        CssErrorCode::UnknownProperty => Ok(CssErrorCodeName::UnknownProperty),
        CssErrorCode::UnsupportedProperty => Ok(CssErrorCodeName::UnsupportedProperty),
        CssErrorCode::InvalidPropertyValue => Ok(CssErrorCodeName::InvalidPropertyValue),
        CssErrorCode::InvalidDeclarationAnnotation => {
            Ok(CssErrorCodeName::InvalidDeclarationAnnotation)
        }
        CssErrorCode::UnknownDescriptor => Ok(CssErrorCodeName::UnknownDescriptor),
        CssErrorCode::UnsupportedDescriptor => Ok(CssErrorCodeName::UnsupportedDescriptor),
        CssErrorCode::InvalidDescriptorValue => Ok(CssErrorCodeName::InvalidDescriptorValue),
        CssErrorCode::InvalidDescriptorCombination => {
            Ok(CssErrorCodeName::InvalidDescriptorCombination)
        }
        CssErrorCode::InvalidColorSyntax => Ok(CssErrorCodeName::InvalidColorSyntax),
        CssErrorCode::NestingLimit => Ok(CssErrorCodeName::NestingLimit),
        _ => Err("public parser returned an unrecognized CssErrorCode variant".into()),
    }
}

fn css_recovery_action_name(action: CssRecoveryAction) -> Result<CssRecoveryActionName, String> {
    match action {
        CssRecoveryAction::RejectInput => Ok(CssRecoveryActionName::RejectInput),
        CssRecoveryAction::DropDeclaration => Ok(CssRecoveryActionName::DropDeclaration),
        CssRecoveryAction::DropDescriptor => Ok(CssRecoveryActionName::DropDescriptor),
        CssRecoveryAction::DropQualifiedRule => Ok(CssRecoveryActionName::DropQualifiedRule),
        CssRecoveryAction::DropAtRule => Ok(CssRecoveryActionName::DropAtRule),
        CssRecoveryAction::DropKeyframeBlock => Ok(CssRecoveryActionName::DropKeyframeBlock),
        CssRecoveryAction::DropSelectorListItem => Ok(CssRecoveryActionName::DropSelectorListItem),
        CssRecoveryAction::ReplaceMediaQueryWithNever => {
            Ok(CssRecoveryActionName::ReplaceMediaQueryWithNever)
        }
        CssRecoveryAction::RetainWithImplicitClosure => {
            Ok(CssRecoveryActionName::RetainWithImplicitClosure)
        }
        CssRecoveryAction::IgnoreUnterminatedComment => {
            Ok(CssRecoveryActionName::IgnoreUnterminatedComment)
        }
        CssRecoveryAction::StopAtNestingLimit => Ok(CssRecoveryActionName::StopAtNestingLimit),
        _ => Err("public parser returned an unrecognized CssRecoveryAction variant".into()),
    }
}

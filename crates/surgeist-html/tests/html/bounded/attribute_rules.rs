#![forbid(unsafe_code)]

use surgeist_html::{
    Attribute, DocumentString, HtmlAttributeApplicability, HtmlAutocompleteContactHint,
    HtmlAutocompleteCredential, HtmlAutocompleteCredentialOutcome, HtmlAutocompleteMode,
    HtmlAutocompleteProcessingState, HtmlHashNameProcessing, HtmlIconSizeError, HtmlInputType,
    HtmlValueInterpreter, HtmlWritingSuggestions, HtmlWritingSuggestionsState, KnownHtmlAttribute,
    KnownHtmlElement, Namespace, QualifiedName, bounded::*,
};

fn autocomplete_raw(value: impl Into<DocumentString>) -> Attribute {
    Attribute::new(KnownHtmlAttribute::Autocomplete.qualified_name(), value)
}

#[test]
fn autocomplete_owner_retains_literal_processing_and_exact_capacity_limits() {
    let interpreter = HtmlValueInterpreter::new(100);
    let name = KnownHtmlElement::Input.qualified_name();
    let raw = autocomplete_raw("section-a shipping home tel webauthn");
    let run = |limits| {
        interpreter.control_autocomplete_controlled(
            &name,
            Some(&raw),
            Some(HtmlInputType::Text),
            limits,
        )
    };
    let first = run(AutocompleteMaterializationLimits::new(
        100_000, 100_000, 100_000,
    ))
    .unwrap();
    let HtmlAutocompleteProcessingState::Processed(processed) = first.report().processing() else {
        panic!("valid telephone field with explicit hints");
    };
    // The selected HTML pin retains the original terminal credential as the
    // field identity; the preceding tel supplies the contact category.
    assert_eq!(processed.field_name(), &DocumentString::from("webauthn"));
    assert_eq!(processed.mode_hint(), Some(HtmlAutocompleteMode::Shipping));
    assert_eq!(
        processed.contact_hint(),
        Some(HtmlAutocompleteContactHint::Home)
    );
    assert_eq!(
        processed.credential_type(),
        HtmlAutocompleteCredentialOutcome::Credential(HtmlAutocompleteCredential::Webauthn)
    );
    assert_eq!(processed.idl_value(), raw.value());
    assert_eq!(first.report().raw(), Some(raw.value()));
    let usage = first.usage();
    let exact = AutocompleteMaterializationLimits::new(
        usage.allocated_bytes(),
        usage.nested_output_records(),
        usage.peak_scratch_records(),
    );
    let repeated = run(exact).unwrap();
    assert_eq!(repeated.report(), first.report());
    assert_eq!(repeated.usage(), usage);
    assert_eq!(exact.allocated_bytes(), usage.allocated_bytes());
    assert_eq!(exact.nested_output_records(), usage.nested_output_records());
    assert_eq!(exact.scratch_records(), usage.peak_scratch_records());
    let bytes = run(AutocompleteMaterializationLimits::new(
        usage.allocated_bytes() - 1,
        usage.nested_output_records(),
        usage.peak_scratch_records(),
    ))
    .unwrap_err();
    assert!(matches!(
        bytes.kind(),
        AutocompleteMaterializationError::BytesLimit { .. }
    ));
    assert!(bytes.usage().allocated_bytes() > 0);
    let output = run(AutocompleteMaterializationLimits::new(
        usage.allocated_bytes(),
        usage.nested_output_records() - 1,
        usage.peak_scratch_records(),
    ))
    .unwrap_err();
    assert!(matches!(
        output.kind(),
        AutocompleteMaterializationError::OutputLimit { .. }
    ));
    let scratch = run(AutocompleteMaterializationLimits::new(
        usage.allocated_bytes(),
        usage.nested_output_records(),
        usage.peak_scratch_records() - 1,
    ))
    .unwrap_err();
    assert!(matches!(
        scratch.kind(),
        AutocompleteMaterializationError::ScratchLimit { .. }
    ));
    assert!(scratch.usage().peak_scratch_records() > 0);
    let (report, observed) = repeated.into_parts();
    assert_eq!(observed, usage);
    assert_eq!(report.raw(), Some(raw.value()));
    assert_eq!(autocomplete_materialization_work_bound(usize::MAX), None);
    assert_eq!(
        autocomplete_materialization_work_bound(1).unwrap()
            - autocomplete_materialization_work_bound(0).unwrap(),
        12
    );
}

#[test]
fn autocomplete_raw_admission_precedes_foreign_role_and_observes_no_allocation() {
    let raw = autocomplete_raw(DocumentString::from_utf16(vec![0xd800, 110, 97, 109, 101]));
    let original = raw.value().clone();
    let foreign = QualifiedName::new(Namespace::svg(), None, "input");
    let zero = AutocompleteMaterializationLimits::new(0, 0, 0);
    let failure = HtmlValueInterpreter::new(4)
        .control_autocomplete_controlled(&foreign, Some(&raw), None, zero)
        .unwrap_err();
    let (kind, usage) = failure.into_parts();
    assert!(matches!(
        kind,
        AutocompleteMaterializationError::RawAdmission(_)
    ));
    assert_eq!(usage, AutocompleteMaterializationUsage::default());
    assert_eq!(raw.value(), &original);
    let unapplied = HtmlValueInterpreter::new(5)
        .control_autocomplete_controlled(&foreign, Some(&raw), None, zero)
        .unwrap();
    assert_eq!(
        unapplied.report().applicability(),
        HtmlAttributeApplicability::ForeignElementNamespace
    );
    assert_eq!(unapplied.report().raw(), Some(&original));
    assert_eq!(
        unapplied.usage(),
        AutocompleteMaterializationUsage::default()
    );
}

#[test]
fn hash_name_scan_separates_first_hash_read_from_report_retention() {
    let raw = DocumentString::from_utf16(vec![0xd800, 97, 35, 0xdc00]);
    let mut scan = HashNameScan::new(&raw);
    for _ in 0..3 {
        let HashNameScanStep::Continue(next) = scan.step() else {
            panic!("retention is a separate action")
        };
        scan = next;
    }
    let HashNameScanStep::Complete(report) = scan.step() else {
        panic!("first hash is ready")
    };
    assert_eq!(report.first_hash_offset(), Some(2));
    assert_eq!(report.processing(), HtmlHashNameProcessing::Name);
    assert_eq!(report.processing_name_units(), Some(&[0xdc00][..]));
    assert_eq!(report.raw(), Some(&raw));
    assert_eq!((HashNameScan::BEGIN_WORK, HashNameScan::STEP_WORK), (1, 4));
    let missing = surgeist_html::HtmlHashNameReferenceReport::missing();
    assert_eq!(missing.raw(), None);
    assert_eq!(missing.processing(), HtmlHashNameProcessing::Missing);
}

#[test]
fn target_recovery_tracks_first_actual_dangerous_unit_offsets() {
    let mut recovery = TargetRecovery::default();
    recovery.unit(0, 12); // Form feed is not TAB/LF/CR.
    recovery.unit(1, 60);
    assert_eq!(recovery.dangerous(), None);
    recovery.unit(2, 10);
    recovery.unit(3, 13);
    recovery.unit(4, 60);
    assert_eq!(recovery.dangerous(), Some((2, 1)));
}

#[test]
fn writing_and_embedded_classifiers_leave_structural_selection_to_the_caller() {
    let interpreter = HtmlValueInterpreter::new(10);
    assert_eq!(
        interpreter
            .writingsuggestions_processing(None)
            .unwrap()
            .intrinsic(),
        Some(HtmlWritingSuggestionsState::Default)
    );
    let raw = DocumentString::from("FALSE");
    let processing = interpreter
        .writingsuggestions_processing(Some(&raw))
        .unwrap();
    assert_eq!(
        processing.intrinsic(),
        Some(HtmlWritingSuggestionsState::Keyword(
            HtmlWritingSuggestions::False
        ))
    );
    assert!(
        HtmlValueInterpreter::new(4)
            .writingsuggestions_processing(Some(&raw))
            .is_err()
    );
    assert!(input_readonly_applies(HtmlInputType::Text));
    assert!(!input_readonly_applies(HtmlInputType::Checkbox));
    assert!(matches!(
        embedded_dimension_applicability(Some(KnownHtmlElement::Input), None),
        EmbeddedDimensionApplicability::Unavailable(EmbeddedDimensionEvidenceNeeded::InputState)
    ));
    assert!(matches!(
        embedded_dimension_applicability(
            Some(KnownHtmlElement::Input),
            Some(EmbeddedDimensionContext::InputState(HtmlInputType::Image))
        ),
        EmbeddedDimensionApplicability::Applicable
    ));
    assert!(matches!(
        embedded_dimension_applicability(
            Some(KnownHtmlElement::Source),
            Some(EmbeddedDimensionContext::SourceParent(
                EmbeddedDimensionParent::Unavailable
            ))
        ),
        EmbeddedDimensionApplicability::Unavailable(EmbeddedDimensionEvidenceNeeded::SourceParent)
    ));
    assert!(matches!(
        embedded_dimension_applicability(
            Some(KnownHtmlElement::Source),
            Some(EmbeddedDimensionContext::SourceParent(
                EmbeddedDimensionParent::Parent(Some(KnownHtmlElement::Picture))
            ))
        ),
        EmbeddedDimensionApplicability::Applicable
    ));
    assert_eq!(stylesheet_link_context_work_bound(10), Some(93));
    assert_eq!(stylesheet_link_context_work_bound(usize::MAX), None);
}

#[test]
fn icon_cursor_exposes_original_arbitrary_precision_ranges_without_materializing() {
    let units: Vec<_> = "999999999999999999999999999999x123"
        .encode_utf16()
        .collect();
    let mut cursor = IconSizeCursor::new(&units);
    for _ in 0..1000 {
        if cursor.is_complete() {
            break;
        }
        cursor.step();
    }
    let Some(Ok(IconSizeShape::Bitmap { width, height })) = cursor.result() else {
        panic!("valid unbounded dimensions")
    };
    assert_eq!(&units[width.clone()], &units[..30]);
    assert_eq!(&units[height.clone()], &[49, 50, 51]);
    let zero: Vec<_> = "01x2".encode_utf16().collect();
    let mut invalid = IconSizeCursor::new(&zero);
    while !invalid.is_complete() {
        invalid.step();
    }
    assert_eq!(invalid.result(), Some(&Err(HtmlIconSizeError::LeadingZero)));
    assert_eq!(IconSizeCursor::STEP_WORK, 7);
}

#[test]
fn work_metadata_accompanies_existing_owner_checked_report_operations() {
    let interpreter = HtmlValueInterpreter::new(100);
    let form = KnownHtmlElement::Form.qualified_name();
    let map = KnownHtmlElement::Map.qualified_name();
    let button = KnownHtmlElement::Button.qualified_name();
    let name = Attribute::new(KnownHtmlAttribute::Name.qualified_name(), "unique");
    let id = Attribute::new(KnownHtmlAttribute::Id.qualified_name(), "actual-id");
    let is = Attribute::new(KnownHtmlAttribute::Is.qualified_name(), "x-submit");
    assert!(
        surgeist_html::HtmlValueInterpreter::form_name_report_work_bound(&form, Some(&name))
            .unwrap()
            > 0
    );
    assert!(
        interpreter
            .form_name(&form, Some(&name))
            .unwrap()
            .intrinsic()
            .is_some()
    );
    assert!(
        surgeist_html::HtmlValueInterpreter::map_name_report_work_bound(&map, Some(&name)).unwrap()
            > 0
    );
    assert!(
        interpreter
            .map_name(&map, Some(&name))
            .unwrap()
            .intrinsic()
            .is_some()
    );
    assert!(
        surgeist_html::HtmlValueInterpreter::id_report_work_bound(&button, Some(&id)).unwrap() > 0
    );
    assert!(
        interpreter
            .id_attribute(&button, Some(&id))
            .unwrap()
            .intrinsic()
            .is_some()
    );
    assert!(
        surgeist_html::HtmlValueInterpreter::is_attribute_report_work_bound(&button, Some(&is))
            .unwrap()
            > 0
    );
    assert!(
        interpreter
            .is_attribute(&button, Some(&is))
            .unwrap()
            .intrinsic()
            .is_some()
    );
}

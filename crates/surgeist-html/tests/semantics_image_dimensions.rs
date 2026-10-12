use surgeist_html::*;

fn integer(s: &str) -> HtmlNonNegativeInteger {
    HtmlNonNegativeInteger::from_decimal_digits(&s.into()).unwrap()
}
fn measurement(n: &str, d: &str) -> HtmlImageDimensionMeasurement {
    HtmlImageDimensionMeasurement::Known(
        HtmlNonNegativeRational::new(integer(n), integer(d)).unwrap(),
    )
}
fn attr(n: &str, v: impl Into<DocumentString>) -> Attribute {
    Attribute::new(QualifiedName::new(Namespace::none(), None, n), v)
}
fn fixture(local: &str, attrs: Vec<Attribute>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    fixture_name(QualifiedName::new(Namespace::html(), None, local), attrs)
}
fn fixture_name(name: QualifiedName, attrs: Vec<Attribute>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 100000, 1000, 1000000, 1000, 10, 10),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let e = b
        .add_element(
            &d,
            name,
            OrderedAttributes::new(attrs).unwrap(),
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&e, &[]).unwrap();
    b.establish_root(&e).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&e).unwrap();
    (c, id)
}
fn participation_revision() -> HtmlListParticipationRevision {
    HtmlListParticipationRevision::new(
        HtmlParticipationProducerId::fresh(),
        1,
        2,
        HtmlParticipationProducerId::fresh(),
        3,
        4,
    )
}
fn resource_revision() -> HtmlNaturalDimensionRevision {
    HtmlNaturalDimensionRevision::new(HtmlNaturalDimensionProducerId::fresh(), 5, 6)
}
fn operation(role: HtmlImageDimensionRole, preferred: bool) -> HtmlImageDimensionOperationKey {
    let operation = HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 7, 8);
    let css = HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 9, 10);
    match (role, preferred) {
        (HtmlImageDimensionRole::Img, true) => {
            HtmlImageDimensionOperationKey::ImgPreferredDensityCorrected {
                operation,
                request: 11,
                density: 12,
                preferred: 13,
            }
        }
        (HtmlImageDimensionRole::Img, false) => HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation,
            request: 11,
            density: 12,
            preferred: 13,
            css,
        },
        (HtmlImageDimensionRole::Input, _) => HtmlImageDimensionOperationKey::InputDefaultSized {
            operation,
            request: 11,
            css,
        },
    }
}
fn expectation(
    id: &HtmlCaptureSubjectId,
    role: HtmlImageDimensionRole,
    op: Option<HtmlImageDimensionOperationKey>,
) -> HtmlImageDimensionExpectation {
    HtmlImageDimensionExpectation::new(
        id.clone(),
        role,
        HtmlImageDimensionSelection::Selected(HtmlNaturalResourceKey::new(
            HtmlNaturalResourceId::fresh(),
            14,
            15,
        )),
        11,
        op,
    )
}
fn fact_limits() -> HtmlImageDimensionFactsLimits {
    HtmlImageDimensionFactsLimits::new(1000, 10000000, 1000000)
}
fn limits() -> HtmlSemanticLimits {
    HtmlSemanticLimits::new(10000000, 100, 10000, 10000, 10000000, 100)
}
fn context(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    rendered: Option<bool>,
    resource: Option<(
        HtmlNaturalDimensionRevision,
        HtmlImageDimensionExpectation,
        HtmlImageDimensionResourceObservation,
    )>,
    axes: Option<(HtmlImageDimensionMeasurement, HtmlImageDimensionMeasurement)>,
) -> HtmlSemanticContext {
    let p = participation_revision();
    let participation = HtmlListParticipation::new(
        c,
        p.clone(),
        vec![HtmlListParticipationFact::new(
            id.clone(),
            None,
            rendered,
            None,
        )],
        10,
    )
    .unwrap();
    let (r, expected, resources) = resource.map_or((None, vec![], vec![]), |(r, e, o)| {
        (Some(r), vec![e], vec![o])
    });
    let boxes = axes.map_or_else(Vec::new, |(w, h)| {
        vec![HtmlImageContentBoxObservation::new(
            c.identity().clone(),
            c.host_facts_revision().clone(),
            id.clone(),
            p.clone(),
            w,
            h,
        )]
    });
    let revision = HtmlImageDimensionRevision::new(r, Some(p.clone()));
    let facts = HtmlImageDimensionFacts::new(
        c,
        c.host_facts_revision().clone(),
        revision.clone(),
        expected,
        resources,
        boxes,
        fact_limits(),
    )
    .unwrap();
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![])
            .with_list_participation(p, Some(participation))
            .with_image_dimensions(revision, Some(facts)),
        100,
    )
    .unwrap()
}
fn empty_context(c: &HtmlCapture) -> HtmlSemanticContext {
    HtmlSemanticContext::new(
        c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]),
        100,
    )
    .unwrap()
}
fn request(id: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::ImageDimensions)
}
fn run(c: &HtmlCapture, cx: &HtmlSemanticContext, id: &HtmlCaptureSubjectId) -> HtmlSemanticReport {
    HtmlSemanticSession::new(
        c,
        cx,
        HtmlValueInterpreter::new(10000),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(id)])
    .unwrap()
}
fn result(r: &HtmlSemanticReport) -> &HtmlImageDimensionReadback {
    match r.results()[0].fact() {
        HtmlSemanticFact::ImageDimensions(HtmlSemanticValue::Known(v)) => v,
        v => panic!("{v:?}"),
    }
}
fn assert_axis(
    axis: &HtmlImageDimensionAxisResult,
    n: &str,
    d: &str,
    origin: HtmlImageDimensionOrigin,
) {
    let HtmlImageDimensionAxisResult::Known(v) = axis else {
        panic!("{axis:?}")
    };
    assert_eq!(v.css_pixels().numerator().decimal_digits(), n);
    assert_eq!(v.css_pixels().denominator().decimal_digits(), d);
    assert_eq!(v.origin(), origin);
}
fn resource(
    c: &HtmlCapture,
    id: &HtmlCaptureSubjectId,
    state: HtmlImgRequestState,
    preferred: bool,
    axes: Option<(HtmlImageDimensionMeasurement, HtmlImageDimensionMeasurement)>,
) -> HtmlSemanticContext {
    let r = resource_revision();
    let op = operation(HtmlImageDimensionRole::Img, preferred);
    let e = expectation(id, HtmlImageDimensionRole::Img, Some(op.clone()));
    let completed = axes.map(|(w, h)| HtmlImageDimensionCompleted::new(op, w, h));
    let o =
        HtmlImageDimensionResourceObservation::img(r.clone(), e.clone(), Some(state), completed);
    context(c, id, Some(false), Some((r, e, o)), None)
}

#[test]
fn all_non_image_input_states_return_zero_without_rendering_resources_or_raw_dimensions() {
    // The 22 source bookkeeping states, independently listed from the defining table.
    for state in [
        "hidden",
        "text",
        "search",
        "tel",
        "url",
        "email",
        "password",
        "date",
        "month",
        "week",
        "time",
        "datetime-local",
        "number",
        "range",
        "color",
        "checkbox",
        "radio",
        "file",
        "submit",
        "reset",
        "button",
        "",
        "unknown",
    ] {
        let (c, id) = fixture(
            "input",
            vec![
                attr("type", state),
                attr("width", "999999999999999999999999"),
                attr("height", "999999999999999999999999"),
            ],
        );
        let cx = empty_context(&c);
        let r = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(16),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[request(&id)])
        .unwrap();
        let v = result(&r);
        assert_eq!(
            v.branch(),
            HtmlImageDimensionBranch::NonImageInput,
            "{state}"
        );
        assert_axis(
            v.width(),
            "0",
            "1",
            HtmlImageDimensionOrigin::NonImageInputZero,
        );
        assert_axis(
            v.height(),
            "0",
            "1",
            HtmlImageDimensionOrigin::NonImageInputZero,
        );
        assert_eq!(
            v.width_attribute(),
            &HtmlImageDimensionAttributeObservation::Uninspected
        );
    }
    let (c, id) = fixture("input", vec![attr("type", "IMAGE")]);
    assert!(
        matches!(run(&c,&empty_context(&c),&id).results()[0].fact(), HtmlSemanticFact::ImageDimensions(HtmlSemanticValue::Missing(HtmlSemanticMissing::BeingRendered(s))) if s == &id)
    );
    let (c, id) = fixture("input", vec![]);
    assert_eq!(
        result(&run(&c, &empty_context(&c), &id)).branch(),
        HtmlImageDimensionBranch::NonImageInput
    );
}

#[test]
fn actual_rendered_content_box_bypasses_attributes_and_unavailable_resources() {
    for local in ["img", "input"] {
        let (c, id) = fixture(
            local,
            vec![
                attr("type", "image"),
                attr("width", "999999999999999999999999"),
                attr("height", "999999999999999999999999"),
            ],
        );
        let cx = context(
            &c,
            &id,
            Some(true),
            None,
            Some((measurement("7", "2"), measurement("9", "4"))),
        );
        let r = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(5),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[request(&id)])
        .unwrap();
        let v = result(&r);
        assert_axis(v.width(), "7", "2", HtmlImageDimensionOrigin::ContentBox);
        assert_axis(v.height(), "9", "4", HtmlImageDimensionOrigin::ContentBox);
        assert_eq!(
            v.height_attribute(),
            &HtmlImageDimensionAttributeObservation::Uninspected
        );
        assert!(v.source_clause().ends_with(if local == "img" {
            "#img-dimensions"
        } else {
            "#input-img-dimensions"
        }));
    }
}
#[test]
fn rendered_zero_is_a_content_box_and_missing_boxes_never_fall_back() {
    let (c, id) = fixture("img", vec![attr("width", "12"), attr("height", "13")]);
    let cx = context(
        &c,
        &id,
        Some(true),
        None,
        Some((measurement("0", "999"), measurement("0", "7"))),
    );
    assert_axis(
        result(&run(&c, &cx, &id)).width(),
        "0",
        "1",
        HtmlImageDimensionOrigin::ContentBox,
    );
    let cx = context(&c, &id, Some(true), None, None);
    assert!(
        matches!(result(&run(&c,&cx,&id)).width(),HtmlImageDimensionAxisResult::Missing(m) if m.reason() == HtmlImageDimensionMissingReason::ContentBoxWidth)
    );
}
#[test]
fn unknown_rendering_remains_influencing_even_with_two_successful_attributes() {
    let (c, id) = fixture(
        "img",
        vec![
            attr("hidden", ""),
            attr("width", "12"),
            attr("height", "13"),
        ],
    );
    let cx = context(&c, &id, None, None, None);
    assert!(
        matches!(run(&c,&cx,&id).results()[0].fact(), HtmlSemanticFact::ImageDimensions(HtmlSemanticValue::Missing(HtmlSemanticMissing::BeingRendered(s))) if s == &id)
    );
}
#[test]
fn known_unavailable_and_broken_img_states_supply_null_argument_zero() {
    for state in [
        HtmlImgRequestState::Unavailable,
        HtmlImgRequestState::Broken,
    ] {
        let (c, id) = fixture("img", vec![]);
        let cx = resource(&c, &id, state, true, None);
        let r = run(&c, &cx, &id);
        assert_axis(
            result(&r).width(),
            "0",
            "1",
            HtmlImageDimensionOrigin::UnavailableImageZero,
        );
        assert_axis(
            result(&r).height(),
            "0",
            "1",
            HtmlImageDimensionOrigin::UnavailableImageZero,
        );
    }
}
#[test]
fn partial_and_complete_availability_select_completed_density_result() {
    for state in [
        HtmlImgRequestState::PartiallyAvailable,
        HtmlImgRequestState::CompletelyAvailable,
    ] {
        let (c, id) = fixture("img", vec![]);
        // External completed result for preferred 600x300 at density 2.
        let cx = resource(
            &c,
            &id,
            state,
            true,
            Some((measurement("300", "1"), measurement("150", "1"))),
        );
        let r = run(&c, &cx, &id);
        assert_axis(
            result(&r).width(),
            "300",
            "1",
            HtmlImageDimensionOrigin::ImgPreferredDensityCorrected,
        );
        assert_axis(
            result(&r).height(),
            "150",
            "1",
            HtmlImageDimensionOrigin::ImgPreferredDensityCorrected,
        );
    }
}
#[test]
fn separate_img_and_input_default_sizing_roles_are_consumed_without_css_reimplementation() {
    let (c, id) = fixture("img", vec![]);
    let cx = resource(
        &c,
        &id,
        HtmlImgRequestState::CompletelyAvailable,
        false,
        Some((measurement("100", "1"), measurement("50", "1"))),
    );
    assert_axis(
        result(&run(&c, &cx, &id)).height(),
        "50",
        "1",
        HtmlImageDimensionOrigin::ImgDefaultSized,
    );
    let (c, id) = fixture("input", vec![attr("type", "image")]);
    let r = resource_revision();
    let op = operation(HtmlImageDimensionRole::Input, false);
    let e = expectation(&id, HtmlImageDimensionRole::Input, Some(op.clone()));
    let o = HtmlImageDimensionResourceObservation::input(
        r.clone(),
        e.clone(),
        Some(HtmlInputImageAvailability::Available),
        Some(HtmlImageDimensionCompleted::new(
            op,
            measurement("100", "1"),
            measurement("50", "1"),
        )),
    );
    let cx = context(&c, &id, Some(false), Some((r, e, o)), None);
    assert_axis(
        result(&run(&c, &cx, &id)).width(),
        "100",
        "1",
        HtmlImageDimensionOrigin::InputDefaultSized,
    );
}
#[test]
fn input_known_not_available_is_distinct_from_missing_availability() {
    let (c, id) = fixture("input", vec![attr("type", "image")]);
    for (availability, zero) in [
        (Some(HtmlInputImageAvailability::NotAvailable), true),
        (None, false),
    ] {
        let r = resource_revision();
        let e = expectation(&id, HtmlImageDimensionRole::Input, None);
        let o =
            HtmlImageDimensionResourceObservation::input(r.clone(), e.clone(), availability, None);
        let cx = context(&c, &id, Some(false), Some((r, e, o)), None);
        let r = run(&c, &cx, &id);
        if zero {
            assert_axis(
                result(&r).width(),
                "0",
                "1",
                HtmlImageDimensionOrigin::UnavailableImageZero,
            );
        } else {
            assert!(
                matches!(result(&r).width(),HtmlImageDimensionAxisResult::Missing(m) if m.reason()==HtmlImageDimensionMissingReason::Availability)
            );
        }
    }
}
#[test]
fn unavailable_completed_axes_are_independent_and_overridden_axes_do_not_copy_them() {
    let (c, id) = fixture("img", vec![attr("width", "12")]);
    let cx = resource(
        &c,
        &id,
        HtmlImgRequestState::CompletelyAvailable,
        true,
        Some((
            HtmlImageDimensionMeasurement::Unsupported(
                HtmlNaturalDimensionUnsupported::NonRational,
            ),
            measurement("9", "4"),
        )),
    );
    let r = run(&c, &cx, &id);
    assert_axis(
        result(&r).width(),
        "12",
        "1",
        HtmlImageDimensionOrigin::ContentAttribute,
    );
    assert_axis(
        result(&r).height(),
        "9",
        "4",
        HtmlImageDimensionOrigin::ImgPreferredDensityCorrected,
    );
    let (c, id) = fixture("img", vec![]);
    let cx = resource(
        &c,
        &id,
        HtmlImgRequestState::PartiallyAvailable,
        true,
        Some((
            measurement("7", "2"),
            HtmlImageDimensionMeasurement::Unavailable,
        )),
    );
    let r = run(&c, &cx, &id);
    assert_axis(
        result(&r).width(),
        "7",
        "2",
        HtmlImageDimensionOrigin::ImgPreferredDensityCorrected,
    );
    assert!(
        matches!(result(&r).height(),HtmlImageDimensionAxisResult::Missing(m) if m.reason()==HtmlImageDimensionMissingReason::CompletedHeight)
    );
}
#[test]
fn unavailable_resource_or_completed_pair_never_mints_null_or_discards_proved_axis() {
    let (c, id) = fixture("img", vec![attr("width", "12")]);
    let cx = context(&c, &id, Some(false), None, None);
    let r = run(&c, &cx, &id);
    assert_axis(
        result(&r).width(),
        "12",
        "1",
        HtmlImageDimensionOrigin::ContentAttribute,
    );
    assert!(
        matches!(result(&r).height(),HtmlImageDimensionAxisResult::Missing(m) if m.reason()==HtmlImageDimensionMissingReason::ResourceObservation)
    );
    let cx = resource(&c, &id, HtmlImgRequestState::PartiallyAvailable, true, None);
    assert!(
        matches!(result(&run(&c,&cx,&id)).height(),HtmlImageDimensionAxisResult::Missing(m) if m.reason()==HtmlImageDimensionMissingReason::CompletedHeight)
    );
}
#[test]
fn common_prefix_sign_zero_and_exact_huge_integer_overrides_are_preserved() {
    for (raw, expected, recovery) in [
        (
            " +12tail",
            Some("12"),
            HtmlImageDimensionRecovery::Recovered,
        ),
        ("-000tail", Some("0"), HtmlImageDimensionRecovery::Recovered),
        ("-1", None, HtmlImageDimensionRecovery::Negative),
        ("", None, HtmlImageDimensionRecovery::InvalidSyntax),
        (
            "340282366920938463463374607431768211456",
            Some("340282366920938463463374607431768211456"),
            HtmlImageDimensionRecovery::Recovered,
        ),
    ] {
        let (c, id) = fixture("img", vec![attr("width", raw), attr("height", "3")]);
        let cx = context(&c, &id, Some(false), None, None);
        let r = run(&c, &cx, &id);
        let v = result(&r);
        if let Some(expected) = expected {
            assert_axis(
                v.width(),
                expected,
                "1",
                HtmlImageDimensionOrigin::ContentAttribute,
            );
        } else {
            assert!(matches!(
                v.width(),
                HtmlImageDimensionAxisResult::Missing(_)
            ));
        }
        assert!(
            matches!(v.width_attribute(),HtmlImageDimensionAttributeObservation::Present { attribute,recovery:r } if attribute.value().code_units()==raw.encode_utf16().collect::<Vec<_>>() && *r==recovery)
        );
        assert_axis(
            v.height(),
            "3",
            "1",
            HtmlImageDimensionOrigin::ContentAttribute,
        );
    }
}
#[test]
fn exact_utf16_and_expanded_attribute_selection_use_original_slots() {
    for (raw, expected) in [
        (vec![0xd800, 49, 50], None),
        (vec![49, 50, 0xdfff], Some("12")),
        (vec![0], None),
    ] {
        let (c, id) = fixture(
            "img",
            vec![
                attr("width", DocumentString::from_utf16(raw.clone())),
                attr("height", "4"),
            ],
        );
        let cx = context(&c, &id, Some(false), None, None);
        let r = run(&c, &cx, &id);
        let v = result(&r);
        assert!(
            matches!(v.width_attribute(),HtmlImageDimensionAttributeObservation::Present {attribute,..} if attribute.value().code_units()==raw)
        );
        if let Some(n) = expected {
            assert_axis(
                v.width(),
                n,
                "1",
                HtmlImageDimensionOrigin::ContentAttribute,
            );
        } else {
            assert!(matches!(
                v.width(),
                HtmlImageDimensionAxisResult::Missing(_)
            ));
        }
    }
    let (c, id) = fixture(
        "img",
        vec![
            Attribute::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), None, "width"),
                "99",
            ),
            attr("WIDTH", "98"),
            Attribute::new(
                QualifiedName::new(Namespace::none(), Some("p".into()), "width"),
                "5",
            ),
            attr("height", "6"),
        ],
    );
    let cx = context(&c, &id, Some(false), None, None);
    let r = run(&c, &cx, &id);
    assert_axis(
        result(&r).width(),
        "5",
        "1",
        HtmlImageDimensionOrigin::ContentAttribute,
    );
}
#[test]
fn raw_caps_are_enforced_before_prefix_projection_in_unrendered_branch() {
    let (c, id) = fixture("img", vec![attr("width", "1tail"), attr("height", "2")]);
    let cx = context(&c, &id, Some(false), None, None);
    let error = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(4),
        limits(),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        HtmlSemanticFailureKind::Attribute(_)
    ));
    assert_eq!(error.usage().scratch_records(), 0);
}
#[test]
fn unrelated_and_foreign_elements_are_inapplicable() {
    for name in [
        QualifiedName::new(Namespace::html(), None, "iframe"),
        QualifiedName::new(Namespace::from_uri("urn:x"), None, "img"),
        QualifiedName::new(Namespace::html(), None, "IMG"),
    ] {
        let (c, id) = fixture_name(name, vec![attr("width", "999999999")]);
        let cx = empty_context(&c);
        assert!(matches!(
            run(&c, &cx, &id).results()[0].fact(),
            HtmlSemanticFact::ImageDimensions(HtmlSemanticValue::Inapplicable)
        ));
    }
}
#[test]
fn resource_join_rejects_stale_selection_operation_and_wrong_roles() {
    let (c, id) = fixture("img", vec![]);
    let rev = resource_revision();
    let op = operation(HtmlImageDimensionRole::Img, true);
    let e = expectation(&id, HtmlImageDimensionRole::Img, Some(op.clone()));
    let make = |obs| {
        HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlImageDimensionRevision::new(Some(rev.clone()), None),
            vec![e.clone()],
            vec![obs],
            vec![],
            fact_limits(),
        )
    };
    let stale = HtmlImageDimensionResourceObservation::img(
        resource_revision(),
        e.clone(),
        Some(HtmlImgRequestState::Broken),
        None,
    );
    assert!(matches!(
        make(stale),
        Err(HtmlImageDimensionFactsError::ResourceRevisionMismatch)
    ));
    let stale_binding = expectation(&id, HtmlImageDimensionRole::Img, Some(op.clone()));
    let obs = HtmlImageDimensionResourceObservation::img(
        rev.clone(),
        stale_binding,
        Some(HtmlImgRequestState::Broken),
        None,
    );
    assert!(matches!(
        make(obs),
        Err(HtmlImageDimensionFactsError::BindingMismatch(_))
    ));
    let wrong = HtmlImageDimensionCompleted::new(
        operation(HtmlImageDimensionRole::Input, false),
        measurement("1", "1"),
        measurement("1", "1"),
    );
    let obs = HtmlImageDimensionResourceObservation::img(
        rev.clone(),
        e.clone(),
        Some(HtmlImgRequestState::PartiallyAvailable),
        Some(wrong),
    );
    assert!(matches!(
        make(obs),
        Err(HtmlImageDimensionFactsError::InvalidOperation(_))
    ));
    let obs = HtmlImageDimensionResourceObservation::input(
        rev.clone(),
        e.clone(),
        Some(HtmlInputImageAvailability::NotAvailable),
        None,
    );
    assert!(matches!(
        make(obs),
        Err(HtmlImageDimensionFactsError::RoleMismatch(_))
    ));
    let obs = HtmlImageDimensionResourceObservation::img(
        rev.clone(),
        e.clone(),
        Some(HtmlImgRequestState::Broken),
        Some(HtmlImageDimensionCompleted::new(
            op,
            measurement("1", "1"),
            measurement("1", "1"),
        )),
    );
    assert!(matches!(
        make(obs),
        Err(HtmlImageDimensionFactsError::InvalidAvailability(_))
    ));
}
#[test]
fn available_resource_requires_selected_key_and_unavailable_selection_does_not_certify_observation()
{
    let (c, id) = fixture("img", vec![]);
    let rev = resource_revision();
    for (selection, error_available) in [
        (HtmlImageDimensionSelection::KnownNone, true),
        (HtmlImageDimensionSelection::Unavailable, false),
    ] {
        let e = HtmlImageDimensionExpectation::new(
            id.clone(),
            HtmlImageDimensionRole::Img,
            selection,
            11,
            None,
        );
        let o = HtmlImageDimensionResourceObservation::img(
            rev.clone(),
            e.clone(),
            Some(HtmlImgRequestState::PartiallyAvailable),
            None,
        );
        let r = HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlImageDimensionRevision::new(Some(rev.clone()), None),
            vec![e],
            vec![o],
            vec![],
            fact_limits(),
        );
        if error_available {
            assert!(matches!(
                r,
                Err(HtmlImageDimensionFactsError::InvalidAvailability(_))
            ));
        } else {
            assert!(matches!(
                r,
                Err(HtmlImageDimensionFactsError::BindingMismatch(_))
            ));
        }
    }
}
#[test]
fn box_join_rejects_false_rendering_and_stale_capture_host_or_participation() {
    let (c, id) = fixture("img", vec![]);
    let p = participation_revision();
    let revision = HtmlImageDimensionRevision::new(None, Some(p.clone()));
    let box_obs = || {
        HtmlImageContentBoxObservation::new(
            c.identity().clone(),
            c.host_facts_revision().clone(),
            id.clone(),
            p.clone(),
            measurement("1", "1"),
            measurement("2", "1"),
        )
    };
    let facts = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        revision.clone(),
        vec![],
        vec![],
        vec![box_obs()],
        fact_limits(),
    )
    .unwrap();
    let part = HtmlListParticipation::new(
        &c,
        p.clone(),
        vec![HtmlListParticipationFact::new(
            id.clone(),
            None,
            Some(false),
            None,
        )],
        10,
    )
    .unwrap();
    let input = HtmlSemanticContextInput::new(vec![])
        .with_list_participation(p.clone(), Some(part))
        .with_image_dimensions(revision.clone(), Some(facts));
    assert!(matches!(
        HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 100),
        Err(HtmlSemanticContextError::ImageDimensionsFacts(
            HtmlImageDimensionFactsError::ContradictoryBox(_)
        ))
    ));
    let stale = HtmlImageContentBoxObservation::new(
        c.identity().clone(),
        c.host_facts_revision().clone(),
        id.clone(),
        participation_revision(),
        measurement("1", "1"),
        measurement("2", "1"),
    );
    assert!(matches!(
        HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            revision,
            vec![],
            vec![],
            vec![stale],
            fact_limits()
        ),
        Err(HtmlImageDimensionFactsError::ParticipationRevisionMismatch)
    ));
    assert!(matches!(
        HtmlImageDimensionFacts::new(
            &c,
            HtmlHostFactsRevision::fresh(),
            HtmlImageDimensionRevision::new(None, None),
            vec![],
            vec![],
            vec![],
            fact_limits()
        ),
        Err(HtmlImageDimensionFactsError::HostRevisionMismatch)
    ));
}
#[test]
fn finite_fact_admission_counts_incoming_capacity_bytes_and_validation_work() {
    let (c, id) = fixture("img", vec![]);
    let r = resource_revision();
    let e = expectation(&id, HtmlImageDimensionRole::Img, None);
    let revision = HtmlImageDimensionRevision::new(Some(r), None);
    for (limit, kind) in [
        (
            HtmlImageDimensionFactsLimits::new(0, usize::MAX, usize::MAX),
            0,
        ),
        (HtmlImageDimensionFactsLimits::new(100, 0, usize::MAX), 1),
        (HtmlImageDimensionFactsLimits::new(100, usize::MAX, 1), 2),
    ] {
        let err = HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            revision.clone(),
            vec![e.clone()],
            vec![],
            vec![],
            limit,
        )
        .unwrap_err();
        assert!(match kind {
            0 => err == HtmlImageDimensionFactsError::RecordLimit,
            1 => err == HtmlImageDimensionFactsError::ByteLimit,
            _ => err == HtmlImageDimensionFactsError::WorkLimit,
        });
    }
    assert!(matches!(
        HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            revision,
            vec![e.clone(), e],
            vec![],
            vec![],
            fact_limits()
        ),
        Err(HtmlImageDimensionFactsError::DuplicateSubject(_))
    ));
}
#[test]
fn numeric_digit_capacity_work_cancellation_and_atomic_batch_failures_are_qualified_and_sticky() {
    let (c, id) = fixture("img", vec![attr("width", "123456789"), attr("height", "2")]);
    let cx = context(&c, &id, Some(false), None, None);
    for l in [
        HtmlSemanticLimits::new(100000, 10, 10, 8, 1000000, 10),
        HtmlSemanticLimits::new(0, 10, 10, 100, 1000000, 10),
        HtmlSemanticLimits::new(100000, 0, 10, 100, 1000000, 10),
    ] {
        let mut s = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(100),
            l,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let err = s
            .query(&[
                HtmlSemanticRequest::new(id.clone(), HtmlSemanticQuery::Hidden),
                request(&id),
            ])
            .unwrap_err();
        assert!(matches!(
            err.kind(),
            HtmlSemanticFailureKind::LimitExceeded { .. }
        ));
        assert_eq!(
            err.qualification().unwrap().image_dimension_revision(),
            cx.image_dimension_revision()
        );
        assert_eq!(err.usage().scratch_records(), 0);
        assert_eq!(s.query(&[]).unwrap_err(), err);
    }
    let cancel = HtmlSemanticCancellation::new();
    cancel.cancel();
    let error = HtmlSemanticSession::new(&c, &cx, HtmlValueInterpreter::new(100), limits(), cancel)
        .unwrap()
        .query(&[request(&id)])
        .unwrap_err();
    assert_eq!(error.kind(), &HtmlSemanticFailureKind::Cancelled);
}
#[test]
fn representation_equality_does_not_reduce_distinct_positive_fraction_observations() {
    let (c, id) = fixture("img", vec![]);
    let a = context(
        &c,
        &id,
        Some(true),
        None,
        Some((measurement("1", "2"), measurement("1", "1"))),
    );
    let b = context(
        &c,
        &id,
        Some(true),
        None,
        Some((measurement("2", "4"), measurement("1", "1"))),
    );
    let a = run(&c, &a, &id);
    let b = run(&c, &b, &id);
    assert_ne!(result(&a).width(), result(&b).width());
}

#[test]
fn known_no_selected_resource_needs_observed_unavailability_and_is_not_a_default() {
    let (c, id) = fixture("img", vec![]);
    let r = resource_revision();
    let e = HtmlImageDimensionExpectation::new(
        id.clone(),
        HtmlImageDimensionRole::Img,
        HtmlImageDimensionSelection::KnownNone,
        11,
        None,
    );
    let missing = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlImageDimensionRevision::new(Some(r.clone()), None),
        vec![e.clone()],
        vec![],
        vec![],
        fact_limits(),
    )
    .unwrap();
    let p = participation_revision();
    let part = HtmlListParticipation::new(
        &c,
        p.clone(),
        vec![HtmlListParticipationFact::new(
            id.clone(),
            None,
            Some(false),
            None,
        )],
        10,
    )
    .unwrap();
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![])
            .with_list_participation(p, Some(part))
            .with_image_dimensions(missing.revision().clone(), Some(missing)),
        100,
    )
    .unwrap();
    assert!(matches!(
        result(&run(&c, &cx, &id)).width(),
        HtmlImageDimensionAxisResult::Missing(_)
    ));
    let obs = HtmlImageDimensionResourceObservation::img(
        r.clone(),
        e.clone(),
        Some(HtmlImgRequestState::Unavailable),
        None,
    );
    let cx = context(&c, &id, Some(false), Some((r, e, obs)), None);
    assert_axis(
        result(&run(&c, &cx, &id)).width(),
        "0",
        "1",
        HtmlImageDimensionOrigin::UnavailableImageZero,
    );
}

#[test]
fn completed_request_density_preferred_and_css_revisions_are_independently_joined() {
    let (c, id) = fixture("img", vec![]);
    let r = resource_revision();
    let operation_rev = HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 1, 2);
    let css = HtmlImageSizingRevision::new(HtmlParticipationProducerId::fresh(), 3, 4);
    let key = HtmlImageDimensionOperationKey::ImgDefaultSized {
        operation: operation_rev.clone(),
        request: 11,
        density: 12,
        preferred: 13,
        css: css.clone(),
    };
    let expected = expectation(&id, HtmlImageDimensionRole::Img, Some(key));
    for changed in [
        HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation: operation_rev.clone(),
            request: 99,
            density: 12,
            preferred: 13,
            css: css.clone(),
        },
        HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation: operation_rev.clone(),
            request: 11,
            density: 99,
            preferred: 13,
            css: css.clone(),
        },
        HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation: operation_rev.clone(),
            request: 11,
            density: 12,
            preferred: 99,
            css: css.clone(),
        },
        HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation: operation_rev.clone(),
            request: 11,
            density: 12,
            preferred: 13,
            css: HtmlImageSizingRevision::new(css.producer().clone(), css.configuration(), 99),
        },
        HtmlImageDimensionOperationKey::ImgDefaultSized {
            operation: HtmlImageSizingRevision::new(
                operation_rev.producer().clone(),
                99,
                operation_rev.revision(),
            ),
            request: 11,
            density: 12,
            preferred: 13,
            css: css.clone(),
        },
    ] {
        let obs = HtmlImageDimensionResourceObservation::img(
            r.clone(),
            expected.clone(),
            Some(HtmlImgRequestState::CompletelyAvailable),
            Some(HtmlImageDimensionCompleted::new(
                changed,
                measurement("300", "1"),
                measurement("150", "1"),
            )),
        );
        assert!(matches!(
            HtmlImageDimensionFacts::new(
                &c,
                c.host_facts_revision().clone(),
                HtmlImageDimensionRevision::new(Some(r.clone()), None),
                vec![expected.clone()],
                vec![obs],
                vec![],
                fact_limits()
            ),
            Err(HtmlImageDimensionFactsError::InvalidOperation(_))
        ));
    }
}

#[test]
fn context_and_terminal_failure_keep_expected_combined_keys_and_reject_stale_facts() {
    let (c, id) = fixture("img", vec![]);
    let r = resource_revision();
    let actual = HtmlImageDimensionRevision::new(Some(r.clone()), None);
    let facts = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        actual.clone(),
        vec![],
        vec![],
        vec![],
        fact_limits(),
    )
    .unwrap();
    let stale = HtmlImageDimensionRevision::new(
        Some(HtmlNaturalDimensionRevision::new(
            r.producer().clone(),
            r.configuration(),
            99,
        )),
        None,
    );
    let input = HtmlSemanticContextInput::new(vec![]).with_image_dimensions(stale, Some(facts));
    assert!(matches!(
        HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 10),
        Err(HtmlSemanticContextError::ImageDimensionsRevisionMismatch)
    ));
    let cx = HtmlSemanticContext::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlSemanticContextInput::new(vec![]).with_image_dimensions(actual.clone(), None),
        10,
    )
    .unwrap();
    let cancellation = HtmlSemanticCancellation::new();
    cancellation.cancel();
    let error = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        limits(),
        cancellation,
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap_err();
    assert_eq!(
        error.qualification().unwrap().image_dimension_revision(),
        Some(&actual)
    );
    let (other, _) = fixture("img", vec![]);
    let facts = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        actual.clone(),
        vec![],
        vec![],
        vec![],
        fact_limits(),
    )
    .unwrap();
    assert!(matches!(
        HtmlSemanticContext::new(
            &other,
            other.host_facts_revision().clone(),
            HtmlSemanticContextInput::new(vec![]).with_image_dimensions(actual, Some(facts)),
            10
        ),
        Err(HtmlSemanticContextError::ImageDimensionsCaptureMismatch)
    ));
}

#[test]
fn intrinsic_author_resource_binding_contradiction_is_rejected_without_using_intrinsic_axes_as_getter()
 {
    let (c, id) = fixture("img", vec![]);
    let rev = resource_revision();
    let expected = expectation(&id, HtmlImageDimensionRole::Img, None);
    let dims = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlImageDimensionRevision::new(Some(rev.clone()), None),
        vec![expected],
        vec![],
        vec![],
        fact_limits(),
    )
    .unwrap();
    let other_key = HtmlNaturalResourceKey::new(HtmlNaturalResourceId::fresh(), 1, 2);
    let natural = HtmlNaturalDimensions::new(
        &c,
        c.host_facts_revision().clone(),
        rev.clone(),
        vec![HtmlNaturalDimensionFact::selection(
            id.clone(),
            Some(other_key),
        )],
        vec![],
        HtmlNaturalDimensionLimits::new(10, 100000, 100000),
    )
    .unwrap();
    let input = HtmlSemanticContextInput::new(vec![])
        .with_natural_dimensions(rev, Some(natural))
        .with_image_dimensions(dims.revision().clone(), Some(dims));
    assert!(matches!(
        HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 10),
        Err(HtmlSemanticContextError::ImageDimensionsFacts(
            HtmlImageDimensionFactsError::ContradictoryResource(_)
        ))
    ));
}

#[test]
fn numerator_success_then_denominator_reservation_failure_records_cumulative_bytes_and_sticks() {
    let (c, id) = fixture("img", vec![]);
    let cx = context(
        &c,
        &id,
        Some(true),
        None,
        Some((measurement("12345", "67"), measurement("1", "1"))),
    );
    // Observe only the admitted outer record allocation: digit-zero fails before
    // any numeric buffer. Its actual layout may grow with the public fact enum.
    let baseline = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        HtmlSemanticLimits::new(100000, 1, 10, 0, 1000000, 10),
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request(&id)])
    .unwrap_err()
    .usage()
    .allocated_bytes();
    let l = HtmlSemanticLimits::new(100000, 1, 10, 100, baseline + 5, 10);
    let mut session = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(100),
        l,
        HtmlSemanticCancellation::new(),
    )
    .unwrap();
    let e = session.query(&[request(&id)]).unwrap_err();
    assert_eq!(e.usage().allocated_bytes(), baseline + 5);
    assert_eq!(
        e.kind(),
        &HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::AllocatedBytes,
            limit: baseline + 5,
            required: baseline + 7
        }
    );
    assert_eq!(e.usage().scratch_records(), 0);
    assert_eq!(session.query(&[]).unwrap_err(), e);
}

#[test]
fn unsupported_rendered_measurements_remain_typed_and_do_not_use_author_or_natural_fallbacks() {
    let (c, id) = fixture("img", vec![attr("width", "12"), attr("height", "13")]);
    let cx = context(
        &c,
        &id,
        Some(true),
        None,
        Some((
            HtmlImageDimensionMeasurement::Unsupported(
                HtmlNaturalDimensionUnsupported::NonRational,
            ),
            HtmlImageDimensionMeasurement::Unsupported(HtmlNaturalDimensionUnsupported::NonFinite),
        )),
    );
    let r = run(&c, &cx, &id);
    assert_eq!(
        result(&r).width(),
        &HtmlImageDimensionAxisResult::Unsupported(HtmlNaturalDimensionUnsupported::NonRational)
    );
    assert_eq!(
        result(&r).height(),
        &HtmlImageDimensionAxisResult::Unsupported(HtmlNaturalDimensionUnsupported::NonFinite)
    );
}

#[test]
fn box_records_reject_foreign_capture_and_host_even_when_subject_and_revision_match() {
    let (c, id) = fixture("img", vec![]);
    let (foreign, _) = fixture("img", vec![]);
    let p = participation_revision();
    for (capture, host, is_capture) in [
        (
            foreign.identity().clone(),
            c.host_facts_revision().clone(),
            true,
        ),
        (c.identity().clone(), HtmlHostFactsRevision::fresh(), false),
    ] {
        let b = HtmlImageContentBoxObservation::new(
            capture,
            host,
            id.clone(),
            p.clone(),
            measurement("1", "1"),
            measurement("2", "1"),
        );
        let e = HtmlImageDimensionFacts::new(
            &c,
            c.host_facts_revision().clone(),
            HtmlImageDimensionRevision::new(None, Some(p.clone())),
            vec![],
            vec![],
            vec![b],
            fact_limits(),
        )
        .unwrap_err();
        assert_eq!(
            e,
            if is_capture {
                HtmlImageDimensionFactsError::CaptureMismatch
            } else {
                HtmlImageDimensionFactsError::HostRevisionMismatch
            }
        );
    }
}

#[test]
fn incoming_measurement_digit_storage_is_admitted_in_addition_to_record_backing() {
    let (c, id) = fixture("img", vec![]);
    let p = participation_revision();
    let b = HtmlImageContentBoxObservation::new(
        c.identity().clone(),
        c.host_facts_revision().clone(),
        id,
        p.clone(),
        measurement(&"9".repeat(1000), "1"),
        measurement("1", "1"),
    );
    let bytes = std::mem::size_of::<HtmlImageContentBoxObservation>() + 999;
    let e = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        HtmlImageDimensionRevision::new(None, Some(p)),
        vec![],
        vec![],
        vec![b],
        HtmlImageDimensionFactsLimits::new(1, bytes, 100000),
    )
    .unwrap_err();
    assert_eq!(e, HtmlImageDimensionFactsError::ByteLimit);
}

#[test]
fn non_element_subjects_do_not_require_rendering_or_resource_facts() {
    let mut b = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 10000, 100, 100000, 100, 10, 10),
        HtmlHostFactsRevision::fresh(),
    );
    let d = b
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    b.establish_children(&d, &[]).unwrap();
    let c = b.finish().unwrap();
    let id = c.subject_id(&d).unwrap();
    let cx = empty_context(&c);
    assert!(matches!(
        run(&c, &cx, &id).results()[0].fact(),
        HtmlSemanticFact::ImageDimensions(HtmlSemanticValue::Inapplicable)
    ));
}

#[test]
fn completed_operation_key_is_retained_only_when_resource_result_is_used() {
    let (c, id) = fixture("img", vec![]);
    let cx = resource(
        &c,
        &id,
        HtmlImgRequestState::CompletelyAvailable,
        true,
        Some((measurement("3", "1"), measurement("4", "1"))),
    );
    let report = run(&c, &cx, &id);
    assert!(matches!(
        result(&report).operation(),
        Some(
            HtmlImageDimensionOperationKey::ImgPreferredDensityCorrected {
                request: 11,
                density: 12,
                preferred: 13,
                ..
            }
        )
    ));
    let (c, id) = fixture("img", vec![attr("width", "5"), attr("height", "6")]);
    let cx = resource(
        &c,
        &id,
        HtmlImgRequestState::CompletelyAvailable,
        true,
        Some((measurement("3", "1"), measurement("4", "1"))),
    );
    assert!(result(&run(&c, &cx, &id)).operation().is_none());
}

#[test]
fn known_equivalent_rendering_without_css_box_cannot_join_a_css_content_box_observation() {
    let (c, id) = fixture("img", vec![]);
    let p = participation_revision();
    let part = HtmlListParticipation::new(
        &c,
        p.clone(),
        vec![HtmlListParticipationFact::new(
            id.clone(),
            None,
            Some(true),
            Some(false),
        )],
        1,
    )
    .unwrap();
    let box_observation = HtmlImageContentBoxObservation::new(
        c.identity().clone(),
        c.host_facts_revision().clone(),
        id.clone(),
        p.clone(),
        measurement("1", "1"),
        measurement("2", "1"),
    );
    let rev = HtmlImageDimensionRevision::new(None, Some(p.clone()));
    let facts = HtmlImageDimensionFacts::new(
        &c,
        c.host_facts_revision().clone(),
        rev.clone(),
        vec![],
        vec![],
        vec![box_observation],
        fact_limits(),
    )
    .unwrap();
    let input = HtmlSemanticContextInput::new(vec![])
        .with_list_participation(p, Some(part))
        .with_image_dimensions(rev, Some(facts));
    assert!(matches!(
        HtmlSemanticContext::new(&c, c.host_facts_revision().clone(), input, 10),
        Err(HtmlSemanticContextError::ImageDimensionsFacts(
            HtmlImageDimensionFactsError::ContradictoryBox(_)
        ))
    ));
}

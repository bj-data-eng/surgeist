use crate::semantics::support::{
    Charge, ExpectedUsage, assert_exact_and_one_short_caps, capture_limits, context,
    result_record_bytes, semantic_limits_without_nested_output as limits, session, validated_work,
};
use surgeist_html::*;
use surgeist_html_semantics::HtmlHostFactsRevision;
use surgeist_html_semantics::{
    HtmlCapture, HtmlCaptureBuilder, HtmlCaptureBuilderSubject, HtmlCaptureRegistryInput,
    HtmlCaptureSubjectId, HtmlCapturedChildIds, HtmlCapturedIsValue, HtmlCapturedShadowProperties,
    HtmlScriptingStatus, HtmlSemanticCancellation, HtmlSemanticDependencies, HtmlSemanticFact,
    HtmlSemanticFailureKind, HtmlSemanticLimits, HtmlSemanticMissing, HtmlSemanticQuery,
    HtmlSemanticReport, HtmlSemanticRequest, HtmlSemanticResource, HtmlSemanticResult,
    HtmlSemanticSession, HtmlSemanticUnsupported, HtmlSemanticValue, HtmlUsemapAuthorDefinition,
    HtmlUsemapMatch, HtmlUsemapProcessingResult, HtmlUsemapReferenceResult,
    HtmlUsemapSelectedTargetDiagnostic,
};

fn capture_builder() -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut builder = HtmlCaptureBuilder::new(capture_limits(), HtmlHostFactsRevision::fresh());
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (builder, document)
}
fn element(
    builder: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    name: QualifiedName,
    attrs: OrderedAttributes,
) -> HtmlCaptureBuilderSubject {
    builder
        .add_element(
            document,
            name,
            attrs,
            HtmlCapturedIsValue::Unavailable,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}
fn attributes(attrs: &[(&str, &str)]) -> OrderedAttributes {
    OrderedAttributes::new(attrs.iter().map(|(name, value)| {
        Attribute::new(QualifiedName::new(Namespace::none(), None, *name), *value)
    }))
    .unwrap()
}
fn html(
    builder: &mut HtmlCaptureBuilder,
    document: &HtmlCaptureBuilderSubject,
    name: &str,
    attrs: &[(&str, &str)],
) -> HtmlCaptureBuilderSubject {
    element(
        builder,
        document,
        QualifiedName::new(Namespace::html(), None, name),
        attributes(attrs),
    )
}
fn children(
    builder: &mut HtmlCaptureBuilder,
    parent: &HtmlCaptureBuilderSubject,
    children: &[HtmlCaptureBuilderSubject],
) {
    builder.establish_children(parent, children).unwrap();
}
fn request(subject: &HtmlCaptureSubjectId) -> HtmlSemanticRequest {
    HtmlSemanticRequest::new(subject.clone(), HtmlSemanticQuery::UsemapReference)
}
fn query(capture: &HtmlCapture, image: &HtmlCaptureSubjectId, raw: usize) -> HtmlSemanticReport {
    session(
        capture,
        &context(capture),
        limits(),
        raw,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(image)])
    .unwrap()
}
fn value(report: &HtmlSemanticReport) -> &HtmlUsemapReferenceResult {
    match report.results()[0].fact() {
        HtmlSemanticFact::UsemapReference(HtmlSemanticValue::Known(value)) => value,
        other => panic!("expected known reference, got {other:?}"),
    }
}
fn missing(report: &HtmlSemanticReport) -> &HtmlSemanticMissing {
    match report.results()[0].fact() {
        HtmlSemanticFact::UsemapReference(HtmlSemanticValue::Missing(missing)) => missing,
        other => panic!("expected missing evidence, got {other:?}"),
    }
}
fn no_tree(raw: Option<&str>) -> (HtmlCapture, HtmlCaptureSubjectId) {
    let (mut builder, document) = capture_builder();
    let image = html(
        &mut builder,
        &document,
        "img",
        &raw.map(|value| vec![("usemap", value)]).unwrap_or_default(),
    );
    let capture = builder.finish().unwrap();
    let image = capture.subject_id(&image).unwrap();
    (capture, image)
}
fn connected(
    raw: &str,
    maps: &[&[(&str, &str)]],
) -> (HtmlCapture, HtmlCaptureSubjectId, Vec<HtmlCaptureSubjectId>) {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", raw)]);
    children(&mut builder, &image, &[]);
    let mut ordered = vec![image.clone()];
    let mut map_handles = vec![];
    for attrs in maps {
        let map = html(&mut builder, &document, "map", attrs);
        children(&mut builder, &map, &[]);
        ordered.push(map.clone());
        map_handles.push(map);
    }
    children(&mut builder, &document, &ordered);
    let capture = builder.finish().unwrap();
    let image = capture.subject_id(&image).unwrap();
    let maps = map_handles
        .iter()
        .map(|map| capture.subject_id(map).unwrap())
        .collect();
    (capture, image, maps)
}
fn assert_map(
    value: &HtmlUsemapReferenceResult,
    map: &HtmlCaptureSubjectId,
    matched_by: HtmlUsemapMatch,
) {
    assert_eq!(
        value.processing(),
        &HtmlUsemapProcessingResult::Map {
            subject: map.clone(),
            matched_by
        }
    );
}

#[test]
fn missing_and_source_null_guards_do_not_require_unknown_tree_relations() {
    for (raw, author, processing) in [
        (
            None,
            HtmlUsemapAuthorDefinition::Missing,
            HtmlUsemapProcessingResult::Missing,
        ),
        (
            Some(""),
            HtmlUsemapAuthorDefinition::NoLeadingHash,
            HtmlUsemapProcessingResult::NoHash,
        ),
        (
            Some("abc"),
            HtmlUsemapAuthorDefinition::NoLeadingHash,
            HtmlUsemapProcessingResult::NoHash,
        ),
        (
            Some("abc#"),
            HtmlUsemapAuthorDefinition::NoLeadingHash,
            HtmlUsemapProcessingResult::EmptyTail,
        ),
    ] {
        let (capture, image) = no_tree(raw);
        let report = query(&capture, &image, 4);
        assert_eq!(value(&report).author_definition(), &author);
        assert_eq!(value(&report).processing(), &processing);
        assert_eq!(value(&report).selected_target_diagnostic(), None);
        assert_eq!(report.usage().peak_scratch_records(), 0);
        assert_eq!(report.usage().nested_output_records(), 0);
        assert_eq!(
            value(&report).raw_attribute().map(Attribute::value),
            raw.map(DocumentString::from).as_ref()
        );
    }
}

#[test]
fn existential_name_witness_and_first_id_processing_target_keep_id_only_diagnostic() {
    let (capture, image, maps) = connected("#x", &[&[("id", "x")], &[("name", "x")]]);
    let report = query(&capture, &image, 2);
    assert_map(value(&report), &maps[0], HtmlUsemapMatch::Id);
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(maps[1].clone())
    );
    assert_eq!(
        value(&report).selected_target_diagnostic(),
        Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch)
    );
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn name_id_and_both_match_kinds_and_complete_absence_are_source_exact() {
    for (attrs, matched_by, witness, diagnostic) in [
        (vec![("name", "x")], HtmlUsemapMatch::Name, true, None),
        (
            vec![("id", "x")],
            HtmlUsemapMatch::Id,
            false,
            Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch),
        ),
        (
            vec![("id", "x"), ("name", "x")],
            HtmlUsemapMatch::IdAndName,
            true,
            None,
        ),
        (
            vec![("id", "x"), ("name", "other")],
            HtmlUsemapMatch::Id,
            false,
            Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch),
        ),
    ] {
        let (capture, image, maps) = connected("#x", &[&attrs]);
        let report = query(&capture, &image, 5);
        assert_map(value(&report), &maps[0], matched_by);
        assert_eq!(
            value(&report).author_definition(),
            &if witness {
                HtmlUsemapAuthorDefinition::NameMatch(maps[0].clone())
            } else {
                HtmlUsemapAuthorDefinition::NoNameMatch
            }
        );
        assert_eq!(value(&report).selected_target_diagnostic(), diagnostic);
    }
    let (capture, image, _) = connected("#x", &[&[("name", "X")], &[]]);
    let report = query(&capture, &image, 2);
    assert_eq!(
        value(&report).processing(),
        &HtmlUsemapProcessingResult::NoMatchingMap
    );
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NoNameMatch
    );
    assert_eq!(value(&report).selected_target_diagnostic(), None);
}

#[test]
fn malformed_local_form_hashes_and_spaces_preserve_processing_and_name_evidence() {
    for (raw, name, leading) in [
        ("prefix#x", "x", false),
        ("##x", "#x", true),
        ("#x#y", "x#y", true),
        ("#a b", "a b", true),
        (" # A%20", " A%20", false),
    ] {
        let (capture, image, maps) = connected(raw, &[&[("name", name)]]);
        let report = query(&capture, &image, 20);
        assert_map(value(&report), &maps[0], HtmlUsemapMatch::Name);
        assert_eq!(
            value(&report).author_definition(),
            &if leading {
                HtmlUsemapAuthorDefinition::NameMatch(maps[0].clone())
            } else {
                HtmlUsemapAuthorDefinition::NoLeadingHash
            }
        );
        assert_eq!(value(&report).selected_target_diagnostic(), None);
    }
}

#[test]
fn lone_hash_can_have_empty_name_witness_while_processing_is_null() {
    let (capture, image, maps) = connected("#", &[&[], &[("name", "")]]);
    let report = query(&capture, &image, 1);
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(maps[1].clone())
    );
    assert_eq!(
        value(&report).processing(),
        &HtmlUsemapProcessingResult::EmptyTail
    );
    assert_eq!(value(&report).selected_target_diagnostic(), None);
    let (capture, image, _) = connected("#", &[&[]]);
    assert_eq!(
        value(&query(&capture, &image, 1)).author_definition(),
        &HtmlUsemapAuthorDefinition::NoNameMatch
    );
}

#[test]
fn non_map_and_foreign_id_matches_do_not_block_exact_html_map_candidates() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let div = html(&mut builder, &document, "div", &[("id", "x")]);
    let foreign = element(
        &mut builder,
        &document,
        QualifiedName::new(Namespace::svg(), None, "map"),
        attributes(&[("id", "x")]),
    );
    let upper = html(&mut builder, &document, "MAP", &[("name", "x")]);
    let wrong_attrs = element(
        &mut builder,
        &document,
        KnownHtmlElement::Map.qualified_name(),
        OrderedAttributes::new([
            Attribute::new(QualifiedName::new(Namespace::svg(), None, "name"), "x"),
            Attribute::new(QualifiedName::new(Namespace::none(), None, "NAME"), "x"),
        ])
        .unwrap(),
    );
    let selected = element(
        &mut builder,
        &document,
        QualifiedName::new(Namespace::html(), Some("h".into()), "map"),
        attributes(&[("name", "x")]),
    );
    for node in [&image, &div, &foreign, &upper, &wrong_attrs] {
        children(&mut builder, node, &[]);
    }
    children(
        &mut builder,
        &document,
        &[
            image.clone(),
            div,
            foreign,
            upper,
            wrong_attrs,
            selected.clone(),
        ],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_map(
        value(&report),
        &capture.subject_id(&selected).unwrap(),
        HtmlUsemapMatch::Name,
    );
}

#[test]
fn exact_utf16_names_include_null_supplementary_and_isolated_surrogate_units() {
    let (mut builder, document) = capture_builder();
    let name = DocumentString::from_utf16(vec![0xd800, 0, 0xd83d, 0xde00, 35]);
    let raw = Attribute::new(
        QualifiedName::new(Namespace::none(), Some("p".into()), "usemap"),
        DocumentString::from_utf16(vec![35, 0xd800, 0, 0xd83d, 0xde00, 35]),
    );
    let image = element(
        &mut builder,
        &document,
        KnownHtmlElement::Img.qualified_name(),
        OrderedAttributes::new([raw.clone()]).unwrap(),
    );
    let map = element(
        &mut builder,
        &document,
        KnownHtmlElement::Map.qualified_name(),
        OrderedAttributes::new([Attribute::new(
            KnownHtmlAttribute::Name.qualified_name(),
            name,
        )])
        .unwrap(),
    );
    children(&mut builder, &document, &[map.clone(), image.clone()]);
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 6);
    assert_map(
        value(&report),
        &capture.subject_id(&map).unwrap(),
        HtmlUsemapMatch::Name,
    );
    assert_eq!(value(&report).raw_attribute(), Some(&raw));
    assert_eq!(
        value(&report).reference().processing_name_units(),
        Some(&[0xd800, 0, 0xd83d, 0xde00, 35][..])
    );
}

#[test]
fn an_element_root_map_is_included_and_other_document_roots_are_not_searched() {
    let (mut builder, document) = capture_builder();
    let map = html(&mut builder, &document, "map", &[("name", "x")]);
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let other = html(&mut builder, &document, "map", &[("name", "x")]);
    children(&mut builder, &map, std::slice::from_ref(&image));
    builder.establish_root(&map).unwrap();
    builder.establish_root(&other).unwrap();
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_map(
        value(&report),
        &capture.subject_id(&map).unwrap(),
        HtmlUsemapMatch::Name,
    );
    // A separate root whose own children are unavailable cannot create Missing.
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(capture.subject_id(&map).unwrap())
    );
}

#[test]
fn shadow_and_template_associations_are_not_ordinary_lookup_edges() {
    for shadow in [false, true] {
        let (mut builder, document) = capture_builder();
        let host = html(
            &mut builder,
            &document,
            if shadow { "div" } else { "template" },
            &[],
        );
        let fragment = if shadow {
            builder
                .add_shadow_root(
                    &document,
                    &host,
                    HtmlCapturedShadowProperties::new(
                        HtmlShadowRootProperties::new(
                            HtmlShadowRootMode::Closed,
                            HtmlSlotAssignment::Named,
                            false,
                            false,
                            false,
                        ),
                        false,
                        false,
                        false,
                    ),
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap()
        } else {
            let fragment = builder.add_fragment(&document).unwrap();
            builder
                .establish_template_contents(&host, &fragment)
                .unwrap();
            fragment
        };
        let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
        let inside = html(&mut builder, &document, "map", &[("name", "x")]);
        let outside = html(&mut builder, &document, "map", &[("name", "x")]);
        children(&mut builder, &fragment, &[inside.clone(), image.clone()]);
        children(&mut builder, &host, &[]);
        children(&mut builder, &document, &[host, outside]);
        let capture = builder.finish().unwrap();
        let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
        assert_map(
            value(&report),
            &capture.subject_id(&inside).unwrap(),
            HtmlUsemapMatch::Name,
        );
    }
}

#[test]
fn required_unavailable_relations_remain_missing_and_release_current_scratch() {
    let (capture, image) = no_tree(Some("#x"));
    assert_eq!(
        missing(&query(&capture, &image, 2)),
        &HtmlSemanticMissing::Parent(image)
    );
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let id_map = html(&mut builder, &document, "map", &[("id", "x")]);
    children(&mut builder, &document, &[id_map.clone(), image.clone()]);
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_eq!(
        missing(&report),
        &HtmlSemanticMissing::OrderedChildren(capture.subject_id(&id_map).unwrap())
    );
    assert_eq!(report.usage().scratch_records(), 0);
    assert!(report.usage().peak_scratch_records() > 0);
}

#[test]
fn a_combined_first_match_does_not_read_its_children_or_later_unavailable_branches() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let map = html(
        &mut builder,
        &document,
        "map",
        &[("id", "x"), ("name", "x")],
    );
    let later = html(
        &mut builder,
        &document,
        "div",
        &[("id", "far-too-long-unconsulted")],
    );
    children(
        &mut builder,
        &document,
        &[map.clone(), image.clone(), later],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_map(
        value(&report),
        &capture.subject_id(&map).unwrap(),
        HtmlUsemapMatch::IdAndName,
    );
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn raw_and_actually_inspected_candidate_values_cross_the_whole_value_cap() {
    let (capture, image) = no_tree(Some("#long"));
    let cx = context(&capture);
    let mut owner = session(&capture, &cx, limits(), 4, HtmlSemanticCancellation::new());
    let error = owner.query(&[request(&image)]).unwrap_err();
    assert!(
        matches!(error.kind(), HtmlSemanticFailureKind::Attribute(error) if error.required() == 5 && error.capacity() == 4)
    );
    assert_eq!(owner.query(&[]).unwrap_err(), error);
    let (capture, image, _) = connected("prefix#x", &[&[("id", "x"), ("name", "too-long!")]]);
    let error = session(
        &capture,
        &context(&capture),
        limits(),
        8,
        HtmlSemanticCancellation::new(),
    )
    .query(&[request(&image)])
    .unwrap_err();
    assert!(
        matches!(error.kind(), HtmlSemanticFailureKind::Attribute(error) if error.required() == 9)
    );
}

#[test]
fn wrong_actual_roles_are_typed_without_inspecting_unrelated_attributes() {
    let (mut builder, document) = capture_builder();
    let div = html(
        &mut builder,
        &document,
        "div",
        &[("usemap", "oversize-unrelated")],
    );
    let foreign = element(
        &mut builder,
        &document,
        QualifiedName::new(Namespace::svg(), None, "img"),
        attributes(&[("usemap", "oversize-unrelated")]),
    );
    let text = builder.add_text(&document, "#x".into()).unwrap();
    let image = element(
        &mut builder,
        &document,
        KnownHtmlElement::Img.qualified_name(),
        OrderedAttributes::new([Attribute::new(
            QualifiedName::new(Namespace::svg(), None, "usemap"),
            "#x",
        )])
        .unwrap(),
    );
    let capture = builder.finish().unwrap();
    for subject in [div, text, document] {
        let report = query(&capture, &capture.subject_id(&subject).unwrap(), 0);
        assert!(matches!(
            report.results()[0].fact(),
            HtmlSemanticFact::UsemapReference(HtmlSemanticValue::Inapplicable)
        ));
    }
    let report = query(&capture, &capture.subject_id(&foreign).unwrap(), 0);
    assert!(matches!(
        report.results()[0].fact(),
        HtmlSemanticFact::UsemapReference(HtmlSemanticValue::Unsupported(
            HtmlSemanticUnsupported::ForeignNamespace
        ))
    ));
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 0);
    assert_eq!(value(&report).raw_attribute(), None);
    assert_eq!(
        value(&report).processing(),
        &HtmlUsemapProcessingResult::Missing
    );
}

#[test]
fn exact_work_admits_completion_and_one_short_rejects_without_partial_batch() {
    let (capture, image) = no_tree(Some("abcd"));
    let cx = context(&capture);
    // The hash-name scanner and classification steps are the HTML owner's
    // private layout and not among the semantic query work reference (docs/reference/html-operations.md), so work is
    // required to be charged but is not pinned.
    // The exact cap on every resource reproduces the baseline report; the one
    // result record is the only allocation and nothing is nested or scratch.
    let report = assert_exact_and_one_short_caps(
        "usemap reference",
        &capture,
        &cx,
        4,
        HtmlSemanticLimits::new(1000000, 1, 0, 0, 1000000, 0),
        &[request(&image)],
        ExpectedUsage {
            output_records: Charge::Exactly(1),
            allocated_bytes: Charge::Exactly(result_record_bytes()),
            ..ExpectedUsage::none()
        },
    );
    assert_eq!(report.usage().scratch_records(), 0);
    // The helper has proved this total is the least admitting work cap.
    let work = report.usage().work();
    let mut short = session(
        &capture,
        &cx,
        HtmlSemanticLimits::new(work - 1, 1, 0, 0, 1000000, 0),
        4,
        HtmlSemanticCancellation::new(),
    );
    let failure = short.query(&[request(&image)]).unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::Work, required, .. } if *required == work)
    );
    assert_eq!(short.query(&[]).unwrap_err(), failure);
    // The distinction between stops before and after the HTML owner's completion
    // step depends on its private layout, so it is not asserted. Instead every
    // cap below the validated total, which includes every point before the
    // result is ready, must stop with exactly Work, limit == cap and
    // required > cap, without ever exceeding the cap.
    for cap in 0..work {
        let mut cut = session(
            &capture,
            &cx,
            HtmlSemanticLimits::new(cap, 1, 0, 0, 1000000, 0),
            4,
            HtmlSemanticCancellation::new(),
        );
        let failure = cut.query(&[request(&image)]).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::Work, limit, required } if *limit == cap && *required > cap),
            "cap {cap}: {failure:?}"
        );
        assert!(failure.usage().work() <= cap);
        assert_eq!(cut.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn physical_output_and_scratch_boundaries_are_admitted_before_retention_and_cleaned_after_stop() {
    let (capture, image) = no_tree(Some("abcd"));
    let cx = context(&capture);
    for (limits, resource) in [
        (
            HtmlSemanticLimits::new(100000, 0, 10, 0, 1000000, 0),
            HtmlSemanticResource::OutputRecords,
        ),
        (
            HtmlSemanticLimits::new(
                100000,
                1,
                10,
                0,
                std::mem::size_of::<HtmlSemanticResult>() - 1,
                0,
            ),
            HtmlSemanticResource::AllocatedBytes,
        ),
    ] {
        let mut session = session(&capture, &cx, limits, 4, HtmlSemanticCancellation::new());
        let failure = session.query(&[request(&image)]).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: actual, .. } if *actual == resource)
        );
        assert_eq!(failure.usage().work(), 0);
        assert_eq!(failure.usage().allocated_bytes(), 0);
        assert_eq!(session.query(&[]).unwrap_err(), failure);
    }
    let (capture, image, _) = connected("#x", &[&[("name", "x")]]);
    let cx = context(&capture);
    let mut zero = session(
        &capture,
        &cx,
        HtmlSemanticLimits::new(100000, 1, 0, 0, 1000000, 0),
        2,
        HtmlSemanticCancellation::new(),
    );
    let failure = zero.query(&[request(&image)]).unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded {
            resource: HtmlSemanticResource::ScratchRecords,
            ..
        }
    ));
    assert_eq!(zero.usage().scratch_records(), 0);
    // The id and name walk reserves scratch before it finishes. Rather than a
    // private offset, cut the validated total at every cap below it: each stop
    // is exactly Work with limit == cap and required > cap, scratch is always
    // released, the stop is sticky, and some cut falls after scratch was first
    // reserved.
    let total = validated_work(
        "usemap id and name walk",
        &capture,
        &cx,
        2,
        HtmlSemanticLimits::new(1000000, 1, 10, 0, 1000000, 0),
        &[request(&image)],
    );
    let mut reached_scratch = false;
    for cap in 0..total {
        let mut late = session(
            &capture,
            &cx,
            HtmlSemanticLimits::new(cap, 1, 10, 0, 1000000, 0),
            2,
            HtmlSemanticCancellation::new(),
        );
        let failure = late.query(&[request(&image)]).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::Work, limit, required } if *limit == cap && *required > cap),
            "cap {cap}: {failure:?}"
        );
        reached_scratch |= late.usage().peak_scratch_records() > 0;
        assert_eq!(late.usage().scratch_records(), 0);
        assert_eq!(late.query(&[]).unwrap_err(), failure);
    }
    assert!(reached_scratch);
}

#[test]
fn batch_order_qualification_foreign_identity_and_completed_report_survive_sticky_cancellation() {
    let (capture, image, maps) = connected("#x", &[&[("name", "x")]]);
    let cx = context(&capture);
    let cancellation = HtmlSemanticCancellation::new();
    let mut owner = session(&capture, &cx, limits(), 2, cancellation.clone());
    let report = owner.query(&[request(&image), request(&image)]).unwrap();
    assert_eq!(report.results().len(), 2);
    assert_eq!(report.results()[0].request(), &request(&image));
    assert_eq!(report.results()[0].fact(), report.results()[1].fact());
    assert_map(value(&report), &maps[0], HtmlUsemapMatch::Name);
    assert_eq!(report.capture(), capture.identity());
    assert_eq!(report.context(), cx.identity());
    assert_eq!(report.host_revision(), capture.host_facts_revision());
    assert_eq!(
        report.dependencies(),
        HtmlSemanticDependencies::ConservativeRecompute
    );
    cancellation.cancel();
    let failure = owner.query(&[request(&image)]).unwrap_err();
    assert_eq!(failure.kind(), &HtmlSemanticFailureKind::Cancelled);
    assert_eq!(failure.qualification().unwrap(), report.qualification());
    assert_eq!(owner.query(&[]).unwrap_err(), failure);
    assert_map(value(&report), &maps[0], HtmlUsemapMatch::Name);
    let (other, foreign) = no_tree(None);
    let other_context = context(&other);
    assert!(
        HtmlSemanticSession::new(
            &capture,
            &other_context,
            HtmlValueInterpreter::new(2),
            limits(),
            HtmlSemanticCancellation::new()
        )
        .is_err()
    );
    let failure = session(&capture, &cx, limits(), 2, HtmlSemanticCancellation::new())
        .query(&[request(&foreign)])
        .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::Capture(_)
    ));
    assert_eq!(
        failure.qualification().unwrap().capture(),
        capture.identity()
    );
}

#[test]
fn after_first_processing_match_only_remaining_name_evidence_is_inspected() {
    let (capture, image, maps) = connected(
        "#x",
        &[
            &[("id", "x")],
            &[("name", "x"), ("id", "uninspected-too-long")],
        ],
    );
    let report = query(&capture, &image, 2);
    assert_map(value(&report), &maps[0], HtmlUsemapMatch::Id);
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(maps[1].clone())
    );
    assert_eq!(
        value(&report).selected_target_diagnostic(),
        Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch)
    );
    // Non-leading input needs no author witness. The selected id-only map can
    // succeed before reading its unavailable children or any later map.
    let (mut builder, document) = capture_builder();
    let selected = html(&mut builder, &document, "map", &[("id", "x")]);
    let image = html(&mut builder, &document, "img", &[("usemap", "prefix#x")]);
    children(&mut builder, &document, &[selected.clone(), image.clone()]);
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 8);
    assert_map(
        value(&report),
        &capture.subject_id(&selected).unwrap(),
        HtmlUsemapMatch::Id,
    );
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NoLeadingHash
    );
    assert_eq!(
        value(&report).selected_target_diagnostic(),
        Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch)
    );
}

#[test]
fn unrelated_roots_and_associated_contents_do_not_supply_a_missing_name_witness() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let other = html(&mut builder, &document, "map", &[("name", "x")]);
    children(&mut builder, &image, &[]);
    builder.establish_root(&image).unwrap();
    builder.establish_root(&other).unwrap();
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_eq!(
        value(&report).processing(),
        &HtmlUsemapProcessingResult::NoMatchingMap
    );
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NoNameMatch
    );
    for shadow in [false, true] {
        let (mut builder, document) = capture_builder();
        let host = html(
            &mut builder,
            &document,
            if shadow { "div" } else { "template" },
            &[],
        );
        let fragment = if shadow {
            builder
                .add_shadow_root(
                    &document,
                    &host,
                    HtmlCapturedShadowProperties::new(
                        HtmlShadowRootProperties::new(
                            HtmlShadowRootMode::Open,
                            HtmlSlotAssignment::Named,
                            false,
                            false,
                            false,
                        ),
                        false,
                        false,
                        false,
                    ),
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap()
        } else {
            let fragment = builder.add_fragment(&document).unwrap();
            builder
                .establish_template_contents(&host, &fragment)
                .unwrap();
            fragment
        };
        let inside = html(&mut builder, &document, "map", &[("name", "x")]);
        let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
        children(&mut builder, &fragment, &[inside]);
        children(&mut builder, &host, &[]);
        children(&mut builder, &image, &[]);
        children(&mut builder, &document, &[host, image.clone()]);
        let capture = builder.finish().unwrap();
        let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
        assert_eq!(
            value(&report).processing(),
            &HtmlUsemapProcessingResult::NoMatchingMap
        );
        assert_eq!(
            value(&report).author_definition(),
            &HtmlUsemapAuthorDefinition::NoNameMatch
        );
    }
}

#[test]
fn later_unconsulted_branches_do_not_increase_first_combined_match_work() {
    let mut observed = vec![];
    for tail in ["", "a very long unrelated id value after the first match"] {
        let (mut builder, document) = capture_builder();
        let map = html(&mut builder, &document, "map", &[("name", "x")]);
        let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
        let later = html(&mut builder, &document, "div", &[("id", tail)]);
        children(
            &mut builder,
            &document,
            &[map.clone(), image.clone(), later],
        );
        let capture = builder.finish().unwrap();
        let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
        assert_map(
            value(&report),
            &capture.subject_id(&map).unwrap(),
            HtmlUsemapMatch::Name,
        );
        observed.push(report.usage().work());
    }
    assert_eq!(observed[0], observed[1]);
}

#[test]
fn late_raw_admission_failure_discards_the_whole_batch_and_preserves_qualification() {
    let (mut builder, document) = capture_builder();
    let first = html(&mut builder, &document, "img", &[("usemap", "abcd")]);
    let second = html(&mut builder, &document, "img", &[("usemap", "oversize")]);
    let capture = builder.finish().unwrap();
    let first = capture.subject_id(&first).unwrap();
    let second = capture.subject_id(&second).unwrap();
    let cx = context(&capture);
    let mut owner = session(&capture, &cx, limits(), 4, HtmlSemanticCancellation::new());
    let failure = owner
        .query(&[request(&first), request(&second)])
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlSemanticFailureKind::Attribute(error) if error.required() == 8)
    );
    // The first request completed before the second failed, so the failure has
    // charged more than that first request alone.
    let first_work = validated_work(
        "usemap first image request",
        &capture,
        &cx,
        4,
        limits(),
        &[request(&first)],
    );
    assert!(failure.usage().work() > first_work);
    assert_eq!(
        failure.qualification().unwrap().capture(),
        capture.identity()
    );
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(owner.query(&[]).unwrap_err(), failure);
}

#[test]
fn a_later_name_witness_resolves_an_unknown_subtree_after_the_first_processing_match() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let selected = html(&mut builder, &document, "map", &[("id", "x")]);
    let gap = html(&mut builder, &document, "div", &[]);
    let witness = html(&mut builder, &document, "map", &[("name", "x")]);
    children(&mut builder, &image, &[]);
    children(&mut builder, &selected, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), selected.clone(), gap, witness.clone()],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_map(
        value(&report),
        &capture.subject_id(&selected).unwrap(),
        HtmlUsemapMatch::Id,
    );
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(capture.subject_id(&witness).unwrap())
    );
    assert_eq!(
        value(&report).selected_target_diagnostic(),
        Some(HtmlUsemapSelectedTargetDiagnostic::IdOnlyMatch)
    );
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn unknown_children_of_the_selected_map_do_not_block_a_later_name_witness() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let selected = html(&mut builder, &document, "map", &[("id", "x")]);
    let witness = html(&mut builder, &document, "map", &[("name", "x")]);
    children(&mut builder, &image, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), selected.clone(), witness.clone()],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_map(
        value(&report),
        &capture.subject_id(&selected).unwrap(),
        HtmlUsemapMatch::Id,
    );
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(capture.subject_id(&witness).unwrap())
    );
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn an_unknown_subtree_before_processing_selection_cannot_be_skipped() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let gap = html(&mut builder, &document, "div", &[]);
    let later = html(&mut builder, &document, "map", &[("name", "x")]);
    children(&mut builder, &image, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), gap.clone(), later],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_eq!(
        missing(&report),
        &HtmlSemanticMissing::OrderedChildren(capture.subject_id(&gap).unwrap())
    );
    assert_eq!(report.usage().scratch_records(), 0);
}

#[test]
fn unresolved_name_evidence_retains_the_first_of_multiple_unknown_subtrees() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let selected = html(&mut builder, &document, "map", &[("id", "x")]);
    let first = html(&mut builder, &document, "div", &[]);
    let second = html(&mut builder, &document, "div", &[]);
    let different = html(&mut builder, &document, "map", &[("name", "y")]);
    children(&mut builder, &image, &[]);
    children(&mut builder, &selected, &[]);
    children(&mut builder, &different, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), selected, first.clone(), second, different],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 2);
    assert_eq!(
        missing(&report),
        &HtmlSemanticMissing::OrderedChildren(capture.subject_id(&first).unwrap())
    );
    assert_eq!(report.usage().scratch_records(), 0);
    assert!(report.usage().peak_scratch_records() > 0);
}

#[test]
fn a_later_empty_name_witness_resolves_a_gap_without_creating_a_processing_target() {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#")]);
    let gap = html(&mut builder, &document, "div", &[]);
    let witness = html(&mut builder, &document, "map", &[("name", "")]);
    children(&mut builder, &image, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), gap, witness.clone()],
    );
    let capture = builder.finish().unwrap();
    let report = query(&capture, &capture.subject_id(&image).unwrap(), 1);
    assert_eq!(
        value(&report).author_definition(),
        &HtmlUsemapAuthorDefinition::NameMatch(capture.subject_id(&witness).unwrap())
    );
    assert_eq!(
        value(&report).processing(),
        &HtmlUsemapProcessingResult::EmptyTail
    );
    assert_eq!(value(&report).selected_target_diagnostic(), None);
    assert_eq!(report.usage().scratch_records(), 0);
}

fn fixed_processing_with_gap(
    name: &str,
) -> (HtmlCapture, HtmlCaptureSubjectId, HtmlCaptureSubjectId) {
    let (mut builder, document) = capture_builder();
    let image = html(&mut builder, &document, "img", &[("usemap", "#x")]);
    let selected = html(&mut builder, &document, "map", &[("id", "x")]);
    let gap = html(&mut builder, &document, "div", &[]);
    let witness = html(&mut builder, &document, "map", &[("name", name)]);
    children(&mut builder, &image, &[]);
    children(&mut builder, &selected, &[]);
    children(
        &mut builder,
        &document,
        &[image.clone(), selected.clone(), gap, witness],
    );
    let capture = builder.finish().unwrap();
    let image = capture.subject_id(&image).unwrap();
    let selected = capture.subject_id(&selected).unwrap();
    (capture, image, selected)
}

#[test]
fn resumed_lookup_stops_at_every_work_cut_and_releases_physical_scratch() {
    let (capture, image, selected) = fixed_processing_with_gap("x");
    let cx = context(&capture);
    // The id lookup, deferred gap and witness proof use the HTML owner's private
    // step layout, so work is required to be charged but is not pinned. The exact
    // cap must reproduce the report, whose scratch is the walk's frames.
    let report = assert_exact_and_one_short_caps(
        "usemap resumed lookup",
        &capture,
        &cx,
        2,
        HtmlSemanticLimits::new(1000000, 1, 10, 0, 1000000, 0),
        &[request(&image)],
        ExpectedUsage {
            output_records: Charge::Exactly(1),
            peak_scratch_records: Charge::AtLeast(2),
            allocated_bytes: Charge::AtLeast(
                result_record_bytes() + 2 * std::mem::size_of::<HtmlCapturedChildIds<'_>>(),
            ),
            ..ExpectedUsage::none()
        },
    );
    assert_map(value(&report), &selected, HtmlUsemapMatch::Id);
    assert_eq!(report.usage().scratch_records(), 0);
    assert_eq!(
        report.usage().allocated_bytes(),
        report.usage().output_records() * std::mem::size_of::<HtmlSemanticResult>()
            + report.usage().peak_scratch_records()
                * std::mem::size_of::<HtmlCapturedChildIds<'_>>()
    );
    // Every cut below the validated total, including the cuts that fall before
    // the deferred gap's first retention and before its resume, stops with
    // exactly Work, limit == cap and required > cap, never exceeds the cap,
    // releases scratch and is sticky.
    for cap in 0..report.usage().work() {
        let mut owner = session(
            &capture,
            &cx,
            HtmlSemanticLimits::new(cap, 1, 10, 0, 1000000, 0),
            2,
            HtmlSemanticCancellation::new(),
        );
        let failure = owner.query(&[request(&image)]).unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlSemanticFailureKind::LimitExceeded { resource: HtmlSemanticResource::Work, limit, required } if *limit == cap && *required > cap),
            "cap {cap}: {failure:?}"
        );
        assert!(failure.usage().work() <= cap);
        assert_eq!(owner.usage().scratch_records(), 0);
        assert_eq!(owner.query(&[]).unwrap_err(), failure);
    }
}

#[test]
fn a_terminal_raw_failure_after_a_deferred_gap_discards_the_batch_and_remains_sticky() {
    let (capture, image, _) = fixed_processing_with_gap("oversize");
    let cx = context(&capture);
    let mut owner = session(&capture, &cx, limits(), 2, HtmlSemanticCancellation::new());
    // The first subject is an actual successful inapplicable query. No prefix
    // can be returned when the second query fails after deferring its gap.
    let document = capture.node(&image).unwrap().node_document();
    let failure = owner
        .query(&[request(document.subject()), request(&image)])
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlSemanticFailureKind::Attribute(error)
        if error.required() == 8 && error.capacity() == 2)
    );
    // The failure comes after the successful first request and the gap deferral,
    // so it has charged more than that first request alone.
    let first = validated_work(
        "usemap document request",
        &capture,
        &cx,
        2,
        limits(),
        &[request(document.subject())],
    );
    assert!(failure.usage().work() > first);
    assert_eq!(owner.usage().scratch_records(), 0);
    assert_eq!(
        failure.qualification().unwrap().capture(),
        capture.identity()
    );
    assert_eq!(failure.qualification().unwrap().context(), cx.identity());
    assert_eq!(owner.query(&[]).unwrap_err(), failure);
}

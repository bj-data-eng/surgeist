//! Real request-proof frames are input custody, including older proof owners.
//! Native AutoPhrase/history/flow capabilities, without fabricated frame data.
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");
const PHRASE: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/phrase-ahem/PhraseAhem.ttf");

fn fixture() -> (System, PreparedText, ScopeAlignmentLimits) {
    // The retained BudouX0.9.3 Japanese reference qualifies 日本へ as one
    // phrase; its genuine internal normal cuts are1/2 after relaxation.
    let body = Style {
        font: Font::new().family("Surgeist Phrase Ahem").family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        locale: Some("ja".into()),
        word_break: WordBreak::AutoPhrase,
        overflow_wrap: OverflowWrap::Anywhere,
        ..Style::default()
    };
    let mut source = Source::identified("日本へ", Id::from_u64(1578080), 51);
    source.span(Range::new(0, 3), body.clone());
    let mut input = super::capture::captured_input(1);
    let caps = super::capture::generous();
    input.source = source.clone();
    input.origins = vec![ParagraphOrigin {
        text_node: None,
        source,
        local: Range::new(0, 3),
        aggregate: Range::new(0, 3),
        kind: ParagraphOriginKind::Authored,
    }];
    input.scope_alignment_limits = Some(caps);
    input.paragraphs[0].paragraph_strut = ParagraphStrut::AllocatedContributions;
    let font_reference = ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Hani").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    };
    let SourceProcessingScope::Block {
        base_style,
        font_reference: parent_reference,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!()
    };
    *base_style = body.clone();
    *parent_reference = Some(font_reference);
    let SourceProcessingScope::Inline {
        style,
        alignment_boundary,
        font_reference: child_reference,
        ..
    } = &mut input.scopes[1]
    else {
        unreachable!()
    };
    *style = Some(body);
    *alignment_boundary = Some(ScopeAlignmentBoundary::NonInitial);
    *child_reference = Some(font_reference);
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let analysis = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analysis = crate::test_wrapping::bind_missing(&analysis, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system.register_fonts(PHRASE.to_vec()).unwrap();
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    (system, prepared, caps)
}
fn full(caps: ScopeAlignmentLimits) -> ScopeAlignmentAllowance {
    ScopeAlignmentAllowance::new(caps.records(), caps.payload_bytes())
}
fn request(
    prepared: &PreparedText,
    start: PreparedBoundary,
    end: PreparedBoundary,
    termination: PreparedTermination,
) -> LineSelectionRequest {
    let mut value = super::super::request(prepared);
    value.start = start;
    value.end = end;
    let first = value.start.view().participant_offset();
    let last = value.end.view().participant_offset();
    value.participants = prepared
        .participants()
        .skip(first)
        .take(last - first)
        .collect();
    value.alternative = match &termination {
        PreparedTermination::Conditional(cut) => cut.alternative(),
        _ => None,
    };
    value.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    value.context.end = match &termination {
        PreparedTermination::Ordinary(offer)
            if offer.view().kind() == PreparedBreakKind::ParagraphEnd =>
        {
            LineEnd::Paragraph
        }
        _ => LineEnd::Soft,
    };
    value.context.available_inline = Some(1.0);
    value.context.alignment = Alignment::Start;
    value.context.spacing = SpacingPolicy::None;
    value.termination = termination;
    value
}
fn finalize(selected: &SelectedLine) -> FinalizedLine {
    assert_eq!(selected.scopes().len(), 1);
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                scope: selected.scopes().next().unwrap(),
                displacement: BaselineShift::try_new(0.0).unwrap(),
            }]),
        })
        .unwrap()
}
fn ordinary(prepared: &PreparedText) -> FinalizedLine {
    let start = prepared.boundary_views().next().unwrap().handle();
    let ceiling = prepared
        .opportunity_views()
        .find(|v| v.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let frame = finalize(
        &prepared
            .select(request(
                prepared,
                start,
                ceiling.view().boundary(),
                PreparedTermination::Ordinary(ceiling),
            ))
            .unwrap(),
    );
    assert!(frame.measurements().fit().advance() > 1.0);
    frame
}
fn search(prepared: &PreparedText, frame: &FinalizedLine) -> PreparedConditionalSearch {
    let original = frame.selected_line();
    let PreparedTermination::Ordinary(ceiling) = &original.request().termination else {
        unreachable!()
    };
    prepared
        .source_segments()
        .next()
        .unwrap()
        .conditional_search(
            &original.request().start,
            None,
            ceiling,
            ConditionalPurpose::Overflow,
            &[],
            crate::ConditionalLineContext::Finalized(frame),
        )
        .unwrap()
}
fn phrase_request(
    prepared: &PreparedText,
    query: &PreparedConditionalSearch,
) -> LineSelectionRequest {
    let portions = query.phrase_portions();
    assert_eq!(portions.len(), 1);
    let portion = portions[0].clone();
    request(
        prepared,
        portion.start(),
        portion.end(),
        PreparedTermination::PhraseMeasurement(portion),
    )
}
fn census(
    input: ScopeAlignmentCensusInput<'_>,
    related: &[ScopeAlignmentProduct<'_>],
    caps: ScopeAlignmentLimits,
) -> ScopeAlignmentCensus {
    scope_alignment_census(input, ScopeAlignmentProducts::Mixed(related), full(caps)).unwrap()
}
fn no_external(result: &ScopeAlignmentCensus) {
    assert_eq!(
        result.external_retention(),
        ScopeAlignmentRetention::default()
    );
    assert_eq!(result.usage().font_queries(), 0);
}
fn owned_after_prepared(
    prepared: &PreparedText,
    frames: &[ScopeAlignmentProduct<'_>],
    caps: ScopeAlignmentLimits,
) {
    let result = census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(prepared)),
        frames,
        caps,
    );
    assert!(result.external_retention().records() > 0);
    assert!(result.external_retention().payload_bytes() > 0);
}
fn published(prepared: &PreparedText, frame: &FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: frame.clone(),
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}

#[test]
fn phrase_measurement_input_and_selected_product_own_their_actual_ordinary_query_frame() {
    let (system, prepared, caps) = fixture();
    let frame = ordinary(&prepared);
    let query = search(&prepared, &frame);
    let request = phrase_request(&prepared, &query);
    // Native admission precedes ownership assertions: this is a real measured
    // phrase request, not merely a resource receipt for an arbitrary request.
    let selected = prepared.select(request.clone()).unwrap();
    let measured = finalize(&selected);
    assert!(measured.measurements().fit().advance() > 1.0);
    let stats = system.stats();
    let roots = [ScopeAlignmentProduct::Finalized(&frame)];
    let operation = census(
        ScopeAlignmentCensusInput::Selection {
            prepared: &prepared,
            request: &request,
        },
        &roots,
        caps,
    );
    no_external(&operation);
    owned_after_prepared(&prepared, &roots, caps);
    let product = census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Selected(&selected)),
        &roots,
        caps,
    );
    no_external(&product);
    assert_eq!(
        product.input_retention(),
        selected.scope_alignment_retention().unwrap()
    );
    assert_eq!(system.stats(), stats);
}

fn history(
    prepared: &PreparedText,
    frame: &FinalizedLine,
) -> (FinalizedLine, Vec<FinalizedLine>, LineSelectionRequest) {
    let query = search(prepared, frame);
    let measured = finalize(&prepared.select(phrase_request(prepared, &query)).unwrap());
    let normal = query
        .begin(&[PhraseOverflowDecision::Overflows(measured.clone())])
        .unwrap();
    assert_eq!(normal.stage(), ConditionalStage::Normal);
    assert_eq!(
        normal
            .opportunities()
            .iter()
            .map(|cut| cut.boundary().view().source_before().get())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let frames = normal
        .opportunities()
        .iter()
        .map(|cut| {
            let selected = prepared
                .select(request(
                    prepared,
                    cut.start(),
                    cut.boundary(),
                    PreparedTermination::Conditional(cut.clone()),
                ))
                .unwrap();
            let frame = finalize(&selected);
            assert!(frame.measurements().fit().advance() > 1.0);
            frame
        })
        .collect::<Vec<_>>();
    let dispositions = frames
        .iter()
        .cloned()
        .map(ConditionalDisposition::Exhausted)
        .collect::<Vec<_>>();
    let hyphens = normal.advance(&dispositions).unwrap().unwrap();
    assert_eq!(hyphens.stage(), ConditionalStage::Hyphens);
    assert!(hyphens.opportunities().is_empty());
    let arbitrary = hyphens.advance(&[]).unwrap().unwrap();
    assert_eq!(arbitrary.stage(), ConditionalStage::Arbitrary);
    let cut = arbitrary
        .opportunities()
        .first()
        .expect("genuine arbitrary cut after the discharged native tiers");
    let request = request(
        prepared,
        cut.start(),
        cut.boundary(),
        PreparedTermination::Conditional(cut.clone()),
    );
    (measured, frames, request)
}

#[test]
fn selected_history_owns_direct_and_transitive_older_frames_once_even_through_aliases() {
    let (system, prepared, caps) = fixture();
    let frame = ordinary(&prepared);
    let (measured, normal, request) = history(&prepared, &frame);
    let selected = prepared.select(request).unwrap();
    let line = finalize(&selected);
    let frame_clone = frame.clone();
    let measured_clone = measured.clone();
    let selected_clone = selected.clone();
    let stats = system.stats();
    let mut roots = vec![
        ScopeAlignmentProduct::Finalized(&frame),
        ScopeAlignmentProduct::Finalized(&measured),
    ];
    roots.extend(normal.iter().map(ScopeAlignmentProduct::Finalized));
    roots.extend([
        ScopeAlignmentProduct::Finalized(&frame_clone),
        ScopeAlignmentProduct::Finalized(&measured_clone),
        ScopeAlignmentProduct::Selected(&selected_clone),
    ]);
    let product = census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Selected(&selected)),
        &roots,
        caps,
    );
    no_external(&product);
    assert_eq!(
        product.input_retention(),
        selected.scope_alignment_retention().unwrap()
    );
    let finalized = census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&line)),
        &roots,
        caps,
    );
    no_external(&finalized);
    assert_eq!(
        finalized.input_retention(),
        line.scope_alignment_retention().unwrap()
    );
    owned_after_prepared(&prepared, &roots[..2], caps);
    assert_eq!(system.stats(), stats);
}

#[test]
fn selection_allowance_must_cover_the_typed_history_input_and_preserves_prior_publication() {
    let (system, prepared, caps) = fixture();
    let frame = ordinary(&prepared);
    let old = published(&prepared, &frame);
    let (_measured, _normal, request) = history(&prepared, &frame);
    let accepted = prepared.select(request.clone()).unwrap();
    let accepted_line = finalize(&accepted);
    let stats = system.stats();
    let input = census(
        ScopeAlignmentCensusInput::Selection {
            prepared: &prepared,
            request: &request,
        },
        &[],
        caps,
    )
    .input_retention();
    let bare = prepared.scope_alignment_retention().unwrap();
    assert!(input.records() > bare.records());
    assert!(input.payload_bytes() > bare.payload_bytes());
    for (allowance, expected_kind, limit) in [
        (
            ScopeAlignmentAllowance::new(input.records() - 1, caps.payload_bytes()),
            ScopeAlignmentLimitKind::Records,
            input.records() - 1,
        ),
        (
            ScopeAlignmentAllowance::new(caps.records(), input.payload_bytes() - 1),
            ScopeAlignmentLimitKind::PayloadBytes,
            input.payload_bytes() - 1,
        ),
    ] {
        let error = prepared
            .select_with_scope_allowance(request.clone(), allowance)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
        let Some(ErrorDetail::ScopeAlignment {
            reason:
                ScopeAlignmentError::Limit {
                    kind,
                    required,
                    limit: actual_limit,
                },
        }) = error.detail()
        else {
            panic!("typed input must refuse its actual deficient resource: {error:?}")
        };
        assert_eq!(*kind, expected_kind);
        assert_eq!(*actual_limit, limit);
        assert!(*required > limit);
    }
    let retry = prepared.select(request).unwrap();
    let retry = finalize(&retry);
    assert_eq!(retry.scope_allocations(), accepted_line.scope_allocations());
    assert_eq!(old.source().text().code_units(), [0x65e5, 0x672c, 0x3078]);
    assert_eq!(frame.prepared_text(), prepared);
    assert_eq!(system.stats(), stats);
}

#[test]
fn actual_phrase_flow_target_owns_its_query_frame_before_native_observation() {
    let (system, prepared, caps) = fixture();
    let frame = ordinary(&prepared);
    let query = search(&prepared, &frame);
    let phrase = phrase_request(&prepared, &query);
    let flow = prepared
        .measurement_flow(
            &phrase.segment,
            &phrase.start,
            None,
            &phrase.termination,
            None,
        )
        .unwrap();
    let segments = flow.segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].role(), FlowSegmentRole::Target);
    let request = FlowObservationRequest {
        context: phrase.context.clone(),
        atomics: vec![],
        controls: vec![],
        line_strut: None,
        line_box_struts: vec![],
        boundary_context: vec![],
    };
    let observation = prepared.observe_flow(&segments[0], &request).unwrap();
    assert!(observation.has_retained_flow_participation());
    flow.validate_observations(&[observation]).unwrap();
    let stats = system.stats();
    let roots = [ScopeAlignmentProduct::Finalized(&frame)];
    let operation = census(
        ScopeAlignmentCensusInput::Flow {
            prepared: &prepared,
            segment: &segments[0],
            request: &request,
        },
        &roots,
        caps,
    );
    no_external(&operation);
    owned_after_prepared(&prepared, &roots, caps);
    assert!(
        operation.input_retention().records()
            > prepared.scope_alignment_retention().unwrap().records()
    );
    assert!(
        operation.input_retention().payload_bytes()
            > prepared
                .scope_alignment_retention()
                .unwrap()
                .payload_bytes()
    );
    assert_eq!(system.stats(), stats);
}

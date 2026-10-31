//! Bounded private lookup behavior over authentic public-produced source data.
//!
//! Fresh ledgers isolate each borrowed lookup; they do not describe the complete
//! preparation/selection transaction or its retained input storage. Exact work
//! follows the documented event/candidate visits, not a shaping algorithm cost.
//! These tests establish fallible admission/results, not an execution trace.
use super::{control_frame_owner, selected_frame};
use crate::processing::contextual::alignment::ScopeAlignmentBudget;
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

struct Fixture {
    system: System,
    prepared: PreparedText,
    selected: SelectedLine,
    prior: FinalizedLine,
    layout: Layout,
    domain: CapturedScope,
    outer: CapturedScope,
    inner: CapturedScope,
    control: PreparedControl,
}

fn fixture(classified: bool) -> Fixture {
    use SourceProcessingEvent as E;
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    let reference = ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    };
    let mut source = Source::identified("XX", Id::from_u64(1578076), 59);
    source.span(Range::new(0, 2), style.clone());
    let mut scopes = vec![SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: style.clone(),
        used_direction: ParagraphBaseDirection::LeftToRight,
        font_reference: Some(reference),
    }];
    // Genuine unrelated empty boxes remain in the captured inventory. They are
    // not injected owner entries or substitute selected scope rows.
    scopes.extend((0..2).map(|_| SourceProcessingScope::Inline {
        horizontal_inner_direction: None,
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
        alignment_boundary: None,
        font_reference: None,
    }));
    for boundary in [
        ScopeAlignmentBoundary::Initial,
        ScopeAlignmentBoundary::NonInitial,
    ] {
        scopes.push(SourceProcessingScope::Inline {
            horizontal_inner_direction: None,
            trim: WhiteSpaceTrim::default(),
            style: Some(style.clone()),
            boundaries: None,
            alignment_boundary: classified.then_some(boundary),
            font_reference: classified.then_some(reference),
        });
    }
    let input = ContextualSourceInput {
        joint_flows: Vec::new(),
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        scope_alignment_limits: Some(ScopeAlignmentLimits::new(4096, 4096, 1_000_000, 16_777_216)),
        origins: (0..2)
            .map(|i| ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(i, i + 1),
                aggregate: Range::new(i, i + 1),
                kind: ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Continuous,
        }],
        controls: vec![ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::OptionalBreak { wrap: Wrap::Word },
        }],
        replacements: vec![],
        computed_line_boundaries: vec![],
        boundary_policies: vec![ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(0),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        }],
        scopes,
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        }],
        events: vec![
            E::EnterScope(0),
            E::EnterParagraph(0),
            E::EnterScope(1),
            E::ExitScope(1),
            E::EnterScope(2),
            E::ExitScope(2),
            E::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            E::Connection(0),
            E::EnterScope(3),
            E::EnterScope(4),
            E::Control {
                control: 0,
                affinity: Affinity::After,
            },
            E::Origin {
                origin: 1,
                affinity: Affinity::After,
            },
            E::ExitScope(4),
            E::ExitScope(3),
            E::ExitParagraph(0),
            E::ExitScope(0),
        ],
    };
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let domain = captured.captured_scope(0).unwrap();
    let outer = captured.captured_scope(3).unwrap();
    let inner = captured.captured_scope(4).unwrap();
    assert_eq!(inner.parent(), Some(outer.clone()));
    assert_eq!(outer.parent(), Some(domain.clone()));
    let analyzed = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.control_views().count(), 1);
    let control = prepared.control_views().next().unwrap().handle();
    assert_eq!(
        control.view().kind(),
        &ParagraphControlKind::OptionalBreak { wrap: Wrap::Word }
    );
    assert_eq!(control.captured_control().unwrap().scope(), inner);
    let selected = prepared.select(request(&prepared)).unwrap();
    assert_eq!(selected.request().controls.len(), 1);
    assert_eq!(selected.request().controls[0].occurrence, control);
    assert!(selected.request().controls[0].strut.is_none());
    let scopes = if classified {
        assert_eq!(
            selected
                .scopes()
                .map(|scope| scope.captured_scope())
                .collect::<Vec<_>>(),
            [outer.clone(), inner.clone()]
        );
        ScopeAllocations::Complete(
            selected
                .scopes()
                .map(|scope| ScopeAllocation {
                    scope,
                    displacement: BaselineShift::try_new(0.0).unwrap(),
                })
                .collect(),
        )
    } else {
        assert_eq!(selected.scopes().len(), 0);
        ScopeAllocations::None
    };
    let prior = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes,
        })
        .unwrap();
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: prior.clone(),
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    Fixture {
        system,
        prepared,
        selected,
        prior,
        layout,
        domain,
        outer,
        inner,
        control,
    }
}

fn request(prepared: &PreparedText) -> LineSelectionRequest {
    let termination = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    LineSelectionRequest {
        segment: crate::test_wrapping::only_source_segment(prepared),
        start: prepared.boundary_views().next().unwrap().handle(),
        end: termination.view().boundary(),
        participants: prepared.participants().collect(),
        termination: PreparedTermination::Ordinary(termination),
        alternative: None,
        incoming: None,
        context: LineContext {
            alignment_insets: LineAlignmentInsets::ZERO,
            formatted_line: FormattedLineContext::default(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            line_box_direction: LineBoxDirection::LeftToRight,
            orientation: LineOrientation::Horizontal,
            start: LineStart::Paragraph,
            end: LineEnd::Paragraph,
            available_inline: None,
            indent: 0.0,
            alignment: Alignment::Start,
            tabs: TabContext::NoTabs,
            spacing: SpacingPolicy::None,
        },
        atomics: vec![],
        controls: prepared
            .control_views()
            .map(|view| ControlAllocation {
                occurrence: view.handle(),
                strut: None,
                boundary: None,
            })
            .collect(),
        boundary_context: vec![],
        line_strut: None,
        line_box_struts: vec![],
    }
}

fn budget(work: usize) -> ScopeAlignmentBudget {
    ScopeAlignmentBudget::new(ScopeAlignmentLimits::new(0, 0, work, 0), 0, 0).unwrap()
}
fn usage(budget: &ScopeAlignmentBudget, work: usize) {
    let usage = budget.usage();
    assert_eq!(usage.structural_work(), work);
    assert_eq!(usage.font_queries(), 0);
    assert_eq!(usage.records(), 0);
    assert_eq!(usage.payload_bytes(), 0);
    assert_eq!(usage.peak_payload_bytes(), 0);
}
fn refusal(error: Error, required: usize, limit: usize) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected exact lookup resource refusal, got {error:?}")
    };
    assert_eq!(
        *reason,
        ScopeAlignmentError::Limit {
            kind: ScopeAlignmentLimitKind::StructuralWork,
            required,
            limit,
        }
    );
}
fn publication(fixture: &Fixture) {
    assert_eq!(fixture.prior.selected_line(), fixture.selected);
    assert_eq!(
        (
            fixture.prior.feedback().advance,
            fixture.prior.feedback().baseline,
            fixture.prior.feedback().line_height
        ),
        (20.0, 8.0, 10.0)
    );
    assert_eq!(fixture.layout.source().text().code_units(), [88, 88]);
    assert_eq!(
        fixture
            .layout
            .lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 2)]
    );
    assert_eq!(fixture.layout.bounds(), Rect::new(0.0, 92.0, 20.0, 10.0));
    assert_eq!(
        fixture
            .prior
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), AHEM);
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [(58, 10.0), (58, 10.0)]
    );
}

#[test]
fn unnominated_real_control_lookup_requires_all_scope_visits_before_enclosure_fallback() {
    let fixture = fixture(true);
    publication(&fixture);
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let selected_usage = fixture.selected.scope_alignment_usage();
    let topology = fixture
        .prepared
        .inner
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .unwrap();
    // Derive the semantic occurrence from the actual public-issued control
    // handle. The raw capture's control0 is never substituted into this lookup.
    let occurrence = fixture.prepared.inner.tables.controls[fixture.control.index].occurrence;
    // All five captured scopes have no endpoint nomination. One event visit and
    // five failed candidate checks precede the actual enclosing inner scope.
    for limit in [0, 1, 5] {
        let mut ledger = budget(limit);
        refusal(
            control_frame_owner(&topology.capture, topology, occurrence, &mut ledger).unwrap_err(),
            limit + 1,
            limit,
        );
        usage(&ledger, limit);
    }
    let mut ledger = budget(6);
    assert_eq!(
        control_frame_owner(&topology.capture, topology, occurrence, &mut ledger).unwrap(),
        Some(fixture.inner.capture_ordinal())
    );
    usage(&ledger, 6);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
    assert_eq!(fixture.selected.scope_alignment_usage(), selected_usage);
    publication(&fixture);
}

#[test]
fn nearest_selected_frame_lookup_requires_each_authentic_selected_candidate() {
    let fixture = fixture(true);
    publication(&fixture);
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let selected_usage = fixture.selected.scope_alignment_usage();
    let rows = &fixture.selected.inner.scopes.rows;
    // Public selection issued the actual parent-before-child rows [outer,inner].
    // Both contain the inner owner; the nearest result is the inner row1.
    for limit in [0, 1] {
        let mut ledger = budget(limit);
        refusal(
            selected_frame(rows, fixture.inner.capture_ordinal(), &mut ledger).unwrap_err(),
            limit + 1,
            limit,
        );
        usage(&ledger, limit);
    }
    for (owner, expected) in [
        (&fixture.inner, Some(1)),
        (&fixture.outer, Some(0)),
        (&fixture.domain, None),
    ] {
        let mut ledger = budget(2);
        assert_eq!(
            selected_frame(rows, owner.capture_ordinal(), &mut ledger).unwrap(),
            expected
        );
        usage(&ledger, 2);
    }
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
    assert_eq!(fixture.selected.scope_alignment_usage(), selected_usage);
    publication(&fixture);
}

#[test]
fn authentic_selection_without_classified_frames_needs_no_frame_lookup_work() {
    let fixture = fixture(false);
    publication(&fixture);
    assert!(fixture.prepared.scope_alignment_retention().is_some());
    assert_eq!(
        fixture
            .prepared
            .scope_alignment_usage()
            .unwrap()
            .font_queries(),
        0
    );
    let stats = fixture.system.stats();
    let prepared_usage = fixture.prepared.scope_alignment_usage();
    let selected_usage = fixture.selected.scope_alignment_usage();
    let mut ledger = budget(0);
    assert_eq!(
        selected_frame(
            &fixture.selected.inner.scopes.rows,
            fixture.inner.capture_ordinal(),
            &mut ledger
        )
        .unwrap(),
        None
    );
    usage(&ledger, 0);
    assert_eq!(fixture.system.stats(), stats);
    assert_eq!(fixture.prepared.scope_alignment_usage(), prepared_usage);
    assert_eq!(fixture.selected.scope_alignment_usage(), selected_usage);
    publication(&fixture);
}

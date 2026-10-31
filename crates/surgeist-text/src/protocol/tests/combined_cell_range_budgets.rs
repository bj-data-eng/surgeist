//! Immutable operation limits over genuine mixed, multi-receipt placements.
use super::*;
use crate::{ScopeAlignmentError, ScopeAlignmentLimitKind, SourceRangeGeometry};

const GENEROUS: ScopeAlignmentLimits =
    ScopeAlignmentLimits::new(1_000_000, 1_000_000, 1_000_000_000, 268_435_456);
const CELLS: usize = 4;

#[derive(Clone, Copy, Debug)]
enum Payload {
    Aa,
    Tab,
}
impl Payload {
    fn text(self) -> &'static str {
        match self {
            Self::Aa => "AA",
            Self::Tab => "\t",
        }
    }
    fn expected(self) -> Vec<Rect> {
        match self {
            // Raw Ahem AA10/10, H8/-2, negative width actions, x-scale.5.
            Self::Aa => vec![
                Rect::new(95.0, 20.0, 5.0, 10.0),
                Rect::new(100.0, 20.0, 5.0, 10.0),
            ],
            // Actual absolute T10, first strict stop10, no provider or baseline.
            Self::Tab => vec![Rect::new(95.0, 20.0, 10.0, 10.0)],
        }
    }
}

// The existing single-Cell factory has the right metric/source premises, but
// cannot issue multiple actual receipt roots. This retains its real complete
// capture pattern, adding ordinary B/SPACE origins between distinct Inline boxes.
fn captured_paragraph(payload: Payload, limits: ScopeAlignmentLimits) -> Captured {
    let text = [payload.text(); CELLS].join("B ");
    let root_style = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        text_combine_upright: TextCombineUpright::None,
        tab_size: TabSize::length(10.0).unwrap(),
        wrap: Wrap::Word,
        ..Style::default()
    };
    let body = Style {
        text_combine_upright: TextCombineUpright::All,
        ..root_style.clone()
    };
    let mut source = Source::identified(text.as_str(), Id::from_u64(784_994_701), 1);
    let mut origins = Vec::new();
    let mut scopes = vec![SourceProcessingScope::Block {
        topology: SourceProcessingTopology::Complete,
        trim: WhiteSpaceTrim::default(),
        base_style: root_style,
        used_direction: ParagraphBaseDirection::LeftToRight,
        writing_axis: Some(SourceWritingAxis::Vertical),
        horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
        font_reference: Some(reference(LineOrientation::Vertical)),
    }];
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    let mut at = 0;
    for cell in 0..CELLS {
        if cell > 0 {
            let ordinal = origins.len();
            events.push(SourceProcessingEvent::Connection(ordinal - 1));
            origins.push(ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(at, at + 2),
                aggregate: Range::new(at, at + 2),
                kind: ParagraphOriginKind::Authored,
            });
            events.push(SourceProcessingEvent::Origin {
                origin: ordinal,
                affinity: Affinity::After,
            });
            at += 2;
        }
        let scope = cell + 1;
        let end = at + payload.text().encode_utf16().count();
        source.span(Range::new(at, end), body.clone());
        scopes.push(SourceProcessingScope::Inline {
            trim: WhiteSpaceTrim::default(),
            style: Some(body.clone()),
            boundaries: None,
            alignment_boundary: None,
            font_reference: Some(reference(LineOrientation::Vertical)),
            horizontal_inner_direction: Some(ParagraphBaseDirection::LeftToRight),
        });
        let ordinal = origins.len();
        if ordinal > 0 {
            events.push(SourceProcessingEvent::Connection(ordinal - 1));
        }
        events.push(SourceProcessingEvent::EnterScope(scope));
        origins.push(ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(at, end),
            aggregate: Range::new(at, end),
            kind: ParagraphOriginKind::Authored,
        });
        events.push(SourceProcessingEvent::Origin {
            origin: ordinal,
            affinity: Affinity::After,
        });
        events.push(SourceProcessingEvent::ExitScope(scope));
        at = end;
    }
    events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    // Every resource is this same final Source value, including all actual spans.
    for origin in &mut origins {
        origin.source = source.clone();
    }
    let connections = (0..origins.len() - 1)
        .map(|before| SourceProcessingConnection {
            before_origin: before,
            after_origin: before + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect::<Vec<_>>();
    let boundary_policies = (0..connections.len())
        .map(|connection| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(connection),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
    let processed = Arc::new(
        process_contextual_source(ContextualSourceInput {
            source,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            envelope: SourceProcessingEnvelope::Complete,
            origins,
            connections,
            boundary_policies,
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(limits),
            scopes,
            paragraphs: vec![SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            }],
            events,
        })
        .unwrap(),
    );
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::Mixed,
    )
    .unwrap();
    assert_eq!(domain.outputs().len(), 1);
    let analyzed = crate::test_wrapping::bind_missing(domain.outputs().next().unwrap(), Wrap::Word);
    assert_eq!(analyzed.combined_cells().len(), CELLS);
    for (cell, scope) in analyzed.combined_cells().zip(1..=CELLS) {
        assert_eq!(cell.scope(), processed.captured_scope(scope).unwrap());
        assert_eq!(
            cell.presentation_code_points().collect::<Vec<_>>(),
            payload.text().chars().map(u32::from).collect::<Vec<_>>()
        );
    }
    Captured {
        processed,
        analyzed,
    }
}

struct Fixture {
    capture: Captured,
    prepared: PreparedText,
    selected: Vec<SelectedLine>,
    originals: Vec<FinalizedLine>,
    replayed: Vec<FinalizedLine>,
    layout: Layout,
}
fn build(payload: Payload, limits: ScopeAlignmentLimits, bytes: &[u8]) -> Fixture {
    let capture = captured_paragraph(payload, limits);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(&capture.analyzed, options(LineOrientation::Vertical))
        .unwrap();
    assert_eq!(prepared.cell_views().len(), CELLS);
    for cell in prepared.cell_views() {
        assert_eq!(cell.extent(), 10.0);
        assert_eq!(cell.has_provider_input(), matches!(payload, Payload::Aa));
    }
    let segment = prepared.source_segments().next().unwrap();
    let offers = segment
        .opportunity_views()
        .filter(|offer| {
            matches!(
                offer.kind(),
                PreparedBreakKind::Allowed | PreparedBreakKind::ParagraphEnd
            )
        })
        .filter(|offer| {
            let end = offer.boundary();
            let at = end.view().source_after().get();
            offer.kind() == PreparedBreakKind::ParagraphEnd
                || at.checked_sub(1).is_some_and(|before| {
                    capture.processed.source().text().code_units()[before] == 0x20
                })
        })
        .map(|offer| offer.handle())
        .collect::<Vec<_>>();
    // Select the actual SPACE opportunities among all admitted break sites;
    // the caller need not select the extra sites beside a composition.
    // These are genuine SPACE opportunities and the issued segment end, not
    // fabricated paragraph ends or cuts inside either composition.
    assert_eq!(offers.len(), CELLS);
    let participants = prepared.participants().collect::<Vec<_>>();
    let mut start = segment.start();
    let mut selected = Vec::new();
    let mut originals = Vec::new();
    let mut replayed = Vec::new();
    for (index, offer) in offers.into_iter().enumerate() {
        let end = offer.view().boundary();
        let mut input = request(&prepared, TabContext::NoTabs);
        input.start = start.clone();
        input.end = end.clone();
        input.participants = participants
            [start.view().participant_offset()..end.view().participant_offset()]
            .to_vec();
        input.termination = PreparedTermination::Ordinary(offer);
        input.context.start = if index == 0 {
            LineStart::Paragraph
        } else {
            LineStart::Continuation
        };
        input.context.end = if index + 1 == CELLS {
            LineEnd::Paragraph
        } else {
            LineEnd::Soft
        };
        let selection = prepared.select(input).unwrap();
        let line = selection
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::None,
            })
            .unwrap();
        let replay = line.with_spacing(SpacingAllocations::None).unwrap();
        assert_eq!(replay.selected_line(), selection);
        assert_eq!(replay.feedback(), line.feedback());
        assert_eq!(replay.cell_views().len(), 1);
        for run in replay.run_views().filter(|run| run.cell().is_some()) {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.font_size(), 10.0);
            let metrics = run
                .effective_instance()
                .unwrap()
                .metrics(run.face().catalogue_identity())
                .unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
            assert_eq!(
                run.glyphs()
                    .map(|g| (g.id(), g.advance()))
                    .collect::<Vec<_>>(),
                [(35, 10.0), (35, 10.0)]
            );
        }
        if matches!(payload, Payload::Tab) {
            assert!(
                replay
                    .run_views()
                    .filter(|run| run.cell().is_some())
                    .all(|run| run.glyphs().next().is_none())
            );
        }
        selected.push(selection);
        originals.push(line);
        replayed.push(replay);
        start = end;
    }
    assert_eq!(start, segment.end());
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: replayed
            .iter()
            .enumerate()
            .map(|(index, line)| {
                PlacementEntry::Formatted(LinePlacement {
                    line: line.clone(),
                    anchor: LineAnchor {
                        inline_origin: 20.0,
                        baseline: 100.0 + 40.0 * index as f64,
                    },
                    orientation: LineOrientation::Vertical,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.lines().len(), CELLS);
    drop(system);
    Fixture {
        capture,
        prepared,
        selected,
        originals,
        replayed,
        layout,
    }
}
fn origin(fixture: &Fixture, index: usize) -> PreparedOrigin {
    let position = fixture.capture.processed.captured_origin(index).unwrap();
    fixture
        .prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&position))
        .unwrap()
        .handle()
}
fn rects(result: &SourceRangeFragments<'_>) -> Vec<Rect> {
    result
        .units()
        .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
        .collect()
}
fn first(fixture: &Fixture, payload: Payload) -> SourceRangeFragments<'_> {
    let original = origin(fixture, 0);
    fixture
        .layout
        .source_range_fragments(&original, Range::new(0, payload.text().len()))
        .unwrap()
}
fn values(usage: ScopeAlignmentUsage, kind: ScopeAlignmentLimitKind) -> usize {
    match kind {
        ScopeAlignmentLimitKind::StructuralWork => usage.structural_work(),
        ScopeAlignmentLimitKind::Records => usage.records(),
        ScopeAlignmentLimitKind::PayloadBytes => usage.peak_payload_bytes(),
        ScopeAlignmentLimitKind::FontQueries => panic!("Range never resolves fonts"),
    }
}
fn setup_max(fixture: &Fixture, kind: ScopeAlignmentLimitKind) -> usize {
    let mut observations = vec![
        fixture
            .capture
            .processed
            .captured_scope(0)
            .unwrap()
            .scope_alignment_usage()
            .unwrap(),
        fixture.prepared.scope_alignment_usage().unwrap(),
    ];
    observations.extend(
        fixture
            .selected
            .iter()
            .map(|row| row.scope_alignment_usage().unwrap()),
    );
    observations.extend(
        fixture
            .originals
            .iter()
            .chain(&fixture.replayed)
            .map(|row| row.scope_alignment_usage().unwrap()),
    );
    observations
        .into_iter()
        .map(|usage| values(usage, kind))
        .max()
        .unwrap()
}
fn with_limit(kind: ScopeAlignmentLimitKind, cap: usize) -> ScopeAlignmentLimits {
    ScopeAlignmentLimits::new(
        if kind == ScopeAlignmentLimitKind::Records {
            cap
        } else {
            GENEROUS.records()
        },
        GENEROUS.font_queries(),
        if kind == ScopeAlignmentLimitKind::StructuralWork {
            cap
        } else {
            GENEROUS.structural_work()
        },
        if kind == ScopeAlignmentLimitKind::PayloadBytes {
            cap
        } else {
            GENEROUS.payload_bytes()
        },
    )
}
fn refusal(error: Error, kind: ScopeAlignmentLimitKind, cap: usize) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment {
        reason:
            ScopeAlignmentError::Limit {
                kind: actual,
                required,
                limit,
            },
    }) = error.detail()
    else {
        panic!("Range must own a real cap refusal, not missing metrics/UnsupportedCell: {error:?}");
    };
    assert_eq!(*actual, kind);
    assert_eq!(*limit, cap);
    assert!(*required > cap);
}

#[test]
fn active_range_caps_are_source_immutable_and_fail_after_complete_real_setup() {
    let bytes = qualified_font(); // existing direct Shaper/font qualification, no fake font.
    for payload in [Payload::Aa, Payload::Tab] {
        let adequate = build(payload, GENEROUS, &bytes);
        let successful = first(&adequate, payload);
        assert_eq!(rects(&successful), payload.expected());
        let usage = successful
            .scope_alignment_usage()
            .expect("actual Cell query activates accounting");
        assert_eq!(usage.font_queries(), 0);
        assert!(usage.payload_bytes() > 0);
        assert!(usage.peak_payload_bytes() >= usage.payload_bytes());
        for kind in [
            ScopeAlignmentLimitKind::StructuralWork,
            ScopeAlignmentLimitKind::Records,
            ScopeAlignmentLimitKind::PayloadBytes,
        ] {
            let cap = values(usage, kind).checked_sub(1).unwrap();
            // Observable phase receipts select a cap, not a mirrored buffer formula.
            // If no genuine phase headroom exists, this fixture is insufficient;
            // failure here cannot be reported as successful Range-limit evidence.
            assert!(
                cap >= setup_max(&adequate, kind),
                "{payload:?}/{kind:?} needs a genuine setup/query cap window"
            );
            let limited = build(payload, with_limit(kind, cap), &bytes);
            // Holding a real ordinary query from the SAME mixed placement proves
            // absence of feature demand, and gives a prior result to preserve.
            let ordinary = origin(&limited, 1);
            let ordinary_range = Range::new(
                ordinary.view().local_range().start,
                ordinary.view().local_range().start + 1,
            );
            let prior = limited
                .layout
                .source_range_fragments(&ordinary, ordinary_range)
                .unwrap();
            assert!(prior.scope_alignment_usage().is_none());
            assert!(
                prior
                    .units()
                    .flat_map(|unit| unit.pieces())
                    .all(|piece| matches!(piece, SourceRangeGeometry::Line(_)))
            );
            let saved = rects(&prior);
            assert!(!saved.is_empty());
            let clusters = limited.layout.clusters();
            let bounds = limited.layout.bounds();
            let advances = limited
                .replayed
                .iter()
                .map(|row| row.feedback().advance)
                .collect::<Vec<_>>();
            let original = origin(&limited, 0);
            refusal(
                limited
                    .layout
                    .source_range_fragments(&original, Range::new(0, payload.text().len()))
                    .unwrap_err(),
                kind,
                cap,
            );
            assert_eq!(limited.layout.bounds(), bounds);
            assert_eq!(limited.layout.clusters(), clusters);
            assert_eq!(
                limited
                    .replayed
                    .iter()
                    .map(|row| row.feedback().advance)
                    .collect::<Vec<_>>(),
                advances
            );
            assert_eq!(rects(&prior), saved);
            assert_eq!(prior.origin(), &ordinary);
            assert_eq!(prior.local_range(), ordinary_range);
            // Adequate immutable equivalent and original successful observation
            // remain usable; recovery never mutates the refused capture's caps.
            assert_eq!(rects(&successful), payload.expected());
            assert_eq!(rects(&first(&adequate, payload)), payload.expected());
        }
    }
}

#[test]
fn whole_mixed_placed_input_and_none_replay_preserve_cell_range_geometry() {
    let bytes = qualified_font();
    for payload in [Payload::Aa, Payload::Tab] {
        let fixture = build(payload, GENEROUS, &bytes);
        let result = first(&fixture, payload);
        assert_eq!(rects(&result), payload.expected());
        let usage = result.scope_alignment_usage().unwrap();
        // Public usage covers the complete active operation; this is not an
        // independent exact allocation census/per-buffer witness attribution.
        assert_eq!(usage.font_queries(), 0);
        assert!(usage.records() > 0);
        assert!(usage.payload_bytes() > 0);
        assert!(usage.peak_payload_bytes() >= usage.payload_bytes());
        for (old, replay) in fixture.originals.iter().zip(&fixture.replayed) {
            assert_eq!(old.feedback(), replay.feedback());
            assert_eq!(old.selected_line(), replay.selected_line());
        }
        assert_eq!(fixture.layout.lines().len(), CELLS);
        let ordinary = origin(&fixture, 1);
        let native = fixture
            .layout
            .source_range_fragments(
                &ordinary,
                Range::new(
                    ordinary.view().local_range().start,
                    ordinary.view().local_range().start + 1,
                ),
            )
            .unwrap();
        assert!(native.scope_alignment_usage().is_none());
        assert!(!rects(&native).is_empty());
    }
}

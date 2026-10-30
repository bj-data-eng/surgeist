//! Real computed source-box cuts retain their exterior lexical frame.
//! This new-API qualification uses public capture and placement producers only.
use super::capture::{captured_input, generous, reference, style};
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn exterior_input(side: SourceBoxSide) -> ContextualSourceInput {
    let mut input = captured_input(2);
    let mut source = Source::identified("AA", Id::from_u64(1_578_711), 43);
    // Scope styles provide font context; author the actual body style too.
    source.span(Range::new(0, 2), style(10.0, 10.0));
    input.source = source.clone();
    input.origins = [Range::new(0, 1), Range::new(1, 2)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    input.connections = vec![SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    let SourceProcessingScope::Block {
        base_style,
        font_reference,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!()
    };
    *base_style = style(10.0, 10.0);
    *font_reference = Some(reference());
    for (index, size) in [(1, 10.0), (2, 30.0)] {
        let SourceProcessingScope::Inline {
            style: context,
            alignment_boundary,
            font_reference,
            ..
        } = &mut input.scopes[index]
        else {
            unreachable!()
        };
        *context = Some(style(size, f64::from(size)));
        *alignment_boundary = Some(ScopeAlignmentBoundary::NonInitial);
        *font_reference = Some(reference());
    }
    // Child2 is a genuine empty inline, with no nominated boundary pair or
    // origin. Its source-box cuts are before EnterScope2 and after ExitScope2;
    // both are inside parent1. Naming child2 is not custody of child2's frame.
    use SourceProcessingEvent as E;
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::EnterScope(1),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Connection(0),
        E::EnterScope(2),
        E::ExitScope(2),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    input.computed_line_boundaries = vec![ComputedLineBoundaryInput::Scope { scope: 2, side }];
    input.scope_alignment_limits = Some(generous());
    input
}

#[test]
fn before_enter_and_after_exit_computed_occurrences_use_the_parent_frame_not_the_target_child() {
    for side in [SourceBoxSide::Before, SourceBoxSide::After] {
        let processed = Arc::new(process_contextual_source(exterior_input(side)).unwrap());
        let parent = processed.captured_scope(1).unwrap();
        let child = processed.captured_scope(2).unwrap();
        assert_eq!(child.parent(), Some(parent.clone()));
        let cause = processed.captured_computed_line_boundary(0).unwrap();
        let analyzed = AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let mut fonts = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fonts.register_fonts(AHEM.to_vec()).unwrap();
        let prepared = fonts
            .prepare(
                &crate::test_wrapping::bind_missing(&analyzed, Wrap::Word),
                PreparationOptions::default(),
            )
            .unwrap();
        let forced = prepared
            .opportunity_views()
            .find(|offer| offer.kind() == PreparedBreakKind::Forced)
            .unwrap();
        assert_eq!(
            forced
                .computed_line_boundary()
                .unwrap()
                .view()
                .causes()
                .collect::<Vec<_>>(),
            [cause]
        );
        let mut request = super::super::request(&prepared);
        request.end = forced.boundary();
        request.participants = prepared
            .participants()
            .take(request.end.view().participant_offset())
            .collect();
        assert!(
            request
                .participants
                .iter()
                .any(|part| matches!(part, PreparedParticipant::ComputedLineBoundary(_)))
        );
        request.termination = PreparedTermination::Ordinary(forced.handle());
        request.context.end = LineEnd::Forced;
        request.context.available_inline = None;
        request.context.spacing = SpacingPolicy::None;
        let coverage = PlacementCoverage::Range {
            start: request.start.clone(),
            end: request.end.clone(),
        };
        let selected = prepared.select(request).unwrap();
        assert_eq!(
            selected
                .scopes()
                .map(|scope| scope.captured_scope())
                .collect::<Vec<_>>()
                .as_slice(),
            std::slice::from_ref(&parent)
        );
        let line = selected
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::Complete(
                    selected
                        .scopes()
                        .map(|scope| ScopeAllocation {
                            scope,
                            displacement: BaselineShift::try_new(3.0).unwrap(),
                        })
                        .collect(),
                ),
            })
            .unwrap();
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.scope_allocations()[0].scope, parent);
        assert_eq!(
            line.mapping_views()
                .filter(|piece| piece.kind() == PresentationMappingKind::ComputedLineBoundary)
                .map(|piece| piece.source_range())
                .collect::<Vec<_>>(),
            [Range::new(1, 1)]
        );
        let layout = Layout::from_finalized_lines(PlacementRequest {
            prepared: prepared.clone(),
            coverage,
            entries: vec![PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 20.0,
                    baseline: 50.0,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(layout.source().id(), Some(Id::from_u64(1_578_711)));
        assert_eq!(layout.source().revision(), 43);
        assert_eq!(layout.source().text().code_units(), [65, 65]);
        assert_eq!(
            layout.run_views().next().unwrap().origin(),
            Point::new(20.0, 47.0)
        );
        // Ahem: UPEM1000, hhea800/-200, advance1000. At10px the cell
        // is8 over/2 under. Parent+3 gives y=50-3-8=39; the source-less
        // computed occurrence follows A at inline10 and uses that same frame.
        let positions = layout
            .positions()
            .filter(|position| position.source_cursor() == Cursor::new(1, Affinity::After))
            .collect::<Vec<_>>();
        assert_eq!(positions.len(), 1);
        assert_eq!(
            layout.position_geometry(&positions[0]).unwrap().rect(),
            Rect::new(30.0, 39.0, 1.0, 10.0)
        );
        let origin = prepared.origin_views().next().unwrap().handle();
        let query = layout
            .source_range_fragments(&origin, Range::new(0, 1))
            .unwrap();
        assert_eq!(
            query
                .units()
                .next()
                .unwrap()
                .pieces()
                .next()
                .unwrap()
                .rect(),
            Rect::new(20.0, 39.0, 10.0, 10.0)
        );
    }
}

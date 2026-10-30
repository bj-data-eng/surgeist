//! Focused constraints whose logical measurements must survive one CW basis.
use super::*;

#[test]
fn actual_finite_word_cut_and_intrinsic_whole_keep_distinct_selection_contributions() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(
        &mut s,
        "A B",
        Style {
            white_space: WhiteSpace::Collapse,
            ..own_style()
        },
        false,
        CW,
    );
    let cut = p
        .opportunity_views()
        .find(|o| {
            o.kind() == PreparedBreakKind::Allowed && o.boundary().view().source_after().get() == 2
        })
        .unwrap()
        .handle();
    let mut finite = selected_request(&p, start(&p), cut.clone());
    finite.context.available_inline = Some(10.0);
    let first = p
        .select(finite)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let mut finite = selected_request(&p, cut.view().boundary(), terminal(&p));
    finite.context.available_inline = Some(10.0);
    let second = p
        .select(finite)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    for line in [&first, &second] {
        assert_eq!(
            line.selected_line().request().context.available_inline,
            Some(10.0)
        );
        assert_eq!(line.feedback().advance, 10.0);
        assert_eq!(line.measurements().fit().advance(), 10.0);
        assert_eq!(
            line.measurements().min_content_contribution().advance(),
            10.0
        );
    }
    assert_eq!(first.selected_line().request().context.end, LineEnd::Soft);
    assert_eq!(
        first.selected_line().request().termination,
        crate::PreparedTermination::Ordinary(cut)
    );
    let split = placed(&p, vec![(first, 20.0, 100.0), (second, 20.0, 80.0)]);
    assert_eq!(
        split.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 3)]
    );
    assert_eq!(split.lines()[0].bounds(), Rect::new(98.0, 20.0, 10.0, 10.0));
    assert_eq!(split.lines()[1].bounds(), Rect::new(78.0, 20.0, 10.0, 10.0));
    let intrinsic = whole(&p);
    assert_eq!(
        intrinsic.selected_line().request().context.available_inline,
        None
    );
    assert_eq!(intrinsic.feedback().advance, 30.0);
    assert_eq!(intrinsic.measurements().fit().advance(), 30.0);
    assert_eq!(
        intrinsic
            .measurements()
            .max_content_contribution()
            .advance(),
        30.0
    );
    // This is the contribution of this exact whole selection, not a global
    // paragraph minimum. The actual legal selected word rows above each give10.
    assert_eq!(
        intrinsic
            .measurements()
            .min_content_contribution()
            .advance(),
        30.0
    );
    let whole = placed(&p, vec![(intrinsic, 20.0, 100.0)]);
    assert_frames(&whole, Rect::new(98.0, 20.0, 10.0, 30.0));
    assert_eq!(
        whole
            .cursor(Cursor::new(3, Affinity::Before))
            .unwrap()
            .rect(),
        Rect::new(98.0, 50.0, 10.0, 1.0)
    );
    assert_eq!(whole.source().text().code_units(), [65, 32, 66]);
    assert_eq!(split.source().text().code_units(), [65, 32, 66]);
}

#[test]
fn signed_word_spacing_and_capacity_dependent_hanging_keep_real_cw_pens() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(
        &mut s,
        "A A",
        Style {
            word_spacing: -12.0,
            ..own_style()
        },
        false,
        CW,
    );
    for capacity in [None, Some(18.0)] {
        let mut r = selected_request(&p, start(&p), terminal(&p));
        r.context.available_inline = capacity;
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        // The provider's SPACE advance remains10. Its actual signed spacing
        // contribution−2 moves the following real A to pen8. Equal halves
        // move SPACE paint from logical10 by−6 to4, without changing raw data.
        assert_eq!(line.feedback().advance, 18.0);
        assert_eq!(
            glyphs(&line),
            [
                (35, 0.0, 0.0, 0.0, 10.0),
                (3, 4.0, 0.0, 0.0, 10.0),
                (35, 8.0, 0.0, 0.0, 10.0)
            ]
        );
        let layout = placed(&p, vec![(line, 20.0, 100.0)]);
        assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 18.0));
        assert_eq!(
            layout
                .cursor(Cursor::new(2, Affinity::After))
                .unwrap()
                .rect(),
            Rect::new(98.0, 28.0, 10.0, 1.0)
        );
        assert_eq!(layout.source().text().code_units(), [65, 32, 65]);
    }
    let p = capture(&mut s, "A ", own_style(), false, CW);
    let intrinsic = whole(&p);
    assert_eq!(intrinsic.feedback().advance, 20.0);
    assert_eq!(
        intrinsic
            .measurements()
            .max_content_contribution()
            .advance(),
        20.0
    );
    assert_eq!(intrinsic.feedback().hanging_end, 0.0);
    let mut finite = selected_request(&p, start(&p), terminal(&p));
    finite.context.available_inline = Some(15.0);
    let line = p
        .select(finite)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        line.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert_eq!(
        (
            line.feedback().advance,
            line.measurements().fit().advance(),
            line.measurements().alignment().advance(),
            line.feedback().hanging_end
        ),
        (20.0, 10.0, 15.0, 5.0)
    );
    let space = line
        .edge_units()
        .find(|e| e.source_pieces().collect::<Vec<_>>() == [Range::new(1, 2)])
        .unwrap();
    assert_eq!(
        (
            space.advance(),
            space.excluded_contribution(),
            space.outward_overlap()
        ),
        (10.0, 5.0, 5.0)
    );
    assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, 15.0, 10.0));
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    // The finite metric frame remains15. The raw geometry hull also retains
    // the real hanging SPACE through20; alignment15 cannot erase geometry.
    assert_eq!(layout.orientation(), CW);
    assert_eq!(layout.metrics().orientation(), CW);
    assert_eq!(layout.lines()[0].orientation(), CW);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(98.0, 20.0, 10.0, 15.0)
    );
    assert_eq!(layout.bounds(), Rect::new(98.0, 20.0, 10.0, 20.0));
    // Explicit Metrics follows the separate alignment-flow frame, not raw bounds.
    assert_eq!(layout.metrics().size(), Size::new(10.0, 15.0));
    assert_eq!(layout.metrics().full_width(), 10.0);
    assert_eq!(
        layout
            .cursor(Cursor::new(2, Affinity::Before))
            .unwrap()
            .rect(),
        Rect::new(98.0, 40.0, 10.0, 1.0)
    );
    assert_eq!(layout.source().text().code_units(), [65, 32]);

    // Same genuine source/font/request in the preserved Horizontal basis:
    // the line metric frame and raw geometry hull were already distinct.
    let hp = capture(&mut s, "A ", own_style(), false, H);
    let mut hr = selected_request(&hp, start(&hp), terminal(&hp));
    hr.context.available_inline = Some(15.0);
    let horizontal = hp
        .select(hr)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(horizontal.feedback().advance, 20.0);
    assert_eq!(horizontal.measurements().alignment().advance(), 15.0);
    assert_eq!(horizontal.metric_bounds(), Rect::new(0.0, 0.0, 15.0, 10.0));
    let horizontal = placed(&hp, vec![(horizontal, 20.0, 100.0)]);
    assert_eq!(
        horizontal.lines()[0].bounds(),
        Rect::new(20.0, 92.0, 15.0, 10.0)
    );
    assert_eq!(horizontal.bounds(), Rect::new(20.0, 92.0, 20.0, 10.0));
    assert_eq!(horizontal.metrics().size(), Size::new(15.0, 10.0));
    assert_eq!(horizontal.metrics().full_width(), 15.0);
    assert_eq!(horizontal.source().text().code_units(), [65, 32]);
}

#[test]
fn actual_computed_scope_mandatory_keeps_cause_and_source_without_a_control_proxy() {
    use crate::{SourceBoxSide, SourceProcessingEvent as E};
    let original = source("AB");
    let processed = std::sync::Arc::new(
        crate::process_contextual_source(crate::ContextualSourceInput {
            scope_alignment_limits: None,
            source: original.clone(),
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            envelope: crate::SourceProcessingEnvelope::Complete,
            origins: [Range::new(0, 1), Range::new(1, 2)]
                .into_iter()
                .map(|range| crate::ParagraphOrigin {
                    text_node: None,
                    source: original.clone(),
                    local: range,
                    aggregate: range,
                    kind: crate::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![crate::SourceProcessingConnection {
                before_origin: 0,
                after_origin: 1,
                kind: crate::ParagraphConnectionKind::Continuous,
            }],
            boundary_policies: vec![crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            }],
            controls: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![crate::ComputedLineBoundaryInput::Scope {
                scope: 1,
                side: SourceBoxSide::Before,
            }],
            scopes: vec![
                crate::SourceProcessingScope::Block {
                    writing_axis: None,
                    horizontal_inner_direction: None,
                    font_reference: None,
                    topology: crate::SourceProcessingTopology::Complete,
                    trim: Default::default(),
                    base_style: own_style(),
                    used_direction: ParagraphBaseDirection::LeftToRight,
                },
                crate::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: None,
                    boundaries: None,
                },
            ],
            paragraphs: vec![crate::SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: Affinity::After,
                },
                E::Connection(0),
                E::EnterScope(1),
                E::Origin {
                    origin: 1,
                    affinity: Affinity::After,
                },
                E::ExitScope(1),
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let cause = processed.captured_computed_line_boundary(0).unwrap();
    assert_eq!(
        cause.target(),
        crate::CapturedComputedLineBoundaryTarget::Scope {
            scope: processed.captured_scope(1).unwrap(),
            side: SourceBoxSide::Before
        }
    );
    assert_eq!(cause.domain(), processed.captured_scope(0).unwrap());
    assert_eq!(cause.paragraph(), processed.captured_paragraph(0).unwrap());
    assert_eq!(cause.source_position().get(), 1);
    let a = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_processed_paragraph(
            processed.paragraphs().next().unwrap(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = s
        .prepare(&a, PreparationOptions::default().with_orientation(CW))
        .unwrap();
    assert_eq!(p.control_views().count(), 0);
    let forces = p
        .opportunity_views()
        .filter(|o| o.kind() == PreparedBreakKind::Forced)
        .collect::<Vec<_>>();
    assert_eq!(forces.len(), 1);
    let participant = forces[0].computed_line_boundary().unwrap();
    assert_eq!(participant.view().causes().collect::<Vec<_>>(), [cause]);
    assert_eq!(participant.view().source_position().get(), 1);
    assert_eq!(participant.view().processed_position().get(), 1);
    assert_eq!(p.participants().filter(|part|matches!(part,PreparedParticipant::ComputedLineBoundary(c) if *c==participant)).count(),1);
    assert!(matches!(
        reason(
            &p.select(selected_request(&p, start(&p), terminal(&p)))
                .unwrap_err()
        ),
        ProtocolError::IllegalOpportunity { .. }
    ));
    let force = forces[0].handle();
    let first = p
        .select(selected_request(&p, start(&p), force.clone()))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    let second = p
        .select(selected_request(&p, force.view().boundary(), terminal(&p)))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (10.0, 10.0)
    );
    let layout = placed(&p, vec![(first, 20.0, 100.0), (second, 20.0, 80.0)]);
    assert_eq!(
        layout.lines().iter().map(|l| l.range()).collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(98.0, 20.0, 10.0, 10.0)
    );
    assert_eq!(
        layout.lines()[1].bounds(),
        Rect::new(78.0, 20.0, 10.0, 10.0)
    );
    assert_eq!(layout.source(), &original);
}

#[test]
fn real_endpoint_strips_keep_source_free_slots_and_exact_cw_position_selection() {
    use crate::ParagraphControlKind as K;
    let mut value = input(source("A"), own_style(), false);
    value.controls = vec![
        crate::ParagraphControl {
            at: 0,
            kind: K::InlineBoundaryStart,
        },
        crate::ParagraphControl {
            at: 1,
            kind: K::InlineBoundaryEnd,
        },
    ];
    value.anchor_order = (0..2)
        .map(|control| crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Control(control),
            affinity: Affinity::After,
        })
        .collect();
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = prepare(&mut s, value, CW);
    let controls = p.control_views().map(|c| c.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let allocation = |c: &PreparedControl| {
        crate::InlineBoundaryAllocation::try_new(match c.view().kind() {
            K::InlineBoundaryStart => 2.0,
            K::InlineBoundaryEnd => 3.0,
            _ => panic!("only actual endpoints in this profile"),
        })
        .unwrap()
    };
    let mut r = selected_request(&p, start(&p), terminal(&p));
    for c in &mut r.controls {
        c.boundary = Some(allocation(&c.occurrence));
    }
    while let Some(occurrence) = p.boundary_context_demand(&r).unwrap() {
        r.boundary_context
            .push(crate::InlineBoundaryContextAllocation {
                boundary: allocation(&occurrence),
                occurrence,
            });
    }
    let line = p
        .select(r)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 15.0);
    let slots = line.control_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 2);
    for ((slot, control), interval) in slots.iter().zip(&controls).zip([(0.0, 2.0), (12.0, 15.0)]) {
        assert_eq!(slot.control(), *control);
        assert_eq!(slot.inline_bounds(), interval);
        assert_eq!(slot.level().get(), 0);
    }
    let fragments = line
        .fragment_views()
        .filter(|f| matches!(f.owner(), crate::FinalizedFragmentOwner::Control(_)))
        .collect::<Vec<_>>();
    assert_eq!(fragments.len(), 2);
    for ((fragment, control), rect) in fragments.iter().zip(&controls).zip([
        Rect::new(0.0, 0.0, 2.0, 10.0),
        Rect::new(12.0, 0.0, 3.0, 10.0),
    ]) {
        assert_eq!(
            fragment.owner(),
            crate::FinalizedFragmentOwner::Control(control.clone())
        );
        assert_eq!(fragment.bounds(), Some(rect));
        assert!(fragment.origin().is_none());
        assert!(fragment.local_range().is_none());
        assert!(fragment.aggregate_range().is_none());
        assert!(fragment.processed_range().is_none());
    }
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_frames(&layout, Rect::new(98.0, 20.0, 10.0, 15.0));
    assert_eq!(
        layout.run_views().next().unwrap().origin(),
        Point::new(100.0, 22.0)
    );
    assert_eq!(layout.glyph_runs()[0].glyphs()[0].id(), 35);
    assert_eq!(layout.glyph_runs()[0].glyphs()[0].range(), Range::new(0, 1));
    let positions = layout.positions().collect::<Vec<_>>();
    let at = |index, y| {
        positions
            .iter()
            .find(|p| {
                p.source_cursor().index() == index
                    && layout.position_geometry(p).unwrap().rect() == Rect::new(98.0, y, 10.0, 1.0)
            })
            .expect("actual minted endpoint side")
    };
    for (index, before, after, extent) in [(0, 20.0, 22.0, 2.0), (1, 32.0, 35.0, 3.0)] {
        let selection = layout
            .position_selection(at(index, before), at(index, after))
            .unwrap();
        assert!(!selection.is_collapsed());
        assert!(selection.source_pieces().is_empty());
        assert!(selection.source_text().is_empty());
        assert!(selection.plaintext().is_empty());
        assert_eq!(selection.geometry().rects().len(), 1);
        assert_eq!(
            selection.geometry().rects()[0].rect(),
            Rect::new(98.0, before, 10.0, extent)
        );
    }
    let source_only = layout
        .selection(Selection::new(
            Cursor::new(0, Affinity::After),
            Cursor::new(1, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(source_only.rects().len(), 1);
    assert_eq!(
        source_only.rects()[0].rect(),
        Rect::new(98.0, 22.0, 10.0, 10.0)
    );
    let complete = layout.position_selection(at(0, 20.0), at(1, 35.0)).unwrap();
    assert_eq!(complete.source_text().code_units(), [65]);
    assert_eq!(complete.plaintext().code_units(), [65]);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn non_square_sideways_ax_width_is_actual_inline_height_not_cross_width() {
    let mut s = system_with(crate::shaping_test_fonts::sideways_ahem());
    let p = capture(
        &mut s,
        "X",
        Style {
            line_height: crate::LineHeight::Absolute(20.0),
            ..own_style()
        },
        false,
        CW,
    );
    let line = whole(&p);
    // Genuine font8/2 plus half-leading5 gives before13/after7.
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.feedback().line_height, 20.0);
    assert_eq!(
        line.run_views().next().unwrap().origin(),
        Point::new(0.0, 13.0)
    );
    let layout = placed(&p, vec![(line, 20.0, 100.0)]);
    assert_frames(&layout, Rect::new(93.0, 20.0, 20.0, 10.0));
    let mut state = crate::Accessibility::default();
    let parent = accesskit::NodeId(1515006);
    let update = layout
        .accessibility_update(&mut state, parent, parent, Point::default())
        .unwrap();
    let nodes = update
        .nodes
        .iter()
        .filter(|(_, n)| n.role() == accesskit::Role::TextRun)
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1);
    let node = &nodes[0].1;
    assert_eq!(
        node.text_direction(),
        Some(accesskit::TextDirection::TopToBottom)
    );
    assert_eq!(node.character_widths(), Some(&[10.0][..]));
    let b = node.bounds().unwrap();
    assert_eq!((b.x0, b.y0, b.x1, b.y1), (93.0, 20.0, 113.0, 30.0));
    assert_ne!(b.x1 - b.x0, b.y1 - b.y0);
    assert_eq!(layout.source().text().code_units(), [88]);
    assert!(
        layout
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap()
            .is_some()
    );
}

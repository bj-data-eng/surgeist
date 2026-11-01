//! Parent-side box edges relocate retained Ahem ink without painting edges.
//! Exact outline bounds, caller translation and integer raster displacement
//! independently qualify the same placed source geometry.
use super::*;

// Nest actual nominated C inside P while preserving both AA origins and their
// genuine continuous source connection.
pub(super) fn nest_pair_input(input: &mut text::ContextualSourceInput) {
    use text::SourceProcessingEvent as E;
    assert_eq!(input.source.text(), &text::Text::from("AA"));
    assert_eq!(input.controls.len(), 2);
    assert_eq!(input.scopes.len(), 2);
    input.controls.extend([
        text::ParagraphControl {
            at: 1,
            kind: text::ParagraphControlKind::InlineBoundaryStart,
        },
        text::ParagraphControl {
            at: 2,
            kind: text::ParagraphControlKind::InlineBoundaryEnd,
        },
    ]);
    let mut child = input.scopes[1].clone();
    let text::SourceProcessingScope::Inline { boundaries, .. } = &mut child else {
        unreachable!()
    };
    *boundaries = Some(text::SourceProcessingScopeBoundaries { start: 2, end: 3 });
    input.scopes.push(child);
    let before_body = input
        .events
        .iter()
        .position(|e| matches!(e, E::Origin { origin: 1, .. }))
        .unwrap();
    input.events.splice(
        before_body..before_body,
        [
            E::EnterScope(2),
            E::Control {
                control: 2,
                affinity: text::Affinity::After,
            },
        ],
    );
    let before_parent_end = input
        .events
        .iter()
        .position(|e| matches!(e, E::Control { control: 1, .. }))
        .unwrap();
    input.events.splice(
        before_parent_end..before_parent_end,
        [
            E::Control {
                control: 3,
                affinity: text::Affinity::After,
            },
            E::ExitScope(2),
        ],
    );
}

fn placed_nested(prepared: &text::PreparedText, positive: bool) -> text::Layout {
    let controls = prepared
        .control_views()
        .map(|v| v.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 4);
    // Resolve true nominations/parents, never equal source cuts or view ordinals.
    let ps = controls
        .iter()
        .find(|control| {
            let source = control.captured_control().unwrap();
            source
                .scope()
                .parent()
                .is_some_and(|p| p.role() == text::ScopeRole::Domain)
                && *control.view().kind() == text::ParagraphControlKind::InlineBoundaryStart
        })
        .unwrap();
    let p = ps.captured_control().unwrap().scope();
    let cs = controls
        .iter()
        .find(|control| {
            let source = control.captured_control().unwrap();
            source.scope().parent() == Some(p.clone())
                && *control.view().kind() == text::ParagraphControlKind::InlineBoundaryStart
        })
        .unwrap();
    let c = cs.captured_control().unwrap().scope();
    let pe = controls
        .iter()
        .find(|control| control.captured_control() == p.exit_control())
        .unwrap();
    let ce = controls
        .iter()
        .find(|control| control.captured_control() == c.exit_control())
        .unwrap();
    assert_eq!(p.entry_control(), ps.captured_control());
    assert_eq!(c.entry_control(), cs.captured_control());
    assert!(
        ps.captured_control().unwrap().event_position()
            < cs.captured_control().unwrap().event_position()
    );
    assert!(
        cs.captured_control().unwrap().event_position()
            < ce.captured_control().unwrap().event_position()
    );
    assert!(
        ce.captured_control().unwrap().event_position()
            < pe.captured_control().unwrap().event_position()
    );
    let p_context = prepared
        .inline_box_edge_context(
            text::InlineBoxSource::captured(p.clone()),
            text::InlineBoxSource::captured(p.parent().unwrap()),
            text::LineEdgeSide::Start,
        )
        .unwrap();
    let c_context = prepared
        .inline_box_edge_context(
            text::InlineBoxSource::captured(c),
            text::InlineBoxSource::captured(p),
            text::LineEdgeSide::End,
        )
        .unwrap();
    let allocations = [
        (ps, 1.0, p_context.clone()),
        (cs, 2.0, c_context.clone()),
        (ce, 3.0, c_context),
        (pe, 4.0, p_context),
    ]
    .into_iter()
    .map(|(control, amount, context)| text::ControlAllocation {
        occurrence: control.clone(),
        strut: None,
        boundary: Some(text::InlineBoundaryAllocation::box_edge(
            context,
            text::InlineBoxEdgeComponents::from_f64(0.0, if positive { amount } else { 0.0 }, 0.0)
                .unwrap(),
        )),
    })
    .collect();
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
    let end = segment.end_opportunity();
    let request = text::LineSelectionRequest {
        inline_boxes: None,
        segment: segment.clone(),
        start: segment.start(),
        end: segment.end(),
        termination: text::PreparedTermination::Ordinary(end),
        participants: prepared.participants().collect(),
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls: allocations,
        boundary_context: vec![],
        line_box_struts: vec![],
        line_strut: None,
        context: text::LineContext {
            alignment_insets: text::LineAlignmentInsets::ZERO,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            line_box_direction: text::LineBoxDirection::LeftToRight,
            formatted_line: text::FormattedLineContext::default(),
            orientation: text::LineOrientation::Horizontal,
            start: text::LineStart::Paragraph,
            end: text::LineEnd::Paragraph,
            available_inline: Some(32.0),
            indent: 0.0,
            alignment: text::Alignment::Start,
            tabs: text::TabContext::NoTabs,
            spacing: text::SpacingPolicy::None,
        },
    };
    assert!(
        prepared
            .boundary_context_demand(&request)
            .unwrap()
            .is_none()
    );
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (if positive { 30.0 } else { 20.0 }, 8.0, 10.0)
    );
    let packet = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|g| {
                    (
                        g.range(),
                        g.id(),
                        run.origin().x + g.pen() + f64::from(g.x()),
                        g.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        packet,
        [
            (text::Range::new(0, 1), 35, 0.0, 10.0),
            (
                text::Range::new(1, 2),
                35,
                if positive { 14.0 } else { 10.0 },
                10.0
            )
        ]
    );
    for (control, bounds) in [
        (ps, if positive { (10.0, 11.0) } else { (10.0, 10.0) }),
        (cs, if positive { (24.0, 26.0) } else { (20.0, 20.0) }),
        (ce, if positive { (11.0, 14.0) } else { (10.0, 10.0) }),
        (pe, if positive { (26.0, 30.0) } else { (20.0, 20.0) }),
    ] {
        assert_eq!(
            line.control_slots()
                .find(|slot| slot.control() == *control)
                .unwrap()
                .strip_interval(),
            bounds
        );
    }
    let layout = text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 0.0,
                baseline: BASELINE,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source().id(), Some(text::Id::from_u64(640725)));
    assert_eq!(layout.source().text(), &text::Text::from("AA"));
    for (start, end) in [(0, 1), (1, 2)] {
        let before = layout
            .positions()
            .find(|p| p.source_cursor() == text::Cursor::new(start, text::Affinity::After))
            .unwrap();
        let after = layout
            .positions()
            .find(|p| p.source_cursor() == text::Cursor::new(end, text::Affinity::Before))
            .unwrap();
        let selected = layout.position_selection(&before, &after).unwrap();
        assert_eq!(selected.source_text(), &text::Text::from("A"));
        assert_eq!(selected.plaintext(), &text::Text::from("A"));
    }
    layout
}

#[test]
fn nested_box_edges_relocate_only_owned_ahem_ink_through_real_render() {
    qualify_ahem();
    let mut system = system(AHEM.to_vec());
    let prepared = captured_pair(&mut system, true);
    let layouts = [
        placed_nested(&prepared, false),
        placed_nested(&prepared, true),
    ];
    drop(prepared);
    drop(system);
    let mut renderer =
        pollster::block_on(Renderer::new(Options::default())).expect("real GPU adapter");
    assert!(renderer.default_wgpu_device_queue().is_some());
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(32.0, 32.0), 1.0)).unwrap();
    let mut frames = Vec::new();
    for (layout, positive) in layouts.iter().zip([false, true]) {
        // Caller transform is a separate once-only paint operation. Layout and
        // original source positions above stay unchanged.
        let scene =
            prepare_text_layout(layout, Transform::translation(3.0, 2.0).unwrap(), limits())
                .unwrap();
        assert!(std::ptr::eq(scene.layout(), layout));
        let mut glyphs = Vec::new();
        let mut hull = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for command in &scene.scene().commands {
            let (run, transform) = outline(command); // No control/background rectangles.
            assert_eq!(run.font().bytes(), AHEM);
            assert_eq!(run.instance().size(), 10.0);
            assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
            let [a, b, c, d, tx, ty] = transform.as_array();
            assert_eq!([a, b, c, d], [1.0, 0.0, 0.0, 1.0]);
            for glyph in run.glyphs() {
                glyphs.push((glyph.id(), tx + glyph.x(), ty + glyph.y(), glyph.advance()));
            }
            let rect = run.bounds().ink_rect().unwrap();
            hull.0 = hull.0.min(tx + rect.x());
            hull.1 = hull.1.min(ty + rect.y());
            hull.2 = hull.2.max(tx + rect.x() + rect.width());
            hull.3 = hull.3.max(ty + rect.y() + rect.height());
        }
        glyphs.sort_by(|a, b| a.1.total_cmp(&b.1));
        assert_eq!(
            glyphs,
            [
                (35, 3.0, 14.0, 10.0),
                (35, if positive { 17.0 } else { 13.0 }, 14.0, 10.0)
            ]
        );
        assert_eq!(hull, (3.0, 6.0, if positive { 27.0 } else { 23.0 }, 16.0));
        let stats =
            pollster::block_on(renderer.render(&mut surface, scene.scene(), Parameters::default()))
                .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!((image.size().width(), image.size().height()), (32, 32));
        frames.push(image.into_rgba());
    }
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
    let [zero, positive] = frames.as_slice() else {
        unreachable!()
    };
    let painted = |frame: &[u8]| frame.chunks_exact(4).filter(|p| p[3] > 0).count();
    assert!(painted(zero) > 0);
    assert_eq!(painted(zero), painted(positive));
    for (frame, second) in [(zero, 13), (positive, 17)] {
        assert!(
            (6..16).any(|y| (3..13).any(|x| pixel(frame, x, y)[3] > 0)),
            "first actual glyph paints"
        );
        assert!(
            (6..16).any(|y| (second..second + 10).any(|x| pixel(frame, x, y)[3] > 0)),
            "relocated owned glyph paints"
        );
    }
    for y in 0..SIDE {
        for x in 0..SIDE {
            if x < 13 {
                assert_eq!(pixel(positive, x, y), pixel(zero, x, y));
            } else if x < 17 {
                assert_eq!(pixel(positive, x, y)[3], 0, "unpainted real endpoint gap");
            } else {
                assert_eq!(pixel(positive, x, y), pixel(zero, x - 4, y));
            }
            if !(6..16).contains(&y) || !(3..23).contains(&x) {
                assert_eq!(pixel(zero, x, y)[3], 0);
            }
            if !(6..16).contains(&y) || !(3..27).contains(&x) {
                assert_eq!(pixel(positive, x, y)[3], 0);
            }
        }
    }
}

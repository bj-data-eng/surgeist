//! Signed endpoint flow relocates the owned A without painting its margins or strips.
use super::*;

fn line_with_components(p: &text::PreparedText, signed: bool) -> text::FinalizedLine {
    let start = p
        .control_views()
        .find(|v| *v.kind() == text::ParagraphControlKind::InlineBoundaryStart)
        .unwrap()
        .handle();
    let end = p
        .control_views()
        .find(|v| *v.kind() == text::ParagraphControlKind::InlineBoundaryEnd)
        .unwrap()
        .handle();
    let owner = start.captured_control().unwrap().scope();
    assert_eq!(end.captured_control().unwrap().scope(), owner);
    let parent = owner.parent().unwrap();
    assert_eq!(parent.role(), text::ScopeRole::Domain);
    let context = p
        .inline_box_edge_context(
            text::InlineBoxSource::captured(owner),
            text::InlineBoxSource::captured(parent),
            text::LineEdgeSide::Start,
        )
        .unwrap();
    let mut segments = p.source_segments();
    assert_eq!(segments.len(), 1);
    let segment = segments.next().unwrap();
    let end_offer = segment.end_opportunity();
    let participants = p.participants().collect::<Vec<_>>();
    let controls = participants
        .iter()
        .filter_map(|part| {
            let text::PreparedParticipant::Control(occurrence) = part else {
                return None;
            };
            let is_start = *occurrence == start;
            assert!(is_start || *occurrence == end);
            let (margin, border, padding) = match (signed, is_start) {
                (true, true) => (3.0, 2.0, 1.0),
                (true, false) => (-4.0, 1.0, 2.0),
                (false, _) => (0.0, 0.0, 0.0),
            };
            Some(text::ControlAllocation {
                occurrence: occurrence.clone(),
                strut: None,
                boundary: Some(text::InlineBoundaryAllocation::box_edge(
                    context.clone(),
                    text::InlineBoxEdgeComponents::from_f64(margin, border, padding).unwrap(),
                )),
            })
        })
        .collect();
    let request = text::LineSelectionRequest {
        start: segment.start(),
        end: segment.end(),
        segment,
        termination: text::PreparedTermination::Ordinary(end_offer),
        participants,
        alternative: None,
        incoming: None,
        atomics: vec![],
        controls,
        boundary_context: vec![],
        line_box_struts: vec![],
        line_strut: None,
        context: text::LineContext {
            alignment_insets: text::LineAlignmentInsets::ZERO,
            formatted_line: text::FormattedLineContext::default(),
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            line_box_direction: text::LineBoxDirection::LeftToRight,
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
    assert!(p.boundary_context_demand(&request).unwrap().is_none());
    let line = p
        .select(request)
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, if signed { 25.0 } else { 20.0 });
    let expected_second = if signed { 16.0 } else { 10.0 };
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!(
                instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds(),
                Some(text::Rect::new(0.0, -8.0, 10.0, 10.0))
            );
            glyphs.push((
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    glyphs.sort_by_key(|row| row.0.start);
    assert_eq!(
        glyphs,
        [
            (text::Range::new(0, 1), 35, 0.0, 0.0, 0.0, 10.0),
            (text::Range::new(1, 2), 35, expected_second, 0.0, 0.0, 10.0),
        ]
    );
    let opening = line.control_slots().find(|s| s.control() == start).unwrap();
    let closing = line.control_slots().find(|s| s.control() == end).unwrap();
    assert_eq!(
        opening.strip_interval(),
        if signed { (13.0, 16.0) } else { (10.0, 10.0) }
    );
    assert_eq!(
        closing.strip_interval(),
        if signed { (26.0, 29.0) } else { (20.0, 20.0) }
    );
    assert_eq!(
        closing.flow_interval(),
        if signed { (26.0, 25.0) } else { (20.0, 20.0) }
    );
    line
}

fn place_signed_pair(p: &text::PreparedText, line: text::FinalizedLine) -> text::Layout {
    text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: p.clone(),
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
    .unwrap()
}

fn observe_scene(scene: &crate::PreparedTextScene<'_>, second: f64) {
    let mut glyphs = Vec::new();
    let mut hull = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for command in &scene.scene().commands {
        let (run, transform) = outline(command); // Neither margin nor strip is paint.
        assert_eq!(run.font().bytes(), AHEM);
        assert_eq!(run.font().index(), 0);
        assert_eq!(run.instance().size(), 10.0);
        assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
        assert!(run.path_elements().count() > 0);
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
    assert_eq!(glyphs, [(35, 2.0, 13.0, 10.0), (35, second, 13.0, 10.0)]);
    assert_eq!(hull, (2.0, 5.0, second + 10.0, 15.0));
}

#[test]
fn signed_box_flow_retracts_without_clipping_or_painting_the_retained_ahem_packet() {
    qualify_ahem();
    let mut system = system(AHEM.to_vec());
    // The maintained capture has source A outside P and source A inside P,
    // with a real Continuous connection and actual nominated P endpoints.
    let p = captured_pair(&mut system, false);
    let prior = line_with_components(&p, false);
    let signed = line_with_components(&p, true);
    let signed = signed.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(signed.feedback().advance, 25.0);
    let prior = prior.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(prior.feedback().advance, 20.0);
    let layouts = [place_signed_pair(&p, prior), place_signed_pair(&p, signed)];
    for layout in &layouts {
        assert_eq!(layout.source().id(), Some(text::Id::from_u64(640725)));
        assert_eq!(layout.source().text(), &text::Text::from("AA"));
    }
    let publication = layouts[0]
        .run_views()
        .next()
        .unwrap()
        .face()
        .catalogue_identity()
        .clone();
    system.refresh_fonts().unwrap();
    assert_ne!(
        &publication,
        system
            .registered_font_faces()
            .next()
            .unwrap()
            .catalogue_identity()
    );
    drop(p);
    drop(system);
    let caller = Transform::translation(2.0, 1.0).unwrap();
    let prior_scene = prepare_text_layout(&layouts[0], caller, limits()).unwrap();
    observe_scene(&prior_scene, 12.0);
    let mut renderer =
        pollster::block_on(Renderer::new(Options::default())).expect("real GPU adapter");
    assert!(renderer.default_wgpu_device_queue().is_some());
    let mut surface =
        pollster::block_on(renderer.create_headless(Size::new(32.0, 32.0), 1.0)).unwrap();
    let mut frames = Vec::new();
    for (layout, second) in layouts.iter().zip([12.0, 18.0]) {
        assert!(
            layout
                .run_views()
                .all(|run| run.face().catalogue_identity() == &publication)
        );
        let scene = prepare_text_layout(layout, caller, limits()).unwrap();
        assert!(std::ptr::eq(scene.layout(), layout));
        observe_scene(&scene, second);
        let stats =
            pollster::block_on(renderer.render(&mut surface, scene.scene(), Parameters::default()))
                .unwrap();
        assert_eq!(stats.route, Some(RenderRoute::DirectVello));
        let image = pollster::block_on(renderer.read_headless(&surface)).unwrap();
        assert_eq!((image.size().width(), image.size().height()), (32, 32));
        frames.push(image.into_rgba());
    }
    // A subsequent scene and font-catalogue replacement cannot mutate prior ink.
    observe_scene(&prior_scene, 12.0);
    assert!(renderer.default_device_has_no_terminal_signal_for_test());
    let [zero, signed] = frames.as_slice() else {
        unreachable!()
    };
    let painted = |frame: &[u8]| frame.chunks_exact(4).filter(|p| p[3] > 0).count();
    assert!(painted(zero) > 0);
    assert_eq!(painted(zero), painted(signed));
    assert!(
        (5..15).any(|y| pixel(signed, 27, y)[3] > 0),
        "actual A ink survives beyond caller-translated signed flow end27"
    );
    for y in 0..SIDE {
        for x in 0..SIDE {
            if x < 12 {
                assert_eq!(pixel(signed, x, y), pixel(zero, x, y));
            } else if x < 18 {
                assert_eq!(
                    pixel(signed, x, y)[3],
                    0,
                    "transparent margin/strip gap at({x},{y})"
                );
            } else {
                assert_eq!(pixel(signed, x, y), pixel(zero, x - 6, y));
            }
            if !(5..15).contains(&y) || !(2..22).contains(&x) {
                assert_eq!(pixel(zero, x, y)[3], 0);
            }
            if !(5..15).contains(&y) || !((2..12).contains(&x) || (18..28).contains(&x)) {
                assert_eq!(pixel(signed, x, y)[3], 0, "closing strip is not paint");
            }
        }
    }
}

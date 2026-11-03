//! Signed box flow and source-less strips retain separate interaction geometry.
use super::*;

fn components(narrow: bool, margin: f64, border: f64, padding: f64) -> t::InlineBoxEdgeComponents {
    if narrow {
        // All authored values in these cases are exact in both scalar lanes.
        t::InlineBoxEdgeComponents::from_f32(margin as f32, border as f32, padding as f32).unwrap()
    } else {
        t::InlineBoxEdgeComponents::from_f64(margin, border, padding).unwrap()
    }
}

fn captured(out_of_flow: bool) -> t::PreparedText {
    let mut source = t::Source::identified(t::Text::from("A"), t::Id::from_u64(1_635_101), 1);
    let mut body = vec![
        E::EnterScope(1),
        control(0),
        origin(0),
        control(1),
        E::ExitScope(1),
    ];
    if out_of_flow {
        // A real Source OOF occurrence after the real closing scope. Its declared
        // external size is not an in-flow allocation and must not donate advance.
        source.inline_box(t::InlineBox::new(
            t::Id::from_u64(1_635_199),
            t::InlineBoxKind::OutOfFlow,
            1,
            t::Size::new(7.0, 9.0),
        ));
        body.push(E::Atomic {
            atomic: 0,
            affinity: t::Affinity::After,
        });
    }
    let prepared = prepare_capture(
        source,
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1)],
        &[(K::InlineBoundaryStart, 0), (K::InlineBoundaryEnd, 1)],
        &[(0, 1)],
        body,
        AHEM,
    );
    assert_eq!(prepared.source_segments().len(), 1);
    prepared
}

fn execute(p: &t::PreparedText, narrow: bool, signed: bool) -> t::FinalizedLine {
    let start = p
        .control_views()
        .find(|v| *v.kind() == K::InlineBoundaryStart)
        .unwrap()
        .handle();
    let end = p
        .control_views()
        .find(|v| *v.kind() == K::InlineBoundaryEnd)
        .unwrap()
        .handle();
    let owner = start.captured_control().unwrap().scope();
    assert_eq!(end.captured_control().unwrap().scope(), owner);
    let parent = owner.parent().unwrap();
    assert_eq!(parent.role(), t::ScopeRole::Domain);
    let context = p
        .inline_box_edge_context(
            t::InlineBoxSource::captured(owner),
            t::InlineBoxSource::captured(parent),
            t::LineEdgeSide::Start,
        )
        .unwrap();
    let mut input = request(
        p,
        p.source_segments().next().unwrap().start(),
        paragraph_end(p),
        &[],
        100.0,
    );
    input.context.spacing = t::SpacingPolicy::None;
    for allocation in &mut input.controls {
        let is_start = allocation.occurrence == start;
        assert!(is_start || allocation.occurrence == end);
        let value = match (signed, is_start) {
            (true, true) => components(narrow, 3.0, 2.0, 1.0),
            (true, false) => components(narrow, -4.0, 1.0, 2.0),
            (false, _) => components(narrow, 0.0, 0.0, 0.0),
        };
        allocation.boundary = Some(t::InlineBoundaryAllocation::box_edge(
            context.clone(),
            value,
        ));
    }
    input.atomics = p
        .atomic_views()
        .map(|view| {
            assert_eq!(view.original_box().kind(), t::InlineBoxKind::OutOfFlow);
            assert_eq!(view.original_box().id(), t::Id::from_u64(1_635_199));
            assert_eq!(view.source_position().get(), 1);
            assert!(view.capture_position().is_some());
            t::AtomicAllocation {
                occurrence: view.handle(),
                allocation: t::AtomicLineAllocation::OutOfFlow,
            }
        })
        .collect();
    assert!(p.boundary_context_demand(&input).unwrap().is_none());
    finalize(p, input)
}

fn packet(line: &t::FinalizedLine, expected_x: f64) {
    let mut actual = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
        assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
        for glyph in run.glyphs() {
            assert_eq!(
                instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds(),
                Some(t::Rect::new(0.0, -8.0, 10.0, 10.0))
            );
            actual.push((
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    assert_eq!(
        actual,
        [(t::Range::new(0, 1), 35, expected_x, 0.0, 0.0, 10.0)]
    );
}

fn placed(p: &t::PreparedText, line: t::FinalizedLine) -> t::Layout {
    assert_eq!(
        line.atomic_slots().len(),
        0,
        "this helper places no external OOF geometry"
    );
    t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 0.0,
                baseline: 8.0,
            },
            orientation: t::LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn source_and_strip_case(narrow: bool) {
    let p = captured(false);
    let prior = execute(&p, narrow, false);
    let line = execute(&p, narrow, true);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (15.0, 8.0, 10.0)
    );
    packet(&line, 6.0);
    let expected = [
        (K::InlineBoundaryStart, (0.0, 6.0), (3.0, 6.0)),
        (K::InlineBoundaryEnd, (16.0, 15.0), (16.0, 19.0)),
    ];
    for (kind, flow, strip) in &expected {
        let slot = line
            .control_slots()
            .find(|v| v.control().view().kind() == kind)
            .unwrap();
        assert_eq!(slot.flow_interval(), *flow);
        assert_eq!(slot.strip_interval(), *strip);
        assert_eq!(
            slot.level().get(),
            0,
            "LTR A and its neutral source endpoints have level zero"
        );
        let fragment = line
            .fragment_views()
            .find(|f| f.owner() == t::FinalizedFragmentOwner::Control(slot.control()))
            .unwrap();
        assert_eq!(
            (
                fragment.bounds().unwrap().origin.x,
                fragment.bounds().unwrap().size.width
            ),
            (strip.0, 3.0)
        );
        assert!(
            fragment.origin().is_none()
                && fragment.aggregate_range().is_none()
                && fragment.local_range().is_none()
                && fragment.processed_range().is_none()
        );
    }
    let replay = line.with_spacing(t::SpacingAllocations::None).unwrap();
    assert_eq!(replay.feedback().advance, 15.0);
    packet(&replay, 6.0);
    // A later operation leaves the previous accepted zero-component product usable.
    assert_eq!(prior.feedback().advance, 10.0);
    packet(
        &prior.with_spacing(t::SpacingAllocations::None).unwrap(),
        0.0,
    );
    let layout = placed(&p, replay);
    let owner = p.origin_views().next().unwrap().handle();
    let query = layout
        .source_range_fragments(&owner, t::Range::new(0, 1))
        .unwrap();
    assert_eq!(query.origin(), &owner);
    assert_eq!(query.local_range(), t::Range::new(0, 1));
    let pieces = query
        .units()
        .flat_map(|unit| {
            unit.pieces()
                .map(|piece| {
                    let t::SourceRangeGeometry::Line(piece) = piece else {
                        panic!("ordinary A retains ordinary source geometry")
                    };
                    (piece.advance_edges(), piece.rect())
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(pieces, [((6.0, 16.0), t::Rect::new(6.0, 0.0, 10.0, 10.0))]);
    assert_eq!(layout.source().id(), Some(t::Id::from_u64(1_635_101)));
    assert_eq!(layout.source().text(), &t::Text::from("A"));
    // Strip endpoints retain source-level Before/After, not flow-start/flow-end.
    for (left, right, source) in [(3.0, 6.0, 0), (16.0, 19.0, 1)] {
        // Generic hit testing first chooses a source cursor. The glyph's
        // coincident edge can therefore win over a distinct control occurrence.
        // These unique projections identify the actual opening-right and
        // closing-left control sides; exact movement reaches their other sides.
        let (cursor, movement) = if source == 0 {
            (
                t::Cursor::new(0, t::Affinity::Before),
                t::Movement::PreviousCluster,
            )
        } else {
            (
                t::Cursor::new(1, t::Affinity::After),
                t::Movement::NextCluster,
            )
        };
        let mut matching = layout
            .positions()
            .filter(|position| position.source_cursor() == cursor);
        let endpoint = matching
            .next()
            .expect("one genuine control side at this source cut");
        assert!(
            matching.next().is_none(),
            "the exact source projection is unique in this input"
        );
        let other = layout.move_position(&endpoint, movement).unwrap();
        let (before, after) = if source == 0 {
            (other, endpoint)
        } else {
            (endpoint, other)
        };
        assert_eq!(
            before.source_cursor(),
            t::Cursor::new(source, t::Affinity::After)
        );
        assert_eq!(
            after.source_cursor(),
            t::Cursor::new(source, t::Affinity::Before)
        );
        assert_eq!(
            layout.position_geometry(&before).unwrap().rect().origin.x,
            left
        );
        assert_eq!(
            layout.position_geometry(&after).unwrap().rect().origin.x,
            right
        );
        let selection = layout.position_selection(&before, &after).unwrap();
        assert!(selection.source_text().is_empty() && selection.plaintext().is_empty());
        assert_eq!(
            selection
                .geometry()
                .rects()
                .iter()
                .map(|r| r.rect().size.width)
                .sum::<f64>(),
            3.0
        );
    }
    let mut caret_x = layout
        .positions()
        .map(|position| layout.position_geometry(&position).unwrap().rect().origin.x)
        .collect::<Vec<_>>();
    caret_x.sort_by(f64::total_cmp);
    caret_x.dedup();
    assert_eq!(caret_x, [3.0, 6.0, 16.0, 19.0]);
}

#[test]
fn signed_f32_box_margins_do_not_become_original_range_or_strip_carets() {
    source_and_strip_case(true);
}
#[test]
fn signed_f64_box_margins_do_not_become_original_range_or_strip_carets() {
    source_and_strip_case(false);
}

fn static_site_case(narrow: bool) {
    let p = captured(true);
    let occurrence = p.atomic_views().next().unwrap().handle();
    let prior = execute(&p, narrow, false);
    let line = execute(&p, narrow, true);
    assert_eq!(line.feedback().advance, 15.0);
    packet(&line, 6.0);
    let end = line
        .control_slots()
        .find(|s| *s.control().view().kind() == K::InlineBoundaryEnd)
        .unwrap();
    assert_eq!(end.strip_interval(), (16.0, 19.0));
    assert_eq!(end.flow_interval(), (16.0, 15.0));
    let replay = line.with_spacing(t::SpacingAllocations::None).unwrap();
    for output in [&line, &replay] {
        let slots = output.atomic_slots().collect::<Vec<_>>();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].occurrence, occurrence);
        assert_eq!(slots[0].allocation, t::AtomicLineAllocation::OutOfFlow);
        assert_eq!(
            slots[0].origin.x, 15.0,
            "the real subsequent OOF static site follows signed flow, not strip end19"
        );
    }
    assert_eq!(prior.atomic_slots().next().unwrap().origin.x, 10.0);
    assert_eq!(prior.feedback().advance, 10.0);
    packet(&prior, 0.0);
}

#[test]
fn signed_f32_closing_margin_retracts_the_genuine_following_oof_static_site() {
    static_site_case(true);
}
#[test]
fn signed_f64_closing_margin_retracts_the_genuine_following_oof_static_site() {
    static_site_case(false);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn signed_margins_and_source_less_strips_do_not_donate_ax_character_bounds() {
    use accesskit::{NodeId, Role, TextPosition, TextSelection};
    let p = captured(false);
    let layout = placed(&p, execute(&p, false, true));
    let mut state = t::Accessibility::new(10);
    let parent = NodeId(1000);
    let update = layout
        .accessibility_update(&mut state, parent, parent, t::Point::new(100.0, 200.0))
        .unwrap();
    let paragraph = &update.nodes.iter().find(|(id, _)| *id == parent).unwrap().1;
    assert_eq!(paragraph.role(), Role::Paragraph);
    let mut units = Vec::new();
    for id in paragraph.children() {
        let node = &update
            .nodes
            .iter()
            .find(|(actual, _)| actual == id)
            .unwrap()
            .1;
        assert_eq!(node.role(), Role::TextRun);
        let value = node.value().unwrap();
        let mut byte = 0;
        for (index, length) in node.character_lengths().iter().copied().enumerate() {
            let end = byte + usize::from(length);
            assert_eq!(&value[byte..end], "A");
            units.push((
                TextPosition {
                    node: *id,
                    character_index: index,
                },
                TextPosition {
                    node: *id,
                    character_index: index + 1,
                },
            ));
            byte = end;
        }
        assert_eq!(byte, value.len());
        assert!(
            !node.character_lengths().is_empty(),
            "source-less controls do not invent AX units"
        );
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (106.0, 200.0, 116.0, 210.0)
        );
        if let Some(positions) = node.character_positions() {
            assert_eq!(positions, &[0.0]);
        }
        if let Some(widths) = node.character_widths() {
            assert_eq!(widths, &[10.0]);
        }
    }
    assert_eq!(
        units.len(),
        1,
        "one original A, independent of TextRun grouping"
    );
    // One endpoint occurrence at each source cut: no fabricated nested ambiguity.
    assert_eq!(
        layout
            .access_selection(
                &state,
                t::Selection::new(
                    t::Cursor::new(0, t::Affinity::After),
                    t::Cursor::new(1, t::Affinity::Before),
                )
            )
            .unwrap(),
        Some(TextSelection {
            anchor: units[0].0,
            focus: units[0].1
        })
    );
}

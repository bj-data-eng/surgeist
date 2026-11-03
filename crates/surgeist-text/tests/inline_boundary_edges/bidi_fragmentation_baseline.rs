//! One captured box has separated visual members before wrapping and adjacent
//! members after a genuine optional break, while its used BP edges execute once.
use super::*;

#[derive(Debug, PartialEq)]
enum Logical {
    Text(t::Range, u8),
    Control(K, usize, u8),
}

fn captured_override() -> t::PreparedText {
    let p = prepare_capture(
        t::Source::identified("abcABC", t::Id::from_u64(1_646_101), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 2), (2, 3), (3, 4), (4, 6)],
        &[
            (K::InlineBoundaryStart, 2),
            (K::RightToLeftOverride, 3),
            (K::InlineBoundaryEnd, 4),
            (
                K::OptionalBreak {
                    wrap: t::Wrap::Word,
                },
                4,
            ),
            (K::PopDirectionalFormat, 6),
        ],
        &[(0, 2)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            E::Connection(1),
            control(1),
            origin(2),
            control(2),
            E::ExitScope(1),
            control(3),
            E::Connection(2),
            origin(3),
            control(4),
        ],
        AHEM,
    );
    assert_eq!(p.source_segments().len(), 1);
    assert_eq!(
        p.analyzed_source().source().text(),
        &t::Text::from("abcABC")
    );
    assert_eq!(p.analyzed_source().direction(), t::Direction::LeftToRight);
    assert_eq!(
        p.origin_views()
            .map(|v| v.aggregate_range())
            .collect::<Vec<_>>(),
        [
            t::Range::new(0, 2),
            t::Range::new(2, 3),
            t::Range::new(3, 4),
            t::Range::new(4, 6),
        ]
    );
    assert!(
        p.control_views()
            .all(|v| v.capture_position().is_some() && v.handle().captured_control().is_some())
    );
    // X4 gives ABC level1 under the actual RLO; abc remain level0. Native X9
    // reinsertion retains the preceding level for RLO/PDF, while the genuine
    // After-affinity box/optional controls inherit their actual following side.
    let logical = p
        .participants()
        .map(|part| match part {
            t::PreparedParticipant::Text(unit) => {
                Logical::Text(unit.view().source_range(), unit.view().level().get())
            }
            t::PreparedParticipant::Control(control) => Logical::Control(
                control.view().kind().clone(),
                control.view().source_position().get(),
                control.view().level().get(),
            ),
            other => panic!("this source has only actual text and captured controls: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        logical,
        [
            Logical::Text(t::Range::new(0, 1), 0),
            Logical::Text(t::Range::new(1, 2), 0),
            Logical::Control(K::InlineBoundaryStart, 2, 0),
            Logical::Text(t::Range::new(2, 3), 0),
            Logical::Control(K::RightToLeftOverride, 3, 0),
            Logical::Text(t::Range::new(3, 4), 1),
            Logical::Control(K::InlineBoundaryEnd, 4, 1),
            Logical::Control(
                K::OptionalBreak {
                    wrap: t::Wrap::Word
                },
                4,
                1
            ),
            Logical::Text(t::Range::new(4, 5), 1),
            Logical::Text(t::Range::new(5, 6), 1),
            Logical::Control(K::PopDirectionalFormat, 6, 1),
        ]
    );
    let head = font::table(AHEM, *b"head");
    let hhea = font::table(AHEM, *b"hhea");
    let hmtx = font::table(AHEM, *b"hmtx");
    assert_eq!(u16::from_be_bytes(head[18..20].try_into().unwrap()), 1000);
    assert_eq!(i16::from_be_bytes(hhea[4..6].try_into().unwrap()), 800);
    assert_eq!(i16::from_be_bytes(hhea[6..8].try_into().unwrap()), -200);
    for glyph in [67, 68, 69, 35, 36, 37] {
        assert_eq!(
            u16::from_be_bytes(hmtx[glyph * 4..glyph * 4 + 2].try_into().unwrap()),
            1000
        );
    }
    p
}

fn endpoint(p: &t::PreparedText, kind: K) -> t::PreparedControl {
    let mut controls = p
        .control_views()
        .filter(|v| *v.kind() == kind)
        .map(|v| v.handle());
    let control = controls
        .next()
        .expect("one authentic nominated endpoint of this kind");
    assert!(controls.next().is_none());
    control
}

fn bp(narrow: bool, border: f64) -> t::InlineBoxEdgeComponents {
    if narrow {
        t::InlineBoxEdgeComponents::from_f32(0.0, border as f32, 0.0).unwrap()
    } else {
        t::InlineBoxEdgeComponents::from_f64(0.0, border, 0.0).unwrap()
    }
}

fn line(
    p: &t::PreparedText,
    narrow: bool,
    start: t::PreparedBoundary,
    end: t::PreparedOpportunity,
    available: f64,
) -> t::FinalizedLine {
    let opening = endpoint(p, K::InlineBoundaryStart);
    let closing = endpoint(p, K::InlineBoundaryEnd);
    let owner = opening.captured_control().unwrap().scope();
    assert_eq!(owner.entry_control(), opening.captured_control());
    assert_eq!(owner.exit_control(), closing.captured_control());
    assert_eq!(closing.captured_control().unwrap().scope(), owner);
    let parent = owner.parent().unwrap();
    assert_eq!(parent.role(), t::ScopeRole::Domain);
    let context = p
        .inline_box_edge_context(
            t::InlineBoxSource::captured(owner),
            t::InlineBoxSource::captured(parent),
            t::LineEdgeSide::Start,
        )
        .unwrap();
    let allocation = |control: &t::PreparedControl| {
        let width = if *control == opening {
            3.0
        } else {
            assert_eq!(*control, closing, "only real endpoints request BP geometry");
            5.0
        };
        t::InlineBoundaryAllocation::box_edge(context.clone(), bp(narrow, width))
    };
    let mut input = request(p, start, end, &[], available);
    input.context.spacing = t::SpacingPolicy::None;
    for row in &mut input.controls {
        if row.occurrence == opening || row.occurrence == closing {
            row.boundary = Some(allocation(&row.occurrence));
        } else {
            assert!(row.boundary.is_none());
        }
    }
    while let Some(control) = p.boundary_context_demand(&input).unwrap() {
        let boundary = allocation(&control);
        input
            .boundary_context
            .push(t::InlineBoundaryContextAllocation {
                occurrence: control,
                boundary,
            });
    }
    finalize(p, input)
}

fn actual_optional_break(p: &t::PreparedText) -> t::PreparedOpportunity {
    let control = endpoint(
        p,
        K::OptionalBreak {
            wrap: t::Wrap::Word,
        },
    );
    let mut offers = soft_at(p, 4).into_iter().filter(|offer| {
        let count = offer.view().boundary().view().participant_offset();
        matches!(p.participants().take(count).last(),
            Some(t::PreparedParticipant::Control(previous)) if previous == control)
    });
    let offer = offers
        .next()
        .expect("the genuine optional control issues its actual allowed cut");
    assert!(offers.next().is_none());
    assert_eq!(offer.view().kind(), t::PreparedBreakKind::Allowed);
    assert_eq!(offer.view().boundary().view().source_before().get(), 4);
    assert_eq!(offer.view().boundary().view().source_after().get(), 4);
    offer
}

fn packet(line: &t::FinalizedLine) -> Vec<(t::Range, u32, f64)> {
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.font_size(), 10.0);
        let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
        assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
            assert_eq!(
                instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds(),
                Some(t::Rect::new(0.0, -8.0, 10.0, 10.0))
            );
            glyphs.push((glyph.range(), glyph.id(), run.origin().x + glyph.pen()));
        }
    }
    glyphs.sort_by(|a, b| a.2.total_cmp(&b.2));
    glyphs
}

fn strip(line: &t::FinalizedLine, p: &t::PreparedText, kind: K, expected: (f64, f64)) {
    let control = endpoint(p, kind);
    let mut slots = line.control_slots().filter(|s| s.control() == control);
    let slot = slots.next().expect("actual source endpoint executes once");
    assert!(slots.next().is_none());
    assert_eq!(slot.strip_interval(), expected);
    assert_eq!(
        slot.flow_interval(),
        expected,
        "these cases author zero margins"
    );
}

fn place(p: &t::PreparedText, lines: &[t::FinalizedLine]) -> t::Layout {
    let layout = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                t::PlacementEntry::Formatted(t::LinePlacement {
                    line: line.clone(),
                    anchor: t::LineAnchor {
                        inline_origin: 0.0,
                        baseline: 8.0 + i as f64 * 10.0,
                    },
                    orientation: t::LineOrientation::Horizontal,
                    atomics: vec![],
                })
            })
            .collect(),
    })
    .unwrap();
    assert_eq!(layout.source().id(), Some(t::Id::from_u64(1_646_101)));
    assert_eq!(layout.source().text(), &t::Text::from("abcABC"));
    layout
}

fn placed_packet(layout: &t::Layout) -> Vec<(u32, f64, f64)> {
    let mut glyphs = layout
        .run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        run.origin().x + glyph.pen() + f64::from(glyph.x()),
                        run.origin().y + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    glyphs.sort_by(|a, b| a.2.total_cmp(&b.2).then_with(|| a.1.total_cmp(&b.1)));
    glyphs
}

fn wide(narrow: bool) {
    let p = captured_override();
    let output = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        paragraph_end(&p),
        100.0,
    );
    assert_eq!(output.feedback().advance, 68.0);
    assert_eq!(
        packet(&output),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(5, 6), 37, 33.0),
            (t::Range::new(4, 5), 36, 43.0),
            (t::Range::new(3, 4), 35, 53.0),
        ]
    );
    strip(&output, &p, K::InlineBoundaryStart, (20.0, 23.0));
    strip(&output, &p, K::InlineBoundaryEnd, (63.0, 68.0));
    let layout = place(&p, &[output]);
    assert_eq!(
        placed_packet(&layout),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 23.0, 8.0),
            (37, 33.0, 8.0),
            (36, 43.0, 8.0),
            (35, 53.0, 8.0)
        ]
    );
}

fn wrapped(narrow: bool) {
    let p = captured_override();
    let cut = actual_optional_break(&p);
    let continuation = cut.view().boundary();
    let first = line(
        &p,
        narrow,
        p.source_segments().next().unwrap().start(),
        cut,
        48.0,
    );
    let second = line(&p, narrow, continuation, paragraph_end(&p), 48.0);
    assert_eq!(
        (first.feedback().advance, second.feedback().advance),
        (48.0, 20.0)
    );
    assert_eq!(
        packet(&first),
        [
            (t::Range::new(0, 1), 67, 0.0),
            (t::Range::new(1, 2), 68, 10.0),
            (t::Range::new(2, 3), 69, 23.0),
            (t::Range::new(3, 4), 35, 33.0),
        ]
    );
    strip(&first, &p, K::InlineBoundaryStart, (20.0, 23.0));
    strip(&first, &p, K::InlineBoundaryEnd, (43.0, 48.0));
    assert_eq!(
        packet(&second),
        [
            (t::Range::new(5, 6), 37, 0.0),
            (t::Range::new(4, 5), 36, 10.0)
        ]
    );
    assert!(
        second.control_slots().all(|slot| !matches!(
            slot.control().view().kind(),
            K::InlineBoundaryStart | K::InlineBoundaryEnd
        )),
        "context receipts do not execute a future/donated edge"
    );
    let layout = place(&p, &[first, second]);
    assert_eq!(
        placed_packet(&layout),
        [
            (67, 0.0, 8.0),
            (68, 10.0, 8.0),
            (69, 23.0, 8.0),
            (35, 33.0, 8.0),
            (37, 0.0, 18.0),
            (36, 10.0, 18.0)
        ]
    );
}

#[test]
fn f32_bp_edges_enclose_separated_visual_members_of_one_captured_box() {
    wide(true);
}
#[test]
fn f64_bp_edges_enclose_separated_visual_members_of_one_captured_box() {
    wide(false);
}
#[test]
fn f32_optional_break_makes_the_members_adjacent_without_donating_edges_to_cb() {
    wrapped(true);
}
#[test]
fn f64_optional_break_makes_the_members_adjacent_without_donating_edges_to_cb() {
    wrapped(false);
}

#[path = "bidi_fragmentation_functional.rs"]
mod bidi_fragmentation_functional;

#[path = "empty_box_family.rs"]
mod empty_box_family;

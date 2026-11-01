//! Actual edge components separate between-TCU shaping while preserving shared EGCs.
//! Qualified real GSUB and Arabic resources retain source, context and replay identity.
use super::*;

fn c(m: f64, b: f64, p: f64) -> t::InlineBoxEdgeComponents {
    t::InlineBoxEdgeComponents::from_f64(m, b, p).unwrap()
}
fn zero() -> t::InlineBoxEdgeComponents {
    c(0.0, 0.0, 0.0)
}
fn edge_context(p: &t::PreparedText) -> t::InlineBoxEdgeContext {
    let controls = p.control_views().map(|v| v.handle()).collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    let first = controls[0].captured_control().unwrap();
    let last = controls[1].captured_control().unwrap();
    let scope = first.scope();
    assert_eq!(scope.entry_control(), Some(first.clone()));
    assert_eq!(scope.exit_control(), Some(last.clone()));
    let owner = p.native_inline_box(first, last).unwrap();
    assert_eq!(owner, t::InlineBoxSource::captured(scope.clone()));
    p.inline_box_edge_context(
        owner,
        t::InlineBoxSource::captured(scope.parent().unwrap()),
        if p.analyzed_source().direction() == t::Direction::RightToLeft {
            t::LineEdgeSide::End
        } else {
            t::LineEdgeSide::Start
        },
    )
    .unwrap()
}
fn edge_facts(
    p: &t::PreparedText,
    parts: [t::InlineBoxEdgeComponents; 2],
) -> Vec<t::InlineBoundaryContextAllocation> {
    let context = edge_context(p);
    p.control_views()
        .zip(parts)
        .map(|(view, components)| t::InlineBoundaryContextAllocation {
            occurrence: view.handle(),
            boundary: t::InlineBoundaryAllocation::box_edge(context.clone(), components),
        })
        .collect()
}
fn set_selected(r: &mut t::LineSelectionRequest, facts: &[t::InlineBoundaryContextAllocation]) {
    for row in &mut r.controls {
        row.boundary = Some(
            facts
                .iter()
                .find(|fact| fact.occurrence == row.occurrence)
                .unwrap()
                .boundary
                .clone(),
        );
    }
}
fn answer_signed(
    p: &t::PreparedText,
    r: &mut t::LineSelectionRequest,
    facts: &[t::InlineBoundaryContextAllocation],
) {
    while let Some(needed) = p.boundary_context_demand(r).unwrap() {
        assert!(
            !r.boundary_context
                .iter()
                .any(|fact| fact.occurrence == needed)
        );
        r.boundary_context.push(
            facts
                .iter()
                .find(|fact| fact.occurrence == needed)
                .unwrap()
                .clone(),
        );
    }
}
fn whole_request(p: &t::PreparedText, width: f64) -> t::LineSelectionRequest {
    request(
        p,
        p.boundary_views().next().unwrap().handle(),
        paragraph_end(p),
        &[(0, 0.0), (1, 0.0)],
        width,
    )
}
fn signed_full(
    p: &t::PreparedText,
    parts: [t::InlineBoxEdgeComponents; 2],
    width: f64,
) -> t::FinalizedLine {
    let facts = edge_facts(p, parts);
    let mut r = whole_request(p, width);
    set_selected(&mut r, &facts);
    answer_signed(p, &mut r, &facts);
    finalize(p, r)
}
fn qualify_font(line: &t::FinalizedLine, bytes: &[u8]) {
    let word = |data: &[u8], at| u16::from_be_bytes(data[at..at + 2].try_into().unwrap());
    assert_eq!(word(font::table(bytes, *b"head"), 18), 1000);
    let hmtx = font::table(bytes, *b"hmtx");
    for glyph in [35, 58, 82, 100, 277] {
        assert_eq!(word(hmtx, glyph * 4), 1000);
    }
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), 10.0);
    }
}
fn paints(line: &t::FinalizedLine) -> Vec<(t::Range, u32, f32, f64)> {
    let mut rows = line
        .run_views()
        .flat_map(|run| {
            let x = run.origin().x;
            run.glyphs()
                .map(move |g| {
                    (
                        g.range(),
                        g.id(),
                        g.advance(),
                        x + g.pen() + f64::from(g.x()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| (row.0.start, row.0.end));
    rows
}

#[test]
fn zero_negative_and_cancelling_edges_select_real_between_tcu_ccmp_outcomes() {
    let bytes = font::ligature();
    let p = prepare_capture(
        t::Source::identified("AA", t::Id::from_u64(1635001), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1), (1, 2)],
        &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, 2)],
        &[(0, 1)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            control(1),
            E::ExitScope(1),
        ],
        &bytes,
    );
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [t::Range::new(0, 1), t::Range::new(1, 2)]
    );
    assert!(p.unit_views().all(|v| v.level().get() == 0));
    // Actual retained font.rs GSUB: DFLT/ccmp A35+A35->X58; UPEM1000,
    // hmtx35/58=1000. The owning font/table tests remain unchanged.
    let neutral = signed_full(&p, [zero(), zero()], 100.0);
    qualify_font(&neutral, &bytes);
    assert_eq!(paints(&neutral), [(t::Range::new(0, 2), 58, 10.0, 0.0)]);
    assert_eq!(neutral.feedback().advance, 10.0);
    for (component, second, flow, strip) in [
        (c(-5.0, 0.0, 0.0), 5.0, 15.0, (5.0, 5.0)),
        (c(-5.0, 2.0, 3.0), 10.0, 20.0, (5.0, 10.0)),
    ] {
        let line = signed_full(&p, [component, zero()], 100.0);
        qualify_font(&line, &bytes);
        assert_eq!(
            paints(&line),
            [
                (t::Range::new(0, 1), 35, 10.0, 0.0),
                (t::Range::new(1, 2), 35, 10.0, second)
            ]
        );
        assert_eq!(line.feedback().advance, flow);
        let start = p.control_views().next().unwrap().handle();
        let slot = line.control_slots().find(|v| v.control() == start).unwrap();
        assert_eq!(slot.flow_interval(), (10.0, second));
        assert_eq!(slot.strip_interval(), strip);
        let replay = line.with_spacing(t::SpacingAllocations::None).unwrap();
        assert_eq!(paints(&replay), paints(&line));
        assert_ne!(
            line.selected_line().request(),
            neutral.selected_line().request()
        );
    }
    assert_eq!(paints(&neutral), [(t::Range::new(0, 2), 58, 10.0, 0.0)]);
}

#[test]
fn nonzero_components_do_not_divide_the_real_cross_origin_grapheme_packet() {
    let bytes = font::ligature(); // this font maps acute769->35, NOT GPOS mark100
    let p = prepare_capture(
        t::Source::identified("A\u{301}B", t::Id::from_u64(1635002), 1),
        block(style(t::WhiteSpace::Preserve), false, false),
        &[(0, 1), (1, 3)],
        &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, 3)],
        &[(0, 1)],
        vec![
            origin(0),
            E::Connection(0),
            E::EnterScope(1),
            control(0),
            origin(1),
            control(1),
            E::ExitScope(1),
        ],
        &bytes,
    );
    assert_eq!(
        p.analyzed_source().graphemes().collect::<Vec<_>>(),
        [t::Range::new(0, 2), t::Range::new(2, 3)]
    );
    let start = p.control_views().next().unwrap().handle();
    assert!(!p.control_is_boundary_exact(&start).unwrap());
    for components in [zero(), c(-5.0, 0.0, 0.0), c(-5.0, 2.0, 3.0)] {
        let line = signed_full(&p, [components, zero()], 100.0);
        assert_eq!(
            ids(&line),
            [(t::Range::new(0, 2), 58), (t::Range::new(2, 3), 36)]
        );
        assert_eq!(
            line.run_views()
                .map(|run| run.glyphs().count())
                .sum::<usize>(),
            2
        );
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.font_size(), 10.0);
        }
        // Geometry is BoxEdge's actual whole-packet placement, not the old
        // Contribution interior-event pen. No newly invented EGC/caret cut.
        let first = line
            .run_views()
            .flat_map(|run| run.glyphs().map(|g| g.advance()).collect::<Vec<_>>())
            .next()
            .unwrap();
        assert_eq!(first, 10.0);
    }
}

#[test]
fn offselection_negative_or_cancelling_context_stops_real_arabic_joining_without_flow() {
    for rtl in [false, true] {
        let bytes = font::arabic();
        let p = prepare_capture(
            t::Source::new("\u{628}\u{628}\u{628}"),
            block(style(t::WhiteSpace::Preserve), rtl, true),
            &[(0, 1), (1, 2), (2, 3)],
            &[(K::InlineBoundaryStart, 1), (K::InlineBoundaryEnd, 2)],
            &[(0, 1)],
            vec![
                origin(0),
                E::Connection(0),
                E::EnterScope(1),
                control(0),
                origin(1),
                control(1),
                E::ExitScope(1),
                E::Connection(1),
                origin(2),
            ],
            &bytes,
        );
        let offers = soft_at(&p, 1);
        assert_eq!(offers.len(), 1);
        let make = || {
            request(
                &p,
                p.boundary_views().next().unwrap().handle(),
                offers[0].clone(),
                &[],
                100.0,
            )
        };
        let neutral_facts = edge_facts(&p, [zero(), zero()]);
        let mut r = make();
        assert!(r.controls.is_empty());
        answer_signed(&p, &mut r, &neutral_facts);
        assert_eq!(r.boundary_context.len(), 2);
        let neutral = finalize(&p, r);
        qualify_font(&neutral, &bytes);
        assert_eq!(ids(&neutral), [(t::Range::new(0, 1), 100)]); // actual init lookup
        for component in [c(-5.0, 0.0, 0.0), c(-5.0, 2.0, 3.0)] {
            let facts = edge_facts(&p, [component, zero()]);
            let mut r = make();
            answer_signed(&p, &mut r, &facts);
            assert_eq!(r.boundary_context.len(), 1);
            let material = finalize(&p, r);
            qualify_font(&material, &bytes);
            assert_eq!(ids(&material), [(t::Range::new(0, 1), 35)]); // actual isol lookup
            assert_eq!(material.feedback().advance, 10.0);
            assert_eq!(material.control_slots().count(), 0);
            assert_eq!(
                material.measurements().margin_edge_fit().authority(),
                t::MarginEdgeFitAuthority::NativeOnly,
                "context-only BoxEdge does not grant selected CSS flow authority"
            );
            for run in material.run_views() {
                assert_eq!(run.face().bytes(), bytes);
                assert_eq!(run.font_size(), 10.0);
            }
        }
        assert_eq!(ids(&neutral), [(t::Range::new(0, 1), 100)]);
    }
}

#[path = "signed_shaping/whitespace_hanging.rs"]
mod whitespace_hanging;

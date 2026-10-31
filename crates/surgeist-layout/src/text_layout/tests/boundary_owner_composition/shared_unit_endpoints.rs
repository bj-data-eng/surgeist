//! Exact physical pairs keep a shared marked unit whole and own their BP once.
use super::super::float_encounters::CachedHost;
use super::*;

const SOURCE: &str = "A\u{301}B";

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}

fn qualify_mark_tables(font: &[u8]) {
    assert_eq!(nominal(font, 35), 10.0);
    assert_eq!(nominal(font, 36), 10.0);
    let gdef = table(font, *b"GDEF");
    let classes = usize::from(u16_at(gdef, 4));
    assert_eq!(u16_at(gdef, classes), 2);
    assert_eq!(u16_at(gdef, classes + 2), 2);
    assert_eq!(
        (0..6)
            .map(|i| u16_at(gdef, classes + 4 + i * 2))
            .collect::<Vec<_>>(),
        [35, 35, 1, 100, 100, 3],
    );
    let gpos = table(font, *b"GPOS");
    let list = usize::from(u16_at(gpos, 8));
    assert_eq!(u16_at(gpos, list), 1);
    let lookup = list + usize::from(u16_at(gpos, list + 2));
    assert_eq!(u16_at(gpos, lookup), 4);
    assert_eq!(u16_at(gpos, lookup + 4), 1);
    let sub = lookup + usize::from(u16_at(gpos, lookup + 6));
    assert_eq!(u16_at(gpos, sub), 1);
    assert_eq!(u16_at(gpos, sub + 6), 1);
    for (coverage, glyph) in [(2, 100), (4, 35)] {
        let at = sub + usize::from(u16_at(gpos, sub + coverage));
        assert_eq!(
            (u16_at(gpos, at), u16_at(gpos, at + 2), u16_at(gpos, at + 4)),
            (1, 1, glyph)
        );
    }
    let marks = sub + usize::from(u16_at(gpos, sub + 8));
    assert_eq!((u16_at(gpos, marks), u16_at(gpos, marks + 2)), (1, 0));
    let mark = marks + usize::from(u16_at(gpos, marks + 4));
    let bases = sub + usize::from(u16_at(gpos, sub + 10));
    assert_eq!(u16_at(gpos, bases), 1);
    let base = bases + usize::from(u16_at(gpos, bases + 2));
    assert_eq!(
        (
            u16_at(gpos, mark),
            u16_at(gpos, mark + 2),
            u16_at(gpos, mark + 4)
        ),
        (1, 100, 200)
    );
    assert_eq!(
        (
            u16_at(gpos, base),
            u16_at(gpos, base + 2),
            u16_at(gpos, base + 4)
        ),
        (1, 400, 600)
    );
}

fn binding(
    font: &[u8],
    rtl: bool,
    endpoints: bool,
    interior: bool,
    nested: bool,
) -> TextParagraphBinding<u32> {
    capture(
        if interior {
            ["A", "\u{301}B"]
        } else {
            ["A\u{301}", "B"]
        },
        [
            style(text::WordBreak::Normal, text::LineBreak::Anywhere, false),
            style(text::WordBreak::KeepAll, text::LineBreak::Anywhere, false),
        ],
        font,
        nested,
        Wrap::Word,
        rtl,
        endpoints,
    )
}

fn qualify_source(p: &TextParagraphBinding<u32>, rtl: bool, nested: bool) {
    assert_eq!(
        p.prepared.analyzed_source().source().text().code_units(),
        [65, 769, 66]
    );
    assert_eq!(
        p.prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [text::Range::new(0, 2), text::Range::new(2, 3)]
    );
    for (index, origin) in p.origins.iter().enumerate() {
        assert_eq!(
            origin.origin.view().source().id(),
            Some(text::Id::from_u64(75001 + index as u64))
        );
        assert_eq!(origin.origin.view().source().revision(), 23);
    }
    // Plain Latin resolves L0 in an LTR paragraph and L2 in an RTL paragraph;
    // inherited marks retain the base level. This does not mirror the mark.
    let latin = if rtl { 2 } else { 0 };
    assert!(
        p.prepared
            .unit_views()
            .all(|unit| unit.level().get() == latin)
    );
    // Compare actual participant prefixes, not source/caret affinity at a
    // projected endpoint. No issued ending may separate the shared glyph unit.
    let participants = p.prepared.participants().collect::<Vec<_>>();
    let at = |range| {
        participants.iter().position(|participant| matches!(
        participant, text::PreparedParticipant::Text(unit) if unit.view().source_range() == range
    )).unwrap()
    };
    let base = at(text::Range::new(0, 1));
    let mark = at(text::Range::new(1, 2));
    let following = at(text::Range::new(2, 3));
    for offer in p.prepared.opportunity_views() {
        let end = offer.boundary().view().participant_offset();
        assert_eq!(base < end, mark < end);
        if offer.kind() == text::PreparedBreakKind::Allowed {
            assert!(mark < end && end <= following);
        }
    }
    let ordinary = p
        .prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::Allowed)
        .expect("actual Anywhere endpoint2 remains available")
        .handle();
    let start = p.prepared.boundary_views().next().unwrap().handle();
    let mut source_segments = p.prepared.source_segments();
    let source_segment = source_segments.next().expect("fixture source segment");
    assert_eq!(source_segments.count(), 0, "one structural fixture segment");
    let search = source_segment
        .conditional_search(
            &start,
            None,
            &ordinary,
            text::ConditionalPurpose::MinContent,
            &[],
            None,
        )
        .unwrap();
    assert!(search.phrase_portions().is_empty());
    assert!(
        search.begin(&[]).unwrap().opportunities().is_empty(),
        "no conditional endpoint may split A from its attached acute"
    );
    if p.controls.is_empty() {
        return;
    }
    if nested {
        assert_eq!(p.nodes, [100, 102, 1, 103, 104, 2, 105, 101]);
    } else {
        assert_eq!(p.nodes, [100, 1, 101, 102, 2, 103]);
    }
    assert_eq!(p.controls.len(), if nested { 6 } else { 4 });
    let members = p
        .prepared
        .captured_owner()
        .unwrap()
        .members()
        .collect::<Vec<_>>();
    assert_eq!(members.len(), 1);
    let captured = &members[0];
    let scopes = captured
        .scope_intersections()
        .map(|row| row.scope())
        .filter(|scope| scope.role() == text::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), if nested { 3 } else { 2 });
    let pairs: &[(u32, u32)] = if nested {
        &[(100, 101), (102, 103), (104, 105)]
    } else {
        &[(100, 101), (102, 103)]
    };
    for (scope, (start, end)) in scopes.iter().zip(pairs) {
        let endpoint = |node| p.controls.iter().find(|row| row.node == node).unwrap();
        assert_eq!(
            scope.entry_control(),
            endpoint(*start).occurrence.captured_control()
        );
        assert_eq!(
            scope.exit_control(),
            endpoint(*end).occurrence.captured_control()
        );
    }
    for row in &p.controls {
        assert_eq!(
            row.occurrence.view().kind(),
            if (row.node - 100).is_multiple_of(2) {
                &ParagraphControlKind::InlineBoundaryStart
            } else {
                &ParagraphControlKind::InlineBoundaryEnd
            }
        );
        let at = row.occurrence.view().source_position().get();
        assert_eq!(
            row.occurrence
                .captured_control()
                .unwrap()
                .source_position()
                .get(),
            at
        );
        assert_eq!(
            row.occurrence.view().level().get(),
            if at == 3 { u8::from(rtl) } else { latin }
        );
        assert_eq!(
            p.prepared
                .control_is_boundary_exact(&row.occurrence)
                .unwrap(),
            at != 1
        );
    }
}

fn prove_controls<S: LayoutScalar>(font: &[u8], rtl: bool) {
    qualify_mark_tables(font);
    for (endpoints, interior) in [(false, true), (true, false)] {
        let p = binding(font, rtl, endpoints, interior, false);
        qualify_source(&p, rtl, false);
        assert_pair::<S>(
            &p,
            font,
            SOURCE,
            Some(2),
            rtl,
            &[(35, 10.0), (100, 0.0), (36, 10.0)],
        );
        let whole = compute::<S>(&p, rtl, AvailableOf::MAX_CONTENT);
        let lines = qualify(
            &whole,
            &p,
            font,
            SOURCE,
            &[text::Range::new(0, 3)],
            &[(35, 10.0), (100, 0.0), (36, 10.0)],
        );
        assert_eq!(lines[0].feedback().baseline, 8.0);
        for run in lines[0].run_views() {
            let instance = run.effective_instance().unwrap();
            let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
            assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
        }
    }
}

fn check_geometry<S: LayoutScalar, Tree: LayoutTree<Node = u32, Scalar = S>>(
    input: &LayoutFlowInput<'_, Tree>,
    p: &TextParagraphBinding<u32>,
    font: &[u8],
    edges: (f64, f64),
    first_pair: (u32, u32),
    finite: [f64; 2],
    dirty: &[u32],
) -> Vec<CompletedLayoutBatchOf<u32, S>>
where
    Tree::MeasureError: std::fmt::Debug,
{
    let first = edges.0 + 10.0 + edges.1;
    let whole = first + 10.0;
    let mut packets = Vec::new();
    for (width, advances, ranges) in [
        (
            AvailableOf::MAX_CONTENT,
            vec![whole],
            vec![text::Range::new(0, 3)],
        ),
        (
            AvailableOf::MIN_CONTENT,
            vec![first, 10.0],
            vec![text::Range::new(0, 2), text::Range::new(2, 3)],
        ),
        (
            AvailableOf::definite(S::from_f64(finite[0])),
            vec![first, 10.0],
            vec![text::Range::new(0, 2), text::Range::new(2, 3)],
        ),
        (
            AvailableOf::definite(S::from_f64(finite[1])),
            vec![first, 10.0],
            vec![text::Range::new(0, 2), text::Range::new(2, 3)],
        ),
    ] {
        let batch = compute_layout_with_flow_invalidated(
            input,
            0,
            LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
            dirty,
        )
        .unwrap();
        let layout = batch.placed_text()[0].layout();
        assert_eq!(layout.source().text().code_units(), [65, 769, 66]);
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.range())
                .collect::<Vec<_>>(),
            ranges
        );
        let mut lines = vec![None; ranges.len()];
        for row in batch.final_inline_fragments() {
            let fragment = row.fragment();
            if matches!(
                fragment.finalized_fragment().view().owner(),
                text::FinalizedFragmentOwner::Control(_)
            ) {
                let view = fragment.finalized_fragment().view();
                assert!(view.origin().is_none());
                assert!(view.local_range().is_none());
                assert!(view.aggregate_range().is_none());
                assert!(view.processed_range().is_none());
            }
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), p.prepared);
            lines[fragment.line_index()] = Some(line);
        }
        let lines = lines.into_iter().map(Option::unwrap).collect::<Vec<_>>();
        let mut controls = Vec::new();
        let mut glyphs = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            let selected = line.selected_line();
            let request = selected.request();
            assert_eq!(line.feedback().advance, advances[index]);
            assert_eq!(line.measurements().fit().advance(), advances[index]);
            assert_eq!(
                (line.feedback().baseline, line.feedback().line_height),
                (8.0, 10.0)
            );
            assert_eq!(layout.lines()[index].baseline(), 8.0 + index as f64 * 10.0);
            assert!(request.alternative.is_none() && request.incoming.is_none());
            let last = request.participants.last().unwrap().clone();
            assert_eq!(
                line.prefix_at(&text::TextSeam::AfterParticipant(last))
                    .unwrap()
                    .advance(),
                advances[index]
            );
            for slot in line.control_slots() {
                controls.push(slot.control());
            }
            for run in line.run_views() {
                assert_eq!(run.face().bytes(), font);
                assert_eq!(run.face().index(), 0);
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                for glyph in run.glyphs() {
                    glyphs.push((glyph.id(), glyph.range(), glyph.advance()));
                }
            }
        }
        for control in &p.controls {
            assert_eq!(
                controls
                    .iter()
                    .filter(|actual| **actual == control.occurrence)
                    .count(),
                1
            );
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(
                    entries
                        .iter()
                        .filter(|row| row.node() == control.node)
                        .count(),
                    1
                );
            }
        }
        assert_eq!(controls.len(), p.controls.len());
        glyphs.sort_by_key(|glyph| glyph.0);
        assert_eq!(
            glyphs,
            [
                (35, text::Range::new(0, 2), 10.0),
                (36, text::Range::new(2, 3), 10.0),
                (100, text::Range::new(0, 2), 0.0),
            ]
        );
        let root = batch
            .unrounded_entries()
            .iter()
            .find(|row| row.node() == 0)
            .unwrap()
            .output();
        let expected_width = match width {
            AvailableOf::MinContent => first,
            AvailableOf::MaxContent => whole,
            AvailableOf::Definite(value) => value.to_f64(),
        };
        assert_eq!(
            root.size,
            Size::new(
                S::from_f64(expected_width),
                S::from_f64(ranges.len() as f64 * 10.0)
            )
        );
        if matches!(width, AvailableOf::MaxContent) {
            let start = edges.0;
            let positions = lines[0]
                .run_views()
                .flat_map(|run| {
                    let origin = run.origin().x;
                    run.glyphs()
                        .map(move |glyph| {
                            (
                                glyph.id(),
                                origin + glyph.pen() + f64::from(glyph.x()),
                                glyph.y(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert_eq!(
                positions,
                [(35, start, 0.0), (100, start + 3.0, -4.0), (36, first, 0.0)]
            );
            for (node, expected) in [
                (first_pair.0, (0.0, start)),
                (first_pair.1, (start + 10.0, first)),
            ] {
                let control = &p
                    .controls
                    .iter()
                    .find(|row| row.node == node)
                    .unwrap()
                    .occurrence;
                let slot = lines[0]
                    .control_slots()
                    .find(|slot| &slot.control() == control)
                    .unwrap();
                assert_eq!(slot.inline_bounds(), expected);
                for entries in [batch.unrounded_entries(), batch.final_entries()] {
                    let endpoint = entries
                        .iter()
                        .find(|row| row.node() == node)
                        .unwrap()
                        .output();
                    assert_eq!(
                        endpoint.size,
                        Size::new(S::from_f64(expected.1 - expected.0), S::ZERO)
                    );
                }
            }
        }
        let positions = lines[0]
            .run_views()
            .flat_map(|run| {
                let origin = run.origin().x;
                run.glyphs()
                    .map(move |glyph| {
                        (
                            glyph.id(),
                            origin + glyph.pen() + f64::from(glyph.x()),
                            glyph.y(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let base = positions.iter().find(|row| row.0 == 35).unwrap();
        let mark = positions.iter().find(|row| row.0 == 100).unwrap();
        assert_eq!((mark.1 - base.1, mark.2 - base.2), (3.0, -4.0));
        packets.push(batch);
    }
    packets
}

#[test]
fn semantic_and_true_unit_boundary_pairs_qualify_the_same_actual_mark_font() {
    let font = super::super::canonical::fonts::mark_ahem();
    for rtl in [false, true] {
        prove_controls::<f32>(&font, rtl);
        prove_controls::<f64>(&font, rtl);
    }
}

#[test]
fn physical_sibling_endpoints_inside_shared_mark_unit_admit_zero_and_positive_edges() {
    fn lane<S: LayoutScalar>() -> Vec<(bool, bool, Option<TextBindingError>)> {
        let font = super::super::canonical::fonts::mark_ahem();
        let mut results = Vec::new();
        for rtl in [false, true] {
            prove_controls::<S>(&font, rtl);
            for positive in [false, true] {
                let p = binding(&font, rtl, true, true, false);
                qualify_source(&p, rtl, false);
                assert_eq!(
                    p.origins[0].origin.view().local_range(),
                    text::Range::new(0, 1)
                );
                assert_eq!(
                    p.origins[1].origin.view().local_range(),
                    text::Range::new(0, 2)
                );
                let edges = if positive {
                    vec![
                        (
                            100,
                            InlineBoundaryEdgeOf::try_new(S::from_f64(1.0), S::from_f64(1.0))
                                .unwrap(),
                        ),
                        (
                            101,
                            InlineBoundaryEdgeOf::try_new(S::from_f64(1.0), S::from_f64(2.0))
                                .unwrap(),
                        ),
                    ]
                } else {
                    Vec::new()
                };
                let tree = tree::<S>(&p, rtl, &edges);
                let error = match LayoutFlowInput::try_new(
                    &tree,
                    0,
                    vec![p.clone()],
                    crate::WrappingInput {
                        scopes: Vec::new(),
                        child_order: Vec::new(),
                    },
                    Vec::new(),
                ) {
                    Ok(input) => {
                        let _packets = check_geometry(
                            &input,
                            &p,
                            &font,
                            if positive { (2.0, 3.0) } else { (0.0, 0.0) },
                            (100, 101),
                            if positive { [15.0, 14.0] } else { [10.0, 9.0] },
                            &[],
                        );
                        None
                    }
                    Err(error) => match error.kind() {
                        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::TextBinding(
                            reason,
                        )) => Some(*reason),
                        other => panic!("unexpected setup failure: {other:?}"),
                    },
                };
                results.push((rtl, positive, error));
            }
        }
        results
    }
    let actual = (lane::<f32>(), lane::<f64>());
    let expected = vec![
        (false, false, None),
        (false, true, None),
        (true, false, None),
        (true, true, None),
    ];
    assert_eq!(
        actual,
        (expected.clone(), expected),
        "genuine paired endpoint chronology must be accepted without weakening binding validation"
    );
}

#[test]
fn physical_nested_pairs_keep_the_mark_attached_and_count_each_edge_once() {
    fn lane<S: LayoutScalar>() {
        let font = super::super::canonical::fonts::mark_ahem();
        for rtl in [false, true] {
            prove_controls::<S>(&font, rtl);
            let p = binding(&font, rtl, true, true, true);
            qualify_source(&p, rtl, true);
            let edges = [
                (102, InlineBoundaryEdgeOf::try_new(S::ONE, S::ONE).unwrap()),
                (
                    103,
                    InlineBoundaryEdgeOf::try_new(S::ONE, S::from_f64(2.0)).unwrap(),
                ),
            ];
            let tree = tree::<S>(&p, rtl, &edges);
            let input = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![p.clone()],
                crate::WrappingInput {
                    scopes: Vec::new(),
                    child_order: Vec::new(),
                },
                Vec::new(),
            )
            .unwrap();
            let _packets =
                check_geometry(&input, &p, &font, (2.0, 3.0), (102, 103), [15.0, 14.0], &[]);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

// Own every observed value: cloning finalized handles alone would retain shared
// storage and would not detect mutation of a previously completed packet.
fn snapshot<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
) -> impl std::fmt::Debug + PartialEq + use<S> {
    let fragments = |rows: &[InlineFragmentOutputEntryOf<u32, S>]| {
        rows.iter()
            .map(|row| {
                let fragment = row.fragment();
                (
                    row.owner().clone(),
                    fragment.rect(),
                    fragment.baseline(),
                    fragment.line_index(),
                )
            })
            .collect::<Vec<_>>()
    };
    let layout = batch.placed_text()[0].layout();
    let lines = layout
        .lines()
        .iter()
        .map(|line| (line.range(), line.bounds(), line.baseline()))
        .collect::<Vec<_>>();
    let positions = layout
        .positions()
        .map(|position| {
            (
                position.source_cursor(),
                layout.position_geometry(&position).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let glyphs = layout
        .glyph_runs()
        .into_iter()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .iter()
                .map(move |glyph| {
                    (
                        glyph.id(),
                        glyph.range(),
                        glyph.advance(),
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let selected = batch
        .final_inline_fragments()
        .iter()
        .map(|row| {
            let line = row.fragment().finalized_fragment().finalized_line();
            let controls = line
                .control_slots()
                .map(|slot| (slot.control(), slot.inline_bounds(), slot.slot()))
                .collect::<Vec<_>>();
            (line.selected_line().request().clone(), controls)
        })
        .collect::<Vec<_>>();
    (
        batch.unrounded_entries().to_vec(),
        batch.final_entries().to_vec(),
        fragments(batch.unrounded_inline_fragments()),
        fragments(batch.final_inline_fragments()),
        layout.source().text().code_units().to_vec(),
        lines,
        positions,
        glyphs,
        selected,
    )
}

fn cached_packets<S: LayoutScalar>(
    host: &CachedHost<S>,
    p: &TextParagraphBinding<u32>,
    font: &[u8],
    end: f64,
    dirty: &[u32],
) -> Vec<CompletedLayoutBatchOf<u32, S>> {
    let input = LayoutFlowInput::try_new(
        host,
        0,
        vec![p.clone()],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    check_geometry(&input, p, font, (2.0, end), (100, 101), [15.0, 14.0], dirty)
}

#[test]
fn changed_end_edge_reuses_preparation_and_matches_fresh_current_packets() {
    fn lane<S: LayoutScalar>() {
        let font = super::super::canonical::fonts::mark_ahem();
        prove_controls::<S>(&font, false);
        let p = binding(&font, false, true, true, false);
        qualify_source(&p, false, false);
        let preparation = p.prepared.clone();
        let edges = [
            (100, InlineBoundaryEdgeOf::try_new(S::ONE, S::ONE).unwrap()),
            (
                101,
                InlineBoundaryEdgeOf::try_new(S::ONE, S::from_f64(2.0)).unwrap(),
            ),
        ];
        let mut host = CachedHost::new(tree::<S>(&p, false, &edges));
        let old = cached_packets(&host, &p, &font, 3.0, &[]);
        let old_facts = old.iter().map(snapshot).collect::<Vec<_>>();
        old[0].apply_to(&mut host).unwrap();
        let warm_old = cached_packets(&host, &p, &font, 3.0, &[]);
        assert_eq!(warm_old.iter().map(snapshot).collect::<Vec<_>>(), old_facts);
        warm_old[0].apply_to(&mut host).unwrap();

        let before = host.tree.layout_input(101).as_inline_boundary().unwrap();
        assert_eq!(
            before.edge(),
            InlineBoundaryEdgeOf::try_new(S::ONE, S::from_f64(2.0)).unwrap()
        );
        let changed_input = InlineBoundaryInputOf::new(
            before.kind(),
            before.metrics(),
            InlineBoundaryEdgeOf::try_new(S::ONE, S::from_f64(4.0)).unwrap(),
        )
        .with_writing_mode(before.writing_mode())
        .with_direction(before.direction())
        .with_vertical_align(before.vertical_align());
        host.tree
            .insert_input(101, LayoutInputOf::inline_boundary(changed_input));
        let changed = cached_packets(&host, &p, &font, 5.0, &[101]);
        assert!(
            changed
                .iter()
                .all(|packet| packet.invalidated_nodes().contains(&101))
        );
        let changed_facts = changed.iter().map(snapshot).collect::<Vec<_>>();
        let cold_host = CachedHost::new(host.tree.clone());
        let fresh = cached_packets(&cold_host, &p, &font, 5.0, &[]);
        assert_eq!(
            fresh.iter().map(snapshot).collect::<Vec<_>>(),
            changed_facts
        );
        changed[0].apply_to(&mut host).unwrap();
        let warm_current = cached_packets(&host, &p, &font, 5.0, &[]);
        assert_eq!(
            warm_current.iter().map(snapshot).collect::<Vec<_>>(),
            changed_facts
        );
        assert_eq!(p.prepared, preparation);
        assert_eq!(old.iter().map(snapshot).collect::<Vec<_>>(), old_facts);
    }
    lane::<f32>();
    lane::<f64>();
}

//! Exact physical pairs keep a shared marked unit whole and own their BP once.
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

fn binding(font: &[u8], rtl: bool, endpoints: bool, interior: bool) -> TextParagraphBinding<u32> {
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
        false,
        Wrap::Word,
        rtl,
        endpoints,
    )
}

fn qualify_source(p: &TextParagraphBinding<u32>, rtl: bool) {
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
    let search = p
        .prepared
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
    assert_eq!(p.nodes, [100, 1, 101, 102, 2, 103]);
    assert_eq!(p.controls.len(), 4);
    let captured = p.prepared.captured_paragraph().unwrap();
    let scopes = captured
        .scope_intersections()
        .map(|row| row.scope())
        .filter(|scope| scope.role() == text::ScopeRole::Inline)
        .collect::<Vec<_>>();
    assert_eq!(scopes.len(), 2);
    for (scope, first) in scopes.iter().zip([0, 2]) {
        assert_eq!(
            scope.entry_control(),
            p.controls[first].occurrence.captured_control()
        );
        assert_eq!(
            scope.exit_control(),
            p.controls[first + 1].occurrence.captured_control()
        );
    }
    for (index, row) in p.controls.iter().enumerate() {
        assert_eq!(
            row.occurrence.view().kind(),
            if index % 2 == 0 {
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
        let p = binding(font, rtl, endpoints, interior);
        qualify_source(&p, rtl);
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

fn check_geometry<S: LayoutScalar>(
    input: &LayoutFlowInput<'_, PublicLayoutTreeOf<S>>,
    p: &TextParagraphBinding<u32>,
    font: &[u8],
    positive: bool,
) {
    let first = if positive { 15.0 } else { 10.0 };
    let whole = first + 10.0;
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
            AvailableOf::definite(S::from_f64(first)),
            vec![first, 10.0],
            vec![text::Range::new(0, 2), text::Range::new(2, 3)],
        ),
        (
            AvailableOf::definite(S::from_f64(first - 1.0)),
            vec![first, 10.0],
            vec![text::Range::new(0, 2), text::Range::new(2, 3)],
        ),
    ] {
        let batch = compute_layout_with_flow(
            input,
            0,
            LayoutRootRequestOf::viewport(Size::new(width, AvailableOf::MAX_CONTENT)).unwrap(),
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
        assert_eq!(controls.len(), 4);
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
            let start = if positive { 2.0 } else { 0.0 };
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
            for (node, expected) in [(100, (0.0, start)), (101, (start + 10.0, first))] {
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
    }
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
                let p = binding(&font, rtl, true, true);
                qualify_source(&p, rtl);
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
                ) {
                    Ok(input) => {
                        check_geometry(&input, &p, &font, positive);
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

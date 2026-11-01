//! Parent-side box edges surround real shared mark and GSUB packets once.
//!
//! The mark case has C.End between base A and an attached acute in a different
//! original/sibling scope. The maintained ccmp AA->X case spans two EGCs within C.
//! Both retain actual P/C/D nominations, BoxEdge P/C and Contribution0 for D.
//! Formatter fields remain B; C's separate parent basis preserves Source levels.

use super::*;

fn endpoint(p: &TextParagraphBinding<u32>, node: u32) -> text::PreparedControl {
    p.controls
        .iter()
        .find(|row| row.node == node)
        .unwrap()
        .occurrence
        .clone()
}

fn used_direction(rtl: bool) -> Direction {
    if rtl { Direction::Rtl } else { Direction::Ltr }
}

fn parent_axes(rtl: bool) -> FlowAxes {
    FlowAxes::new(WritingMode::HorizontalTb, used_direction(rtl))
}

fn packet_tree<S: LayoutScalar>(p: &TextParagraphBinding<u32>, rtl: bool) -> PublicLayoutTreeOf<S> {
    // Reuse actual native text participation, metrics, flattened child order
    // and Block input. The unrelated D endpoints retain Native Contribution0.
    let mut tree = tree::<S>(p, rtl, &[]);
    for row in &p.controls {
        let LayoutInputViewOf::InlineBoundary(old) = tree.layout_input(row.node) else {
            panic!("maintained capture/tree issues authentic endpoint inputs")
        };
        assert_eq!(old.direction(), used_direction(rtl));
        assert_eq!(old.writing_mode(), WritingMode::HorizontalTb);
        if matches!(row.node, 104 | 105) {
            assert_eq!(old.parent_edge_axes(), None);
            continue;
        }
        let (amount, immediate_parent_rtl) = match row.node {
            100 => (1.0, rtl),
            101 => (4.0, rtl),
            102 => (2.0, !rtl),
            103 => (3.0, !rtl),
            _ => unreachable!("nested helper owns only P/C/D endpoints"),
        };
        let input = InlineBoundaryInputOf::box_edge(
            old.kind(),
            old.metrics(),
            InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(amount), S::ZERO).unwrap(),
            parent_axes(immediate_parent_rtl),
        )
        .with_writing_mode(old.writing_mode())
        .with_direction(old.direction())
        .with_vertical_align(old.vertical_align());
        assert_eq!(
            input.direction(),
            used_direction(rtl),
            "old formatter fields remain B"
        );
        assert_eq!(
            input.parent_edge_axes(),
            Some(parent_axes(immediate_parent_rtl))
        );
        tree = tree.input(row.node, LayoutInputOf::inline_boundary(input));
    }
    tree
}

fn family(p: &TextParagraphBinding<u32>) -> WrappingInput<u32> {
    let scopes: Vec<_> = [100, 102, 104]
        .map(|node| endpoint(p, node).captured_control().unwrap().scope())
        .into_iter()
        .collect();
    let [outer, before, after] = scopes.as_slice() else {
        unreachable!()
    };
    assert_eq!(before.parent(), Some(outer.clone()));
    assert_eq!(after.parent(), Some(outer.clone()));
    assert_eq!(outer.parent().unwrap().role(), text::ScopeRole::Domain);
    for (scope, (start, end)) in scopes.iter().zip([(100, 101), (102, 103), (104, 105)]) {
        let first = endpoint(p, start).captured_control().unwrap();
        let last = endpoint(p, end).captured_control().unwrap();
        assert_eq!(scope.entry_control(), Some(first.clone()));
        assert_eq!(scope.exit_control(), Some(last.clone()));
        assert_eq!(
            p.prepared.native_inline_box(first, last).unwrap(),
            text::InlineBoxSource::captured(scope.clone())
        );
    }
    WrappingInput {
        gaps: Vec::new(),
        scopes: scopes
            .into_iter()
            .map(|scope| WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            })
            .collect(),
        child_order: Vec::new(),
    }
}

fn execute<S: LayoutScalar>(
    p: &TextParagraphBinding<u32>,
    rtl: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let tree = packet_tree::<S>(p, rtl);
    let input = LayoutFlowInput::try_new(&tree, 0, vec![p.clone()], family(p), Vec::new()).unwrap();
    for (start, end) in [(100, 101), (102, 103), (104, 105)] {
        let scope = endpoint(p, start).captured_control().unwrap().scope();
        assert_eq!(
            input.associations.captured_binding(&scope),
            Some(TextScopeAssociation::Inline { start, end })
        );
    }
    compute_layout_with_flow(
        &input,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::MAX_CONTENT,
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn actual_line<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> text::FinalizedLine {
    let mut lines = Vec::new();
    for fragment in batch.final_inline_fragments() {
        let line = fragment.fragment().finalized_fragment().finalized_line();
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    assert_eq!(lines.len(), 1);
    lines.pop().unwrap()
}

fn glyph_rows(line: &text::FinalizedLine, font: &[u8]) -> Vec<(u32, f32, f64, f32, text::Range)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            let instance = run.effective_instance().unwrap();
            let metrics = instance.metrics(run.face().catalogue_identity()).unwrap();
            assert_eq!(
                (metrics.units_per_em(), metrics.ascent(), metrics.descent()),
                (1000, 8.0, -2.0)
            );
            let origin = run.origin().x;
            run.glyphs()
                .map(move |glyph| {
                    (
                        glyph.id(),
                        glyph.advance(),
                        origin + glyph.pen() + f64::from(glyph.x()),
                        glyph.y(),
                        glyph.range(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn strip<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &TextParagraphBinding<u32>,
    node: u32,
    lo: f64,
    hi: f64,
    right: bool,
) {
    let row = batch
        .unrounded_entries()
        .iter()
        .find(|row| row.node() == node)
        .unwrap()
        .output();
    assert_eq!(
        (row.location.x, row.size.width),
        (S::from_f64(lo), S::from_f64(hi - lo))
    );
    assert_eq!(
        row.border,
        if right {
            Edges::new(S::ZERO, S::from_f64(hi - lo), S::ZERO, S::ZERO)
        } else {
            Edges::new(S::ZERO, S::ZERO, S::ZERO, S::from_f64(hi - lo))
        }
    );
    assert_eq!(row.padding, Edges::ZERO);
    let occurrence = endpoint(p, node);
    let line = actual_line(batch);
    assert_eq!(
        line.control_slots()
            .find(|slot| slot.control() == occurrence)
            .unwrap()
            .strip_interval(),
        (lo, hi)
    );
    let fragments: Vec<_> = batch
        .unrounded_inline_fragments()
        .iter()
        .filter(|entry| matches!(entry.owner(),InlineFragmentOwner::Node(n) if *n==node))
        .collect();
    assert_eq!(fragments.len(), 1);
    assert!(matches!(
        fragments[0].fragment().finalized_fragment().view().owner(),
        text::FinalizedFragmentOwner::Control(_)
    ));
    let rect = fragments[0].fragment().rect().unwrap();
    assert_eq!(
        (rect.origin().x, rect.size().width, rect.size().height),
        (S::from_f64(lo), S::from_f64(hi - lo), S::from_f64(10.0))
    );
}

fn surround<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &TextParagraphBinding<u32>,
    rtl: bool,
) {
    let line = actual_line(batch);
    assert_eq!(line.prepared_text(), p.prepared);
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (30.0, 8.0, 10.0)
    );
    assert_eq!(line.measurements().fit().advance(), 30.0);
    assert_eq!(
        line.measurements().max_content_contribution().advance(),
        30.0
    );
    assert_eq!(line.control_slots().count(), 6);
    for row in &p.controls {
        assert_eq!(
            line.control_slots()
                .filter(|slot| slot.control() == row.occurrence)
                .count(),
            1
        );
    }
    // Non-endpoint payload order is still shared packet then B, also in B RTL:
    // the actual strong-L payloads have resolved level2, not mirrored glyphs.
    if rtl {
        for (node, lo, hi, right) in [
            (101, 0.0, 4.0, false),
            (102, 4.0, 6.0, false),
            (103, 16.0, 19.0, true),
            (100, 29.0, 30.0, true),
        ] {
            strip(batch, p, node, lo, hi, right);
        }
    } else {
        for (node, lo, hi, right) in [
            (100, 0.0, 1.0, false),
            (103, 1.0, 4.0, false),
            (102, 14.0, 16.0, true),
            (101, 26.0, 30.0, true),
        ] {
            strip(batch, p, node, lo, hi, right);
        }
    }
}

fn mark<S: LayoutScalar>(rtl: bool) {
    let font = super::super::super::canonical::fonts::mark_ahem();
    // Literal GDEF classes and GPOS anchors (400,600)-(100,200), UPEM1000,
    // size10 imply mark relative (+3,-4), raw packet advance10.
    qualify_mark_tables(&font);
    let p = binding(&font, rtl, true, true, true);
    qualify_source(&p, rtl, true);
    assert_eq!(
        p.origins[0].origin.view().local_range(),
        text::Range::new(0, 1)
    );
    assert_eq!(
        p.origins[1].origin.view().local_range(),
        text::Range::new(0, 2)
    );
    assert_eq!(
        p.origins[0].origin.view().source().text().code_units(),
        [65]
    );
    assert_eq!(
        p.origins[1].origin.view().source().text().code_units(),
        [769, 66]
    );
    assert_eq!(endpoint(&p, 103).view().source_position().get(), 1);
    assert!(
        !p.prepared
            .control_is_boundary_exact(&endpoint(&p, 103))
            .unwrap()
    );
    let original_levels: Vec<_> = p
        .prepared
        .control_views()
        .map(|row| (row.handle(), row.level()))
        .collect();

    let baseline = compute::<S>(&p, rtl, AvailableOf::MAX_CONTENT);
    let ordinary = qualify(
        &baseline,
        &p,
        &font,
        SOURCE,
        &[text::Range::new(0, 3)],
        &[(35, 10.0), (100, 0.0), (36, 10.0)],
    )
    .remove(0);
    assert_eq!(
        glyph_rows(&ordinary, &font),
        [
            (35, 10.0, 0.0, 0.0, text::Range::new(0, 2)),
            (100, 0.0, 3.0, -4.0, text::Range::new(0, 2)),
            (36, 10.0, 10.0, 0.0, text::Range::new(2, 3)),
        ]
    );

    let batch = execute::<S>(&p, rtl);
    surround(&batch, &p, rtl);
    let line = actual_line(&batch);
    let (base, b) = if rtl { (6.0, 19.0) } else { (4.0, 16.0) };
    // End103's actual source event is inside the cross-origin EGC. Moving its
    // BoxEdge to the parent's allocated physical side does NOT detach, duplicate
    // or place acute at an endpoint pen. The entire real provider packet moves.
    let expected = [
        (35, 10.0, base, 0.0, text::Range::new(0, 2)),
        (100, 0.0, base + 3.0, -4.0, text::Range::new(0, 2)),
        (36, 10.0, b, 0.0, text::Range::new(2, 3)),
    ];
    assert_eq!(glyph_rows(&line, &font), expected);
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(glyph_rows(&replay, &font), expected);
    assert_eq!(
        p.prepared
            .control_views()
            .map(|row| (row.handle(), row.level()))
            .collect::<Vec<_>>(),
        original_levels
    );
    assert_eq!(
        glyph_rows(&ordinary, &font)[0].2,
        0.0,
        "prior real packet remains immutable"
    );
}

fn qualify_ccmp_tables(font: &[u8]) {
    // Maintained canonical::fonts::ligature_ahem authors ccmp, lookup4/format1:
    // coverageA35 + second componentA35 -> genuine resultX58, hmtx1000.
    assert_eq!(nominal(font, 35), 10.0);
    assert_eq!(nominal(font, 58), 10.0);
    assert_eq!(nominal(font, 36), 10.0);
    let gsub = table(font, *b"GSUB");
    let features = usize::from(u16_at(gsub, 6));
    assert_eq!(u16_at(gsub, features), 1);
    assert_eq!(&gsub[features + 2..features + 6], b"ccmp");
    let list = usize::from(u16_at(gsub, 8));
    assert_eq!(u16_at(gsub, list), 1);
    let lookup = list + usize::from(u16_at(gsub, list + 2));
    assert_eq!(u16_at(gsub, lookup), 4);
    assert_eq!(u16_at(gsub, lookup + 4), 1);
    let sub = lookup + usize::from(u16_at(gsub, lookup + 6));
    assert_eq!(
        (0..12)
            .map(|i| u16_at(gsub, sub + 2 * i))
            .collect::<Vec<_>>(),
        [1, 18, 1, 8, 1, 4, 58, 2, 35, 1, 1, 35]
    );
}

fn ccmp<S: LayoutScalar>(rtl: bool) {
    let font = super::super::super::canonical::fonts::ligature_ahem();
    qualify_ccmp_tables(&font);
    // The maintained constructor issues P around actual sibling C/D. AA stays
    // within C's ONE genuine origin, so its new nonzero exterior edges do not
    // invent an interior shaping separation. No new font/capture is authored.
    let p = capture(
        ["AA", "B"],
        [
            style(text::WordBreak::Normal, text::LineBreak::Anywhere, false),
            style(text::WordBreak::KeepAll, text::LineBreak::Anywhere, false),
        ],
        &font,
        true,
        Wrap::Word,
        rtl,
        true,
    );
    assert_eq!(
        p.prepared.analyzed_source().source().text().code_units(),
        [65, 65, 66]
    );
    assert_eq!(
        p.prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [
            text::Range::new(0, 1),
            text::Range::new(1, 2),
            text::Range::new(2, 3)
        ]
    );
    for (i, raw, aggregate) in [
        (0, vec![65, 65], text::Range::new(0, 2)),
        (1, vec![66], text::Range::new(2, 3)),
    ] {
        let origin = &p.origins[i].origin;
        assert_eq!(
            origin.view().source().id(),
            Some(text::Id::from_u64(75001 + i as u64))
        );
        assert_eq!(origin.view().source().revision(), 23);
        assert_eq!(origin.view().source().text().code_units(), raw);
        assert_eq!(origin.view().local_range(), text::Range::new(0, raw.len()));
        assert_eq!(origin.view().aggregate_range(), aggregate);
    }
    assert!(
        p.prepared
            .unit_views()
            .all(|unit| unit.level().get() == if rtl { 2 } else { 0 })
    );
    let levels: Vec<_> = p
        .prepared
        .control_views()
        .map(|row| (row.handle(), row.level()))
        .collect();
    let baseline = compute::<S>(&p, rtl, AvailableOf::MAX_CONTENT);
    let ordinary = qualify(
        &baseline,
        &p,
        &font,
        "AAB",
        &[text::Range::new(0, 3)],
        &[(58, 10.0), (36, 10.0)],
    )
    .remove(0);
    assert_eq!(
        glyph_rows(&ordinary, &font),
        [
            (58, 10.0, 0.0, 0.0, text::Range::new(0, 2)),
            (36, 10.0, 10.0, 0.0, text::Range::new(2, 3)),
        ]
    );
    let batch = execute::<S>(&p, rtl);
    surround(&batch, &p, rtl);
    let line = actual_line(&batch);
    let (base, b) = if rtl { (6.0, 19.0) } else { (4.0, 16.0) };
    let expected = [
        (58, 10.0, base, 0.0, text::Range::new(0, 2)),
        (36, 10.0, b, 0.0, text::Range::new(2, 3)),
    ];
    assert_eq!(glyph_rows(&line, &font), expected);
    assert_eq!(
        line.run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        2
    );
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(glyph_rows(&replay, &font), expected);
    assert_eq!(
        p.prepared
            .control_views()
            .map(|row| (row.handle(), row.level()))
            .collect::<Vec<_>>(),
        levels
    );
    assert_eq!(glyph_rows(&ordinary, &font)[0].2, 0.0);
}

#[test]
fn parent_side_box_edges_surround_one_real_cross_origin_mark_packet() {
    for rtl in [false, true] {
        mark::<f32>(rtl);
        mark::<f64>(rtl);
    }
}

#[test]
fn parent_side_box_edges_keep_one_real_two_egc_ccmp_packet_once() {
    for rtl in [false, true] {
        ccmp::<f32>(rtl);
        ccmp::<f64>(rtl);
    }
}

// Nominated scopes retain each shared packet once. The ccmp case spans two EGCs
// within C; the mark case supplies an interior endpoint across original and
// sibling scopes. Neither inserts a shaping separation inside the packet.

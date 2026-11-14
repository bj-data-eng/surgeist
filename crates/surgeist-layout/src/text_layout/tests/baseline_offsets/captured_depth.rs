use super::*;
use crate::engine::depth_tests::on_bounded_stack;

fn assert_captured_named_chain<S: LayoutScalar>(depth: u32, source: BaselineSource) {
    premises();
    // Ahem X (glyph 58) has the same 800-unit ink top as its ascender.
    // Check the retained outline, independently of the placed glyph positions.
    assert_eq!(word(table(*b"head"), 50), 0);
    let glyph = usize::from(word(table(*b"loca"), 58 * 2)) * 2;
    let outline = table(*b"glyf");
    assert_eq!(
        (1..=4)
            .map(|index| i16::from_be_bytes(outline[glyph + 2 * index..][..2].try_into().unwrap()))
            .collect::<Vec<_>>(),
        [0, -200, 1000, 800]
    );

    let width = |node| 10.0 * f64::from(depth + 2 - node);
    let mut tree = PublicLayoutTreeOf::<S>::new();
    let mut paragraphs = Vec::new();
    for node in 0..=depth {
        let text = depth + 1 + node;
        let boxes = if node == depth {
            Vec::new()
        } else {
            vec![(node + 1, width(node + 1), 20.0)]
        };
        let paragraph = native(
            if node == depth { "X\nX" } else { "X" },
            10.0,
            10.0,
            WritingMode::HorizontalTb,
            &boxes,
        );
        assert_eq!(
            paragraph.prepared.baseline_reference(),
            Some(text::FontBaseline::Alphabetic)
        );
        let children = if node == depth {
            vec![text]
        } else {
            vec![text, node + 1]
        };
        tree = tree
            .children(node, children)
            .children(text, [])
            .style(
                node,
                NodeInputOf {
                    display: if node == 0 {
                        Display::Block
                    } else {
                        Display::InlineBlock
                    },
                    inline_baseline_context: InlineBaselineContext::Named(BaselineName::Alphabetic),
                    baseline_source: source,
                    vertical_align: VerticalAlignOf::Baseline,
                    size: Size::new(px(width(node)), px(20.0)),
                    ..NodeInputOf::default()
                },
            )
            .input(text, paragraph.input());
        paragraphs.push(paragraph);
    }
    let bindings = paragraphs
        .iter()
        .enumerate()
        .map(|(node, paragraph)| {
            let node = u32::try_from(node).unwrap();
            paragraph.binding(node, depth + 1 + node)
        })
        .collect();
    let batch = solve(&tree, WritingMode::HorizontalTb, bindings)
        .expect("captured named atomic preparation returns on the caller stack");
    assert_eq!(batch.node_layouts().len(), 2 * (depth + 1) as usize);
    assert_eq!(batch.placed_text().len(), (depth + 1) as usize);

    // Preserve+nowrap admits no soft cut. A nonterminal line is X followed
    // by a 20-high child. First selects 8 (8 before/12 after), Last selects 18
    // (18 before/2 after), so both envelopes remain 20 and child block offset 0.
    // The terminal's preserved LF is the only forced cut: its ports are 8/18.
    let selected = if source == BaselineSource::First {
        8.0
    } else {
        18.0
    };
    for node in 0..=depth {
        let actual = output(&batch, node);
        assert_eq!(
            actual.size,
            Size::new(n(width(node)), n(20.0)),
            "node {node}"
        );
        assert_eq!(
            actual.location,
            Point::new(n(if node == 0 { 0.0 } else { 10.0 }), S::ZERO),
            "node {node}"
        );
        assert_eq!(
            actual.source_index,
            crate::SourceIndex::new(usize::from(node != 0)),
            "node {node}"
        );
        let pair = actual.named_baselines.get(BaselineName::Alphabetic);
        let (first, last) = if node == depth {
            (8.0, 18.0)
        } else {
            (selected, selected)
        };
        fact(pair.first(), PhysicalAxis::Vertical, first);
        fact(pair.last(), PhysicalAxis::Vertical, last);

        let text_node = depth + 1 + node;
        let text_top = if node == depth { 0.0 } else { selected - 8.0 };
        for phase in [Phase::Unrounded, Phase::Final] {
            let actual = readback::observe(
                &batch,
                text_node,
                phase,
                readback::Kind::NonBox(crate::NonBoxKind::InlineText),
            );
            assert_eq!(actual.source_index, crate::SourceIndex::ZERO);
            assert_eq!(actual.location, Point::new(S::ZERO, n(text_top)));
            let text_size = Size::new(n(10.0), n(if node == depth { 20.0 } else { 10.0 }));
            assert_eq!(actual.size, text_size);
            assert_eq!(actual.content_size, text_size);
        }

        // Text retains each selected source unit. The preserved LF is a
        // distinct zero-advance anchor after the first X, with its actual line
        // metric band; it neither disappears nor becomes an extra line.
        let expected_fragments = if node == depth {
            vec![
                (text::Range::new(0, 1), 0, (0.0, 0.0, 10.0, 10.0), 8.0),
                (text::Range::new(1, 2), 0, (10.0, 0.0, 0.0, 10.0), 8.0),
                (text::Range::new(2, 3), 1, (0.0, 10.0, 10.0, 10.0), 18.0),
            ]
        } else {
            vec![(
                text::Range::new(0, 1),
                0,
                (0.0, text_top, 10.0, 10.0),
                selected,
            )]
        };
        let origin = paragraphs[node as usize]
            .prepared
            .origin_views()
            .next()
            .unwrap()
            .handle();
        for fragments in [
            batch.unrounded_inline_fragments(),
            batch.final_inline_fragments(),
        ] {
            let fragments = fragments
                .iter()
                .filter(|row| *row.owner() == crate::InlineFragmentOwner::Node(text_node))
                .collect::<Vec<_>>();
            assert_eq!(fragments.len(), expected_fragments.len(), "node {node}");
            for (row, (range, line_index, (x, y, width, height), baseline)) in
                fragments.iter().zip(&expected_fragments)
            {
                let fragment = row.fragment();
                let issued = fragment.finalized_fragment();
                assert_eq!(
                    issued.view().owner(),
                    text::FinalizedFragmentOwner::Origin(origin.clone())
                );
                assert_eq!(issued.view().local_range(), Some(*range));
                assert_eq!(issued.view().aggregate_range(), Some(*range));
                assert_eq!(
                    issued
                        .finalized_line()
                        .prepared_text()
                        .analyzed_source()
                        .identity(),
                    paragraphs[node as usize]
                        .prepared
                        .analyzed_source()
                        .identity()
                );
                assert_eq!(fragment.line_index(), *line_index);
                let rect = fragment.rect().unwrap();
                assert_eq!(rect.origin(), Point::new(n(*x), n(*y)));
                assert_eq!(rect.size(), Size::new(n(*width), n(*height)));
                assert_eq!(fragment.baseline(), Point::new(S::ZERO, n(*baseline)));
            }
        }

        let layout = placed(&batch, node);
        let expected_baselines = if node == depth {
            vec![8.0, 18.0]
        } else {
            vec![selected]
        };
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            expected_baselines,
            "node {node}"
        );
        let mut glyph_tops = Vec::new();
        for run in layout.run_views() {
            let origin = run.origin();
            for glyph in run.glyphs() {
                glyph_tops.push(text::Point::new(
                    origin.x + glyph.pen() + f64::from(glyph.x()),
                    origin.y + f64::from(glyph.y()) - 8.0,
                ));
            }
        }
        // Placed Text retains absolute anchors; source fragments and Node
        // outputs above retain their actual paragraph/parent local frames.
        let absolute_x = 10.0 * f64::from(node);
        let expected_tops = if node == depth {
            vec![
                text::Point::new(absolute_x, 0.0),
                text::Point::new(absolute_x, 10.0),
            ]
        } else {
            vec![text::Point::new(absolute_x, selected - 8.0)]
        };
        assert_eq!(glyph_tops, expected_tops, "node {node}");
        if node < depth {
            assert_eq!(layout.inline_boxes().len(), 1);
            assert_eq!(
                layout.inline_boxes()[0].rect(),
                text::Rect::new(absolute_x + 10.0, 0.0, width(node + 1), 20.0)
            );
        } else {
            assert!(layout.inline_boxes().is_empty());
        }
        raw(layout, 10.0, 10.0);
        custody(layout, &paragraphs[node as usize].original);
    }
}

#[test]
fn shallow_captured_named_atomics_preserve_both_baseline_sources() {
    on_bounded_stack(
        "text_layout::tests::baseline_offsets::captured_depth::shallow_captured_named_atomics_preserve_both_baseline_sources",
        || {
            for source in [BaselineSource::First, BaselineSource::Last] {
                assert_captured_named_chain::<f32>(2, source);
                assert_captured_named_chain::<f64>(2, source);
            }
        },
    );
}

#[test]
fn deep_captured_named_atomics_preserve_both_baseline_sources_f32() {
    on_bounded_stack(
        "text_layout::tests::baseline_offsets::captured_depth::deep_captured_named_atomics_preserve_both_baseline_sources_f32",
        || {
            for source in [BaselineSource::First, BaselineSource::Last] {
                assert_captured_named_chain::<f32>(24, source);
            }
        },
    );
}

#[test]
fn deep_captured_named_atomics_preserve_both_baseline_sources_f64() {
    on_bounded_stack(
        "text_layout::tests::baseline_offsets::captured_depth::deep_captured_named_atomics_preserve_both_baseline_sources_f64",
        || {
            for source in [BaselineSource::First, BaselineSource::Last] {
                assert_captured_named_chain::<f64>(24, source);
            }
        },
    );
}

//! Public sizing, containing-frame, replay and refusal witnesses. The font and
//! allocation premises remain those independently qualified by the parent.
use super::*;

fn central() -> l::InlineBaselineContext {
    l::InlineBaselineContext::Named(l::BaselineName::Central)
}

fn attach<S: l::LayoutScalar>(
    host: &mut Host<S>,
    container: u32,
    fixture: &Fixture,
) -> l::TextParagraphBinding<u32> {
    let origin = fixture.prepared.origin_views().next().unwrap().handle();
    let text = host.push(
        Some(container),
        l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone())),
    );
    l::TextParagraphBinding {
        prepared: fixture.prepared.clone(),
        container,
        nodes: vec![text],
        origins: vec![l::TextOriginBinding { origin, node: text }],
        controls: vec![],
        atomics: vec![],
        line_strut: Some(STRUT),
    }
}

fn available<S: l::LayoutScalar>(width: f64, height: f64) -> l::Size<l::AvailableOf<S>> {
    l::Size::new(
        l::AvailableOf::definite(n(width)),
        l::AvailableOf::definite(n(height)),
    )
}

fn solve<S: l::LayoutScalar>(
    host: &Host<S>,
    bindings: &[l::TextParagraphBinding<u32>],
    space: l::Size<l::AvailableOf<S>>,
    changed: &[u32],
) -> Result<l::CompletedLayoutBatchOf<u32, S>, l::LayoutErrorOf<u32, S>> {
    let flow = l::LayoutFlowInput::try_new(host, 0, bindings.to_vec(), l::WrappingInput::default())
        .expect("actual capture, host and issued origin must form a valid binding");
    l::compute_layout_with_flow_invalidated(
        &flow,
        0,
        l::LayoutRootRequestOf::viewport(space).unwrap(),
        changed,
    )
}

fn placed<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
) -> &t::Layout {
    batch
        .placed_text()
        .iter()
        .find(|p| p.container() == container)
        .unwrap()
        .layout()
}

fn central_offsets<S: l::LayoutScalar>(facts: l::NamedBaselinesOf<S>) -> [S; 2] {
    let pair = facts.get(l::BaselineName::Central);
    [pair.first(), pair.last()].map(|fact| match fact {
        l::BaselineFactOf::Available(coordinate) => {
            assert_eq!(coordinate.axis(), l::PhysicalAxis::Horizontal);
            coordinate.offset()
        }
        other => panic!("expected actual Central authority, got {other:?}"),
    })
}

fn assert_named<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    container: u32,
    offsets: [f64; 2],
) {
    assert_eq!(
        central_offsets(node(batch.unrounded_entries(), container).named_baselines),
        offsets.map(n)
    );
}

fn lines<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    binding: &l::TextParagraphBinding<u32>,
) -> Vec<t::FinalizedLine> {
    let mut selected = std::collections::BTreeMap::new();
    for row in batch.unrounded_inline_fragments() {
        if matches!(row.owner(), l::InlineFragmentOwner::Node(node) if binding.nodes.contains(node))
        {
            let fragment = row.fragment();
            let line = fragment.finalized_fragment().finalized_line();
            assert_eq!(line.prepared_text(), binding.prepared);
            assert_eq!(
                fragment.finalized_fragment().view().origin(),
                Some(binding.origins[0].origin.clone())
            );
            selected
                .entry(fragment.line_index())
                .or_insert_with(|| line.clone());
        }
    }
    selected.into_values().collect()
}

fn assert_frame<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    binding: &l::TextParagraphBinding<u32>,
    fixture: &Fixture,
    centers: &[f64],
    inline: (f64, f64, f64), // cell top, allocated inline capacity, actual run start
    border_origin: (f64, f64),
    direction: l::Direction,
) {
    let (top, capacity, run_start) = inline;
    let (gx, gy) = border_origin;
    let text = placed(batch, binding.container);
    assert_eq!(text.source(), &fixture.source);
    assert_eq!(
        text.analyzed_source().identity(),
        fixture.prepared.analyzed_source().identity()
    );
    assert_eq!(text.orientation(), t::LineOrientation::Vertical);
    assert_eq!(text.lines().len(), centers.len());
    let receipts = lines(batch, binding);
    assert_eq!(receipts.len(), centers.len());
    for (index, ((line, receipt), &c)) in
        text.lines().iter().zip(&receipts).zip(centers).enumerate()
    {
        let range = if centers.len() == 1 {
            t::Range::new(0, 2)
        } else {
            t::Range::new(index, index + 1)
        };
        assert_eq!(line.range(), range);
        assert_eq!(line.baseline(), c);
        assert_eq!(line.bounds(), t::Rect::new(c - 2.0, top, 10.0, capacity));
        assert_eq!(
            (receipt.feedback().baseline, receipt.feedback().line_height),
            (8.0, 10.0)
        );
        assert_eq!(
            receipt.feedback().advance,
            if centers.len() == 1 { 8.0 } else { 4.0 }
        );
        let selection = receipt.selected_line();
        let request = selection.request();
        assert_eq!(request.line_strut, Some(STRUT));
        assert_eq!(request.context.available_inline, Some(capacity));
        assert_eq!(
            request.context.base_direction,
            t::ParagraphBaseDirection::LeftToRight
        );
        assert_eq!(
            request.context.line_box_direction,
            if direction == l::Direction::Ltr {
                t::LineBoxDirection::LeftToRight
            } else {
                t::LineBoxDirection::RightToLeft
            }
        );
        assert_eq!(request.context.orientation, t::LineOrientation::Vertical);
    }
    let runs = text.run_views().collect::<Vec<_>>();
    let mut glyphs = Vec::new();
    for run in runs {
        assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
        assert_eq!(run.face().bytes(), fixture.bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            fixture.prepared.font_catalogue_identity()
        );
        assert_eq!(run.effective_instance().unwrap().font_size(), 4.0);
        let index = if centers.len() == 1 {
            0
        } else {
            run.range().start
        };
        assert_eq!(run.origin(), t::Point::new(centers[index], run_start));
        assert!(text.lines().iter().any(|line| {
            line.range().start <= run.range().start && run.range().end <= line.range().end
        }));
        glyphs.extend(
            run.glyphs()
                .map(|g| (g.id(), g.x(), g.y(), g.advance(), g.pen())),
        );
    }
    assert_eq!(
        glyphs,
        [
            (274, -2.0, 3.2, 4.0, 0.0),
            (
                274,
                -2.0,
                3.2,
                4.0,
                if centers.len() == 1 { 4.0 } else { 0.0 }
            ),
        ]
    );
    let mut source_ranges = Vec::new();
    for row in batch.unrounded_inline_fragments() {
        if matches!(row.owner(), l::InlineFragmentOwner::Node(node) if binding.nodes.contains(node))
        {
            let fragment = row.fragment();
            let c = centers[fragment.line_index()];
            let view = fragment.finalized_fragment().view();
            let source_range = view.aggregate_range().unwrap();
            assert_eq!(source_range.len(), 1);
            assert!(source_range.start < 2);
            assert_eq!(view.local_range(), Some(source_range));
            let line_range = text.lines()[fragment.line_index()].range();
            assert!(line_range.start <= source_range.start && source_range.end <= line_range.end);
            source_ranges.push(source_range);
            // The common u0 anchor is distinct from each aligned source unit.
            assert_eq!(fragment.baseline(), l::Point::new(n(c - gx), n(top - gy)));
            let pen = if centers.len() == 1 {
                [0.0, 4.0][source_range.start]
            } else {
                0.0
            };
            let rect = fragment.rect().unwrap();
            assert_eq!(
                rect.origin(),
                l::Point::new(n(c - gx - 2.0), n(run_start - gy + pen))
            );
            assert_eq!(rect.size(), l::Size::new(n(4.0), n(4.0)));
        }
    }
    source_ranges.sort_by_key(|range| range.start);
    assert_eq!(source_ranges, [t::Range::new(0, 1), t::Range::new(1, 2)]);
}

fn intrinsic_and_auto<S: l::LayoutScalar>() {
    let fixture = fixture();
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        for (inline, width, height) in [
            (l::AvailableOf::MIN_CONTENT, 20.0, 4.0),
            (l::AvailableOf::MAX_CONTENT, 10.0, 8.0),
        ] {
            let mut input = root(mode, l::Direction::Ltr, central());
            input.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO);
            let mut host = Host::<S>::new(input);
            let binding = attach(&mut host, 0, &fixture);
            let batch = solve(
                &host,
                std::slice::from_ref(&binding),
                l::Size::new(l::AvailableOf::MAX_CONTENT, inline),
                &[],
            )
            .unwrap();
            let output = node(batch.unrounded_entries(), 0);
            assert_eq!(output.location, l::Point::ZERO);
            assert_eq!(output.size, l::Size::new(n(width), n(height)));
            let centers = if width == 10.0 {
                vec![2.0]
            } else if mode == l::WritingMode::VerticalRl {
                vec![12.0, 2.0]
            } else {
                vec![2.0, 12.0]
            };
            assert_frame(
                &batch,
                &binding,
                &fixture,
                &centers,
                (0.0, height, 0.0),
                (0.0, 0.0),
                l::Direction::Ltr,
            );
            assert_named(&batch, 0, [centers[0], *centers.last().unwrap()]);
        }
        for finite in [false, true] {
            let mut input = root(mode, l::Direction::Ltr, central());
            input.size = l::Size::new(l::PreferredSizeOf::AUTO, px(4.0));
            let mut host = Host::<S>::new(input);
            let binding = attach(&mut host, 0, &fixture);
            let batch = solve(
                &host,
                std::slice::from_ref(&binding),
                l::Size::new(
                    if finite {
                        l::AvailableOf::definite(n(40.0))
                    } else {
                        l::AvailableOf::MAX_CONTENT
                    },
                    l::AvailableOf::definite(n(4.0)),
                ),
                &[],
            )
            .unwrap();
            let gx = if finite && mode == l::WritingMode::VerticalRl {
                20.0
            } else {
                0.0
            };
            assert_eq!(
                node(batch.unrounded_entries(), 0).location,
                l::Point::new(n(gx), n(0.0))
            );
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(20.0), n(4.0))
            );
            let local = if mode == l::WritingMode::VerticalRl {
                [12.0, 2.0]
            } else {
                [2.0, 12.0]
            };
            assert_frame(
                &batch,
                &binding,
                &fixture,
                &local.map(|c| c + gx),
                (0.0, 4.0, 0.0),
                (gx, 0.0),
                l::Direction::Ltr,
            );
            assert_named(&batch, 0, local);
        }
    }
}

fn insets_and_direction<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let fixture = fixture_in_direction(direction);
        for (mode, centers) in [
            (l::WritingMode::VerticalRl, [35.0, 25.0]),
            (l::WritingMode::VerticalLr, [5.0, 15.0]),
        ] {
            let mut input = root(mode, direction, central());
            input.box_sizing = l::BoxSizing::BorderBox;
            input.size = l::Size::new(px(50.0), px(17.0));
            input.padding = l::Edges {
                left: l::LengthOf::value(l::LengthPercentageOf::px(n(3.0)).unwrap()),
                right: l::LengthOf::value(l::LengthPercentageOf::px(n(7.0)).unwrap()),
                top: l::LengthOf::value(l::LengthPercentageOf::px(n(4.0)).unwrap()),
                bottom: l::LengthOf::value(l::LengthPercentageOf::px(n(6.0)).unwrap()),
            };
            let mut host = Host::<S>::new(input);
            let binding = attach(&mut host, 0, &fixture);
            let batch = solve(
                &host,
                std::slice::from_ref(&binding),
                available(50.0, 17.0),
                &[],
            )
            .unwrap();
            assert_eq!(
                node(batch.unrounded_entries(), 0).size,
                l::Size::new(n(50.0), n(17.0))
            );
            assert_frame(
                &batch,
                &binding,
                &fixture,
                &centers,
                (
                    4.0,
                    7.0,
                    if direction == l::Direction::Ltr {
                        4.0
                    } else {
                        7.0
                    },
                ),
                (0.0, 0.0),
                direction,
            );
            assert_named(&batch, 0, centers);
        }
    }
}

fn fractional_translation<S: l::LayoutScalar>() {
    let fixture = fixture();
    for mode in [l::WritingMode::VerticalRl, l::WritingMode::VerticalLr] {
        let mut outer = root(l::WritingMode::VerticalLr, l::Direction::Ltr, central());
        outer.size = l::Size::new(px(100.0), px(100.0));
        outer.padding.left = l::LengthOf::value(l::LengthPercentageOf::px(n(7.25)).unwrap());
        outer.padding.top = l::LengthOf::value(l::LengthPercentageOf::px(n(9.5)).unwrap());
        let mut host = Host::<S>::new(outer);
        let mut child = root(mode, l::Direction::Ltr, central());
        child.size = l::Size::new(px(40.0), px(4.0));
        let container = host.push(Some(0), l::LayoutInputOf::box_input(child));
        let binding = attach(&mut host, container, &fixture);
        let batch = solve(
            &host,
            std::slice::from_ref(&binding),
            available(100.0, 100.0),
            &[],
        )
        .unwrap();
        assert_eq!(
            node(batch.unrounded_entries(), container).location,
            l::Point::new(n(7.25), n(9.5))
        );
        let text = placed(&batch, container);
        assert_eq!(text.lines().len(), 2);
        // Reference: docs/reference.md named-frame and canonical-placement
        // contracts: raw named facts remain unsnapped. Final packet facts
        // reconstruct the once-snapped Text anchor from its published border.
        let raw_offsets = if mode == l::WritingMode::VerticalRl {
            [32.0, 22.0]
        } else {
            [2.0, 12.0]
        };
        assert_named(&batch, container, raw_offsets);
        for (entries, final_packet) in [
            (batch.unrounded_entries(), false),
            (batch.final_entries(), true),
        ] {
            let child = node(entries, container);
            let offsets = central_offsets(child.named_baselines);
            let parent_offsets = central_offsets(node(entries, 0).named_baselines);
            for i in 0..2 {
                let anchor = child.location.x.to_f64() + offsets[i].to_f64();
                assert_eq!(parent_offsets[i].to_f64(), anchor);
                if final_packet {
                    assert_eq!(anchor, text.lines()[i].baseline());
                    assert_eq!(anchor.fract(), 0.0);
                } else {
                    assert_eq!(anchor, 7.25 + raw_offsets[i]);
                    assert_eq!((anchor + 0.5).floor(), text.lines()[i].baseline());
                }
            }
        }
        let first = text.lines()[0].baseline();
        let last = text.lines()[1].baseline();
        assert_eq!(
            last - first,
            if mode == l::WritingMode::VerticalRl {
                -10.0
            } else {
                10.0
            }
        );
        assert_eq!(text.source(), &fixture.source);
        for receipt in lines(&batch, &binding) {
            assert_eq!(
                (receipt.feedback().baseline, receipt.feedback().line_height),
                (8.0, 10.0)
            );
        }
        for run in text.run_views() {
            assert_eq!(run.glyph_basis(), t::GlyphBasis::Unrotated);
            assert_eq!(run.face().bytes(), fixture.bytes);
            for glyph in run.glyphs() {
                assert_eq!(
                    (glyph.id(), glyph.x(), glyph.y(), glyph.advance()),
                    (274, -2.0, 3.2, 4.0)
                );
            }
        }
    }
}

fn replay_host<S: l::LayoutScalar>(
    mode: l::WritingMode,
    height: f64,
    fixture: &Fixture,
) -> (Host<S>, l::TextParagraphBinding<u32>, u32) {
    let mut input = root(mode, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(height));
    let mut host = Host::<S>::new(input);
    let mut child = root(mode, l::Direction::Ltr, central());
    child.size = l::Size::new(l::PreferredSizeOf::AUTO, l::PreferredSizeOf::AUTO);
    let container = host.push(Some(0), l::LayoutInputOf::box_input(child));
    let binding = attach(&mut host, container, fixture);
    // A genuine ordinary numeric sibling outside the complete paragraph's
    // captured node window participates in the public cache operation.
    let mut leaf = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
    leaf.block_flow_context = l::BlockFlowContext::Independent;
    leaf.size = l::Size::new(px(5.0), px(4.0));
    let leaf = host.push(Some(0), l::LayoutInputOf::box_input(leaf));
    (host, binding, leaf)
}

fn cache_and_frame_change<S: l::LayoutScalar>() {
    let fixture = fixture();
    for (mode, centers) in [
        (l::WritingMode::VerticalRl, [32.0, 22.0]),
        (l::WritingMode::VerticalLr, [2.0, 12.0]),
    ] {
        let (mut host, binding, leaf) = replay_host::<S>(mode, 4.0, &fixture);
        let cold_origin = (
            if mode == l::WritingMode::VerticalRl {
                20.0
            } else {
                0.0
            },
            0.0,
        );
        let changed_origin = (
            if mode == l::WritingMode::VerticalRl {
                30.0
            } else {
                0.0
            },
            0.0,
        );
        let cold = solve(
            &host,
            std::slice::from_ref(&binding),
            available(40.0, 4.0),
            &[],
        )
        .unwrap();
        assert_frame(
            &cold,
            &binding,
            &fixture,
            &centers,
            (0.0, 4.0, 0.0),
            cold_origin,
            l::Direction::Ltr,
        );
        assert!(
            cold.cache_store_entries()
                .iter()
                .any(|row| row.node() == leaf)
        );
        cold.apply_to(&mut host).unwrap();
        assert_eq!((host.prepares.get(), host.commits), (1, 1));
        let hits = host.hits[leaf as usize].get();
        let warm = solve(
            &host,
            std::slice::from_ref(&binding),
            available(40.0, 4.0),
            &[],
        )
        .unwrap();
        assert!(
            host.hits[leaf as usize].get() > hits,
            "eligible numeric sibling actually reuses its geometry cache"
        );
        assert_frame(
            &warm,
            &binding,
            &fixture,
            &centers,
            (0.0, 4.0, 0.0),
            cold_origin,
            l::Direction::Ltr,
        );
        assert_eq!(semantic(&cold), semantic(&warm));
        warm.apply_to(&mut host).unwrap();
        host.change(0, |input| input.size.height = px(8.0));
        let changed = solve(
            &host,
            std::slice::from_ref(&binding),
            available(40.0, 8.0),
            &[0],
        )
        .unwrap();
        let c = centers[0];
        assert_frame(
            &changed,
            &binding,
            &fixture,
            &[c],
            (0.0, 8.0, 0.0),
            changed_origin,
            l::Direction::Ltr,
        );
        assert_eq!(
            node(changed.unrounded_entries(), binding.container).size,
            l::Size::new(n(10.0), n(8.0))
        );
        assert_named(&changed, binding.container, [2.0, 2.0]);
        let (fresh, fresh_binding, fresh_leaf) = replay_host::<S>(mode, 8.0, &fixture);
        let fresh_batch = solve(
            &fresh,
            std::slice::from_ref(&fresh_binding),
            available(40.0, 8.0),
            &[],
        )
        .unwrap();
        assert_frame(
            &fresh_batch,
            &fresh_binding,
            &fixture,
            &[c],
            (0.0, 8.0, 0.0),
            changed_origin,
            l::Direction::Ltr,
        );
        assert_eq!(semantic(&changed), semantic(&fresh_batch));
        assert_eq!(
            node(changed.unrounded_entries(), leaf),
            node(fresh_batch.unrounded_entries(), fresh_leaf)
        );
        changed.apply_to(&mut host).unwrap();
        assert_frame(
            &cold,
            &binding,
            &fixture,
            &centers,
            (0.0, 4.0, 0.0),
            cold_origin,
            l::Direction::Ltr,
        );
        assert_frame(
            &warm,
            &binding,
            &fixture,
            &centers,
            (0.0, 4.0, 0.0),
            cold_origin,
            l::Direction::Ltr,
        );
        assert_frame(
            host.published.batch.as_ref().unwrap(),
            &binding,
            &fixture,
            &[c],
            (0.0, 8.0, 0.0),
            changed_origin,
            l::Direction::Ltr,
        );
    }
}

fn refusals_preserve_publication<S: l::LayoutScalar>() {
    let fixture = fixture();
    let mut input = root(l::WritingMode::VerticalRl, l::Direction::Ltr, central());
    input.size = l::Size::new(px(40.0), px(4.0));
    let mut host = Host::<S>::new(input);
    let binding = attach(&mut host, 0, &fixture);
    let old = solve(
        &host,
        std::slice::from_ref(&binding),
        available(40.0, 4.0),
        &[],
    )
    .unwrap();
    old.apply_to(&mut host).unwrap();
    let published_nodes = semantic(host.published.batch.as_ref().unwrap());
    host.change(0, |input| {
        input.inline_baseline_context = l::InlineBaselineContext::Named(l::BaselineName::Alphabetic)
    });
    let error = solve(
        &host,
        std::slice::from_ref(&binding),
        available(40.0, 4.0),
        &[0],
    )
    .unwrap_err();
    assert_eq!(error.site(), l::LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), l::LayoutOperation::TextCapture);
    assert!(matches!(
        error.kind(),
        l::LayoutErrorKindOf::MissingContext(l::LayoutMissingContext::NamedInlineBaseline)
    ));
    // These failures are actual immutable computations, not sink-prepare
    // attempts. Their absence of a new batch leaves published state intact.
    for mode in [
        l::WritingMode::HorizontalTb,
        l::WritingMode::SidewaysRl,
        l::WritingMode::SidewaysLr,
    ] {
        host.change(0, |input| {
            input.inline_baseline_context = l::InlineBaselineContext::Physical;
            input.writing_mode = mode;
        });
        let error = solve(
            &host,
            std::slice::from_ref(&binding),
            available(40.0, 4.0),
            &[0],
        )
        .unwrap_err();
        assert_eq!(error.site(), l::LayoutErrorSiteOf::Node(0));
        if mode == l::WritingMode::SidewaysLr {
            assert_eq!(error.operation(), l::LayoutOperation::TextCapture);
            assert!(matches!(
                error.kind(),
                l::LayoutErrorKindOf::UnsupportedCapability(
                    l::LayoutUnsupportedCapability::TextWritingMode(l::WritingMode::SidewaysLr)
                )
            ));
        } else {
            let l::LayoutErrorKindOf::Text(text_error) = error.kind() else {
                panic!("expected actual orientation error: {error:?}")
            };
            assert_eq!(
                text_error.detail(),
                Some(&t::ErrorDetail::InvalidTextProtocol {
                    reason: t::ProtocolError::MismatchedContext {
                        field: t::ProtocolContextField::Orientation
                    },
                })
            );
        }
        assert_eq!(
            semantic(host.published.batch.as_ref().unwrap()),
            published_nodes
        );
        assert_frame(
            host.published.batch.as_ref().unwrap(),
            &binding,
            &fixture,
            &[32.0, 22.0],
            (0.0, 4.0, 0.0),
            (0.0, 0.0),
            l::Direction::Ltr,
        );
    }
    host.change(0, |input| {
        input.writing_mode = l::WritingMode::VerticalRl;
        input.inline_baseline_context = central();
    });
    let restored = solve(&host, &[binding], available(40.0, 4.0), &[0]).unwrap();
    assert_eq!(semantic(&old), semantic(&restored));
    // SidewaysLr remains supported for genuine box-only layout; the guard
    // belongs to the Text-backed adapter rather than FlowAxes generally.
    let mut input = root(
        l::WritingMode::SidewaysLr,
        l::Direction::Ltr,
        l::InlineBaselineContext::Physical,
    );
    input.size = l::Size::new(px(40.0), px(4.0));
    let boxes = Host::<S>::new(input);
    let batch = boxes.compute(l::WritingMode::SidewaysLr, &[]).unwrap();
    assert!(batch.placed_text().is_empty());
    assert_eq!(
        node(batch.unrounded_entries(), 0).size,
        l::Size::new(n(40.0), n(4.0))
    );
}

fn interaction_and_scroll<S: l::LayoutScalar>() {
    let fixture = fixture();
    for (mode, centers) in [
        (l::WritingMode::VerticalRl, [32.0, 22.0]),
        (l::WritingMode::VerticalLr, [2.0, 12.0]),
    ] {
        let batch = compute::<S>(&fixture, mode, central());
        let text = placed(&batch, 0);
        // Carets and selection consume the finalized allocation's 10-unit
        // metric band, while the qualified source fragments are only 4 wide.
        // Source affinity distinguishes the two genuine sides of the soft cut.
        for (cursor, c, y) in [
            (t::Cursor::new(0, t::Affinity::After), centers[0], 0.0),
            (t::Cursor::new(1, t::Affinity::Before), centers[0], 4.0),
            (t::Cursor::new(1, t::Affinity::After), centers[1], 0.0),
            (t::Cursor::new(2, t::Affinity::Before), centers[1], 4.0),
        ] {
            assert_eq!(
                text.cursor(cursor).unwrap().rect(),
                t::Rect::new(c - 2.0, y, 10.0, 1.0)
            );
        }
        let selected = text
            .selection(t::Selection::new(
                t::Cursor::new(0, t::Affinity::After),
                t::Cursor::new(2, t::Affinity::Before),
            ))
            .unwrap();
        assert_eq!(
            selected
                .rects()
                .iter()
                .map(|rect| (rect.line(), rect.rect()))
                .collect::<Vec<_>>(),
            [
                (0, t::Rect::new(centers[0] - 2.0, 0.0, 10.0, 4.0)),
                (1, t::Rect::new(centers[1] - 2.0, 0.0, 10.0, 4.0)),
            ]
        );
        for (index, c) in centers.into_iter().enumerate() {
            let t::Hit::Text(hit) = text.hit(t::Point::new(c, 0.5)) else {
                panic!("actual native leading half-cell must hit its text occurrence");
            };
            assert_eq!(
                hit.source_cursor(),
                t::Cursor::new(index, t::Affinity::After)
            );
            assert_eq!(
                text.position_geometry(&hit).unwrap().rect(),
                t::Rect::new(c - 2.0, 0.0, 10.0, 1.0)
            );
        }

        // Two H10 allocations consume 20 physical block units in a 12-unit
        // scrollport: eight units toward flow-end. Vrl has negative physical
        // x scrolling and Vlr positive x scrolling; inline y does not overflow.
        let mut input = root(mode, l::Direction::Ltr, central());
        input.size = l::Size::new(px(12.0), px(4.0));
        input.overflow =
            l::ComputedOverflow::try_new(l::Overflow::Auto, l::Overflow::Auto).unwrap();
        let mut host = Host::<S>::new(input);
        let binding = attach(&mut host, 0, &fixture);
        let narrow = solve(
            &host,
            std::slice::from_ref(&binding),
            available(12.0, 4.0),
            &[],
        )
        .unwrap();
        let expected_centers = if mode == l::WritingMode::VerticalRl {
            [4.0, -6.0]
        } else {
            [2.0, 12.0]
        };
        assert_frame(
            &narrow,
            &binding,
            &fixture,
            &expected_centers,
            (0.0, 4.0, 0.0),
            (0.0, 0.0),
            l::Direction::Ltr,
        );
        for entries in [narrow.unrounded_entries(), narrow.final_entries()] {
            let scroll = node(entries, 0).scroll_geometry.unwrap();
            let range = scroll.physical_range();
            assert_eq!(
                (range.y().minimum(), range.y().maximum()),
                (S::ZERO, S::ZERO)
            );
            let (minimum, maximum, left) = if mode == l::WritingMode::VerticalRl {
                (-8.0, 0.0, -8.0)
            } else {
                (0.0, 8.0, 0.0)
            };
            assert_eq!(
                (range.x().minimum(), range.x().maximum()),
                (n(minimum), n(maximum))
            );
            assert_eq!(
                scroll.scrollable_overflow().origin(),
                l::Point::new(n(left), n(0.0))
            );
            assert_eq!(
                scroll.scrollable_overflow().size(),
                l::Size::new(n(20.0), n(4.0))
            );
        }
    }
}

#[test]
fn native_intrinsic_and_auto_frames_f32() {
    intrinsic_and_auto::<f32>();
}
#[test]
fn native_intrinsic_and_auto_frames_f64() {
    intrinsic_and_auto::<f64>();
}
#[test]
fn native_insets_and_authentic_containing_direction_f32() {
    insets_and_direction::<f32>();
}
#[test]
fn native_insets_and_authentic_containing_direction_f64() {
    insets_and_direction::<f64>();
}
#[test]
fn native_fractional_ancestor_preserves_named_physical_anchor_f32() {
    fractional_translation::<f32>();
}
#[test]
fn native_fractional_ancestor_preserves_named_physical_anchor_f64() {
    fractional_translation::<f64>();
}
#[test]
fn native_participating_cache_replay_and_changed_inline_frame_f32() {
    cache_and_frame_change::<f32>();
}
#[test]
fn native_participating_cache_replay_and_changed_inline_frame_f64() {
    cache_and_frame_change::<f64>();
}
#[test]
fn native_refusals_preserve_published_packet_and_box_only_guard_f32() {
    refusals_preserve_publication::<f32>();
}
#[test]
fn native_refusals_preserve_published_packet_and_box_only_guard_f64() {
    refusals_preserve_publication::<f64>();
}
#[test]
fn native_interaction_and_physical_scroll_ranges_f32() {
    interaction_and_scroll::<f32>();
}
#[test]
fn native_interaction_and_physical_scroll_ranges_f64() {
    interaction_and_scroll::<f64>();
}

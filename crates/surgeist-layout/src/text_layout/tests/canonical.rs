//! Canonical placement over actual registered shaping and original occurrences.

#[path = "canonical_fonts.rs"]
pub(super) mod fonts;

use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use surgeist_text as text;

pub(super) fn paragraph(bytes: Vec<u8>, pieces: &[&str]) -> TextParagraphBinding<u32> {
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 12.5,
        line_height: text::LineHeight::Absolute(16.0),
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let source = text::Source::new(pieces.concat());
    let mut at = 0;
    let origins = pieces
        .iter()
        .map(|piece| {
            let source = text::Source::new(*piece);
            let len = source.text().len();
            let origin = text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, len),
                aggregate: text::Range::new(at, at + len),
                kind: text::ParagraphOriginKind::Authored,
            };
            at += len;
            origin
        })
        .collect::<Vec<_>>();
    let connections = origins
        .iter()
        .skip(1)
        .map(|origin| text::ParagraphConnection {
            at: origin.aggregate.start,
            kind: text::ParagraphConnectionKind::Continuous,
        })
        .collect::<Vec<_>>();
    let boundary_policies = (0..connections.len())
        .map(|index| text::ParagraphBoundaryPolicy {
            definition: text::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: style.wrap,
            following_wrap: style.wrap,
            ancestor_wrap: style.wrap,
        })
        .collect();
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source,
        base_style: style,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        origins,
        connections,
        controls: vec![],
        boundary_policies,
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, ancestor_wrap);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origins = prepared
        .origin_views()
        .enumerate()
        .map(|(index, view)| TextOriginBinding {
            origin: view.handle(),
            node: 10 + u32::try_from(index).unwrap(),
        })
        .collect::<Vec<_>>();
    TextParagraphBinding {
        nodes: origins.iter().map(|origin| origin.node).collect(),
        prepared,
        container: 2,
        origins,
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    }
}

fn nested_batch<S: LayoutScalar>(
    binding: &TextParagraphBinding<u32>,
    moved: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let flex = |width, height| NodeInputOf {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: Some(AlignItems::Start),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    };
    let margin = |left, top| Edges {
        left: LengthAutoOf::px(S::from_f64(left)),
        top: LengthAutoOf::px(S::from_f64(top)),
        ..Edges::all(LengthAutoOf::ZERO)
    };
    // Flex ancestry prevents block-margin collapse. Both actual fractional
    // ancestor locations are .75 before Layout's selected box rounding policy.
    // The moved case adds an integral (4,5), preserving every rounding phase.
    let outer = NodeInputOf {
        margin: margin(
            if moved { 4.75 } else { 0.75 },
            if moved { 5.75 } else { 0.75 },
        ),
        ..flex(60.75, 50.75)
    };
    let container = NodeInputOf {
        display: Display::Block,
        margin: margin(0.75, 0.75),
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(30.5)),
            PreferredSizeOf::px(S::from_f64(24.5)),
        ),
        ..NodeInputOf::default()
    };
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, binding.nodes.clone())
        .style(0, flex(100.0, 100.0))
        .style(1, outer)
        .style(2, container);
    for origin in &binding.origins {
        tree = tree.children(origin.node, []).input(
            origin.node,
            LayoutInputOf::inline_text(TextParticipation::new(origin.origin.clone())),
        );
    }
    let capture = TextLayoutInput::try_new(&tree, 0, vec![binding.clone()]).unwrap();
    compute_layout_with_text(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(100.0)),
            AvailableOf::definite(S::from_f64(100.0)),
        ))
        .unwrap(),
    )
    .unwrap()
}

fn output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    final_phase: bool,
) -> NodeOutputOf<S> {
    let entries = if final_phase {
        batch.final_entries()
    } else {
        batch.unrounded_entries()
    };
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn assert_rect(actual: text::Rect, expected: text::Rect) {
    assert_eq!(actual, expected);
}

#[derive(Debug, PartialEq)]
struct GlyphFacts {
    id: u32,
    range: text::Range,
    x: f32,
    y: f32,
    advance: f32,
    pen: f64,
}

#[derive(Debug, PartialEq)]
struct Geometry {
    glyphs: Vec<GlyphFacts>,
    relative_glyph_origins: Vec<text::Point>,
    relative_clusters: Vec<text::Cluster>,
    relative_carets: Vec<(text::Cursor, text::Rect)>,
    relative_selection: Vec<text::Rect>,
}

fn assert_placement<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    binding: &TextParagraphBinding<u32>,
    moved: bool,
    mark: bool,
) -> Geometry {
    let shift = if moved {
        text::Point::new(4.0, 5.0)
    } else {
        text::Point::new(0.0, 0.0)
    };
    let container_origin = text::Point::new(2.0 + shift.x, 2.0 + shift.y);
    // Each .75 committed ancestor origin is 1. The source bounds and carets use
    // the resulting global origin 2, not the pre-rounding cumulative origin 1.5.
    for (node, raw, committed) in [
        (
            1,
            text::Point::new(0.75 + shift.x, 0.75 + shift.y),
            text::Point::new(1.0 + shift.x, 1.0 + shift.y),
        ),
        (2, text::Point::new(0.75, 0.75), text::Point::new(1.0, 1.0)),
    ] {
        assert_eq!(
            output(batch, node, false).location,
            Point::new(S::from_f64(raw.x), S::from_f64(raw.y))
        );
        assert_eq!(
            output(batch, node, true).location,
            Point::new(S::from_f64(committed.x), S::from_f64(committed.y))
        );
    }
    let placed = batch.placed_text()[0].layout();
    assert_eq!(batch.placed_text()[0].container(), 2);
    assert_eq!(batch.placed_text()[0].nodes(), binding.nodes.as_slice());
    assert_eq!(
        placed.source().text().code_units(),
        if mark { &[0x41, 0x301] } else { &[0x41, 0x41] }
    );
    assert_eq!(placed.lines().len(), 1);
    let baseline = 14.0 + shift.y;
    // Ahem ascent 10 + half-leading (16-12.5)/2 = 11.75. Only the
    // global baseline 13.75 is snapped to 14; the local 16px strut remains rigid.
    assert_eq!(placed.lines()[0].baseline(), baseline);
    assert_eq!(placed.lines()[0].range(), text::Range::new(0, 2));
    let run = placed.glyph_runs();
    assert_eq!(run.len(), 1);
    assert_eq!(
        run[0].origin(),
        text::Point::new(container_origin.x, baseline)
    );
    assert_eq!(run[0].advance(), 12.5);
    let glyphs = run[0].glyphs();
    assert_eq!(
        glyphs.iter().map(|glyph| glyph.id()).collect::<Vec<_>>(),
        if mark { vec![35, 100] } else { vec![58] }
    );
    assert_eq!(glyphs[0].range(), text::Range::new(0, 2));
    assert_eq!(glyphs[0].advance(), 12.5);
    assert_eq!(glyphs[0].pen() + f64::from(glyphs[0].x()), 0.0);
    assert_eq!(glyphs[0].y(), 0.0);
    if mark {
        // Literal GPOS anchors: (400,600)-(100,200), size 12.5/UPEM 1000.
        assert_eq!(glyphs[1].range(), text::Range::new(0, 2));
        assert_eq!(glyphs[1].pen() + f64::from(glyphs[1].x()), 3.75);
        assert_eq!(glyphs[1].y(), -5.0);
        assert_eq!(glyphs[1].advance(), 0.0);
    }
    let anchor = text::Point::new(container_origin.x, baseline);
    let relative = |rect: text::Rect| {
        text::Rect::new(
            rect.origin.x - anchor.x,
            rect.origin.y - anchor.y,
            rect.size.width,
            rect.size.height,
        )
    };
    let clusters = placed.clusters();
    // Text's existing extraction policy retains one interaction component per
    // original scalar atom and equally shares a merged provider advance. Both
    // fonts yield a 12.5px provider cluster spanning two atoms: two 6.25px rows.
    // These rows are source geometry, not separate glyphs or glyph ink bounds.
    assert_eq!(clusters.len(), 2);
    for (cluster, start, x) in [
        (&clusters[0], 0, container_origin.x),
        (&clusters[1], 1, container_origin.x + 6.25),
    ] {
        assert_eq!(cluster.range(), text::Range::new(start, start + 1));
        assert_rect(
            cluster.bounds(),
            text::Rect::new(x, 2.25 + shift.y, 6.25, 16.0),
        );
    }
    let relative_clusters = clusters
        .into_iter()
        .map(|cluster| text::Cluster::new(cluster.range(), relative(cluster.bounds())))
        .collect();
    let positions = placed.positions().collect::<Vec<_>>();
    // Legal positions also respect actual glyph extents. The ligature's glyph
    // spans 0..2, excluding the interior grapheme stop; the combining sequence
    // has no interior grapheme stop. Both expose only the two outer positions.
    let expected_positions = [
        (
            text::Cursor::new(0, text::Affinity::After),
            container_origin.x,
        ),
        (
            text::Cursor::new(2, text::Affinity::Before),
            container_origin.x + 12.5,
        ),
    ];
    assert_eq!(positions.len(), expected_positions.len());
    for (position, (cursor, x)) in positions.iter().zip(expected_positions) {
        assert_eq!(position.source_cursor(), cursor);
        assert_rect(
            placed.position_geometry(position).unwrap().rect(),
            text::Rect::new(x, 2.25 + shift.y, 1.0, 16.0),
        );
    }
    let relative_carets = positions
        .iter()
        .map(|position| {
            let geometry = placed.position_geometry(position).unwrap();
            (position.source_cursor(), relative(geometry.rect()))
        })
        .collect();
    let selection = placed
        .position_selection(positions.first().unwrap(), positions.last().unwrap())
        .unwrap();
    assert_eq!(selection.geometry().rects().len(), 1);
    assert_rect(
        selection.geometry().rects()[0].rect(),
        text::Rect::new(container_origin.x, 2.25 + shift.y, 12.5, 16.0),
    );
    let relative_selection = selection
        .geometry()
        .rects()
        .iter()
        .map(|rect| relative(rect.rect()))
        .collect();
    let split = binding.origins.len() == 2;
    // Fragment rows retain actual selected-unit/source intersections. Their
    // cardinality is not a promise that one grapheme or glyph yields one row.
    // Check each row against its scalar component geometry and require complete
    // original-origin coverage, including both halves of the one-origin case.
    for origin in &binding.origins {
        let start = usize::try_from(origin.node - 10).unwrap();
        let source_units = origin.origin.view().source().text().code_units().to_vec();
        assert_eq!(
            source_units,
            if !split {
                if mark {
                    vec![0x41, 0x301]
                } else {
                    vec![0x41, 0x41]
                }
            } else if mark && start == 1 {
                vec![0x301]
            } else {
                vec![0x41]
            }
        );
    }
    let mut covered_ranges = Vec::new();
    for entry in batch.final_inline_fragments() {
        let crate::InlineFragmentOwner::Node(node) = entry.owner() else {
            panic!("source fragment owner")
        };
        let origin = binding
            .origins
            .iter()
            .find(|origin| origin.node == *node)
            .unwrap();
        let fragment = entry.fragment();
        let view = fragment.finalized_fragment().view();
        assert_eq!(view.origin(), Some(origin.origin.clone()));
        let origin_start = usize::try_from(origin.node - 10).unwrap();
        let origin_range = if split {
            text::Range::new(origin_start, origin_start + 1)
        } else {
            text::Range::new(0, 2)
        };
        let aggregate = view.aggregate_range().unwrap();
        assert!(
            origin_range.start <= aggregate.start
                && aggregate.start < aggregate.end
                && aggregate.end <= origin_range.end
        );
        assert_eq!(
            view.local_range(),
            Some(text::Range::new(
                aggregate.start - origin_range.start,
                aggregate.end - origin_range.start
            ))
        );
        covered_ranges.push((*node, aggregate));
        // Original-source bounds follow the real component intersections. They
        // do not duplicate a ligature's whole glyph advance into each origin.
        let x = 6.25 * aggregate.start as f64;
        let width = 6.25 * (aggregate.end - aggregate.start) as f64;
        let rect = fragment.rect().unwrap();
        assert_eq!(rect.origin(), Point::new(S::from_f64(x), S::from_f64(0.25)));
        assert_eq!(
            rect.size(),
            Size::new(S::from_f64(width), S::from_f64(16.0))
        );
        assert_eq!(fragment.baseline(), Point::new(S::ZERO, S::from_f64(12.0)));
        assert_rect(
            text::Rect::new(
                rect.origin().x.to_f64() + container_origin.x,
                rect.origin().y.to_f64() + container_origin.y,
                rect.size().width.to_f64(),
                rect.size().height.to_f64(),
            ),
            text::Rect::new(container_origin.x + x, 2.25 + shift.y, width, 16.0),
        );
    }
    for origin in &binding.origins {
        let origin_start = if split {
            usize::try_from(origin.node - 10).unwrap()
        } else {
            0
        };
        let origin_end = origin_start + if split { 1 } else { 2 };
        let mut ranges = covered_ranges
            .iter()
            .filter_map(|(node, range)| (*node == origin.node).then_some(*range))
            .collect::<Vec<_>>();
        ranges.sort_unstable_by_key(|range| (range.start, range.end));
        let mut covered_end = origin_start;
        for range in ranges {
            assert!(
                range.start <= covered_end,
                "actual source fragments leave a hole in their original origin"
            );
            covered_end = covered_end.max(range.end);
        }
        assert_eq!(covered_end, origin_end);
    }
    Geometry {
        glyphs: glyphs
            .iter()
            .map(|glyph| GlyphFacts {
                id: glyph.id(),
                range: glyph.range(),
                x: glyph.x(),
                y: glyph.y(),
                advance: glyph.advance(),
                pen: glyph.pen(),
            })
            .collect(),
        relative_glyph_origins: glyphs
            .iter()
            .map(|glyph| text::Point::new(glyph.pen() + f64::from(glyph.x()), f64::from(glyph.y())))
            .collect(),
        relative_clusters,
        relative_carets,
        relative_selection,
    }
}

fn assert_repartition_and_nested_translation<S: LayoutScalar>(mark: bool) {
    let bytes = if mark {
        fonts::mark_ahem()
    } else {
        fonts::ligature_ahem()
    };
    let whole = paragraph(bytes.clone(), &[if mark { "A\u{301}" } else { "AA" }]);
    let split = paragraph(bytes, &["A", if mark { "\u{301}" } else { "A" }]);
    let reference = nested_batch::<S>(&whole, false);
    let expected = assert_placement(&reference, &whole, false, mark);
    for binding in [&whole, &split] {
        for moved in [false, true] {
            let batch = nested_batch::<S>(binding, moved);
            assert_eq!(assert_placement(&batch, binding, moved, mark), expected);
        }
    }
}

#[test]
fn connected_ligature_origins_keep_fractional_geometry_through_nested_canonical_placement() {
    assert_repartition_and_nested_translation::<f32>(false);
    assert_repartition_and_nested_translation::<f64>(false);
}

#[test]
fn connected_combining_mark_origins_keep_attachment_through_nested_canonical_placement() {
    assert_repartition_and_nested_translation::<f32>(true);
    assert_repartition_and_nested_translation::<f64>(true);
}

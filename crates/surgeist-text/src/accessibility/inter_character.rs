//! Actual allocated native lines project the same predecessor-owned cells to AX.
//! Independent Ahem10 arithmetic: three10px advances + two(4+2)px gaps =42;
//! cell widths16/16/10, horizontal cross8/2, nominal upright cross5/5.
use crate::*;
use accesskit::{NodeId, Role, TextDirection};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn font(orientation: LineOrientation) -> Vec<u8> {
    if orientation != LineOrientation::Vertical {
        return AHEM.to_vec();
    }
    // Real Wide/Hani source maps to licensed chi274 with unchanged contours
    // and metrics, as in Render's upright fixture. This is not a Han design.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    cmap.extend(28_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(1_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(274_u32.to_be_bytes());
    crate::test_fonts::replace_tables(AHEM, vec![(*b"cmap", cmap)])
}

fn selected(orientation: LineOrientation, tracking: f64) -> (PreparedText, SelectedLine) {
    let upright = orientation == LineOrientation::Vertical;
    let bytes = font(orientation);
    let native = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(native.head().unwrap().units_per_em(), 1000);
    assert_eq!(
        native
            .charmap()
            .map(if upright { 0x4e00_u32 } else { 0x3a7 }),
        Some(GlyphId::new(274))
    );
    let metrics = native.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(274)), Some(1000.0));
    assert_eq!(
        (
            native.hhea().unwrap().ascender().to_i16(),
            native.hhea().unwrap().descender().to_i16()
        ),
        (800, -200)
    );
    let analyzed = AnalyzedSource::new(
        Source::identified(
            if upright { "一一一" } else { "ΧΧΧ" },
            Id::from_u64(1566061),
            11,
        ),
        Style {
            font: Font::new().family("Ahem"),
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            letter_spacing: tracking,
            ..Style::default()
        },
        if upright {
            AnalysisTypography::Upright {
                computed_direction: ParagraphBaseDirection::RightToLeft,
            }
        } else {
            AnalysisTypography::HorizontalComposition
        },
    )
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let selected = prepared
        .select(LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                alignment_insets: LineAlignmentInsets::ZERO,
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::InterCharacter,
            },
        })
        .unwrap();
    assert_eq!(selected.spacing_slots().len(), 2);
    (prepared, selected)
}

fn amounts(selected: &SelectedLine, amount: f64) -> SpacingAllocations {
    SpacingAllocations::Complete(
        selected
            .spacing_slots()
            .map(|slot| SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn cell(orientation: LineOrientation, edge: f64, width: f64) -> Rect {
    match orientation {
        LineOrientation::Horizontal => Rect::new(20.0 + edge, 92.0, width, 10.0),
        LineOrientation::SidewaysClockwise => Rect::new(98.0, 20.0 + edge, 10.0, width),
        LineOrientation::SidewaysCounterclockwise => {
            Rect::new(92.0, 20.0 - edge - width, 10.0, width)
        }
        LineOrientation::Vertical => Rect::new(95.0, 20.0 + edge, 10.0, width),
    }
}

fn projected(layout: &Layout, orientation: LineOrientation, gap: f64) {
    let edges = [0.0, 10.0 + gap, 20.0 + 2.0 * gap, 30.0 + 2.0 * gap];
    assert_eq!(layout.bounds(), cell(orientation, 0.0, edges[3]));
    assert_eq!(layout.source().id(), Some(Id::from_u64(1566061)));
    assert_eq!(layout.source().revision(), 11);
    let upright = orientation == LineOrientation::Vertical;
    assert_eq!(
        layout.source().text().code_units(),
        if upright { [0x4e00; 3] } else { [0x3a7; 3] }
    );
    let raw = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), font(orientation));
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance(), glyph.pen()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        edges[..3]
            .iter()
            .map(|&pen| (274, 10.0, pen))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        layout
            .run_views()
            .map(|run| run.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3)]
    );
    assert_eq!(
        layout
            .clusters()
            .iter()
            .map(|cluster| cluster.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
    let parent = NodeId(1000);
    let origin = Point::new(3.0, 7.0);
    let mut state = Accessibility::new(100);
    let update = layout
        .accessibility_update(&mut state, parent, parent, origin)
        .unwrap();
    let runs = update
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::TextRun)
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 3);
    for (index, pair) in edges.windows(2).enumerate() {
        let rect = cell(orientation, pair[0], pair[1] - pair[0]);
        let (_, node) = runs[index];
        assert_eq!(node.value(), Some(if upright { "一" } else { "Χ" }));
        assert_eq!(node.character_lengths(), [if upright { 3 } else { 2 }]);
        assert_eq!(node.character_positions(), Some([0.0_f32].as_slice()));
        assert_eq!(
            node.character_widths(),
            Some([(pair[1] - pair[0]) as f32].as_slice())
        );
        assert_eq!(
            node.bounds(),
            Some(accesskit::Rect {
                x0: rect.origin.x + 3.0,
                y0: rect.origin.y + 7.0,
                x1: rect.origin.x + rect.size.width + 3.0,
                y1: rect.origin.y + rect.size.height + 7.0,
            })
        );
        assert_eq!(node.font_family(), Some("Ahem"));
        assert_eq!(node.font_size(), Some(10.0));
        assert_eq!(
            node.text_direction(),
            Some(match orientation {
                LineOrientation::Horizontal => TextDirection::LeftToRight,
                LineOrientation::SidewaysClockwise | LineOrientation::Vertical => {
                    TextDirection::TopToBottom
                }
                LineOrientation::SidewaysCounterclockwise => TextDirection::BottomToTop,
            })
        );
        let anchor = Cursor::new(index, Affinity::After);
        let focus = Cursor::new(index + 1, Affinity::Before);
        let selected = layout.selection(Selection::new(anchor, focus)).unwrap();
        assert_eq!(selected.rects().len(), 1);
        assert_eq!(selected.rects()[0].rect(), rect);
        let access = layout
            .access_selection(&state, Selection::new(anchor, focus))
            .unwrap()
            .unwrap();
        assert_eq!(access.anchor.node, runs[index].0);
        assert_eq!(access.anchor.character_index, 0);
        assert_eq!(access.focus.node, runs[index].0);
        assert_eq!(access.focus.character_index, 1);
    }
    let whole = layout
        .access_selection(
            &state,
            Selection::new(
                Cursor::new(0, Affinity::After),
                Cursor::new(3, Affinity::Before),
            ),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        (whole.anchor.node, whole.anchor.character_index),
        (runs[0].0, 0)
    );
    assert_eq!(
        (whole.focus.node, whole.focus.character_index),
        (runs[2].0, 1)
    );
    let cursor = Cursor::new(1, Affinity::After);
    let prior = layout.access_position(&state, cursor).unwrap().unwrap();
    assert_eq!((prior.node, prior.character_index), (runs[1].0, 0));
    let error = layout
        .accessibility_update(&mut state, parent, parent, Point::new(f64::INFINITY, 7.0))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    assert_eq!(layout.access_position(&state, cursor).unwrap(), Some(prior));
    let retry = layout
        .accessibility_update(&mut state, parent, parent, origin)
        .unwrap();
    assert_eq!(
        retry.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        update.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    );
}

fn qualifies(orientation: LineOrientation) {
    let (control_prepared, control_selected) = selected(orientation, 0.0);
    let control = control_selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(control.feedback().advance, 30.0);
    projected(&place(&control_prepared, control), orientation, 0.0);
    let (prepared, selected) = selected(orientation, 4.0);
    let natural = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(natural.feedback().advance, 38.0);
    projected(&place(&prepared, natural.clone()), orientation, 4.0);
    let zero = natural.with_spacing(amounts(&selected, 0.0)).unwrap();
    projected(&place(&prepared, zero), orientation, 4.0);
    let expanded = natural.with_spacing(amounts(&selected, 2.0)).unwrap();
    assert_eq!(expanded.feedback().advance, 42.0);
    let published = place(&prepared, expanded.clone());
    assert_eq!(
        published.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
    projected(&published, orientation, 6.0);
    let error = expanded
        .with_spacing(SpacingAllocations::Complete(vec![]))
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::MissingSpacingSlot { slot: 0 },
        })
    );
    assert_eq!(expanded.feedback().advance, 42.0);
    projected(&published, orientation, 6.0);
    let restored = expanded.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().advance, 38.0);
    projected(&place(&prepared, restored), orientation, 4.0);
}

#[test]
fn allocated_horizontal_cells_reach_ax_bounds_widths_and_exact_source_selection() {
    qualifies(LineOrientation::Horizontal);
}

#[test]
fn allocated_clockwise_cells_reach_ax_with_physical_bounds_and_inline_widths() {
    qualifies(LineOrientation::SidewaysClockwise);
}

#[test]
fn allocated_upright_cells_reach_ax_without_changing_original_source_custody() {
    qualifies(LineOrientation::Vertical);
}

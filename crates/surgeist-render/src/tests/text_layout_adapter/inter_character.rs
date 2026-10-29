//! Native allocated character gaps must reach actual retained Render outlines.
//! Ahem chi274 has advance1000 and ink x0..1000/y400..600 at UPEM1000.
//! At size10, three glyphs plus two authored4/allocated2 gaps give42px.
use super::*;
use kurbo::Shape as _;
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

fn font(orientation: text::LineOrientation) -> Vec<u8> {
    if orientation != text::LineOrientation::Vertical {
        return AHEM.to_vec();
    }
    // Same real Wide/Hani -> unchanged chi outline as the upright adapter
    // fixture: only cmap changes, with no claim of a Han glyph design.
    let mut cmap = vec![0, 0, 0, 1, 0, 3, 0, 10];
    cmap.extend(12_u32.to_be_bytes());
    cmap.extend([0, 12, 0, 0]);
    cmap.extend(28_u32.to_be_bytes());
    cmap.extend(0_u32.to_be_bytes());
    cmap.extend(1_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(0x4e00_u32.to_be_bytes());
    cmap.extend(274_u32.to_be_bytes());
    replace_tables(vec![(*b"cmap", cmap)])
}

fn selected(
    orientation: text::LineOrientation,
    tracking: f64,
) -> (
    text::PreparedText,
    text::SelectedLine,
    text::System,
    Vec<u8>,
) {
    let bytes = font(orientation);
    let native = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(native.head().unwrap().units_per_em(), 1000);
    let cp: u32 = if orientation == text::LineOrientation::Vertical {
        0x4e00
    } else {
        0x3a7
    };
    assert_eq!(native.charmap().map(cp), Some(GlyphId::new(274)));
    let metrics = native.glyph_metrics(Size::unscaled(), LocationRef::default());
    assert_eq!(metrics.advance_width(GlyphId::new(274)), Some(1000.0));
    let ink = metrics.bounds(GlyphId::new(274)).unwrap();
    assert_eq!(
        [ink.x_min, ink.y_min, ink.x_max, ink.y_max],
        [0.0, 400.0, 1000.0, 600.0]
    );
    let upright = orientation == text::LineOrientation::Vertical;
    let analyzed = text::AnalyzedSource::new(
        text::Source::identified(
            if upright { "一一一" } else { "ΧΧΧ" },
            text::Id::from_u64(1566060),
            9,
        ),
        text::Style {
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            letter_spacing: tracking,
            ..style()
        },
        if upright {
            text::AnalysisTypography::Upright {
                computed_direction: text::ParagraphBaseDirection::RightToLeft,
            }
        } else {
            text::AnalysisTypography::HorizontalComposition
        },
    )
    .unwrap();
    let mut system = system(bytes.clone());
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    let end = prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let selected = prepared
        .select(text::LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: text::PreparedTermination::Ordinary(end),
            participants: prepared.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::InterCharacter,
            },
        })
        .unwrap();
    assert_eq!(selected.spacing_slots().len(), 2);
    (prepared, selected, system, bytes)
}

fn amounts(selected: &text::SelectedLine, amount: f64) -> text::SpacingAllocations {
    text::SpacingAllocations::Complete(
        selected
            .spacing_slots()
            .map(|slot| text::SpacingAllocation { slot, amount })
            .collect(),
    )
}

fn place(prepared: &text::PreparedText, line: text::FinalizedLine) -> text::Layout {
    text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared: prepared.clone(),
        coverage: text::PlacementCoverage::WholeParagraph,
        entries: vec![text::PlacementEntry::Formatted(text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn consumed(input: &text::Layout, bytes: &[u8], gap: f64) {
    let orientation = input.run_views().next().unwrap().orientation();
    let upright = orientation == text::LineOrientation::Vertical;
    let pen = [0.0, 10.0 + gap, 20.0 + 2.0 * gap];
    assert_eq!(
        input.run_views().map(|run| run.range()).collect::<Vec<_>>(),
        [text::Range::new(0, 3)]
    );
    assert_eq!(
        input
            .clusters()
            .iter()
            .map(|cluster| cluster.range())
            .collect::<Vec<_>>(),
        [
            text::Range::new(0, 1),
            text::Range::new(1, 2),
            text::Range::new(2, 3)
        ]
    );
    let raw = input
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.face().index(), 0);
            assert_eq!(run.font_size(), 10.0);
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                        glyph.pen(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw,
        pen.into_iter()
            .map(|pen| (
                274,
                if upright { -5.0 } else { 0.0 },
                if upright { 8.0 } else { 0.0 },
                10.0,
                pen,
            ))
            .collect::<Vec<_>>()
    );
    let scene =
        prepare_text_layout(input, Transform::translation(7.0, 9.0).unwrap(), limits()).unwrap();
    assert!(std::ptr::eq(scene.layout(), input));
    assert_eq!(scene.scene().commands.len(), 1);
    let (run, transform) = outline(&scene.scene().commands[0]);
    assert_eq!(run.font().bytes(), bytes);
    assert_eq!(run.font().index(), 0);
    assert_eq!(run.instance().size(), 10.0);
    assert!(run.instance().coordinates().is_empty());
    assert_eq!(run.instance().synthesis(), GlyphSynthesis::none());
    assert_eq!(
        run.glyphs(),
        pen.map(|pen| if upright {
            TextGlyph::try_new(274, -5.0, 8.0 + pen, 10.0).unwrap()
        } else {
            TextGlyph::try_new(274, pen, 0.0, 10.0).unwrap()
        })
    );
    let advance = 30.0 + 2.0 * gap;
    let (coefficients, physical_ink) = match orientation {
        text::LineOrientation::Horizontal => (
            [1.0, 0.0, 0.0, 1.0, 27.0, 109.0],
            kurbo::Rect::new(27.0, 103.0, 27.0 + advance, 105.0),
        ),
        text::LineOrientation::SidewaysClockwise => (
            [0.0, 1.0, -1.0, 0.0, 107.0, 29.0],
            kurbo::Rect::new(111.0, 29.0, 113.0, 29.0 + advance),
        ),
        text::LineOrientation::Vertical => (
            [1.0, 0.0, 0.0, 1.0, 107.0, 29.0],
            kurbo::Rect::new(102.0, 31.0, 112.0, 23.0 + advance),
        ),
    };
    assert_eq!(transform.as_array(), coefficients);
    let mut path = run.path().clone();
    path.apply_affine(kurbo::Affine::new(transform.as_array()));
    assert_eq!(path.bounding_box(), physical_ink);
}

fn qualifies(orientation: text::LineOrientation) {
    let (control_prepared, control_selected, _, control_bytes) = selected(orientation, 0.0);
    let control = control_selected
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    assert_eq!(control.feedback().advance, 30.0);
    consumed(&place(&control_prepared, control), &control_bytes, 0.0);
    let (prepared, selected, system, bytes) = selected(orientation, 4.0);
    let stats = system.stats();
    let natural = selected.finalize(text::SpacingAllocations::None).unwrap();
    assert_eq!(natural.feedback().advance, 38.0);
    consumed(&place(&prepared, natural.clone()), &bytes, 4.0);
    let zero = natural.with_spacing(amounts(&selected, 0.0)).unwrap();
    assert_eq!(zero.feedback().advance, 38.0);
    consumed(&place(&prepared, zero), &bytes, 4.0);
    let expanded = natural.with_spacing(amounts(&selected, 2.0)).unwrap();
    assert_eq!(expanded.feedback().advance, 42.0);
    let published = place(&prepared, expanded.clone());
    let expected_units = if orientation == text::LineOrientation::Vertical {
        [0x4e00; 3]
    } else {
        [0x3a7; 3]
    };
    assert_eq!(published.source().text().code_units(), expected_units);
    assert_eq!(published.source().id(), Some(text::Id::from_u64(1566060)));
    assert_eq!(published.source().revision(), 9);
    assert_eq!(
        published.analyzed_source().identity(),
        prepared.analyzed_source().identity()
    );
    consumed(&published, &bytes, 6.0);
    let error = expanded
        .with_spacing(text::SpacingAllocations::Complete(vec![]))
        .unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&text::ErrorDetail::InvalidTextProtocol {
            reason: text::ProtocolError::MissingSpacingSlot { slot: 0 },
        })
    );
    assert_eq!(expanded.feedback().advance, 42.0);
    consumed(&published, &bytes, 6.0);
    let error = refuse(
        &published,
        Transform::identity(),
        bounded(2, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 3, 2);
    consumed(&published, &bytes, 6.0);
    assert_eq!(
        system.stats(),
        stats,
        "spacing, placement and Render must not prepare or analyze again"
    );
    let restored = expanded
        .with_spacing(text::SpacingAllocations::None)
        .unwrap();
    assert_eq!(restored.feedback().advance, 38.0);
    consumed(&place(&prepared, restored), &bytes, 4.0);
}

#[test]
fn allocated_horizontal_gaps_preserve_raw_provider_data_and_actual_render_pens() {
    qualifies(text::LineOrientation::Horizontal);
}

#[test]
fn allocated_clockwise_gaps_rotate_actual_outlines_and_pens_once() {
    qualifies(text::LineOrientation::SidewaysClockwise);
}

#[test]
fn allocated_upright_gaps_keep_actual_outline_axes_and_vertical_pens() {
    qualifies(text::LineOrientation::Vertical);
}

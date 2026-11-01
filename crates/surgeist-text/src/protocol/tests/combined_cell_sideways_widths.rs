//! Real vertical Source Cells use horizontal typography in both sideways modes.
//! Packet operands and signed line maps are independent of preparation output.

use super::*;
use surgeist_shaper::{
    Direction, Feature, FontRef, ShaperData, ShapingStatus, Tag, UnicodeBuffer, WidthFeature,
    WidthInputOutcome, WidthObservationCoverage, script,
};

pub(super) fn reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}

pub(super) fn metric_premises(system: &System, bytes: &[u8]) {
    glyph_frame_font_premises(bytes);
    // The retained fixture has only a vertical BASE axis. Horizontal Alphabetic
    // therefore uses its independently requested, documented zero fallback.
    assert_eq!(sfnt::u16_at(sfnt::table(bytes, *b"BASE"), 4), 0);
    let face = system
        .registered_font_faces()
        .find(|face| face.bytes() == bytes)
        .unwrap();
    let instance = crate::EffectiveFontInstance::qualify(face, 10.0, &[]).unwrap();
    let metrics = instance.metrics(face.catalogue_identity()).unwrap();
    assert_eq!(metrics.units_per_em(), 1000);
    assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
    let os2 = sfnt::table(bytes, *b"OS/2");
    if sfnt::u16_at(os2, 62) & 128 == 0 {
        assert_eq!(
            metrics.line_metric_source(),
            crate::FontLineMetricSource::Hhea
        );
    } else {
        assert_eq!(
            metrics.line_metric_source(),
            crate::FontLineMetricSource::Os2Typographic
        );
        assert_eq!(sfnt::u16_at(os2, 68), 800);
        assert_eq!(
            sfnt::u16_at(os2, 70),
            u16::from_be_bytes((-200_i16).to_be_bytes())
        );
    }
    let baseline = instance
        .baseline(
            face.catalogue_identity(),
            reference().script,
            FontBaselineAxis::HorizontalY,
            FontBaseline::Alphabetic,
        )
        .unwrap();
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("real horizontal Alphabetic reference");
    };
    assert_eq!(value.coordinate(), 0.0);
}

fn packet(
    bytes: &[u8],
    text: &str,
    feature: Option<([u8; 4], WidthFeature)>,
    glyph: u32,
    advance: i32,
) {
    let font = FontRef::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    let mut input = UnicodeBuffer::new();
    for (index, ch) in text.chars().enumerate() {
        input.add(ch, u32::try_from(index).unwrap());
    }
    input.set_direction(Direction::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let features = if let Some((tag, observation)) = feature {
        input.observe_width_feature(observation);
        vec![Feature::new(Tag::new(&tag), 1, ..)]
    } else {
        vec![]
    };
    let output = shaper.shape(input, &features);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .map(|g| (g.glyph_id, g.cluster))
            .collect::<Vec<_>>(),
        (0..u32::try_from(text.len()).unwrap())
            .map(|index| (glyph, index))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        output
            .glyph_positions()
            .iter()
            .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
            .collect::<Vec<_>>(),
        vec![(advance, 0, 0, 0); text.len()]
    );
    if let Some((_, feature)) = feature {
        let receipt = output.width_feature_receipt().unwrap().unwrap();
        assert_eq!(receipt.feature(), feature);
        assert_eq!(receipt.coverage(), WidthObservationCoverage::Complete);
        assert_eq!(receipt.input_len(), text.len());
        for index in 0..text.len() {
            assert_eq!(
                receipt.input(index).unwrap().outcome(),
                WidthInputOutcome::Covered
            );
        }
    }
}

pub(super) fn select(prepared: &PreparedText, policy: SpacingPolicy) -> SelectedLine {
    let segments = prepared.source_segments().collect::<Vec<_>>();
    let [segment] = segments.as_slice() else {
        panic!("one actual source segment");
    };
    prepared
        .select(LineSelectionRequest {
            inline_boxes: None,
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
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
                orientation: prepared.orientation(),
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: policy,
            },
        })
        .unwrap()
}

pub(super) fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
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

pub(super) fn ordinary_control(system: &mut System, orientation: LineOrientation, parent: &[u8]) {
    let source = Source::identified("B", Id::from_u64(784_992_701), 1);
    let extent = Range::new(0, source.text().len());
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: Style {
                font: Font::new().family("Ahem"),
                size: 10.0,
                line_height: LineHeight::Absolute(10.0),
                white_space: WhiteSpace::Preserve,
                wrap: Wrap::Word,
                ..Style::default()
            },
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            origins: vec![ParagraphOrigin {
                source: source.clone(),
                text_node: None,
                local: extent,
                aggregate: extent,
                kind: ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.combined_cells().len(), 0);
    let analyzed = crate::test_wrapping::bind_missing(&analyzed, Wrap::Word);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    let line = select(&prepared, SpacingPolicy::None)
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(line.cell_views().count(), 0);
    let layout = place(&prepared, line);
    let runs = layout.glyph_runs();
    let [run] = runs.as_slice() else {
        panic!("one genuine ordinary B packet");
    };
    assert_eq!(run.font().bytes(), parent);
    assert_eq!(
        run.glyphs()
            .iter()
            .map(|g| (g.id(), g.advance()))
            .collect::<Vec<_>>(),
        [(36, 10.0)]
    );
    let sign = if orientation == LineOrientation::SidewaysClockwise {
        1.0
    } else {
        -1.0
    };
    assert_eq!(run.glyph_frame().outline_x(), Point::new(0.0, sign));
    assert_eq!(run.glyph_frame().outline_y(), Point::new(-sign, 0.0));
}

pub(super) fn square_x(orientation: LineOrientation) -> f64 {
    match orientation {
        LineOrientation::SidewaysClockwise => 98.0,
        LineOrientation::SidewaysCounterclockwise => 92.0,
        _ => panic!("sideways test profile"),
    }
}

pub(super) fn square_y(orientation: LineOrientation, start: f64) -> f64 {
    match orientation {
        LineOrientation::SidewaysClockwise => 20.0 + start,
        LineOrientation::SidewaysCounterclockwise => 10.0 - start,
        _ => panic!("sideways test profile"),
    }
}

pub(super) fn receipt(prepared: &PreparedText, processed: &ProcessedSource) {
    let scope = processed.captured_scope(1).unwrap();
    let metrics = prepared.scope_metrics(&scope).unwrap();
    assert_eq!(metrics.font_reference(), reference());
    // The captured named reference does not change default preparation's
    // native raw-design-zero option into an explicit Alphabetic request.
    assert_eq!(metrics.common_baseline_reference(), None);
    assert_eq!(metrics.orientation(), prepared.orientation());
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!((metrics.over(), metrics.under()), (8.0, 2.0));
    assert_eq!(metrics.native_reference_coordinate(), 0.0);
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual prepared Cell");
    };
    assert_eq!(
        (cell.extent(), cell.before(), cell.after()),
        (10.0, 8.0, 2.0)
    );
    assert_eq!(cell.source_cell().scope(), scope);
    assert!(cell.has_provider_input());
}

fn cases(orientation: LineOrientation) {
    for (text, glyph, raw_advance, scale) in [
        ("A", 35, 10.0, 1.0),
        ("AA", 36, 5.0, 1.0),
        ("AAAA", 16, 3.0, 5.0 / 6.0),
    ] {
        let (mut system, processed, analyzed, bytes) = source_with_profile(
            text,
            &[],
            composition_font(),
            AnalysisTypography::HorizontalComposition,
            reference(),
        );
        assert_eq!(
            processed.captured_scope(1).unwrap().writing_axis(),
            Some(SourceWritingAxis::Vertical)
        );
        assert_eq!(
            analyzed.typography(),
            AnalysisTypography::HorizontalComposition
        );
        metric_premises(&system, &bytes);
        ordinary_control(&mut system, orientation, &parent_font());
        packet(&bytes, text, None, 35, 1000);
        if text != "A" {
            for (tag, feature, glyph, advance) in [
                (*b"hwid", WidthFeature::HalfWidth, 36, 500),
                (*b"twid", WidthFeature::ThirdWidth, 58, 400),
                (*b"qwid", WidthFeature::QuarterWidth, 16, 300),
            ] {
                packet(&bytes, text, Some((tag, feature)), glyph, advance);
            }
        }
        // A fits unchanged; AA's widest fitting covered packet is hwid10.
        // AAAA's narrowest covered qwid12 needs horizontal scale10/12.
        let prepared = system
            .prepare(
                &analyzed,
                PreparationOptions::default().with_orientation(orientation),
            )
            .expect(
                "authentic vertical Source Cell must prepare with sideways horizontal typography",
            );
        assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
        receipt(&prepared, &processed);
        let selected = select(&prepared, SpacingPolicy::None);
        let line = selected
            .finalize(LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: ScopeAllocations::None,
            })
            .unwrap();
        one_em(&line);
        let raw = line
            .run_views()
            .flat_map(|run| {
                assert!(run.cell().is_some());
                assert_eq!(run.face().bytes(), bytes.as_slice());
                assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
                run.glyphs()
                    .map(|g| (g.range(), g.id(), g.pen(), g.advance(), g.x(), g.y()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            raw,
            (0..text.len())
                .map(|i| (
                    Range::new(i, i + 1),
                    glyph,
                    f64::from(u32::try_from(i).unwrap()) * f64::from(raw_advance),
                    raw_advance,
                    0.0,
                    0.0
                ))
                .collect::<Vec<_>>()
        );
        for line in [
            line.clone(),
            line.with_spacing(SpacingAllocations::None).unwrap(),
        ] {
            let layout = place(&prepared, line);
            assert_eq!(layout.source(), processed.source());
            assert_eq!(
                layout.lines()[0].bounds(),
                Rect::new(
                    square_x(orientation),
                    square_y(orientation, 0.0),
                    10.0,
                    10.0
                )
            );
            let packets = layout.glyph_runs();
            assert_eq!(packets.len(), 1);
            let run = &packets[0];
            let frame = run.glyph_frame();
            assert_eq!(run.font().bytes(), bytes.as_slice());
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
            close(frame.outline_x().x, scale);
            assert_eq!(frame.outline_x().y, 0.0);
            assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
            assert_eq!(
                frame.origin(),
                Point::new(square_x(orientation), square_y(orientation, 0.0) + 8.0)
            );
            for (i, g) in run.glyphs().iter().enumerate() {
                close(
                    frame.origin().x + frame.outline_x().x * g.pen(),
                    square_x(orientation)
                        + scale * f64::from(u32::try_from(i).unwrap()) * f64::from(raw_advance),
                );
            }
        }
    }
}

#[test]
fn clockwise_cells_keep_normal_hwid_and_compressed_packets_physically_horizontal() {
    cases(LineOrientation::SidewaysClockwise);
}
#[test]
fn counterclockwise_cells_keep_normal_hwid_and_compressed_packets_physically_horizontal() {
    cases(LineOrientation::SidewaysCounterclockwise);
}

//! Exterior OOF static sites use actual Cell outer edges, with genuine
//! captured scope references, source affinity, caller placement and replay.
use super::*;
use skrifa::{GlyphId, MetadataProvider, raw::TableProvider};

fn native_reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}

fn native_font() -> Vec<u8> {
    // A matching Native CW profile genuinely uses horizontal metrics. Reuse
    // the actual registered Ahem derivative, without adding unused V/BASE data.
    let bytes = crate::vertical_test_fonts::horizontal(800, -200, 0, 1000);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(
        face.localized_strings(skrifa::string::StringId::FAMILY_NAME)
            .english_or_first()
            .unwrap()
            .to_string(),
        "Ahem"
    );
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(face.hhea().unwrap().descender().to_i16(), -200);
    assert_eq!(face.os2().unwrap().fs_selection().bits() & 0x80, 0);
    assert!(face.fvar().is_err());
    assert!(face.mvar().is_err());
    for (ch, id) in [('A', 35), ('B', 36)] {
        assert_eq!(face.charmap().map(ch), Some(GlyphId::new(id)));
        assert_eq!(
            crate::test_fonts::u16_at(crate::test_fonts::table(&bytes, *b"hmtx"), id as usize * 4),
            1000
        );
    }
    use surgeist_shaper::{Direction, FontRef, ShaperData, ShapingStatus, UnicodeBuffer, script};
    let font = FontRef::new(&bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut input = UnicodeBuffer::new();
    input.add('A', 0);
    input.add('B', 1);
    input.set_direction(Direction::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let output = shaper.shape(input, &[]);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .map(|g| (g.glyph_id, g.cluster))
            .collect::<Vec<_>>(),
        [(35, 0), (36, 1)]
    );
    assert_eq!(
        output
            .glyph_positions()
            .iter()
            .map(|p| (p.x_advance, p.y_advance, p.x_offset, p.y_offset))
            .collect::<Vec<_>>(),
        [(1000, 0, 0, 0); 2]
    );
    bytes
}

fn select_native(prepared: &PreparedText, atomic: PreparedAtomic) -> SelectedLine {
    let mut segments = prepared.source_segments();
    let segment = segments.next().unwrap();
    assert!(segments.next().is_none());
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
            atomics: vec![AtomicAllocation {
                occurrence: atomic,
                allocation: AtomicLineAllocation::OutOfFlow,
            }],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: LineOrientation::SidewaysClockwise,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                alignment_insets: LineAlignmentInsets::ZERO,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
}

fn verify_native_site(
    prepared: &PreparedText,
    line: FinalizedLine,
    atomic: &PreparedAtomic,
    bytes: &[u8],
    tracking: f64,
) {
    let step = 10.0 + tracking;
    // Text3 tracking occurs only between these two outer units, not beyond
    // the complete paragraph edges: two E10 squares plus one authored gap.
    assert_eq!(line.feedback().advance, 20.0 + tracking);
    let cells = line.cell_views().collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    assert_eq!(cells[0].square().origin.x, 0.0);
    assert_eq!(cells[1].square().origin.x, step);
    for cell in &cells {
        assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    }
    let slots = line.atomic_slots().collect::<Vec<_>>();
    let [slot] = slots.as_slice() else {
        panic!("one actual exterior OOF slot");
    };
    assert_eq!(&slot.occurrence, atomic);
    assert_eq!(slot.allocation, AtomicLineAllocation::OutOfFlow);
    assert_eq!(slot.scope_displacement.get(), 0.0);
    // Both affinities choose this same exterior flow edge, including authored
    // outer tracking, not the following
    // Cell's intrinsic inner glyph origin0. The common reference is not a glyph
    // baseline; no inner allocation or fabricated zero-size box is supplied.
    assert_eq!(slot.origin.x, step);
    assert_eq!(
        slot.origin.x,
        cells[0].square().origin.x + cells[0].square().size.width + tracking
    );
    assert_eq!(slot.origin.x, cells[1].square().origin.x);
    assert_eq!(slot.origin.y, line.feedback().baseline);
    let mut glyphs = vec![];
    for run in line.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        assert!(run.cell().is_some());
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.font_size(), 10.0);
        for glyph in run.glyphs() {
            assert_eq!((glyph.advance(), glyph.x(), glyph.y()), (10.0, 0.0, 0.0));
            assert_eq!(
                run.origin().x + glyph.pen(),
                0.0,
                "each one-glyph Cell has its own intrinsic inner origin"
            );
            glyphs.push(glyph.id());
        }
    }
    assert_eq!(glyphs, [35, 36]); // OOF contributes no provider glyph.
    // CW maps logical slot(step, common baseline) to physical(100,20+step) at the
    // supplied line anchor. This Native consuming caller chooses the actual
    // OOF border/content at that site; Text does not perform CSS abs positioning.
    let border = Rect::new(100.0, 20.0 + step, 5.0, 10.0);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::SidewaysClockwise,
            atomics: vec![AtomicPlacement {
                occurrence: atomic.clone(),
                border,
                content: border,
                baseline: 100.0,
            }],
        })],
    })
    .unwrap();
    let boxes = layout.inline_boxes();
    let [box_] = boxes.as_slice() else {
        panic!("one real placed OOF");
    };
    assert_eq!(box_.id(), Id::from_u64(784_994_001));
    assert_eq!(box_.kind(), InlineBoxKind::OutOfFlow);
    assert_eq!(box_.index(), 1);
    assert_eq!(box_.line(), 0);
    assert_eq!(box_.rect(), border);
    let mut origins = vec![];
    for run in layout.run_views() {
        if run.glyphs().next().is_none() {
            continue;
        }
        let frame = run.glyph_frame();
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        for glyph in run.glyphs() {
            origins.push((glyph.id(), frame.origin()));
        }
    }
    assert_eq!(
        origins,
        [
            (35, Point::new(98.0, 28.0)),
            (36, Point::new(98.0, 28.0 + step))
        ]
    );
}

fn qualify_native_site(affinity: Affinity, tracking: f64) {
    let bytes = native_font();
    let reference = native_reference();
    let processed = out_of_flow_capture(
        "AB",
        TextCombineUpright::All,
        affinity,
        Some((reference, tracking)),
    );
    assert_eq!(
        processed.captured_scope(0).unwrap().font_reference(),
        Some(reference)
    );
    assert!(
        processed
            .captured_scope(0)
            .unwrap()
            .scope_alignment_limits()
            .is_some()
    );
    let domain = AnalyzedDomain::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert!(Arc::ptr_eq(domain.processed_source(), &processed));
    assert_eq!(domain.outputs().len(), 1);
    let source = domain.outputs().next().unwrap();
    assert_eq!(source.combined_cells().len(), 2);
    let _ = preserved_oof(source, &processed, affinity);
    let analyzed = crate::test_wrapping::bind_missing(source, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default()
                .with_orientation(LineOrientation::SidewaysClockwise)
                .with_baseline_reference(FontBaseline::Alphabetic),
        )
        .unwrap();
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(0).unwrap())
        .unwrap();
    assert_eq!(metrics.font_reference(), reference);
    assert_eq!(metrics.font_size(), 10.0);
    let instance = metrics.effective_instance().unwrap();
    assert_eq!(instance.face().bytes(), bytes.as_slice());
    assert_eq!(instance.axis_count(), 0);
    let horizontal = instance
        .metrics(instance.face().catalogue_identity())
        .unwrap();
    assert_eq!(horizontal.line_metric_source(), FontLineMetricSource::Hhea);
    assert_eq!((horizontal.ascent(), horizontal.descent()), (8.0, -2.0));
    let baseline = instance
        .baseline(
            instance.face().catalogue_identity(),
            reference.script,
            reference.axis,
            reference.baseline,
        )
        .unwrap();
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("genuine Alphabetic0");
    };
    assert_eq!(value.coordinate(), 0.0);
    assert_eq!(
        value.provenance(),
        FontBaselineProvenance::HorizontalAlphabeticZero
    );
    let cells = prepared
        .cell_views()
        .map(|view| {
            assert_eq!(
                (view.extent(), view.before(), view.after()),
                (10.0, 8.0, 2.0)
            );
            assert!(view.has_provider_input());
            view.handle()
        })
        .collect::<Vec<_>>();
    assert_eq!(cells.len(), 2);
    let mut atomics = prepared.atomic_views();
    let view = atomics.next().unwrap();
    assert!(atomics.next().is_none());
    assert_eq!(
        view.capture_position(),
        Some(processed.captured_atomic(0).unwrap())
    );
    assert_eq!(view.original_box().id(), Id::from_u64(784_994_001));
    assert_eq!(view.original_box().kind(), InlineBoxKind::OutOfFlow);
    let atomic = view.handle();
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [
            PreparedParticipant::Cell(cells[0].clone()),
            PreparedParticipant::Atomic(atomic.clone()),
            PreparedParticipant::Cell(cells[1].clone()),
        ]
    );
    let line = select_native(&prepared, atomic.clone())
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    verify_native_site(&prepared, line.clone(), &atomic, &bytes, tracking);
    drop(system);
    verify_native_site(
        &prepared,
        line.with_spacing(SpacingAllocations::None).unwrap(),
        &atomic,
        &bytes,
        tracking,
    );
}

#[test]
fn native_before_affinity_uses_the_preceding_cell_outer_edge() {
    qualify_native_site(Affinity::Before, 0.0);
}

#[test]
fn native_after_affinity_uses_the_following_cell_outer_edge() {
    qualify_native_site(Affinity::After, 0.0);
}

#[test]
fn native_before_affinity_uses_the_cell_flow_endpoint_after_real_tracking() {
    qualify_native_site(Affinity::Before, 2.0);
}

#[test]
fn native_after_affinity_uses_the_following_cell_start_after_real_tracking() {
    qualify_native_site(Affinity::After, 2.0);
}

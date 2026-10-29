//! Failures exercised through actual public upright preparation/selection.
use super::*;
use crate::{
    Error, ErrorCode, ErrorDetail, FontBaselineScript, FontMetricTable, FontMetricTableError,
    ParagraphTabReference, TabPreparation, TabSize, TextStyleOwner, UprightContent,
    UprightContentPiece, UprightMetricReference, UprightTabMeasurement,
    VerticalOriginReferenceReason, VerticalShapingContextReason,
};

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        ..Style::default()
    }
}

fn paragraph(source: Source, base_style: Style) -> ParagraphInput {
    let length = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, length),
            aggregate: Range::new(0, length),
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if length == 0 {
            vec![crate::ParagraphAnchor {
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
                affinity: crate::Affinity::After,
            }]
        } else {
            vec![]
        },
    }
}

fn prepare(
    bytes: &[u8],
    input: ParagraphInput,
    tabs: TabPreparation,
) -> crate::Result<PreparedText> {
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            input,
            AnalysisTypography::Upright {
                computed_direction: ParagraphBaseDirection::RightToLeft,
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analysis.direction(), Direction::LeftToRight);
    system_with(bytes).prepare(
        &analysis,
        PreparationOptions::try_new(1.0, tabs)
            .unwrap()
            .with_orientation(LineOrientation::Vertical),
    )
}

fn presentation(content: &UprightContent) -> &[UprightContentPiece] {
    let UprightContent::Presentation(value) = content else {
        panic!("real presentation must retain actual ordered content")
    };
    value.pieces()
}

fn supplier(error: &Error, glyph: u32) -> &UprightContent {
    // The nominal metric owner rejects this actual post-GSUB glyph before any
    // outline/table lookup: maxp inventory remains exactly281.
    assert_eq!(error.code, ErrorCode::InvalidRange);
    let Some(ErrorDetail::VerticalMetricSupplier { content, glyph_id }) = error.detail() else {
        panic!("actual nominal supplier failure expected: {error:?}")
    };
    assert_eq!(*glyph_id, glyph);
    let original = std::error::Error::source(error)
        .unwrap()
        .downcast_ref::<Error>()
        .unwrap();
    assert_eq!(original.code, ErrorCode::InvalidRange);
    assert_eq!(original.message, "glyph outside retained face inventory");
    assert_eq!(
        original.detail(),
        Some(&ErrorDetail::FontGlyphRange { glyph, count: 281 })
    );
    assert!(std::error::Error::source(original).is_none());
    content
}

fn substitution(from: u16, to: u16) -> Vec<u8> {
    let gsub = ot_layout(
        *b"latn",
        vec![(*b"vert", vec![0])],
        vec![(1, 0, single(from, to))],
    );
    with_native_metrics(&sfnt::replace_tables(AHEM, vec![(*b"GSUB", gsub)]))
}

fn qualify_substitution(bytes: &[u8], scalar: char, from: u16, to: u16) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(
        font.charmap().map(scalar).unwrap().to_u32(),
        u32::from(from)
    );
    assert!(font.glyf().is_ok());
    let font = surgeist_shaper::FontRef::from_index(bytes, 0).unwrap();
    let data = surgeist_shaper::ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    input.add(scalar, 0);
    input.set_direction(surgeist_shaper::Direction::TopToBottom);
    input.set_script(surgeist_shaper::script::LATIN);
    let output = shaper.shape(input, &[]);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    assert_eq!(output.glyph_infos()[0].glyph_id, u32::from(to));
}

#[test]
fn actual_post_vert_glyph_failure_keeps_the_original_typed_supplier_source() {
    let bytes = substitution(58, 9999);
    qualify_substitution(&bytes, 'X', 58, 9999);
    let mut control = style();
    control.font = control.font.feature(r#""vert" 0"#);
    prepare(
        &bytes,
        paragraph(Source::new("X"), control),
        TabPreparation::FontAdvance,
    )
    .unwrap();
    let error = prepare(
        &bytes,
        paragraph(Source::new("X"), style()),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(
        presentation(supplier(&error, 9999)),
        [UprightContentPiece::Source {
            range: Range::new(0, 1)
        }]
    );
}

#[test]
fn space_and_zero_tab_supplier_failures_keep_their_distinct_operation_and_style_owners() {
    for (scalar, size, role, owner) in [
        (
            ' ',
            TabSize::spaces(4.0).unwrap(),
            UprightTabMeasurement::SpaceInterval,
            TextStyleOwner::TabReference,
        ),
        (
            '0',
            TabSize::length(40.0).unwrap(),
            UprightTabMeasurement::ZeroThreshold,
            TextStyleOwner::Source {
                range: Range::new(1, 2),
            },
        ),
    ] {
        let original = skrifa::FontRef::new(AHEM).unwrap();
        let from = u16::try_from(original.charmap().map(scalar).unwrap().to_u32()).unwrap();
        let bytes = substitution(from, 9999);
        qualify_substitution(&bytes, scalar, from, 9999);
        // X and the glyph-free default strut remain valid; only the demanded
        // source-free hypothetical character executes the bad substitution.
        prepare(
            &bytes,
            paragraph(Source::new("X"), style()),
            TabPreparation::FontAdvance,
        )
        .unwrap();
        let mut tab_style = style();
        tab_style.tab_size = size;
        tab_style.letter_spacing = 1.0;
        let mut source = Source::new("X\t");
        source.span(Range::new(1, 2), tab_style);
        let mut input = paragraph(source, style());
        input.tab_reference =
            Some(ParagraphTabReference::new(style(), ParagraphBaseDirection::RightToLeft).unwrap());
        assert_eq!(
            input.tab_reference.as_ref().unwrap().direction(),
            ParagraphBaseDirection::RightToLeft
        );
        let error = prepare(&bytes, input, TabPreparation::CssStops).unwrap_err();
        assert_eq!(
            supplier(&error, 9999),
            &UprightContent::TabMeasurement { role, owner }
        );
    }
}

#[test]
fn a_real_selected_morx_plan_has_a_precise_upright_context_refusal() {
    // Version2, one all-directions noncontextual format8 subtable, X58→82.
    let mut morx = words(&[2, 0]);
    morx.extend(
        [1_u32, 1, 36, 0, 1, 20, 0x2000_0004, 1]
            .into_iter()
            .flat_map(u32::to_be_bytes),
    );
    morx.extend(words(&[8, 58, 1, 82]));
    let bytes = with_native_metrics(&sfnt::replace_tables(
        &sfnt::remove_tables(AHEM, &[*b"GSUB", *b"GPOS"]),
        vec![(*b"morx", morx)],
    ));
    let parsed = skrifa::FontRef::new(&bytes).unwrap();
    assert!(parsed.morx().is_ok());
    assert!(parsed.gsub().is_err());
    qualify_substitution(&bytes, 'X', 58, 82);
    let error = prepare(
        &bytes,
        paragraph(Source::new("X"), style()),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::ShapingRefused);
    let Some(ErrorDetail::VerticalShapingContext { content, reason }) = error.detail() else {
        panic!("actual native context: {error:?}")
    };
    assert_eq!(*reason, VerticalShapingContextReason::Morx);
    assert_eq!(
        presentation(content),
        [UprightContentPiece::Source {
            range: Range::new(0, 1)
        }]
    );
}

fn fractional_reference() -> Vec<u8> {
    sfnt::replace_tables(
        &base::two_axis_ahem(),
        vec![(
            *b"BASE",
            base::base(
                1,
                None,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"idtp"],
                    scripts: vec![base::Script {
                        tag: *b"latn",
                        default_index: 0,
                        coordinates: vec![
                            base::Coord::Design(0),
                            base::Coord::Adjusted {
                                coordinate: 1000,
                                adjustment: Some(base::variation_index(0, 0)),
                            },
                        ],
                    }],
                }),
                Some(base::region_store(
                    &[[0, 16384, 16384], [0, 16384, 16384]],
                    3,
                )),
            ),
        )],
    )
}

#[test]
fn valid_finer_grid_base_refuses_provider_precision_without_becoming_malformed_font_data() {
    let bytes = fractional_reference();
    let system = system_with(&bytes);
    let data = system.registered_font_faces().next().unwrap().to_owned();
    let coordinates = [8192, 8192];
    let instance = data.unsynthesized_instance(1000.0, &coordinates).unwrap();
    let observation = instance
        .upright_metrics(
            system.font_catalogue_identity(),
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!(observation.central_coordinate_or_distance(), 500.375);
    assert_eq!(observation.reference(), UprightMetricReference::FontDesignX);
    let mut authored = style();
    authored.font = authored
        .font
        .variation(r#""wght" 0.5"#)
        .variation(r#""wdth" 0.5"#);
    let error = prepare(
        &bytes,
        paragraph(Source::new("X"), authored.clone()),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    let Some(ErrorDetail::VerticalOriginReference {
        content,
        reference,
        value,
        reason,
    }) = error.detail()
    else {
        panic!("provider reference precision: {error:?}")
    };
    assert_eq!(
        (*reference, *value, *reason),
        (
            UprightMetricReference::FontDesignX,
            500.375,
            VerticalOriginReferenceReason::NotHalfUnit
        )
    );
    assert_eq!(
        presentation(content),
        [UprightContentPiece::Source {
            range: Range::new(0, 1)
        }]
    );
    let malformed = sfnt::replace_tables(&bytes, vec![(*b"BASE", vec![0])]);
    let error = prepare(
        &malformed,
        paragraph(Source::new("X"), authored),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidFontMetrics);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::FontMetricTable {
            table: FontMetricTable::Base,
            reason: FontMetricTableError::Malformed
        })
    );
}

fn bad_joint_mark_font() -> Vec<u8> {
    // ContextSubst format3 sees the complete actual A+acute EGC; its nested
    // single lookup substitutes A only in that context, never a standalone A.
    let context = words(&[3, 2, 1, 14, 20, 0, 1, 1, 1, 35, 1, 1, 100]);
    let gsub = ot_layout(
        *b"latn",
        vec![(*b"ccmp", vec![0])],
        vec![(5, 0, context), (1, 0, single(35, 9999))],
    );
    let font = with_native_metrics(&sfnt::replace_tables(
        &crate::shaping_test_fonts::mark_ahem(),
        vec![(*b"GSUB", gsub)],
    ));
    let parsed = skrifa::FontRef::new(&font).unwrap();
    assert_eq!(parsed.charmap().map('A').unwrap().to_u32(), 35);
    assert_eq!(parsed.charmap().map('\u{301}').unwrap().to_u32(), 100);
    assert_eq!(parsed.maxp().unwrap().num_glyphs(), 281);
    assert!(parsed.gsub().unwrap().lookup_list().is_ok());
    // Original body shaping is valid; the hypothetical complete EGC is first
    // realized as a genuine indication capture, which performs no GSUB.
    prepare(
        &font,
        paragraph(Source::new("A"), style()),
        TabPreparation::FontAdvance,
    )
    .unwrap();
    font
}

fn offer(prepared: &PreparedText, at: usize) -> crate::PreparedOpportunity {
    prepared
        .opportunity_views()
        .find(|view| {
            view.kind() == PreparedBreakKind::Discretionary
                && view.boundary().view().source_before().get() == at
        })
        .unwrap()
        .handle()
}

fn through_offer(
    prepared: &PreparedText,
    start: crate::PreparedBoundary,
    end: crate::PreparedOpportunity,
    incoming: Option<crate::HyphenationContinuation>,
) -> LineSelectionRequest {
    let mut input = request(prepared);
    input.start = start;
    input.end = end.view().boundary();
    input.incoming = incoming;
    input.alternative = Some(end.view().alternatives().next().unwrap().handle());
    input.termination = PreparedTermination::Ordinary(end);
    let first = input.start.view().participant_offset();
    let last = input.end.view().participant_offset();
    input.participants = prepared
        .participants()
        .skip(first)
        .take(last - first)
        .collect();
    input.controls.clear();
    input.context.start = if first == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    input.context.end = LineEnd::Soft;
    input.context.orientation = LineOrientation::Vertical;
    input.context.base_direction = ParagraphBaseDirection::LeftToRight;
    input.context.line_box_direction = LineBoxDirection::LeftToRight;
    input.context.alignment = crate::Alignment::Start;
    input.context.available_inline = None;
    input.context.spacing = SpacingPolicy::None;
    input
}

#[test]
fn a_genuine_body_and_indication_mark_egc_failure_retains_both_ordered_pieces() {
    let bytes = bad_joint_mark_font();
    let mut authored = style();
    authored.hyphens = crate::Hyphens::Manual;
    let mut input = paragraph(Source::new("AA\u{ad}XXX"), authored);
    input.replacements.push(
        crate::ParagraphReplacement::resolved_native_logical_before(
            crate::ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(2, 3),
            },
            3,
            vec!["\u{301}".into()],
        )
        .unwrap(),
    );
    let prepared = prepare(&bytes, input, TabPreparation::FontAdvance).unwrap();
    assert_eq!(prepared.alternative_views().count(), 1);
    let counts = prepared
        .hyphenation_cut_views()
        .next()
        .unwrap()
        .counts()
        .unwrap();
    assert_eq!((counts.total, counts.before, counts.after), (5, 2, 3));
    let input = through_offer(
        &prepared,
        prepared.boundary_views().next().unwrap().handle(),
        offer(&prepared, 3),
        None,
    );
    let error = prepared.select(input).unwrap_err();
    assert_eq!(
        presentation(supplier(&error, 9999)),
        [
            UprightContentPiece::Source {
                range: Range::new(1, 2)
            },
            UprightContentPiece::Indication {
                alternative: 0,
                range: Range::new(0, 1),
                insertion_anchor: 3
            },
        ]
    );
}

fn tall_mark_font() -> Vec<u8> {
    let source = with_native_metrics(&crate::shaping_test_fonts::mark_ahem());
    let original_glyf = sfnt::table(&source, *b"glyf");
    let original_loca = sfnt::table(&source, *b"loca");
    assert_eq!(sfnt::u16_at(sfnt::table(&source, *b"head"), 50), 0);
    let count = usize::from(sfnt::u16_at(sfnt::table(&source, *b"maxp"), 4));
    // A real eight-point rectangle, with collinear intermediate points so all
    // stored signed16 deltas are representable across y[-32768,32767].
    let mut mark = words(&[1, 0, (-32768_i16) as u16, 1000, 32767, 7, 0]);
    mark.extend([1_u8; 8]);
    for delta in [0_i16, 1000, 0, 0, 0, -1000, 0, 0] {
        mark.extend(delta.to_be_bytes());
    }
    for delta in [-32768_i16, 0, 32767, 32767, 1, 0, -32767, -32767] {
        mark.extend(delta.to_be_bytes());
    }
    let mut glyf = Vec::new();
    let mut loca = Vec::new();
    for glyph in 0..count {
        loca.extend(words(&[offset(glyf.len() / 2)]));
        if glyph == 100 {
            glyf.extend(&mark);
        } else {
            let first = 2 * usize::from(sfnt::u16_at(original_loca, 2 * glyph));
            let last = 2 * usize::from(sfnt::u16_at(original_loca, 2 * (glyph + 1)));
            glyf.extend(&original_glyf[first..last]);
        }
    }
    loca.extend(words(&[offset(glyf.len() / 2)]));
    let mut maxp = sfnt::table(&source, *b"maxp").to_vec();
    let max_points = sfnt::u16_at(&maxp, 6).max(8);
    maxp[6..8].copy_from_slice(&max_points.to_be_bytes());
    let mut vmtx = sfnt::table(&source, *b"vmtx").to_vec();
    vmtx[100 * 4 + 2..100 * 4 + 4].copy_from_slice(&(-32768_i16).to_be_bytes());
    let mut vhea = sfnt::table(&source, *b"vhea").to_vec();
    // Actual mark extrema: TSB=-32768, bottom=1200-(-32768)-65535,
    // extent=-32768+65535. Other glyphs keep their qualified100 TSB.
    for (at, value) in [(12, -32768_i16), (14, -31567), (16, 32767)] {
        vhea[at..at + 2].copy_from_slice(&value.to_be_bytes());
    }
    let source = sfnt::remove_tables(&source, &[*b"GPOS", *b"GSUB"]);
    sfnt::replace_tables(
        &source,
        vec![
            (*b"glyf", glyf),
            (*b"loca", loca),
            (*b"maxp", maxp),
            (*b"vmtx", vmtx),
            (*b"vhea", vhea),
        ],
    )
}

#[test]
fn actual_overlong_combining_context_preserves_the_first_provider_refusal() {
    let bytes = tall_mark_font();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert!(font.gpos().is_err());
    assert!(font.gsub().is_err());
    assert_eq!(font.charmap().map('\u{301}').unwrap().to_u32(), 100);
    let bounds = font
        .glyph_metrics(Size::unscaled(), LocationRef::default())
        .bounds(GlyphId::new(100))
        .unwrap();
    assert_eq!(
        [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
        [0.0, -32768.0, 1000.0, 32767.0]
    );
    let header = font.vhea().unwrap();
    assert_eq!(
        (
            header.min_top_side_bearing().to_i16(),
            header.min_bottom_side_bearing().to_i16(),
            header.y_max_extent().to_i16()
        ),
        (-32768, -31567, 32767)
    );
    // Native advance1200 and originY32767+TSB(-32768) fit exactly. The first
    // provider refusal still belongs to the actual combining-context limit.
    let system = system_with(&bytes);
    let data = system.registered_font_faces().next().unwrap().to_owned();
    let instance = data.unsynthesized_instance(1000.0, &[]).unwrap();
    let nominal = instance
        .vertical_glyph_metrics(100, system.font_catalogue_identity())
        .unwrap();
    assert_eq!((nominal.advance(), nominal.origin_y()), (1200.0, -1.0));
    prepare(
        &bytes,
        paragraph(Source::new("A\u{301}"), style()),
        TabPreparation::FontAdvance,
    )
    .unwrap();
    let text = format!("A{}", "\u{301}".repeat(16400));
    let error = prepare(
        &bytes,
        paragraph(Source::new(text.as_str()), style()),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::ShapingRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ShapingRefused {
            reason: crate::ShapingRefusalReason::ContextLimit
        })
    );
}

#[test]
fn a_valid_large_base_reference_has_a_numeric_provider_range_failure_and_glyph_free_control() {
    // ItemVariationData LONG_WORDS encodes one real signed32 delta for wght1.
    let mut store = words(&[1]);
    store.extend(12_u32.to_be_bytes());
    store.extend(words(&[1]));
    store.extend(22_u32.to_be_bytes());
    store.extend(words(&[1, 1, 0, 16384, 16384, 1, 0x8001, 1, 0]));
    store.extend(i32::MAX.to_be_bytes());
    let axis = base::Axis {
        tags: vec![*b"ideo", *b"idtp"],
        scripts: vec![base::Script {
            tag: *b"latn",
            default_index: 0,
            coordinates: vec![
                base::Coord::Design(0),
                base::Coord::Adjusted {
                    coordinate: 1000,
                    adjustment: Some(base::variation_index(0, 0)),
                },
            ],
        }],
    };
    let bytes = sfnt::replace_tables(
        &sfnt::variable_ahem(),
        vec![(*b"BASE", base::base(1, None, Some(axis), Some(store)))],
    );
    let system = system_with(&bytes);
    let data = system.registered_font_faces().next().unwrap().to_owned();
    let coordinates = [16384];
    let instance = data.unsynthesized_instance(1000.0, &coordinates).unwrap();
    let metrics = instance
        .upright_metrics(
            system.font_catalogue_identity(),
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!(metrics.central_coordinate_or_distance(), 1073742323.5);
    assert_eq!(
        (metrics.used_under(), metrics.used_over()),
        (-1073742323.5, 1073742323.5)
    );
    let mut authored = style();
    authored.font = authored.font.variation(r#""wght" 1"#);
    prepare(
        &bytes,
        paragraph(Source::new(""), authored.clone()),
        TabPreparation::FontAdvance,
    )
    .unwrap();
    let error = prepare(
        &bytes,
        paragraph(Source::new("X"), authored),
        TabPreparation::FontAdvance,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    let Some(ErrorDetail::VerticalOriginReference {
        content,
        value,
        reference,
        reason,
    }) = error.detail()
    else {
        panic!("actual provider reference range: {error:?}")
    };
    assert_eq!(
        (*reference, *value, *reason),
        (
            UprightMetricReference::FontDesignX,
            1073742323.5,
            VerticalOriginReferenceReason::OutOfRange
        )
    );
    assert_eq!(
        presentation(content),
        [UprightContentPiece::Source {
            range: Range::new(0, 1)
        }]
    );
}

fn native_style(indication: &str, placement: crate::HyphenationPlacement) -> Style {
    Style {
        hyphens: crate::Hyphens::Manual,
        locale: Some("und-Latn".into()),
        native_hyphenation_convention: Some(
            crate::NativeHyphenationConvention::language_script(
                "und-Latn",
                "Latn",
                indication.into(),
                placement,
            )
            .unwrap(),
        ),
        ..style()
    }
}

#[test]
fn two_distinct_indications_sharing_one_real_mark_egc_keep_both_local_ranges_and_anchors() {
    let bytes = bad_joint_mark_font();
    let mut source = Source::new("XX\u{ad}\u{ad}XX\u{ad}XX");
    source.span(
        Range::new(3, 4),
        native_style("\u{301}", crate::HyphenationPlacement::LogicalBefore),
    );
    let mut input = paragraph(
        source,
        native_style("A", crate::HyphenationPlacement::LogicalAfter),
    );
    // This genuine separately offered native alternative captures the complete
    // A+acute font query. It is never emitted or borrowed as this line's text.
    input.replacements.push(
        crate::ParagraphReplacement::resolved_native_logical_before(
            crate::ParagraphReplacementDefinition::Source {
                origin: 0,
                source: Range::new(6, 7),
            },
            7,
            vec!["A\u{301}".into()],
        )
        .unwrap(),
    );
    let prepared = prepare(&bytes, input, TabPreparation::FontAdvance).unwrap();
    assert_eq!(prepared.alternative_views().count(), 3);
    let first_offer = offer(&prepared, 3);
    let second_offer = offer(&prepared, 4);
    let first = prepared
        .select(through_offer(
            &prepared,
            prepared.boundary_views().next().unwrap().handle(),
            first_offer.clone(),
            None,
        ))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    let continuation = first
        .hyphenation_continuation()
        .expect("actual LogicalAfter has receiving-line custody");
    let error = prepared
        .select(through_offer(
            &prepared,
            first_offer.view().boundary(),
            second_offer,
            Some(continuation),
        ))
        .unwrap_err();
    assert_eq!(
        presentation(supplier(&error, 9999)),
        [
            UprightContentPiece::Indication {
                alternative: 0,
                range: Range::new(0, 1),
                insertion_anchor: 3
            },
            UprightContentPiece::Indication {
                alternative: 1,
                range: Range::new(0, 1),
                insertion_anchor: 4
            },
        ]
    );
}

fn repeated_gpos(attachment: bool) -> Vec<u8> {
    // Distinct real lookup indices may share one stored SinglePos subtable.
    // 32730 top-level applications +40 genuine contextual nested applications
    // stay in ordinary i32 range but exceed the provider's half-unit domain.
    let count = if attachment { 32722_u16 } else { 32731_u16 };
    let nested = if attachment { 49_u16 } else { 40_u16 };
    let mut scripts = words(&[1]);
    scripts.extend(*b"latn");
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0]));
    let mut features = words(&[1]);
    features.extend(*b"vkrn");
    features.extend(words(&[8, 0, count]));
    features.extend(words(&(0..count).collect::<Vec<_>>()));
    let mark = words(&[
        4,
        0,
        1,
        8,
        1,
        34,
        40,
        1,
        12,
        24,
        1,
        0,
        6,
        1,
        (-32768_i16) as u16,
        0,
        1,
        4,
        1,
        32767,
        0,
        1,
        1,
        100,
        1,
        1,
        35,
    ]);
    let first = 2 + 2 * usize::from(count);
    let shared = first + if attachment { mark.len() } else { 0 };
    let single = words(&[1, 0, 1, 8, 1, 8, 1, 32767, 1, 1, 35]);
    let mut lookups = words(&[count]);
    for index in 0..count {
        lookups.extend(words(&[offset(if attachment && index == 0 {
            first
        } else if index + 1 == count {
            shared + single.len()
        } else {
            shared
        })]));
    }
    if attachment {
        lookups.extend(mark);
    }
    lookups.extend(single);
    lookups.extend(words(&[7, 0, 1, 8, 3, 1, nested, 8 + 4 * nested]));
    for _ in 0..nested {
        lookups.extend(words(&[0, u16::from(attachment)]));
    }
    lookups.extend(words(&[1, 1, 35]));
    let mut table = words(&[
        1,
        0,
        10,
        offset(10 + scripts.len()),
        offset(10 + scripts.len() + features.len()),
    ]);
    table.extend(scripts);
    table.extend(features);
    table.extend(lookups);
    table
}

#[test]
fn actual_accumulated_gpos_offset_reports_its_half_unit_range_owner() {
    for (attachment, text, expected_stage, count) in [
        (false, "A", crate::VerticalPositionStage::Gpos, 32731),
        (
            true,
            "A\u{301}",
            crate::VerticalPositionStage::Attachment,
            32722,
        ),
    ] {
        let bytes = with_native_metrics(&sfnt::replace_tables(
            &crate::shaping_test_fonts::mark_ahem(),
            vec![(*b"GPOS", repeated_gpos(attachment))],
        ));
        let font = skrifa::FontRef::new(&bytes).unwrap();
        assert_eq!(
            font.gpos().unwrap().lookup_list().unwrap().lookups().len(),
            count
        );
        assert_eq!(font.charmap().map('A').unwrap().to_u32(), 35);
        let mut authored = style();
        authored.font = authored.font.feature(r#""vkrn" 1"#);
        let error = prepare(
            &bytes,
            paragraph(Source::new(text), authored),
            TabPreparation::FontAdvance,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::ShapingRefused);
        let Some(ErrorDetail::VerticalPositionRange { content, stage }) = error.detail() else {
            panic!("actual accumulated GPOS range error: {error:?}")
        };
        assert_eq!(*stage, expected_stage);
        assert_eq!(
            presentation(content),
            [UprightContentPiece::Source {
                range: Range::new(0, text.encode_utf16().count())
            }]
        );
    }
}

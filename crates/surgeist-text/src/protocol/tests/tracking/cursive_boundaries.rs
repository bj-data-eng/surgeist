//! Actual Arabic letters and punctuation distinguish tracking from justification.
use super::fixtures::allocations;
use super::*;
use crate::{SpacingBoundaryGeometry, SpacingSlotGeometry, SpacingSlotOwner, SpacingSlotTarget};
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, JoiningType, Script},
    script::ScriptWithExtensions,
};

fn arabic(tracking: f64) -> Style {
    Style {
        locale: Some("ar".into()),
        ..own(tracking)
    }
}

fn unicode(cp: u32, category: GeneralCategory, joining: JoiningType) {
    assert_eq!(
        CodePointMapData::<GeneralCategory>::new().get32(cp),
        category
    );
    assert_eq!(CodePointMapData::<Script>::new().get32(cp), Script::Arabic);
    assert_eq!(
        ScriptWithExtensions::new()
            .get_script_extensions_val32(cp)
            .iter()
            .collect::<Vec<_>>(),
        [Script::Arabic]
    );
    assert_eq!(CodePointMapData::<JoiningType>::new().get32(cp), joining);
}

fn font(bytes: &[u8], punctuation: bool) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(font.charmap().map('\u{621}'), Some(GlyphId::new(36)));
    assert_eq!(font.charmap().map('\u{628}'), Some(GlyphId::new(35)));
    if punctuation {
        assert_eq!(font.charmap().map('\u{60d}'), Some(GlyphId::new(82)));
    }
    for tag in [*b"hmtx", *b"hhea", *b"glyf", *b"loca"] {
        assert_eq!(sfnt::table(bytes, tag), sfnt::table(AHEM, tag));
    }
    assert!(font.table_data(skrifa::Tag::new(b"GPOS")).is_none());
    assert!(font.table_data(skrifa::Tag::new(b"fvar")).is_none());
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for id in [35, 36, 82, 100, 277] {
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    assert!(font.gsub().unwrap().lookup_list().is_ok());
    let gsub = sfnt::table(bytes, *b"GSUB");
    let scripts = usize::from(word(gsub, 4));
    assert_eq!(word(gsub, scripts), 1);
    assert_eq!(&gsub[scripts + 2..scripts + 6], b"arab");
    let features = usize::from(word(gsub, 6));
    let lookups = usize::from(word(gsub, 8));
    assert_eq!((word(gsub, features), word(gsub, lookups)), (4, 4));
    // Independently parse the real feature record, lookup and single-substitute
    // coverage/output correspondence; provider output is not the glyph oracle.
    for (ordinal, (tag, inputs, outputs)) in [
        (*b"fina", vec![35, 58], vec![82, 274]),
        (*b"init", vec![35], vec![100]),
        (*b"isol", vec![35], vec![35]),
        (*b"medi", vec![35], vec![277]),
    ]
    .into_iter()
    .enumerate()
    {
        let record = features + 2 + 6 * ordinal;
        assert_eq!(&gsub[record..record + 4], &tag);
        let feature = features + usize::from(word(gsub, record + 4));
        assert_eq!(word(gsub, feature + 2), 1);
        let index = usize::from(word(gsub, feature + 4));
        let lookup = lookups + usize::from(word(gsub, lookups + 2 + index * 2));
        assert_eq!((word(gsub, lookup), word(gsub, lookup + 4)), (1, 1));
        let sub = lookup + usize::from(word(gsub, lookup + 6));
        assert_eq!(word(gsub, sub), 2);
        assert_eq!(usize::from(word(gsub, sub + 4)), outputs.len());
        assert_eq!(
            (0..outputs.len())
                .map(|i| word(gsub, sub + 6 + 2 * i))
                .collect::<Vec<_>>(),
            outputs
        );
        let coverage = sub + usize::from(word(gsub, sub + 2));
        assert_eq!(word(gsub, coverage), 1);
        assert_eq!(usize::from(word(gsub, coverage + 2)), inputs.len());
        assert_eq!(
            (0..inputs.len())
                .map(|i| word(gsub, coverage + 4 + 2 * i))
                .collect::<Vec<_>>(),
            inputs
        );
    }
}

fn source(text: &str, second: f64) -> Source {
    let mut source = Source::identified(text, Id::from_u64(1566040), 1);
    assert_eq!(source.text().len(), 2);
    source.span(Range::new(1, 2), arabic(second));
    source
}

fn prepare(bytes: &[u8], source: &Source, first: f64) -> PreparedText {
    let extent = Range::new(0, 2);
    let input = ParagraphInput {
        source: source.clone(),
        base_style: arabic(first),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::RightToLeft,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: extent,
            aggregate: extent,
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.source(), source);
    assert_eq!(analyzed.style().locale.as_deref(), Some("ar"));
    assert_eq!(analyzed.direction(), crate::Direction::RightToLeft);
    assert_eq!(
        analyzed.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), source);
    assert!(prepared.unit_views().all(|unit| unit.level().get() == 1));
    assert_eq!(prepared.origin_views().next().unwrap().source(), source);
    prepared
}

fn select(prepared: &PreparedText, method: SpacingPolicy) -> SelectedLine {
    let mut value = request(prepared);
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    value.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    value.context.available_inline = None;
    value.context.spacing = method;
    prepared.select(value).unwrap()
}

fn sorted_raw(line: &FinalizedLine, bytes: &[u8]) -> Vec<RawGlyph> {
    let mut rows = raw(line, bytes);
    rows.sort_by_key(|glyph| glyph.0.start);
    rows
}

fn source_paints(line: &FinalizedLine) -> Vec<(Range, f64)> {
    let mut rows = line
        .run_views()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .map(move |glyph| (glyph.range(), origin.x + glyph.pen() + f64::from(glyph.x())))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|row| row.0.start);
    rows
}

fn protected(
    prepared: &PreparedText,
    line: &FinalizedLine,
    authored: f64,
    bytes: &[u8],
    expected_raw: &[RawGlyph],
) {
    let selected = line.selected_line();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let SpacingSlotTarget::Boundary {
        visual_before,
        visual_after,
    } = slots[0].target()
    else {
        panic!("actual visual endpoint units required")
    };
    let origin = prepared.origin_views().next().unwrap().handle();
    for (unit, extent) in [
        (visual_before, Range::new(1, 2)),
        (visual_after, Range::new(0, 1)),
    ] {
        assert_eq!(
            unit.pieces().map(|piece| piece.owner()).collect::<Vec<_>>(),
            [SpacingSlotOwner::Source {
                origin: origin.clone(),
                local: extent,
                aggregate: extent
            }]
        );
    }
    let row = line.spacing_slots().next().unwrap();
    assert_eq!(
        row.geometry,
        SpacingSlotGeometry::Boundary(SpacingBoundaryGeometry::ProtectedCursive)
    );
    assert_eq!(row.authored_word_spacing, 0.0);
    assert_eq!(row.authored_letter_spacing, authored);
    assert_eq!(row.advance, authored);
    assert_eq!(row.allocation, 0.0);
    assert!(!row.can_expand());
    let zero = line.with_spacing(allocations(&selected, &[0.0])).unwrap();
    assert_eq!(sorted_raw(&zero, bytes), expected_raw);
    assert_eq!(zero.feedback().advance, line.feedback().advance);
    assert_eq!(
        reason(
            &line
                .with_spacing(allocations(&selected, &[2.0]))
                .unwrap_err()
        ),
        &ProtocolError::IneligibleSpacingSlot { slot: 0 }
    );
    assert_eq!(sorted_raw(line, bytes), expected_raw);
    assert_eq!(line.selected_line(), selected);
}

#[test]
fn same_script_joining_and_nonjoining_arabic_letters_have_no_tracking_or_extra_character_gap() {
    // Actual Unicode17 facts, not a conclusion inferred from locale or font.
    unicode(
        0x628,
        GeneralCategory::OtherLetter,
        JoiningType::DualJoining,
    );
    unicode(0x621, GeneralCategory::OtherLetter, JoiningType::NonJoining);
    let bytes = crate::shaping_test_fonts::arabic_nonjoining_ahem();
    font(&bytes, false);
    for (text, ids) in [("\u{628}\u{628}", [100, 82]), ("\u{621}\u{621}", [36, 36])] {
        let control_source = source(text, 0.0);
        let control_prepared = prepare(&bytes, &control_source, 0.0);
        let control = select(&control_prepared, SpacingPolicy::None)
            .finalize(SpacingAllocations::None)
            .unwrap();
        let expected_raw = [
            (Range::new(0, 1), ids[0], 0.0, 0.0, 10.0),
            (Range::new(1, 2), ids[1], 0.0, 0.0, 10.0),
        ];
        assert_eq!(sorted_raw(&control, &bytes), expected_raw);
        assert_eq!(control.feedback().advance, 20.0);
        assert_eq!(
            source_paints(&control),
            [(Range::new(0, 1), 10.0), (Range::new(1, 2), 0.0)]
        );
        let source = source(text, 8.0);
        let prepared = prepare(&bytes, &source, 4.0);
        assert_eq!(prepared.analyzed_source().style().letter_spacing, 4.0);
        assert_eq!(source.spans()[0].style().letter_spacing, 8.0);
        let authored = select(&prepared, SpacingPolicy::None)
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(sorted_raw(&authored, &bytes), expected_raw);
        assert_eq!(authored.feedback().advance, 20.0);
        assert_eq!(source_paints(&authored), source_paints(&control));
        let line = select(&prepared, SpacingPolicy::InterCharacter)
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(sorted_raw(&line, &bytes), expected_raw);
        assert_eq!(line.feedback().advance, 20.0);
        assert_eq!(line.measurements().alignment().advance(), 20.0);
        assert_eq!(source_paints(&line), source_paints(&control));
        protected(&prepared, &line, 0.0, &bytes, &expected_raw);
        assert_eq!(prepared.analyzed_source().source(), &source);
    }
}

#[test]
fn arabic_punctuation_keeps_authored_tracking_but_has_no_default_cursive_justification_gap() {
    // U060D is Po/Arab/NonJoining: §7.5.2 treats it as a letter only for
    // justification. §8.2.1's authored Letter/Number no-gap rule does not.
    unicode(
        0x60d,
        GeneralCategory::OtherPunctuation,
        JoiningType::NonJoining,
    );
    unicode(0x621, GeneralCategory::OtherLetter, JoiningType::NonJoining);
    let mapping = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x60d, 82), (0x621, 36), (0x628, 35)],
    );
    let bytes = sfnt::replace_tables(
        &crate::shaping_test_fonts::arabic_nonjoining_ahem(),
        vec![(*b"cmap", sfnt::table(&mapping, *b"cmap").to_vec())],
    );
    font(&bytes, true);
    let control_source = source("\u{60d}\u{621}", 0.0);
    let control_prepared = prepare(&bytes, &control_source, 0.0);
    let control = select(&control_prepared, SpacingPolicy::None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    let expected_raw = [
        (Range::new(0, 1), 82, 0.0, 0.0, 10.0),
        (Range::new(1, 2), 36, 0.0, 0.0, 10.0),
    ];
    assert_eq!(sorted_raw(&control, &bytes), expected_raw);
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(
        source_paints(&control),
        [(Range::new(0, 1), 10.0), (Range::new(1, 2), 0.0)]
    );
    let source = source("\u{60d}\u{621}", 8.0);
    let prepared = prepare(&bytes, &source, 4.0);
    let authored = select(&prepared, SpacingPolicy::None)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(sorted_raw(&authored, &bytes), expected_raw);
    assert_eq!(authored.feedback().advance, 26.0);
    assert_eq!(
        source_paints(&authored),
        [(Range::new(0, 1), 16.0), (Range::new(1, 2), 0.0)]
    );
    let line = select(&prepared, SpacingPolicy::InterCharacter)
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(sorted_raw(&line, &bytes), expected_raw);
    assert_eq!(line.feedback().advance, 26.0);
    assert_eq!(line.measurements().alignment().advance(), 26.0);
    assert_eq!(source_paints(&line), source_paints(&authored));
    protected(&prepared, &line, 6.0, &bytes, &expected_raw);
    assert_eq!(prepared.analyzed_source().source(), &source);
}

//! Native automatic opportunities with retained source, font and cut controls.
//! Cuts are from the qualified en-US2005-05-30 source exceptions and the
//! separately adopted exact-word democrat correction, not engine measurements.
use super::*;
use crate::{HyphenateCharacter, Hyphens, LineHeight, PlaintextItem, Range, Text};
use skrifa::MetadataProvider;

fn auto_fixture() -> Vec<u8> {
    // Repository-authored cmap over the unchanged licensed Ahem resource.
    // All letters use actual Ahem A(35), whose advance is1000 at UPEM1000.
    // The optional98+99 ligature cannot match any mapped character here.
    let mut mappings = vec![(0x20, 3)];
    mappings.extend((b'a'..=b'z').map(|value| (u32::from(value), 35)));
    crate::shaping_test_fonts::range_ligature_ahem(&[98, 99], &mappings)
}

fn auto_prepare(word: &str, hyphens: Hyphens, locale: Option<&str>) -> (PreparedText, Vec<u8>) {
    let bytes = auto_fixture();
    let font = skrifa::FontRef::new(&bytes).unwrap();
    for value in word.chars() {
        assert!(value.is_ascii_lowercase());
        assert_eq!(font.charmap().map(u32::from(value)).unwrap().to_u32(), 35);
    }
    let source = Source::identified(word, crate::Id::from_u64(4201), 7);
    let input = ParagraphInput {
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style: Style {
            size: 10.0,
            line_height: LineHeight::Absolute(10.0),
            hyphens,
            locale: locale.map(str::to_owned),
            hyphenate_character: HyphenateCharacter::String(Text::from("")),
            ..style()
        },
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        origins: vec![paragraph::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, word.len()),
            aggregate: Range::new(0, word.len()),
            kind: paragraph::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input).unwrap(),
        crate::Wrap::Word,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    // Retained preparation must own its resources after the producer is gone.
    drop(system);
    (prepared, bytes)
}

fn auto_glyphs(line: &FinalizedLine, bytes: &[u8]) -> Vec<(Range, u32, f32)> {
    let mut actual = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
        actual.extend(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    actual.sort_by_key(|(range, _, _)| (range.start, range.end));
    actual
}

fn assert_auto_case(word: &str, hyphens: Hyphens, locale: Option<&str>, cuts: &[usize]) {
    let (prepared, bytes) = auto_prepare(word, hyphens, locale);
    let original = word.encode_utf16().collect::<Vec<_>>();
    let analyzed = prepared.analyzed_source();
    assert_eq!(analyzed.source().text().code_units(), original);
    assert_eq!(analyzed.style().hyphens, hyphens);
    assert_eq!(analyzed.style().locale.as_deref(), locale);
    let origin = prepared.origin_views().next().unwrap().handle();
    assert_eq!(origin.view().source().id(), Some(crate::Id::from_u64(4201)));
    assert_eq!(origin.view().source().revision(), 7);
    assert_eq!(origin.view().source().text().code_units(), original);
    let mut plaintext = Vec::new();
    for item in analyzed
        .processed_source()
        .plaintext_items(crate::Affinity::Before)
    {
        match item {
            PlaintextItem::Text(text) => plaintext.extend_from_slice(text.code_units()),
            PlaintextItem::InlineBox(_) => panic!("word fixture has no atomic occurrence"),
        }
    }
    assert_eq!(plaintext, original);
    // Actual complete native selection and registered provider qualify source,
    // resource, literal glyphs and width before the intended missing-cut failure.
    let whole = prepared
        .select(request(&prepared))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert!(whole.selected_line().request().alternative.is_none());
    assert_eq!(
        whole.selected_line().request().context.end,
        LineEnd::Paragraph
    );
    assert_eq!(
        (
            whole.feedback().advance,
            whole.feedback().baseline,
            whole.feedback().line_height
        ),
        (word.len() as f64 * 10.0, 8.0, 10.0)
    );
    assert_eq!(
        auto_glyphs(&whole, &bytes),
        (0..word.len())
            .map(|i| (Range::new(i, i + 1), 35, 10.0))
            .collect::<Vec<_>>()
    );

    let actual = prepared
        .opportunity_views()
        .map(|offer| (offer.boundary().view().source_before().get(), offer.kind()))
        .collect::<Vec<_>>();
    let expected = cuts
        .iter()
        .copied()
        .map(|at| (at, PreparedBreakKind::Discretionary))
        .chain([(word.len(), PreparedBreakKind::ParagraphEnd)])
        .collect::<Vec<_>>();
    assert_eq!(
        actual, expected,
        "actual native {word}/{hyphens:?}/{locale:?}"
    );

    // Existing APIs also observe the real selected alternative and continuation.
    // These checks are prospective until the positive-cut assertion reaches GREEN.
    for &at in cuts {
        let offer = prepared
            .opportunity_views()
            .find(|offer| offer.boundary().view().source_before().get() == at)
            .unwrap();
        let alternatives = offer.alternatives().collect::<Vec<_>>();
        assert_eq!(alternatives.len(), 1);
        let alternative = &alternatives[0];
        assert!(alternative.text().is_empty());
        assert_eq!(
            alternative.source_range(),
            None,
            "automatic insertion displaced no authored text"
        );
        assert_eq!(alternative.insertion_anchor().get(), at);
        assert_eq!(alternative.opportunity(), offer.handle());
        let mut first_request = request(&prepared);
        first_request.end = offer.boundary();
        first_request.participants = prepared
            .participants()
            .take(first_request.end.view().participant_offset())
            .collect();
        first_request.termination = PreparedTermination::Ordinary(offer.handle());
        first_request.alternative = Some(alternative.handle());
        first_request.context.end = LineEnd::Soft;
        let first = prepared
            .select(first_request)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert_eq!(first.feedback().advance, at as f64 * 10.0);
        assert_eq!(
            auto_glyphs(&first, &bytes),
            (0..at)
                .map(|i| (Range::new(i, i + 1), 35, 10.0))
                .collect::<Vec<_>>()
        );
        let mut continued_request = request(&prepared);
        continued_request.start = offer.boundary();
        continued_request.participants = prepared
            .participants()
            .skip(continued_request.start.view().participant_offset())
            .collect();
        continued_request.context.start = LineStart::Continuation;
        let continued = prepared
            .select(continued_request)
            .unwrap()
            .finalize(SpacingAllocations::None)
            .unwrap();
        assert!(continued.selected_line().request().alternative.is_none());
        assert_eq!(
            continued.feedback().advance,
            (word.len() - at) as f64 * 10.0
        );
        assert_eq!(
            auto_glyphs(&continued, &bytes),
            (at..word.len())
                .map(|i| (Range::new(i, i + 1), 35, 10.0))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            prepared.analyzed_source().source().text().code_units(),
            original
        );
    }
}

#[test]
fn native_automatic_en_us_table_uses_the_source_exception_cut() {
    assert_auto_case("table", Hyphens::Auto, Some("en-US"), &[2]);
}

#[test]
fn native_automatic_en_us_present_honors_the_zero_cut_exception() {
    assert_auto_case("present", Hyphens::Auto, Some("en-US"), &[]);
}

#[test]
fn native_automatic_en_us_associate_retains_both_source_exception_cuts() {
    assert_auto_case("associate", Hyphens::Auto, Some("en-US"), &[2, 4]);
}

#[test]
fn native_automatic_en_us_democrat_uses_the_separate_exact_word_correction() {
    assert_auto_case("democrat", Hyphens::Auto, Some("en-US"), &[3, 4]);
}

#[test]
fn native_automatic_en_us_requires_auto_and_the_qualified_language() {
    for (hyphens, locale) in [
        (Hyphens::None, Some("en-US")),
        (Hyphens::Manual, Some("en-US")),
        (Hyphens::Auto, None),
        (Hyphens::Auto, Some("en")),
        (Hyphens::Auto, Some("en-GB")),
    ] {
        assert_auto_case("associate", hyphens, locale, &[]);
    }
}

#[test]
fn native_direct_analysis_counts_differ_from_the_actual_raw_standalone_operation() {
    let bytes = auto_fixture();
    let source = Source::identified("ab\u{ad}cd", crate::Id::from_u64(4202), 8);
    let own = Style {
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        hyphens: Hyphens::Manual,
        hyphenate_character: HyphenateCharacter::String(Text::from("")),
        ..style()
    };
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let analyzed = AnalyzedSource::new(source.clone(), own.clone()).unwrap();
    // This existing operation is genuinely raw and remains unconstrained.
    let raw = system
        .layout_analyzed(
            &analyzed,
            crate::Options {
                width: Some(20.0),
                quantize: false,
                ..crate::Options::default()
            },
        )
        .unwrap();
    assert_eq!(raw.source().text().code_units(), [97, 98, 0xad, 99, 100]);
    assert_eq!(
        raw.lines()
            .iter()
            .map(|line| line.range())
            .collect::<Vec<_>>(),
        [Range::new(0, 3), Range::new(3, 5)]
    );
    let mut advances = vec![0.0; raw.lines().len()];
    for run in raw.inline_metric_facts().runs() {
        advances[run.line()] += run.advance();
    }
    assert_eq!(advances, [20.0, 20.0]);
    // System::prepare is Native even for this same direct analysis allocation.
    let native = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert_eq!(native.analyzed_source().identity(), analyzed.identity());
    let whole = native
        .select(request(&native))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (
            whole.feedback().advance,
            whole.feedback().baseline,
            whole.feedback().line_height
        ),
        (40.0, 8.0, 10.0)
    );
    assert_eq!(
        auto_glyphs(&whole, &bytes)
            .into_iter()
            .filter(|(_, _, advance)| *advance != 0.0)
            .collect::<Vec<_>>(),
        [
            (Range::new(0, 1), 35, 10.0),
            (Range::new(1, 2), 35, 10.0),
            (Range::new(3, 4), 35, 10.0),
            (Range::new(4, 5), 35, 10.0)
        ]
    );
    assert_eq!(
        native
            .opportunity_views()
            .map(|offer| (offer.boundary().view().source_before().get(), offer.kind()))
            .collect::<Vec<_>>(),
        [(5, PreparedBreakKind::ParagraphEnd)],
        "Native admission must not infer raw semantics from absent explicit paragraph direction"
    );
}

//! Logical-after indications preserve source and affect continuation geometry.
use super::*;
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
const ORIGINAL: [u16; 6] = [65, 66, 173, 67, 68, 69];

fn analyzed(placement: text::HyphenationPlacement) -> text::AnalyzedSource {
    let convention = text::NativeHyphenationConvention::language_script(
        "und-Latn",
        "Latn",
        text::Text::from("A"),
        placement,
    )
    .unwrap();
    let style = text::Style {
        hyphens: text::Hyphens::Manual,
        locale: Some("und-Latn".into()),
        hyphenate_character: text::HyphenateCharacter::Auto,
        native_hyphenation_convention: Some(convention),
        ..ahem_style(10.0, 10.0)
    };
    let source = Source::identified("AB\u{ad}CDE", text::Id::from_u64(66077), 7);
    let value = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source: source.clone(),
            base_style: style,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, 6),
                aggregate: text::Range::new(0, 6),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let value = crate::test_support::text::in_flat_wrapping_context(value, text::Wrap::Word);
    assert_eq!(value.source().text().code_units(), ORIGINAL);
    let style = value.style();
    assert_eq!(style.hyphens, text::Hyphens::Manual);
    assert_eq!(style.locale.as_deref(), Some("und-Latn"));
    assert_eq!(
        style.hyphenate_limit_chars,
        text::HyphenateLimitChars::default()
    );
    let convention = style.native_hyphenation_convention.as_ref().unwrap();
    assert_eq!(convention.locale(), Some("und-Latn"));
    assert_eq!(convention.script(), "Latn");
    value
}

fn system() -> text::System {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    system
}

fn prepared(placement: text::HyphenationPlacement) -> text::Result<text::PreparedText> {
    system().prepare(&analyzed(placement), text::PreparationOptions::default())
}

fn qualify_prepared(prepared: &text::PreparedText) {
    assert_eq!(
        prepared.analyzed_source().source().text().code_units(),
        ORIGINAL
    );
    let origins = prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].source().id(), Some(text::Id::from_u64(66077)));
    assert_eq!(origins[0].source().revision(), 7);
    assert_eq!(origins[0].source().text().code_units(), ORIGINAL);
    let cuts = prepared.hyphenation_cut_views().collect::<Vec<_>>();
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].provenance(), text::HyphenationProvenance::Source);
    assert_eq!(
        cuts[0].counts(),
        Some(text::HyphenationCounts {
            total: 5,
            before: 2,
            after: 3
        })
    );
    assert_eq!(
        cuts[0].opportunity().unwrap().view().kind(),
        text::PreparedBreakKind::Discretionary
    );
    let alternatives = prepared.alternative_views().collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 1);
    assert_eq!(alternatives[0].text().code_units(), [65]);
    assert_eq!(alternatives[0].source_range(), Some(text::Range::new(2, 3)));
    assert_eq!(alternatives[0].insertion_anchor().get(), 3);
}

fn compute<S: LayoutScalar>(
    prepared: &text::PreparedText,
    available: AvailableOf<S>,
    width: Option<f64>,
) -> CompletedLayoutBatchOf<u32, S> {
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    width.map_or(PreferredSizeOf::AUTO, |width| {
                        PreferredSizeOf::px(S::from_f64(width))
                    }),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(origin.clone())),
        );
    let binding = TextParagraphBinding {
        prepared: prepared.clone(),
        container: 0,
        nodes: vec![1],
        origins: vec![TextOriginBinding { origin, node: 1 }],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    let captured = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request =
        LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT)).unwrap();
    let result = compute_layout_with_flow(&captured, 0, request).unwrap();
    assert_eq!(result.placed_text().len(), 1);
    assert_eq!(result.placed_text()[0].nodes(), [1]);
    assert_eq!(
        result.placed_text()[0]
            .layout()
            .source()
            .text()
            .code_units(),
        ORIGINAL
    );
    result
}

fn lines<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) -> Vec<text::FinalizedLine> {
    let mut lines = vec![None; batch.placed_text()[0].layout().lines().len()];
    for entry in batch.final_inline_fragments() {
        let f = entry.fragment();
        let line = f.finalized_fragment().finalized_line();
        if let Some(prior) = &lines[f.line_index()] {
            assert_eq!(prior, &line);
        }
        lines[f.line_index()] = Some(line);
    }
    lines.into_iter().map(|line| line.unwrap()).collect()
}

fn glyphs(line: &text::FinalizedLine) -> Vec<(text::Range, u32, f32)> {
    let mut glyphs = Vec::new();
    for run in line.run_views() {
        assert_eq!(run.face().bytes(), AHEM);
        assert_eq!(
            run.face().catalogue_identity(),
            line.prepared_text().font_catalogue_identity()
        );
        let instance = run.effective_instance().unwrap();
        assert!(instance.face().is_same_face(&run.face()));
        assert_eq!(instance.font_size(), 10.0);
        // A retained zero-advance formatting row is admitted only for this
        // actual original SHY occurrence, never for an arbitrary zero glyph.
        glyphs.extend(
            run.glyphs()
                .filter(|glyph| {
                    if glyph.advance() == 0.0 {
                        assert_eq!(glyph.range(), text::Range::new(2, 3));
                        assert_eq!(
                            line.prepared_text()
                                .analyzed_source()
                                .source()
                                .text()
                                .code_units()[2],
                            0xad
                        );
                        assert!(
                            line.prepared_text()
                                .alternative_views()
                                .any(|alternative| alternative.source_range()
                                    == Some(text::Range::new(2, 3)))
                        );
                        false
                    } else {
                        true
                    }
                })
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance())),
        );
    }
    glyphs
}

fn qualify_whole<S: LayoutScalar>(p: &text::PreparedText) {
    let batch = compute::<S>(p, AvailableOf::definite(S::from_f64(100.0)), Some(100.0));
    let actual = lines(&batch);
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].feedback().advance, 50.0);
    assert_eq!(
        (
            actual[0].feedback().occupied_start,
            actual[0].feedback().occupied_end
        ),
        (0.0, 50.0)
    );
    assert_eq!(
        glyphs(&actual[0]),
        [
            (text::Range::new(0, 1), 35, 10.0),
            (text::Range::new(1, 2), 36, 10.0),
            (text::Range::new(3, 4), 37, 10.0),
            (text::Range::new(4, 5), 38, 10.0),
            (text::Range::new(5, 6), 39, 10.0),
        ]
    );
    assert!(actual[0].selected_line().request().alternative.is_none());
}

fn assert_layout<S: LayoutScalar>(p: &text::PreparedText, after: bool) {
    let batch = compute::<S>(p, AvailableOf::definite(S::from_f64(40.0)), Some(40.0));
    let actual = lines(&batch);
    assert_eq!(actual.len(), 2);
    let expected = if after { [20.0, 40.0] } else { [30.0, 30.0] };
    for (i, line) in actual.iter().enumerate() {
        assert_eq!(line.feedback().advance, expected[i]);
        assert_eq!(
            (line.feedback().occupied_start, line.feedback().occupied_end),
            (0.0, expected[i])
        );
        assert_eq!(line.feedback().baseline, 8.0);
        assert_eq!(line.feedback().line_height, 10.0);
        assert_eq!(
            batch.placed_text()[0].layout().lines()[i].range(),
            [text::Range::new(0, 3), text::Range::new(3, 6)][i]
        );
        assert_eq!(
            batch.placed_text()[0].layout().lines()[i].baseline(),
            8.0 + 10.0 * i as f64
        );
    }
    let mut first = vec![
        (text::Range::new(0, 1), 35, 10.0),
        (text::Range::new(1, 2), 36, 10.0),
    ];
    let mut second = vec![
        (text::Range::new(3, 4), 37, 10.0),
        (text::Range::new(4, 5), 38, 10.0),
        (text::Range::new(5, 6), 39, 10.0),
    ];
    if after {
        second.insert(0, (text::Range::new(3, 3), 35, 10.0));
    } else {
        first.push((text::Range::new(3, 3), 35, 10.0));
    }
    assert_eq!(glyphs(&actual[0]), first);
    assert_eq!(glyphs(&actual[1]), second);
    assert!(actual[0].selected_line().request().alternative.is_some());
    for (available, expected) in [
        (AvailableOf::MIN_CONTENT, if after { 40.0 } else { 30.0 }),
        (AvailableOf::MAX_CONTENT, 50.0),
    ] {
        let intrinsic = compute::<S>(p, available, None);
        let root = intrinsic
            .final_entries()
            .iter()
            .find(|e| e.node() == 0)
            .unwrap()
            .output();
        assert_eq!(root.size.width, S::from_f64(expected));
    }
}

fn before<S: LayoutScalar>() {
    let p = prepared(text::HyphenationPlacement::LogicalBefore).unwrap();
    qualify_prepared(&p);
    qualify_whole::<S>(&p);
    assert_layout::<S>(&p, false);
}

#[test]
fn native_before_qualifies_real_ahem_count_and_layout_f32() {
    before::<f32>();
}
#[test]
fn native_before_qualifies_real_ahem_count_and_layout_f64() {
    before::<f64>();
}

fn positive_after<S: LayoutScalar>() {
    // Each scalar reaches its own missing behavior after independent Before
    // resource/font/count/geometry and intrinsic controls have passed.
    before::<S>();
    let p = prepared(text::HyphenationPlacement::LogicalAfter)
        .expect("admitted real native LogicalAfter must support continuation placement");
    qualify_prepared(&p);
    qualify_whole::<S>(&p);
    assert_layout::<S>(&p, true);
}
#[test]
fn native_after_places_real_indication_on_continuation_f32() {
    positive_after::<f32>();
}
#[test]
fn native_after_places_real_indication_on_continuation_f64() {
    positive_after::<f64>();
}

//! Material tracking cannot assume external Arabic/Syriac joining is independent.
//! This regression uses only the existing public preparation/selection/result API.
use super::*;
use icu_properties::{
    CodePointMapData,
    props::{GeneralCategory, JoiningType, Script},
    script::ScriptWithExtensions,
};

fn fixture() -> Vec<u8> {
    let base = crate::shaping_test_fonts::arabic_nonjoining_ahem();
    let original = sfnt::table(&base, *b"GSUB");
    let script_start = usize::from(word(original, 4));
    let feature_start = usize::from(word(original, 6));
    assert_eq!(word(original, script_start), 1);
    assert_eq!(&original[script_start + 2..script_start + 6], b"arab");
    let script =
        &original[script_start + usize::from(word(original, script_start + 6))..feature_start];
    // Both real ScriptList records select their own default LangSys, which
    // references the existing real init/fina/isol/medi feature lookups.
    let mut scripts = Vec::new();
    scripts.extend_from_slice(&2_u16.to_be_bytes());
    scripts.extend_from_slice(b"arab");
    scripts.extend_from_slice(&14_u16.to_be_bytes());
    scripts.extend_from_slice(b"syrc");
    scripts.extend_from_slice(&u16::try_from(14 + script.len()).unwrap().to_be_bytes());
    scripts.extend_from_slice(script);
    scripts.extend_from_slice(script);
    let added = scripts.len() - (feature_start - script_start);
    let mut gsub = original[..script_start].to_vec();
    for at in [6, 8] {
        gsub[at..at + 2].copy_from_slice(
            &u16::try_from(usize::from(word(original, at)) + added)
                .unwrap()
                .to_be_bytes(),
        );
    }
    gsub.extend_from_slice(&scripts);
    gsub.extend_from_slice(&original[feature_start..]);
    let cmap = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0x42, 36), (0x628, 35), (0x712, 35)],
    );
    sfnt::replace_tables(
        &base,
        vec![
            (*b"GSUB", gsub),
            (*b"cmap", sfnt::table(&cmap, *b"cmap").to_vec()),
        ],
    )
}

fn qualify(bytes: &[u8]) {
    font_premises(bytes, false);
    let font = skrifa::FontRef::new(bytes).unwrap();
    for (cp, script) in [(0x628, Script::Arabic), (0x712, Script::Syriac)] {
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get32(cp),
            GeneralCategory::OtherLetter
        );
        assert_eq!(
            CodePointMapData::<JoiningType>::new().get32(cp),
            JoiningType::DualJoining
        );
        assert_eq!(CodePointMapData::<Script>::new().get32(cp), script);
        assert_eq!(
            ScriptWithExtensions::new()
                .get_script_extensions_val32(cp)
                .iter()
                .collect::<Vec<_>>(),
            [script]
        );
        assert_eq!(
            font.charmap().map(char::from_u32(cp).unwrap()),
            Some(GlyphId::new(35))
        );
    }
    assert_ne!(Script::Arabic, Script::Syriac);
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for id in [35, 82, 100, 277] {
        assert_eq!(metrics.advance_width(GlyphId::new(id)), Some(1000.0));
    }
    assert!(font.table_data(skrifa::Tag::new(b"GPOS")).is_none());
    assert!(font.gsub().unwrap().lookup_list().is_ok());
    let gsub = sfnt::table(bytes, *b"GSUB");
    let scripts = usize::from(word(gsub, 4));
    assert_eq!(word(gsub, scripts), 2);
    for (index, tag) in [b"arab", b"syrc"].into_iter().enumerate() {
        let record = scripts + 2 + 6 * index;
        assert_eq!(&gsub[record..record + 4], tag);
        let script = scripts + usize::from(word(gsub, record + 4));
        assert_eq!(word(gsub, script + 2), 0);
        let language = script + usize::from(word(gsub, script));
        assert_eq!(
            (word(gsub, language + 2), word(gsub, language + 4)),
            (0xffff, 4)
        );
        assert_eq!(
            (0..4)
                .map(|i| word(gsub, language + 6 + i * 2))
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
    }
    let features = usize::from(word(gsub, 6));
    let lookups = usize::from(word(gsub, 8));
    assert_eq!((word(gsub, features), word(gsub, lookups)), (4, 4));
    for (ordinal, (tag, inputs, outputs)) in [
        (*b"fina", vec![35, 58], vec![82, 274]),
        (*b"init", vec![35], vec![100]),
        (*b"isol", vec![35], vec![35]),
        (*b"medi", vec![35], vec![277]),
    ]
    .into_iter()
    .enumerate()
    {
        let record = features + 2 + ordinal * 6;
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
                .map(|i| word(gsub, sub + 6 + i * 2))
                .collect::<Vec<_>>(),
            outputs
        );
        let coverage = sub + usize::from(word(gsub, sub + 2));
        assert_eq!(
            (word(gsub, coverage), usize::from(word(gsub, coverage + 2))),
            (1, inputs.len())
        );
        assert_eq!(
            (0..inputs.len())
                .map(|i| word(gsub, coverage + 4 + i * 2))
                .collect::<Vec<_>>(),
            inputs
        );
    }
}

fn analyzed(source: &Source, tracking: f64) -> AnalyzedSource {
    let extent = Range::new(0, source.text().len());
    let value = ParagraphInput {
        source: source.clone(),
        base_style: own(tracking),
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
        &AnalyzedSource::from_paragraph(value, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.source(), source);
    assert!(analyzed.style().locale.is_none());
    assert_eq!(analyzed.direction(), crate::Direction::RightToLeft);
    analyzed
}

fn execute(prepared: &PreparedText) -> crate::Result<FinalizedLine> {
    let mut value = request(prepared);
    value.context.base_direction = ParagraphBaseDirection::RightToLeft;
    value.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::None;
    prepared.select(value)?.finalize(SpacingAllocations::None)
}

#[test]
fn material_tracking_refuses_unproven_external_joining_between_real_arabic_and_syriac_runs() {
    let bytes = fixture();
    qualify(&bytes);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    // Each isolated scalar is nominal35/isol35, proving that the combined
    // init100/fina82 below are actual context-sensitive substitutions.
    for text in ["\u{628}", "\u{712}"] {
        let source = Source::new(text);
        let prepared = system
            .prepare(&analyzed(&source, 0.0), PreparationOptions::default())
            .unwrap();
        let isolated = execute(&prepared).unwrap();
        assert_eq!(isolated.run_views().len(), 1);
        assert_eq!(
            raw(&isolated, &bytes),
            [(Range::new(0, 1), 35, 0.0, 0.0, 10.0)]
        );
        assert_eq!(isolated.feedback().advance, 10.0);
    }
    let source = Source::identified("\u{628}\u{712}", Id::from_u64(1566050), 1);
    assert_eq!(source.text().code_units(), [0x628, 0x712]);
    let analysis = analyzed(&source, 0.0);
    assert_eq!(
        analysis.graphemes().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    assert!(prepared.unit_views().all(|unit| unit.level().get() == 1));
    let origin = prepared.origin_views().next().unwrap();
    assert_eq!(origin.source(), &source);
    assert_eq!(origin.local_range(), Range::new(0, 2));
    assert_eq!(origin.aggregate_range(), Range::new(0, 2));
    let control = execute(&prepared).unwrap();
    let mut runs = control
        .run_views()
        .map(|run| {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
            assert_eq!(run.effective_instance().unwrap().axis_count(), 0);
            assert_eq!(run.glyphs().len(), 1);
            run.range()
        })
        .collect::<Vec<_>>();
    runs.sort_by_key(|range| range.start);
    assert_eq!(runs, [Range::new(0, 1), Range::new(1, 2)]);
    let mut control_raw = raw(&control, &bytes);
    control_raw.sort_by_key(|glyph| glyph.0.start);
    assert_eq!(
        control_raw,
        [
            (Range::new(0, 1), 100, 0.0, 0.0, 10.0),
            (Range::new(1, 2), 82, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(control.measurements().alignment().advance(), 20.0);
    let mut paints = control
        .run_views()
        .map(|run| {
            (
                run.range(),
                run.origin().x + run.glyphs().next().unwrap().pen(),
            )
        })
        .collect::<Vec<_>>();
    paints.sort_by_key(|row| row.0.start);
    assert_eq!(paints, [(Range::new(0, 1), 10.0), (Range::new(1, 2), 0.0)]);
    // Both bodies are single-scalar invocations. Their observed init/fina forms
    // depend on real pre/post context outside each invocation; an in-buffer
    // structural observation cannot prove that moving them apart is safe.
    let material = analyzed(&source, 4.0);
    let result = system
        .prepare(&material, PreparationOptions::default())
        .and_then(|prepared| execute(&prepared));
    let error =
        result.expect_err("material tracking must refuse the unproven external joining relation");
    assert_eq!(error.code, crate::ErrorCode::LayoutFailed);
    assert_eq!(control.feedback().advance, 20.0);
    assert_eq!(
        raw(&execute(&prepared).unwrap(), &bytes),
        raw(&control, &bytes)
    );
    assert_eq!(prepared.analyzed_source().source(), &source);
}

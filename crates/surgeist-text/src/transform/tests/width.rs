use crate::{AnalyzedSource, CaseTransform, Range, Source, Style, Text, TextTransform, WhiteSpace};

fn style() -> Style {
    Style {
        text_transform: TextTransform::Set(crate::TextTransformSet::none().with_full_width(true)),
        ..Style::default()
    }
}

fn input(source: Source, white_space: WhiteSpace) -> crate::ParagraphInput {
    let len = source.text().len();
    crate::ParagraphInput {
        source: source.clone(),
        base_style: Style {
            white_space,
            ..style()
        },
        paragraph_strut: crate::ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: crate::ParagraphBaseDirection::LeftToRight,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        boundary_policies: vec![],
        anchor_order: vec![],
    }
}

fn paragraph(source: Source, white_space: WhiteSpace) -> AnalyzedSource {
    AnalyzedSource::from_paragraph(input(source, white_space)).unwrap()
}

#[test]
fn width_and_neutral_component_styles_preserve_connected_origin_mark_and_oof_custody() {
    let mut source = Source::new("AｶﾞA");
    source.span(Range::new(3, 4), Style::default());
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(6322),
        crate::InlineBoxKind::OutOfFlow,
        3,
        crate::Size::new(0.0, 0.0),
    ));
    let mut i = input(source.clone(), WhiteSpace::Preserve);
    i.origins = [(0, 1), (1, 2), (2, 3), (3, 3), (3, 4)]
        .into_iter()
        .map(|(start, end)| crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(start, end),
            aggregate: Range::new(start, end),
            kind: crate::ParagraphOriginKind::Authored,
        })
        .collect();
    i.connections = [1, 2, 3, 3]
        .into_iter()
        .map(|at| crate::ParagraphConnection {
            at,
            kind: crate::ParagraphConnectionKind::Continuous,
        })
        .collect();
    i.boundary_policies = (0..4)
        .map(|index| crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    i.anchor_order = vec![
        crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::EmptyOrigin(3),
            affinity: crate::Affinity::After,
        },
        crate::ParagraphAnchor {
            item: crate::ParagraphAnchorItem::Atomic(0),
            affinity: crate::Affinity::After,
        },
    ];
    let analyzed = AnalyzedSource::from_paragraph(i).unwrap();
    assert_eq!(analyzed.inner.projection.text, "Ａカ\u{3099}A");
    assert_eq!(analyzed.inner.analysis.graphemes, [0..1, 1..3, 3..4]);
    let map = &analyzed.inner.transform;
    for (row, origin) in [(0, 0), (1, 1), (2, 2), (3, 4)] {
        assert_eq!(map.original_pieces(row)[0].origin, origin);
        assert_eq!(
            map.original_pieces(row)[0].aggregate,
            Range::new(row, row + 1)
        );
    }
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
}

#[test]
fn every_raw_wide_narrow_record_executes_once_in_its_declared_direction() {
    // Independent interpretation of retained UnicodeData fields, not generated
    // output or normalization results. Wide is reversed; narrow is forward.
    let mut records = Vec::new();
    for line in include_str!("../../../data/UnicodeData-width-18.0.0.txt").lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<_> = line.split(';').collect();
        let cp = u32::from_str_radix(fields[0], 16).unwrap();
        let mut decomposition = fields[5].split_whitespace();
        let tag = decomposition.next().unwrap();
        let target = u32::from_str_radix(decomposition.next().unwrap(), 16).unwrap();
        assert!(decomposition.next().is_none());
        let (input, output) = match tag {
            "<wide>" => (target, cp),
            "<narrow>" => (cp, target),
            other => panic!("unexpected raw relation {other}"),
        };
        records.push((input, output));
        assert_eq!(super::super::width::map(input), Some(output));
        let source = Source::new(char::from_u32(input).unwrap().to_string());
        let result = AnalyzedSource::new(source.clone(), style()).unwrap();
        assert_eq!(result.inner.projection.atoms[0].code_point, output);
        assert_eq!(
            result.inner.projection.atoms[0].source,
            Range::new(0, source.text().len())
        );
        assert_eq!(result.processed_source().processed_text(), source.text());
        assert_eq!(result.source(), &source);
    }
    assert_eq!(records.len(), 226);
    records.sort_unstable_by_key(|pair| pair.0);
    assert!(records.windows(2).all(|pair| pair[0].0 != pair[1].0));
    assert_eq!(super::super::width_data::MAPPINGS.len(), records.len());
}

#[test]
fn raw_width_is_not_recursive_compatibility_or_canonical_normalization() {
    for (input, expected) in [
        ("A!09$¢£¬¯", "Ａ！０９＄￠￡￢￣"),
        ("\u{ffa0}\u{ffa1}", "\u{3164}\u{3131}"),
        ("\u{ff76}\u{ff9e}", "\u{30ab}\u{3099}"),
        ("Ａ\u{3000}²ﬀⅣÅ\u{1100}", "Ａ\u{3000}²ﬀⅣÅ\u{1100}"),
    ] {
        let analyzed = AnalyzedSource::new(Source::new(input), style()).unwrap();
        assert_eq!(analyzed.inner.projection.text, expected);
        assert_eq!(analyzed.inner.projection.atoms.len(), input.chars().count());
    }
    let opaque = Source::new(Text::from_code_units(vec![0xd800, 0xfffd, 0xdc00, 0x41]));
    let analyzed = AnalyzedSource::new(opaque.clone(), style()).unwrap();
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        [0xd800, 0xfffd, 0xdc00, 0xff21]
    );
    assert_eq!(analyzed.source(), &opaque);
}

#[test]
fn actual_case_then_width_then_kana_keep_each_original_emission_owner() {
    for (case, width, kana, expected) in [
        (CaseTransform::None, false, false, "ßｧ"),
        (CaseTransform::Uppercase, false, false, "SSｧ"),
        (CaseTransform::None, false, true, "ßｱ"),
        (CaseTransform::None, true, false, "ßァ"),
        (CaseTransform::Uppercase, true, false, "ＳＳァ"),
        (CaseTransform::Uppercase, true, true, "ＳＳア"),
    ] {
        let source = Source::new("ßｧ");
        let analyzed = AnalyzedSource::new(
            source.clone(),
            Style {
                text_transform: TextTransform::Set(
                    crate::TextTransformSet::none()
                        .with_case(case)
                        .with_full_width(width)
                        .with_full_size_kana(kana),
                ),
                ..Style::default()
            },
        )
        .unwrap();
        assert_eq!(analyzed.inner.projection.text, expected);
        assert_eq!(analyzed.processed_source().processed_text(), source.text());
        for atom in &analyzed.inner.projection.atoms {
            assert_eq!(atom.processed, atom.source);
        }
        let map = &analyzed.inner.transform;
        assert_eq!(map.original_pieces(0)[0].aggregate, Range::new(0, 1));
        assert_eq!(map.original_pieces(1)[0].aggregate, Range::new(1, 2));
        if case == CaseTransform::Uppercase {
            assert_eq!(
                analyzed
                    .inner
                    .projection
                    .atoms
                    .iter()
                    .map(|atom| atom.source)
                    .collect::<Vec<_>>(),
                [Range::new(0, 1), Range::new(0, 1), Range::new(1, 2)]
            );
        }
    }
    let mut source = Source::new("ABC");
    source.span(Range::new(1, 2), Style::default());
    assert_eq!(
        AnalyzedSource::new(source, style())
            .unwrap()
            .inner
            .projection
            .text,
        "ＡBＣ"
    );
    for phase in [
        crate::TextStylePhase::Standalone,
        crate::TextStylePhase::ProcessedParagraph,
    ] {
        assert_eq!(
            crate::TextStyleFeature::FullWidth.support(phase),
            crate::TextStyleSupport::Supported
        );
    }
    assert!(crate::TextStyleFeature::ALL.contains(&crate::TextStyleFeature::FullWidth));
    assert!(!crate::TextTransformSet::none().full_width());
    assert_eq!(
        TextTransform::Set(
            crate::TextTransformSet::none()
                .with_full_width(true)
                .with_full_width(false)
        ),
        TextTransform::none()
    );
}

#[test]
fn only_executed_preserved_space_maps_and_raw_identity_remains_a_distinct_profile() {
    for (mode, processed, expected) in [
        (WhiteSpace::Collapse, " A B ", " Ａ Ｂ "),
        (WhiteSpace::Preserve, " A\tB\n ", "\u{3000}Ａ\tＢ\n\u{3000}"),
        (
            WhiteSpace::PreserveSpaces,
            " A B  ",
            "\u{3000}Ａ\u{3000}Ｂ\u{3000}\u{3000}",
        ),
        (WhiteSpace::Discard, "AB", "ＡＢ"),
    ] {
        let source = Source::new(" A\tB\n ");
        let result = paragraph(source.clone(), mode);
        assert_eq!(
            result.processed_source().processed_text(),
            &Text::from(processed)
        );
        assert_eq!(result.inner.projection.text, expected);
        assert_eq!(result.source(), &source);
    }
    let raw = AnalyzedSource::new(Source::new(" A "), style()).unwrap();
    assert_eq!(raw.processed_source().processed_text(), &Text::from(" A "));
    assert_eq!(raw.inner.projection.text, "\u{3000}Ａ\u{3000}");
    // Same original/processed text and Style width request, different actual
    // processing owners: surviving Collapse SPACE must not impersonate Preserve.
    let collapse = paragraph(Source::new("A B"), WhiteSpace::Collapse);
    let preserve = paragraph(Source::new("A B"), WhiteSpace::Preserve);
    assert_eq!(
        collapse.processed_source().processed_text(),
        preserve.processed_source().processed_text()
    );
    assert_eq!(collapse.inner.projection.text, "Ａ Ｂ");
    assert_eq!(preserve.inner.projection.text, "Ａ\u{3000}Ｂ");
}

#[test]
fn source_local_processing_lends_real_preservation_to_the_transform_owner() {
    for (mode, processed, expected) in [
        (WhiteSpace::Collapse, "A B C", "Ａ Ｂ Ｃ"),
        (WhiteSpace::Preserve, "A B\tC", "Ａ\u{3000}Ｂ\tＣ"),
        (
            WhiteSpace::PreserveSpaces,
            "A B C",
            "Ａ\u{3000}Ｂ\u{3000}Ｃ",
        ),
        (WhiteSpace::Discard, "ABC", "ＡＢＣ"),
    ] {
        let source = Source::new("A B\tC");
        let executed = crate::process_source_whitespace(
            source.clone(),
            Style {
                white_space: mode,
                ..style()
            },
            crate::DocumentSegmentBreaks::LineFeedDefault,
        )
        .unwrap();
        assert_eq!(executed.processed_text(), &Text::from(processed));
        // Invoke the same analysis/transform owner on the genuine source-local
        // result; no caller-authored preservation flag or replayed Style fact.
        let paragraph = crate::protocol::paragraph::ParagraphData::standalone(&source);
        let input =
            crate::projection::SourceProjection::from_processed(&executed, &paragraph).unwrap();
        let validated =
            crate::ValidatedStyle::for_processed(executed.base_style().clone()).unwrap();
        let mut styles =
            crate::engine::resolve::analyze_styles(&source, &[], &validated, &paragraph);
        let (actual, _) = super::super::apply(input, &mut styles, &executed, &paragraph).unwrap();
        assert_eq!(actual.text, expected);
        assert_eq!(executed.source(), &source);
    }
}

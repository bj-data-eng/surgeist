//! Functional qualification of the new source-stage identity, before fonts or
//! combined-upright selection. Literal Unicode mappings are the oracle.

use crate::transform::relation::RelationEvent;
use crate::{
    Affinity, AnalysisTypography, AnalyzedSource, CaseTransform, PlaintextItem, Range, Source,
    Style, TextTransform, TextTransformSet, WhiteSpace,
};

fn style(case: CaseTransform, width: bool, kana: bool) -> Style {
    Style {
        text_transform: TextTransform::Set(
            TextTransformSet::none()
                .with_case(case)
                .with_full_width(width)
                .with_full_size_kana(kana),
        ),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn analyze(source: &Source, style: Style) -> AnalyzedSource {
    AnalyzedSource::new(
        source.clone(),
        style,
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap()
}

fn processed_paragraph(source: &Source, style: Style) -> AnalyzedSource {
    let range = Range::new(0, source.text().len());
    AnalyzedSource::from_paragraph(
        crate::ParagraphInput {
            source: source.clone(),
            base_style: style,
            paragraph_strut: crate::ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: crate::ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                text_node: Some(
                    crate::CompleteTextNodeContribution::new(source.text().clone(), range).unwrap(),
                ),
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        },
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap()
}

fn ordinary(analyzed: &AnalyzedSource, source: &Source, presentation: &str) {
    assert_eq!(analyzed.source(), source);
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
    assert_eq!(analyzed.inner.projection.text, presentation);
    assert_eq!(
        analyzed
            .processed_source()
            .plaintext_items(Affinity::After)
            .flat_map(|item| match item {
                PlaintextItem::Text(text) => text.code_units().to_vec(),
                PlaintextItem::InlineBox(_) => vec![],
            })
            .collect::<Vec<_>>(),
        source.text().code_units()
    );
}

// `atoms` is actual emission order, which need not be input-row storage order.
fn emissions(analyzed: &AnalyzedSource) -> Vec<(u32, u32, usize, Range)> {
    let map = &analyzed.inner.transform;
    map.atoms
        .iter()
        .map(|&id| {
            let emission = &map.emissions[id];
            (
                emission.code_point,
                emission.selection_identity,
                emission.subposition,
                map.components[emission.component].source,
            )
        })
        .collect()
}

#[test]
fn unchanged_scalars_keep_identity_and_original_mark_and_format_ancestry() {
    let source = Source::new("A\u{301}\u{2065}\u{200d}");
    let analyzed = analyze(
        &source,
        Style {
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
    );
    ordinary(&analyzed, &source, "A\u{301}\u{2065}\u{200d}");
    assert_eq!(
        emissions(&analyzed),
        [
            (0x41, 0x41, 0, Range::new(0, 1)),
            (0x301, 0x301, 0, Range::new(1, 2)),
            (0x2065, 0x2065, 0, Range::new(2, 3)),
            (0x200d, 0x200d, 0, Range::new(3, 4)),
        ]
    );
    assert_eq!(
        analyzed
            .inner
            .transform
            .rows
            .iter()
            .map(|row| row.code_point)
            .collect::<Vec<_>>(),
        [0x41, 0x301, 0x2065, 0x200d]
    );
}

#[test]
fn sharp_s_has_two_occurrences_even_when_full_width_changes_both_paint_scalars() {
    let source = Source::new("ß");
    for (width, presentation, scalar) in [(false, "SS", 0x53), (true, "ＳＳ", 0xff33)] {
        let analyzed = analyze(&source, style(CaseTransform::Uppercase, width, false));
        ordinary(&analyzed, &source, presentation);
        assert_eq!(
            emissions(&analyzed),
            [
                (scalar, 0x53, 0, Range::new(0, 1)),
                (scalar, 0x53, 1, Range::new(0, 1)),
            ]
        );
        let map = &analyzed.inner.transform;
        assert_eq!(map.rows.len(), 1);
        assert_eq!(map.rows[0].code_point, 0xdf);
        assert_eq!(map.original_pieces(0)[0].aggregate, Range::new(0, 1));
        assert_eq!(map.original_pieces(0)[0].local, Range::new(0, 1));
    }
}

#[test]
fn literal_full_width_digit_does_not_acquire_ascii_selection_identity() {
    let source = Source::new("9９");
    let analyzed = analyze(&source, style(CaseTransform::None, true, false));
    ordinary(&analyzed, &source, "９９");
    assert_eq!(
        emissions(&analyzed),
        [
            (0xff19, 0x39, 0, Range::new(0, 1)),
            (0xff19, 0xff19, 0, Range::new(1, 2)),
        ]
    );
    assert_eq!(analyzed.inner.transform.rows[0].code_point, 0x39);
    assert_eq!(analyzed.inner.transform.rows[1].code_point, 0xff19);
}

#[test]
fn math_auto_identity_is_the_actual_post_mapping_scalar_with_original_custody() {
    // MathML Core's retained italic mapping is U+0061 -> U+1D44E. The
    // complete one-scalar original text node, not a glyph, admits MathAuto.
    let source = Source::new("a");
    let analyzed = processed_paragraph(
        &source,
        Style {
            text_transform: TextTransform::MathAuto,
            white_space: WhiteSpace::Preserve,
            ..Style::default()
        },
    );
    ordinary(&analyzed, &source, "\u{1d44e}");
    assert_eq!(
        emissions(&analyzed),
        [(0x1d44e, 0x1d44e, 0, Range::new(0, 1))]
    );
    let map = &analyzed.inner.transform;
    assert_eq!(map.rows.len(), 1);
    assert_eq!(map.rows[0].code_point, 0x61);
    assert_eq!(map.original_pieces(0).len(), 1);
    assert_eq!(
        analyzed
            .inner
            .paragraph
            .origin_map
            .view_origin(map.original_pieces(0)[0].origin)
            .expect("original contribution belongs to this view")
            .get(),
        0
    );
    assert_eq!(map.original_pieces(0)[0].aggregate, Range::new(0, 1));
    assert_eq!(map.original_pieces(0)[0].local, Range::new(0, 1));
    assert_eq!(
        analyzed.inner.projection.atoms[0].processed,
        Range::new(0, 1)
    );
    assert_eq!(
        analyzed.inner.projection.atoms[0].presentation,
        Range::new(0, 2)
    );
}

#[test]
fn full_size_kana_identity_excludes_width_and_does_not_commute_the_paint_maps() {
    // Unicode width maps FF67 -> 30A1; full-size-kana maps FF67 -> FF71
    // and 30A1 -> 30A2. Identity therefore is FF71, while paint is 30A2.
    let source = Source::new("ｧ");
    for (width, kana, presentation, paint, identity) in [
        (false, true, "ｱ", 0xff71, 0xff71),
        (true, false, "ァ", 0x30a1, 0xff67),
        (true, true, "ア", 0x30a2, 0xff71),
    ] {
        let analyzed = analyze(&source, style(CaseTransform::None, width, kana));
        ordinary(&analyzed, &source, presentation);
        assert_eq!(
            emissions(&analyzed),
            [(paint, identity, 0, Range::new(0, 1))]
        );
        assert_eq!(analyzed.inner.transform.rows[0].code_point, 0xff67);
    }
}

#[test]
fn preserved_space_width_is_paint_policy_while_selection_still_sees_space() {
    let source = Source::new("A B");
    for (mode, presentation, paint_space) in [
        (WhiteSpace::Collapse, "Ａ Ｂ", 0x20),
        (WhiteSpace::Preserve, "Ａ　Ｂ", 0x3000),
    ] {
        let mut transform = style(CaseTransform::None, true, false);
        transform.white_space = mode;
        let analyzed = processed_paragraph(&source, transform);
        ordinary(&analyzed, &source, presentation);
        assert_eq!(
            emissions(&analyzed),
            [
                (0xff21, 0x41, 0, Range::new(0, 1)),
                (paint_space, 0x20, 0, Range::new(1, 2)),
                (0xff22, 0x42, 0, Range::new(2, 3)),
            ]
        );
    }
}

#[test]
fn delayed_iota_preserves_repeated_noncontiguous_correspondence_and_real_removal() {
    // Root full uppercase expands 1F80 to 1F08 + delayed 0399. The acute
    // remains between those emissions. The separately Greek-owned psili is
    // genuinely removed before the delayed iota is flushed.
    let mut source = Source::new("\u{1f80}\u{301}\u{313}");
    let mut greek = style(CaseTransform::Uppercase, false, false);
    greek.locale = Some("el".to_owned());
    source.span(Range::new(2, 3), greek);
    let analyzed = analyze(&source, style(CaseTransform::Uppercase, false, false));
    ordinary(&analyzed, &source, "\u{1f08}\u{301}Ι");
    assert_eq!(
        emissions(&analyzed),
        [
            (0x1f08, 0x1f08, 0, Range::new(0, 1)),
            (0x301, 0x301, 0, Range::new(1, 2)),
            (0x399, 0x399, 1, Range::new(0, 1)),
        ]
    );
    let map = &analyzed.inner.transform;
    assert_eq!(map.rows[2].code_point, 0x313);
    assert!(map.rows[2].outputs.is_empty());
    assert_eq!(map.original_pieces(2)[0].aggregate, Range::new(2, 3));
    assert!(matches!(
        map.events(),
        [
            RelationEvent::Text { atom: 0 },
            RelationEvent::Text { atom: 1 },
            RelationEvent::Removal { row: 2 },
            RelationEvent::Text { atom: 2 }
        ]
    ));
}

#[test]
fn an_original_combining_iota_can_emit_a_letter_without_losing_its_ancestry() {
    let source = Source::new("α\u{345}\u{301}");
    let analyzed = analyze(&source, style(CaseTransform::Uppercase, false, false));
    ordinary(&analyzed, &source, "Α\u{301}Ι");
    assert_eq!(
        emissions(&analyzed),
        [
            (0x391, 0x391, 0, Range::new(0, 1)),
            (0x301, 0x301, 0, Range::new(2, 3)),
            (0x399, 0x399, 0, Range::new(1, 2)),
        ]
    );
    let map = &analyzed.inner.transform;
    assert_eq!(map.rows[1].code_point, 0x345);
    assert_eq!(map.original_pieces(1)[0].aggregate, Range::new(1, 2));
    assert_eq!(map.original_pieces(1)[0].local, Range::new(1, 2));
}

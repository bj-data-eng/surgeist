use crate::*;

fn style() -> Style {
    Style {
        text_transform: TextTransform::MathAuto,
        ..Style::default()
    }
}

fn node(text: impl Into<Text>, range: Range) -> CompleteTextNodeContribution {
    CompleteTextNodeContribution::new(text.into(), range).unwrap()
}

fn input(source: Source) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style: style(),
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        paragraph_strut: ParagraphStrut::DefaultFont,
        tab_reference: None,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![ParagraphOrigin {
            source: source.clone(),
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: ParagraphOriginKind::Authored,
            text_node: Some(node(source.text().clone(), Range::new(0, len))),
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if len == 0 {
            vec![ParagraphAnchor {
                item: ParagraphAnchorItem::EmptyOrigin(0),
                affinity: Affinity::After,
            }]
        } else {
            vec![]
        },
    }
}

fn connection_policies(i: &mut ParagraphInput) {
    i.boundary_policies = (0..i.connections.len())
        .map(|index| ParagraphBoundaryPolicy {
            definition: ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: Wrap::Word,
            following_wrap: Wrap::Word,
            ancestor_wrap: Wrap::Word,
        })
        .collect();
}

fn reason(error: &Error) -> &TextTransformError {
    match error.detail() {
        Some(ErrorDetail::TextTransform { reason }) => reason,
        _ => panic!("{error:?}"),
    }
}

#[test]
fn every_raw_c1_pair_executes_with_actual_supplementary_output_and_original_custody() {
    // Interpret the retained raw HTML independently of the generated lookup.
    let mut count = 0;
    for row in include_str!("../../../data/MathMLCore-20250624-italic-mappings.html")
        .lines()
        .filter(|line| line.starts_with("<tr style="))
    {
        let pair: Vec<_> = row
            .split("U+")
            .skip(1)
            .map(|part| u32::from_str_radix(part.split('<').next().unwrap().trim(), 16).unwrap())
            .collect();
        assert_eq!(pair.len(), 2);
        let source = Source::new(char::from_u32(pair[0]).unwrap().to_string());
        let result = AnalyzedSource::from_paragraph(
            input(source.clone()),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(result.inner.projection.atoms.len(), 1);
        let atom = &result.inner.projection.atoms[0];
        assert_eq!(atom.code_point, pair[1]);
        assert_eq!(atom.source, Range::new(0, 1));
        assert_eq!(atom.processed, Range::new(0, 1));
        assert_eq!(
            atom.presentation,
            Range::new(0, char::from_u32(pair[1]).unwrap().len_utf16())
        );
        assert_eq!(result.source(), &source);
        assert_eq!(result.processed_source().processed_text(), source.text());
        assert_eq!(
            result.inner.transform.original_pieces(0)[0].local,
            Range::new(0, 1)
        );
        count += 1;
    }
    assert_eq!(count, 112);
}

#[test]
fn complete_original_content_not_origin_size_kind_or_resource_identity_owns_eligibility() {
    let source = Source::new("cos");
    for separate in [false, true] {
        let mut i = input(source.clone());
        i.origins = (0..3)
            .map(|at| ParagraphOrigin {
                source: source.clone(),
                local: Range::new(at, at + 1),
                aggregate: Range::new(at, at + 1),
                kind: if at == 1 {
                    ParagraphOriginKind::Generated
                } else {
                    ParagraphOriginKind::Authored
                },
                text_node: Some(if separate {
                    node(
                        Text::from_code_units(vec![source.text().code_units()[at]]),
                        Range::new(0, 1),
                    )
                } else {
                    node("cos", Range::new(at, at + 1))
                }),
            })
            .collect();
        i.connections = [1, 2]
            .map(|at| ParagraphConnection {
                at,
                kind: ParagraphConnectionKind::Continuous,
            })
            .to_vec();
        connection_policies(&mut i);
        let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        assert_eq!(
            a.inner.projection.text,
            if separate { "𝑐𝑜𝑠" } else { "cos" }
        );
        for (row, piece) in a.inner.transform.rows.iter().enumerate() {
            assert_eq!(piece.source, Range::new(row, row + 1));
            assert_eq!(a.inner.transform.original_pieces(row)[0].origin, row);
        }
    }
    for text in ["1", "∞", "𝑐", "c\u{301}", "c\u{fe0f}", "", "\u{1d400}"] {
        let a = AnalyzedSource::from_paragraph(
            input(Source::new(text)),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(a.inner.projection.text, text);
    }
    let opaque = Text::from_code_units(vec![0xd800, 0xfffd, 0xdc00]);
    let a = AnalyzedSource::from_paragraph(
        input(Source::new(opaque.clone())),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(a.source().text(), &opaque);
    assert_eq!(
        a.inner
            .projection
            .atoms
            .iter()
            .map(|a| a.code_point)
            .collect::<Vec<_>>(),
        [0xd800, 0xfffd, 0xdc00]
    );
}

#[test]
fn node_ranges_and_exact_correspondence_are_validated_even_without_math_demand() {
    for range in [Range::new(2, 1), Range::new(0, 2)] {
        let e = CompleteTextNodeContribution::new(Text::from("c"), range).unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidRange);
        assert_eq!(
            reason(&e),
            &TextTransformError::InvalidTextNodeRange { range, text_len: 1 }
        );
    }
    let half = node(
        Text::from_code_units(vec![0xd800, 0xdc00]),
        Range::new(1, 2),
    );
    assert_eq!(half.range(), Range::new(1, 2));
    let mut low = input(Source::new(Text::from_code_units(vec![0xdc00])));
    low.origins[0].text_node = Some(half);
    let actual =
        AnalyzedSource::from_paragraph(low, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert_eq!(actual.inner.projection.atoms[0].code_point, 0xdc00);

    let mut i = input(Source::new("c"));
    i.base_style.text_transform = TextTransform::none();
    i.origins[0].text_node = Some(node("A", Range::new(0, 1)));
    let e = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
        .unwrap_err();
    assert_eq!(e.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        reason(&e),
        &TextTransformError::TextNodeMismatch { origin: 0 }
    );
}

#[test]
fn demand_uses_only_winning_aggregate_styles_and_precedes_real_discard() {
    let mut discarded = input(Source::new(" "));
    discarded.base_style.white_space = WhiteSpace::Discard;
    discarded.origins[0].text_node = None;
    assert_eq!(
        reason(
            &AnalyzedSource::from_paragraph(
                discarded.clone(),
                crate::AnalysisTypography::HorizontalComposition
            )
            .unwrap_err()
        ),
        &TextTransformError::MissingTextNodeContext { origin: 0 }
    );
    discarded.origins[0].text_node = Some(node(" ", Range::new(0, 1)));
    let actual =
        AnalyzedSource::from_paragraph(discarded, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
    assert!(actual.inner.projection.atoms.is_empty());
    assert_eq!(actual.source().text(), &Text::from(" "));

    for whitespace in [WhiteSpace::Preserve, WhiteSpace::Discard] {
        let mut i = input(Source::new(" c "));
        i.base_style.white_space = whitespace;
        i.origins[0].text_node = None;
        let e = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
            .unwrap_err();
        assert_eq!(
            reason(&e),
            &TextTransformError::MissingTextNodeContext { origin: 0 }
        );
    }
    let mut resource = Source::new("c");
    resource.span(Range::new(0, 1), style());
    let mut aggregate = Source::new("c");
    aggregate.span(Range::new(0, 1), style());
    aggregate.span(Range::new(0, 1), Style::default());
    aggregate.span(Range::new(0, 0), style());
    let mut i = input(aggregate);
    i.origins[0].source = resource;
    i.origins[0].text_node = None;
    let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
        .unwrap();
    assert_eq!(a.inner.projection.text, "c");
    let mut i = input(Source::new("c"));
    i.base_style = Style::default();
    i.source.span(Range::new(0, 1), style());
    i.origins[0].text_node = None;
    assert_eq!(
        reason(
            &AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
                .unwrap_err()
        ),
        &TextTransformError::MissingTextNodeContext { origin: 0 }
    );
}

#[test]
fn original_eligibility_and_actual_surviving_phase_i_value_are_distinct() {
    let mut i = input(Source::new(" c "));
    i.base_style.white_space = WhiteSpace::Discard;
    let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
        .unwrap();
    assert_eq!(a.processed_source().processed_text(), &Text::from("c"));
    assert_eq!(a.inner.projection.text, "c");
    for whitespace in [
        WhiteSpace::Preserve,
        WhiteSpace::PreserveSpaces,
        WhiteSpace::Discard,
    ] {
        let mut i = input(Source::new("c"));
        i.base_style.white_space = whitespace;
        i.segment_breaks = DocumentSegmentBreaks::Defined(vec![Range::new(0, 1)]);
        let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        assert_eq!(
            a.inner.projection.text,
            match whitespace {
                WhiteSpace::Preserve => "\n",
                WhiteSpace::PreserveSpaces => " ",
                WhiteSpace::Discard => "",
                _ => unreachable!(),
            }
        );
        assert_eq!(a.source().text(), &Text::from("c"));
    }
    let mut i = input(Source::new(""));
    i.origins[0].text_node = None;
    assert!(
        AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition).is_ok()
    );
    let mut i = input(Source::new(""));
    i.origins.clear();
    i.anchor_order.clear();
    assert!(
        AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition).is_ok()
    );
}

#[test]
fn raw_capability_refuses_math_including_empty_and_set_hashes_remain_distinct() {
    let processed = process_source_whitespace(
        Source::new(" c "),
        Style {
            white_space: WhiteSpace::Discard,
            ..style()
        },
        DocumentSegmentBreaks::LineFeedDefault,
    )
    .unwrap();
    assert_eq!(processed.processed_text(), &Text::from("c")); // Phase I alone is not MathAuto execution.

    for text in ["c", ""] {
        let e = AnalyzedSource::new(
            Source::new(text),
            style(),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap_err();
        assert!(matches!(
            e.detail(),
            Some(ErrorDetail::UnsupportedTextStyle {
                feature: TextStyleFeature::MathAuto,
                reason: UnsupportedTextStyleReason::RequiresCompleteTextNodeContext
            })
        ));
    }
    assert_eq!(
        TextStyleFeature::MathAuto.support(TextStylePhase::ProcessedParagraph),
        TextStyleSupport::Supported
    );
    assert_eq!(
        TextStyleFeature::MathAuto.support(TextStylePhase::Standalone),
        TextStyleSupport::Unsupported(UnsupportedTextStyleReason::RequiresCompleteTextNodeContext)
    );
    assert!(TextStyleFeature::ALL.contains(&TextStyleFeature::MathAuto));
    let hash = |transform: TextTransform| {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        transform.hash(&mut h);
        h.finish()
    };
    assert_eq!(TextTransform::default(), TextTransform::none());
    assert_ne!(hash(TextTransform::MathAuto), hash(TextTransform::none()));
    assert_ne!(
        hash(TextTransform::Set(
            TextTransformSet::none().with_full_width(true)
        )),
        hash(TextTransform::none())
    );
}

#[test]
fn math_rows_leave_original_casing_context_available_to_neighboring_sets() {
    let source = Source::new("ΟΣ");
    let mut i = input(source.clone());
    i.source.span(
        Range::new(1, 2),
        Style {
            text_transform: TextTransform::Set(
                TextTransformSet::none().with_case(CaseTransform::Lowercase),
            ),
            ..Style::default()
        },
    );
    i.origins = vec![
        ParagraphOrigin {
            source: source.clone(),
            local: Range::new(0, 1),
            aggregate: Range::new(0, 1),
            kind: ParagraphOriginKind::Authored,
            text_node: Some(node("Ο", Range::new(0, 1))),
        },
        ParagraphOrigin {
            source,
            local: Range::new(1, 2),
            aggregate: Range::new(1, 2),
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        },
    ];
    i.connections = vec![ParagraphConnection {
        at: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    connection_policies(&mut i);
    let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
        .unwrap();
    assert_eq!(a.inner.projection.text, "𝛰ς");
}

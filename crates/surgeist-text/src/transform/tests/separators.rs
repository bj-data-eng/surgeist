use crate::{
    AnalyzedSource, Range, Source, Style, Text, TextTransform, TextTransformSet, WhiteSpace,
    WordSpaceTransform,
};

fn style(operation: WordSpaceTransform) -> Style {
    Style {
        word_space_transform: operation,
        ..Style::default()
    }
}

fn input(source: Source, style: Style) -> crate::ParagraphInput {
    let len = source.text().len();
    crate::ParagraphInput {
        source: source.clone(),
        base_style: style,
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

#[test]
fn explicit_rows_replace_before_casing_without_changing_original_phase_i_or_plaintext() {
    for (operation, expected) in [
        (WordSpaceTransform::None, "A\u{200b}A"),
        (WordSpaceTransform::Space, "A A"),
        (WordSpaceTransform::IdeographicSpace, "A\u{3000}A"),
    ] {
        let source = Source::new("A\u{200b}A");
        let a = AnalyzedSource::new(
            source.clone(),
            style(operation),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(a.source(), &source);
        assert_eq!(a.processed_source().processed_text(), source.text());
        assert_eq!(a.inner.projection.text, expected);
        let row = &a.inner.transform.rows[1];
        assert_eq!(row.code_point, 0x200b);
        assert_eq!(row.source, Range::new(1, 2));
        assert_eq!(row.processed, Range::new(1, 2));
        assert_eq!(row.separator_space, operation == WordSpaceTransform::Space);
        assert_eq!(
            a.inner.transform.original_pieces(1)[0].aggregate,
            Range::new(1, 2)
        );
        let plaintext: Vec<_> = a
            .processed_source()
            .plaintext_items(crate::Affinity::After)
            .flat_map(|item| match item {
                crate::PlaintextItem::Text(t) => t.code_units().to_vec(),
                crate::PlaintextItem::InlineBox(_) => vec![],
            })
            .collect();
        assert_eq!(plaintext, source.text().code_units());
    }
    for (operation, expected) in [
        (WordSpaceTransform::None, "οσ\u{200b}α"),
        (WordSpaceTransform::Space, "ος α"),
        (WordSpaceTransform::IdeographicSpace, "ος\u{3000}α"),
    ] {
        let a = AnalyzedSource::new(
            Source::new("ΟΣ\u{200b}Α"),
            Style {
                text_transform: TextTransform::Set(
                    TextTransformSet::none().with_case(crate::CaseTransform::Lowercase),
                ),
                ..style(operation)
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(a.inner.projection.text, expected);
    }
}

#[test]
fn generated_space_is_width_eligible_in_every_mode_without_replaying_phase_i() {
    for mode in [
        WhiteSpace::Preserve,
        WhiteSpace::Collapse,
        WhiteSpace::Discard,
        WhiteSpace::PreserveSpaces,
    ] {
        for (text, expected) in [
            ("A\u{200b}A", "Ａ\u{3000}Ａ"),
            ("A\u{200b}\u{200b}A", "Ａ\u{3000}\u{3000}Ａ"),
        ] {
            let source = Source::new(text);
            let a = AnalyzedSource::from_paragraph(
                input(
                    source.clone(),
                    Style {
                        white_space: mode,
                        text_transform: TextTransform::Set(
                            TextTransformSet::none().with_full_width(true),
                        ),
                        ..style(WordSpaceTransform::Space)
                    },
                ),
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(a.inner.projection.text, expected, "{mode:?}");
            assert_eq!(a.processed_source().processed_text(), source.text());
        }
        let a = AnalyzedSource::from_paragraph(
            input(
                Source::new("A A"),
                Style {
                    white_space: mode,
                    text_transform: TextTransform::Set(
                        TextTransformSet::none().with_full_width(true),
                    ),
                    ..style(WordSpaceTransform::Space)
                },
            ),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            a.inner.projection.text,
            match mode {
                WhiteSpace::Collapse => "Ａ Ａ",
                WhiteSpace::Discard => "ＡＡ",
                _ => "Ａ\u{3000}Ａ",
            }
        );
    }
}

#[test]
fn actual_winning_component_style_is_the_only_separator_authority() {
    let mut source = Source::new("A\u{200b}A\u{200b}A");
    source.span(
        Range::new(1, 2),
        style(WordSpaceTransform::IdeographicSpace),
    );
    source.span(Range::new(1, 2), Style::default());
    source.span(
        Range::new(3, 4),
        style(WordSpaceTransform::IdeographicSpace),
    );
    let a = AnalyzedSource::new(
        source,
        style(WordSpaceTransform::Space),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(a.inner.projection.text, "A\u{200b}A\u{3000}A");
    // Other authored separators and spaces are not candidate U+200B rows.
    let a = AnalyzedSource::new(
        Source::new(" \u{a0}\u{3000}\u{200b}"),
        style(WordSpaceTransform::Space),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(a.inner.projection.text, " \u{a0}\u{3000} ");
    for phase in [
        crate::TextStylePhase::Standalone,
        crate::TextStylePhase::ProcessedParagraph,
    ] {
        assert_eq!(
            crate::TextStyleFeature::ExplicitWordSeparators.support(phase),
            crate::TextStyleSupport::Supported
        );
    }
    assert!(
        crate::TextStyleFeature::ALL.contains(&crate::TextStyleFeature::ExplicitWordSeparators)
    );
    assert_eq!(
        Style::default().word_space_transform,
        WordSpaceTransform::None
    );
}

#[test]
fn forced_character_adjacency_uses_the_executed_phase_i_value() {
    for (mode, text, processed, expected) in [
        (
            WhiteSpace::Preserve,
            "A\u{200b}\nA",
            "A\u{200b}\nA",
            "A\u{200b}\nA",
        ),
        (
            WhiteSpace::Preserve,
            "A\n\u{200b}A",
            "A\n\u{200b}A",
            "A\n\u{200b}A",
        ),
        (WhiteSpace::Collapse, "A\u{200b}\nA", "A\u{200b}A", "A A"),
        (
            WhiteSpace::PreserveSpaces,
            "A\u{200b}\nA",
            "A\u{200b} A",
            "A  A",
        ),
        (WhiteSpace::Discard, "A\u{200b} \nA", "A\u{200b}A", "A A"),
    ] {
        let a = AnalyzedSource::from_paragraph(
            input(
                Source::new(text),
                Style {
                    white_space: mode,
                    ..style(WordSpaceTransform::Space)
                },
            ),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(
            a.processed_source().processed_text(),
            &Text::from(processed),
            "{mode:?}"
        );
        assert_eq!(a.inner.projection.text, expected, "{mode:?}");
    }
}

#[test]
fn genuine_controls_and_atomic_occurrences_determine_mixed_adjacency() {
    use crate::{
        ParagraphAnchor, ParagraphAnchorItem as Item, ParagraphControl,
        ParagraphControlKind as Kind,
    };
    for (kinds, expected) in [
        (vec![Kind::ForcedBreak], "A\u{200b}A"),
        (
            vec![
                Kind::ForcedBreak,
                Kind::InlineBoundaryStart,
                Kind::InlineBoundaryEnd,
            ],
            "A\u{200b}A",
        ),
        (
            vec![
                Kind::ForcedBreak,
                Kind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                },
            ],
            "A A",
        ),
        (
            vec![
                Kind::OptionalBreak {
                    wrap: crate::Wrap::Word,
                },
                Kind::ForcedBreak,
            ],
            "A\u{200b}A",
        ),
    ] {
        let mut i = input(Source::new("A\u{200b}A"), style(WordSpaceTransform::Space));
        i.controls = kinds
            .into_iter()
            .map(|kind| ParagraphControl { at: 1, kind })
            .collect();
        i.anchor_order = (0..i.controls.len())
            .map(|n| ParagraphAnchor {
                item: Item::Control(n),
                affinity: crate::Affinity::After,
            })
            .collect();
        let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        assert_eq!(a.inner.projection.text, expected);
    }
    for (kind, expected) in [
        (crate::InlineBoxKind::OutOfFlow, "A\u{200b}A"),
        (crate::InlineBoxKind::InFlow, "A A"),
    ] {
        let mut source = Source::new("A\u{200b}A");
        source.inline_box(crate::InlineBox::new(
            crate::Id::from_u64(6301),
            kind,
            1,
            crate::Size::new(0.0, 0.0),
        ));
        let mut i = input(source, style(WordSpaceTransform::Space));
        i.controls = vec![ParagraphControl {
            at: 1,
            kind: Kind::ForcedBreak,
        }];
        i.anchor_order = vec![
            ParagraphAnchor {
                item: Item::Control(0),
                affinity: crate::Affinity::After,
            },
            ParagraphAnchor {
                item: Item::Atomic(0),
                affinity: crate::Affinity::After,
            },
        ];
        i.boundary_policies = vec![
            crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::AtomicBefore(0),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            },
            crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::AtomicAfter(0),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            },
        ];
        let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
            .unwrap();
        assert_eq!(a.inner.projection.text, expected);
    }
}

#[test]
fn deleted_rows_empty_origins_and_oof_remain_transparent_to_the_actual_forced_neighbor() {
    use crate::{
        ParagraphAnchor, ParagraphAnchorItem as Item, ParagraphControl, ParagraphControlKind,
    };
    let mut source = Source::new("A \u{200b}A");
    source.inline_box(crate::InlineBox::new(
        crate::Id::from_u64(6302),
        crate::InlineBoxKind::OutOfFlow,
        1,
        crate::Size::new(0.0, 0.0),
    ));
    let mut i = input(
        source.clone(),
        Style {
            white_space: WhiteSpace::Discard,
            ..style(WordSpaceTransform::Space)
        },
    );
    i.origins = [(0, 1), (1, 1), (1, 4)]
        .into_iter()
        .map(|(start, end)| crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(start, end),
            aggregate: Range::new(start, end),
            kind: crate::ParagraphOriginKind::Authored,
        })
        .collect();
    i.connections = (0..2)
        .map(|_| crate::ParagraphConnection {
            at: 1,
            kind: crate::ParagraphConnectionKind::Continuous,
        })
        .collect();
    i.boundary_policies = (0..2)
        .map(|index| crate::ParagraphBoundaryPolicy {
            definition: crate::ParagraphBoundaryDefinition::Connection(index),
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    i.controls = vec![ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    }];
    i.anchor_order = vec![
        ParagraphAnchor {
            item: Item::Control(0),
            affinity: crate::Affinity::After,
        },
        ParagraphAnchor {
            item: Item::EmptyOrigin(1),
            affinity: crate::Affinity::After,
        },
        ParagraphAnchor {
            item: Item::Atomic(0),
            affinity: crate::Affinity::After,
        },
    ];
    let a = AnalyzedSource::from_paragraph(i, crate::AnalysisTypography::HorizontalComposition)
        .unwrap();
    assert_eq!(a.source(), &source);
    assert_eq!(
        a.processed_source().processed_text(),
        &Text::from("A\u{200b}A")
    );
    assert_eq!(a.inner.projection.text, "A\u{200b}A");
    assert_eq!(
        a.inner.transform.original_pieces(1)[0].aggregate,
        Range::new(2, 3)
    );
    assert_eq!(a.inner.transform.original_pieces(1)[0].origin, 2);
}

#[test]
fn genuine_source_local_phase_i_receipts_allow_generated_width_in_all_modes() {
    for mode in [
        WhiteSpace::Preserve,
        WhiteSpace::Collapse,
        WhiteSpace::Discard,
        WhiteSpace::PreserveSpaces,
    ] {
        let source = Source::new("A\u{200b}A");
        let processed = crate::process_source_whitespace(
            source.clone(),
            Style {
                white_space: mode,
                text_transform: TextTransform::Set(TextTransformSet::none().with_full_width(true)),
                ..style(WordSpaceTransform::Space)
            },
            crate::DocumentSegmentBreaks::LineFeedDefault,
        )
        .unwrap();
        let paragraph = crate::protocol::paragraph::ParagraphData::standalone(&source);
        let projection =
            crate::projection::SourceProjection::from_processed(&processed, &paragraph).unwrap();
        let validated =
            crate::ValidatedStyle::for_processed(processed.base_style().clone()).unwrap();
        let mut styles =
            crate::engine::resolve::analyze_styles(&source, &[], &validated, &paragraph);
        let (actual, map) =
            super::super::apply(projection, &mut styles, &processed, &paragraph).unwrap();
        assert_eq!(actual.text, "Ａ\u{3000}Ａ", "{mode:?}");
        assert_eq!(processed.processed_text(), source.text());
        assert_eq!(map.rows[1].code_point, 0x200b);
        assert!(map.rows[1].separator_space);
    }
}

fn original_separator_at_disconnected_join(
    operation: WordSpaceTransform,
    separator_before_join: bool,
    forced_on_same_side: bool,
) -> AnalyzedSource {
    use crate::*;
    let source = Source::new("A\u{200b}B");
    let at = if separator_before_join { 2 } else { 1 };
    let control = SourceProcessingEvent::Control {
        control: 0,
        affinity: Affinity::After,
    };
    let connection = SourceProcessingEvent::Connection(0);
    let control_before_connection = separator_before_join == forced_on_same_side;
    let mut events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    events.extend(if control_before_connection {
        [control, connection]
    } else {
        [connection, control]
    });
    events.extend([
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let input = ContextualSourceInput {
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source: source.clone(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: [Range::new(0, at), Range::new(at, 3)]
            .into_iter()
            .map(|range| ParagraphOrigin {
                source: source.clone(),
                local: range,
                aggregate: range,
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            })
            .collect(),
        connections: vec![SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind: ParagraphConnectionKind::Disconnected,
        }],
        controls: vec![ParagraphControl {
            at,
            kind: ParagraphControlKind::ForcedBreak,
        }],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style: style(operation),
            used_direction: ParagraphBaseDirection::LeftToRight,
        }],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events,
    };
    let processed = std::sync::Arc::new(process_contextual_source(input).unwrap());
    let analyzed = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.processed_source().processed_text(), source.text());
    assert_eq!(
        analyzed.inner.paragraph.controls[0].kind,
        ParagraphControlKind::ForcedBreak
    );
    analyzed
}

#[test]
fn original_separator_disconnected_before_join_space_ignores_other_side_forced_break() {
    let analyzed = original_separator_at_disconnected_join(WordSpaceTransform::Space, true, false);
    assert_eq!(analyzed.inner.projection.text, "A B");
}

#[test]
fn original_separator_disconnected_before_join_ideographic_space_ignores_other_side_forced_break() {
    let analyzed =
        original_separator_at_disconnected_join(WordSpaceTransform::IdeographicSpace, true, false);
    assert_eq!(analyzed.inner.projection.text, "A\u{3000}B");
}

#[test]
fn original_separator_disconnected_after_join_space_ignores_other_side_forced_break() {
    let analyzed = original_separator_at_disconnected_join(WordSpaceTransform::Space, false, false);
    assert_eq!(analyzed.inner.projection.text, "A B");
}

#[test]
fn original_separator_disconnected_after_join_ideographic_space_ignores_other_side_forced_break() {
    let analyzed =
        original_separator_at_disconnected_join(WordSpaceTransform::IdeographicSpace, false, false);
    assert_eq!(analyzed.inner.projection.text, "A\u{3000}B");
}

#[test]
fn original_separator_disconnected_same_side_forced_break_suppresses_both_replacements() {
    for operation in [
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        for before in [true, false] {
            let analyzed = original_separator_at_disconnected_join(operation, before, true);
            assert_eq!(analyzed.inner.projection.text, "A\u{200b}B");
        }
    }
}

fn original_separator_flat_disconnected(
    text: &str,
    at: usize,
    operation: WordSpaceTransform,
) -> crate::ParagraphInput {
    use crate::*;
    let source = Source::new(text);
    let mut captured = input(source.clone(), style(operation));
    captured.origins = [Range::new(0, at), Range::new(at, source.text().len())]
        .into_iter()
        .map(|range| ParagraphOrigin {
            source: source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        })
        .collect();
    captured.connections = vec![ParagraphConnection {
        at,
        kind: ParagraphConnectionKind::Disconnected,
    }];
    captured.controls = vec![ParagraphControl {
        at,
        kind: ParagraphControlKind::ForcedBreak,
    }];
    captured.anchor_order = vec![ParagraphAnchor {
        item: ParagraphAnchorItem::Control(0),
        affinity: Affinity::After,
    }];
    captured
}

fn assert_original_separator_flat_side_missing(at: usize, operation: WordSpaceTransform) {
    let error = AnalyzedSource::from_paragraph(
        original_separator_flat_disconnected("A\u{200b}B", at, operation),
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::UnsupportedFeature);
    assert!(matches!(
        error.detail(),
        Some(crate::ErrorDetail::TextTransform { .. })
    ));
}

#[test]
fn original_separator_disconnected_flat_before_join_space_requires_control_side() {
    assert_original_separator_flat_side_missing(2, WordSpaceTransform::Space);
}

#[test]
fn original_separator_disconnected_flat_before_join_ideographic_space_requires_control_side() {
    assert_original_separator_flat_side_missing(2, WordSpaceTransform::IdeographicSpace);
}

#[test]
fn original_separator_disconnected_flat_after_join_space_requires_control_side() {
    assert_original_separator_flat_side_missing(1, WordSpaceTransform::Space);
}

#[test]
fn original_separator_disconnected_flat_after_join_ideographic_space_requires_control_side() {
    assert_original_separator_flat_side_missing(1, WordSpaceTransform::IdeographicSpace);
}

#[test]
fn original_separator_disconnected_flat_none_does_not_demand_control_side() {
    for at in [1, 2] {
        let analyzed = AnalyzedSource::from_paragraph(
            original_separator_flat_disconnected("A\u{200b}B", at, WordSpaceTransform::None),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.inner.projection.text, "A\u{200b}B");
        assert_eq!(analyzed.source().text(), &Text::from("A\u{200b}B"));
    }
}

#[test]
fn original_separator_disconnected_flat_known_source_hard_break_suppression_needs_no_control_side()
{
    for operation in [
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        for (text, at) in [("A\u{200b}\nB", 3), ("A\n\u{200b}B", 1)] {
            let analyzed = AnalyzedSource::from_paragraph(
                original_separator_flat_disconnected(text, at, operation),
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(analyzed.inner.projection.text, text);
            assert_eq!(analyzed.source().text(), &Text::from(text));
        }
    }
}

#[test]
fn original_separator_disconnected_flat_intervening_original_blocks_unknown_forced_influence() {
    for (operation, before, after) in [
        (WordSpaceTransform::Space, "A XB", "AX B"),
        (
            WordSpaceTransform::IdeographicSpace,
            "A\u{3000}XB",
            "AX\u{3000}B",
        ),
    ] {
        for (text, at, expected) in [("A\u{200b}XB", 3, before), ("AX\u{200b}B", 1, after)] {
            let analyzed = AnalyzedSource::from_paragraph(
                original_separator_flat_disconnected(text, at, operation),
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(analyzed.inner.projection.text, expected);
            assert_eq!(analyzed.source().text(), &Text::from(text));
        }
    }
}

#[test]
fn original_separator_disconnected_flat_refusal_identifies_original_range_and_connection() {
    for operation in [
        WordSpaceTransform::Space,
        WordSpaceTransform::IdeographicSpace,
    ] {
        for at in [1, 2] {
            let error = AnalyzedSource::from_paragraph(
                original_separator_flat_disconnected("A\u{200b}B", at, operation),
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap_err();
            assert_eq!(
                error.detail(),
                Some(&crate::ErrorDetail::TextTransform {
                    reason: crate::TextTransformError::MissingOriginalSeparatorBoundaryContext {
                        source: Range::new(1, 2),
                        at,
                        connection: 0,
                    },
                })
            );
        }
    }
}

#[test]
fn original_separator_disconnected_flat_split_surrogate_barrier_blocks_control_influence() {
    for (operation, expected) in [
        (WordSpaceTransform::Space, "\u{fffd}\u{fffd} B"),
        (
            WordSpaceTransform::IdeographicSpace,
            "\u{fffd}\u{fffd}\u{3000}B",
        ),
    ] {
        let analyzed = AnalyzedSource::from_paragraph(
            original_separator_flat_disconnected("😀\u{200b}B", 1, operation),
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        assert_eq!(analyzed.inner.projection.text, expected);
        assert_eq!(analyzed.source().text(), &Text::from("😀\u{200b}B"));
    }
}

#[test]
fn original_separator_disconnected_flat_uncertain_forcing_on_both_sides_is_not_proven_suppression()
{
    use crate::*;
    let mut captured =
        original_separator_flat_disconnected("A\u{200b}B", 1, WordSpaceTransform::Space);
    captured.origins = [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
        .into_iter()
        .map(|range| ParagraphOrigin {
            source: captured.source.clone(),
            local: range,
            aggregate: range,
            kind: ParagraphOriginKind::Authored,
            text_node: None,
        })
        .collect();
    captured.connections.push(ParagraphConnection {
        at: 2,
        kind: ParagraphConnectionKind::Disconnected,
    });
    captured.controls.push(ParagraphControl {
        at: 2,
        kind: ParagraphControlKind::ForcedBreak,
    });
    captured.anchor_order.push(ParagraphAnchor {
        item: ParagraphAnchorItem::Control(1),
        affinity: Affinity::After,
    });
    let error =
        AnalyzedSource::from_paragraph(captured, crate::AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(matches!(error.detail(), Some(ErrorDetail::TextTransform {
        reason: TextTransformError::MissingOriginalSeparatorBoundaryContext { source, .. }
    }) if *source == Range::new(1, 2)));
}

#[test]
fn original_separator_disconnected_flat_reversed_empty_occurrences_refuse_contradictory_connection_order()
 {
    use crate::*;
    let mut captured =
        original_separator_flat_disconnected("A\u{200b}B", 1, WordSpaceTransform::Space);
    captured.origins = [
        Range::new(0, 1),
        Range::new(1, 1),
        Range::new(1, 1),
        Range::new(1, 3),
    ]
    .into_iter()
    .map(|range| ParagraphOrigin {
        source: captured.source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    })
    .collect();
    captured.connections = (0..3)
        .map(|_| ParagraphConnection {
            at: 1,
            kind: ParagraphConnectionKind::Disconnected,
        })
        .collect();
    captured.anchor_order = [
        ParagraphAnchorItem::EmptyOrigin(2),
        ParagraphAnchorItem::EmptyOrigin(1),
        ParagraphAnchorItem::Control(0),
    ]
    .into_iter()
    .map(|item| ParagraphAnchor {
        item,
        affinity: Affinity::After,
    })
    .collect();
    let error =
        AnalyzedSource::from_paragraph(captured, crate::AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::InvalidConnection {
                connection: 1,
                at: 1
            },
        })
    );
}

#[test]
fn original_separator_disconnected_flat_nearer_nonforced_control_blocks_uncertain_forced_influence()
{
    use crate::{
        Affinity, ParagraphAnchor, ParagraphAnchorItem, ParagraphControl,
        ParagraphControlKind as Kind, Wrap,
    };
    for (operation, expected) in [
        (WordSpaceTransform::Space, "A B"),
        (WordSpaceTransform::IdeographicSpace, "A\u{3000}B"),
    ] {
        for at in [1, 2] {
            let mut captured = original_separator_flat_disconnected("A\u{200b}B", at, operation);
            let kinds = if at == 1 {
                [Kind::ForcedBreak, Kind::OptionalBreak { wrap: Wrap::Word }]
            } else {
                [Kind::OptionalBreak { wrap: Wrap::Word }, Kind::ForcedBreak]
            };
            captured.controls = kinds
                .into_iter()
                .map(|kind| ParagraphControl { at, kind })
                .collect();
            captured.anchor_order = (0..2)
                .map(|control| ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(control),
                    affinity: Affinity::After,
                })
                .collect();
            let analyzed = AnalyzedSource::from_paragraph(
                captured,
                crate::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            assert_eq!(analyzed.inner.projection.text, expected);
        }
    }
}

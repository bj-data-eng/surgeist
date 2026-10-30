//! Functional/characterization contracts for exact conditional transformation.
//! Pinned full Unicode casing and native pre-casing separator replacement are
//! the literal oracles; source continuity never enlarges output coverage.

use crate::*;
use std::sync::Arc;

fn lower() -> TextTransform {
    TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Lowercase))
}

fn neighbor() -> Style {
    Style {
        white_space: WhiteSpace::Preserve,
        text_transform: TextTransform::none(),
        ..Style::default()
    }
}

fn input(
    text: &str,
    cut: usize,
    relationship: Option<ParagraphConnectionKind>,
    following: Style,
) -> ContextualSourceInput {
    let mut input = super::input(text, WhiteSpace::Preserve);
    let end = input.source.text().len();
    input.source.span(Range::new(cut, end), following);
    input.origins = [Range::new(0, cut), Range::new(cut, end)]
        .into_iter()
        .map(|range| super::origin(&input.source, range))
        .collect();
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!("the genuine source fixture has an owning Block")
    };
    base_style.text_transform = lower();
    input.paragraphs = vec![
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::LeftToRight,
            paragraph_strut: ParagraphStrut::DefaultFont,
        },
        SourceProcessingParagraph {
            base_direction: ParagraphBaseDirection::RightToLeft,
            paragraph_strut: ParagraphStrut::AllocatedContributions,
        },
    ];
    input.joint_flows.clear();
    input.connections.clear();
    input.boundary_policies.clear();
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        SourceProcessingEvent::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
    ];
    if let Some(kind) = relationship {
        input.connections.push(SourceProcessingConnection {
            before_origin: 0,
            after_origin: 1,
            kind,
        });
        input.events.push(SourceProcessingEvent::Connection(0));
        if kind == ParagraphConnectionKind::Continuous {
            input
                .boundary_policies
                .push(super::policy(ParagraphBoundaryDefinition::Connection(0)));
        }
    }
    input.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::EnterParagraph(1),
        SourceProcessingEvent::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(0),
    ]);
    input
}

fn capture(input: ContextualSourceInput) -> Arc<ProcessedSource> {
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    assert!(processed.captured_joint_flow(0).is_none());
    assert_eq!(processed.paragraphs().count(), 2);
    processed
}

fn exact(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    ordinal: usize,
    extent: Range,
    expected: &str,
) {
    assert_eq!(analyzed.source(), processed.source());
    assert_eq!(
        analyzed.processed_source().processed_text(),
        processed.source().text()
    );
    assert_eq!(analyzed.inner.projection.text, expected);
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        expected.chars().map(u32::from).collect::<Vec<_>>()
    );
    assert_eq!(
        analyzed.inner.paragraph.base_direction,
        Some(if ordinal == 0 {
            ParagraphBaseDirection::LeftToRight
        } else {
            ParagraphBaseDirection::RightToLeft
        })
    );
    assert_eq!(
        analyzed.inner.paragraph.paragraph_strut,
        if ordinal == 0 {
            ParagraphStrut::DefaultFont
        } else {
            ParagraphStrut::AllocatedContributions
        }
    );
    assert_eq!(analyzed.inner.paragraph.origins.len(), 1);
    assert_eq!(analyzed.inner.paragraph.origins[0].aggregate, extent);
    assert_eq!(analyzed.inner.paragraph.origins[0].local, extent);
    assert_eq!(
        analyzed.inner.paragraph.origins[0].original(analyzed.processed_source()),
        processed.source()
    );
    let origins = &analyzed.inner.paragraph.origin_map;
    let key = origins.flow_origin(origins.view(0).unwrap());
    assert_eq!(
        key.captured_event(),
        Some(processed.captured_origin(ordinal).unwrap().index)
    );
    assert_eq!(
        analyzed.inner.transform.rows.len(),
        expected.chars().count()
    );
    for (index, atom) in analyzed.inner.projection.atoms.iter().enumerate() {
        let original = Range::new(extent.start + index, extent.start + index + 1);
        assert_eq!(atom.source, original);
        assert_eq!(atom.processed, original);
        assert_eq!(analyzed.inner.transform.original_pieces(index).len(), 1);
        let piece = analyzed.inner.transform.original_pieces(index)[0];
        assert_eq!(piece.origin, key);
        assert_eq!(piece.aggregate, original);
        assert_eq!(piece.local, original);
    }
    let raw = processed.paragraphs().nth(ordinal).unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [extent]);
    assert!(Arc::ptr_eq(raw.processed_source(), processed));
    assert!(
        processed
            .captured_paragraph(ordinal)
            .unwrap()
            .joint_flow()
            .is_none()
    );
}

#[test]
fn sigma_context_reads_the_neighbors_actual_post_separator_scalars() {
    // U+200B is Case_Ignorable, so following Alpha prevents Final_Sigma.
    // The neighbor's own pre-casing SPACE replacement instead stops that
    // context, making Sigma final. The neighbor itself requests no casing.
    for (separator, first_text, following_text) in [
        (WordSpaceTransform::None, "οσ", "\u{200b}Α"),
        (WordSpaceTransform::Space, "ος", " Α"),
    ] {
        let processed = capture(input(
            "ΟΣ\u{200b}Α",
            2,
            Some(ParagraphConnectionKind::Continuous),
            Style {
                word_space_transform: separator,
                ..neighbor()
            },
        ));
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let first = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        exact(&first, &processed, 0, Range::new(0, 2), first_text);
        let following = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        exact(&following, &processed, 1, Range::new(2, 4), following_text);
        assert_eq!(
            super::text(raw[0].plaintext_items(Affinity::After)),
            [0x39f, 0x3a3]
        );
        assert_eq!(
            super::text(raw[1].plaintext_items(Affinity::After)),
            [0x200b, 0x391]
        );
        assert_eq!(processed.processed_text(), &Text::from("ΟΣ\u{200b}Α"));
    }
}

#[test]
fn sigma_without_a_preceding_cased_character_does_not_demand_unknown_following_context() {
    // Final_Sigma requires a preceding cased character. At this actual domain
    // start that condition is false, so its after-context cannot change sigma.
    for relationship in [
        None,
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
    ] {
        let processed = capture(input("ΣΑ", 1, relationship, neighbor()));
        let raw = processed.paragraphs().next().unwrap();
        let first = AnalyzedSource::from_processed_paragraph(
            raw.clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        exact(&first, &processed, 0, Range::new(0, 1), "σ");
        assert_eq!(super::text(raw.plaintext_items(Affinity::After)), [0x3a3]);
        assert_eq!(processed.processed_text(), &Text::from("ΣΑ"));
    }
}

#[test]
fn reading_neighbor_case_context_does_not_validate_its_unconsumed_font_fields() {
    // Source structure/whitespace capture admits this finite invalid font
    // field. processed_span_styles validates only requested coverage; looking
    // at connected Alpha's transform/Unicode context must retain that boundary.
    let processed = capture(input(
        "ΟΣΑ",
        2,
        Some(ParagraphConnectionKind::Continuous),
        Style {
            size: -1.0,
            ..neighbor()
        },
    ));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let first = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    exact(&first, &processed, 0, Range::new(0, 2), "οσ");
    assert_eq!(
        super::text(raw[0].plaintext_items(Affinity::After)),
        [0x39f, 0x3a3]
    );
    assert_eq!(
        super::text(raw[1].plaintext_items(Affinity::After)),
        [0x391]
    );

    // Requesting that actual neighbor consumes its own font style and keeps
    // the existing validation refusal; narrow context is no blanket admission.
    let error = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidStyle);
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(2, 3)]
    );
    assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
    assert!(Arc::ptr_eq(raw[0].processed_source(), &processed));
}

#[test]
fn missing_conditional_mapping_reports_transformation_and_actual_original_endpoints() {
    let processed = capture(input("ΟΣΑ", 2, None, neighbor()));
    let first = processed.paragraphs().next().unwrap();
    let error =
        AnalyzedSource::from_processed_paragraph(first, AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Transformation,
            },
        })
    );
    assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
    assert!(processed.captured_connection(0).is_none());
}

#[test]
fn missing_completed_unit_context_reports_segmentation_without_inventing_a_fence() {
    let processed = capture(input("i\u{307}", 1, None, neighbor()));
    let first = processed.paragraphs().next().unwrap();
    let error =
        AnalyzedSource::from_processed_paragraph(first, AnalysisTypography::HorizontalComposition)
            .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Segmentation,
            },
        })
    );
    assert_eq!(processed.processed_text(), &Text::from("i\u{307}"));
    assert!(processed.captured_connection(0).is_none());
}

#[test]
fn known_completed_unit_crossing_reports_its_actual_original_coverage_requirement() {
    let processed = capture(input(
        "i\u{307}",
        1,
        Some(ParagraphConnectionKind::Continuous),
        neighbor(),
    ));
    let first = processed.paragraphs().next().unwrap();
    let error = AnalyzedSource::from_processed_paragraph(
        first.clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::TextTransform {
            reason: TextTransformError::IndivisibleCoverage {
                source: Range::new(0, 2),
            },
        })
    );
    assert_eq!(
        first.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1)]
    );
    assert_eq!(processed.processed_text(), &Text::from("i\u{307}"));
}

#[test]
fn capitalization_reads_connected_word_initiality_without_joining_exact_outputs() {
    let capitalize =
        TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Capitalize));
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = input(
            "abc",
            1,
            relationship,
            Style {
                text_transform: capitalize,
                ..neighbor()
            },
        );
        let SourceProcessingScope::Block { base_style, .. } = &mut request.scopes[0] else {
            unreachable!()
        };
        base_style.text_transform = capitalize;
        let processed = capture(request);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        // At the actual domain start, a is the first lowercase letter unit
        // whether the following member continues this word or starts another.
        let first = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        exact(&first, &processed, 0, Range::new(0, 1), "A");
        let second = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        );
        if let Some(kind) = relationship {
            let expected = if kind == ParagraphConnectionKind::Continuous {
                "bc"
            } else {
                "Bc"
            };
            exact(&second.unwrap(), &processed, 1, Range::new(1, 3), expected);
        } else {
            let error = second.unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::SourceProcessing {
                    reason: SourceProcessingError::MissingSourceContinuity {
                        before_origin: 0,
                        after_origin: 1,
                        operation: SourceContinuityOperation::Transformation,
                    },
                })
            );
        }
        assert_eq!(processed.processed_text(), &Text::from("abc"));
    }
}

fn styled_prefix(
    locale: &str,
    relationship: Option<ParagraphConnectionKind>,
) -> ContextualSourceInput {
    let mut request = input("AB", 1, relationship, neighbor());
    request.source.span(
        Range::new(0, 1),
        Style {
            locale: Some(locale.to_owned()),
            text_transform: TextTransform::Set(
                TextTransformSet::none().with_case(CaseTransform::Uppercase),
            ),
            ..neighbor()
        },
    );
    for origin in &mut request.origins {
        origin.source = request.source.clone();
    }
    request
}

#[test]
fn exact_identity_view_does_not_execute_an_unrequested_prefix_locale() {
    for locale in ["en-US", "en--US"] {
        let processed = capture(styled_prefix(
            locale,
            Some(ParagraphConnectionKind::Continuous),
        ));
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let second = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_or_else(|error| panic!("unrequested prefix locale {locale:?}: {error:?}"));
        exact(&second, &processed, 1, Range::new(1, 2), "B");
        let first = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        );
        if locale == "en-US" {
            exact(&first.unwrap(), &processed, 0, Range::new(0, 1), "A");
        } else {
            assert_eq!(first.unwrap_err().code, ErrorCode::InvalidStyle);
        }
        assert_eq!(processed.processed_text(), &Text::from("AB"));
    }
}

#[test]
fn irish_uppercase_a_excludes_prefix_rules_without_unknown_following_context() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let processed = capture(styled_prefix("ga", relationship));
        let raw = processed.paragraphs().next().unwrap();
        let first = AnalyzedSource::from_processed_paragraph(
            raw,
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_or_else(|error| panic!("relationship {relationship:?}: {error:?}"));
        exact(&first, &processed, 0, Range::new(0, 1), "A");
        assert_eq!(processed.processed_text(), &Text::from("AB"));
    }
}

#[test]
fn a_requested_forced_stop_does_not_execute_a_remote_separator_request() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = input(
            "A\n\u{200b}",
            2,
            relationship,
            Style {
                word_space_transform: WordSpaceTransform::Space,
                ..neighbor()
            },
        );
        let SourceProcessingScope::Block { base_style, .. } = &mut request.scopes[0] else {
            unreachable!()
        };
        base_style.text_transform = TextTransform::none();
        let processed = capture(request);
        let raw = processed.paragraphs().next().unwrap();
        let first = AnalyzedSource::from_processed_paragraph(
            raw.clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_or_else(|error| panic!("relationship {relationship:?}: {error:?}"));
        exact(&first, &processed, 0, Range::new(0, 2), "A\n");
        assert_eq!(
            super::text(raw.plaintext_items(Affinity::After)),
            [0x41, 0x0a]
        );
        assert_eq!(processed.processed_text(), &Text::from("A\n\u{200b}"));
    }
}

#[test]
fn irish_prefix_effect_requires_its_actual_cross_word_source_relationship() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let mut request = input("t-uisce", 1, relationship, neighbor());
        request.source.span(
            Range::new(0, 1),
            Style {
                locale: Some("ga".to_owned()),
                text_transform: TextTransform::Set(
                    TextTransformSet::none().with_case(CaseTransform::Uppercase),
                ),
                ..neighbor()
            },
        );
        for origin in &mut request.origins {
            origin.source = request.source.clone();
        }
        let processed = capture(request);
        let raw = processed.paragraphs().next().unwrap();
        let first = AnalyzedSource::from_processed_paragraph(
            raw,
            AnalysisTypography::HorizontalComposition,
        );
        if let Some(kind) = relationship {
            let expected = if kind == ParagraphConnectionKind::Continuous {
                "t"
            } else {
                "T"
            };
            exact(&first.unwrap(), &processed, 0, Range::new(0, 1), expected);
        } else {
            let error = first.unwrap_err();
            assert_eq!(error.code, ErrorCode::UnsupportedFeature);
            assert_eq!(
                error.detail(),
                Some(&ErrorDetail::SourceProcessing {
                    reason: SourceProcessingError::MissingSourceContinuity {
                        before_origin: 0,
                        after_origin: 1,
                        operation: SourceContinuityOperation::Transformation,
                    },
                })
            );
        }
        assert_eq!(processed.processed_text(), &Text::from("t-uisce"));
    }
}

fn requested_suffix_with_prefix(
    text: &str,
    relationship: Option<ParagraphConnectionKind>,
    following: Style,
    prefix: Style,
) -> ContextualSourceInput {
    let mut request = input(text, 1, relationship, following);
    request.source.span(Range::new(0, 1), prefix);
    for origin in &mut request.origins {
        origin.source = request.source.clone();
    }
    request
}

fn suffix_custody(
    analyzed: &AnalyzedSource,
    processed: &Arc<ProcessedSource>,
    extent: Range,
    expected: &str,
) {
    assert_eq!(analyzed.source(), processed.source());
    assert_eq!(
        analyzed.processed_source().processed_text(),
        processed.source().text()
    );
    assert_eq!(analyzed.inner.projection.text, expected);
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|atom| atom.code_point)
            .collect::<Vec<_>>(),
        expected.chars().map(u32::from).collect::<Vec<_>>()
    );
    assert_eq!(
        analyzed.inner.paragraph.base_direction,
        Some(ParagraphBaseDirection::RightToLeft)
    );
    assert_eq!(
        analyzed.inner.paragraph.paragraph_strut,
        ParagraphStrut::AllocatedContributions
    );
    assert_eq!(analyzed.inner.paragraph.origins.len(), 1);
    assert_eq!(analyzed.inner.paragraph.origins[0].aggregate, extent);
    assert_eq!(analyzed.inner.paragraph.origins[0].local, extent);
    assert_eq!(
        analyzed.inner.paragraph.origins[0].original(analyzed.processed_source()),
        processed.source()
    );
    let origins = &analyzed.inner.paragraph.origin_map;
    let key = origins.flow_origin(origins.view(0).unwrap());
    assert_eq!(
        key.captured_event(),
        Some(processed.captured_origin(1).unwrap().index)
    );
    // These fixtures have one BMP scalar per input row, independently of
    // expansion/removal. Output length is deliberately not the row count.
    assert_eq!(analyzed.inner.transform.rows.len(), extent.len());
    for index in 0..extent.len() {
        let original = Range::new(extent.start + index, extent.start + index + 1);
        assert_eq!(analyzed.inner.transform.rows[index].source, original);
        let component = analyzed
            .inner
            .transform
            .components
            .iter()
            .find(|component| {
                component.owner == crate::transform::relation::ComponentOwner::Original(index)
            })
            .expect("each original input retains its component, including removals");
        assert_eq!(component.processed, original);
        assert_eq!(analyzed.inner.transform.original_pieces(index).len(), 1);
        let piece = analyzed.inner.transform.original_pieces(index)[0];
        assert_eq!(piece.origin, key);
        assert_eq!(piece.aggregate, original);
        assert_eq!(piece.local, original);
    }
    let raw = processed.paragraphs().nth(1).unwrap();
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [extent]);
    assert!(Arc::ptr_eq(raw.processed_source(), processed));
    assert!(processed.captured_joint_flow(0).is_none());
}

fn suffix_emissions(analyzed: &AnalyzedSource) -> Vec<(u32, usize, usize)> {
    let map = &analyzed.inner.transform;
    map.atoms
        .iter()
        .map(|&id| {
            let emitted = map.emissions[id];
            (
                emitted.code_point,
                map.original_row_for_atom(emitted.atom).unwrap(),
                emitted.subposition,
            )
        })
        .collect()
}

fn requires_transform_edge(error: &Error) {
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Transformation,
            },
        })
    );
}

#[test]
fn unowned_wbr_replacement_supplies_its_actual_pre_case_boundary() {
    for (replacement, expected) in [
        (WordSpaceTransform::None, "οσ"),
        (WordSpaceTransform::Space, "ος"),
    ] {
        let mut request = input(
            "ΟΣΑ",
            2,
            Some(ParagraphConnectionKind::Continuous),
            neighbor(),
        );
        request.controls.push(ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::Wbr {
                style: Box::new(Style {
                    word_space_transform: replacement,
                    ..neighbor()
                }),
            },
        });
        let following = request
            .events
            .iter()
            .position(|event| matches!(event, SourceProcessingEvent::Origin { origin: 1, .. }))
            .unwrap();
        request.events.insert(
            following,
            SourceProcessingEvent::Control {
                control: 0,
                affinity: Affinity::After,
            },
        );
        let processed = capture(request);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let wbr = processed.captured_control(0).unwrap();
        assert_eq!(wbr.paragraph(), processed.captured_paragraph(1).unwrap());
        assert_eq!(wbr.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(wbr.source_position().get(), 2);
        assert!(raw[0].data().controls.is_empty());
        assert_eq!(raw[1].data().controls.len(), 1);
        assert!(
            matches!(wbr.kind(), ParagraphControlKind::Wbr { style } if style.word_space_transform == replacement)
        );
        let edge = processed.captured_connection(0).unwrap();
        assert!(edge.event_position() < wbr.event_position());
        assert!(wbr.event_position() < processed.captured_origin(1).unwrap());
        // The control belongs to the other exact view. Its own replacement
        // supplies context, never an extra output participant in this view.
        let first = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        exact(&first, &processed, 0, Range::new(0, 2), expected);
        assert!(first.inner.paragraph.controls.is_empty());
        assert_eq!(processed.processed_text(), &Text::from("ΟΣΑ"));
        assert_eq!(
            super::text(raw[0].plaintext_items(Affinity::After)),
            [0x39f, 0x3a3]
        );
        assert_eq!(
            super::text(raw[1].plaintext_items(Affinity::After)),
            [0x391]
        );
    }
}

#[test]
fn irish_lowercase_suffix_uses_raw_prefix_without_executing_its_invalid_locale() {
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let request = requested_suffix_with_prefix(
            "nÓg",
            relationship,
            Style {
                locale: Some("ga".to_owned()),
                text_transform: lower(),
                ..neighbor()
            },
            Style {
                locale: Some("en--US".to_owned()),
                text_transform: TextTransform::Set(
                    TextTransformSet::none().with_case(CaseTransform::Uppercase),
                ),
                ..neighbor()
            },
        );
        let processed = capture(request);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let second = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        );
        if let Some(kind) = relationship {
            let second = second.unwrap();
            if kind == ParagraphConnectionKind::Continuous {
                // Native Irish lowercase inserts the hyphen in Ó's actual
                // mapping, even though the unrequested prefix asks to uppercase.
                suffix_custody(&second, &processed, Range::new(1, 3), "-óg");
                assert_eq!(
                    suffix_emissions(&second),
                    [(0x2d, 0, 0), (0xf3, 0, 1), (0x67, 1, 0)]
                );
                assert_eq!(second.inner.projection.atoms[0].source, Range::new(1, 2));
                assert_eq!(second.inner.projection.atoms[1].source, Range::new(1, 2));
            } else {
                suffix_custody(&second, &processed, Range::new(1, 3), "óg");
                assert_eq!(suffix_emissions(&second), [(0xf3, 0, 0), (0x67, 1, 0)]);
            }
        } else {
            requires_transform_edge(&second.unwrap_err());
        }
        let prefix = AnalyzedSource::from_processed_paragraph(
            raw[0].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_err();
        assert_eq!(prefix.code, ErrorCode::InvalidStyle);
        assert_eq!(super::text(raw[0].plaintext_items(Affinity::After)), [0x6e]);
        assert_eq!(
            super::text(raw[1].plaintext_items(Affinity::After)),
            [0xd3, 0x67]
        );
        assert_eq!(processed.processed_text(), &Text::from("nÓg"));
    }
}

#[test]
fn irish_uppercase_suffix_removes_only_its_own_hyphen_with_real_prefix_continuity() {
    use crate::transform::relation::RelationEvent;
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let request = requested_suffix_with_prefix(
            "t-uisce",
            relationship,
            Style {
                locale: Some("ga".to_owned()),
                text_transform: TextTransform::Set(
                    TextTransformSet::none().with_case(CaseTransform::Uppercase),
                ),
                ..neighbor()
            },
            neighbor(),
        );
        let processed = capture(request);
        let raw = processed.paragraphs().collect::<Vec<_>>();
        let second = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        );
        if let Some(kind) = relationship {
            let second = second.unwrap();
            if kind == ParagraphConnectionKind::Continuous {
                suffix_custody(&second, &processed, Range::new(1, 7), "UISCE");
                assert_eq!(
                    suffix_emissions(&second),
                    [
                        (0x55, 1, 0),
                        (0x49, 2, 0),
                        (0x53, 3, 0),
                        (0x43, 4, 0),
                        (0x45, 5, 0)
                    ]
                );
                assert!(second.inner.transform.rows[0].outputs.is_empty());
                assert_eq!(
                    second
                        .inner
                        .transform
                        .events()
                        .iter()
                        .filter(|event| matches!(event, RelationEvent::Removal { row: 0, .. }))
                        .count(),
                    1
                );
            } else {
                suffix_custody(&second, &processed, Range::new(1, 7), "-UISCE");
                assert_eq!(
                    suffix_emissions(&second),
                    [
                        (0x2d, 0, 0),
                        (0x55, 1, 0),
                        (0x49, 2, 0),
                        (0x53, 3, 0),
                        (0x43, 4, 0),
                        (0x45, 5, 0)
                    ]
                );
                assert!(
                    second
                        .inner
                        .transform
                        .events()
                        .iter()
                        .all(|event| !matches!(event, RelationEvent::Removal { .. }))
                );
            }
        } else {
            requires_transform_edge(&second.unwrap_err());
        }
        assert_eq!(super::text(raw[0].plaintext_items(Affinity::After)), [0x74]);
        assert_eq!(
            super::text(raw[1].plaintext_items(Affinity::After)),
            "-uisce".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(processed.processed_text(), &Text::from("t-uisce"));
    }
}

fn foreign_combining_unit_requires_complete_coverage(marks: usize, suffix: bool) {
    let text = format!("A{}", "\u{301}".repeat(marks));
    let cut = if suffix { marks } else { 1 };
    let mut request = input(
        &text,
        cut,
        Some(ParagraphConnectionKind::Continuous),
        neighbor(),
    );
    let SourceProcessingScope::Block { base_style, .. } = &mut request.scopes[0] else {
        unreachable!("fixture has its genuine Block owner")
    };
    base_style.text_transform = TextTransform::none();
    let processed = capture(request);
    let raw = processed.paragraphs().nth(usize::from(suffix)).unwrap();
    let error = AnalyzedSource::from_processed_paragraph(
        raw.clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::TextTransform {
            reason: TextTransformError::IndivisibleCoverage {
                source: Range::new(0, marks + 1),
            },
        })
    );
    let own = if suffix {
        Range::new(marks, marks + 1)
    } else {
        Range::new(0, 1)
    };
    assert_eq!(raw.source_pieces().collect::<Vec<_>>(), [own]);
    assert_eq!(processed.processed_text(), &Text::from(text.as_str()));
    assert!(processed.captured_joint_flow(0).is_none());
}

#[test]
fn small_foreign_combining_unit_reports_its_complete_required_coverage() {
    foreign_combining_unit_requires_complete_coverage(32, false);
}

#[test]
fn long_foreign_combining_unit_reports_its_complete_required_coverage() {
    foreign_combining_unit_requires_complete_coverage(16_400, false);
}

#[test]
fn small_foreign_combining_prefix_reports_its_complete_required_coverage() {
    foreign_combining_unit_requires_complete_coverage(32, true);
}

#[test]
fn long_foreign_combining_prefix_reports_its_complete_required_coverage() {
    foreign_combining_unit_requires_complete_coverage(16_400, true);
}

fn one_captured_word_retains_its_mapping(
    original: String,
    expected: String,
    case: CaseTransform,
    locale: Option<&str>,
) {
    let length = original.len();
    let mut request = super::input(original.as_str(), WhiteSpace::Preserve);
    let SourceProcessingScope::Block { base_style, .. } = &mut request.scopes[0] else {
        unreachable!("fixture has its genuine Block owner")
    };
    base_style.text_transform = TextTransform::Set(TextTransformSet::none().with_case(case));
    base_style.locale = locale.map(str::to_owned);
    assert_eq!(request.origins.len(), 1);
    assert_eq!(request.paragraphs.len(), 1);
    assert!(request.joint_flows.is_empty());
    let source = request.source.clone();
    let processed = Arc::new(process_contextual_source(request).unwrap());
    let raw = processed.paragraphs().next().unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        raw.clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.text, expected);
    assert_eq!(analyzed.source(), &source);
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(original.as_str()));
    assert_eq!(
        raw.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, length)]
    );
    assert_eq!(analyzed.inner.projection.atoms.len(), length);
    assert_eq!(analyzed.inner.transform.rows.len(), length);
    let key = analyzed
        .inner
        .paragraph
        .origin_map
        .flow_origin(analyzed.inner.paragraph.origin_map.view(0).unwrap());
    assert_eq!(
        key.captured_event(),
        Some(processed.captured_origin(0).unwrap().index)
    );
    for (index, (atom, expected)) in analyzed
        .inner
        .projection
        .atoms
        .iter()
        .zip(expected.chars())
        .enumerate()
    {
        let original_range = Range::new(index, index + 1);
        assert_eq!(atom.code_point, u32::from(expected));
        assert_eq!(atom.source, original_range);
        assert_eq!(atom.processed, original_range);
        let pieces = analyzed.inner.transform.original_pieces(index);
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].origin, key);
        assert_eq!(pieces[0].aggregate, original_range);
        assert_eq!(pieces[0].local, original_range);
    }
}

fn one_captured_word_capitalizes_only_its_initial_letter(length: usize) {
    one_captured_word_retains_its_mapping(
        "a".repeat(length),
        format!("A{}", "a".repeat(length - 1)),
        CaseTransform::Capitalize,
        None,
    );
}

#[test]
fn short_captured_word_capitalizes_only_its_initial_letter() {
    one_captured_word_capitalizes_only_its_initial_letter(32);
}

#[test]
fn long_captured_word_capitalizes_only_its_initial_letter() {
    one_captured_word_capitalizes_only_its_initial_letter(16_400);
}

fn one_irish_word_with_no_finite_prefix_uppercases_every_letter(length: usize) {
    // Repeated t has none of the native t-vowel, t-hyphen-vowel or ts patterns.
    one_captured_word_retains_its_mapping(
        "t".repeat(length),
        "T".repeat(length),
        CaseTransform::Uppercase,
        Some("ga"),
    );
}

#[test]
fn short_irish_word_without_a_finite_prefix_uppercases_every_letter() {
    one_irish_word_with_no_finite_prefix_uppercases_every_letter(32);
}

#[test]
fn long_irish_word_without_a_finite_prefix_uppercases_every_letter() {
    one_irish_word_with_no_finite_prefix_uppercases_every_letter(16_400);
}

#[test]
fn an_empty_raw_member_does_not_own_an_adjacent_japanese_phrase_separator() {
    use crate::protocol::paragraph::SemanticAnchorItem;
    use crate::transform::relation::RelationEvent;

    let sentence = "今日は天気です。";
    let empty_resource = Source::identified("", Id::from_u64(78461), 0);
    let build = |relationship: Option<ParagraphConnectionKind>| {
        let mut request = super::input(sentence, WhiteSpace::Preserve);
        let end = request.source.text().len();
        request.origins = vec![
            super::origin(&request.source, Range::new(0, 3)),
            ParagraphOrigin {
                source: empty_resource.clone(),
                local: Range::new(0, 0),
                aggregate: Range::new(3, 3),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            },
            super::origin(&request.source, Range::new(3, end)),
        ];
        let SourceProcessingScope::Block {
            topology,
            base_style,
            ..
        } = &mut request.scopes[0]
        else {
            unreachable!("actual source domain root")
        };
        *topology = SourceProcessingTopology::Complete;
        base_style.locale = Some("ja".to_owned());
        base_style.text_transform = TextTransform::none();
        base_style.word_space_transform = WordSpaceTransform::SpaceAutoPhrase;
        request.paragraphs = vec![
            SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            },
            SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::RightToLeft,
                paragraph_strut: ParagraphStrut::AllocatedContributions,
            },
            SourceProcessingParagraph {
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
            },
        ];
        request.joint_flows.clear();
        request.controls.clear();
        request.connections.clear();
        request.boundary_policies.clear();
        request.events = vec![SourceProcessingEvent::EnterScope(0)];
        for ordinal in 0..3 {
            request.events.extend([
                SourceProcessingEvent::EnterParagraph(ordinal),
                SourceProcessingEvent::Origin {
                    origin: ordinal,
                    affinity: Affinity::After,
                },
            ]);
            if ordinal < 2
                && let Some(kind) = relationship
            {
                request.connections.push(SourceProcessingConnection {
                    before_origin: ordinal,
                    after_origin: ordinal + 1,
                    kind,
                });
                request
                    .events
                    .push(SourceProcessingEvent::Connection(ordinal));
                if kind == ParagraphConnectionKind::Continuous {
                    request.boundary_policies.push(super::policy(
                        ParagraphBoundaryDefinition::Connection(ordinal),
                    ));
                }
            }
            request
                .events
                .push(SourceProcessingEvent::ExitParagraph(ordinal));
        }
        request.events.push(SourceProcessingEvent::ExitScope(0));
        Arc::new(process_contextual_source(request).unwrap())
    };

    // The published native phrase precedent is 今日は|天気です。. This
    // independent known-Continuous control observes its actual generated-cut
    // receipt, rather than assigning custody by numeric position or affinity.
    let known = build(Some(ParagraphConnectionKind::Continuous));
    let preceding = known.paragraphs().next().unwrap();
    let control = AnalyzedSource::from_processed_paragraph(
        preceding,
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(control.inner.projection.text, "今日は ");
    assert_eq!(control.inner.transform.virtuals.len(), 1);
    let separator = &control.inner.transform.virtuals[0];
    assert_eq!(separator.source, 3);
    assert_eq!(separator.processed, 3);
    assert_eq!(control.inner.transform.scopes[separator.scope].ordinal, 0);
    assert_eq!(
        separator.capture_cut,
        known.captured_connection(0).unwrap().event_position().index
    );
    let topology = control.inner.paragraph.scope_topology.as_ref().unwrap();
    assert_eq!(
        topology.capture.events[separator.capture_cut].paragraph,
        Some(0)
    );
    assert!(separator.capture_cut < known.captured_paragraph(1).unwrap().enter_position().index);
    assert_eq!(
        control.inner.paragraph.origins[0].aggregate,
        Range::new(0, 3)
    );

    // Only the empty middle is requested in the diagnostic cases. Its genuine
    // anchor is later than the preceding paragraph's canonical generated cut.
    for relationship in [
        Some(ParagraphConnectionKind::Continuous),
        Some(ParagraphConnectionKind::Disconnected),
        None,
    ] {
        let processed = build(relationship);
        let source = processed.source().clone();
        let raw = processed.paragraphs().collect::<Vec<_>>();
        assert_eq!(raw.len(), 3);
        assert_eq!(
            raw[0].source_pieces().collect::<Vec<_>>(),
            [Range::new(0, 3)]
        );
        assert_eq!(
            raw[1].source_pieces().collect::<Vec<_>>(),
            [Range::new(3, 3)]
        );
        assert_eq!(
            raw[2].source_pieces().collect::<Vec<_>>(),
            [Range::new(3, source.text().len())]
        );
        let middle = AnalyzedSource::from_processed_paragraph(
            raw[1].clone(),
            AnalysisTypography::HorizontalComposition,
        )
        .unwrap_or_else(|error| panic!("empty member with {relationship:?}: {error:?}"));
        assert_eq!(middle.inner.projection.text, "");
        assert!(middle.inner.projection.atoms.is_empty());
        assert!(middle.inner.transform.rows.is_empty());
        assert!(middle.inner.transform.emissions.is_empty());
        assert!(middle.inner.transform.virtuals.is_empty());
        assert_eq!(middle.source(), &source);
        assert_eq!(processed.processed_text(), &Text::from(sentence));
        assert_eq!(
            middle.inner.paragraph.base_direction,
            Some(ParagraphBaseDirection::RightToLeft)
        );
        assert_eq!(
            middle.inner.paragraph.paragraph_strut,
            ParagraphStrut::AllocatedContributions
        );
        assert_eq!(middle.inner.paragraph.origins.len(), 1);
        let origin = &middle.inner.paragraph.origins[0];
        assert_eq!(origin.aggregate, Range::new(3, 3));
        assert_eq!(origin.local, Range::new(0, 0));
        assert_eq!(origin.original(middle.processed_source()), &empty_resource);
        let origins = &middle.inner.paragraph.origin_map;
        assert_eq!(
            origins
                .flow_origin(origins.view(0).unwrap())
                .captured_event(),
            Some(processed.captured_origin(1).unwrap().index)
        );
        assert!(matches!(
            middle.inner.transform.events(),
            [RelationEvent::Anchor { order: 0 }]
        ));
        assert_eq!(middle.inner.paragraph.anchor_order.len(), 1);
        assert!(matches!(
            middle.inner.paragraph.anchor_order[0].item,
            SemanticAnchorItem::EmptyOrigin(0)
        ));
        assert_eq!(
            middle.inner.paragraph.anchor_order[0].affinity,
            Affinity::After
        );
        assert!(Arc::ptr_eq(raw[1].processed_source(), &processed));
        assert!(super::text(raw[1].plaintext_items(Affinity::After)).is_empty());
        assert!(processed.captured_joint_flow(0).is_none());
        assert!(processed.captured_control(0).is_none());
    }
}

fn one_captured_ascii_phrase_run_preserves_its_original_output(length: usize) {
    // Retained phrase/data.rs (BudouX 0.9.3) has no feature key containing
    // U+0061 and has BIAS=-3216. Every complete all-a feature window therefore
    // contributes zero; unavailable edge windows also contribute zero. Each
    // native score remains -3216, so no internal cut is positive. Expected
    // text is this literal input, never production model output as an oracle.
    let text = "a".repeat(length);
    let mut request = super::input(text.clone(), WhiteSpace::Preserve);
    request
        .source
        .set_identity(Some(Id::from_u64(78470)), u64::try_from(length).unwrap());
    request.origins = vec![super::origin(&request.source, Range::new(0, length))];
    let SourceProcessingScope::Block {
        topology,
        base_style,
        ..
    } = &mut request.scopes[0]
    else {
        unreachable!("one actual captured source domain")
    };
    *topology = SourceProcessingTopology::Complete;
    base_style.locale = Some("ja".to_owned());
    base_style.word_space_transform = WordSpaceTransform::SpaceAutoPhrase;
    base_style.text_transform = TextTransform::none();
    request.joint_flows.clear();
    request.connections.clear();
    request.boundary_policies.clear();
    request.controls.clear();
    let source = request.source.clone();
    let processed = Arc::new(process_contextual_source(request).unwrap());
    assert_eq!(processed.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from(text.clone()));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 1);
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, length)]
    );
    let analyzed = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    exact(&analyzed, &processed, 0, Range::new(0, length), &text);
    assert_eq!(analyzed.inner.transform.phrases.len(), 1);
    assert!(analyzed.inner.transform.virtuals.is_empty());
    assert_eq!(analyzed.inner.transform.emissions.len(), length);
    assert_eq!(
        super::text(raw[0].plaintext_items(Affinity::After)),
        vec![0x61; length]
    );
    assert!(processed.captured_joint_flow(0).is_none());
    assert!(processed.captured_connection(0).is_none());
    assert!(processed.captured_control(0).is_none());
    let actual_capture = &raw[0].data().scope_topology.as_ref().unwrap().capture;
    assert!(Arc::ptr_eq(
        actual_capture,
        &analyzed
            .inner
            .paragraph
            .scope_topology
            .as_ref()
            .unwrap()
            .capture
    ));
}

#[test]
fn short_ascii_phrase_run_retains_one_native_receipt_without_inserting_space() {
    one_captured_ascii_phrase_run_preserves_its_original_output(32);
}

#[test]
fn long_ascii_phrase_run_retains_one_native_receipt_without_inserting_space() {
    one_captured_ascii_phrase_run_preserves_its_original_output(16_400);
}

#[test]
fn exact_unconditional_greek_alpha_does_not_transform_unknown_preceding_sigma() {
    let following = Style {
        text_transform: lower(),
        ..neighbor()
    };
    let processed = capture(input("ΟΣΑ", 2, None, following));
    let raw = processed.paragraphs().collect::<Vec<_>>();
    let alpha = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .expect("unconditional Alpha needs no preceding Sigma transformation");
    exact(&alpha, &processed, 1, Range::new(2, 3), "α");
    let error = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: 0,
                after_origin: 1,
                operation: SourceContinuityOperation::Transformation,
            },
        })
    ));
}

//! Actual font-free interpretation, original custody and phase-admission witnesses.
use std::sync::Arc;

use icu_properties::{CodePointMapData, props::BidiClass};

use crate::{
    Affinity, AnalysisTypography, AnalyzedSource, CaseTransform, ContextualSourceInput, Direction,
    DocumentSegmentBreaks, Error, ErrorCode, ErrorDetail, Font, FontCatalogueMode, Id, LineHeight,
    LineOrientation, Options, ParagraphAnchor, ParagraphAnchorItem, ParagraphBaseDirection,
    ParagraphConnection, ParagraphConnectionKind, ParagraphControl, ParagraphControlKind,
    ParagraphInput, ParagraphOrigin, ParagraphOriginKind, ParagraphStrut, ParagraphTerminalEdge,
    PreparationOptions, ProtocolContextField, ProtocolError, Range, Source,
    SourceProcessingEnvelope, SourceProcessingEvent, SourceProcessingParagraph,
    SourceProcessingScope, SourceProcessingTopology, Style, System, SystemOptions, Text,
    TextTransform, TextTransformSet, WhiteSpace, WhiteSpaceTrim, Wrap, WrappingPolicy,
    process_contextual_source,
};

use super::wrapping::Event;
use crate::protocol::paragraph::SemanticAnchorItem;

#[cfg(feature = "text-accessibility")]
mod accessibility;

const H: AnalysisTypography = AnalysisTypography::HorizontalComposition;
const LTR: ParagraphBaseDirection = ParagraphBaseDirection::LeftToRight;
const RTL: ParagraphBaseDirection = ParagraphBaseDirection::RightToLeft;
const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn upright(computed_direction: ParagraphBaseDirection) -> AnalysisTypography {
    AnalysisTypography::Upright { computed_direction }
}
fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}
fn source(text: impl Into<Text>) -> Source {
    Source::identified(text, Id::from_u64(1533001), 17)
}
fn origin(source: &Source, range: Range) -> ParagraphOrigin {
    ParagraphOrigin {
        source: source.clone(),
        local: range,
        aggregate: range,
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    }
}
fn native(source: Source) -> ParagraphInput {
    ParagraphInput {
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![origin(&source, Range::new(0, source.text().len()))],
        source,
        base_style: style(),
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        base_direction: LTR,
        paragraph_strut: ParagraphStrut::DefaultFont,
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}
fn contextual(source: Source) -> ContextualSourceInput {
    ContextualSourceInput {
        origins: vec![origin(&source, Range::new(0, source.text().len()))],
        source,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        computed_line_boundaries: vec![],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Unknown,
            used_direction: LTR,
            trim: WhiteSpaceTrim::default(),
            base_style: style(),
        }],
        paragraphs: vec![SourceProcessingParagraph {
            base_direction: LTR,
            paragraph_strut: ParagraphStrut::DefaultFont,
        }],
        events: vec![
            SourceProcessingEvent::EnterScope(0),
            SourceProcessingEvent::EnterParagraph(0),
            SourceProcessingEvent::Origin {
                origin: 0,
                affinity: Affinity::After,
            },
            SourceProcessingEvent::ExitParagraph(0),
            SourceProcessingEvent::ExitScope(0),
        ],
    }
}
fn protocol(error: Error, expected: ProtocolError) {
    assert_eq!(error.code, ErrorCode::InvalidTextProtocol);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::InvalidTextProtocol { reason: expected })
    );
}
fn zero_levels(analyzed: &AnalyzedSource, count: usize) {
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    assert_eq!(analyzed.inner.analysis.base_level, 0);
    assert_eq!(analyzed.inner.analysis.levels, vec![0; count]);
}
fn text_cuts(analyzed: &AnalyzedSource) -> Vec<usize> {
    analyzed
        .inner
        .logical
        .events
        .iter()
        .enumerate()
        .filter_map(|(i, e)| matches!(e.kind, Event::Text(_)).then_some(i))
        .collect()
}

#[test]
fn used_ltr_is_executed_before_uba_while_computed_rtl_and_raw_properties_survive() {
    let original = source("א1ب()");
    let horizontal = AnalyzedSource::new(original.clone(), style(), H).unwrap();
    assert_eq!(horizontal.direction(), Direction::RightToLeft);
    // Pinned original properties: R, EN, AL, ON, ON. A forced base alone
    // would still leave odd R/AL and numeric level2, unlike effective strong L.
    let expected = [
        BidiClass::RightToLeft,
        BidiClass::EuropeanNumber,
        BidiClass::ArabicLetter,
        BidiClass::OtherNeutral,
        BidiClass::OtherNeutral,
    ];
    for (computed, typography) in [(LTR, upright(LTR)), (RTL, upright(RTL))] {
        let analyzed = AnalyzedSource::new(original.clone(), style(), typography).unwrap();
        zero_levels(&analyzed, 5);
        assert_eq!(analyzed.typography(), upright(computed));
        assert_eq!(analyzed.source(), &original);
        assert_eq!(
            analyzed.source().text().code_units(),
            [0x5d0, 49, 0x628, 40, 41]
        );
        assert_eq!(analyzed.style().direction, Direction::Auto);
        for (i, class) in expected.iter().enumerate() {
            assert_eq!(
                CodePointMapData::<BidiClass>::new()
                    .get32(analyzed.inner.projection.atoms[i].code_point),
                *class
            );
            assert_eq!(analyzed.inner.analysis.infos[i].0.bidi_class, *class);
            assert_eq!(
                analyzed.inner.analysis.infos[i].0.bracket,
                horizontal.inner.analysis.infos[i].0.bracket
            );
            assert_eq!(
                analyzed.inner.analysis.infos[i].0.script,
                horizontal.inner.analysis.infos[i].0.script
            );
        }
        assert_eq!(analyzed.inner.direction.query(0, 5), Some((Some(0), true)));
        assert_eq!(analyzed.identity(), analyzed.clone().identity());
        assert_ne!(analyzed.identity(), horizontal.identity());
        let equal = AnalyzedSource::new(original.clone(), style(), typography).unwrap();
        assert_ne!(analyzed.identity(), equal.identity());
    }
    let old = AnalyzedSource::new(source("א1"), style(), H).unwrap();
    assert_eq!(old.inner.analysis.levels, [1, 2]);
}

#[test]
fn literal_direction_instructions_keep_original_classes_and_invisibility_without_effects() {
    for (open, close) in [
        ('\u{202a}', '\u{202c}'),
        ('\u{202b}', '\u{202c}'),
        ('\u{202d}', '\u{202c}'),
        ('\u{202e}', '\u{202c}'),
        ('\u{2066}', '\u{2069}'),
        ('\u{2067}', '\u{2069}'),
        ('\u{2068}', '\u{2069}'),
    ] {
        let original = source(format!("{open}AB{close}"));
        let analyzed =
            AnalyzedSource::from_paragraph(native(original.clone()), upright(RTL)).unwrap();
        zero_levels(&analyzed, 4);
        assert_eq!(analyzed.source(), &original);
        for i in [0, 3] {
            let info = analyzed.inner.analysis.infos[i].0;
            assert_eq!(
                info.bidi_class,
                CodePointMapData::<BidiClass>::new()
                    .get32(analyzed.inner.projection.atoms[i].code_point)
            );
            assert!(!info.contributes_to_shaping());
        }
        assert!(analyzed.inner.paragraph.controls.is_empty());
        assert_eq!(analyzed.inner.direction.query(0, 4), Some((Some(0), true)));
    }
    let old = AnalyzedSource::from_paragraph(native(source("\u{202e}AB\u{202c}")), H).unwrap();
    assert_eq!(&old.inner.analysis.levels[1..3], [1, 1]);
}

#[test]
fn captured_direction_instructions_keep_source_free_owners_pairing_and_chronology() {
    for (open, close) in [
        (
            ParagraphControlKind::LeftToRightEmbedding,
            ParagraphControlKind::PopDirectionalFormat,
        ),
        (
            ParagraphControlKind::RightToLeftEmbedding,
            ParagraphControlKind::PopDirectionalFormat,
        ),
        (
            ParagraphControlKind::LeftToRightOverride,
            ParagraphControlKind::PopDirectionalFormat,
        ),
        (
            ParagraphControlKind::RightToLeftOverride,
            ParagraphControlKind::PopDirectionalFormat,
        ),
        (
            ParagraphControlKind::LeftToRightIsolate,
            ParagraphControlKind::PopDirectionalIsolate,
        ),
        (
            ParagraphControlKind::RightToLeftIsolate,
            ParagraphControlKind::PopDirectionalIsolate,
        ),
        (
            ParagraphControlKind::FirstStrongIsolate,
            ParagraphControlKind::PopDirectionalIsolate,
        ),
    ] {
        let mut input = native(source("AB"));
        input.controls = vec![
            ParagraphControl {
                at: 0,
                kind: open.clone(),
            },
            ParagraphControl {
                at: 2,
                kind: close.clone(),
            },
        ];
        input.anchor_order = (0..2)
            .map(|i| ParagraphAnchor {
                item: ParagraphAnchorItem::Control(i),
                affinity: Affinity::Before,
            })
            .collect();
        let h = AnalyzedSource::from_paragraph(input.clone(), H).unwrap();
        let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
        zero_levels(&a, 2);
        assert_eq!(a.inner.analysis.control_levels, [0, 0]);
        assert_eq!(a.source().text().code_units(), [65, 66]);
        assert_eq!(a.inner.paragraph.controls[0].kind, open);
        assert_eq!(a.inner.paragraph.controls[1].kind, close);
        let controls: Vec<_> = a
            .inner
            .logical
            .events
            .iter()
            .filter_map(|event| match event.kind {
                Event::Anchor(anchor) => match anchor.item {
                    SemanticAnchorItem::Control(i) => Some(i),
                    _ => None,
                },
                _ => None,
            })
            .collect();
        assert_eq!(controls, [0, 1]);
        assert_eq!(
            a.inner.direction.query(0, a.inner.logical.events.len()),
            Some((Some(0), true))
        );
        if open == ParagraphControlKind::RightToLeftOverride {
            assert_eq!(h.inner.analysis.levels, [1, 1]);
        }
        if open == ParagraphControlKind::RightToLeftIsolate {
            assert_eq!(h.inner.analysis.levels, [2, 2]);
        }
    }
}

#[test]
fn source_bodies_and_terminal_empty_context_survive_strong_l_projection() {
    let a = AnalyzedSource::new(source("A\nא\n"), style(), upright(RTL)).unwrap();
    zero_levels(&a, 4);
    assert_eq!(
        a.inner.analysis.infos[1].0.bidi_class,
        BidiClass::ParagraphSeparator
    );
    assert_eq!(
        a.inner.analysis.infos[3].0.bidi_class,
        BidiClass::ParagraphSeparator
    );
    assert_eq!(a.inner.direction.query(0, 2), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(2, 4), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(4, 4), Some((None, false)));
    assert_eq!(a.inner.direction.query(0, 3), None);
    let empty = AnalyzedSource::new(source(""), style(), upright(RTL)).unwrap();
    zero_levels(&empty, 0);
    assert_eq!(empty.inner.direction.query(0, 0), Some((None, false)));
}

#[test]
fn consecutive_native_forces_retain_distinct_empty_body_and_terminal_custody() {
    let mut input = native(source("Aא"));
    input.controls = [1, 1, 2]
        .into_iter()
        .map(|at| ParagraphControl {
            at,
            kind: ParagraphControlKind::ForcedBreak,
        })
        .collect();
    input.anchor_order = (0..3)
        .map(|i| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(i),
            affinity: Affinity::Before,
        })
        .collect();
    let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
    zero_levels(&a, 2);
    assert_eq!(a.inner.analysis.control_levels, [0, 0, 0]);
    assert_eq!(a.source().text().code_units(), [65, 0x5d0]);
    assert_eq!(a.inner.logical.events.len(), 5);
    assert_eq!(a.inner.direction.query(0, 2), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(2, 3), Some((None, false)));
    assert_eq!(a.inner.direction.query(3, 5), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(5, 5), Some((None, false)));
    assert_eq!(a.inner.direction.query(1, 4), None);
    assert_eq!(text_cuts(&a), [0, 3]);
}

#[test]
fn native_override_crossing_a_force_cannot_merge_bodies_or_affect_either_content() {
    let mut input = native(source("Aא"));
    input.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::RightToLeftOverride,
        },
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::ForcedBreak,
        },
        ParagraphControl {
            at: 2,
            kind: ParagraphControlKind::PopDirectionalFormat,
        },
    ];
    input.anchor_order = (0..3)
        .map(|i| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(i),
            affinity: Affinity::Before,
        })
        .collect();
    let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
    zero_levels(&a, 2);
    assert_eq!(a.inner.analysis.control_levels, [0, 0, 0]);
    assert_eq!(a.inner.direction.query(0, 3), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(3, 5), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(0, 5), None);
}

#[test]
fn contradictory_captured_used_rtl_refuses_at_both_native_fronts_without_relabeling_style() {
    let mut input = native(source("א1"));
    input.base_direction = RTL;
    let old = AnalyzedSource::from_paragraph(input.clone(), H).unwrap();
    assert_eq!(old.direction(), Direction::RightToLeft);
    protocol(
        AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap_err(),
        ProtocolError::MismatchedContext {
            field: ProtocolContextField::BaseDirection,
        },
    );
    let mut capture = contextual(source("א1"));
    capture.paragraphs[0].base_direction = RTL;
    let SourceProcessingScope::Block { used_direction, .. } = &mut capture.scopes[0] else {
        unreachable!()
    };
    *used_direction = RTL;
    let p = Arc::new(process_contextual_source(capture).unwrap());
    let view = p.paragraphs().next().unwrap();
    let old = AnalyzedSource::from_processed_paragraph(view.clone(), H).unwrap();
    assert_eq!(old.direction(), Direction::RightToLeft);
    protocol(
        AnalyzedSource::from_processed_paragraph(view, upright(LTR)).unwrap_err(),
        ProtocolError::MismatchedContext {
            field: ProtocolContextField::BaseDirection,
        },
    );
    let bad_style = Style {
        direction: Direction::RightToLeft,
        ..style()
    };
    let error = AnalyzedSource::new(source("A"), bad_style.clone(), upright(RTL)).unwrap_err();
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedTextStyle {
            feature: crate::TextStyleFeature::ExplicitTextDirection,
            reason: crate::UnsupportedTextStyleReason::RequiresParagraphBaseDirection,
        })
    );
    let mut input = native(source("A"));
    input.base_style = bad_style;
    assert_eq!(
        AnalyzedSource::from_paragraph(input, upright(RTL))
            .unwrap_err()
            .detail(),
        error.detail()
    );
}

#[test]
fn actual_contextual_computed_edges_cut_used_bodies_without_inserting_source_text() {
    use crate::{ComputedLineBoundaryInput, SourceBoxSide};
    use SourceProcessingEvent as E;
    let original = source("Aא");
    let mut input = contextual(original.clone());
    input.origins = vec![
        origin(&original, Range::new(0, 1)),
        origin(&original, Range::new(1, 2)),
    ];
    input.connections = vec![crate::SourceProcessingConnection {
        before_origin: 0,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![crate::ParagraphBoundaryPolicy {
        definition: crate::ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }];
    input.scopes.push(SourceProcessingScope::Inline {
        trim: WhiteSpaceTrim::default(),
        style: None,
        boundaries: None,
    });
    input.computed_line_boundaries = vec![
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::Before,
        },
        ComputedLineBoundaryInput::Scope {
            scope: 1,
            side: SourceBoxSide::After,
        },
    ];
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Connection(0),
        E::EnterScope(1),
        E::Origin {
            origin: 1,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let p = Arc::new(process_contextual_source(input).unwrap());
    let view = p.paragraphs().next().unwrap();
    assert_eq!(
        view.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
    let a = AnalyzedSource::from_processed_paragraph(view, upright(RTL)).unwrap();
    zero_levels(&a, 2);
    assert_eq!(a.typography(), upright(RTL));
    assert_eq!(a.inner.analysis.computed_levels, [0, 0]);
    assert_eq!(a.source(), &original);
    assert!(a.inner.paragraph.controls.is_empty());
    assert_eq!(text_cuts(&a), [0, 2]);
    assert_eq!(a.inner.direction.query(0, 2), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(2, 4), Some((Some(0), true)));
    assert_eq!(a.inner.direction.query(4, 4), Some((None, false)));
    assert_eq!(a.inner.direction.query(0, 3), None);
}

#[test]
fn real_transforms_and_collapsed_space_keep_original_correspondence_before_upright_analysis() {
    let original = source("a  א");
    let mut input = contextual(original.clone());
    let SourceProcessingScope::Block { base_style, .. } = &mut input.scopes[0] else {
        unreachable!()
    };
    base_style.white_space = WhiteSpace::Collapse;
    let requested_transform = TextTransform::Set(
        TextTransformSet::none()
            .with_case(CaseTransform::Uppercase)
            .with_full_width(true),
    );
    base_style.text_transform = requested_transform;
    let p = Arc::new(process_contextual_source(input).unwrap());
    assert_eq!(p.processed_text().code_units(), [97, 32, 0x5d0]);
    assert!(!p.preserves_space_at(1));
    let operations = p.operations();
    assert!(operations.iter().any(|row| matches!(
        row.kind,
        crate::processing::MappingKind::Delete(crate::processing::DeletionCause::CollapsedSpace)
    )));
    let a = AnalyzedSource::from_processed_paragraph(p.paragraphs().next().unwrap(), upright(RTL))
        .unwrap();
    zero_levels(&a, 3);
    assert_eq!(a.inner.projection.text, "Ａ א");
    assert_eq!(a.source(), &original);
    assert_eq!(a.source().text().code_units(), [97, 32, 32, 0x5d0]);
    assert_eq!(a.processed_source().processed_text(), p.processed_text());
    assert_eq!(
        a.graphemes().collect::<Vec<_>>(),
        [
            Range::new(0, 1),
            Range::new(1, 2),
            Range::new(2, 3),
            Range::new(3, 4)
        ]
    );
    // Phase-I deletion is retained by the executed processing receipt;
    // transform input rows retain emitted original pieces separately.
    let original_pieces: Vec<_> = (0..a.inner.transform.rows.len())
        .flat_map(|i| {
            a.inner
                .transform
                .original_pieces(i)
                .iter()
                .map(|p| p.aggregate)
        })
        .collect();
    assert!(original_pieces.contains(&Range::new(0, 1)));
    assert!(operations.iter().any(|row| row.source == Range::new(2, 3)
        && matches!(
            row.kind,
            crate::processing::MappingKind::Delete(
                crate::processing::DeletionCause::CollapsedSpace
            )
        )));
    assert!(original_pieces.contains(&Range::new(3, 4)));
    // Pinned Text3 full-width maps only preserved original SPACE. Equal
    // Phase-I spelling cannot substitute for the executed preservation fact.
    let mut preserved_input = contextual(source("a א"));
    let SourceProcessingScope::Block { base_style, .. } = &mut preserved_input.scopes[0] else {
        unreachable!()
    };
    base_style.text_transform = requested_transform;
    let preserved = Arc::new(process_contextual_source(preserved_input).unwrap());
    assert_eq!(preserved.processed_text(), p.processed_text());
    assert!(preserved.preserves_space_at(1));
    let preserved_analysis = AnalyzedSource::from_processed_paragraph(
        preserved.paragraphs().next().unwrap(),
        upright(RTL),
    )
    .unwrap();
    zero_levels(&preserved_analysis, 3);
    assert_eq!(preserved_analysis.inner.projection.text, "Ａ\u{3000}א");
    assert_eq!(
        preserved_analysis.source().text().code_units(),
        [97, 32, 0x5d0]
    );
    drop(p);
    assert_eq!(a.inner.projection.text, "Ａ א");
}

#[test]
fn continuous_and_disconnected_original_surrogates_do_not_gain_new_correspondence() {
    for (kind, graphemes, points) in [
        (
            ParagraphConnectionKind::Continuous,
            vec![Range::new(0, 2)],
            vec![0x10000],
        ),
        (
            ParagraphConnectionKind::Disconnected,
            vec![Range::new(0, 1), Range::new(1, 2)],
            vec![0xd800, 0xdc00],
        ),
    ] {
        let mut input = native(source(Text::from_code_units(vec![0xd800, 0xdc00])));
        input.origins = vec![
            ParagraphOrigin {
                source: Source::new(Text::from_code_units(vec![0xd800])),
                local: Range::new(0, 1),
                aggregate: Range::new(0, 1),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            },
            ParagraphOrigin {
                source: Source::new(Text::from_code_units(vec![0xdc00])),
                local: Range::new(0, 1),
                aggregate: Range::new(1, 2),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            },
        ];
        input.connections = vec![ParagraphConnection { at: 1, kind }];
        if kind == ParagraphConnectionKind::Continuous {
            input.boundary_policies = vec![crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: Wrap::Word,
                following_wrap: Wrap::Word,
                ancestor_wrap: Wrap::Word,
            }];
        }
        let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
        assert_eq!(a.source().text().code_units(), [0xd800, 0xdc00]);
        assert_eq!(a.graphemes().collect::<Vec<_>>(), graphemes);
        assert_eq!(
            a.inner
                .projection
                .atoms
                .iter()
                .map(|a| a.code_point)
                .collect::<Vec<_>>(),
            points
        );
        zero_levels(&a, points.len());
    }
    let lone = source(Text::from_code_units(vec![0xd800, 0x5d0]));
    let a = AnalyzedSource::new(lone.clone(), style(), upright(RTL)).unwrap();
    zero_levels(&a, 2);
    assert_eq!(a.source(), &lone);
    assert_eq!(a.inner.projection.atoms[0].code_point, 0xd800);
    assert_eq!(a.inner.projection.atoms[0].glyph, '\u{fffd}');
    assert_eq!(
        a.inner.analysis.infos[0].0.bidi_class,
        BidiClass::LeftToRight
    );
}

#[test]
fn wrapping_derivation_preserves_actual_typography_body_and_source_with_new_pair_permissions() {
    let a = AnalyzedSource::from_paragraph(native(source("אA")), upright(RTL)).unwrap();
    let issued: Vec<_> = a.wrapping_relations().map(|r| r.handle()).collect();
    assert!(!issued.is_empty());
    let word = a
        .with_wrapping_policies(
            issued
                .iter()
                .cloned()
                .map(|relation| WrappingPolicy {
                    relation,
                    ancestor_wrap: Wrap::Word,
                })
                .collect(),
        )
        .unwrap();
    let nowrap = a
        .with_wrapping_policies(
            issued
                .iter()
                .cloned()
                .map(|relation| WrappingPolicy {
                    relation,
                    ancestor_wrap: Wrap::None,
                })
                .collect(),
        )
        .unwrap();
    for (derived, expected) in [(&word, Wrap::Word), (&nowrap, Wrap::None)] {
        assert_ne!(derived.identity(), a.identity());
        assert_eq!(derived.typography(), upright(RTL));
        zero_levels(derived, 2);
        assert_eq!(derived.source(), a.source());
        assert_eq!(
            derived.graphemes().collect::<Vec<_>>(),
            [Range::new(0, 1), Range::new(1, 2)]
        );
        assert_eq!(derived.inner.direction.query(0, 2), Some((Some(0), true)));
        assert!(
            derived
                .wrapping_relations()
                .all(|r| r.ancestor_wrap() == Some(expected))
        );
        assert!(
            derived
                .inner
                .analysis
                .infos
                .iter()
                .any(|row| row.0.bidi_class == BidiClass::RightToLeft)
        );
    }
    assert!(a.wrapping_relations().all(|r| r.ancestor_wrap().is_none()));
    let error = word
        .with_wrapping_policies(vec![WrappingPolicy {
            relation: issued[0].clone(),
            ancestor_wrap: Wrap::None,
        }])
        .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::InvalidTextProtocol {
            reason: ProtocolError::ForeignWrappingRelation { .. }
        })
    ));
}

#[test]
fn orientation_mismatches_precede_fonts_and_matching_upright_demands_a_font() {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    assert_eq!(s.registered_font_faces().count(), 0);
    let a = AnalyzedSource::new(source("A"), style(), upright(RTL)).unwrap();
    let identity = a.identity();
    let catalogue = s.font_catalogue_identity().clone();
    let stats = s.stats();
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        protocol(
            s.prepare(
                &a,
                PreparationOptions::default().with_orientation(orientation),
            )
            .unwrap_err(),
            ProtocolError::MismatchedContext {
                field: ProtocolContextField::Orientation,
            },
        );
    }
    // Matching issued upright typography now reaches real font admission.
    // The intentionally empty registered-only catalogue cannot provide a face.
    assert_eq!(
        s.prepare(
            &a,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical)
        )
        .unwrap_err()
        .code,
        ErrorCode::NoUsableFontFace,
    );
    assert_eq!(a.identity(), identity);
    assert_eq!(a.typography(), upright(RTL));
    assert_eq!(s.font_catalogue_identity(), &catalogue);
    assert_eq!(s.stats(), stats);
    let old = AnalyzedSource::new(source("A"), style(), H).unwrap();
    protocol(
        s.prepare(
            &old,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap_err(),
        ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation,
        },
    );
}

#[test]
fn warm_horizontal_layout_cannot_substitute_for_upright_analysis_and_prior_receipts_survive() {
    let mut s = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    s.register_fonts(AHEM.to_vec()).unwrap();
    let original = source("A");
    let h = AnalyzedSource::new(original.clone(), style(), H).unwrap();
    let first = s.layout_analyzed(&h, Options::default()).unwrap();
    let warm = s.layout_analyzed(&h, Options::default()).unwrap();
    assert_eq!(first.key(), warm.key());
    assert_eq!(s.stats().layout_hits(), 1);
    assert_eq!(
        s.prepare(&h, PreparationOptions::default())
            .unwrap()
            .orientation(),
        LineOrientation::Horizontal
    );
    assert_eq!(
        s.prepare(
            &h,
            PreparationOptions::default().with_orientation(LineOrientation::SidewaysClockwise)
        )
        .unwrap()
        .orientation(),
        LineOrientation::SidewaysClockwise
    );
    let a = AnalyzedSource::new(original.clone(), style(), upright(RTL)).unwrap();
    let before = s.stats();
    protocol(
        s.layout_analyzed(&a, Options::default()).unwrap_err(),
        ProtocolError::MismatchedContext {
            field: ProtocolContextField::Orientation,
        },
    );
    assert_eq!(s.stats(), before);
    let again = s.layout_analyzed(&h, Options::default()).unwrap();
    assert_eq!(again.key(), first.key());
    assert_eq!(again.source(), &original);
    drop(s);
    assert_eq!(first.source(), &original);
    assert_eq!(
        first
            .run_views()
            .flat_map(|r| r.glyphs().map(|g| g.id()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [35]
    );
    assert_eq!(a.typography(), upright(RTL));
}

#[test]
fn empty_physical_endpoint_scope_does_not_invent_strong_characters_or_source() {
    let mut input = native(source(""));
    input.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    input.anchor_order = vec![
        ParagraphAnchor {
            item: ParagraphAnchorItem::Control(0),
            affinity: Affinity::After,
        },
        ParagraphAnchor {
            item: ParagraphAnchorItem::EmptyOrigin(0),
            affinity: Affinity::After,
        },
        ParagraphAnchor {
            item: ParagraphAnchorItem::Control(1),
            affinity: Affinity::After,
        },
    ];
    let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
    zero_levels(&a, 0);
    assert!(a.source().text().is_empty());
    assert!(text_cuts(&a).is_empty());
    assert_eq!(a.inner.logical.events.len(), 3);
    assert_eq!(a.inner.direction.query(0, 3), Some((None, false)));
    assert_eq!(a.inner.analysis.control_levels, [0, 0]);
    assert_eq!(a.inner.paragraph.anchor_order.len(), 3);
}

#[test]
fn nearest_block_tab_rtl_is_independent_of_actual_ltr_paragraph_authority() {
    let reference = crate::ParagraphTabReference::new(
        Style {
            tab_size: crate::TabSize::length(24.0).unwrap(),
            ..style()
        },
        RTL,
    )
    .unwrap();
    let mut input = native(source("A\tא"));
    input.tab_reference = Some(reference.clone());
    let a = AnalyzedSource::from_paragraph(input, upright(RTL)).unwrap();
    zero_levels(&a, 3);
    assert_eq!(a.inner.paragraph.base_direction, Some(LTR));
    assert_eq!(a.inner.paragraph.tab_reference, Some(reference));
    assert_eq!(
        a.inner.analysis.infos[1].0.bidi_class,
        BidiClass::SegmentSeparator
    );

    let mut input = contextual(source("A\tא"));
    let SourceProcessingScope::Block {
        used_direction,
        base_style,
        ..
    } = &mut input.scopes[0]
    else {
        unreachable!()
    };
    *used_direction = RTL;
    base_style.tab_size = crate::TabSize::length(24.0).unwrap();
    assert_eq!(input.paragraphs[0].base_direction, LTR);
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let a = AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        upright(RTL),
    )
    .unwrap();
    zero_levels(&a, 3);
    let reference = a.inner.paragraph.tab_reference.as_ref().unwrap();
    assert_eq!(reference.direction(), RTL);
    assert_eq!(
        reference.style().tab_size,
        crate::TabSize::length(24.0).unwrap()
    );
    assert_eq!(a.inner.paragraph.base_direction, Some(LTR));
    assert_eq!(a.source().text().code_units(), [65, 9, 0x5d0]);
    // This operation observes capture only; no hypothetical tab measurement,
    // font reference or Vertical glyph profile is admitted by the test.
}

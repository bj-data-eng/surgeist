//! Actual upright capability refusals before preparation can publish a packet.
use super::*;
use crate::{
    Affinity, Decoration, DecorationKind, DecorationOffset, DecorationThickness, Error, ErrorCode,
    ErrorDetail, FontMetricTable, FontMetricTableError, Id, InlineBox, InlineBoxKind,
    ParagraphAnchor, ParagraphAnchorItem, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphControl, ParagraphControlKind, TextStyleOwner, UprightDecorationRole,
};

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}

fn paragraph(source: Source, base_style: Style) -> ParagraphInput {
    let len = source.text().len();
    ParagraphInput {
        source: source.clone(),
        base_style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    }
}

fn analyze(input: ParagraphInput, typography: AnalysisTypography) -> AnalyzedSource {
    let source = input.source.clone();
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, typography).unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.typography(), typography);
    assert_eq!(analyzed.direction(), Direction::LeftToRight);
    analyzed
}

fn prepare(bytes: &[u8], input: ParagraphInput) -> crate::Result<PreparedText> {
    let analysis = analyze(
        input,
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        },
    );
    system_with(bytes).prepare(
        &analysis,
        PreparationOptions::default().with_orientation(LineOrientation::Vertical),
    )
}

fn decoration(role: UprightDecorationRole) -> Decoration {
    match role {
        UprightDecorationRole::Offset => {
            Decoration::solid().with_thickness(DecorationThickness::try_absolute(1.0).unwrap())
        }
        UprightDecorationRole::Thickness => {
            Decoration::solid().with_offset(DecorationOffset::try_absolute(2.0).unwrap())
        }
        UprightDecorationRole::OffsetAndThickness => Decoration::solid(),
    }
}

fn decorated(kind: DecorationKind, decoration: Decoration) -> Style {
    match kind {
        DecorationKind::Underline => Style {
            underline: decoration,
            ..style()
        },
        DecorationKind::Strikethrough => Style {
            strikethrough: decoration,
            ..style()
        },
    }
}

fn assert_decoration(
    error: Error,
    owner: TextStyleOwner,
    kind: DecorationKind,
    role: UprightDecorationRole,
) {
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UprightDecoration { owner, kind, role })
    );
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn actual_in_flow_and_positioned_atomics_retain_supplied_occurrence_order() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    let mut source = Source::identified("XX", Id::from_u64(1544010), 4);
    source.inline_box(InlineBox::new(
        Id::from_u64(1544011),
        InlineBoxKind::OutOfFlow,
        1,
        crate::Size::new(9.0, 7.0),
    ));
    source.inline_box(InlineBox::new(
        Id::from_u64(1544012),
        InlineBoxKind::InFlow,
        1,
        crate::Size::new(11.0, 13.0),
    ));
    source.inline_box(InlineBox::new(
        Id::from_u64(1544013),
        InlineBoxKind::OutOfFlow,
        1,
        crate::Size::new(13.0, 15.0),
    ));
    assert_eq!(source.boxes()[1].id(), Id::from_u64(1544012));
    assert_eq!(source.boxes()[1].kind(), InlineBoxKind::InFlow);
    assert_eq!(source.boxes()[2].kind(), InlineBoxKind::OutOfFlow);
    let mut input = paragraph(source, style());
    // All three are actual same-seam occurrences. The semantic order is
    // InFlow1, OutOfFlow2, OutOfFlow0, independent of source-box storage order.
    input.anchor_order = [1, 2, 0]
        .into_iter()
        .map(|index| ParagraphAnchor {
            item: ParagraphAnchorItem::Atomic(index),
            affinity: Affinity::After,
        })
        .collect();
    input.boundary_policies = (0..3)
        .flat_map(|index| {
            [
                ParagraphBoundaryDefinition::AtomicBefore(index),
                ParagraphBoundaryDefinition::AtomicAfter(index),
            ]
        })
        .map(|definition| ParagraphBoundaryPolicy {
            definition,
            preceding_wrap: crate::Wrap::Word,
            following_wrap: crate::Wrap::Word,
            ancestor_wrap: crate::Wrap::Word,
        })
        .collect();
    let h_analysis = analyze(input.clone(), AnalysisTypography::HorizontalComposition);
    let horizontal = system_with(&bytes)
        .prepare(&h_analysis, PreparationOptions::default())
        .unwrap();
    assert_eq!(
        horizontal
            .participants()
            .filter_map(|item| match item {
                PreparedParticipant::Atomic(atomic) => Some((
                    atomic.view().original_box().id(),
                    atomic.view().original_box().kind()
                )),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            (Id::from_u64(1544012), InlineBoxKind::InFlow),
            (Id::from_u64(1544013), InlineBoxKind::OutOfFlow),
            (Id::from_u64(1544011), InlineBoxKind::OutOfFlow),
        ]
    );
    let vertical = prepare(&bytes, input)
        .expect("native upright admission must retain genuine mixed-flow atomic occurrences");
    assert_eq!(
        vertical
            .participants()
            .filter_map(|item| match item {
                PreparedParticipant::Atomic(atomic) => Some((
                    atomic.view().original_box().id(),
                    atomic.view().original_box().kind()
                )),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            (Id::from_u64(1544012), InlineBoxKind::InFlow),
            (Id::from_u64(1544013), InlineBoxKind::OutOfFlow),
            (Id::from_u64(1544011), InlineBoxKind::OutOfFlow),
        ]
    );
}

#[test]
fn actual_body_decoration_reports_kind_missing_role_and_exact_original_style_range() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    for kind in [DecorationKind::Underline, DecorationKind::Strikethrough] {
        for role in [
            UprightDecorationRole::Offset,
            UprightDecorationRole::Thickness,
            UprightDecorationRole::OffsetAndThickness,
        ] {
            let mut source = Source::new("XX");
            source.span(Range::new(1, 2), decorated(kind, decoration(role)));
            let error = prepare(&bytes, paragraph(source, style())).expect_err(
                "enabled automatic upright decoration must refuse before packet publication",
            );
            assert_decoration(
                error,
                TextStyleOwner::Source {
                    range: Range::new(1, 2),
                },
                kind,
                role,
            );
        }
    }
}

fn control_input(own: Style) -> ParagraphInput {
    let mut input = paragraph(Source::new("XXXXXX"), style());
    input.controls = vec![
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::Wbr {
                style: Box::new(style()),
            },
        },
        ParagraphControl {
            at: 3,
            kind: ParagraphControlKind::DiscretionaryBreak {
                style: Box::new(Style {
                    hyphenate_character: crate::HyphenateCharacter::String(crate::Text::from("X")),
                    ..own
                }),
            },
        },
    ];
    input.anchor_order = (0..2)
        .map(|index| ParagraphAnchor {
            item: ParagraphAnchorItem::Control(index),
            affinity: Affinity::After,
        })
        .collect();
    input
}

#[test]
fn actual_discretionary_indication_decoration_keeps_its_control_style_occurrence() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    // A genuine control with a real selectable X indication qualifies this
    // style as consumed input; the preceding Wbr makes its occurrence nonzero.
    let positive = prepare(&bytes, control_input(style())).unwrap();
    assert_eq!(
        positive
            .control_views()
            .map(|control| control.source_position().get())
            .collect::<Vec<_>>(),
        [0, 3]
    );
    assert!(
        positive
            .alternative_views()
            .any(|alternative| alternative.text().code_units() == [88])
    );
    for kind in [DecorationKind::Underline, DecorationKind::Strikethrough] {
        for role in [
            UprightDecorationRole::Offset,
            UprightDecorationRole::Thickness,
            UprightDecorationRole::OffsetAndThickness,
        ] {
            let error = prepare(&bytes, control_input(decorated(kind, decoration(role))))
                .expect_err("potentially selectable decorated indication must refuse before packet publication");
            assert_decoration(error, TextStyleOwner::Control { occurrence: 1 }, kind, role);
        }
    }
}

#[test]
fn body_decoration_refuses_before_a_consumed_malformed_nominal_metric_table() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    let malformed = sfnt::replace_tables(&bytes, vec![(*b"vmtx", vec![0])]);
    let face = skrifa::FontRef::new(&malformed).unwrap();
    assert_eq!(face.head().unwrap().units_per_em(), 1000);
    assert_eq!(face.charmap().map('X'), Some(GlyphId::new(58)));
    assert_eq!(face.vhea().unwrap().number_of_long_ver_metrics(), 281);
    assert_eq!(sfnt::table(&malformed, *b"vmtx"), [0]);
    let actual_failure = prepare(&malformed, paragraph(Source::new("X"), style()))
        .expect_err("the same real native glyph path must consume the bad vmtx without decoration");
    assert_eq!(actual_failure.code, ErrorCode::InvalidFontMetrics);
    assert!(matches!(
        actual_failure.detail(),
        Some(ErrorDetail::VerticalMetricSupplier { .. })
    ));
    let original = std::error::Error::source(&actual_failure)
        .unwrap()
        .downcast_ref::<Error>()
        .unwrap();
    assert_eq!(
        original.detail(),
        Some(&ErrorDetail::FontMetricTable {
            table: FontMetricTable::Vmtx,
            reason: FontMetricTableError::Malformed,
        })
    );
    for kind in [DecorationKind::Underline, DecorationKind::Strikethrough] {
        let error = prepare(
            &malformed,
            paragraph(Source::new("X"), decorated(kind, Decoration::solid())),
        )
        .expect_err("automatic decoration admission precedes the demanded bad nominal metric");
        assert_decoration(
            error,
            TextStyleOwner::Source {
                range: Range::new(0, 1),
            },
            kind,
            UprightDecorationRole::OffsetAndThickness,
        );
    }
}

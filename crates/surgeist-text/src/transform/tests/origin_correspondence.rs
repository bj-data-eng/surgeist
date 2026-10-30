//! Characterization of real source correspondence plus functional qualification
//! of the new private key/map seam. Keys are interpreted with their retained
//! owner; they are not foreign-capture capabilities or joint-analysis authority.

use crate::protocol::paragraph::{FlowOriginKey, OriginSource};
use crate::transform::relation::RelationEvent;
use crate::*;
use std::sync::Arc;

fn style(transform: TextTransform) -> Style {
    Style {
        white_space: WhiteSpace::Preserve,
        text_transform: transform,
        ..Style::default()
    }
}

fn original(source: &Source, aggregate: Range, kind: ParagraphOriginKind) -> ParagraphOrigin {
    let local = Range::new(0, source.text().len());
    ParagraphOrigin {
        source: source.clone(),
        local,
        aggregate,
        kind,
        text_node: Some(CompleteTextNodeContribution::new(source.text().clone(), local).unwrap()),
    }
}

fn empty(source: &Source, at: usize) -> ParagraphOrigin {
    ParagraphOrigin {
        source: source.clone(),
        local: Range::new(0, 0),
        aggregate: Range::new(at, at),
        kind: ParagraphOriginKind::Authored,
        text_node: None,
    }
}

fn paragraph() -> SourceProcessingParagraph {
    SourceProcessingParagraph {
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
    }
}

fn policy() -> ParagraphBoundaryPolicy {
    ParagraphBoundaryPolicy {
        definition: ParagraphBoundaryDefinition::Connection(0),
        preceding_wrap: Wrap::Word,
        following_wrap: Wrap::Word,
        ancestor_wrap: Wrap::Word,
    }
}

fn origin_event(origin: usize) -> SourceProcessingEvent {
    SourceProcessingEvent::Origin {
        origin,
        affinity: Affinity::After,
    }
}

fn capture_input(source: Source, base_style: Style) -> ContextualSourceInput {
    ContextualSourceInput {
        scope_alignment_limits: None,
        computed_line_boundaries: vec![],
        source,
        segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
        envelope: SourceProcessingEnvelope::Complete,
        origins: vec![],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        scopes: vec![SourceProcessingScope::Block {
            topology: SourceProcessingTopology::Unknown,
            trim: WhiteSpaceTrim::default(),
            base_style,
            used_direction: ParagraphBaseDirection::LeftToRight,
            writing_axis: None,
            horizontal_inner_direction: None,
            font_reference: None,
        }],
        paragraphs: vec![paragraph(), paragraph()],
        joint_flows: vec![],
        events: vec![],
    }
}

fn late_view(tail: &str, base_style: Style) -> (ContextualSourceInput, Source) {
    let tail_source = Source::new(tail);
    let before_source = Source::new("z");
    let empty_source = Source::new("");
    let mut input = capture_input(Source::new(format!("z{tail}")), base_style);
    // Input IDs are 0=tail, 1=z, 2=empty, while global event order is 1,2,0.
    input.origins = vec![
        original(
            &tail_source,
            Range::new(1, 1 + tail_source.text().len()),
            ParagraphOriginKind::Generated,
        ),
        original(
            &before_source,
            Range::new(0, 1),
            ParagraphOriginKind::Authored,
        ),
        empty(&empty_source, 1),
    ];
    input.controls = vec![ParagraphControl {
        at: 1,
        kind: ParagraphControlKind::ForcedBreak,
    }];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 2,
        after_origin: 0,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![policy()];
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(1),
        origin_event(1),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::EnterParagraph(0),
        origin_event(2),
        SourceProcessingEvent::Control {
            control: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::Connection(0),
        origin_event(0),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    (input, tail_source)
}

fn exact_paragraph(source: &Source, resource: &Source) -> AnalyzedSource {
    let empty_source = Source::new("");
    AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: style(TextTransform::MathAuto),
            terminal_edge: ParagraphTerminalEdge::BlockEnd,
            paragraph_strut: ParagraphStrut::DefaultFont,
            tab_reference: None,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![
                empty(&empty_source, 0),
                empty(&empty_source, 0),
                original(resource, Range::new(0, 1), ParagraphOriginKind::Generated),
            ],
            connections: vec![
                ParagraphConnection {
                    at: 0,
                    kind: ParagraphConnectionKind::Continuous,
                },
                ParagraphConnection {
                    at: 0,
                    kind: ParagraphConnectionKind::Continuous,
                },
            ],
            controls: vec![],
            boundary_policies: (0..2)
                .map(|index| ParagraphBoundaryPolicy {
                    definition: ParagraphBoundaryDefinition::Connection(index),
                    ..policy()
                })
                .collect(),
            replacements: vec![],
            anchor_order: vec![
                ParagraphAnchor {
                    item: ParagraphAnchorItem::EmptyOrigin(0),
                    affinity: Affinity::After,
                },
                ParagraphAnchor {
                    item: ParagraphAnchorItem::EmptyOrigin(1),
                    affinity: Affinity::After,
                },
            ],
        },
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap()
}

fn text<'a>(items: impl Iterator<Item = PlaintextItem<'a>>) -> Vec<u16> {
    items
        .flat_map(|item| match item {
            PlaintextItem::Text(text) => text.code_units().to_vec(),
            PlaintextItem::InlineBox(_) => vec![],
        })
        .collect()
}

#[test]
fn noncaptured_maps_preserve_canonical_and_distinct_original_source_resources() {
    let source = Source::identified("a", Id::from_u64(78410), 3);
    let canonical = AnalyzedSource::new(
        source.clone(),
        style(TextTransform::none()),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let resource = Source::identified("a", Id::from_u64(78411), 5);
    assert_ne!(
        source, resource,
        "equal text does not erase supplied original Source identity"
    );
    let captured_resource = exact_paragraph(&source, &resource);
    assert_eq!(canonical.inner.projection.text, "a");
    assert_eq!(captured_resource.inner.projection.text, "\u{1d44e}");
    for (analyzed, index) in [(&canonical, 0), (&captured_resource, 2)] {
        let paragraph = &analyzed.inner.paragraph;
        let view = paragraph.origin_map.view(index).unwrap();
        let key = paragraph.origin_map.flow_origin(view);
        assert_eq!(view.get(), index);
        assert!(matches!(key, FlowOriginKey::ExactOrdinal(actual) if actual == index));
        assert_eq!(key.captured_event(), None);
        assert_eq!(paragraph.origin_map.view_origin(key), Some(view));
        assert_eq!(
            paragraph
                .origin_map
                .visit_view_origin(key, || Ok(()))
                .unwrap(),
            Some(view)
        );
        assert_eq!(paragraph.origin_map.view(index + 1), None);
        let piece = analyzed.inner.transform.original_pieces(0)[0];
        assert_eq!(piece.origin, key);
        assert_eq!(piece.aggregate, Range::new(0, 1));
        assert_eq!(piece.local, Range::new(0, 1));
        assert_eq!(analyzed.source(), &source);
        assert_eq!(analyzed.processed_source().processed_text(), source.text());
    }
    let canonical_origin = &canonical.inner.paragraph.origins[0];
    let resource_origin = &captured_resource.inner.paragraph.origins[2];
    assert!(matches!(canonical_origin.source, OriginSource::Canonical));
    assert_eq!(
        canonical_origin.original(canonical.processed_source()),
        &source
    );
    assert!(matches!(resource_origin.source, OriginSource::Captured(_)));
    assert_eq!(
        resource_origin.original(captured_resource.processed_source()),
        &resource
    );
    assert_eq!(resource_origin.kind, ParagraphOriginKind::Generated);
    assert!(resource_origin.text_node.is_some());
    assert_eq!(
        captured_resource.inner.projection.atoms[0].presentation,
        Range::new(0, 2)
    );
}

#[test]
fn later_raw_view_maps_empty_and_generated_origins_to_real_full_capture_events() {
    let (input, resource) = late_view("a", style(TextTransform::MathAuto));
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(raw.len(), 2);
    assert_eq!(raw[0].source_range(), Range::new(0, 1));
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(1, 1), Range::new(1, 2)]
    );
    let early = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let later = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let map = &later.inner.paragraph.origin_map;
    let empty_view = map.view(0).unwrap();
    let emitted_view = map.view(1).unwrap();
    let empty_key = map.flow_origin(empty_view);
    let emitted_key = map.flow_origin(emitted_view);
    assert_eq!(empty_key.captured_event(), Some(5));
    assert_eq!(emitted_key.captured_event(), Some(8));
    assert_eq!(
        empty_key.captured_event(),
        Some(processed.captured_origin(2).unwrap().index)
    );
    assert_eq!(
        emitted_key.captured_event(),
        Some(processed.captured_origin(0).unwrap().index)
    );
    assert_eq!(map.view_origin(empty_key), Some(empty_view));
    assert_eq!(map.view_origin(emitted_key), Some(emitted_view));
    assert_eq!(map.view(2), None);
    let early_map = &early.inner.paragraph.origin_map;
    let early_key = early_map.flow_origin(early_map.view(0).unwrap());
    assert_eq!(early_key.captured_event(), Some(2));
    assert_eq!(map.view_origin(early_key), None);
    assert_eq!(early_map.view_origin(emitted_key), None);
    let exact_source = Source::new("a");
    let exact = exact_paragraph(&exact_source, &resource);
    let exact_map = &exact.inner.paragraph.origin_map;
    let exact_key = exact_map.flow_origin(exact_map.view(2).unwrap());
    assert!(matches!(exact_key, FlowOriginKey::ExactOrdinal(2)));
    // CapturedEvent(2) and ExactOrdinal(2) are both real emitted-original
    // keys here. Their equal numeric payload does not authorize kind erasure.
    assert_eq!(exact_map.view_origin(early_key), None);
    assert_eq!(early_map.view_origin(exact_key), None);
    assert_eq!(
        map.view_origin(exact_key),
        None,
        "wrong key kind is not a local ordinal alias"
    );
    assert_eq!(exact_map.view_origin(emitted_key), None);

    assert_eq!(later.inner.projection.text, "\u{1d44e}");
    let piece = later.inner.transform.original_pieces(0)[0];
    assert_eq!(piece.origin, emitted_key);
    assert_eq!(piece.aggregate, Range::new(1, 2));
    assert_eq!(piece.local, Range::new(0, 1));
    let metadata = &later.inner.paragraph.origins[map.view_origin(piece.origin).unwrap().get()];
    assert_eq!(metadata.original(later.processed_source()), &resource);
    assert_eq!(metadata.kind, ParagraphOriginKind::Generated);
    assert!(metadata.text_node.is_some());
    assert_eq!(later.inner.paragraph.controls.len(), 1);
    assert!(matches!(
        later.inner.paragraph.controls[0].kind,
        ParagraphControlKind::ForcedBreak
    ));
    assert_eq!(later.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("za"));
    assert_eq!(text(raw[1].plaintext_items(Affinity::After)), [0x61]);
}

#[test]
fn real_child_domain_origin_stays_outside_parent_view_correspondence() {
    let a = Source::new("a");
    let q = Source::new("q");
    let z = Source::new("z");
    let mut source = Source::new("zqa");
    source.inline_box(InlineBox::new(
        Id::from_u64(7841),
        InlineBoxKind::OutOfFlow,
        1,
        Size::new(0.0, 0.0),
    ));
    let mut input = capture_input(source.clone(), style(TextTransform::MathAuto));
    input.origins = vec![
        original(&q, Range::new(1, 2), ParagraphOriginKind::Authored),
        original(&a, Range::new(2, 3), ParagraphOriginKind::Generated),
        original(&z, Range::new(0, 1), ParagraphOriginKind::Authored),
    ];
    input.connections = vec![SourceProcessingConnection {
        before_origin: 2,
        after_origin: 1,
        kind: ParagraphConnectionKind::Continuous,
    }];
    input.boundary_policies = vec![policy()];
    input.scopes.push(SourceProcessingScope::OutOfFlowBlock {
        topology: SourceProcessingTopology::Unknown,
        trim: WhiteSpaceTrim::default(),
        base_style: style(TextTransform::none()),
        used_direction: ParagraphBaseDirection::LeftToRight,
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        atomic: 0,
    });
    input.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
        origin_event(2),
        SourceProcessingEvent::Atomic {
            atomic: 0,
            affinity: Affinity::Before,
        },
        SourceProcessingEvent::EnterScope(1),
        SourceProcessingEvent::EnterParagraph(1),
        origin_event(0),
        SourceProcessingEvent::ExitParagraph(1),
        SourceProcessingEvent::ExitScope(1),
        SourceProcessingEvent::Connection(0),
        origin_event(1),
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ];
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let raw = processed.paragraphs().collect::<Vec<_>>();
    assert_eq!(
        raw[0].source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(2, 3)]
    );
    assert_eq!(
        raw[1].source_pieces().collect::<Vec<_>>(),
        [Range::new(1, 2)]
    );
    let parent = AnalyzedSource::from_processed_paragraph(
        raw[0].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let child = AnalyzedSource::from_processed_paragraph(
        raw[1].clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let parent_map = &parent.inner.paragraph.origin_map;
    let child_map = &child.inner.paragraph.origin_map;
    let child_key = child_map.flow_origin(child_map.view(0).unwrap());
    let a_key = parent_map.flow_origin(parent_map.view(1).unwrap());
    assert_eq!(child_key.captured_event(), Some(6));
    assert_eq!(a_key.captured_event(), Some(10));
    assert_eq!(parent_map.view_origin(child_key), None);
    assert_eq!(child_map.view_origin(a_key), None);
    assert_eq!(parent.inner.projection.text, "\u{1d467}\u{1d44e}");
    assert_eq!(child.inner.projection.text, "q");
    assert_eq!(parent.inner.transform.original_pieces(1)[0].origin, a_key);
    assert_eq!(
        parent.inner.transform.original_pieces(1)[0].aggregate,
        Range::new(2, 3)
    );
    assert_eq!(
        child.inner.transform.original_pieces(0)[0].origin,
        child_key
    );
    assert_eq!(
        child.inner.paragraph.origins[child_map.view_origin(child_key).unwrap().get()]
            .original(child.processed_source()),
        &q
    );
    assert_eq!(parent.source(), &source);
    assert_eq!(processed.processed_text(), &Text::from("zqa"));
    assert_eq!(text(raw[0].plaintext_items(Affinity::After)), [0x7a, 0x61]);
    assert_eq!(text(raw[1].plaintext_items(Affinity::After)), [0x71]);
}

#[test]
fn delayed_greek_outputs_keep_one_typed_origin_and_exact_noncontiguous_local_pieces() {
    let uppercase = style(TextTransform::Set(
        TextTransformSet::none().with_case(CaseTransform::Uppercase),
    ));
    let (mut input, resource) = late_view("\u{1f80}\u{301}\u{313}", uppercase.clone());
    input.source.span(
        Range::new(3, 4),
        Style {
            locale: Some("el".to_owned()),
            ..uppercase
        },
    );
    let source = input.source.clone();
    let processed = Arc::new(process_contextual_source(input).unwrap());
    let raw = processed.paragraphs().nth(1).unwrap();
    let analyzed = AnalyzedSource::from_processed_paragraph(
        raw.clone(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    assert_eq!(analyzed.inner.projection.text, "\u{1f08}\u{301}Ι");
    let origins = &analyzed.inner.paragraph.origin_map;
    let key = origins.flow_origin(origins.view(1).unwrap());
    assert_eq!(key.captured_event(), Some(8));
    let map = &analyzed.inner.transform;
    for row in 0..3 {
        let piece = map.original_pieces(row)[0];
        assert_eq!(piece.origin, key);
        assert_eq!(piece.aggregate, Range::new(row + 1, row + 2));
        assert_eq!(piece.local, Range::new(row, row + 1));
    }
    assert_eq!(
        map.atoms
            .iter()
            .map(|&id| {
                let emission = map.emissions[id];
                let row = map.original_row_for_atom(emission.atom).unwrap();
                (
                    row,
                    emission.subposition,
                    map.original_pieces(row)[0].origin,
                )
            })
            .collect::<Vec<_>>(),
        [(0, 0, key), (1, 0, key), (0, 1, key)]
    );
    assert!(matches!(
        map.events(),
        [
            RelationEvent::Anchor { .. },
            RelationEvent::Anchor { .. },
            RelationEvent::Text { atom: 0 },
            RelationEvent::Text { atom: 1 },
            RelationEvent::Removal { row: 2, .. },
            RelationEvent::Text { atom: 2 },
        ]
    ));
    let metadata = &analyzed.inner.paragraph.origins[origins.view_origin(key).unwrap().get()];
    assert_eq!(metadata.original(analyzed.processed_source()), &resource);
    assert_eq!(analyzed.source(), &source);
    assert_eq!(processed.processed_text(), source.text());
    assert_eq!(
        text(raw.plaintext_items(Affinity::After)),
        "\u{1f80}\u{301}\u{313}".encode_utf16().collect::<Vec<_>>()
    );
}

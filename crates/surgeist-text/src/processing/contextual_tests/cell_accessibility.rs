//! Genuine unexpanded Wbr anchors within one native Cell and AX word.
#![cfg(feature = "text-accessibility")]

use super::*;
use crate::protocol::prepared::cell_sources::{
    CellCorrespondence, CellPartSource, CellSourceVisit,
};
use accesskit::{NodeId, Role, TextPosition, TreeUpdate};

const COUNT: usize = 32;
const PARENT: NodeId = NodeId(1);

fn ax_originals(update: &TreeUpdate, original: &str) -> Vec<(TextPosition, TextPosition)> {
    let (_, parent) = update.nodes.iter().find(|(id, _)| *id == PARENT).unwrap();
    assert_eq!(parent.role(), Role::Paragraph);
    let mut text = String::new();
    let mut units = Vec::new();
    for id in parent.children() {
        let (_, node) = update
            .nodes
            .iter()
            .find(|(candidate, _)| candidate == id)
            .unwrap();
        assert_eq!(node.role(), Role::TextRun);
        let value = node.value().unwrap();
        text.push_str(value);
        assert!(node.character_lengths().iter().all(|&length| length == 1));
        assert_eq!(node.character_lengths().len(), value.len());
        // An implicit paragraph/document start0 is sufficient. Every later A
        // continues the same actual Cell word, despite intervening Wbr anchors.
        for &start in node.word_starts() {
            assert!(usize::from(start) < node.character_lengths().len());
            assert_eq!(units.len() + usize::from(start), 0);
        }
        assert_eq!(node.character_positions(), None);
        assert_eq!(node.character_widths(), None);
        for character_index in 0..node.character_lengths().len() {
            units.push((
                TextPosition {
                    node: *id,
                    character_index,
                },
                TextPosition {
                    node: *id,
                    character_index: character_index + 1,
                },
            ));
        }
    }
    assert_eq!(text, original, "unexpanded Wbr adds no original AX value");
    assert_eq!(units.len(), COUNT);
    units
}

#[test]
fn many_genuine_unexpanded_wbr_controls_remain_inside_one_native_all_cell_and_its_ax_word() {
    let text = "A".repeat(COUNT);
    let mut value = super::base(&text, TextCombineUpright::All);
    let own = match &value.scopes[0] {
        SourceProcessingScope::Block { base_style, .. } => base_style.clone(),
        _ => unreachable!("the existing base owns the actual root Block"),
    };
    value.origins = (0..COUNT)
        .map(|index| super::super::origin(&value.source, Range::new(index, index + 1)))
        .collect();
    value.connections = (0..COUNT - 1)
        .map(|before| SourceProcessingConnection {
            before_origin: before,
            after_origin: before + 1,
            kind: ParagraphConnectionKind::Continuous,
        })
        .collect();
    value.boundary_policies = (0..COUNT - 1)
        .map(|connection| super::super::policy(ParagraphBoundaryDefinition::Connection(connection)))
        .collect();
    value.controls = (1..COUNT)
        .map(|at| ParagraphControl {
            at,
            kind: ParagraphControlKind::Wbr {
                style: Box::new(Style {
                    word_space_transform: WordSpaceTransform::None,
                    ..own.clone()
                }),
            },
        })
        .collect();
    // Only actual root/paragraph/origin/connection/control events. In particular
    // no Inline scope, endpoint, Atomic or fake box is inserted around any Wbr.
    value.events = vec![
        SourceProcessingEvent::EnterScope(0),
        SourceProcessingEvent::EnterParagraph(0),
    ];
    for origin in 0..COUNT {
        if origin > 0 {
            value
                .events
                .push(SourceProcessingEvent::Connection(origin - 1));
            value.events.push(super::event(origin - 1));
        }
        value.events.push(SourceProcessingEvent::Origin {
            origin,
            affinity: Affinity::After,
        });
    }
    value.events.extend([
        SourceProcessingEvent::ExitParagraph(0),
        SourceProcessingEvent::ExitScope(0),
    ]);
    let (processed, analyzed) = super::analyze(value);
    assert_eq!(processed.source().text(), &Text::from(text.as_str()));
    assert_eq!(processed.processed_text(), &Text::from(text.as_str()));
    assert!(processed.source().boxes().is_empty());
    assert!(processed.captured_scope(1).is_none());
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    let [source_cell] = cells.as_slice() else {
        panic!("one uninterrupted genuine root-All Cell")
    };
    assert_eq!(source_cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(source_cell.mode(), TextCombineUpright::All);
    assert_eq!(
        (
            source_cell.selection_unit_count(),
            source_cell.used_unit_count()
        ),
        (COUNT, COUNT)
    );
    assert_eq!(
        source_cell.used_code_points().collect::<Vec<_>>(),
        vec![0x41; COUNT]
    );
    assert_eq!(
        source_cell.presentation_code_points().collect::<Vec<_>>(),
        vec![0x41; COUNT]
    );
    let pieces = source_cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), COUNT);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert!(piece.is_used() && piece.has_presentation_emission());
    }
    for index in 0..COUNT - 1 {
        let control = processed.captured_control(index).unwrap();
        assert_eq!(control.source_position().get(), index + 1);
        assert_eq!(control.scope(), processed.captured_scope(0).unwrap());
        assert_eq!(
            control.paragraph(),
            processed.captured_paragraph(0).unwrap()
        );
        assert!(processed.captured_origin(index).unwrap() < control.event_position());
        assert!(control.event_position() < processed.captured_origin(index + 1).unwrap());
        let ParagraphControlKind::Wbr { style } = control.kind() else {
            panic!("real captured Wbr")
        };
        assert_eq!(style.word_space_transform, WordSpaceTransform::None);
    }
    assert!(processed.captured_control(COUNT - 1).is_none());

    let (_system, prepared, bytes) = super::prepare(&analyzed);
    assert_eq!(prepared.orientation(), ORIENTATION);
    assert_eq!(prepared.cell_views().len(), 1);
    let prepared_cell = prepared.cell_views().next().unwrap();
    assert_eq!(prepared_cell.source_cell().scope(), source_cell.scope());
    assert_eq!(prepared_cell.extent(), 10.0);
    assert_eq!(
        prepared.participants().collect::<Vec<_>>(),
        [PreparedParticipant::Cell(prepared_cell.handle())]
    );
    let segment = {
        let mut segments = prepared.source_segments();
        let segment = segments.next().unwrap();
        assert!(segments.next().is_none());
        segment
    };
    // Parent finish is specifically for allocated exterior inline endpoints.
    // Interior Wbr controls require no invented allocation/geometry, so use the
    // real issued Cell participant directly in the ordinary selection protocol.
    let line = prepared
        .select(LineSelectionRequest {
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants: prepared.participants().collect(),
            termination: PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: LineContext {
                formatted_line: FormattedLineContext::default(),
                base_direction: ParagraphBaseDirection::LeftToRight,
                line_box_direction: LineBoxDirection::LeftToRight,
                orientation: ORIENTATION,
                start: LineStart::Paragraph,
                end: LineEnd::Paragraph,
                available_inline: None,
                alignment_insets: LineAlignmentInsets::ZERO,
                indent: 0.0,
                alignment: Alignment::Start,
                tabs: TabContext::NoTabs,
                spacing: SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(line.cell_views().len(), 1);
    assert_eq!(line.feedback().advance, 10.0);
    let layout = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: ORIENTATION,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), processed.source());
    let raw = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(raw, vec![(35, 10.0); COUNT]);
    assert_eq!(layout.positions().count(), 2);

    // Observe the natural shared correspondence of this real selected Cell.
    // No hand-authored map/occurrence or comparison/index implementation oracle.
    let native = &layout.inner.lines[0].finalized;
    let [cell] = native.data.cells.as_slice() else {
        panic!("one actual finalized Cell")
    };
    let selected = &native.selected;
    let mut observation = None; // Explicit Native observation owner, as in AX.
    let correspondence = CellCorrespondence::new(
        &selected.prepared,
        &selected.presentation,
        &selected.request.segment,
        cell.unit,
        &mut observation,
    )
    .unwrap();
    let mut originals = Vec::new();
    let mut controls = Vec::new();
    correspondence
        .visit_sources(&mut observation, |source, _| {
            match source {
                CellSourceVisit::Part(part) => {
                    assert!(matches!(part.source(), CellPartSource::Original));
                    let [piece] = part.part().pieces.as_slice() else {
                        panic!("one actual A origin")
                    };
                    originals.push(piece.aggregate);
                }
                CellSourceVisit::Control(occurrence) => {
                    let fact = &selected.prepared.analyzed.paragraph.controls[occurrence.control()];
                    let ParagraphControlKind::Wbr { style } = &fact.kind else {
                        panic!("retained Wbr anchor")
                    };
                    assert_eq!(style.word_space_transform, WordSpaceTransform::None);
                    controls.push(fact.at);
                }
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(
        originals,
        (0..COUNT)
            .map(|index| Range::new(index, index + 1))
            .collect::<Vec<_>>()
    );
    assert_eq!(controls, (1..COUNT).collect::<Vec<_>>());
    correspondence.drop(&mut observation).unwrap();

    let mut state = Accessibility::new(94_000);
    let source_update = analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let source_units = ax_originals(&source_update, &text);
    let placed_update = layout
        .accessibility_update(&mut state, PARENT, PARENT, Point::new(0.0, 0.0))
        .unwrap();
    let placed_units = ax_originals(&placed_update, &text);
    assert_eq!(placed_units, source_units);
    let middle = COUNT / 2;
    assert_eq!(
        layout
            .access_position(&state, Cursor::new(middle, Affinity::After))
            .unwrap(),
        Some(placed_units[middle].0)
    );
}

#[test]
fn genuine_case_deleted_original_member_refuses_cell_mapping_without_publishing_ids_or_binding() {
    let original = "i\u{307}";
    let mut value = super::base(original, TextCombineUpright::All);
    let SourceProcessingScope::Block { base_style, .. } = &mut value.scopes[0] else {
        unreachable!("actual Complete root Block");
    };
    base_style.locale = Some("lt".to_owned());
    base_style.text_transform =
        TextTransform::Set(TextTransformSet::none().with_case(CaseTransform::Uppercase));
    let own = base_style.clone();
    value.source.span(Range::new(0, 2), own);
    value.origins = vec![super::super::origin(&value.source, Range::new(0, 2))];
    let (processed, analyzed) = super::analyze(value);
    assert_eq!(processed.source().text(), &Text::from(original));
    assert_eq!(analyzed.inner.host_projection.graphemes, [0, 2]);
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one real All I Cell after Lithuanian casing");
    };
    assert_eq!(cell.scope(), processed.captured_scope(0).unwrap());
    assert_eq!(cell.presentation_code_points().collect::<Vec<_>>(), [0x49]);
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    let [piece] = pieces.as_slice() else {
        panic!("only i emitted an actual Cell part");
    };
    assert_eq!(piece.source(), processed.source());
    assert_eq!(piece.aggregate(), Range::new(0, 1));
    assert_eq!(
        piece.origin_position(),
        processed.captured_origin(0).unwrap()
    );
    // Lithuanian uppercasing really deletes dot-above after soft-dotted i.
    // The original host EGC0..2 cannot be assigned wholly to Cell ancestry0..1.
    let previous = AnalyzedSource::new(
        Source::new("safe"),
        Style::default(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let probe = AnalyzedSource::new(
        Source::new("next"),
        Style::default(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut state = Accessibility::new(95_000);
    previous
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let before = previous
        .access_position(&state, Cursor::new(0, Affinity::After))
        .unwrap();
    assert!(before.is_some());
    let mut expected = state.clone();
    let error = analyzed
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::UnsupportedAccessibilityCellMapping {
            cell: 0,
            range: Range::new(0, 2),
        })
    );
    assert_eq!(
        previous
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap(),
        before
    );
    let actual = probe
        .accessibility_update(&mut state, PARENT, PARENT)
        .unwrap();
    let expected = probe
        .accessibility_update(&mut expected, PARENT, PARENT)
        .unwrap();
    assert_eq!(
        actual
            .nodes
            .iter()
            .map(|(id, node)| (*id, node.role(), node.value()))
            .collect::<Vec<_>>(),
        expected
            .nodes
            .iter()
            .map(|(id, node)| (*id, node.role(), node.value()))
            .collect::<Vec<_>>()
    );
}

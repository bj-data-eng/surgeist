//! Transactional float trials over real native nodes and Text-issued receipts.

use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use crate::*;
use surgeist_text as text;

fn selected_line(
    prepared: &text::PreparedText,
    start: text::PreparedBoundary,
    end: text::PreparedOpportunity,
) -> FinalizedLine {
    let from = start.view().participant_offset();
    let boundary = end.view().boundary();
    let to = boundary.view().participant_offset();
    let participants: Vec<_> = prepared.participants().skip(from).take(to - from).collect();
    let controls = participants
        .iter()
        .filter_map(|participant| match participant {
            PreparedParticipant::Control(occurrence) => Some(text::ControlAllocation {
                occurrence: occurrence.clone(),
                boundary: None,
                strut: Some(text::ControlStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                    alignment: text::LineBoxAlignment::Baseline,
                }),
            }),
            _ => None,
        })
        .collect();
    prepared
        .select(text::LineSelectionRequest {
            start,
            end: boundary,
            participants,
            termination: text::PreparedTermination::Ordinary(end.clone()),
            alternative: None,
            incoming: None,
            context: text::LineContext {
                alignment_insets: surgeist_text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: if from == 0 {
                    text::LineStart::Paragraph
                } else {
                    text::LineStart::Continuation
                },
                end: if end.view().kind() == text::PreparedBreakKind::Forced {
                    LineEnd::Forced
                } else {
                    LineEnd::Paragraph
                },
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
            atomics: Vec::new(),
            controls,
            boundary_context: Vec::new(),
            line_strut: None,
            line_box_struts: Vec::new(),
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap()
}

fn replay_at(block: f64) -> EncounterReplay {
    EncounterReplay {
        boxes: None,
        preceding: None,
        source_floor: block,
        logical: block,
        placed: block,
        defer_preceding: false,
    }
}

fn lane<S: LayoutScalar>() {
    // Both floats are actual native children at original gaps. They are not
    // Text atomics, invented participants or manually constructed packets.
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, text::Source::new("A"), style.clone()),
            ParagraphPart::Control {
                node: 3,
                kind: text::ParagraphControlKind::ForcedBreak,
            },
            ParagraphPart::text(4, text::Source::new("B "), style.clone()),
            ParagraphPart::text(6, text::Source::new("C"), style.clone()),
        ],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let mut binding = paragraph.binding(0);
    binding.nodes = vec![1, 2, 3, 4, 5, 6];
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, binding.nodes.iter().copied())
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        )
        .input(1, paragraph.text_input(1))
        .input(4, paragraph.text_input(4))
        .input(6, paragraph.text_input(6))
        .input(
            3,
            LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(
                InlineMetricsOf::try_new(S::from_f64(8.0), S::from_f64(10.0)).unwrap(),
            )),
        );
    for (node, extent) in [(2, 10.0), (5, 20.0)] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(extent)),
                    PreferredSizeOf::px(S::from_f64(extent)),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    let capture =
        LayoutFlowInput::try_new(&tree, 0, vec![binding.clone()], WrappingInput::default())
            .unwrap();
    let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    let child_request = ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::new(Some(S::from_f64(100.0)), None),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        Size::new(
            AvailableOf::definite(S::from_f64(100.0)),
            AvailableOf::MAX_CONTENT,
        ),
    );
    let mut packets = Vec::new();
    let mut events = Vec::new();
    for (node, offset, extent) in [(2, 1, 10.0), (5, 4, 20.0)] {
        let packet = owner.probe_float_child(node, child_request).unwrap();
        assert_eq!(packet.output.size, Size::splat(S::from_f64(extent)));
        let pending = PendingFloat {
            placement: super::super::floats::PendingPlacement::Unresolved,
            node,
            source_index: offset,
            side: Float::Left,
            clear: Clear::None,
            block_start: S::ZERO,
            size: packet.output.size,
            content_size: packet.output.content_size,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            float_exclusion: FloatExclusion::MarginBox,
            child_compute_geometry: packet.output.scroll_geometry,
        };
        events.push(
            FloatEvent::new(
                pending,
                packet.clone(),
                &binding,
                &capture.associations,
                offset,
                child_request,
            )
            .unwrap_or_else(|_| panic!("actual native float event")),
        );
        packets.push(packet);
    }
    let forced = paragraph
        .prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::Forced)
        .unwrap()
        .handle();
    let terminal = paragraph
        .prepared
        .opportunity_views()
        .find(|offer| offer.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let initial = paragraph.prepared.boundary_views().next().unwrap().handle();
    let first = selected_line(&paragraph.prepared, initial, forced.clone());
    let second = selected_line(&paragraph.prepared, forced.view().boundary(), terminal);
    assert_eq!(
        (first.feedback().line_height, second.feedback().line_height),
        (10.0, 10.0)
    );
    let base = FloatExclusions::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::splat(S::from_f64(100.0)),
        S::from_f64(100.0),
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    let mut encounters =
        Encounters::new(events, base, S::ZERO, FloatInlineSpace::Finite, true, true);
    let width = |encounters: &Encounters<u32, S>, block: f64| {
        let band = encounters
            .band()
            .query_rectangular_line_band(S::from_f64(block), S::from_f64(block + 5.0));
        band.inline_end - band.inline_start
    };
    encounters.checkpoint_group();
    let first_basis = encounters
        .replay(&owner, 0, &first, replay_at(0.0), false)
        .unwrap();
    assert_eq!(
        first_basis.iter().map(|row| row.event).collect::<Vec<_>>(),
        [0]
    );
    assert_eq!(
        encounters.replay_event_visits, 1,
        "the later group's event is not scanned"
    );
    encounters.accept();
    assert_eq!(width(&encounters, 0.0), S::from_f64(90.0));
    encounters.checkpoint_group();
    let prefix = encounters.accepted[0]
        .as_ref()
        .unwrap()
        .placement
        .text()
        .cloned()
        .unwrap();

    // A candidate and then an accepted losing trial both use the actual second
    // float packet, at the trial's supplied source-line floor30. Neither may
    // survive restoration to the real group entry at block10.
    let visits = encounters.replay_event_visits;
    let losing = encounters
        .replay(&owner, 0, &second, replay_at(30.0), false)
        .unwrap();
    assert_eq!(losing.iter().map(|row| row.event).collect::<Vec<_>>(), [1]);
    assert_eq!(
        encounters.replay_event_visits - visits,
        1,
        "accepted earlier events are not rescanned"
    );
    assert_eq!(width(&encounters, 30.0), S::from_f64(80.0));
    encounters.reset_candidate();
    assert!(encounters.accepted[1].is_none());
    assert!(encounters.tentative[1].is_none());
    assert_eq!(width(&encounters, 30.0), S::from_f64(100.0));
    encounters
        .replay(&owner, 0, &second, replay_at(30.0), false)
        .unwrap();
    encounters.accept();
    assert_eq!(
        encounters.accepted[1]
            .as_ref()
            .unwrap()
            .placement
            .text()
            .unwrap()
            .logical_origin
            .block,
        S::from_f64(30.0)
    );
    encounters.restart_group();
    assert!(encounters.accepted[1].is_none());
    let retained = encounters.accepted[0]
        .as_ref()
        .unwrap()
        .placement
        .text()
        .unwrap();
    assert_eq!(retained.logical_origin, prefix.logical_origin);
    assert_eq!(retained.line, prefix.line);
    assert_eq!(width(&encounters, 0.0), S::from_f64(90.0));
    assert_eq!(width(&encounters, 30.0), S::from_f64(100.0));
    let winner = encounters
        .replay(&owner, 0, &second, replay_at(10.0), false)
        .unwrap();
    encounters.accept();
    assert_eq!(
        encounters.accepted[1]
            .as_ref()
            .unwrap()
            .placement
            .text()
            .unwrap()
            .logical_origin
            .block,
        S::from_f64(10.0)
    );

    // The final whole replay starts from the true incoming ledger and produces
    // the same selected encounter basis; discarded trials publish no packets.
    encounters.restart_sequence();
    assert!(encounters.accepted.iter().all(Option::is_none));
    assert_eq!(width(&encounters, 0.0), S::from_f64(100.0));
    assert_eq!(
        encounters
            .replay(&owner, 0, &first, replay_at(0.0), false)
            .unwrap(),
        first_basis
    );
    encounters.accept();
    assert_eq!(
        encounters
            .replay(&owner, 0, &second, replay_at(10.0), false)
            .unwrap(),
        winner
    );
    encounters.accept();
    let output = encounters.finish();
    assert_eq!(
        output
            .children
            .iter()
            .map(|child| child.pending.node)
            .collect::<Vec<_>>(),
        [2, 5]
    );
    for (child, original) in output.children.iter().zip(packets) {
        assert!(Arc::ptr_eq(&child.packet, &original));
        owner.accept_float_child(&child.packet).unwrap();
    }
    assert_eq!(
        output.children[1]
            .pending
            .placement
            .text()
            .unwrap()
            .logical_origin
            .block,
        S::from_f64(10.0)
    );
}

#[test]
fn real_float_group_trials_undo_only_the_suffix_and_publish_the_replayed_packets() {
    lane::<f32>();
    lane::<f64>();
}

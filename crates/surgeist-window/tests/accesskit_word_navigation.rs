#![forbid(unsafe_code)]

//! Real-consumer regression for word navigation over original selectable units.
//!
//! A TextRun's unit count may exceed 255 even though each stored word start is
//! a byte-sized index. These fixtures retain valid original text and units;
//! neither physical caret geometry nor a native accessibility client is needed.

use accesskit::{
    Action, Node, NodeId, Role, TextDirection, TextPosition, TextSelection, Tree, TreeId,
    TreeUpdate,
};
use accesskit_consumer::Tree as ConsumerTree;

const ROOT: NodeId = NodeId(1);
const INPUT: NodeId = NodeId(2);
const RUN: NodeId = NodeId(3);
const LONG_RUN_LENGTH: usize = 262;

struct Fixture {
    name: &'static str,
    text: String,
    character_lengths: Vec<u8>,
    // Independent word boundaries are expressed in original usize units.
    word_starts: Vec<usize>,
}

fn ordinary_ascii() -> Fixture {
    Fixture {
        name: "ordinary 32-unit ASCII single word",
        text: "a".repeat(32),
        character_lengths: vec![1; 32],
        word_starts: vec![0],
    }
}

fn long_ascii() -> Fixture {
    Fixture {
        name: "262-unit ASCII single word",
        text: "a".repeat(LONG_RUN_LENGTH),
        character_lengths: vec![1; LONG_RUN_LENGTH],
        word_starts: vec![0],
    }
}

fn long_utf8() -> Fixture {
    Fixture {
        name: "262-unit UTF8 single word",
        text: "é".repeat(LONG_RUN_LENGTH),
        character_lengths: vec![2; LONG_RUN_LENGTH],
        word_starts: vec![0],
    }
}

fn two_words(name: &'static str, second_word_start: usize) -> Fixture {
    Fixture {
        name,
        text: format!(
            "{} {}",
            "a".repeat(second_word_start - 1),
            "b".repeat(LONG_RUN_LENGTH - second_word_start)
        ),
        character_lengths: vec![1; LONG_RUN_LENGTH],
        word_starts: vec![0, second_word_start],
    }
}

fn all_fixtures() -> [Fixture; 5] {
    [
        ordinary_ascii(),
        long_ascii(),
        long_utf8(),
        two_words("262-unit ASCII words starting at 0 and 5", 5),
        two_words("262-unit ASCII words starting at 0 and 255", 255),
    ]
}

fn raw_position(character_index: usize) -> TextPosition {
    TextPosition {
        node: RUN,
        character_index,
    }
}

fn consumer_tree(fixture: &Fixture) -> ConsumerTree {
    assert_eq!(
        fixture
            .character_lengths
            .iter()
            .map(|length| usize::from(*length))
            .sum::<usize>(),
        fixture.text.len(),
        "{} must preserve its original UTF8 byte lengths",
        fixture.name
    );
    assert!(
        fixture.word_starts.windows(2).all(|pair| pair[0] < pair[1]),
        "{} must have sorted original word boundaries",
        fixture.name
    );
    let encoded_word_starts: Vec<_> = fixture
        .word_starts
        .iter()
        .map(|start| u8::try_from(*start).expect("fixture word starts fit the producer protocol"))
        .collect();

    let mut root = Node::new(Role::Window);
    root.set_children([INPUT]);

    let mut input = Node::new(Role::MultilineTextInput);
    input.set_children([RUN]);
    input.add_action(Action::Focus);
    input.add_action(Action::SetTextSelection);
    input.set_text_direction(TextDirection::LeftToRight);
    input.set_text_selection(TextSelection {
        anchor: raw_position(0),
        focus: raw_position(0),
    });

    let mut run = Node::new(Role::TextRun);
    run.set_value(fixture.text.clone());
    run.set_character_lengths(fixture.character_lengths.clone());
    run.set_word_starts(encoded_word_starts);
    run.set_text_direction(TextDirection::LeftToRight);

    ConsumerTree::new(
        TreeUpdate {
            nodes: vec![(ROOT, root), (INPUT, input), (RUN, run)],
            tree: Some(Tree::new(ROOT)),
            tree_id: TreeId::ROOT,
            focus: INPUT,
        },
        true,
    )
}

#[test]
fn real_consumer_preserves_original_text_units_and_every_logical_position() {
    for fixture in all_fixtures() {
        let tree = consumer_tree(&fixture);
        let state = tree.state();
        let input = state.node_by_tree_local_id(INPUT, TreeId::ROOT).unwrap();
        let run = state.node_by_tree_local_id(RUN, TreeId::ROOT).unwrap();

        assert!(state.is_host_focused(), "{}", fixture.name);
        assert_eq!(state.focus_in_tree().id(), input.id(), "{}", fixture.name);
        assert!(input.supports_text_ranges(), "{}", fixture.name);
        assert_eq!(
            run.data().value(),
            Some(fixture.text.as_str()),
            "{}",
            fixture.name
        );
        assert_eq!(
            run.data().character_lengths(),
            fixture.character_lengths,
            "{}",
            fixture.name
        );
        assert_eq!(
            run.data()
                .word_starts()
                .iter()
                .map(|start| usize::from(*start))
                .collect::<Vec<_>>(),
            fixture.word_starts,
            "{}",
            fixture.name
        );
        assert_eq!(
            input.document_range().text(),
            fixture.text,
            "{}",
            fixture.name
        );

        let true_end = fixture.character_lengths.len();
        assert_eq!(
            input.document_start().to_raw(),
            raw_position(0),
            "{}",
            fixture.name
        );
        assert_eq!(
            input.document_end().to_raw(),
            raw_position(true_end),
            "{}",
            fixture.name
        );
        for index in 0..=true_end {
            let position = input.text_position_from_global_usv_index(index).unwrap();
            assert_eq!(
                position.to_raw(),
                raw_position(index),
                "{} index {index}",
                fixture.name
            );
            assert_eq!(
                position.to_global_usv_index(),
                index,
                "{} index {index}",
                fixture.name
            );
            assert_eq!(
                position.to_global_utf16_index(),
                index,
                "{} index {index}",
                fixture.name
            );
            assert_eq!(
                position.is_document_end(),
                index == true_end,
                "{} index {index}",
                fixture.name
            );
        }
        assert!(
            input
                .text_position_from_global_usv_index(true_end + 1)
                .is_none(),
            "{}",
            fixture.name
        );
    }
}

fn assert_word_navigation(fixture: Fixture) {
    let tree = consumer_tree(&fixture);
    let input = tree
        .state()
        .node_by_tree_local_id(INPUT, TreeId::ROOT)
        .unwrap();
    let true_end = fixture.character_lengths.len();
    let mut mismatches = Vec::new();

    for index in 0..=true_end {
        let position = input.text_position_from_global_usv_index(index).unwrap();
        let expected_word_start = fixture.word_starts.contains(&index);
        let actual_word_start = position.is_word_start();
        if actual_word_start != expected_word_start {
            mismatches.push(format!(
                "index {index}, is_word_start: actual {actual_word_start}, expected {expected_word_start}"
            ));
        }

        // Read the fixture's declared boundaries at their original width. The
        // first strictly later start ends this word; the true end ends the last.
        let expected_forward = fixture
            .word_starts
            .iter()
            .copied()
            .find(|start| *start > index)
            .unwrap_or(true_end);
        let expected_backward = fixture
            .word_starts
            .iter()
            .copied()
            .rfind(|start| *start < index)
            .unwrap_or(0);
        for (operation, actual, expected) in [
            (
                "forward_to_word_start",
                position.forward_to_word_start().to_raw(),
                expected_forward,
            ),
            (
                "forward_to_word_end",
                position.forward_to_word_end().to_raw(),
                expected_forward,
            ),
            (
                "backward_to_word_start",
                position.backward_to_word_start().to_raw(),
                expected_backward,
            ),
        ] {
            let expected = raw_position(expected);
            if actual != expected {
                mismatches.push(format!(
                    "index {index}, {operation}: actual {actual:?}, expected {expected:?}"
                ));
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "{}:\n{}",
        fixture.name,
        mismatches.join("\n")
    );
}

#[test]
fn ordinary_single_word_navigation_uses_original_units() {
    assert_word_navigation(ordinary_ascii());
}

#[test]
fn long_ascii_single_word_navigation_uses_original_units() {
    assert_word_navigation(long_ascii());
}

#[test]
fn long_utf8_single_word_navigation_uses_original_units() {
    assert_word_navigation(long_utf8());
}

#[test]
fn long_two_word_navigation_uses_original_units() {
    assert_word_navigation(two_words("262-unit ASCII words starting at 0 and 5", 5));
}

#[test]
fn maximum_encoded_word_start_preserves_original_navigation_indices() {
    assert_word_navigation(two_words("262-unit ASCII words starting at 0 and 255", 255));
}

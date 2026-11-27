//! Native range operations with authored HTML877d UTF-16 expectations.

use std::sync::atomic::AtomicBool;
use surgeist_dom::{
    Dom, DomBatch, DomError,
    identity::NodeId,
    limits::{DomLimits, DomNameLimits, DomResource},
    source::{DomSourceEdit, DomSourceLimits, DomSourceProgress},
};
use surgeist_html::{
    elements::input::{
        HtmlInputSelectionDirection as Direction, HtmlRangeTextMode as Mode,
        HtmlRangeTextTarget as Target, HtmlTextSelectionEdit as Edit,
    },
    values::{DocumentString, Namespace},
};

fn dom_limits(units: usize) -> DomLimits {
    DomLimits::new(
        128,
        units,
        512,
        0,
        0,
        8,
        100_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
}

fn control(name: &str, value: DocumentString) -> (Dom, NodeId) {
    let mut dom = Dom::new(dom_limits(8192)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let control = batch.create_element_ns(&document, Namespace::html(), name);
    batch.insert_before(&control, &document, None);
    if name == "textarea" {
        let text = batch.create_text(&document, value);
        batch.insert_before(&text, &control, None);
    } else {
        batch.set_attribute(&control, "value", value);
    }
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let node = commit.created(&control).unwrap().unwrap();
    (dom, node)
}

#[test]
fn range_splice_storage_refusal_preserves_value_selection_and_clean_flag() {
    let (mut dom, node) = control("input", "xyz".into());
    let units = dom.usage().string_code_units();
    let before = dom.snapshot();
    dom.set_limits(dom_limits(units)).unwrap();
    let edit = || {
        DomSourceEdit::set_range_text(
            node,
            DocumentString::from_utf16(vec![0xd800, 0, 0xdc00, 0x0061]),
            Target::Offsets {
                start: 1,
                end: 2,
                mode: Mode::Select,
            },
        )
    };
    assert_eq!(
        apply(&mut dom, edit()).unwrap_err(),
        DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            limit: units
        },
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let current = dom.snapshot();
    let state = current.node(node).unwrap().as_input().unwrap().state();
    assert_eq!(state.current_value(), &DocumentString::from("xyz"));
    assert!(!state.flags().dirty_value());
    assert!(dom.pending_source_edit().is_none());

    dom.set_limits(dom_limits(units + 3)).unwrap();
    assert!(matches!(
        apply(&mut dom, edit()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    let after = dom.snapshot();
    let state = after.node(node).unwrap().as_input().unwrap().state();
    assert_eq!(
        state.current_value().code_units(),
        [0x0078, 0xd800, 0, 0xdc00, 0x0061, 0x007a]
    );
    assert_eq!(
        state.selection().unwrap().selection_range(),
        Some((1, 5, Direction::None))
    );
    assert!(state.flags().dirty_value());
}

fn source(work: u64, phases: u64) -> DomSourceLimits {
    DomSourceLimits::new(16_000_000, 64_000, work, phases, 512, 512, 1)
}

fn apply(dom: &mut Dom, edit: DomSourceEdit) -> Result<DomSourceProgress, DomError> {
    let cancel = AtomicBool::new(false);
    dom.begin_source_edit(edit, source(4_000_000, 64), &cancel)?
        .advance(&cancel)
}

#[test]
fn textarea_splices_its_normalized_value_without_losing_exact_code_units() {
    let raw = DocumentString::from_utf16(vec![97, 13, 10, 0xd800, 98]);
    let (mut dom, node) = control("textarea", raw.clone());
    assert!(matches!(
        apply(
            &mut dom,
            DomSourceEdit::edit_text_selection(
                node,
                Edit::Range {
                    start: 1,
                    end: 3,
                    direction: Direction::Backward,
                }
            ),
        )
        .unwrap(),
        DomSourceProgress::Complete(_)
    ));
    let before = dom.snapshot();
    let selected = before.node(node).unwrap().as_textarea().unwrap().state();
    assert_eq!(
        selected.selection().selection_range(),
        Some((1, 3, Direction::Backward))
    );
    assert_eq!(selected.raw_value(), &raw);
    assert!(!selected.flags().dirty_value());
    let insert = DocumentString::from_utf16(vec![120, 0xdc00]);
    assert!(matches!(
        apply(
            &mut dom,
            DomSourceEdit::set_range_text(node, insert, Target::CurrentSelection)
        )
        .unwrap(),
        DomSourceProgress::Complete(_),
    ));
    let after = dom.snapshot();
    let state = after.node(node).unwrap().as_textarea().unwrap().state();
    assert_eq!(state.raw_value().code_units(), [97, 120, 0xdc00, 98]);
    assert_eq!(
        state.selection().selection_range(),
        Some((1, 3, Direction::None))
    );
    assert!(state.flags().dirty_value());
    assert_eq!(selected.raw_value(), &raw);
    assert_ne!(before.stamp(), after.stamp());
}

#[test]
fn reversed_range_publishes_only_the_dirty_flag_and_keeps_first_failure() {
    for name in ["input", "textarea"] {
        let (mut dom, node) = control(name, "abcd".into());
        let before = dom.snapshot();
        let cancel = AtomicBool::new(false);
        let edit = DomSourceEdit::set_range_text(
            node,
            "ignored",
            Target::Offsets {
                start: 10,
                end: 9,
                mode: Mode::Select,
            },
        );
        let mut lease = dom
            .begin_source_edit(edit, source(4_000_000, 0), &cancel)
            .unwrap();
        let operation = lease.operation();
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::SourcePublicationLimit)
        ));
        assert_eq!(lease.snapshot().stamp(), before.stamp());
        drop(lease);
        let mut lease = dom
            .resume_source_edit(operation, source(4_000_000, 64), &cancel)
            .unwrap();
        lease.request_cancel();
        let DomSourceProgress::Failed(failure) = lease.advance(&cancel).unwrap() else {
            panic!("the original explicit range error must survive refusal")
        };
        assert_eq!(failure.operation(), operation);
        assert_eq!(failure.error(), &DomError::IndexSize);
        drop(lease);
        let after = dom.snapshot();
        let actual = after.node(node).unwrap();
        if let Some(input) = actual.as_input() {
            let old = before.node(node).unwrap().as_input().unwrap().state();
            assert_eq!(input.state().selection(), old.selection());
            assert_eq!(input.state().current_value(), old.current_value());
            assert!(input.state().flags().dirty_value());
        } else {
            let old = before.node(node).unwrap().as_textarea().unwrap().state();
            let state = actual.as_textarea().unwrap().state();
            assert_eq!(state.selection(), old.selection());
            assert_eq!(state.raw_value(), old.raw_value());
            assert!(state.flags().dirty_value());
        }
    }
}

#[test]
fn inapplicable_input_has_null_selection_without_dirtying_its_current_value() {
    let (mut dom, node) = control("input", "3".into());
    let mut attributes = DomBatch::new();
    attributes.set_attribute(node, "type", "number");
    dom.apply(attributes, &AtomicBool::new(false)).unwrap();
    let before = dom.snapshot();
    assert!(matches!(
        apply(
            &mut dom,
            DomSourceEdit::edit_text_selection(node, Edit::All)
        )
        .unwrap(),
        DomSourceProgress::Complete(_),
    ));
    let invalid_range = Target::Offsets {
        start: 8,
        end: 0,
        mode: Mode::End,
    };
    assert_eq!(
        apply(
            &mut dom,
            DomSourceEdit::set_range_text(node, "ignored", invalid_range)
        )
        .unwrap_err(),
        DomError::InputSelectionInvalidState,
    );
    let after = dom.snapshot();
    assert_eq!(after.stamp(), before.stamp());
    let input = after.node(node).unwrap().as_input().unwrap().state();
    assert!(input.selection().is_none());
    assert!(!input.flags().dirty_value());
    assert_eq!(input.current_value(), &DocumentString::from("3"));
}

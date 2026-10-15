//! Literal Output source/operation expectations from HTML877d41608–41665.
use surgeist_html::{
    DocumentString, HtmlOutputAction as Action, HtmlOutputCursor as Cursor, HtmlOutputCursorError,
    HtmlOutputDefaultSource as Source, HtmlOutputOperation as Operation, output_default_source,
};

#[test]
fn default_source_uses_actual_presence_including_present_empty_without_normalization() {
    let descendant_text = DocumentString::from_utf16(vec![65, 13, 10, 0, 0xd800]);
    for (actual_override, expected_source, expected_units) in [
        (None, Source::DescendantText, &[65, 13, 10, 0, 0xd800][..]),
        (Some(DocumentString::from("")), Source::Override, &[][..]),
        (
            Some(DocumentString::from_utf16(vec![66, 13, 0xdc00, 0])),
            Source::Override,
            &[66, 13, 0xdc00, 0][..],
        ),
    ] {
        let source = output_default_source(actual_override.is_some());
        assert_eq!(source, expected_source);
        // Both strings are already established by the consumer. The selector
        // chooses a source; HTML does not own live descendant traversal here.
        let selected = match source {
            Source::Override => actual_override.as_ref().unwrap(),
            Source::DescendantText => &descendant_text,
        };
        assert_eq!(selected.code_units(), expected_units);
    }
}

fn assert_order(operation: Operation, expected: &[Action]) {
    let mut cursor = Cursor::new(operation);
    for &action in expected {
        assert_eq!(cursor.action(), action);
        assert_eq!(
            cursor.action(),
            action,
            "observation does not acknowledge work"
        );
        if action == Action::Complete {
            assert_eq!(cursor.acknowledge(action), Err(HtmlOutputCursorError));
        } else {
            cursor.acknowledge(action).unwrap();
        }
    }
    assert_eq!(cursor.action(), Action::Complete);
}

#[test]
fn value_setter_reads_and_stores_default_before_replacing_supplied_text() {
    assert_order(
        Operation::ValueSetter,
        &[
            Action::ReadDefaultValue,
            Action::StoreReadDefaultAsOverride,
            Action::ReplaceAllWithSuppliedValue,
            Action::Complete,
        ],
    );
}

#[test]
fn reset_reads_and_replaces_default_before_clearing_override() {
    assert_order(
        Operation::Reset,
        &[
            Action::ReadDefaultValue,
            Action::ReplaceAllWithReadDefault,
            Action::ClearDefaultOverride,
            Action::Complete,
        ],
    );
}

#[test]
fn default_setter_replaces_only_for_null_and_stores_only_for_actual_some() {
    let null: Option<DocumentString> = None;
    assert_order(
        Operation::DefaultValueSetter {
            has_override: null.is_some(),
        },
        &[Action::ReplaceAllWithSuppliedValue, Action::Complete],
    );
    for actual_override in [DocumentString::from(""), DocumentString::from("default")] {
        let present = Some(actual_override);
        assert_order(
            Operation::DefaultValueSetter {
                has_override: present.is_some(),
            },
            &[Action::StoreSuppliedValueAsOverride, Action::Complete],
        );
    }
}

#[test]
fn wrong_acknowledgements_preserve_each_pending_action_and_valid_continuation() {
    let all_actions = [
        Action::ReadDefaultValue,
        Action::StoreReadDefaultAsOverride,
        Action::ReplaceAllWithSuppliedValue,
        Action::StoreSuppliedValueAsOverride,
        Action::ReplaceAllWithReadDefault,
        Action::ClearDefaultOverride,
        Action::Complete,
    ];
    for (operation, expected) in [
        (
            Operation::ValueSetter,
            &[
                Action::ReadDefaultValue,
                Action::StoreReadDefaultAsOverride,
                Action::ReplaceAllWithSuppliedValue,
                Action::Complete,
            ][..],
        ),
        (
            Operation::Reset,
            &[
                Action::ReadDefaultValue,
                Action::ReplaceAllWithReadDefault,
                Action::ClearDefaultOverride,
                Action::Complete,
            ][..],
        ),
        (
            Operation::DefaultValueSetter {
                has_override: false,
            },
            &[Action::ReplaceAllWithSuppliedValue, Action::Complete][..],
        ),
        (
            Operation::DefaultValueSetter { has_override: true },
            &[Action::StoreSuppliedValueAsOverride, Action::Complete][..],
        ),
    ] {
        let mut cursor = Cursor::new(operation);
        for &pending in expected {
            assert_eq!(cursor.action(), pending);
            for acknowledgement in all_actions {
                if acknowledgement == pending && pending != Action::Complete {
                    continue;
                }
                let before = cursor;
                assert_eq!(
                    cursor.acknowledge(acknowledgement),
                    Err(HtmlOutputCursorError)
                );
                assert_eq!(cursor, before);
                assert_eq!(cursor.action(), pending);
            }
            if pending != Action::Complete {
                cursor.acknowledge(pending).unwrap();
            }
        }
        assert_eq!(cursor.action(), Action::Complete);
    }
}

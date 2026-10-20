use surgeist_html::{
    DocumentString, HtmlClosedBy, HtmlClosedByState, HtmlDialogAction as A, HtmlDialogCursor,
    HtmlDialogCursorError, HtmlDialogEventSource as Source, HtmlDialogFreshState,
    HtmlDialogOperation as Op, HtmlDialogOutcome as Outcome, HtmlDialogReply as R,
    HtmlDialogState as State, HtmlDialogToggleDecision, HtmlDialogToggleEffect as T,
    KnownHtmlAttribute, Namespace, computed_dialog_closed_by, dialog_attribute_operation,
    dialog_close_watcher_enabled,
};

fn step(cursor: &mut HtmlDialogCursor, action: A, reply: R) {
    assert_eq!(cursor.action(), action);
    cursor.reply(reply).unwrap();
}
fn complete(cursor: &HtmlDialogCursor) {
    assert_eq!(cursor.action(), A::Complete(Outcome::Completed));
}
fn event(old: State, new: State, cancelable: bool, source: Source) -> A {
    A::FireBeforeToggle {
        old,
        new,
        cancelable,
        source,
    }
}
fn queued(old: State, new: State, source: Source) -> A {
    A::QueueToggle { old, new, source }
}
fn close_prefix(cursor: &mut HtmlDialogCursor) {
    step(cursor, A::ReadOpen, R::Open(true));
    step(
        cursor,
        event(State::Open, State::Closed, false, Source::OperationSource),
        R::Applied,
    );
    step(cursor, A::ReadOpen, R::Open(true));
    step(
        cursor,
        queued(State::Open, State::Closed, Source::OperationSource),
        R::Applied,
    );
    step(cursor, A::RemoveOpen, R::Applied);
}
fn close_fields(cursor: &mut HtmlDialogCursor, result_present: bool) {
    step(cursor, A::ClearModal, R::Applied);
    if result_present {
        step(cursor, A::WriteReturnValue, R::Applied);
    }
    step(cursor, A::ClearRequestCloseReturnValue, R::Applied);
    step(cursor, A::ClearRequestCloseSource, R::Applied);
}

#[test]
fn fresh_dialog_slots_preserve_empty_null_and_false_initial_values() {
    let fresh = HtmlDialogFreshState::fresh();
    assert_eq!(fresh.return_value(), "");
    assert!(!fresh.is_modal());
    assert!(!fresh.has_toggle_task_tracker());
    assert!(!fresh.has_close_watcher());
    assert!(!fresh.has_request_close_return_value());
    assert!(!fresh.has_request_close_source());
    assert!(!fresh.force_close_watcher_enabled());
}

#[test]
fn already_open_show_distinguishes_modal_error_from_nonmodal_return() {
    for (modal, expected) in [(false, Outcome::Completed), (true, Outcome::InvalidState)] {
        let mut cursor = HtmlDialogCursor::new(Op::Show);
        step(&mut cursor, A::ReadOpen, R::Open(true));
        step(&mut cursor, A::ReadModal, R::Modal(modal));
        assert_eq!(cursor.action(), A::Complete(expected));
    }
}

#[test]
fn canceled_show_and_event_created_open_stop_before_queue_or_attribute_effects() {
    for canceled in [true, false] {
        let mut cursor = HtmlDialogCursor::new(Op::Show);
        step(&mut cursor, A::ReadOpen, R::Open(false));
        step(
            &mut cursor,
            event(State::Closed, State::Open, true, Source::Null),
            R::EventCanceled(canceled),
        );
        if !canceled {
            step(&mut cursor, A::ReadOpen, R::Open(true));
        }
        complete(&cursor);
    }
}

#[test]
fn show_queues_before_nested_open_hook_and_preserves_document_local_for_popover_hide() {
    let mut cursor = HtmlDialogCursor::new(Op::Show);
    step(&mut cursor, A::ReadOpen, R::Open(false));
    step(
        &mut cursor,
        event(State::Closed, State::Open, true, Source::Null),
        R::EventCanceled(false),
    );
    step(&mut cursor, A::ReadOpen, R::Open(false));
    step(
        &mut cursor,
        queued(State::Closed, State::Open, Source::Null),
        R::Applied,
    );
    assert_eq!(cursor.action(), A::AddOpen);
    let operation =
        dialog_attribute_operation(&Namespace::none(), KnownHtmlAttribute::Open, false, true)
            .unwrap();
    let mut hook = HtmlDialogCursor::new(operation);
    step(&mut hook, A::ReadFullyActive, R::FullyActive(true));
    step(&mut hook, A::ReadConnected, R::Connected(true));
    step(&mut hook, A::AddOpenDialog, R::Applied);
    step(&mut hook, A::EstablishCloseWatcher, R::Applied);
    complete(&hook);
    // The main cursor cannot run focus/popover work until the real nested hook finishes.
    step(&mut cursor, A::AddOpen, R::Applied);
    step(&mut cursor, A::RememberFocusedElement, R::Applied);
    step(&mut cursor, A::CaptureNodeDocument, R::Applied);
    step(&mut cursor, A::FindTopmostPopoverAncestor, R::Applied);
    step(
        &mut cursor,
        A::HidePopoversUntil {
            focus_previous_element: false,
            fire_events: true,
        },
        R::Applied,
    );
    step(&mut cursor, A::RunDialogFocusingSteps, R::Applied);
    complete(&cursor);
    // The literal trace has no ReadFullyActive or modal write in show itself.
}

#[test]
fn close_event_reentry_rereads_open_and_noncancelable_dispatch_has_no_cancel_reply() {
    let mut closed = HtmlDialogCursor::new(Op::Close {
        result_present: true,
    });
    step(&mut closed, A::ReadOpen, R::Open(false));
    complete(&closed);
    let mut cursor = HtmlDialogCursor::new(Op::Close {
        result_present: false,
    });
    step(&mut cursor, A::ReadOpen, R::Open(true));
    let pending = cursor;
    assert_eq!(
        cursor.reply(R::EventCanceled(true)),
        Err(HtmlDialogCursorError::WrongReply)
    );
    assert_eq!(cursor, pending);
    step(
        &mut cursor,
        event(State::Open, State::Closed, false, Source::OperationSource),
        R::Applied,
    );
    step(&mut cursor, A::ReadOpen, R::Open(false));
    complete(&cursor);
}

#[test]
fn omitted_result_and_supplied_empty_have_distinct_return_value_writes() {
    for result in [None, Some(DocumentString::from(""))] {
        let present = result.is_some();
        let mut cursor = HtmlDialogCursor::new(Op::Close {
            result_present: present,
        });
        close_prefix(&mut cursor);
        step(&mut cursor, A::ReadModal, R::Modal(false));
        step(&mut cursor, A::ReadModal, R::Modal(false));
        close_fields(&mut cursor, present);
        step(
            &mut cursor,
            A::ReadPreviousFocusPresent,
            R::PreviousFocusPresent(false),
        );
        step(&mut cursor, A::QueueClose, R::Applied);
        complete(&cursor);
        assert!(result.as_ref().is_none_or(DocumentString::is_empty));
    }
}

#[test]
fn close_reads_modal_again_after_layer_service_and_clears_focus_before_restoring() {
    for (before, after, inside, restore) in [
        (true, false, false, false),
        (true, true, false, true),
        (false, false, true, true),
    ] {
        let mut cursor = HtmlDialogCursor::new(Op::Close {
            result_present: false,
        });
        close_prefix(&mut cursor);
        step(&mut cursor, A::ReadModal, R::Modal(before));
        if before {
            step(&mut cursor, A::RequestTopLayerRemoval, R::Applied);
        }
        step(&mut cursor, A::ReadModal, R::Modal(after));
        close_fields(&mut cursor, false);
        step(
            &mut cursor,
            A::ReadPreviousFocusPresent,
            R::PreviousFocusPresent(true),
        );
        step(&mut cursor, A::ClearPreviouslyFocusedElement, R::Applied);
        step(
            &mut cursor,
            A::ReadFocusedAnchorInside,
            R::FocusedAnchorInside(inside),
        );
        if restore {
            step(
                &mut cursor,
                A::RestorePreviousFocusWithoutScrolling,
                R::Applied,
            );
        }
        step(&mut cursor, A::QueueClose, R::Applied);
        complete(&cursor);
    }
}

#[test]
fn insertion_stops_at_actual_guards_and_setup_adds_list_before_watcher() {
    for (active, open, connected) in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
        (true, true, true),
    ] {
        let mut cursor = HtmlDialogCursor::new(Op::Inserted);
        step(&mut cursor, A::ReadFullyActive, R::FullyActive(active));
        if active {
            step(&mut cursor, A::ReadOpen, R::Open(open));
            if open {
                step(&mut cursor, A::ReadConnected, R::Connected(connected));
                if connected {
                    step(&mut cursor, A::AddOpenDialog, R::Applied);
                    step(&mut cursor, A::EstablishCloseWatcher, R::Applied);
                }
            }
        }
        complete(&cursor);
    }
}

#[test]
fn open_removal_cleans_up_before_activity_and_equal_assignments_still_read_guards() {
    let namespace = Namespace::none();
    assert_eq!(
        dialog_attribute_operation(&Namespace::html(), KnownHtmlAttribute::Open, true, false),
        None
    );
    assert_eq!(
        dialog_attribute_operation(&namespace, KnownHtmlAttribute::Name, true, false),
        None
    );
    let operation =
        dialog_attribute_operation(&namespace, KnownHtmlAttribute::Open, true, false).unwrap();
    let mut cursor = HtmlDialogCursor::new(operation);
    step(&mut cursor, A::RemoveOpenDialog, R::Applied);
    step(&mut cursor, A::ReadWatcherPresent, R::WatcherPresent(true));
    step(&mut cursor, A::DestroyCloseWatcher, R::Applied);
    step(&mut cursor, A::ClearCloseWatcher, R::Applied);
    step(&mut cursor, A::ReadFullyActive, R::FullyActive(false));
    complete(&cursor);
    for present in [false, true] {
        let operation =
            dialog_attribute_operation(&namespace, KnownHtmlAttribute::Open, present, present)
                .unwrap();
        let mut cursor = HtmlDialogCursor::new(operation);
        step(&mut cursor, A::ReadFullyActive, R::FullyActive(true));
        step(&mut cursor, A::ReadConnected, R::Connected(true));
        complete(&cursor);
    }
}

#[test]
fn removal_cleanup_then_immediate_layer_removal_clears_modal_even_when_closed() {
    for open in [false, true] {
        for in_layer in [false, true] {
            let mut cursor = HtmlDialogCursor::new(Op::Removed);
            step(&mut cursor, A::ReadOpen, R::Open(open));
            if open {
                step(&mut cursor, A::RemoveOpenDialog, R::Applied);
                step(&mut cursor, A::ReadWatcherPresent, R::WatcherPresent(false));
            }
            step(
                &mut cursor,
                A::ReadTopLayerContains,
                R::TopLayerContains(in_layer),
            );
            if in_layer {
                step(&mut cursor, A::RemoveTopLayerImmediately, R::Applied);
            }
            step(&mut cursor, A::ClearModal, R::Applied);
            complete(&cursor);
        }
    }
}

#[test]
fn wrong_terminal_and_unfulfilled_replies_preserve_the_exact_pending_phase() {
    let mut cursor = HtmlDialogCursor::new(Op::Show);
    let before = cursor;
    assert_eq!(
        cursor.reply(R::Connected(false)),
        Err(HtmlDialogCursorError::WrongReply)
    );
    assert_eq!(cursor, before);
    step(&mut cursor, A::ReadOpen, R::Open(false));
    // No reply is made for unavailable/canceled caller work: no false or Applied is invented.
    let pending = cursor;
    assert_eq!(cursor.action(), pending.action());
    assert_eq!(
        cursor.reply(R::Applied),
        Err(HtmlDialogCursorError::WrongReply)
    );
    assert_eq!(cursor, pending);
    cursor.reply(R::EventCanceled(true)).unwrap();
    let terminal = cursor;
    assert_eq!(
        cursor.reply(R::Applied),
        Err(HtmlDialogCursorError::Terminal)
    );
    assert_eq!(cursor, terminal);
}

#[test]
fn closed_by_auto_and_watcher_enable_use_actual_modal_and_force_values() {
    assert_eq!(
        computed_dialog_closed_by(HtmlClosedByState::Auto, false),
        HtmlClosedBy::None
    );
    assert_eq!(
        computed_dialog_closed_by(HtmlClosedByState::Auto, true),
        HtmlClosedBy::CloseRequest
    );
    for keyword in [
        HtmlClosedBy::Any,
        HtmlClosedBy::CloseRequest,
        HtmlClosedBy::None,
    ] {
        for modal in [false, true] {
            let state = HtmlClosedByState::Keyword(keyword);
            assert_eq!(computed_dialog_closed_by(state, modal), keyword);
            assert_eq!(
                dialog_close_watcher_enabled(false, state, modal),
                keyword != HtmlClosedBy::None
            );
            assert!(dialog_close_watcher_enabled(true, state, modal));
        }
    }
}

#[test]
fn toggle_replacement_uses_first_old_latest_new_source_and_postdispatch_clear() {
    let decision = HtmlDialogToggleDecision::new(State::Open, State::Closed, Some(State::Closed));
    assert_eq!(decision.old_state(), State::Closed);
    assert_eq!(decision.new_state(), State::Closed);
    assert_eq!(
        decision.queue_effects().collect::<Vec<_>>(),
        vec![
            T::RemoveTrackedTask,
            T::ClearTracker,
            T::QueueEventFromLatestSource {
                old: State::Closed,
                new: State::Closed
            },
            T::StoreTracker { old: State::Closed },
        ]
    );
    assert_eq!(
        decision.dispatch_effects(),
        [
            T::FireQueuedEvent {
                old: State::Closed,
                new: State::Closed
            },
            T::ClearTracker,
        ]
    );
    let first = HtmlDialogToggleDecision::new(State::Closed, State::Open, None);
    assert_eq!(
        first.queue_effects().collect::<Vec<_>>(),
        vec![
            T::QueueEventFromLatestSource {
                old: State::Closed,
                new: State::Open
            },
            T::StoreTracker { old: State::Closed },
        ]
    );
}

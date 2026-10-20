//! Independent literal traces from HTML877d raw46458–46460,46585–46615.
use surgeist_html::{
    HtmlDialogAction as A, HtmlDialogCursor as C, HtmlDialogCursorError as E,
    HtmlDialogEventSource as Source, HtmlDialogOperation as O, HtmlDialogOutcome as Outcome,
    HtmlDialogReply as R, HtmlDialogState as State,
};

fn step(c: &mut C, action: A, reply: R) {
    assert_eq!(c.action(), action);
    let before = *c;
    let wrong = if matches!(action, A::ReadOpen) {
        R::Applied
    } else {
        R::Open(false)
    };
    assert_eq!(c.reply(wrong), Err(E::WrongReply));
    assert_eq!(*c, before);
    c.reply(reply).unwrap();
}
fn complete(c: &mut C, outcome: Outcome) {
    assert_eq!(c.action(), A::Complete(outcome));
    let before = *c;
    for reply in [R::Applied, R::Open(false), R::EventCanceled(false)] {
        assert_eq!(c.reply(reply), Err(E::Terminal));
        assert_eq!(*c, before);
    }
}
fn initial_guards(c: &mut C) {
    step(c, A::ReadOpen, R::Open(false));
    step(c, A::ReadFullyActive, R::FullyActive(true));
    step(c, A::ReadConnected, R::Connected(true));
    step(c, A::ReadPopoverShowing, R::PopoverShowing(false));
}
fn dispatch(c: &mut C, canceled: bool) {
    assert_eq!(
        c.action(),
        A::FireBeforeToggle {
            old: State::Closed,
            new: State::Open,
            cancelable: true,
            source: Source::Null,
        }
    );
    let before = *c;
    // Dispatch must return its actual cancellation result, not an effect ack.
    assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(*c, before);
    step(
        c,
        A::FireBeforeToggle {
            old: State::Closed,
            new: State::Open,
            cancelable: true,
            source: Source::Null,
        },
        R::EventCanceled(canceled),
    );
}
#[test]
fn initially_open_uses_modal_polarity_before_document_or_connection_checks() {
    for (modal, outcome) in [(true, Outcome::Completed), (false, Outcome::InvalidState)] {
        let mut c = C::new(O::ShowModal);
        step(&mut c, A::ReadOpen, R::Open(true));
        step(&mut c, A::ReadModal, R::Modal(modal));
        complete(&mut c, outcome);
    }
}
#[test]
fn initial_inactivity_disconnection_and_showing_popover_throw_in_source_order() {
    for guard in 0..3 {
        let mut c = C::new(O::ShowModal);
        step(&mut c, A::ReadOpen, R::Open(false));
        step(&mut c, A::ReadFullyActive, R::FullyActive(guard != 0));
        if guard == 0 {
            complete(&mut c, Outcome::InvalidState);
            continue;
        }
        step(&mut c, A::ReadConnected, R::Connected(guard != 1));
        if guard == 1 {
            complete(&mut c, Outcome::InvalidState);
            continue;
        }
        step(&mut c, A::ReadPopoverShowing, R::PopoverShowing(true));
        complete(&mut c, Outcome::InvalidState);
    }
}
#[test]
fn canceled_beforetoggle_returns_without_post_dispatch_reads() {
    let mut c = C::new(O::ShowModal);
    initial_guards(&mut c);
    dispatch(&mut c, true);
    complete(&mut c, Outcome::Completed);
}
#[test]
fn callback_open_disconnection_or_popover_changes_return_normally() {
    for guard in 0..3 {
        let mut c = C::new(O::ShowModal);
        initial_guards(&mut c);
        dispatch(&mut c, false);
        step(&mut c, A::ReadOpen, R::Open(guard == 0));
        if guard == 0 {
            complete(&mut c, Outcome::Completed);
            continue;
        }
        step(&mut c, A::ReadConnected, R::Connected(guard != 1));
        if guard == 1 {
            complete(&mut c, Outcome::Completed);
            continue;
        }
        step(&mut c, A::ReadPopoverShowing, R::PopoverShowing(true));
        complete(&mut c, Outcome::Completed);
    }
}
#[test]
fn successful_show_finishes_open_hook_assertion_blocking_and_conditional_layer_before_focus_tail() {
    for already_in_layer in [false, true] {
        let mut c = C::new(O::ShowModal);
        initial_guards(&mut c);
        dispatch(&mut c, false);
        step(&mut c, A::ReadOpen, R::Open(false));
        // No second fully-active read after dispatch, even if activity changed.
        assert_eq!(c.action(), A::ReadConnected);
        let before = c;
        assert_eq!(c.reply(R::FullyActive(false)), Err(E::WrongReply));
        assert_eq!(c, before);
        step(&mut c, A::ReadConnected, R::Connected(true));
        step(&mut c, A::ReadPopoverShowing, R::PopoverShowing(false));
        step(
            &mut c,
            A::QueueToggle {
                old: State::Closed,
                new: State::Open,
                source: Source::Null,
            },
            R::Applied,
        );
        step(&mut c, A::AddOpen, R::Applied);
        // Assertion is distinct from requestClose's invocation-local capture.
        let before = c;
        assert_eq!(c.reply(R::WatcherPresent(false)), Err(E::WrongReply));
        assert_eq!(c, before);
        step(&mut c, A::AssertCloseWatcherPresent, R::Applied);
        step(&mut c, A::SetModal, R::Applied);
        step(&mut c, A::BlockNodeDocument, R::Applied);
        step(
            &mut c,
            A::ReadTopLayerContains,
            R::TopLayerContains(already_in_layer),
        );
        if !already_in_layer {
            step(&mut c, A::AddTopLayer, R::Applied);
        }
        step(&mut c, A::RememberFocusedElement, R::Applied);
        step(&mut c, A::CaptureNodeDocument, R::Applied);
        step(&mut c, A::FindTopmostPopoverAncestor, R::Applied);
        step(
            &mut c,
            A::HidePopoversUntil {
                focus_previous_element: false,
                fire_events: true,
            },
            R::Applied,
        );
        step(&mut c, A::RunDialogFocusingSteps, R::Applied);
        complete(&mut c, Outcome::Completed);
    }
}

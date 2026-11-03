//! Literal source traces from HTML877d raw62026–62069 and selected cancel correction.
use std::num::NonZeroUsize;
use surgeist_html::{
    HtmlCloseWatcherRequestAction as A, HtmlCloseWatcherRequestCursor as C,
    HtmlCloseWatcherRequestReply as R, HtmlCloseWatcherState,
};

fn step(c: &mut C, action: A, reply: R) {
    assert_eq!(c.action(), action);
    let before = *c;
    assert!(matches!(c.reply(R::CancelActionResult(false)), Err(ref e) if e.is_wrong_reply()));
    assert_eq!(*c, before);
    c.reply(reply).unwrap();
}
fn complete(c: &mut C, result: bool) {
    assert_eq!(c.action(), A::Complete(result));
    let before = *c;
    assert!(matches!(c.reply(R::Applied), Err(ref e) if e.is_terminal()));
    assert_eq!(*c, before);
}
fn request_guards(c: &mut C) {
    step(c, A::ReadActive, R::Active(true));
    step(c, A::ReadEnabled, R::Enabled(true));
    step(c, A::ReadRunningCancelAction, R::RunningCancelAction(false));
    step(c, A::CaptureWindow, R::Applied);
    step(c, A::ReadWindowFullyActive, R::WindowFullyActive(true));
}
fn cancel(c: &mut C, can_prevent: bool, result: bool) {
    step(c, A::SetRunningCancelAction(true), R::Applied);
    assert_eq!(
        c.action(),
        A::RunCancelAction {
            can_prevent_close: can_prevent
        }
    );
    let before = *c;
    assert!(matches!(c.reply(R::Applied), Err(ref e) if e.is_wrong_reply()));
    assert_eq!(*c, before);
    c.reply(R::CancelActionResult(result)).unwrap();
    step(c, A::SetRunningCancelAction(false), R::Applied);
}
fn cancel_without_continuing(c: &mut C) {
    step(c, A::SetRunningCancelAction(true), R::Applied);
    assert_eq!(
        c.action(),
        A::RunCancelAction {
            can_prevent_close: true
        }
    );
    c.reply(R::CancelActionResult(false)).unwrap();
    step(c, A::SetRunningCancelAction(false), R::Applied);
}
#[test]
fn request_guards_short_circuit_in_source_order() {
    for guard in 0..4 {
        let mut c = C::new(true);
        step(&mut c, A::ReadActive, R::Active(guard != 0));
        if guard == 0 {
            complete(&mut c, true);
            continue;
        }
        step(&mut c, A::ReadEnabled, R::Enabled(guard != 1));
        if guard == 1 {
            complete(&mut c, true);
            continue;
        }
        step(
            &mut c,
            A::ReadRunningCancelAction,
            R::RunningCancelAction(guard == 2),
        );
        if guard == 2 {
            complete(&mut c, true);
            continue;
        }
        step(&mut c, A::CaptureWindow, R::Applied);
        step(
            &mut c,
            A::ReadWindowFullyActive,
            R::WindowFullyActive(false),
        );
        complete(&mut c, true);
    }
}
#[test]
fn non_history_request_skips_group_and_activation_queries_and_consumes_after_reset() {
    let mut c = C::new(false);
    request_guards(&mut c);
    cancel(&mut c, true, false);
    step(&mut c, A::ConsumeHistoryActionActivation, R::Applied);
    complete(&mut c, false);
}
#[test]
fn group_strict_inequality_and_activation_determine_cancelability() {
    // Source: groups < allowed AND history activation, with left short circuit.
    for (groups, allowed, activation, expected) in [
        (0, 1, Some(true), true),
        (0, 1, Some(false), false),
        (1, 2, Some(true), true),
        (1, 2, Some(false), false),
        (1, 1, None, false),
        (2, 1, None, false),
        (usize::MAX, usize::MAX, None, false),
    ] {
        let mut c = C::new(true);
        request_guards(&mut c);
        step(
            &mut c,
            A::ReadManagerGroups,
            R::ManagerGroups {
                groups,
                allowed_groups: NonZeroUsize::new(allowed).unwrap(),
            },
        );
        if let Some(active) = activation {
            step(
                &mut c,
                A::ReadHistoryActionActivation,
                R::HistoryActionActivation(active),
            );
        }
        cancel(&mut c, expected, true);
        step(&mut c, A::ReadActive, R::Active(false));
        complete(&mut c, true);
    }
}
#[test]
fn history_activation_cancel_that_does_not_continue_consumes_activation_and_returns_false() {
    // Request to close steps 6-10 (references/html-close-watcher-cancel-result--877d6146487f.md
    // lines 20-27): with requireHistoryActionActivation true, groups below the
    // allowed number and history-action activation make canPreventClose true;
    // shouldContinue false then consumes the activation and returns false
    // without the close algorithm (no guard re-reads, no membership destruction).
    let mut c = C::new(true);
    request_guards(&mut c);
    step(
        &mut c,
        A::ReadManagerGroups,
        R::ManagerGroups {
            groups: 1,
            allowed_groups: NonZeroUsize::new(2).unwrap(),
        },
    );
    step(
        &mut c,
        A::ReadHistoryActionActivation,
        R::HistoryActionActivation(true),
    );
    cancel_without_continuing(&mut c);
    step(&mut c, A::ConsumeHistoryActionActivation, R::Applied);
    complete(&mut c, false);
}
#[test]
fn cancel_that_does_not_continue_is_refused_when_close_cannot_be_prevented() {
    // Request to close step 10.1 asserts canPreventClose when shouldContinue is
    // false (references/html-close-watcher-cancel-result--877d6146487f.md line
    // 25). Both ways to make canPreventClose false (no history-action
    // activation, group limit reached) therefore refuse a false result, using
    // the corrected cancel-action boolean selected in issue #1225 (the pinned
    // "Cancel action" paragraph's "always false" is the preserved source
    // contradiction); the cursor stays untouched.
    for (groups, allowed, activation) in [(1, 2, Some(false)), (2, 2, None)] {
        let mut c = C::new(true);
        request_guards(&mut c);
        step(
            &mut c,
            A::ReadManagerGroups,
            R::ManagerGroups {
                groups,
                allowed_groups: NonZeroUsize::new(allowed).unwrap(),
            },
        );
        if let Some(active) = activation {
            step(
                &mut c,
                A::ReadHistoryActionActivation,
                R::HistoryActionActivation(active),
            );
        }
        step(&mut c, A::SetRunningCancelAction(true), R::Applied);
        let before = c;
        assert!(
            matches!(c.reply(R::CancelActionResult(false)), Err(ref e) if e.is_noncancelable_action_prevented_close())
        );
        assert_eq!(c, before);
    }
}
#[test]
fn continued_cancel_rechecks_all_close_guards_before_membership_and_close_action() {
    for guard in 0..4 {
        let mut c = C::new(false);
        request_guards(&mut c);
        cancel(&mut c, true, true);
        step(&mut c, A::ReadActive, R::Active(guard != 0));
        if guard == 0 {
            complete(&mut c, true);
            continue;
        }
        step(&mut c, A::ReadEnabled, R::Enabled(guard != 1));
        if guard == 1 {
            complete(&mut c, true);
            continue;
        }
        step(
            &mut c,
            A::ReadWindowFullyActive,
            R::WindowFullyActive(guard != 2),
        );
        if guard == 2 {
            complete(&mut c, true);
            continue;
        }
        step(&mut c, A::DestroyMembership, R::Applied);
        step(&mut c, A::RunCloseAction, R::Applied);
        complete(&mut c, true);
    }
}
#[test]
fn impossible_non_cancelable_result_preserves_exact_pending_cursor() {
    let mut c = C::new(true);
    request_guards(&mut c);
    step(
        &mut c,
        A::ReadManagerGroups,
        R::ManagerGroups {
            groups: 1,
            allowed_groups: NonZeroUsize::new(1).unwrap(),
        },
    );
    step(&mut c, A::SetRunningCancelAction(true), R::Applied);
    let before = c;
    assert!(
        matches!(c.reply(R::CancelActionResult(false)), Err(ref e) if e.is_noncancelable_action_prevented_close())
    );
    assert_eq!(c, before);
    c.reply(R::CancelActionResult(true)).unwrap();
    step(&mut c, A::SetRunningCancelAction(false), R::Applied);
    step(&mut c, A::ReadActive, R::Active(false));
    complete(&mut c, true);
}
#[test]
fn checked_running_state_rejects_duplicate_transitions_without_mutation() {
    let mut state = HtmlCloseWatcherState::fresh();
    let fresh = state;
    assert!(matches!(state.finish_cancel_action(), Err(ref e) if e.is_not_running()));
    assert_eq!(state, fresh);
    state.begin_cancel_action().unwrap();
    assert!(state.running_cancel_action());
    let running = state;
    assert!(matches!(state.begin_cancel_action(), Err(ref e) if e.is_already_running()));
    assert_eq!(state, running);
    // Cursor lifetime is independent: discarding a producer cannot reset state.
    let _ = C::new(false);
    assert_eq!(state, running);
    state.finish_cancel_action().unwrap();
    assert_eq!(state, fresh);
    state.begin_cancel_action().unwrap();
    state.finish_cancel_action().unwrap();
    assert_eq!(state, fresh);
}

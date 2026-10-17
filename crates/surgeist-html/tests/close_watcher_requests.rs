//! Literal source traces from HTML877d raw62026–62069 and selected cancel correction.
use std::num::NonZeroUsize;
use surgeist_html::{
    HtmlCloseWatcherRequestAction as A, HtmlCloseWatcherRequestCursor as C,
    HtmlCloseWatcherRequestError as E, HtmlCloseWatcherRequestReply as R, HtmlCloseWatcherState,
    HtmlCloseWatcherStateError as S,
};

fn step(c: &mut C, action: A, reply: R) {
    assert_eq!(c.action(), action);
    let before = *c;
    assert_eq!(c.reply(R::CancelActionResult(false)), Err(E::WrongReply));
    assert_eq!(*c, before);
    c.reply(reply).unwrap();
}
fn complete(c: &mut C, result: bool) {
    assert_eq!(c.action(), A::Complete(result));
    let before = *c;
    assert_eq!(c.reply(R::Applied), Err(E::Terminal));
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
    assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
    assert_eq!(*c, before);
    c.reply(R::CancelActionResult(result)).unwrap();
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
    assert_eq!(
        c.reply(R::CancelActionResult(false)),
        Err(E::NoncancelableActionPreventedClose)
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
    assert_eq!(state.finish_cancel_action(), Err(S::NotRunning));
    assert_eq!(state, fresh);
    state.begin_cancel_action().unwrap();
    assert!(state.running_cancel_action());
    let running = state;
    assert_eq!(state.begin_cancel_action(), Err(S::AlreadyRunning));
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

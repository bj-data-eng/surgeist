//! Literal HTML877d raw46708–46723 source traces; operands remain adapter-owned.
use surgeist_html::{
    HtmlDialogAction as A, HtmlDialogCursor, HtmlDialogCursorError as E, HtmlDialogOperation,
    HtmlDialogOutcome, HtmlDialogReply as R,
};

fn step(c: &mut HtmlDialogCursor, action: A, reply: R) {
    assert_eq!(c.action(), action);
    let before = *c;
    assert_eq!(c.reply(R::EventCanceled(false)), Err(E::WrongReply));
    assert_eq!(*c, before);
    c.reply(reply).unwrap();
}
fn complete(c: &mut HtmlDialogCursor) {
    assert_eq!(c.action(), A::Complete(HtmlDialogOutcome::Completed));
    let before = *c;
    assert_eq!(c.reply(R::Applied), Err(E::Terminal));
    assert_eq!(*c, before);
}
#[test]
fn closed_disconnected_and_inactive_return_at_their_source_guard() {
    let mut c = HtmlDialogCursor::new(HtmlDialogOperation::RequestClose);
    step(&mut c, A::ReadOpen, R::Open(false));
    complete(&mut c);
    let mut c = HtmlDialogCursor::new(HtmlDialogOperation::RequestClose);
    step(&mut c, A::ReadOpen, R::Open(true));
    step(&mut c, A::ReadConnected, R::Connected(false));
    complete(&mut c);
    let mut c = HtmlDialogCursor::new(HtmlDialogOperation::RequestClose);
    step(&mut c, A::ReadOpen, R::Open(true));
    step(&mut c, A::ReadConnected, R::Connected(true));
    step(&mut c, A::ReadFullyActive, R::FullyActive(false));
    complete(&mut c);
}
#[test]
fn both_normal_watcher_returns_require_literal_final_false() {
    for returned in [false, true] {
        let mut c = HtmlDialogCursor::new(HtmlDialogOperation::RequestClose);
        step(&mut c, A::ReadOpen, R::Open(true));
        step(&mut c, A::ReadConnected, R::Connected(true));
        step(&mut c, A::ReadFullyActive, R::FullyActive(true));
        // Assertion of a real nonnull watcher has no absent-watcher early return.
        assert_eq!(c.action(), A::CaptureCloseWatcher);
        let before = c;
        assert_eq!(c.reply(R::WatcherPresent(false)), Err(E::WrongReply));
        assert_eq!(c, before);
        step(&mut c, A::CaptureCloseWatcher, R::Applied);
        step(&mut c, A::SetForceCloseWatcherEnabled(true), R::Applied);
        step(&mut c, A::WriteRequestCloseReturnValue, R::Applied);
        step(&mut c, A::WriteNullRequestCloseSource, R::Applied);
        assert_eq!(
            c.action(),
            A::RequestWatcherClose {
                require_history_action_activation: false
            }
        );
        let before = c;
        assert_eq!(c.reply(R::Applied), Err(E::WrongReply));
        assert_eq!(c, before);
        step(
            &mut c,
            A::RequestWatcherClose {
                require_history_action_activation: false,
            },
            R::WatcherRequestResult(returned),
        );
        step(&mut c, A::SetForceCloseWatcherEnabled(false), R::Applied);
        complete(&mut c);
    }
}

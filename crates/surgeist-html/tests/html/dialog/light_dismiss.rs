use surgeist_html::{
    attributes::HtmlClosedBy,
    elements::dialog::{
        HtmlCloseWatcherRequestAction as WA, HtmlCloseWatcherRequestReply as WR,
        HtmlDialogLightDismissAction as A, HtmlDialogLightDismissCursor as Cursor,
        HtmlDialogLightDismissKind as Kind, HtmlDialogLightDismissOutcome as O,
        HtmlDialogLightDismissReply as R,
    },
};

fn step(cursor: &mut Cursor, action: A, reply: R) {
    assert_eq!(cursor.action(), action);
    cursor.reply(reply).unwrap();
}

fn open_dialogs(kind: Kind) -> Cursor {
    let mut cursor = Cursor::new(kind);
    step(&mut cursor, A::ReadEventTrusted, R::EventTrusted(true));
    step(
        &mut cursor,
        A::CaptureTargetDocument,
        R::TargetDocumentCaptured,
    );
    step(
        &mut cursor,
        A::ReadOpenDialogsEmpty,
        R::OpenDialogsEmpty(false),
    );
    assert_eq!(cursor.action(), A::CaptureNearestClickedDialog);
    cursor
}

fn after_pointerup_comparison(matches: bool) -> Cursor {
    let mut cursor = open_dialogs(Kind::PointerUp);
    step(
        &mut cursor,
        A::CaptureNearestClickedDialog,
        R::NearestClickedDialogCaptured,
    );
    step(
        &mut cursor,
        A::ComparePointerDownTarget,
        R::PointerDownTargetMatches(matches),
    );
    assert_eq!(cursor.action(), A::ClearPointerDownTarget);
    cursor
}

fn reached_watcher() -> Cursor {
    let mut cursor = after_pointerup_comparison(true);
    step(
        &mut cursor,
        A::ClearPointerDownTarget,
        R::PointerDownTargetCleared,
    );
    step(
        &mut cursor,
        A::CompareLastOpenDialog,
        R::LastOpenDialogMatches(false),
    );
    step(&mut cursor, A::ReadClosedBy, R::ClosedBy(HtmlClosedBy::Any));
    assert_eq!(cursor.action(), A::CaptureCloseWatcher);
    cursor
}

#[test]
fn no_open_dialog_returns_before_nearest_and_pointerdown_waits_for_the_original_nullable_result() {
    for kind in [Kind::PointerDown, Kind::PointerUp] {
        let mut untrusted = Cursor::new(kind);
        step(&mut untrusted, A::ReadEventTrusted, R::EventTrusted(false));
        assert_eq!(untrusted.action(), A::Complete(O::UntrustedEvent));

        let mut empty = Cursor::new(kind);
        step(&mut empty, A::ReadEventTrusted, R::EventTrusted(true));
        step(
            &mut empty,
            A::CaptureTargetDocument,
            R::TargetDocumentCaptured,
        );
        step(
            &mut empty,
            A::ReadOpenDialogsEmpty,
            R::OpenDialogsEmpty(true),
        );
        assert_eq!(empty.action(), A::Complete(O::Finished));
        assert!(
            empty
                .reply(R::NearestClickedDialogCaptured)
                .unwrap_err()
                .is_terminal()
        );
    }

    let mut cursor = open_dialogs(Kind::PointerDown);
    assert!(
        cursor
            .reply(R::PointerDownTargetStored)
            .unwrap_err()
            .is_wrong_reply()
    );
    step(
        &mut cursor,
        A::CaptureNearestClickedDialog,
        R::NearestClickedDialogCaptured,
    );
    step(
        &mut cursor,
        A::StorePointerDownTarget,
        R::PointerDownTargetStored,
    );
    assert_eq!(cursor.action(), A::Complete(O::Finished));
}

#[test]
fn pointerup_clears_first_even_when_targets_differ() {
    let mut cursor = after_pointerup_comparison(false);
    assert!(
        cursor
            .reply(R::LastOpenDialogMatches(false))
            .unwrap_err()
            .is_wrong_reply()
    );
    step(
        &mut cursor,
        A::ClearPointerDownTarget,
        R::PointerDownTargetCleared,
    );
    assert_eq!(cursor.action(), A::Complete(O::Finished));
}

#[test]
fn matching_pointer_targets_reach_only_the_last_open_dialog_and_require_any() {
    let mut inside_last = after_pointerup_comparison(true);
    step(
        &mut inside_last,
        A::ClearPointerDownTarget,
        R::PointerDownTargetCleared,
    );
    step(
        &mut inside_last,
        A::CompareLastOpenDialog,
        R::LastOpenDialogMatches(true),
    );
    assert_eq!(inside_last.action(), A::Complete(O::Finished));

    for closed_by in [HtmlClosedBy::None, HtmlClosedBy::CloseRequest] {
        let mut outside = after_pointerup_comparison(true);
        step(
            &mut outside,
            A::ClearPointerDownTarget,
            R::PointerDownTargetCleared,
        );
        step(
            &mut outside,
            A::CompareLastOpenDialog,
            R::LastOpenDialogMatches(false),
        );
        step(&mut outside, A::ReadClosedBy, R::ClosedBy(closed_by));
        assert_eq!(outside.action(), A::Complete(O::Finished));
    }
}

#[test]
fn reached_absent_watcher_is_a_terminal_invariant_outcome() {
    let mut cursor = reached_watcher();
    assert!(
        cursor
            .reply(R::CloseWatcher(WR::Active(true)))
            .unwrap_err()
            .is_wrong_reply()
    );
    step(
        &mut cursor,
        A::CaptureCloseWatcher,
        R::CloseWatcherPresent(false),
    );
    assert_eq!(cursor.action(), A::Complete(O::MissingCloseWatcher));
}

#[test]
fn admitted_watcher_runs_the_original_request_without_requiring_history_activation() {
    for should_continue in [false, true] {
        let mut cursor = reached_watcher();
        step(
            &mut cursor,
            A::CaptureCloseWatcher,
            R::CloseWatcherPresent(true),
        );
        let wrong = cursor
            .reply(R::CloseWatcher(WR::Enabled(true)))
            .unwrap_err();
        assert!(wrong.close_watcher_error().unwrap().is_wrong_reply());
        let mut watcher =
            |action, reply| step(&mut cursor, A::CloseWatcher(action), R::CloseWatcher(reply));
        watcher(WA::ReadActive, WR::Active(true));
        watcher(WA::ReadEnabled, WR::Enabled(true));
        watcher(WA::ReadRunningCancelAction, WR::RunningCancelAction(false));
        watcher(WA::CaptureWindow, WR::Applied);
        watcher(WA::ReadWindowFullyActive, WR::WindowFullyActive(true));
        watcher(WA::SetRunningCancelAction(true), WR::Applied);
        watcher(
            WA::RunCancelAction {
                can_prevent_close: true,
            },
            WR::CancelActionResult(should_continue),
        );
        watcher(WA::SetRunningCancelAction(false), WR::Applied);
        if should_continue {
            watcher(WA::ReadActive, WR::Active(true));
            watcher(WA::ReadEnabled, WR::Enabled(true));
            watcher(WA::ReadWindowFullyActive, WR::WindowFullyActive(true));
            watcher(WA::DestroyMembership, WR::Applied);
            watcher(WA::RunCloseAction, WR::Applied);
        } else {
            watcher(WA::ConsumeHistoryActionActivation, WR::Applied);
        }
        assert_eq!(
            cursor.action(),
            A::Complete(O::WatcherFinished { should_continue })
        );
    }
}

//! Literal independent HTML877d Dialog focusing traces, raw46762–46782.
use surgeist_html::{
    HtmlDialogAction, HtmlDialogCursor, HtmlDialogFocusControl as Control,
    HtmlDialogFocusingAction as A, HtmlDialogFocusingCursor as Cursor,
    HtmlDialogFocusingError as Error, HtmlDialogFocusingReply as R, HtmlDialogOperation,
    HtmlDialogReply, HtmlDialogSubjectFocusFacts as Facts,
    HtmlDialogSubjectFocusingDecision as Decision, dialog_subject_focusing_decision,
};

fn step(c: &mut Cursor, action: A, reply: R) {
    assert_eq!(c.action(), action);
    let before = *c;
    let wrong = if action == A::ReadAllowFocus {
        R::Applied
    } else {
        R::AllowFocus(false)
    };
    assert_eq!(c.reply(wrong), Err(Error::WrongReply));
    assert_eq!(*c, before);
    c.reply(reply).unwrap();
}
fn complete(c: &mut Cursor) {
    assert_eq!(c.action(), A::Complete);
    let before = *c;
    for reply in [R::Applied, R::AllowFocus(true), R::SameOrigin(false)] {
        assert_eq!(c.reply(reply), Err(Error::Terminal));
        assert_eq!(*c, before);
    }
}
fn selected_tail(c: &mut Cursor, control: Control, same_origin: bool) {
    step(c, A::RunFocusingSteps { control }, R::Applied);
    step(c, A::CaptureControlTopDocument { control }, R::Applied);
    step(c, A::ReadSameOrigin { control }, R::SameOrigin(same_origin));
    if same_origin {
        step(c, A::EmptyAutofocusCandidates, R::Applied);
        step(c, A::SetAutofocusProcessed, R::Applied);
    }
    complete(c);
}
#[test]
fn denied_allow_focus_returns_before_autofocus_or_delegate_reads() {
    let mut c = Cursor::new();
    step(&mut c, A::ReadAllowFocus, R::AllowFocus(false));
    complete(&mut c);
}
#[test]
fn autofocus_selects_subject_without_delegate_query_and_runs_complete_origin_tail() {
    for same_origin in [false, true] {
        let mut c = Cursor::new();
        step(&mut c, A::ReadAllowFocus, R::AllowFocus(true));
        step(&mut c, A::ReadAutofocus, R::Autofocus(true));
        selected_tail(&mut c, Control::Subject, same_origin);
    }
}
#[test]
fn absent_autofocus_selects_actual_delegate_or_subject_fallback_and_keeps_that_control() {
    for (present, control) in [(true, Control::Delegate), (false, Control::Subject)] {
        for same_origin in [false, true] {
            let mut c = Cursor::new();
            step(&mut c, A::ReadAllowFocus, R::AllowFocus(true));
            step(&mut c, A::ReadAutofocus, R::Autofocus(false));
            step(
                &mut c,
                A::FindFocusDelegate,
                R::FocusDelegatePresent(present),
            );
            selected_tail(&mut c, control, same_origin);
        }
    }
}
#[test]
fn future_stage_and_applied_instead_of_known_observations_cannot_advance() {
    let mut c = Cursor::new();
    for reply in [
        R::Autofocus(true),
        R::FocusDelegatePresent(true),
        R::SameOrigin(true),
        R::Applied,
    ] {
        let before = c;
        assert_eq!(c.reply(reply), Err(Error::WrongReply));
        assert_eq!(c, before);
    }
    step(&mut c, A::ReadAllowFocus, R::AllowFocus(true));
    for reply in [
        R::FocusDelegatePresent(false),
        R::SameOrigin(false),
        R::Applied,
    ] {
        let before = c;
        assert_eq!(c.reply(reply), Err(Error::WrongReply));
        assert_eq!(c, before);
    }
    step(&mut c, A::ReadAutofocus, R::Autofocus(false));
    for reply in [R::Autofocus(true), R::SameOrigin(true), R::Applied] {
        let before = c;
        assert_eq!(c.reply(reply), Err(Error::WrongReply));
        assert_eq!(c, before);
    }
    step(&mut c, A::FindFocusDelegate, R::FocusDelegatePresent(true));
    step(
        &mut c,
        A::RunFocusingSteps {
            control: Control::Delegate,
        },
        R::Applied,
    );
    step(
        &mut c,
        A::CaptureControlTopDocument {
            control: Control::Delegate,
        },
        R::Applied,
    );
    let before = c;
    assert_eq!(c.reply(R::Applied), Err(Error::WrongReply));
    assert_eq!(c, before);
    step(
        &mut c,
        A::ReadSameOrigin {
            control: Control::Delegate,
        },
        R::SameOrigin(false),
    );
    complete(&mut c);
}
#[test]
fn own_inert_subject_proof_requires_known_nonroot_and_no_delegating_root() {
    assert_eq!(
        dialog_subject_focusing_decision(Facts {
            own_inert: Some(true),
            is_document_element: Some(false),
            has_delegating_focus_root: Some(false),
        }),
        Decision::NoChange
    );
    // Independent source exclusions: unknown/absent own inertness, viewport
    // resolution for a Document element, and delegated-root resolution must
    // each retain the actual focusing requirement. Ancestor inertness is not
    // an input attesting the subject's own inert attribute.
    for facts in [
        Facts {
            own_inert: None,
            is_document_element: Some(false),
            has_delegating_focus_root: Some(false),
        },
        Facts {
            own_inert: Some(false),
            is_document_element: Some(false),
            has_delegating_focus_root: Some(false),
        },
        Facts {
            own_inert: Some(true),
            is_document_element: None,
            has_delegating_focus_root: Some(false),
        },
        Facts {
            own_inert: Some(true),
            is_document_element: Some(true),
            has_delegating_focus_root: Some(false),
        },
        Facts {
            own_inert: Some(true),
            is_document_element: Some(false),
            has_delegating_focus_root: None,
        },
        Facts {
            own_inert: Some(true),
            is_document_element: Some(false),
            has_delegating_focus_root: Some(true),
        },
        Facts {
            own_inert: None,
            is_document_element: None,
            has_delegating_focus_root: None,
        },
    ] {
        assert_eq!(
            dialog_subject_focusing_decision(facts),
            Decision::NeedsActualFocusing
        );
    }
}
#[test]
fn subject_called_focusing_no_change_does_not_skip_dialog_autofocus_tail_or_complete_parent() {
    let mut parent = HtmlDialogCursor::new(HtmlDialogOperation::Show);
    parent.reply(HtmlDialogReply::Open(false)).unwrap();
    parent.reply(HtmlDialogReply::EventCanceled(false)).unwrap();
    parent.reply(HtmlDialogReply::Open(false)).unwrap();
    for action in [
        HtmlDialogAction::QueueToggle {
            old: surgeist_html::HtmlDialogState::Closed,
            new: surgeist_html::HtmlDialogState::Open,
            source: surgeist_html::HtmlDialogEventSource::Null,
        },
        HtmlDialogAction::AddOpen,
        HtmlDialogAction::RememberFocusedElement,
        HtmlDialogAction::CaptureNodeDocument,
        HtmlDialogAction::FindTopmostPopoverAncestor,
        HtmlDialogAction::HidePopoversUntil {
            focus_previous_element: false,
            fire_events: true,
        },
    ] {
        assert_eq!(parent.action(), action);
        parent.reply(HtmlDialogReply::Applied).unwrap();
    }
    let mut child = Cursor::new();
    step(&mut child, A::ReadAllowFocus, R::AllowFocus(true));
    step(&mut child, A::ReadAutofocus, R::Autofocus(true));
    assert_eq!(
        dialog_subject_focusing_decision(Facts {
            own_inert: Some(true),
            is_document_element: Some(false),
            has_delegating_focus_root: Some(false),
        }),
        Decision::NoChange
    );
    step(
        &mut child,
        A::RunFocusingSteps {
            control: Control::Subject,
        },
        R::Applied,
    );
    assert_eq!(
        child.action(),
        A::CaptureControlTopDocument {
            control: Control::Subject
        }
    );
    assert_eq!(parent.action(), HtmlDialogAction::RunDialogFocusingSteps);
    step(
        &mut child,
        A::CaptureControlTopDocument {
            control: Control::Subject,
        },
        R::Applied,
    );
    step(
        &mut child,
        A::ReadSameOrigin {
            control: Control::Subject,
        },
        R::SameOrigin(true),
    );
    step(&mut child, A::EmptyAutofocusCandidates, R::Applied);
    step(&mut child, A::SetAutofocusProcessed, R::Applied);
    complete(&mut child);
    assert_eq!(parent.action(), HtmlDialogAction::RunDialogFocusingSteps);
    parent.reply(HtmlDialogReply::Applied).unwrap();
    assert_eq!(
        parent.action(),
        HtmlDialogAction::Complete(surgeist_html::HtmlDialogOutcome::Completed)
    );
}

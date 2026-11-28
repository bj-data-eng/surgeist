use surgeist_html::{
    bounded::select::{
        HtmlSelectNotificationAction as Action, HtmlSelectNotificationEvent as Event,
        HtmlSelectNotificationReply as Reply, HtmlSelectNotificationReplyError as Error,
        HtmlSelectUserToggleState as State, HtmlSelectUserToggleTarget as Target,
        toggle_select_option_for_user,
    },
    elements::form::{HtmlOptionFlags, HtmlSelectFlags},
};

#[test]
fn enabled_multiple_select_toggles_both_directions_and_notifies_once() {
    for selected in [false, true] {
        for dirty in [false, true] {
            let mut option = HtmlOptionFlags::new(selected, dirty);
            let task = toggle_select_option_for_user(
                State::EnabledMultiple,
                Target::EnabledMember(&mut option),
            )
            .unwrap();
            assert_eq!(option.selectedness(), !selected);
            assert!(option.dirtiness());
            assert_eq!(task.action(), Some(Action::MarkUserValidity));
        }
    }
}

#[test]
fn single_disabled_and_outside_toggles_leave_flags_unchanged_and_create_no_task() {
    let initial = HtmlOptionFlags::new(true, false);
    let mut option = initial;
    for state in [State::Single, State::DisabledMultiple] {
        assert!(toggle_select_option_for_user(state, Target::EnabledMember(&mut option)).is_none());
        assert_eq!(option, initial);
    }
    assert!(
        toggle_select_option_for_user(State::EnabledMultiple, Target::DisabledMember).is_none()
    );
    assert!(toggle_select_option_for_user(State::EnabledMultiple, Target::OutsideList).is_none());
}

#[test]
fn queued_select_notification_waits_for_each_effect_then_dispatches_input_before_change() {
    let mut option = HtmlOptionFlags::default();
    let mut select = HtmlSelectFlags::default();
    let mut task =
        toggle_select_option_for_user(State::EnabledMultiple, Target::EnabledMember(&mut option))
            .unwrap();
    assert!(!select.user_validity());
    assert_eq!(task.action(), Some(Action::MarkUserValidity));
    assert_eq!(
        task.accept(Reply::FallbackLabelUpdated),
        Err(Error::WrongAction)
    );
    assert_eq!(task.action(), Some(Action::MarkUserValidity));

    select.mark_user_validity();
    task.accept(Reply::UserValiditySet).unwrap();
    assert!(select.user_validity());
    assert_eq!(task.action(), Some(Action::UpdateSelectedcontent));
    assert_eq!(task.accept(Reply::UserValiditySet), Err(Error::WrongAction));

    task.accept(Reply::SelectedcontentUpdated).unwrap();
    assert_eq!(task.action(), Some(Action::UpdateFallbackLabel));
    assert_eq!(
        task.accept(Reply::Dispatched(Event::Input)),
        Err(Error::WrongAction)
    );

    task.accept(Reply::FallbackLabelUpdated).unwrap();
    assert_eq!(task.action(), Some(Action::Dispatch(Event::Input)));
    assert_eq!(
        task.accept(Reply::Dispatched(Event::Change)),
        Err(Error::WrongAction)
    );
    task.accept(Reply::Dispatched(Event::Input)).unwrap();
    assert_eq!(task.action(), Some(Action::Dispatch(Event::Change)));
    assert_eq!(
        task.accept(Reply::Dispatched(Event::Input)),
        Err(Error::WrongAction)
    );
    task.accept(Reply::Dispatched(Event::Change)).unwrap();
    assert_eq!(task.action(), None);
    assert_eq!(
        task.accept(Reply::Dispatched(Event::Change)),
        Err(Error::Complete)
    );

    for (event, composed) in [(Event::Input, true), (Event::Change, false)] {
        assert!(event.bubbles());
        assert!(!event.cancelable());
        assert_eq!(event.composed(), composed);
    }
    select.reset();
    assert!(!select.user_validity());
}

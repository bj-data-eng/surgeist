//! Literal expectations from HTML 877d textarea/select/option flag algorithms.
//! These are local transitions, not substitutes for tree or event execution.
use surgeist_html::{
    HtmlOptionFlags, HtmlSelectFlags, HtmlTextareaFlags, HtmlTextareaRawValueAction,
};

#[test]
fn fresh_family_slots_are_false() {
    let textarea = HtmlTextareaFlags::default();
    assert!(!textarea.dirty_value());
    assert!(!textarea.user_validity());
    assert!(!HtmlSelectFlags::default().user_validity());
    let option = HtmlOptionFlags::default();
    assert!(!option.selectedness());
    assert!(!option.dirtiness());
}

#[test]
fn textarea_slots_are_independent_native_state() {
    for (dirty, validity) in [(false, false), (false, true), (true, false), (true, true)] {
        let flags = HtmlTextareaFlags::new(dirty, validity);
        assert_eq!(
            (flags.dirty_value(), flags.user_validity()),
            (dirty, validity)
        );
    }
}

#[test]
fn children_changes_follow_dirty_value_not_user_validity() {
    for validity in [false, true] {
        assert_eq!(
            HtmlTextareaFlags::new(false, validity).children_changed(),
            HtmlTextareaRawValueAction::ReplaceWithChildText
        );
        assert_eq!(
            HtmlTextareaFlags::new(true, validity).children_changed(),
            HtmlTextareaRawValueAction::Keep
        );
    }
}

#[test]
fn textarea_reset_clears_both_slots_and_requires_child_text() {
    let mut flags = HtmlTextareaFlags::new(true, true);
    assert_eq!(
        flags.reset(),
        HtmlTextareaRawValueAction::ReplaceWithChildText
    );
    assert_eq!((flags.dirty_value(), flags.user_validity()), (false, false));
    assert_eq!(
        flags.reset(),
        HtmlTextareaRawValueAction::ReplaceWithChildText
    );
}

#[test]
fn textarea_dirty_write_preserves_user_validity() {
    for validity in [false, true] {
        let mut flags = HtmlTextareaFlags::new(false, validity);
        flags.mark_dirty_value();
        assert_eq!(
            (flags.dirty_value(), flags.user_validity()),
            (true, validity)
        );
        assert_eq!(flags.children_changed(), HtmlTextareaRawValueAction::Keep);
    }
}

#[test]
fn textarea_clone_copies_dirty_without_copying_user_validity() {
    let source = HtmlTextareaFlags::new(true, true);
    let mut fresh_copy = HtmlTextareaFlags::default();
    fresh_copy.clone_dirty_value_from(&source);
    assert_eq!(
        (fresh_copy.dirty_value(), fresh_copy.user_validity()),
        (true, false)
    );
    let mut existing_copy = HtmlTextareaFlags::new(true, true);
    existing_copy.clone_dirty_value_from(&HtmlTextareaFlags::new(false, false));
    assert_eq!(
        (existing_copy.dirty_value(), existing_copy.user_validity()),
        (false, true)
    );
}

#[test]
fn select_reset_clears_established_user_validity() {
    let mut flags = HtmlSelectFlags::new(true);
    assert!(flags.user_validity());
    flags.reset();
    assert!(!flags.user_validity());
    flags.reset();
    assert_eq!(flags, HtmlSelectFlags::default());
}

#[test]
fn selected_attribute_added_then_removed_changes_clean_option_only() {
    let mut flags = HtmlOptionFlags::default();
    assert!(flags.selected_attribute_changed(true));
    assert!(flags.selectedness());
    assert!(!flags.dirtiness());
    assert!(!flags.selected_attribute_changed(true));
    assert!(flags.selected_attribute_changed(false));
    assert!(!flags.selectedness());
    assert!(!flags.selected_attribute_changed(false));
}

#[test]
fn dirty_option_ignores_selected_attribute_even_when_it_differs() {
    for (selected, present) in [(false, true), (true, false)] {
        let mut flags = HtmlOptionFlags::new(selected, true);
        assert!(!flags.selected_attribute_changed(present));
        assert_eq!((flags.selectedness(), flags.dirtiness()), (selected, true));
    }
}

#[test]
fn native_selected_setter_always_makes_dirty() {
    let mut flags = HtmlOptionFlags::default();
    flags.selected_setter(false);
    assert_eq!((flags.selectedness(), flags.dirtiness()), (false, true));
    flags.selected_setter(true);
    assert_eq!((flags.selectedness(), flags.dirtiness()), (true, true));
    assert!(!flags.selected_attribute_changed(false));
}

#[test]
fn option_reset_uses_actual_selected_attribute_and_clears_dirtiness() {
    let mut flags = HtmlOptionFlags::new(true, true);
    flags.reset(false);
    assert_eq!((flags.selectedness(), flags.dirtiness()), (false, false));
    flags.selected_setter(false);
    flags.reset(true);
    assert_eq!((flags.selectedness(), flags.dirtiness()), (true, false));
}

#[test]
fn select_algorithm_selectedness_write_never_clears_dirtiness() {
    for dirty in [false, true] {
        let mut flags = HtmlOptionFlags::new(true, dirty);
        flags.set_selectedness_for_select(false);
        assert_eq!((flags.selectedness(), flags.dirtiness()), (false, dirty));
        flags.set_selectedness_for_select(true);
        assert_eq!((flags.selectedness(), flags.dirtiness()), (true, dirty));
    }
}

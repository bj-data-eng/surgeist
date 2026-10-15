//! Literal HTML 877d oracles: #selectedness-setting-algorithm, select reset,
//! #ask-for-a-reset and #the-selectedcontent-element. The adapter applies real
//! public flags and records state observed by each prescribed refresh.
use surgeist_html::bounded::*;
use surgeist_html::{HtmlOptionFlags, HtmlSelectFlags, KnownHtmlElement as E};

#[derive(Debug, Eq, PartialEq)]
struct Outcome {
    options: Vec<(bool, bool)>,
    validity: bool,
    refreshes: Vec<Vec<(bool, bool)>>,
    normalized: bool,
}
fn run(
    operation: HtmlSelectOperation,
    multiple: bool,
    flags: &[(bool, bool)],
    defaults: &[bool],
    disabled: &[bool],
    size_one: bool,
) -> Outcome {
    let mut options: Vec<_> = flags
        .iter()
        .map(|&(s, d)| HtmlOptionFlags::new(s, d))
        .collect();
    let mut select = HtmlSelectFlags::new(true);
    let mut cursor = HtmlSelectCursor::new(operation, multiple, options.len()).unwrap();
    let mut refreshes = Vec::new();
    let normalized = loop {
        let reply = match cursor.action() {
            HtmlSelectAction::ResetSelectFlags => {
                select.reset();
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ResetOption { ordinal } => {
                options[ordinal].reset(defaults[ordinal]);
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ReadSelectedness { ordinal } => {
                HtmlSelectReply::Selectedness(options[ordinal].selectedness())
            }
            HtmlSelectAction::WriteSelectedness { ordinal, selected } => {
                options[ordinal].set_selectedness_for_select(selected);
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::ReadDisabled { ordinal } => {
                HtmlSelectReply::Disabled(disabled[ordinal])
            }
            HtmlSelectAction::ReadDisplaySize => HtmlSelectReply::DisplaySizeIsOne(size_one),
            HtmlSelectAction::UpdateSelectedcontent => {
                if !multiple {
                    refreshes.push(
                        options
                            .iter()
                            .map(|f| (f.selectedness(), f.dirtiness()))
                            .collect(),
                    );
                }
                HtmlSelectReply::Applied
            }
            HtmlSelectAction::Complete {
                normalization_changed,
            } => break normalization_changed,
        };
        cursor.reply(reply).unwrap();
    };
    Outcome {
        options: options
            .iter()
            .map(|f| (f.selectedness(), f.dirtiness()))
            .collect(),
        validity: select.user_validity(),
        refreshes,
        normalized,
    }
}

#[test]
fn ordinary_normalization_keeps_last_selected_even_if_disabled_and_preserves_dirtiness() {
    let result = run(
        HtmlSelectOperation::Normalize,
        false,
        &[(true, true), (false, false), (true, false), (true, true)],
        &[],
        &[false, true, false, true],
        true,
    );
    assert_eq!(
        result,
        Outcome {
            options: vec![(false, true), (false, false), (false, false), (true, true)],
            validity: true,
            refreshes: vec![],
            normalized: true,
        }
    );
    // A multiple select consumes none of the absent supplemental observations.
    let result = run(
        HtmlSelectOperation::Normalize,
        true,
        &[(true, true), (true, false)],
        &[],
        &[],
        false,
    );
    assert_eq!(result.options, [(true, true), (true, false)]);
    assert!(!result.normalized);
}

#[test]
fn only_size_one_with_no_selection_selects_first_enabled_and_keeps_dirty_flags() {
    let flags = [(false, true), (false, false), (false, true)];
    let enabled = run(
        HtmlSelectOperation::Normalize,
        false,
        &flags,
        &[],
        &[true, false, false],
        true,
    );
    assert_eq!(
        enabled.options,
        [(false, true), (true, false), (false, true)]
    );
    assert!(enabled.normalized);
    let large = run(
        HtmlSelectOperation::Normalize,
        false,
        &flags,
        &[],
        &[true, false, false],
        false,
    );
    assert_eq!(large.options, flags);
    assert!(!large.normalized);
    let disabled = run(
        HtmlSelectOperation::Normalize,
        false,
        &flags,
        &[],
        &[true, true, true],
        true,
    );
    assert_eq!(disabled.options, flags);
    assert!(!disabled.normalized);
}

#[test]
fn removing_multiple_keeps_first_selection_before_normalization_and_refreshes_final_state() {
    let result = run(
        HtmlSelectOperation::MultipleRemoved,
        false,
        &[(true, true), (true, false), (true, true)],
        &[],
        &[true, false, false],
        true,
    );
    assert_eq!(
        result.options,
        [(true, true), (false, false), (false, true)]
    );
    assert_eq!(
        result.refreshes,
        [vec![(true, true), (false, false), (false, true)]]
    );
    assert!(!result.normalized); // Changes occurred in the distinct first-wins pass.
}

#[test]
fn reset_clears_validity_and_all_dirty_flags_before_normalization_and_refresh() {
    let result = run(
        HtmlSelectOperation::ResetControl,
        false,
        &[(false, true), (true, true), (false, true)],
        &[true, false, true],
        &[false, false, true],
        true,
    );
    assert_eq!(
        result,
        Outcome {
            options: vec![(false, false), (false, false), (true, false)],
            validity: false,
            refreshes: vec![vec![(false, false), (false, false), (true, false)]],
            normalized: true,
        }
    );
    let multiple = run(
        HtmlSelectOperation::ResetControl,
        true,
        &[(true, true), (false, true)],
        &[false, true],
        &[],
        false,
    );
    assert_eq!(multiple.options, [(false, false), (true, false)]);
    assert!(!multiple.validity);
    assert!(multiple.refreshes.is_empty());
}

#[test]
fn caller_change_and_ask_for_reset_refresh_even_when_normalization_is_a_noop() {
    for operation in [
        HtmlSelectOperation::ResetSelectedness { has_changed: true },
        HtmlSelectOperation::AskForReset,
    ] {
        let result = run(operation, false, &[(true, true)], &[], &[false], true);
        assert_eq!(result.options, [(true, true)]);
        assert_eq!(result.refreshes, [vec![(true, true)]]);
        assert!(!result.normalized);
    }
    let noop = run(
        HtmlSelectOperation::ResetSelectedness { has_changed: false },
        false,
        &[(true, true)],
        &[],
        &[true],
        true,
    );
    assert!(noop.refreshes.is_empty());
    let empty = run(HtmlSelectOperation::AskForReset, false, &[], &[], &[], true);
    assert_eq!(empty.refreshes, [vec![]]);
    let changed = run(
        HtmlSelectOperation::ResetSelectedness { has_changed: false },
        false,
        &[(true, true), (true, false)],
        &[],
        &[false],
        true,
    );
    assert_eq!(changed.refreshes, [vec![(false, true), (true, false)]]);
    let added = run(
        HtmlSelectOperation::MultipleAdded,
        true,
        &[(true, true), (true, false)],
        &[],
        &[],
        true,
    );
    assert_eq!(added.options, [(true, true), (true, false)]);
    assert!(added.refreshes.is_empty());
}

#[test]
fn dirty_attribute_edits_and_repeated_selected_setter_keep_their_distinct_local_effects() {
    let mut flags = HtmlOptionFlags::new(true, true);
    assert!(!flags.selected_attribute_changed(false));
    let after_attribute = run(
        HtmlSelectOperation::ResetSelectedness { has_changed: false },
        false,
        &[(flags.selectedness(), flags.dirtiness())],
        &[],
        &[false],
        true,
    );
    assert_eq!(after_attribute.options, [(true, true)]);
    assert!(after_attribute.refreshes.is_empty());
    flags.selected_setter(true);
    let after_setter = run(
        HtmlSelectOperation::AskForReset,
        false,
        &[(flags.selectedness(), flags.dirtiness())],
        &[],
        &[false],
        true,
    );
    assert_eq!(after_setter.refreshes, [vec![(true, true)]]);
}

#[test]
fn incomplete_selection_never_clears_selectedcontent_and_first_true_stops_later_reads() {
    let mut cursor = HtmlSelectFirstSelectedCursor::new(3);
    cursor.reply(Some(false)).unwrap();
    cursor.reply(None).unwrap();
    assert_eq!(
        cursor.action(),
        HtmlSelectFirstSelectedAction::Complete(HtmlSelectFirstSelection::Unavailable)
    );
    assert_eq!(
        selectedcontent_update(false, false, HtmlSelectFirstSelection::Unavailable),
        HtmlSelectedcontentUpdate::Unavailable
    );
    assert_eq!(
        selectedcontent_update(false, false, HtmlSelectFirstSelection::NoSelection),
        HtmlSelectedcontentUpdate::Clear
    );
    for (multiple, disabled) in [(true, false), (false, true)] {
        assert_eq!(
            selectedcontent_update(multiple, disabled, HtmlSelectFirstSelection::Unavailable),
            HtmlSelectedcontentUpdate::Skip
        );
    }
    let mut cursor = HtmlSelectFirstSelectedCursor::new(3);
    cursor.reply(Some(false)).unwrap();
    cursor.reply(Some(true)).unwrap();
    let HtmlSelectFirstSelectedAction::Complete(selection) = cursor.action() else {
        panic!("first true is complete")
    };
    assert_eq!(
        selectedcontent_update(false, false, selection),
        HtmlSelectedcontentUpdate::CloneOptionChildren { ordinal: 1 }
    );
    assert_eq!(cursor.reply(None), Err(HtmlSelectCursorError::Reply));
    assert_eq!(
        HtmlSelectFirstSelectedCursor::new(0).action(),
        HtmlSelectFirstSelectedAction::Complete(HtmlSelectFirstSelection::NoSelection)
    );
}

#[test]
fn selectedcontent_ancestors_distinguish_nearest_select_and_nested_disabling_boundaries() {
    let mut state = HtmlSelectedcontentAncestors::default();
    assert_eq!(
        state.observe(Some(E::Button)),
        HtmlSelectedcontentAncestorAction::Continue
    );
    assert_eq!(
        state.observe(Some(E::Select)),
        HtmlSelectedcontentAncestorAction::RememberNearestSelect
    );
    assert!(!state.disabled());
    assert!(state.has_nearest_select());
    assert_eq!(
        state.observe(Some(E::Select)),
        HtmlSelectedcontentAncestorAction::StopDisabled
    );
    assert!(state.disabled());
    for barrier in [E::Option, E::Selectedcontent] {
        let mut state = HtmlSelectedcontentAncestors::default();
        assert_eq!(
            state.observe(Some(barrier)),
            HtmlSelectedcontentAncestorAction::StopDisabled
        );
        assert!(!state.has_nearest_select());
        assert!(state.disabled());
    }
}

#[test]
fn invalid_protocol_replies_and_multiple_context_preserve_the_pending_operation() {
    assert_eq!(
        HtmlSelectCursor::new(HtmlSelectOperation::MultipleRemoved, true, 2).unwrap_err(),
        HtmlSelectCursorError::MultipleContext
    );
    let mut cursor = HtmlSelectCursor::new(HtmlSelectOperation::Normalize, false, 2).unwrap();
    let pending = cursor.action();
    assert_eq!(
        cursor.reply(HtmlSelectReply::Applied),
        Err(HtmlSelectCursorError::Reply)
    );
    assert_eq!(cursor.action(), pending);
    cursor.reply(HtmlSelectReply::Selectedness(true)).unwrap();
    cursor.reply(HtmlSelectReply::Disabled(false)).unwrap();
    cursor.reply(HtmlSelectReply::Selectedness(false)).unwrap();
    assert_eq!(
        cursor.action(),
        HtmlSelectAction::Complete {
            normalization_changed: false
        }
    );
    assert_eq!(
        cursor.reply(HtmlSelectReply::Applied),
        Err(HtmlSelectCursorError::Reply)
    );
}

#[test]
fn option_language_barriers_preserve_first_enabled_fallback_and_prune_nested_lists() {
    // The actual nearest ancestor relation is supplied by the adapter. An
    // outer optgroup cannot disable an option across a select barrier.
    let disabled = |ancestors: &[(E, bool)]| {
        for &(kind, present) in ancestors {
            match option_disabled_ancestor(Some(kind)) {
                HtmlOptionDisabledAncestor::Continue => {}
                HtmlOptionDisabledAncestor::Enabled => return false,
                HtmlOptionDisabledAncestor::ReadOptgroupDisabled => return present,
            }
        }
        false
    };
    let states = [
        disabled(&[(E::Div, false), (E::Optgroup, true)]),
        disabled(&[(E::Select, false), (E::Optgroup, true)]),
    ];
    assert_eq!(states, [true, false]);
    let result = run(
        HtmlSelectOperation::Normalize,
        false,
        &[(false, true), (false, false)],
        &[],
        &states,
        true,
    );
    assert_eq!(result.options, [(false, true), (true, false)]);
    assert_eq!(
        select_option_rule(Some(E::Option), true),
        HtmlSelectOptionRule::IncludeAndPrune
    );
    assert_eq!(
        select_option_rule(Some(E::Optgroup), false),
        HtmlSelectOptionRule::Descend
    );
    assert_eq!(
        select_option_rule(Some(E::Optgroup), true),
        HtmlSelectOptionRule::Prune
    );
    for kind in [E::Select, E::Hr, E::Datalist] {
        assert_eq!(
            select_option_rule(Some(kind), false),
            HtmlSelectOptionRule::Prune
        );
    }
    assert_eq!(
        select_option_rule(None, true),
        HtmlSelectOptionRule::Descend
    );
    let mut outside = HtmlSelectedcontentAncestors::default();
    outside.observe(Some(E::Div));
    outside.observe(None); // Established foreign/name lookalike, not unknown ancestry.
    assert!(!outside.disabled());
    assert!(!outside.has_nearest_select());
}

#[test]
fn option_cached_select_uses_nearest_select_with_only_one_optgroup_and_exact_barriers() {
    let mut single = HtmlOptionSelectAncestors::default();
    assert_eq!(
        single.observe(Some(E::Div)),
        HtmlOptionSelectAncestorAction::Continue
    );
    assert_eq!(
        single.observe(Some(E::Optgroup)),
        HtmlOptionSelectAncestorAction::Continue
    );
    assert_eq!(
        single.observe(Some(E::Select)),
        HtmlOptionSelectAncestorAction::NearestSelect
    );
    assert_eq!(
        single.finish_complete(),
        HtmlOptionSelectAncestorAction::NearestSelect
    );
    let mut nested = HtmlOptionSelectAncestors::default();
    nested.observe(Some(E::Optgroup));
    assert_eq!(
        nested.observe(Some(E::Optgroup)),
        HtmlOptionSelectAncestorAction::NoSelect
    );
    assert_eq!(
        nested.observe(Some(E::Select)),
        HtmlOptionSelectAncestorAction::NoSelect
    );
    for barrier in [E::Datalist, E::Hr, E::Option] {
        let mut cursor = HtmlOptionSelectAncestors::default();
        assert_eq!(
            cursor.observe(Some(barrier)),
            HtmlOptionSelectAncestorAction::NoSelect
        );
        assert_eq!(
            cursor.observe(Some(E::Select)),
            HtmlOptionSelectAncestorAction::NoSelect
        );
    }
    let mut outside = HtmlOptionSelectAncestors::default();
    outside.observe(None);
    assert_eq!(
        outside.finish_complete(),
        HtmlOptionSelectAncestorAction::NoSelect
    );
}

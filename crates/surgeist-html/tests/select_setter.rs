//! Literal Select setter outcomes from HTML877d40372–40385 and40411–40423.
use surgeist_html::HtmlOptionFlags;
use surgeist_html::bounded::{
    HtmlSelectCursorError, HtmlSelectSetterAction as Action, HtmlSelectSetterCursor as Cursor,
    HtmlSelectSetterReply as Reply,
};

#[derive(Debug, Eq, PartialEq)]
struct Outcome {
    flags: Vec<(bool, bool)>,
    actions: Vec<Action>,
    before_selection: Vec<Vec<(bool, bool)>>,
    refreshes: Vec<Vec<(bool, bool)>>,
}

fn flags(options: &[HtmlOptionFlags]) -> Vec<(bool, bool)> {
    options
        .iter()
        .map(|f| (f.selectedness(), f.dirtiness()))
        .collect()
}

fn run(initial: &[(bool, bool)], mut matches: impl FnMut(usize) -> bool) -> Outcome {
    let mut options: Vec<_> = initial
        .iter()
        .map(|&(s, d)| HtmlOptionFlags::new(s, d))
        .collect();
    let mut cursor = Cursor::new(options.len());
    let mut actions = Vec::new();
    let mut before_selection = Vec::new();
    let mut refreshes = Vec::new();
    // Source maximum: one deselection and comparison per option, one chosen
    // write, one update and Complete. A first match can only remove reads.
    for _ in 0..(2 * initial.len() + 3) {
        let action = cursor.action();
        actions.push(action);
        let reply = match action {
            Action::Deselect { ordinal } => {
                options[ordinal].set_selectedness_for_select(false);
                Reply::Applied
            }
            Action::ReadMatch { ordinal } => {
                assert!(!options[ordinal].selectedness());
                Reply::Matches(matches(ordinal))
            }
            Action::SelectAndDirty { ordinal } => {
                before_selection.push(flags(&options));
                options[ordinal].select_for_select_setter();
                Reply::Applied
            }
            Action::UpdateSelectedcontent => {
                refreshes.push(flags(&options));
                Reply::Applied
            }
            Action::Complete => {
                return Outcome {
                    flags: flags(&options),
                    actions,
                    before_selection,
                    refreshes,
                };
            }
        };
        cursor.reply(reply).unwrap();
    }
    panic!("setter did not complete within its source action bound");
}

#[test]
fn value_setter_remembers_first_equal_value_but_deselects_the_complete_list() {
    let actual_values = ["other", "chosen", "chosen", "later"];
    let mut reads = Vec::new();
    let outcome = run(
        &[(true, false), (false, false), (true, true), (true, false)],
        |ordinal| {
            reads.push(ordinal);
            actual_values[ordinal] == "chosen"
        },
    );
    assert_eq!(reads, [0, 1]);
    assert_eq!(
        outcome.actions,
        [
            Action::Deselect { ordinal: 0 },
            Action::ReadMatch { ordinal: 0 },
            Action::Deselect { ordinal: 1 },
            Action::ReadMatch { ordinal: 1 },
            Action::Deselect { ordinal: 2 },
            Action::Deselect { ordinal: 3 },
            Action::SelectAndDirty { ordinal: 1 },
            Action::UpdateSelectedcontent,
            Action::Complete,
        ]
    );
    assert_eq!(
        outcome.before_selection,
        [vec![
            (false, false),
            (false, false),
            (false, true),
            (false, false)
        ]]
    );
    assert_eq!(
        outcome.flags,
        [(false, false), (true, true), (false, true), (false, false)]
    );
    assert_eq!(
        outcome.refreshes,
        [vec![
            (false, false),
            (true, true),
            (false, true),
            (false, false)
        ]]
    );
}

#[test]
fn index_setter_selects_and_dirties_the_last_match_only_after_all_deselections() {
    let outcome = run(&[(true, true), (true, false), (false, false)], |ordinal| {
        ordinal == 2
    });
    assert_eq!(
        outcome.actions,
        [
            Action::Deselect { ordinal: 0 },
            Action::ReadMatch { ordinal: 0 },
            Action::Deselect { ordinal: 1 },
            Action::ReadMatch { ordinal: 1 },
            Action::Deselect { ordinal: 2 },
            Action::ReadMatch { ordinal: 2 },
            Action::SelectAndDirty { ordinal: 2 },
            Action::UpdateSelectedcontent,
            Action::Complete,
        ]
    );
    assert_eq!(
        outcome.before_selection,
        [vec![(false, true), (false, false), (false, false)]]
    );
    assert_eq!(outcome.flags, [(false, true), (false, false), (true, true)]);
    assert_eq!(
        outcome.refreshes,
        [vec![(false, true), (false, false), (true, true)]]
    );
}

#[test]
fn out_of_range_index_leaves_every_option_unselected_without_normalization() {
    let outcome = run(&[(true, false), (true, true)], |ordinal| ordinal == 7);
    assert_eq!(
        outcome.actions,
        [
            Action::Deselect { ordinal: 0 },
            Action::ReadMatch { ordinal: 0 },
            Action::Deselect { ordinal: 1 },
            Action::ReadMatch { ordinal: 1 },
            Action::UpdateSelectedcontent,
            Action::Complete,
        ]
    );
    assert!(outcome.before_selection.is_empty());
    assert_eq!(outcome.flags, [(false, false), (false, true)]);
    assert_eq!(outcome.refreshes, [vec![(false, false), (false, true)]]);
}

#[test]
fn empty_list_still_updates_selectedcontent_without_a_comparison() {
    let outcome = run(&[], |_| panic!("empty option list has no predicate read"));
    assert_eq!(
        outcome.actions,
        [Action::UpdateSelectedcontent, Action::Complete]
    );
    assert_eq!(outcome.flags, []);
    assert!(outcome.before_selection.is_empty());
    assert_eq!(outcome.refreshes, [vec![]]);
}

#[test]
fn chosen_flag_transition_sets_both_fields_from_every_prior_flag_pair() {
    for (selected, dirty) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut option = HtmlOptionFlags::new(selected, dirty);
        option.select_for_select_setter();
        assert_eq!((option.selectedness(), option.dirtiness()), (true, true));
        option.selected_setter(false);
        assert_eq!((option.selectedness(), option.dirtiness()), (false, true));
    }
}

#[test]
fn wrong_replies_preserve_every_phase_and_allow_the_actual_continuation() {
    for (count, first_match) in [(0, None), (1, None), (1, Some(0)), (3, Some(1)), (3, None)] {
        let mut cursor = Cursor::new(count);
        for _ in 0..(2 * count + 3) {
            let pending = cursor.action();
            let wrong: &[Reply] = match pending {
                Action::ReadMatch { .. } => &[Reply::Applied],
                Action::Complete => &[Reply::Applied, Reply::Matches(false), Reply::Matches(true)],
                _ => &[Reply::Matches(false), Reply::Matches(true)],
            };
            for &reply in wrong {
                let before = cursor;
                assert_eq!(cursor.reply(reply), Err(HtmlSelectCursorError::Reply));
                assert_eq!(cursor, before);
                assert_eq!(cursor.action(), pending);
            }
            match pending {
                Action::ReadMatch { ordinal } => cursor
                    .reply(Reply::Matches(Some(ordinal) == first_match))
                    .unwrap(),
                Action::Complete => break,
                _ => cursor.reply(Reply::Applied).unwrap(),
            }
        }
        assert_eq!(cursor.action(), Action::Complete);
    }
}

#[test]
fn maximum_extent_starts_a_bounded_prefix_without_iterating_the_extent() {
    let mut cursor = Cursor::new(usize::MAX);
    assert_eq!(cursor.action(), Action::Deselect { ordinal: 0 });
    cursor.reply(Reply::Applied).unwrap();
    assert_eq!(cursor.action(), Action::ReadMatch { ordinal: 0 });
    cursor.reply(Reply::Matches(true)).unwrap();
    assert_eq!(cursor.action(), Action::Deselect { ordinal: 1 });
    cursor.reply(Reply::Applied).unwrap();
    assert_eq!(cursor.action(), Action::Deselect { ordinal: 2 });
}

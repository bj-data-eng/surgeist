//! The payload parts the census measures are measured once for as long as they
//! are shared and unchanged, so a state whose parts have been measured is not
//! measured again, and an edit costs the parts it replaces.
use super::probe;
use crate::{model::State, *};

fn inputs() -> CssomSheetInputs {
    CssomSheetInputs::external(
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "memo".into(),
            revision: 1,
        },
        true,
    )
}

fn batch(store: &CssomStore) -> CssomBatch {
    store
        .batch(CssomEditGuard::from_snapshot(&store.snapshot()))
        .unwrap()
}

/// A store holding one sheet of `rules` style rules and a media group.
fn store_with_rules(rules: usize) -> (CssomStore, CssomSheetId) {
    let mut text = (0..rules)
        .map(|index| format!(".r{index} {{ color: red; margin: {index}px 0 }}\n"))
        .collect::<String>();
    text.push_str("@media screen { .m { color: blue } }\n");
    let mut store = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
    let mut edit = batch(&store);
    let ticket = edit.create_parsed_sheet(&text, inputs()).unwrap();
    let sheet = store.commit(edit).unwrap().sheet(&ticket).unwrap().clone();
    (store, sheet)
}

/// The payload parts the census walk reaches in `state`: what each record holds
/// behind a shared handle that its source walk visits.
fn parts_in(state: &State) -> usize {
    state.sheets.len() * 2
        + state.rules.len() * 3
        + state.maps.len()
        + state
            .blocks
            .values()
            .map(|block| {
                2 + usize::from(block.replacement_input.is_some())
                    + usize::from(block.selected_font_face.is_some())
            })
            .sum::<usize>()
}

#[test]
fn creating_a_sheet_measures_each_part_of_its_records_once() {
    probe::reset();
    let (store, _) = store_with_rules(30);
    let measured = probe::parts();
    let parts = parts_in(&store.state);
    // Without memoized parts every census of the creation measures them all again.
    assert!(
        measured > 0 && measured <= 2 * parts,
        "{measured} measurements of {parts} parts"
    );
}

#[test]
fn a_state_whose_parts_are_measured_is_not_measured_again() {
    let (store, _) = store_with_rules(30);
    probe::reset();
    let limits = CssomLimits::default();
    for _ in 0..2 {
        // Settling every record again, as a state accounted for from nothing is.
        store.state.unsettled_copy().settle(&limits, &[]).unwrap();
    }
    assert_eq!(probe::parts(), 0);
}

#[test]
fn an_edit_measures_only_the_parts_it_replaces() {
    let (store, sheet) = store_with_rules(30);
    let rule = store.snapshot().sheet_rules(&sheet).unwrap()[3].clone();
    let mut edit = batch(&store);
    probe::reset();
    edit.set_disabled(&sheet, true).unwrap();
    // The sheet record is copied; the parts it holds are shared with the published one.
    assert_eq!(probe::parts(), 0, "disabling replaces no part");
    edit.set_style_selector_text(&rule, ".changed > .selector")
        .unwrap();
    let replaced = probe::parts();
    assert!(
        (1..=3).contains(&replaced),
        "a selector edit replaces the data of one rule, not the parts of {} records",
        edit.staged.rules.len()
    );
}

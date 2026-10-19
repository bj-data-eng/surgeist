#![forbid(unsafe_code)]

use surgeist_html::HtmlFormNativeState;

// HTML877d §4.10.3, raw33372–33375: reset() tests and marks locked-for-reset;
// a newly created Form has not entered that event-bearing operation.
#[test]
fn native_form_starts_unmarked_for_reset() {
    assert!(!HtmlFormNativeState::fresh().locked_for_reset());
}

// HTML877d §4.10.22.3, raw45079–45083 explicitly initializes both booleans false.
#[test]
fn native_form_has_no_entry_list_construction_or_submission_events_in_progress() {
    let state = HtmlFormNativeState::fresh();
    assert!(!state.constructing_entry_list());
    assert!(!state.firing_submission_events());
}

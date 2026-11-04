//! `list` applicability per input type state, from the `list` row of the HTML877d input type
//! attribute summary table (`#input-type-attr-summary`). The expected set is that row's "Yes"
//! cells, written out per state.

use surgeist_html::{attributes::HtmlInputType as Input, bounded::attributes::input_list_applies};

#[test]
fn list_applies_exactly_to_the_states_the_summary_table_marks_yes() {
    let applies = [
        Input::Text,
        Input::Search,
        Input::Telephone,
        Input::Url,
        Input::Email,
        Input::Date,
        Input::Month,
        Input::Week,
        Input::Time,
        Input::LocalDateTime,
        Input::Number,
        Input::Range,
        Input::Color,
    ];
    let does_not_apply = [
        Input::Hidden,
        Input::Password,
        Input::Checkbox,
        Input::Radio,
        Input::File,
        Input::Submit,
        Input::Image,
        Input::Reset,
        Input::Button,
    ];
    for state in applies {
        assert!(input_list_applies(state), "{state:?}");
    }
    for state in does_not_apply {
        assert!(!input_list_applies(state), "{state:?}");
    }
}

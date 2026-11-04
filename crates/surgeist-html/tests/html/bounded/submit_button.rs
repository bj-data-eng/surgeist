//! Submit-button classification from HTML877d `#the-button-element` ("is said to be a submit
//! button"), `#submit-button-state-(type=submit)` and `#image-button-state-(type=image)`.
//! Expected values are the spec's rules written out, not the implementation's output.

use surgeist_html::{
    attributes::{
        HtmlButtonSubmitterContext as Context, HtmlButtonType, HtmlButtonTypeState as State,
        HtmlInputType as Input,
    },
    bounded::attributes::{
        HtmlButtonSubmitterFact as Fact, button_submitter_context_with, input_submitter_context,
    },
};

#[test]
fn only_the_submit_button_and_image_button_states_make_an_input_a_submit_button() {
    let every_state = [
        Input::Hidden,
        Input::Text,
        Input::Search,
        Input::Telephone,
        Input::Url,
        Input::Email,
        Input::Password,
        Input::Date,
        Input::Month,
        Input::Week,
        Input::Time,
        Input::LocalDateTime,
        Input::Number,
        Input::Range,
        Input::Color,
        Input::Checkbox,
        Input::Radio,
        Input::File,
        Input::Submit,
        Input::Image,
        Input::Reset,
        Input::Button,
    ];
    for state in every_state {
        let expected = if matches!(state, Input::Submit | Input::Image) {
            Context::SubmitButton
        } else {
            Context::NotSubmitButton
        };
        assert_eq!(input_submitter_context(state), expected, "{state:?}");
    }
}

/// Runs the rule with fixed answers and returns the verdict with the facts asked, in order.
fn classify(state: State, command: bool, commandfor: bool, select: bool) -> (Context, Vec<Fact>) {
    let mut asked = Vec::new();
    let context = button_submitter_context_with::<()>(state, |fact| {
        asked.push(fact);
        Ok(match fact {
            Fact::CommandAttributePresent => command,
            Fact::CommandforAttributePresent => commandfor,
            Fact::ParentIsSelect => select,
        })
    })
    .unwrap();
    (context, asked)
}

#[test]
fn explicit_button_types_are_decided_without_asking_for_any_fact() {
    for (state, expected) in [
        (
            State::Keyword(HtmlButtonType::Submit),
            Context::SubmitButton,
        ),
        (
            State::Keyword(HtmlButtonType::Reset),
            Context::NotSubmitButton,
        ),
        (
            State::Keyword(HtmlButtonType::Button),
            Context::NotSubmitButton,
        ),
    ] {
        // Every fact would say "Auto would not submit"; the explicit type must ignore them.
        assert_eq!(classify(state, true, true, true), (expected, vec![]));
    }
}

#[test]
fn auto_button_submits_only_without_command_commandfor_and_select_parent() {
    let (context, asked) = classify(State::Auto, false, false, false);
    assert_eq!(context, Context::SubmitButton);
    assert_eq!(
        asked,
        [
            Fact::CommandAttributePresent,
            Fact::CommandforAttributePresent,
            Fact::ParentIsSelect
        ]
    );
}

#[test]
fn auto_button_stops_asking_at_the_first_fact_that_rules_out_a_submit_button() {
    assert_eq!(
        classify(State::Auto, true, true, true),
        (
            Context::NotSubmitButton,
            vec![Fact::CommandAttributePresent]
        )
    );
    assert_eq!(
        classify(State::Auto, false, true, true),
        (
            Context::NotSubmitButton,
            vec![
                Fact::CommandAttributePresent,
                Fact::CommandforAttributePresent
            ]
        )
    );
    assert_eq!(
        classify(State::Auto, false, false, true),
        (
            Context::NotSubmitButton,
            vec![
                Fact::CommandAttributePresent,
                Fact::CommandforAttributePresent,
                Fact::ParentIsSelect
            ]
        )
    );
}

#[test]
fn the_first_fact_error_is_returned_unchanged_and_ends_the_questions() {
    let mut asked = Vec::new();
    let result = button_submitter_context_with(State::Auto, |fact| {
        asked.push(fact);
        match fact {
            Fact::CommandAttributePresent => Ok(false),
            Fact::CommandforAttributePresent => Err("commandfor read refused"),
            Fact::ParentIsSelect => panic!("asked after an error"),
        }
    });
    assert_eq!(result, Err("commandfor read refused"));
    assert_eq!(
        asked,
        [
            Fact::CommandAttributePresent,
            Fact::CommandforAttributePresent
        ]
    );
}

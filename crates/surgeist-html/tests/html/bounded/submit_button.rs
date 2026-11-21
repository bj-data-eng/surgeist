//! Submit-button classification from HTML877d `#the-button-element` ("is said to be a submit
//! button"), `#submit-button-state-(type=submit)` and `#image-button-state-(type=image)`.
//! Expected values are the spec's rules written out, not the implementation's output.

use surgeist_html::{
    attributes::{
        HtmlButtonSubmitterContext as Context, HtmlButtonType, HtmlButtonTypeState as State,
        HtmlInputType as Input, HtmlStandardCommand, command::HtmlCommandState,
    },
    bounded::attributes::{
        HtmlAttributeKeywordRead, HtmlButtonSubmitterFact as Fact, button_command_readback_with,
        button_submitter_context_with, button_type_keyword_with,
        form_enctype_override_keyword_with, form_method_override_keyword_with,
        input_submitter_context,
    },
    values::DocumentString,
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

#[test]
fn button_type_readback_composes_content_recognition_with_the_lazy_facts() {
    let explicit = "SuBmIt".encode_utf16().collect::<Vec<_>>();
    assert_eq!(
        button_type_keyword_with(
            Some(&explicit),
            |_| Ok::<_, ()>(()),
            |_| panic!("an explicit submit state needs no contextual fact"),
        ),
        Ok("submit")
    );
    let invalid = " Submit ".encode_utf16().collect::<Vec<_>>();
    let mut asked = Vec::new();
    let result = button_type_keyword_with(
        Some(&invalid),
        |_| Ok(()),
        |fact| {
            asked.push(fact);
            match fact {
                Fact::CommandAttributePresent => Ok(false),
                Fact::CommandforAttributePresent => Err("the original commandfor refusal"),
                Fact::ParentIsSelect => panic!("the parent must not be reached"),
            }
        },
    );
    assert_eq!(result, Err("the original commandfor refusal"));
    assert_eq!(
        asked,
        [
            Fact::CommandAttributePresent,
            Fact::CommandforAttributePresent
        ]
    );
    let mut facts = 0;
    assert_eq!(
        button_type_keyword_with(
            None,
            |_| Err("the original content header refusal"),
            |_| {
                facts += 1;
                Ok(false)
            },
        ),
        Err("the original content header refusal")
    );
    assert_eq!(facts, 0);
}

#[test]
fn button_command_readback_preserves_intrinsic_states_and_refuses_the_exact_prefix_read() {
    let standard = DocumentString::from("sHoW-PoPoVeR");
    let result = button_command_readback_with(Some(&standard), |_| Ok::<_, ()>(())).unwrap();
    assert_eq!(
        result.state(),
        HtmlCommandState::Standard(HtmlStandardCommand::ShowPopover)
    );
    assert_eq!(result.custom_value(), None);
    let malformed = DocumentString::from(" show-popover");
    for unknown in [None, Some(&malformed)] {
        let result = button_command_readback_with(unknown, |_| Ok::<_, ()>(())).unwrap();
        assert_eq!(result.state(), HtmlCommandState::Unknown);
        assert_eq!(result.custom_value(), None);
    }

    let raw = DocumentString::from_utf16(vec![45, 45, 0xd800, 0, 0xdc00, 32]);
    let mut actions = Vec::new();
    let result = button_command_readback_with(Some(&raw), |action| {
        actions.push(action);
        Ok::<_, HtmlAttributeKeywordRead>(())
    })
    .unwrap();
    assert_eq!(result.state(), HtmlCommandState::Custom);
    assert_eq!(result.custom_value(), Some(&raw));
    assert_eq!(
        actions
            .iter()
            .filter(|&&action| action == HtmlAttributeKeywordRead::KeywordPair)
            .count(),
        2
    );
    assert_eq!(actions.last(), Some(&HtmlAttributeKeywordRead::KeywordPair));
    let mut accepted = actions.len() - 1;
    assert_eq!(
        button_command_readback_with(Some(&raw), |action| {
            if accepted == 0 {
                return Err(action);
            }
            accepted -= 1;
            Ok(())
        }),
        Err(HtmlAttributeKeywordRead::KeywordPair)
    );
    assert_eq!(accepted, 0);
}

#[test]
fn button_submitter_override_readback_distinguishes_absent_from_present_invalid() {
    let encoding_default = "application/x-www-form-urlencoded";
    for (content, method, encoding) in [
        (None, "", ""),
        (Some(""), "get", encoding_default),
        (Some("unknown"), "get", encoding_default),
        (Some("pOsT"), "post", encoding_default),
        (Some("MuLtipart/Form-Data"), "get", "multipart/form-data"),
    ] {
        let raw = content.map(|value| value.encode_utf16().collect::<Vec<_>>());
        assert_eq!(
            form_method_override_keyword_with(raw.as_deref(), |_| Ok::<_, ()>(())),
            Ok(method),
            "{content:?}"
        );
        assert_eq!(
            form_enctype_override_keyword_with(raw.as_deref(), |_| Ok::<_, ()>(())),
            Ok(encoding),
            "{content:?}"
        );
    }
}

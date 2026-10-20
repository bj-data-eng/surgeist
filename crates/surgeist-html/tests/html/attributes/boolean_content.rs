use crate::html::support::html_element as element;
use surgeist_html::{
    Attribute, HtmlAttributeAuthorState as Author, HtmlAttributeConformanceError as Conformance,
    HtmlAttributeObservation as Observation, HtmlAttributeRecognition as Recognition,
    HtmlBooleanAttribute as Boolean, HtmlButtonSubmitterContext as Button, HtmlInputType as Input,
    HtmlScriptAttributeContext as Script, HtmlValueInterpreter,
};

#[test]
fn prohibited_input_states_preserve_present_boolean_content_and_lexical_observation() {
    // The per-state ignore/applicability clauses govern effects and authorship;
    // Boolean Reflect/defaultChecked still observes present content as true.
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, state) in [
        (Boolean::Checked, Input::Text),
        (Boolean::Readonly, Input::Checkbox),
        (Boolean::Alpha, Input::Text),
        (Boolean::Multiple, Input::Text),
        (Boolean::Required, Input::Hidden),
        (Boolean::Formnovalidate, Input::Text),
    ] {
        for value in ["", selected.attribute().local_name(), "false"] {
            let raw = Attribute::new(selected.attribute().qualified_name(), value);
            let report = interpreter
                .boolean_attribute_for_input(selected, &element("input"), Some(&raw), state)
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(report.input_state(), Some(state));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.author(),
                if value == "false" {
                    Author::Nonconforming(Conformance::BooleanSpelling)
                } else {
                    Author::Conforming
                }
            );
            assert_eq!(
                report.recognition(),
                if value.is_empty() {
                    Recognition::Empty
                } else if value == "false" {
                    Recognition::Unrecognized
                } else {
                    Recognition::BooleanName
                }
            );
        }
    }
}

#[test]
fn prohibited_input_states_preserve_known_absent_boolean_content() {
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, state) in [
        (Boolean::Checked, Input::Text),
        (Boolean::Readonly, Input::Checkbox),
        (Boolean::Alpha, Input::Text),
        (Boolean::Multiple, Input::Text),
        (Boolean::Required, Input::Hidden),
        (Boolean::Formnovalidate, Input::Text),
    ] {
        let report = interpreter
            .boolean_attribute_for_input(
                selected,
                &element("input"),
                None::<&surgeist_html::Attribute>,
                state,
            )
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&false));
        assert_eq!(report.observation(), Observation::Missing);
        assert_eq!(report.author(), Author::Missing);
        assert_eq!(report.raw_attribute(), None);
    }
}

#[test]
fn absent_input_type_facts_do_not_erase_known_boolean_content() {
    let interpreter = HtmlValueInterpreter::new(100);
    for selected in [
        Boolean::Alpha,
        Boolean::Checked,
        Boolean::Multiple,
        Boolean::Readonly,
        Boolean::Required,
        Boolean::Formnovalidate,
    ] {
        let raw = Attribute::new(selected.attribute().qualified_name(), "false");
        let present = interpreter
            .boolean_attribute(selected, &element("input"), Some(&raw))
            .unwrap();
        assert_eq!(present.intrinsic(), Some(&true));
        assert_eq!(present.input_state(), None);
        assert_eq!(present.raw_attribute(), Some(&raw));
        assert_eq!(
            present.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        let absent = interpreter
            .boolean_attribute(
                selected,
                &element("input"),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.author(), Author::Missing);
    }
}

#[test]
fn prohibited_script_contexts_preserve_present_content_and_boolean_spelling() {
    // defer/noModule reflection is presence. This async content operation
    // makes no claim about the separate live force-async getter flag.
    let interpreter = HtmlValueInterpreter::new(100);
    for (selected, context) in [
        (Boolean::Defer, Script::ExternalModule),
        (Boolean::Defer, Script::InlineModule),
        (Boolean::Nomodule, Script::ExternalModule),
        (Boolean::Defer, Script::InlineClassic),
        (Boolean::Async, Script::InlineClassic),
        (Boolean::Async, Script::DataBlock),
        (Boolean::Defer, Script::DataBlock),
        (Boolean::Nomodule, Script::DataBlock),
    ] {
        for value in ["", selected.attribute().local_name(), "false"] {
            let raw = Attribute::new(selected.attribute().qualified_name(), value);
            let report = interpreter
                .boolean_attribute_for_script(selected, &element("script"), Some(&raw), context)
                .unwrap();
            assert_eq!(report.intrinsic(), Some(&true));
            assert_eq!(report.script_context(), Some(context));
            assert_eq!(report.raw_attribute(), Some(&raw));
            assert_eq!(
                report.author(),
                if value == "false" {
                    Author::Nonconforming(Conformance::BooleanSpelling)
                } else {
                    Author::Conforming
                }
            );
        }
        let absent = interpreter
            .boolean_attribute_for_script(
                selected,
                &element("script"),
                None::<&surgeist_html::Attribute>,
                context,
            )
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.author(), Author::Missing);
        assert_eq!(absent.raw_attribute(), None);
    }
}

#[test]
fn absent_script_facts_do_not_erase_known_boolean_content() {
    let interpreter = HtmlValueInterpreter::new(100);
    for selected in [Boolean::Async, Boolean::Defer, Boolean::Nomodule] {
        let raw = Attribute::new(selected.attribute().qualified_name(), "false");
        let present = interpreter
            .boolean_attribute(selected, &element("script"), Some(&raw))
            .unwrap();
        assert_eq!(present.intrinsic(), Some(&true));
        assert_eq!(present.script_context(), None);
        assert_eq!(present.raw_attribute(), Some(&raw));
        assert_eq!(
            present.author(),
            Author::Nonconforming(Conformance::BooleanSpelling)
        );
        let absent = interpreter
            .boolean_attribute(
                selected,
                &element("script"),
                None::<&surgeist_html::Attribute>,
            )
            .unwrap();
        assert_eq!(absent.intrinsic(), Some(&false));
        assert_eq!(absent.author(), Author::Missing);
    }
}

#[test]
fn non_submit_button_preserves_formnovalidate_content_presence() {
    let interpreter = HtmlValueInterpreter::new(100);
    for value in ["", "formnovalidate", "false"] {
        let raw = Attribute::new(Boolean::Formnovalidate.attribute().qualified_name(), value);
        let report = interpreter
            .formnovalidate_for_button(&element("button"), Some(&raw), Button::NotSubmitButton)
            .unwrap();
        assert_eq!(report.intrinsic(), Some(&true));
        assert_eq!(
            report.button_submitter_context(),
            Some(Button::NotSubmitButton)
        );
        assert_eq!(report.raw_attribute(), Some(&raw));
        assert_eq!(
            report.author(),
            if value == "false" {
                Author::Nonconforming(Conformance::BooleanSpelling)
            } else {
                Author::Conforming
            }
        );
    }
    let absent = interpreter
        .formnovalidate_for_button(
            &element("button"),
            None::<&surgeist_html::Attribute>,
            Button::NotSubmitButton,
        )
        .unwrap();
    assert_eq!(absent.intrinsic(), Some(&false));
    assert_eq!(absent.author(), Author::Missing);
}

#[test]
fn absent_button_submitter_facts_do_not_erase_formnovalidate_presence() {
    let interpreter = HtmlValueInterpreter::new(100);
    let raw = Attribute::new(
        Boolean::Formnovalidate.attribute().qualified_name(),
        "false",
    );
    let present = interpreter
        .boolean_attribute(Boolean::Formnovalidate, &element("button"), Some(&raw))
        .unwrap();
    assert_eq!(present.intrinsic(), Some(&true));
    assert_eq!(present.button_submitter_context(), None);
    assert_eq!(present.raw_attribute(), Some(&raw));
    assert_eq!(
        present.author(),
        Author::Nonconforming(Conformance::BooleanSpelling)
    );
    let absent = interpreter
        .boolean_attribute(
            Boolean::Formnovalidate,
            &element("button"),
            None::<&surgeist_html::Attribute>,
        )
        .unwrap();
    assert_eq!(absent.intrinsic(), Some(&false));
    assert_eq!(absent.author(), Author::Missing);
}

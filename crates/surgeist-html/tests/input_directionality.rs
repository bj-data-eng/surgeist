use surgeist_html::HtmlInputType;

// HTML877d #auto-directionality-form-associated-elements lists these ten states.
#[test]
fn selected_auto_directionality_input_states_use_current_value() {
    for kind in [
        HtmlInputType::Hidden,
        HtmlInputType::Text,
        HtmlInputType::Search,
        HtmlInputType::Telephone,
        HtmlInputType::Url,
        HtmlInputType::Email,
        HtmlInputType::Password,
        HtmlInputType::Submit,
        HtmlInputType::Reset,
        HtmlInputType::Button,
    ] {
        assert!(kind.uses_current_value_for_directionality(), "{kind:?}");
    }
}

#[test]
fn remaining_input_states_do_not_use_current_value_for_auto_directionality() {
    for kind in [
        HtmlInputType::Date,
        HtmlInputType::Month,
        HtmlInputType::Week,
        HtmlInputType::Time,
        HtmlInputType::LocalDateTime,
        HtmlInputType::Number,
        HtmlInputType::Range,
        HtmlInputType::Color,
        HtmlInputType::Checkbox,
        HtmlInputType::Radio,
        HtmlInputType::File,
        HtmlInputType::Image,
    ] {
        assert!(!kind.uses_current_value_for_directionality(), "{kind:?}");
    }
}

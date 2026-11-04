//! Standard-command validity for a target, from HTML877d "determine if a command is valid for a
//! target" (`#determine-if-command-is-valid`) and the `dialog` "is valid command steps"
//! (`#the-dialog-element`). Expected values are the spec tables, not the implementation's output.

use surgeist_html::{
    attributes::HtmlStandardCommand as Command, vocabulary::KnownHtmlElement as Element,
};

const POPOVER_COMMANDS: [Command; 3] = [
    Command::TogglePopover,
    Command::ShowPopover,
    Command::HidePopover,
];
const DIALOG_COMMANDS: [Command; 3] = [Command::Close, Command::RequestClose, Command::ShowModal];

#[test]
fn popover_commands_are_valid_for_every_html_element_and_dialog_commands_for_none() {
    for command in POPOVER_COMMANDS {
        assert!(command.is_valid_for_every_html_element(), "{command:?}");
    }
    for command in DIALOG_COMMANDS {
        assert!(!command.is_valid_for_every_html_element(), "{command:?}");
    }
}

#[test]
fn only_a_dialog_defines_steps_accepting_close_request_close_and_show_modal() {
    for command in DIALOG_COMMANDS {
        assert!(
            command.is_valid_by_element_steps(Some(Element::Dialog)),
            "{command:?}"
        );
        for other in [
            Element::Div,
            Element::Button,
            Element::Details,
            Element::Input,
        ] {
            assert!(
                !command.is_valid_by_element_steps(Some(other)),
                "{command:?} {other:?}"
            );
        }
        assert!(
            !command.is_valid_by_element_steps(None),
            "a local name without defined steps is false: {command:?}"
        );
    }
}

#[test]
fn dialog_steps_return_false_for_popover_commands() {
    for command in POPOVER_COMMANDS {
        assert!(
            !command.is_valid_by_element_steps(Some(Element::Dialog)),
            "{command:?}"
        );
        assert!(!command.is_valid_by_element_steps(None), "{command:?}");
    }
}

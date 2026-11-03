//! Local command/action content states. Targets and activation are separate.

use super::*;
use crate::internal::microsyntax::tokens::is_ascii_whitespace;

keyword_type!(HtmlStandardCommand, Command, [
    /// The `toggle-popover` command: shows or hides the targeted popover element.
    TogglePopover => "toggle-popover",
    /// The `show-popover` command: shows the targeted popover element.
    ShowPopover => "show-popover",
    /// The `hide-popover` command: hides the targeted popover element.
    HidePopover => "hide-popover",
    /// The `close` command: closes the targeted `dialog` element.
    Close => "close",
    /// The `request-close` command: requests that the targeted `dialog` element close.
    RequestClose => "request-close",
    /// The `show-modal` command: shows the targeted `dialog` element as a modal.
    ShowModal => "show-modal",
]);
keyword_type!(HtmlPopoverTargetAction, Popovertargetaction, [
    /// The `toggle` keyword, also selected by a missing or invalid value: show the target popover
    /// if hidden and hide it if shown.
    Toggle => "toggle",
    /// The `show` keyword: show the target popover.
    Show => "show",
    /// The `hide` keyword: hide the target popover.
    Hide => "hide",
]);

/// A custom command fails the selected local native author checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlCustomCommandError {
    /// The value does not begin with two ASCII hyphens, so it is not a custom command keyword.
    MissingPrefix,
    /// The value ends in an ASCII whitespace code unit. Only the final unit can violate the
    /// boundary rule, because the required `--` prefix begins the value.
    BoundaryWhitespace,
}

impl std::fmt::Display for HtmlCustomCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MissingPrefix => "custom command does not start with two hyphens",
            Self::BoundaryWhitespace => "custom command starts or ends with ASCII whitespace",
        })
    }
}
impl std::error::Error for HtmlCustomCommandError {}

/// Exact custom author keyword with a two-hyphen prefix and no ASCII boundary
/// whitespace. This selected HTML/Infra interpretation does not prove element,
/// target, activation or serialized-character conformance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomCommandKeyword {
    value: DocumentString,
}
impl HtmlCustomCommandKeyword {
    /// Retains exact UTF-16, including NULL and isolated surrogates. Conversion
    /// and shared storage use ordinary allocation, without a physical budget.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlCustomCommandError> {
        let value = value.into();
        if !custom_prefix(&value) {
            return Err(HtmlCustomCommandError::MissingPrefix);
        }
        if custom_boundary_whitespace(&value) {
            return Err(HtmlCustomCommandError::BoundaryWhitespace);
        }
        Ok(Self { value })
    }
    /// Returns the exact keyword, including its `--` prefix, in UTF-16 code units as supplied.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// Native local author spelling. Unknown processing has no author emitter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCommandKeyword {
    /// One of the standard command keywords, emitted in its canonical spelling.
    Standard(HtmlStandardCommand),
    /// A checked custom command keyword, emitted exactly as retained.
    Custom(HtmlCustomCommandKeyword),
}
impl HtmlCommandKeyword {
    /// Emits a no-namespace command with canonical standard spelling or exact
    /// checked Custom payload. No target or execution context is inferred.
    pub fn into_attribute(self) -> Attribute {
        match self {
            Self::Standard(value) => value.into_attribute(),
            Self::Custom(value) => {
                Attribute::new(KnownHtmlAttribute::Command.qualified_name(), value.value)
            }
        }
    }
}

/// Intrinsic command content state, independent of target validity/activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCommandState {
    /// The value matched a standard command keyword.
    Standard(HtmlStandardCommand),
    /// The value is a custom command keyword, which only dispatches the command event on the
    /// target. It starts with two ASCII hyphens.
    Custom,
    /// The Unknown state, which a missing or invalid value selects. No command is valid for a
    /// target in this state.
    Unknown,
}

/// Owner-coupled processing state and exact custom payload. Custom always has
/// its complete prefix-qualified value; other states have none. Author-invalid
/// trailing ASCII whitespace remains observable here and has no native emitter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCommandReadback {
    state: HtmlCommandState,
    custom: Option<DocumentString>,
}
impl HtmlCommandReadback {
    /// Returns the command state. It is `Custom` exactly when `custom_value` is `Some`.
    pub fn state(&self) -> HtmlCommandState {
        self.state
    }
    /// Returns the complete prefix-qualified custom value in exact UTF-16 code units, or `None` in
    /// every other state.
    pub fn custom_value(&self) -> Option<&DocumentString> {
        self.custom.as_ref()
    }
}

fn custom_prefix(value: &DocumentString) -> bool {
    value.code_units().starts_with(&[0x2D, 0x2D])
}
fn custom_boundary_whitespace(value: &DocumentString) -> bool {
    // A prefix-qualified Custom begins with '-' so only the final unit can
    // violate the boundary rule. Interior whitespace is unconstrained.
    value
        .code_units()
        .last()
        .is_some_and(|&unit| is_ascii_whitespace(unit))
}

enum_rule!(COMMAND, HtmlCommandState, Command, Context::Elements(&[KnownHtmlElement::Button]), [28], "attr-button-command",
    Some(HtmlCommandState::Unknown), Some(HtmlCommandState::Unknown), None,
    ["toggle-popover" => ("toggle-popover", HtmlCommandState::Standard(HtmlStandardCommand::TogglePopover)),
     "show-popover" => ("show-popover", HtmlCommandState::Standard(HtmlStandardCommand::ShowPopover)),
     "hide-popover" => ("hide-popover", HtmlCommandState::Standard(HtmlStandardCommand::HidePopover)),
     "close" => ("close", HtmlCommandState::Standard(HtmlStandardCommand::Close)),
     "request-close" => ("request-close", HtmlCommandState::Standard(HtmlStandardCommand::RequestClose)),
     "show-modal" => ("show-modal", HtmlCommandState::Standard(HtmlStandardCommand::ShowModal))]);
enum_rule!(POPOVER_ACTION, HtmlPopoverTargetAction, Popovertargetaction, Context::PopoverTargetAction, [121], "attr-popovertargetaction",
    Some(HtmlPopoverTargetAction::Toggle), Some(HtmlPopoverTargetAction::Toggle), None,
    ["toggle" => ("toggle", HtmlPopoverTargetAction::Toggle),
     "show" => ("show", HtmlPopoverTargetAction::Show),
     "hide" => ("hide", HtmlPopoverTargetAction::Hide)]);

impl HtmlValueInterpreter {
    /// Retains local command processing and independent author observations.
    /// Full raw UTF-16 is admitted before expanded-name applicability. This
    /// per-value cap is not a work/cancellation or physical-allocation budget.
    /// Target resolution, command validity and activation are not performed.
    pub fn command(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlCommandReadback>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(HtmlAttributeInput::value))?;
        let mut result = report(COMMAND.metadata, element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let selection = enum_value_selection(&COMMAND, raw.map(HtmlAttributeInput::value));
        result.recognition = selection.recognition;
        result.author = selection.author;
        result.intrinsic = selection.intrinsic.map(|state| HtmlCommandReadback {
            state,
            custom: None,
        });
        if let Some(value) = raw
            .map(HtmlAttributeInput::value)
            .filter(|value| custom_prefix(value))
        {
            result.recognition = HtmlAttributeRecognition::CustomCommand;
            result.author = if custom_boundary_whitespace(value) {
                HtmlAttributeAuthorState::Nonconforming(
                    HtmlAttributeConformanceError::NonconformingKeyword,
                )
            } else {
                HtmlAttributeAuthorState::Conforming
            };
            result.intrinsic = Some(HtmlCommandReadback {
                state: HtmlCommandState::Custom,
                custom: Some(value.clone()),
            });
        }
        Ok(result)
    }

    /// Local toggle/show/hide content. Applicable input presence keeps an
    /// Unchecked author-context observation until actual input state is supplied.
    pub fn popovertargetaction(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlPopoverTargetAction>, HtmlValueAdmissionError> {
        self.enumerated(&POPOVER_ACTION, element, raw)
    }

    /// Checks input-state author presence independently of local content state.
    /// Submit/Image/Reset/Button permit presence; this does not select a target
    /// or apply the source's separate activation/effect rules.
    pub fn popovertargetaction_for_input(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<HtmlPopoverTargetAction>, HtmlValueAdmissionError> {
        self.enumerated_with_context(&POPOVER_ACTION, element, raw, SuppliedContext::Input(state))
    }
}

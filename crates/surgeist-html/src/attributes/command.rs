//! Local command/action content states. Targets and activation are separate.

use super::*;
use crate::tokens::ascii_whitespace;

keyword_type!(HtmlStandardCommand, Command, [
    TogglePopover => "toggle-popover", ShowPopover => "show-popover",
    HidePopover => "hide-popover", Close => "close",
    RequestClose => "request-close", ShowModal => "show-modal"
]);
keyword_type!(HtmlPopoverTargetAction, Popovertargetaction, [
    Toggle => "toggle", Show => "show", Hide => "hide"
]);

/// A custom command fails the selected local native author checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomCommandError {
    MissingPrefix,
    BoundaryWhitespace,
}

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
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// Native local author spelling. Unknown processing has no author emitter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCommandKeyword {
    Standard(HtmlStandardCommand),
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
    Standard(HtmlStandardCommand),
    Custom,
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
    pub fn state(&self) -> HtmlCommandState {
        self.state
    }
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
        .is_some_and(|&unit| ascii_whitespace(unit))
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlCommandReadback>, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw.map(Attribute::value))?;
        let mut result = report(COMMAND.metadata, element, raw);
        if result.applicability != HtmlAttributeApplicability::Applicable {
            return Ok(result);
        }
        let selection = enum_value_selection(&COMMAND, raw.map(Attribute::value));
        result.recognition = selection.recognition;
        result.author = selection.author;
        result.intrinsic = selection.intrinsic.map(|state| HtmlCommandReadback {
            state,
            custom: None,
        });
        if let Some(value) = raw
            .map(Attribute::value)
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
        element: &QualifiedName,
        raw: Option<&Attribute>,
    ) -> Result<HtmlAttributeReport<HtmlPopoverTargetAction>, HtmlValueAdmissionError> {
        self.enumerated(&POPOVER_ACTION, element, raw)
    }

    /// Checks input-state author presence independently of local content state.
    /// Submit/Image/Reset/Button permit presence; this does not select a target
    /// or apply the source's separate activation/effect rules.
    pub fn popovertargetaction_for_input(
        &self,
        element: &QualifiedName,
        raw: Option<&Attribute>,
        state: HtmlInputType,
    ) -> Result<HtmlAttributeReport<HtmlPopoverTargetAction>, HtmlValueAdmissionError> {
        self.enumerated_with_context(&POPOVER_ACTION, element, raw, SuppliedContext::Input(state))
    }
}

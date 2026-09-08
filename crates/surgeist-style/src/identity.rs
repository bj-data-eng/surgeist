use std::{fmt, str::FromStr};

use crate::{Error, ErrorCode, Result, StateFlag};

macro_rules! style_token {
    ($name:ident, $field:literal, $allow_separators:literal) => {
        #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl AsRef<str>) -> Result<Self> {
                validate_ident(value.as_ref(), $field, $allow_separators).map(Self)
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = Error;

            fn from_str(value: &str) -> Result<Self> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = Error;

            fn try_from(value: &str) -> Result<Self> {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

style_token!(StyleTag, "StyleTag", false);
style_token!(StyleClass, "StyleClass", false);
style_token!(StyleKey, "StyleKey", true);
style_token!(StyleAttributeName, "StyleAttributeName", false);

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StyleAttributeValue(String);

impl StyleAttributeValue {
    pub fn new(value: impl AsRef<str>) -> Result<Self> {
        validate_value(value.as_ref()).map(Self)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self(String::new())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for StyleAttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StyleAttributeValue").field(&self.0).finish()
    }
}

impl fmt::Display for StyleAttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for StyleAttributeValue {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for StyleAttributeValue {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

impl AsRef<str> for StyleAttributeValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StyleAttribute {
    name: StyleAttributeName,
    value: StyleAttributeValue,
}

impl StyleAttribute {
    #[must_use]
    pub const fn new(name: StyleAttributeName, value: StyleAttributeValue) -> Self {
        Self { name, value }
    }

    #[must_use]
    pub const fn name(&self) -> &StyleAttributeName {
        &self.name
    }

    #[must_use]
    pub const fn value(&self) -> &StyleAttributeValue {
        &self.value
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum StyleRole {
    #[default]
    Generic,
    Application,
    Button,
    Text,
    List,
    ListItem,
    Checkbox,
    Textbox,
    Image,
    Canvas,
    Widget,
    Custom(StyleTag),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RangeState {
    InRange,
    OutOfRange,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct StyleState {
    hovered: bool,
    active: bool,
    focused: bool,
    focus_visible: bool,
    focus_within: bool,
    pointer_captured: bool,
    enabled: Option<bool>,
    selected: bool,
    pressed: bool,
    checked: Option<bool>,
    expanded: Option<bool>,
    required: Option<bool>,
    valid: Option<bool>,
    placeholder_shown: bool,
    modal: bool,
    fullscreen: bool,
    popover_open: bool,
    default: bool,
    indeterminate: bool,
    read_write: Option<bool>,
    range_state: Option<RangeState>,
}

impl StyleState {
    #[must_use]
    pub const fn disabled(&self) -> bool {
        matches!(self.enabled, Some(false))
    }

    #[must_use]
    pub const fn hovered(&self) -> bool {
        self.hovered
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub const fn focused(&self) -> bool {
        self.focused
    }

    #[must_use]
    pub const fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    #[must_use]
    pub const fn focus_within(&self) -> bool {
        self.focus_within
    }

    #[must_use]
    pub const fn pointer_captured(&self) -> bool {
        self.pointer_captured
    }

    #[must_use]
    pub const fn enabled(&self) -> Option<bool> {
        self.enabled
    }

    #[must_use]
    pub const fn selected(&self) -> bool {
        self.selected
    }

    #[must_use]
    pub const fn pressed(&self) -> bool {
        self.pressed
    }

    #[must_use]
    pub const fn checked(&self) -> Option<bool> {
        self.checked
    }

    #[must_use]
    pub const fn expanded(&self) -> Option<bool> {
        self.expanded
    }

    #[must_use]
    pub const fn required(&self) -> Option<bool> {
        self.required
    }

    #[must_use]
    pub const fn valid(&self) -> Option<bool> {
        self.valid
    }

    #[must_use]
    pub const fn placeholder_shown(&self) -> bool {
        self.placeholder_shown
    }

    #[must_use]
    pub const fn modal(&self) -> bool {
        self.modal
    }

    #[must_use]
    pub const fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    #[must_use]
    pub const fn popover_open(&self) -> bool {
        self.popover_open
    }

    #[must_use]
    pub const fn default_state(&self) -> bool {
        self.default
    }

    #[must_use]
    pub const fn indeterminate(&self) -> bool {
        self.indeterminate
    }

    #[must_use]
    pub const fn read_write(&self) -> Option<bool> {
        self.read_write
    }

    #[must_use]
    pub const fn range_state(&self) -> Option<RangeState> {
        self.range_state
    }

    #[must_use]
    pub const fn has_flag(&self, flag: StateFlag) -> bool {
        match flag {
            StateFlag::Hovered => self.hovered,
            StateFlag::Active => self.active,
            StateFlag::Focused => self.focused,
            StateFlag::FocusVisible => self.focus_visible,
            StateFlag::FocusWithin => self.focus_within,
            StateFlag::PointerCaptured => self.pointer_captured,
            StateFlag::Disabled => matches!(self.enabled, Some(false)),
            StateFlag::Enabled => matches!(self.enabled, Some(true)),
            StateFlag::Selected => self.selected,
            StateFlag::Pressed => self.pressed,
            StateFlag::Checked => matches!(self.checked, Some(true)),
            StateFlag::Expanded => matches!(self.expanded, Some(true)),
            StateFlag::Required => matches!(self.required, Some(true)),
            StateFlag::Optional => matches!(self.required, Some(false)),
            StateFlag::Valid => matches!(self.valid, Some(true)),
            StateFlag::Invalid => matches!(self.valid, Some(false)),
            StateFlag::PlaceholderShown => self.placeholder_shown,
            StateFlag::Modal => self.modal,
            StateFlag::Fullscreen => self.fullscreen,
            StateFlag::PopoverOpen => self.popover_open,
            StateFlag::Default => self.default,
            StateFlag::Indeterminate => self.indeterminate,
            StateFlag::ReadOnly => matches!(self.read_write, Some(false)),
            StateFlag::ReadWrite => matches!(self.read_write, Some(true)),
            StateFlag::InRange => matches!(self.range_state, Some(RangeState::InRange)),
            StateFlag::OutOfRange => matches!(self.range_state, Some(RangeState::OutOfRange)),
        }
    }

    #[must_use]
    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.enabled = Some(!disabled);
        self
    }

    #[must_use]
    pub const fn with_hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    #[must_use]
    pub const fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[must_use]
    pub const fn with_focus_visible(mut self, focus_visible: bool) -> Self {
        self.focus_visible = focus_visible;
        self
    }

    #[must_use]
    pub const fn with_focus_within(mut self, focus_within: bool) -> Self {
        self.focus_within = focus_within;
        self
    }

    #[must_use]
    pub const fn with_pointer_captured(mut self, pointer_captured: bool) -> Self {
        self.pointer_captured = pointer_captured;
        self
    }

    #[must_use]
    pub const fn with_enabled(mut self, enabled: Option<bool>) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub const fn with_pressed(mut self, pressed: bool) -> Self {
        self.pressed = pressed;
        self
    }

    #[must_use]
    pub const fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    #[must_use]
    pub const fn with_expanded(mut self, expanded: Option<bool>) -> Self {
        self.expanded = expanded;
        self
    }

    #[must_use]
    pub const fn with_required(mut self, required: Option<bool>) -> Self {
        self.required = required;
        self
    }

    #[must_use]
    pub const fn with_valid(mut self, valid: Option<bool>) -> Self {
        self.valid = valid;
        self
    }

    #[must_use]
    pub const fn with_placeholder_shown(mut self, placeholder_shown: bool) -> Self {
        self.placeholder_shown = placeholder_shown;
        self
    }

    #[must_use]
    pub const fn with_modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    #[must_use]
    pub const fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    #[must_use]
    pub const fn with_popover_open(mut self, popover_open: bool) -> Self {
        self.popover_open = popover_open;
        self
    }

    #[must_use]
    pub const fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    #[must_use]
    pub const fn with_indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    #[must_use]
    pub const fn with_read_write(mut self, read_write: Option<bool>) -> Self {
        self.read_write = read_write;
        self
    }

    #[must_use]
    pub const fn with_range_state(mut self, range_state: Option<RangeState>) -> Self {
        self.range_state = range_state;
        self
    }
}

fn validate_ident(value: &str, field: &str, allow_separators: bool) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} cannot be empty"),
        ));
    }

    for ch in trimmed.chars() {
        if ch == '\0' || ch.is_control() || ch.is_whitespace() {
            return Err(Error::new(
                ErrorCode::InvalidString,
                format!("{field} contains unsupported character U+{:04X}", ch as u32),
            ));
        }

        let valid = ch.is_ascii_alphanumeric()
            || matches!(ch, '_' | '-')
            || (allow_separators && matches!(ch, '.' | ':' | '/'));
        if !valid {
            return Err(Error::new(
                ErrorCode::InvalidString,
                format!("{field} contains unsupported character `{ch}`"),
            ));
        }
    }

    Ok(trimmed.to_owned())
}

fn validate_value(value: &str) -> Result<String> {
    if value
        .chars()
        .any(|ch| ch == '\0' || (ch.is_control() && !matches!(ch, '\n' | '\r' | '\t')))
    {
        return Err(Error::new(
            ErrorCode::InvalidString,
            "value contains unsupported control character",
        ));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorCode, StateFlag};

    #[test]
    fn selector_tokens_trim_and_reject_invalid_idents() {
        assert_eq!(StyleTag::new(" button ").unwrap().as_str(), "button");
        assert_eq!(
            StyleClass::new("primary_action").unwrap().as_str(),
            "primary_action"
        );
        assert_eq!(
            StyleAttributeName::new("data-id").unwrap().as_str(),
            "data-id"
        );

        for value in [
            "",
            "   ",
            "two words",
            "bad.name",
            "bad:name",
            "bad/name",
            "\0",
        ] {
            assert_eq!(
                StyleTag::new(value).unwrap_err().code(),
                ErrorCode::InvalidString
            );
            assert_eq!(
                StyleClass::new(value).unwrap_err().code(),
                ErrorCode::InvalidString
            );
            assert_eq!(
                StyleAttributeName::new(value).unwrap_err().code(),
                ErrorCode::InvalidString
            );
        }
    }

    #[test]
    fn style_key_allows_path_key_separators() {
        let key = StyleKey::new(" route:/primary.card ").unwrap();

        assert_eq!(key.as_str(), "route:/primary.card");
        assert_eq!(
            StyleKey::new("two words").unwrap_err().code(),
            ErrorCode::InvalidString
        );
    }

    #[test]
    fn attribute_value_rejects_only_unsupported_controls() {
        assert_eq!(
            StyleAttributeValue::new("line\nrow\tcell\rnext")
                .unwrap()
                .as_str(),
            "line\nrow\tcell\rnext"
        );
        assert_eq!(StyleAttributeValue::empty().as_str(), "");
        assert_eq!(
            StyleAttributeValue::new("\u{0001}").unwrap_err().code(),
            ErrorCode::InvalidString
        );
    }

    #[test]
    fn style_attribute_and_state_are_style_owned_facts() {
        let attribute = StyleAttribute::new(
            StyleAttributeName::new("data-state").unwrap(),
            StyleAttributeValue::new("ready").unwrap(),
        );
        assert_eq!(attribute.name().as_str(), "data-state");
        assert_eq!(attribute.value().as_str(), "ready");

        let state = StyleState::default()
            .with_hovered(true)
            .with_checked(Some(true))
            .with_expanded(Some(false));
        assert!(state.has_flag(StateFlag::Hovered));
        assert!(state.has_flag(StateFlag::Checked));
        assert!(!state.has_flag(StateFlag::Expanded));
    }
}

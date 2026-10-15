//! Fixed intrinsic form-control state from HTML 877d6146487f.
//!
//! These values own fixed slots and local, allocation-free control decisions.
//! They do not establish a captured subject, execute tree effects, normalize
//! strings, move a selection, or dispatch events. Adapters retain those duties.

use crate::HtmlInputValueComparison;

/// Additional Input/textarea value-setter effects after generic value clamping.
/// The caller executes the selected effect with actual value length and platform
/// direction support; this decision does not itself move a cursor or selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTextControlValueSetterFinish {
    NoAdditionalEffect,
    SetEndCursorUnselectAndNoneDirection,
}

/// Textarea's additional value-setter effect from a completed API comparison.
///
/// Compare actual old/new API values after their newline normalization, not raw
/// storage. After the generic relevant-value clamp, a changed API value always
/// requests end cursor, unselection and none direction, including when a range
/// survives clamping. Input's separate finish decision has additional conditions.
///
/// The [textarea setter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-textarea-value)
/// sets raw storage and dirty state independently. The caller owns those effects,
/// normalization/comparison work, actual new API length and none-direction support.
pub fn textarea_value_setter_finish(
    comparison: HtmlInputValueComparison,
) -> HtmlTextControlValueSetterFinish {
    if comparison.is_equal() {
        HtmlTextControlValueSetterFinish::NoAdditionalEffect
    } else {
        HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
    }
}

/// The local raw-value action required by textarea child changes or reset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTextareaRawValueAction {
    /// Preserve the current raw value.
    Keep,
    /// Replace the raw value with actual immediate child Text content.
    ReplaceWithChildText,
}

/// Textarea's dirty-value and user-validity slots.
///
/// HTML's [textarea rules](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-textarea-element)
/// govern these transitions. Raw UTF-16 storage and API newline normalization
/// remain with the consuming adapter. Construction does not certify history or
/// capture qualification; both Boolean slot values are independently retained.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlTextareaFlags {
    dirty_value: bool,
    user_validity: bool,
}

impl HtmlTextareaFlags {
    /// Retain established slot values. A newly created textarea uses `default()`.
    pub const fn new(dirty_value: bool, user_validity: bool) -> Self {
        Self {
            dirty_value,
            user_validity,
        }
    }

    pub const fn dirty_value(&self) -> bool {
        self.dirty_value
    }
    pub const fn user_validity(&self) -> bool {
        self.user_validity
    }

    /// The local decision for actual children-changed steps.
    pub const fn children_changed(&self) -> HtmlTextareaRawValueAction {
        if self.dirty_value {
            HtmlTextareaRawValueAction::Keep
        } else {
            HtmlTextareaRawValueAction::ReplaceWithChildText
        }
    }

    /// Clear both flags for reset, including the reset required by parser pop.
    /// The caller then replaces raw storage from actual child text.
    pub fn reset(&mut self) -> HtmlTextareaRawValueAction {
        self.dirty_value = false;
        self.user_validity = false;
        HtmlTextareaRawValueAction::ReplaceWithChildText
    }

    /// Propagate only the source dirty flag in textarea cloning steps.
    /// The caller independently copies raw storage; target user validity stays.
    pub fn clone_dirty_value_from(&mut self, source: &Self) {
        self.dirty_value = source.dirty_value;
    }

    /// Local flag effect of a raw-value change by the user or the value setter.
    /// This does not perform the setter's API comparison or selection effects.
    pub fn mark_dirty_value(&mut self) {
        self.dirty_value = true;
    }
}

/// Select's intrinsic user-validity slot; this stores no option-list topology.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlSelectFlags {
    user_validity: bool,
}

impl HtmlSelectFlags {
    /// Retain an established slot; newly created selects use `default()`.
    pub const fn new(user_validity: bool) -> Self {
        Self { user_validity }
    }
    pub const fn user_validity(&self) -> bool {
        self.user_validity
    }

    /// The flag step of the [select reset algorithm](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-select-element).
    /// Option reset, normalization and selectedcontent updates follow in order
    /// in the adapter, using its actual option list.
    pub fn reset(&mut self) {
        self.user_validity = false;
    }
}

/// Option's selectedness and dirtiness, separate from its nearest-select cache.
///
/// The [option rules](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-option-element)
/// define the local transitions; list order, disabledness and tree effects are
/// supplied and executed by the actual adapter, never inferred from these flags.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlOptionFlags {
    selectedness: bool,
    dirtiness: bool,
}

impl HtmlOptionFlags {
    /// Retain established slots. A fresh option uses `default()` before its
    /// actual initial selected-attribute steps.
    pub const fn new(selectedness: bool, dirtiness: bool) -> Self {
        Self {
            selectedness,
            dirtiness,
        }
    }
    pub const fn selectedness(&self) -> bool {
        self.selectedness
    }
    pub const fn dirtiness(&self) -> bool {
        self.dirtiness
    }

    /// Added/removed selected-attribute steps, returning source `hasChanged`.
    /// Even `false` requires the caller to reset the cached nearest select's
    /// selectedness when that select exists. A set of an already-present
    /// attribute is not an Added trigger.
    pub fn selected_attribute_changed(&mut self, present: bool) -> bool {
        if self.dirtiness || self.selectedness == present {
            return false;
        }
        self.selectedness = present;
        true
    }

    /// The per-option steps of select reset, before list normalization.
    pub fn reset(&mut self, selected_attribute_present: bool) {
        self.selectedness = selected_attribute_present;
        self.dirtiness = false;
    }

    /// Local steps of the native selected setter. The adapter must subsequently
    /// ask for a reset, even when the supplied value equals the prior value.
    pub fn selected_setter(&mut self, selected: bool) {
        self.selectedness = selected;
        self.dirtiness = true;
    }

    /// Apply one selectedness write chosen by the actual select algorithm.
    /// Dirtiness is preserved. This operation does not choose an option, prove
    /// its list membership, or implement normalization by itself.
    pub fn set_selectedness_for_select(&mut self, selected: bool) {
        self.selectedness = selected;
    }
}

//! Fixed intrinsic form-control state from HTML 877d6146487f.
//!
//! These values own fixed slots and local, allocation-free control decisions.
//! They do not establish a captured subject, execute tree effects, normalize
//! strings, move a selection, or dispatch events. Adapters retain those duties.

use crate::HtmlInputValueComparison;

/// Actual source of an output element's default value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOutputDefaultSource {
    Override,
    DescendantText,
}

/// Select from established actual override presence, including present-empty.
/// Missing capture evidence is not a false presence observation. The caller
/// reads its real nullable override or ordinary descendant Text as selected.
pub const fn output_default_source(has_override: bool) -> HtmlOutputDefaultSource {
    if has_override {
        HtmlOutputDefaultSource::Override
    } else {
        HtmlOutputDefaultSource::DescendantText
    }
}

/// One output value/defaultValue setter or per-control reset operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOutputOperation {
    ValueSetter,
    /// Presence of the actual nullable override when the operation begins.
    DefaultValueSetter {
        has_override: bool,
    },
    Reset,
}

/// The next source-ordered Output operation for the consuming owner to perform.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOutputAction {
    /// Resolve the actual default using output_default_source and retain it.
    ReadDefaultValue,
    StoreReadDefaultAsOverride,
    ReplaceAllWithSuppliedValue,
    StoreSuppliedValueAsOverride,
    ReplaceAllWithReadDefault,
    ClearDefaultOverride,
    Complete,
}

/// An acknowledgement that does not match the pending action, or is terminal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlOutputCursorError;

impl std::fmt::Display for HtmlOutputCursorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("output acknowledgement does not match its pending action")
    }
}

impl std::error::Error for HtmlOutputCursorError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputPhase {
    ReadForValue,
    StoreReadDefault,
    ReplaceSupplied,
    StoreSupplied,
    ReadForReset,
    ReplaceDefault,
    ClearOverride,
    Complete,
}

/// Fixed, allocation-free Output setter/reset order from HTML's
/// [output rules](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#the-output-element).
///
/// The consumer owns actual nullable override/string storage, supplied/read
/// strings, tree replacement, admission, cancellation and publication. Acknowledge
/// an action only after its real operation succeeds. ReadDefaultValue uses the
/// pre-replacement tree/override. No string is normalized and equal strings do
/// not skip the required tree replacement. Reset clears override only after its
/// replacement, while a value setter stores the read default before replacing.
/// This is not a form-wide event-bearing reset or a capture-availability model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlOutputCursor {
    phase: OutputPhase,
}

impl HtmlOutputCursor {
    pub const fn new(operation: HtmlOutputOperation) -> Self {
        let phase = match operation {
            HtmlOutputOperation::ValueSetter => OutputPhase::ReadForValue,
            HtmlOutputOperation::DefaultValueSetter {
                has_override: false,
            } => OutputPhase::ReplaceSupplied,
            HtmlOutputOperation::DefaultValueSetter { has_override: true } => {
                OutputPhase::StoreSupplied
            }
            HtmlOutputOperation::Reset => OutputPhase::ReadForReset,
        };
        Self { phase }
    }

    pub const fn action(&self) -> HtmlOutputAction {
        match self.phase {
            OutputPhase::ReadForValue | OutputPhase::ReadForReset => {
                HtmlOutputAction::ReadDefaultValue
            }
            OutputPhase::StoreReadDefault => HtmlOutputAction::StoreReadDefaultAsOverride,
            OutputPhase::ReplaceSupplied => HtmlOutputAction::ReplaceAllWithSuppliedValue,
            OutputPhase::StoreSupplied => HtmlOutputAction::StoreSuppliedValueAsOverride,
            OutputPhase::ReplaceDefault => HtmlOutputAction::ReplaceAllWithReadDefault,
            OutputPhase::ClearOverride => HtmlOutputAction::ClearDefaultOverride,
            OutputPhase::Complete => HtmlOutputAction::Complete,
        }
    }

    /// A mismatched acknowledgement or one after Complete preserves the cursor.
    /// Never acknowledge missing observations or failed/unperformed operations.
    pub fn acknowledge(
        &mut self,
        completed: HtmlOutputAction,
    ) -> Result<(), HtmlOutputCursorError> {
        if completed != self.action() {
            return Err(HtmlOutputCursorError);
        }
        let next = match self.phase {
            OutputPhase::ReadForValue => OutputPhase::StoreReadDefault,
            OutputPhase::StoreReadDefault => OutputPhase::ReplaceSupplied,
            OutputPhase::ReadForReset => OutputPhase::ReplaceDefault,
            OutputPhase::ReplaceDefault => OutputPhase::ClearOverride,
            OutputPhase::ReplaceSupplied
            | OutputPhase::StoreSupplied
            | OutputPhase::ClearOverride => OutputPhase::Complete,
            OutputPhase::Complete => return Err(HtmlOutputCursorError),
        };
        self.phase = next;
        Ok(())
    }
}

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
        self.set_selectedness_and_dirty(selected);
    }

    /// Select the remembered first match of a Select value/index setter.
    /// The Select setter finishes its whole deselection scan before this write
    /// and updates selectedcontent afterward; it does not ask for a reset.
    pub fn select_for_select_setter(&mut self) {
        self.set_selectedness_and_dirty(true);
    }

    fn set_selectedness_and_dirty(&mut self, selected: bool) {
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

//! Select decisions from HTML 877d6146487f, without a tree or mutation owner.

use crate::KnownHtmlElement;

/// HTML display-size fallback for an absent or failed size attribute.
/// A successfully recovered zero is not a fallback. This is not size IDL.
pub const fn select_display_size_default(multiple: bool) -> u8 {
    if multiple { 4 } else { 1 }
}

/// The source operation surrounding selectedness normalization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectOperation {
    /// Run only the selectedness setting algorithm and return its boolean.
    Normalize,
    /// Normalize, then refresh selectedcontent if either change flag is true.
    ResetSelectedness { has_changed: bool },
    /// Clear user validity, reset every option, normalize, then refresh.
    ResetControl,
    /// Normalize and refresh, even when normalization changes nothing.
    AskForReset,
    /// The multiple attribute was added; request the prescribed refresh.
    MultipleAdded,
    /// Keep the first selection before normalization and the forced refresh.
    MultipleRemoved,
}

/// The next fixed select operation; ordinals address one complete option list.
///
/// The adapter admits each actual read/write before performing it. Writes use
/// [`crate::HtmlOptionFlags`]'s owned transitions; no callback or event dispatch
/// is requested. A refresh means HTML's selectedcontent update operation, which
/// itself returns immediately while multiple is present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectAction {
    /// Apply [`crate::HtmlSelectFlags::reset`] to the actual select flags.
    ResetSelectFlags,
    /// Apply [`crate::HtmlOptionFlags::reset`] using this option's actual
    /// selected-attribute presence, clearing dirtiness before normalization.
    ResetOption {
        ordinal: usize,
    },
    ReadSelectedness {
        ordinal: usize,
    },
    /// Apply [`crate::HtmlOptionFlags::set_selectedness_for_select`], preserving
    /// dirtiness; never run the selected setter or change an authored attribute.
    WriteSelectedness {
        ordinal: usize,
        selected: bool,
    },
    ReadDisabled {
        ordinal: usize,
    },
    /// Observe HTML's recovered display-size concept, not size IDL or authored
    /// positive-number conformance. Recovered zero is a known non-one value.
    ReadDisplaySize,
    UpdateSelectedcontent,
    /// The boolean returned by the ordinary normalization pass, not a summary
    /// of all reset/prepass/selectedcontent writes in the surrounding operation.
    Complete {
        normalization_changed: bool,
    },
}

/// Established data or an acknowledgement for the pending select action.
///
/// False is an observed false value, never missing evidence. An adapter with
/// incomplete options, flags or size must stop without replying/completing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectReply {
    Selectedness(bool),
    Disabled(bool),
    DisplaySizeIsOne(bool),
    Applied,
}

/// A contradictory operation context or a reply to the wrong pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectCursorError {
    MultipleContext,
    Reply,
}
impl std::fmt::Display for HtmlSelectCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::MultipleContext => "select operation contradicts multiple presence",
            Self::Reply => "select reply does not match its pending action",
        })
    }
}
impl std::error::Error for HtmlSelectCursorError {}

#[derive(Clone, Copy, Debug)]
enum Phase {
    ResetSelect,
    ResetOption(usize),
    FirstRead(usize),
    FirstWrite(usize),
    Read(usize),
    Deselect { current: usize, previous: usize },
    Disabled(usize),
    Size,
    Select(usize),
    Refresh,
    Complete,
}

/// Allocation-free, constant-state select algorithm over an established list.
///
/// The adapter owns candidate enumeration and their qualified identities. The
/// supplied extent must describe the complete HTML option list in tree order;
/// an unavailable list is not an empty list. Keep that list and select context
/// coherent through completion. This cursor does not inspect topology or own
/// cancellation, capacity, transaction publication or selectedcontent cloning.
#[derive(Clone, Copy, Debug)]
pub struct HtmlSelectCursor {
    operation: HtmlSelectOperation,
    multiple: bool,
    option_count: usize,
    first_enabled: Option<usize>,
    last_selected: Option<usize>,
    first_selected_seen: bool,
    changed: bool,
    phase: Phase,
}
impl HtmlSelectCursor {
    pub fn new(
        operation: HtmlSelectOperation,
        multiple: bool,
        option_count: usize,
    ) -> Result<Self, HtmlSelectCursorError> {
        if matches!(operation, HtmlSelectOperation::MultipleAdded) && !multiple
            || matches!(operation, HtmlSelectOperation::MultipleRemoved) && multiple
        {
            return Err(HtmlSelectCursorError::MultipleContext);
        }
        let mut cursor = Self {
            operation,
            multiple,
            option_count,
            first_enabled: None,
            last_selected: None,
            first_selected_seen: false,
            changed: false,
            phase: Phase::Complete,
        };
        if operation == HtmlSelectOperation::ResetControl {
            cursor.phase = Phase::ResetSelect;
        } else if operation == HtmlSelectOperation::MultipleRemoved && option_count != 0 {
            cursor.phase = Phase::FirstRead(0);
        } else {
            cursor.begin_normalization();
        }
        Ok(cursor)
    }
    pub fn action(&self) -> HtmlSelectAction {
        match self.phase {
            Phase::ResetSelect => HtmlSelectAction::ResetSelectFlags,
            Phase::ResetOption(ordinal) => HtmlSelectAction::ResetOption { ordinal },
            Phase::FirstRead(ordinal) | Phase::Read(ordinal) => {
                HtmlSelectAction::ReadSelectedness { ordinal }
            }
            Phase::FirstWrite(ordinal) => HtmlSelectAction::WriteSelectedness {
                ordinal,
                selected: false,
            },
            Phase::Deselect { previous, .. } => HtmlSelectAction::WriteSelectedness {
                ordinal: previous,
                selected: false,
            },
            Phase::Disabled(ordinal) => HtmlSelectAction::ReadDisabled { ordinal },
            Phase::Size => HtmlSelectAction::ReadDisplaySize,
            Phase::Select(ordinal) => HtmlSelectAction::WriteSelectedness {
                ordinal,
                selected: true,
            },
            Phase::Refresh => HtmlSelectAction::UpdateSelectedcontent,
            Phase::Complete => HtmlSelectAction::Complete {
                normalization_changed: self.changed,
            },
        }
    }
    /// Advance only after the adapter has admitted and fulfilled the action.
    /// Invalid replies preserve every cursor field and the pending action.
    pub fn reply(&mut self, reply: HtmlSelectReply) -> Result<(), HtmlSelectCursorError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }
    fn begin_normalization(&mut self) {
        if self.multiple || self.option_count == 0 {
            self.finish_normalization();
        } else {
            self.phase = Phase::Read(0);
        }
    }
    fn finish_normalization(&mut self) {
        let refresh = match self.operation {
            HtmlSelectOperation::Normalize => false,
            HtmlSelectOperation::ResetSelectedness { has_changed } => has_changed || self.changed,
            _ => true,
        };
        self.phase = if refresh {
            Phase::Refresh
        } else {
            Phase::Complete
        };
    }
    fn after_selectedness(&mut self, ordinal: usize) {
        if self.first_enabled.is_none() {
            self.phase = Phase::Disabled(ordinal);
        } else {
            self.next_option(ordinal);
        }
    }
    fn next_option(&mut self, ordinal: usize) {
        // ordinal < option_count; even an extent of usize::MAX cannot overflow.
        let next = ordinal + 1;
        if next < self.option_count {
            self.phase = Phase::Read(next);
        } else if self.last_selected.is_none() && self.first_enabled.is_some() {
            self.phase = Phase::Size;
        } else {
            self.finish_normalization();
        }
    }
    fn advance(&mut self, reply: HtmlSelectReply) -> Result<(), HtmlSelectCursorError> {
        match (self.phase, reply) {
            (Phase::ResetSelect, HtmlSelectReply::Applied) => {
                if self.option_count == 0 {
                    self.begin_normalization();
                } else {
                    self.phase = Phase::ResetOption(0);
                }
            }
            (Phase::ResetOption(ordinal), HtmlSelectReply::Applied) => {
                let next = ordinal + 1;
                if next < self.option_count {
                    self.phase = Phase::ResetOption(next);
                } else {
                    self.begin_normalization();
                }
            }
            (Phase::FirstRead(ordinal), HtmlSelectReply::Selectedness(selected)) => {
                if selected && self.first_selected_seen {
                    self.phase = Phase::FirstWrite(ordinal);
                } else {
                    self.first_selected_seen |= selected;
                    self.next_first(ordinal);
                }
            }
            (Phase::FirstWrite(ordinal), HtmlSelectReply::Applied) => self.next_first(ordinal),
            (Phase::Read(ordinal), HtmlSelectReply::Selectedness(selected)) => {
                if selected {
                    if let Some(previous) = self.last_selected {
                        self.phase = Phase::Deselect {
                            current: ordinal,
                            previous,
                        };
                        return Ok(());
                    }
                    self.last_selected = Some(ordinal);
                }
                self.after_selectedness(ordinal);
            }
            (Phase::Deselect { current, .. }, HtmlSelectReply::Applied) => {
                self.changed = true;
                self.last_selected = Some(current);
                self.after_selectedness(current);
            }
            (Phase::Disabled(ordinal), HtmlSelectReply::Disabled(disabled)) => {
                if !disabled {
                    self.first_enabled = Some(ordinal);
                }
                self.next_option(ordinal);
            }
            (Phase::Size, HtmlSelectReply::DisplaySizeIsOne(one)) => {
                if one {
                    self.phase = Phase::Select(self.first_enabled.expect("fallback candidate"));
                } else {
                    self.finish_normalization();
                }
            }
            (Phase::Select(_), HtmlSelectReply::Applied) => {
                self.changed = true;
                self.finish_normalization();
            }
            (Phase::Refresh, HtmlSelectReply::Applied) => self.phase = Phase::Complete,
            _ => return Err(HtmlSelectCursorError::Reply),
        }
        Ok(())
    }
    fn next_first(&mut self, ordinal: usize) {
        let next = ordinal + 1;
        if next < self.option_count {
            self.phase = Phase::FirstRead(next);
        } else {
            self.begin_normalization();
        }
    }
}

/// Intrinsic option-list classification; adapters perform actual traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectOptionRule {
    IncludeAndPrune,
    Prune,
    Descend,
}
pub fn select_option_rule(
    kind: Option<KnownHtmlElement>,
    inside_optgroup: bool,
) -> HtmlSelectOptionRule {
    match kind {
        Some(KnownHtmlElement::Option) => HtmlSelectOptionRule::IncludeAndPrune,
        Some(KnownHtmlElement::Select | KnownHtmlElement::Hr | KnownHtmlElement::Datalist) => {
            HtmlSelectOptionRule::Prune
        }
        Some(KnownHtmlElement::Optgroup) if inside_optgroup => HtmlSelectOptionRule::Prune,
        _ => HtmlSelectOptionRule::Descend,
    }
}

/// Action for one established option ancestor, after own disabled presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOptionDisabledAncestor {
    Continue,
    Enabled,
    ReadOptgroupDisabled,
}
pub fn option_disabled_ancestor(kind: Option<KnownHtmlElement>) -> HtmlOptionDisabledAncestor {
    match kind {
        Some(
            KnownHtmlElement::Option
            | KnownHtmlElement::Select
            | KnownHtmlElement::Datalist
            | KnownHtmlElement::Hr,
        ) => HtmlOptionDisabledAncestor::Enabled,
        Some(KnownHtmlElement::Optgroup) => HtmlOptionDisabledAncestor::ReadOptgroupDisabled,
        _ => HtmlOptionDisabledAncestor::Continue,
    }
}

/// Decision for option's nearest-ancestor-select algorithm, not option-list
/// descent or selectedcontent disabledness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOptionSelectAncestorAction {
    Continue,
    NearestSelect,
    NoSelect,
}
/// HTML #get-the-nearest-ancestor-select given actual nearest-first ancestors.
///
/// The adapter retains the returned select identity and owns ancestry/coverage.
/// `None` in observe means an established other kind, not missing topology.
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlOptionSelectAncestors {
    optgroup_seen: bool,
    terminal: Option<HtmlOptionSelectAncestorAction>,
}
impl HtmlOptionSelectAncestors {
    pub fn observe(&mut self, kind: Option<KnownHtmlElement>) -> HtmlOptionSelectAncestorAction {
        if let Some(result) = self.terminal {
            return result;
        }
        let action = match kind {
            Some(KnownHtmlElement::Datalist | KnownHtmlElement::Hr | KnownHtmlElement::Option) => {
                HtmlOptionSelectAncestorAction::NoSelect
            }
            Some(KnownHtmlElement::Optgroup) if self.optgroup_seen => {
                HtmlOptionSelectAncestorAction::NoSelect
            }
            Some(KnownHtmlElement::Optgroup) => {
                self.optgroup_seen = true;
                HtmlOptionSelectAncestorAction::Continue
            }
            Some(KnownHtmlElement::Select) => HtmlOptionSelectAncestorAction::NearestSelect,
            _ => HtmlOptionSelectAncestorAction::Continue,
        };
        if action != HtmlOptionSelectAncestorAction::Continue {
            self.terminal = Some(action);
        }
        action
    }
    /// End only after ordinary ancestors are established exhausted. Unavailable
    /// parent evidence must not call this or establish a null cached select.
    pub fn finish_complete(&mut self) -> HtmlOptionSelectAncestorAction {
        *self
            .terminal
            .get_or_insert(HtmlOptionSelectAncestorAction::NoSelect)
    }
}

/// One selectedcontent ancestor decision in nearest-to-farthest order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectedcontentAncestorAction {
    Continue,
    RememberNearestSelect,
    StopDisabled,
}
/// Fixed intrinsic state; the adapter retains the actual nearest select ID.
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlSelectedcontentAncestors {
    nearest_select: bool,
    disabled: bool,
}
impl HtmlSelectedcontentAncestors {
    pub fn observe(&mut self, kind: Option<KnownHtmlElement>) -> HtmlSelectedcontentAncestorAction {
        if self.disabled {
            return HtmlSelectedcontentAncestorAction::StopDisabled;
        }
        match kind {
            Some(KnownHtmlElement::Select) if !self.nearest_select => {
                self.nearest_select = true;
                HtmlSelectedcontentAncestorAction::RememberNearestSelect
            }
            Some(
                KnownHtmlElement::Select
                | KnownHtmlElement::Option
                | KnownHtmlElement::Selectedcontent,
            ) => {
                self.disabled = true;
                HtmlSelectedcontentAncestorAction::StopDisabled
            }
            _ => HtmlSelectedcontentAncestorAction::Continue,
        }
    }
    pub fn disabled(&self) -> bool {
        self.disabled
    }
    pub fn has_nearest_select(&self) -> bool {
        self.nearest_select
    }
}

/// Per-target selectedcontent effect after a complete first-selected scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectedcontentUpdate {
    Skip,
    Clear,
    CloneOptionChildren {
        ordinal: usize,
    },
    /// Selection/order is incomplete; no clear or clone is authorized.
    Unavailable,
}
/// A first-selected scan distinguishes proven absence from missing evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectFirstSelection {
    NoSelection,
    Selected { ordinal: usize },
    Unavailable,
}
/// An enabled single-select target requires established selection evidence.
pub fn selectedcontent_update(
    multiple: bool,
    disabled: bool,
    first_selected: HtmlSelectFirstSelection,
) -> HtmlSelectedcontentUpdate {
    if multiple || disabled {
        HtmlSelectedcontentUpdate::Skip
    } else {
        match first_selected {
            HtmlSelectFirstSelection::NoSelection => HtmlSelectedcontentUpdate::Clear,
            HtmlSelectFirstSelection::Selected { ordinal } => {
                HtmlSelectedcontentUpdate::CloneOptionChildren { ordinal }
            }
            HtmlSelectFirstSelection::Unavailable => HtmlSelectedcontentUpdate::Unavailable,
        }
    }
}

/// The next read or established result of the first-selected operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectFirstSelectedAction {
    ReadSelectedness { ordinal: usize },
    Complete(HtmlSelectFirstSelection),
}
/// A first-selected scan over a complete adapter-owned option-list extent.
/// Unknown selectedness is supplied explicitly and never becomes no selection.
#[derive(Clone, Copy, Debug)]
pub struct HtmlSelectFirstSelectedCursor {
    count: usize,
    ordinal: usize,
    result: Option<HtmlSelectFirstSelection>,
}
impl HtmlSelectFirstSelectedCursor {
    pub fn new(option_count: usize) -> Self {
        Self {
            count: option_count,
            ordinal: 0,
            result: (option_count == 0).then_some(HtmlSelectFirstSelection::NoSelection),
        }
    }
    pub fn action(&self) -> HtmlSelectFirstSelectedAction {
        self.result.map_or(
            HtmlSelectFirstSelectedAction::ReadSelectedness {
                ordinal: self.ordinal,
            },
            HtmlSelectFirstSelectedAction::Complete,
        )
    }
    pub fn reply(&mut self, selected: Option<bool>) -> Result<(), HtmlSelectCursorError> {
        if self.result.is_some() {
            return Err(HtmlSelectCursorError::Reply);
        }
        match selected {
            None => self.result = Some(HtmlSelectFirstSelection::Unavailable),
            Some(true) => {
                self.result = Some(HtmlSelectFirstSelection::Selected {
                    ordinal: self.ordinal,
                })
            }
            Some(false) => {
                self.ordinal += 1; // ordinal < count, including usize::MAX extent.
                if self.ordinal == self.count {
                    self.result = Some(HtmlSelectFirstSelection::NoSelection);
                }
            }
        }
        Ok(())
    }
}

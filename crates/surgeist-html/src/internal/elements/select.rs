//! Select decisions from HTML 877d6146487f, without a tree or mutation owner.

use crate::vocabulary::KnownHtmlElement;

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
    ResetSelectedness {
        /// Whether the caller already knows an option changed, which forces the refresh even when
        /// the normalization changes nothing.
        has_changed: bool,
    },
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
/// [`crate::internal::elements::control_values::HtmlOptionFlags`]'s owned transitions; no callback or event dispatch
/// is requested. A refresh means HTML's selectedcontent update operation, which
/// itself returns immediately while multiple is present.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectAction {
    /// Apply [`crate::internal::elements::control_values::HtmlSelectFlags::reset`] to the actual select flags.
    ResetSelectFlags,
    /// Apply [`crate::internal::elements::control_values::HtmlOptionFlags::reset`] using this option's actual
    /// selected-attribute presence, clearing dirtiness before normalization.
    ResetOption {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Read the selectedness of one option. The adapter answers with
    /// `HtmlSelectReply::Selectedness`; an unavailable read is not an answer.
    ReadSelectedness {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Apply [`crate::internal::elements::control_values::HtmlOptionFlags::set_selectedness_for_select`], preserving
    /// dirtiness; never run the selected setter or change an authored attribute.
    WriteSelectedness {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
        /// The selectedness to write: true selects the option, false deselects it.
        selected: bool,
    },
    /// Read whether one option is disabled (HTML877d #concept-option-disabled). The adapter answers
    /// with `HtmlSelectReply::Disabled`.
    ReadDisabled {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Observe HTML's recovered display-size concept, not size IDL or authored
    /// positive-number conformance. Recovered zero is a known non-one value.
    ReadDisplaySize,
    /// Run HTML's selectedcontent update for the select (HTML877d #the-selectedcontent-element).
    /// The update itself returns immediately while `multiple` is present.
    UpdateSelectedcontent,
    /// The boolean returned by the ordinary normalization pass, not a summary
    /// of all reset/prepass/selectedcontent writes in the surrounding operation.
    Complete {
        /// Whether the selectedness setting algorithm changed an option (HTML877d
        /// #selectedness-setting-algorithm), as one boolean for the normalization pass alone.
        normalization_changed: bool,
    },
}

/// Established data or an acknowledgement for the pending select action.
///
/// False is an observed false value, never missing evidence. An adapter with
/// incomplete options, flags or size must stop without replying/completing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectReply {
    /// The selectedness of the option named by `ReadSelectedness`: true when it is selected.
    Selectedness(bool),
    /// Whether the option named by `ReadDisabled` is disabled.
    Disabled(bool),
    /// Whether the select's display size is 1, answering `ReadDisplaySize`.
    DisplaySizeIsOne(bool),
    /// The adapter performed the pending write or refresh.
    Applied,
}

/// A contradictory operation context or a reply to the wrong pending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSelectCursorError {
    kind: HtmlSelectCursorErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlSelectCursorErrorKind {
    MultipleContext,
    Reply,
}

impl HtmlSelectCursorErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlSelectCursorError {
        HtmlSelectCursorError { kind: self }
    }
}

impl From<HtmlSelectCursorErrorKind> for HtmlSelectCursorError {
    fn from(kind: HtmlSelectCursorErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlSelectCursorError {
    /// Whether the cause is: select operation contradicts multiple presence.
    pub fn is_multiple_context(&self) -> bool {
        matches!(self.kind, HtmlSelectCursorErrorKind::MultipleContext)
    }
    /// Whether the cause is: select reply does not match its pending action.
    pub fn is_reply(&self) -> bool {
        matches!(self.kind, HtmlSelectCursorErrorKind::Reply)
    }
}

impl std::fmt::Display for HtmlSelectCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlSelectCursorErrorKind::MultipleContext => {
                f.write_str("select operation contradicts multiple presence")
            }
            HtmlSelectCursorErrorKind::Reply => {
                f.write_str("select reply does not match its pending action")
            }
        }
    }
}

impl std::error::Error for HtmlSelectCursorError {}

/// Pending work for either the Select selectedIndex or value setter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectSetterAction {
    /// Write false through HtmlOptionFlags::set_selectedness_for_select,
    /// preserving the actual option's dirtiness.
    Deselect {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Compare this option's actual index or value against the supplied value.
    ReadMatch {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Apply HtmlOptionFlags::select_for_select_setter to the remembered match.
    SelectAndDirty {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Run HTML's selectedcontent update for the select, which happens even when no option matched
    /// or there are no options.
    UpdateSelectedcontent,
    /// The setter is done; no further reply is accepted.
    Complete,
}

/// Acknowledged effects or a completed actual index/value comparison.
/// An unavailable comparison is not Matches(false); stop without replying.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectSetterReply {
    /// The adapter performed the pending write.
    Applied,
    /// The completed comparison of one option's index or value with the supplied value. An
    /// unavailable comparison is not `Matches(false)`.
    Matches(bool),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SetterPhase {
    /// Deselect before any match is known; the comparison follows.
    Deselect(usize),
    Match(usize),
    /// Deselect after the first match, which is remembered.
    DeselectMatched {
        ordinal: usize,
        matched: usize,
    },
    Select(usize),
    Refresh,
    Complete,
}

/// One finite, allocation-free Select value/index setter over a complete list.
///
/// HTML's [selected index](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#set-the-selected-index)
/// and [value setter](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-select-value)
/// deselect each option before comparing it, stop comparisons after the first
/// match, finish all deselections, select/dirty that match, then refresh even if
/// there was no match or no option. No normalization or AskForReset follows.
///
/// The adapter keeps the complete option list and its qualified identities
/// coherent, supplies the actual predicate, and admits/applies each effect before
/// replying. This cursor owns no strings, tree, host policy or publication;
/// unavailable list evidence is not an empty list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSelectSetterCursor {
    option_count: usize,
    phase: SetterPhase,
}

impl HtmlSelectSetterCursor {
    /// Starts the setter over a complete option list of `option_count` options. An empty list goes
    /// straight to the selectedcontent update. The count must describe the whole list; an
    /// unavailable list is not an empty one.
    pub const fn new(option_count: usize) -> Self {
        Self {
            option_count,
            phase: if option_count == 0 {
                SetterPhase::Refresh
            } else {
                SetterPhase::Deselect(0)
            },
        }
    }

    /// The action the adapter must perform next, or `Complete` when the setter is done.
    pub fn action(&self) -> HtmlSelectSetterAction {
        match self.phase {
            SetterPhase::Deselect(ordinal) | SetterPhase::DeselectMatched { ordinal, .. } => {
                HtmlSelectSetterAction::Deselect { ordinal }
            }
            SetterPhase::Match(ordinal) => HtmlSelectSetterAction::ReadMatch { ordinal },
            SetterPhase::Select(ordinal) => HtmlSelectSetterAction::SelectAndDirty { ordinal },
            SetterPhase::Refresh => HtmlSelectSetterAction::UpdateSelectedcontent,
            SetterPhase::Complete => HtmlSelectSetterAction::Complete,
        }
    }

    /// A wrong reply, including any reply after Complete, preserves the cursor.
    pub fn reply(&mut self, reply: HtmlSelectSetterReply) -> Result<(), HtmlSelectCursorError> {
        let mut next = *self;
        next.advance(reply)?;
        *self = next;
        Ok(())
    }

    fn after_option(
        &mut self,
        ordinal: usize,
        matched: Option<usize>,
    ) -> Result<(), HtmlSelectCursorError> {
        // Every reachable ordinal is below the complete extent. The last valid
        // ordinal for usize::MAX is MAX-1; its successor MAX is representable.
        let next = ordinal
            .checked_add(1)
            .ok_or(HtmlSelectCursorErrorKind::Reply)?;
        self.phase = match (next < self.option_count, matched) {
            (true, None) => SetterPhase::Deselect(next),
            (true, Some(matched)) => SetterPhase::DeselectMatched {
                ordinal: next,
                matched,
            },
            (false, Some(matched)) => SetterPhase::Select(matched),
            (false, None) => SetterPhase::Refresh,
        };
        Ok(())
    }

    fn advance(&mut self, reply: HtmlSelectSetterReply) -> Result<(), HtmlSelectCursorError> {
        match (self.phase, reply) {
            (SetterPhase::Deselect(ordinal), HtmlSelectSetterReply::Applied) => {
                self.phase = SetterPhase::Match(ordinal);
            }
            (SetterPhase::DeselectMatched { ordinal, matched }, HtmlSelectSetterReply::Applied) => {
                self.after_option(ordinal, Some(matched))?
            }
            (SetterPhase::Match(ordinal), HtmlSelectSetterReply::Matches(matches)) => {
                self.after_option(ordinal, matches.then_some(ordinal))?;
            }
            (SetterPhase::Select(_), HtmlSelectSetterReply::Applied) => {
                self.phase = SetterPhase::Refresh;
            }
            (SetterPhase::Refresh, HtmlSelectSetterReply::Applied) => {
                self.phase = SetterPhase::Complete;
            }
            _ => return Err(HtmlSelectCursorErrorKind::Reply.error()),
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    ResetSelect,
    ResetOption(usize),
    FirstRead(usize),
    FirstWrite(usize),
    Read(usize),
    Deselect {
        current: usize,
        previous: usize,
    },
    Disabled(usize),
    /// Carries the first enabled option, the fallback candidate.
    Size(usize),
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
    /// Starts `operation` for a select that has (`multiple`) or lacks the `multiple` attribute and
    /// whose complete option list has `option_count` options. It refuses an operation that
    /// contradicts `multiple`: `MultipleAdded` without it and `MultipleRemoved` with it
    /// (`HtmlSelectCursorError::is_multiple_context`).
    pub fn new(
        operation: HtmlSelectOperation,
        multiple: bool,
        option_count: usize,
    ) -> Result<Self, HtmlSelectCursorError> {
        if matches!(operation, HtmlSelectOperation::MultipleAdded) && !multiple
            || matches!(operation, HtmlSelectOperation::MultipleRemoved) && multiple
        {
            return Err(HtmlSelectCursorErrorKind::MultipleContext.error());
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
    /// The action the adapter must perform next, or `Complete` carrying the normalization result
    /// once the operation is done.
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
            Phase::Size(_) => HtmlSelectAction::ReadDisplaySize,
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
        } else if let (None, Some(fallback)) = (self.last_selected, self.first_enabled) {
            self.phase = Phase::Size(fallback);
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
            (Phase::Size(fallback), HtmlSelectReply::DisplaySizeIsOne(one)) => {
                if one {
                    self.phase = Phase::Select(fallback);
                } else {
                    self.finish_normalization();
                }
            }
            (Phase::Select(_), HtmlSelectReply::Applied) => {
                self.changed = true;
                self.finish_normalization();
            }
            (Phase::Refresh, HtmlSelectReply::Applied) => self.phase = Phase::Complete,
            _ => return Err(HtmlSelectCursorErrorKind::Reply.error()),
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
    /// The node is an option: add it to the list of options and do not descend into it.
    IncludeAndPrune,
    /// The node contributes no options and its descendants are skipped: a `select`, `hr`,
    /// `datalist`, or an `optgroup` nested inside another `optgroup`.
    Prune,
    /// The node contributes no option itself, but its descendants are visited in tree order.
    Descend,
}
/// Classifies one descendant of a select for the list of options (HTML877d
/// #concept-select-option-list). `kind` is the node's known element kind, or `None` for any other
/// node, and `inside_optgroup` tells whether an `optgroup` ancestor lies between the node and the
/// select. The adapter performs the traversal.
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

/// A `button` element: the first stage of HTML877d #the-button-element's rule "If a button
/// element is the first child which is an element of a select element, then it is inert"
/// (forms capture line 7399). Constructed only from a button's known kind.
///
/// The rule is staged so an adapter reads each operand only after the earlier ones hold:
/// [`HtmlSelectButton::new`], then [`HtmlSelectButton::child_of`] with the ordinary-tree parent's
/// kind, then [`HtmlSelectChildButton::is_inert`] with the first-element-child fact. "Child" is the
/// DOM ordinary-tree child, never a flat-tree child, so a slot or shadow edge never supplies the
/// parent. The rule marks the button itself and names no descendants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSelectButton(());
impl HtmlSelectButton {
    /// The button stage for a node whose known element kind is `kind` (`None` for any other node
    /// or element), or `None` when it is not a `button`.
    pub const fn new(kind: Option<KnownHtmlElement>) -> Option<Self> {
        match kind {
            Some(KnownHtmlElement::Button) => Some(Self(())),
            _ => None,
        }
    }
    /// The select-child stage when the button's ordinary-tree parent has known element kind
    /// `parent` (`None` for any other node or element), or `None` when it is not a `select`.
    pub const fn child_of(self, parent: Option<KnownHtmlElement>) -> Option<HtmlSelectChildButton> {
        match parent {
            Some(KnownHtmlElement::Select) => Some(HtmlSelectChildButton(())),
            _ => None,
        }
    }
}

/// A `button` that is a child of a `select`; whether it is the first element child decides the
/// rule (see [`HtmlSelectButton`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlSelectChildButton(());
impl HtmlSelectChildButton {
    /// Whether the button is inert: true exactly when it is the first element among its parent's
    /// children. Text and comment siblings are not elements and do not precede it.
    pub const fn is_inert(self, is_first_element_child: bool) -> bool {
        is_first_element_child
    }
}

/// Action for one established option ancestor, after own disabled presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlOptionDisabledAncestor {
    /// The ancestor is neither an option's disabling ancestor nor a stop; examine the next
    /// ancestor.
    Continue,
    /// The ancestor stops the search with no disabling `optgroup`: an `option`, `select`,
    /// `datalist` or `hr`. The option is not disabled by an ancestor.
    Enabled,
    /// The ancestor is an `optgroup`. The adapter reads its `disabled` attribute, which disables
    /// the option when present (HTML877d #concept-option-disabled).
    ReadOptgroupDisabled,
}
/// Classifies one ancestor of an option, nearest first, for the option's disabledness (HTML877d
/// #concept-option-disabled). `None` stands for an established other kind of element. Only the
/// option's own `disabled` attribute is outside this rule.
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
    /// The ancestor does not decide the search; examine the next one.
    Continue,
    /// The ancestor is the option's nearest ancestor select.
    NearestSelect,
    /// The option has no ancestor select: a `datalist`, `hr` or `option` ancestor intervenes, or a
    /// second `optgroup` is reached.
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
    /// Examines the next ancestor, nearest first, and returns the decision. Once a decision other
    /// than `Continue` is made, every later call returns it again. `None` stands for an established
    /// other kind of element.
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
    /// The ancestor does not decide the search; examine the next one.
    Continue,
    /// The ancestor is the nearest ancestor select; the adapter keeps its identity and continues.
    RememberNearestSelect,
    /// The search stops and the selectedcontent element is disabled: an `option` or another
    /// `selectedcontent` is an ancestor, or a second `select` is reached
    /// (HTML877d #selectedcontent-disabled).
    StopDisabled,
}
/// Fixed intrinsic state; the adapter retains the actual nearest select ID.
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlSelectedcontentAncestors {
    nearest_select: bool,
    disabled: bool,
}
impl HtmlSelectedcontentAncestors {
    /// Examines the next ancestor, nearest first, and returns the decision. After a `StopDisabled`,
    /// every call returns `StopDisabled`. `None` stands for an established other kind of element.
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
    /// Whether the search stopped with the selectedcontent element disabled.
    pub fn disabled(&self) -> bool {
        self.disabled
    }
    /// Whether a nearest ancestor select has been observed.
    pub fn has_nearest_select(&self) -> bool {
        self.nearest_select
    }
}

/// Per-target selectedcontent effect after a complete first-selected scan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectedcontentUpdate {
    /// The target is untouched: the select has `multiple` or the selectedcontent element is
    /// disabled.
    Skip,
    /// Empty the target, because the select has no selected option.
    Clear,
    /// Replace the target's content with clones of the children of one option.
    CloneOptionChildren {
        /// Zero-based position of the option in the complete option list, in tree order.
        ordinal: usize,
    },
    /// Selection/order is incomplete; no clear or clone is authorized.
    Unavailable,
}
/// A first-selected scan distinguishes proven absence from missing evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSelectFirstSelection {
    /// The scan completed and no option is selected.
    NoSelection,
    /// The first option in tree order whose selectedness is true.
    Selected {
        /// Zero-based position of the first selected option in the complete option list.
        ordinal: usize,
    },
    /// The selection evidence is incomplete, so no clear or clone is authorized. It is distinct
    /// from `NoSelection`.
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
    /// Read the selectedness of one option and answer through
    /// `HtmlSelectFirstSelectedCursor::reply`.
    ReadSelectedness {
        /// Zero-based position of the option to read in the complete option list.
        ordinal: usize,
    },
    /// The scan is done with the given result.
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
    /// Starts a scan over a complete option list of `option_count` options. An empty list completes
    /// at once with `NoSelection`.
    pub fn new(option_count: usize) -> Self {
        Self {
            count: option_count,
            ordinal: 0,
            result: (option_count == 0).then_some(HtmlSelectFirstSelection::NoSelection),
        }
    }
    /// The read the adapter must perform next, or the established result.
    pub fn action(&self) -> HtmlSelectFirstSelectedAction {
        self.result.map_or(
            HtmlSelectFirstSelectedAction::ReadSelectedness {
                ordinal: self.ordinal,
            },
            HtmlSelectFirstSelectedAction::Complete,
        )
    }
    /// Answers the pending read: `Some(true)` for a selected option, `Some(false)` for an
    /// unselected one, and `None` when selectedness is unavailable, which completes the scan as
    /// `Unavailable`. It fails with `HtmlSelectCursorError::is_reply` after the scan is complete.
    pub fn reply(&mut self, selected: Option<bool>) -> Result<(), HtmlSelectCursorError> {
        if self.result.is_some() {
            return Err(HtmlSelectCursorErrorKind::Reply.error());
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

#[cfg(test)]
mod setter_tests {
    use super::{
        HtmlSelectSetterAction as Action, HtmlSelectSetterCursor as Cursor,
        HtmlSelectSetterReply as Reply, SetterPhase,
    };

    #[test]
    fn maximum_extent_last_comparison_finishes_without_overflow() {
        // These are valid owner states after an arbitrarily long negative
        // prefix. Seed the private last transition, not a public skip API or
        // an impossible MAX-iteration test.
        for matches in [false, true] {
            let mut cursor = Cursor {
                option_count: usize::MAX,
                phase: SetterPhase::Match(usize::MAX - 1),
            };
            assert_eq!(
                cursor.action(),
                Action::ReadMatch {
                    ordinal: usize::MAX - 1
                }
            );
            cursor.reply(Reply::Matches(matches)).unwrap();
            if matches {
                assert_eq!(
                    cursor.action(),
                    Action::SelectAndDirty {
                        ordinal: usize::MAX - 1
                    }
                );
                cursor.reply(Reply::Applied).unwrap();
            }
            assert_eq!(cursor.action(), Action::UpdateSelectedcontent);
            cursor.reply(Reply::Applied).unwrap();
            assert_eq!(cursor.action(), Action::Complete);
        }
    }

    #[test]
    fn maximum_extent_last_deselection_retains_an_earlier_match() {
        let mut cursor = Cursor {
            option_count: usize::MAX,
            phase: SetterPhase::DeselectMatched {
                ordinal: usize::MAX - 1,
                matched: 0,
            },
        };
        let before = cursor;
        assert!(cursor.reply(Reply::Matches(true)).is_err());
        assert_eq!(cursor, before);
        cursor.reply(Reply::Applied).unwrap();
        assert_eq!(cursor.action(), Action::SelectAndDirty { ordinal: 0 });
        cursor.reply(Reply::Applied).unwrap();
        assert_eq!(cursor.action(), Action::UpdateSelectedcontent);
        cursor.reply(Reply::Applied).unwrap();
        assert_eq!(cursor.action(), Action::Complete);
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    #[test]
    fn html_select_cursor_error_causes_have_their_message_and_exactly_one_predicate() {
        type Predicate = fn(&HtmlSelectCursorError) -> bool;
        let cases: Vec<(HtmlSelectCursorError, &str, Predicate)> = vec![
            (
                HtmlSelectCursorErrorKind::MultipleContext.error(),
                "select operation contradicts multiple presence",
                HtmlSelectCursorError::is_multiple_context,
            ),
            (
                HtmlSelectCursorErrorKind::Reply.error(),
                "select reply does not match its pending action",
                HtmlSelectCursorError::is_reply,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}

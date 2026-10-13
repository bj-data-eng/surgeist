//! Shared intrinsic input rules from HTML877d #the-input-element.
//!
//! These rules own no strings, tree, controller or live identity. Adapters apply
//! their requirements in order to their actual staged state. A returned rule
//! result does not certify a completed DOM transaction or parser construction.
//!
//! For each applicable relevant-value change, the actual value-write adapter
//! must apply HtmlInputSelection::clamp with the actual new length, current
//! selection/cursor and established empty-selection capability before continuing
//! family effects. Reset/type cursors alone do not complete a live operation.

use crate::{DocumentString, HtmlInputType};

impl HtmlInputType {
    pub fn value_mode(self) -> HtmlInputValueMode {
        match self {
            Self::Hidden | Self::Submit | Self::Reset | Self::Button | Self::Image => {
                HtmlInputValueMode::Default
            }
            Self::File => HtmlInputValueMode::Filename,
            Self::Checkbox | Self::Radio => HtmlInputValueMode::DefaultOn,
            _ => HtmlInputValueMode::Value,
        }
    }
    /// Selection support is distinct from Value mode: Email is not selectable.
    pub fn is_selectable(self) -> bool {
        matches!(
            self,
            Self::Text | Self::Search | Self::Telephone | Self::Password | Self::Url
        )
    }
    fn temporal_format(self) -> Option<HtmlInputTemporalFormat> {
        match self {
            Self::Date => Some(HtmlInputTemporalFormat::Date),
            Self::Month => Some(HtmlInputTemporalFormat::Month),
            Self::Week => Some(HtmlInputTemporalFormat::Week),
            Self::Time => Some(HtmlInputTemporalFormat::Time),
            Self::LocalDateTime => Some(HtmlInputTemporalFormat::LocalDateTime),
            _ => None,
        }
    }
    pub fn sanitization(self) -> HtmlInputValueSanitization {
        if self == Self::Range {
            return HtmlInputValueSanitization::Range;
        }
        if self == Self::Color {
            return HtmlInputValueSanitization::Color;
        }
        let string = if self == Self::Number {
            HtmlInputSanitization::Number
        } else if let Some(format) = self.temporal_format() {
            HtmlInputSanitization::Temporal(format)
        } else if self == Self::Email {
            HtmlInputSanitization::Email
        } else if self == Self::Url {
            HtmlInputSanitization::StripCrLfAndTrim
        } else if self.value_mode() != HtmlInputValueMode::Value {
            HtmlInputSanitization::Preserve
        } else {
            HtmlInputSanitization::StripCrLf
        };
        HtmlInputValueSanitization::String(string)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueMode {
    Value,
    Default,
    DefaultOn,
    Filename,
}
impl HtmlInputValueMode {
    /// Selects the native getter's actual storage; specified empty content wins.
    /// Filename requires an established selected-resource fact. This borrowed
    /// rule owns no resource acquisition or identity qualification.
    pub fn readback<'a>(
        self,
        internal: &'a [u16],
        content: Option<&'a [u16]>,
        files: Option<HtmlInputFileSelection>,
    ) -> Option<&'a [u16]> {
        match self {
            Self::Value => Some(internal),
            Self::Default => Some(content.unwrap_or(&[])),
            Self::DefaultOn => Some(content.unwrap_or(&[0x6f, 0x6e])),
            Self::Filename => files.map(|HtmlInputFileSelection::Empty| &[] as &[u16]),
        }
    }
}
/// An established empty selection, distinct from value content or a synthetic path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputFileSelection {
    Empty,
}
/// Filename setter refusal for a nonempty proposed value. No files are created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputFilenameInvalidState;
pub fn input_filename_setter(
    value: &[u16],
) -> Result<HtmlInputFileSelection, HtmlInputFilenameInvalidState> {
    if value.is_empty() {
        Ok(HtmlInputFileSelection::Empty)
    } else {
        Err(HtmlInputFilenameInvalidState)
    }
}
/// Source-selected coordinate for fresh immutable Image construction only.
/// Activation and arbitrary host coordinates are outside this constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputImageCoordinate {
    x: u32,
    y: u32,
}
impl HtmlInputImageCoordinate {
    pub(crate) fn initial() -> Self {
        Self { x: 0, y: 0 }
    }
    pub fn x(self) -> u32 {
        self.x
    }
    pub fn y(self) -> u32 {
        self.y
    }
}
/// Complete sanitization requirement. Range needs actual min/max/step/value
/// content and an exact numeric producer; the borrowed string cursor cannot
/// supply those facts. Color requires the actual dirty/default source,
/// alpha/colorspace attributes and the CSS used-color operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueSanitization {
    String(HtmlInputSanitization),
    Range,
    Color,
}
/// Ordered intrinsic Color serialization steps. CSS owns each conversion,
/// numerical operation and spelling; this cursor owns only HTML's sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputColorEffect {
    ForceOpaqueAlpha,
    Convert(crate::HtmlInputColorSpace),
    QuantizeSrgb,
    ColorFunction,
}
pub struct HtmlInputColorSerialization {
    alpha: bool,
    space: crate::HtmlInputColorSpace,
    step: u8,
}
impl HtmlInputColorSerialization {
    pub fn new(alpha: bool, space: crate::HtmlInputColorSpace) -> Self {
        Self {
            alpha,
            space,
            step: 0,
        }
    }
    /// The caller admits one fixed rule step before this operation. No color
    /// channel, string, tree or environmental fact is inspected here.
    pub fn next_effect(&mut self) -> Option<HtmlInputColorEffect> {
        if self.step == 0 && self.alpha {
            self.step = 1;
        }
        let step = self.step;
        self.step = self.step.saturating_add(1);
        match step {
            0 => Some(HtmlInputColorEffect::ForceOpaqueAlpha),
            1 => Some(HtmlInputColorEffect::Convert(self.space)),
            2 if self.space == crate::HtmlInputColorSpace::LimitedSrgb => {
                Some(HtmlInputColorEffect::QuantizeSrgb)
            }
            3 if self.space == crate::HtmlInputColorSpace::LimitedSrgb && self.alpha => {
                Some(HtmlInputColorEffect::ColorFunction)
            }
            _ => None,
        }
    }
    pub fn html_compatible(&self) -> bool {
        !self.alpha && self.space == crate::HtmlInputColorSpace::LimitedSrgb
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputSanitization {
    Preserve,
    StripCrLf,
    StripCrLfAndTrim,
    /// The complete operation requires actual multiple-attribute presence.
    Email,
    /// Strict source-defined temporal grammar; invalid becomes empty.
    Temporal(HtmlInputTemporalFormat),
    Number,
}
impl HtmlInputSanitization {
    /// Unit-local filtering only. Boundary trimming, Email comma processing
    /// and strict temporal validation/normalization require
    /// HtmlInputSanitizationCursor; this predicate cannot certify them.
    pub fn retain_unit(self, unit: u16) -> bool {
        matches!(
            self,
            Self::Preserve | Self::Email | Self::Temporal(_) | Self::Number
        ) || !matches!(unit, 0x0d | 0x0a)
    }
}

/// Actual source-selected strict temporal input grammar. This fixed rule type
/// does not allocate or supply current-state, calendar UI or stepping behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputTemporalFormat {
    Date,
    Month,
    Week,
    Time,
    LocalDateTime,
}
impl HtmlInputTemporalFormat {
    fn owner(self) -> crate::temporal::TemporalInputFormat {
        match self {
            Self::Date => crate::temporal::TemporalInputFormat::Date,
            Self::Month => crate::temporal::TemporalInputFormat::Month,
            Self::Week => crate::temporal::TemporalInputFormat::Week,
            Self::Time => crate::temporal::TemporalInputFormat::Time,
            Self::LocalDateTime => crate::temporal::TemporalInputFormat::LocalDateTime,
        }
    }
}

/// Actual Email sanitizer mode, selected by specified multiple presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputEmailMode {
    Single,
    Multiple,
}
/// A borrowed surviving source interval. Separators retain actual comma units;
/// no output string or synthetic source position is created by the rule owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputRetainedRange {
    start: usize,
    end: usize,
}
impl HtmlInputRetainedRange {
    pub fn start(self) -> usize {
        self.start
    }
    pub fn end(self) -> usize {
        self.end
    }
}
enum SanitizerStep {
    Temporal,
    Number,
    Leading,
    Trailing,
    Emit,
    Commas,
    Pending,
    Done,
}
enum InputValidationCursor<'a> {
    None,
    Temporal(crate::temporal::TemporalValidationCursor<'a>),
    Number(crate::numeric::FloatingPointGrammarCursor<'a>),
}
/// Complete borrowed sanitizer boundary decisions. Each advance performs at
/// most one admitted source-unit inspection or fixed state action. Adapters
/// apply retain_unit to non-Email-multiple intervals, copy surviving origins,
/// and own allocation/cancellation. This cursor owns none of those capabilities.
pub struct HtmlInputSanitizationCursor<'a> {
    units: &'a [u16],
    policy: HtmlInputSanitization,
    mode: HtmlInputEmailMode,
    start: usize,
    end: usize,
    step: SanitizerStep,
    commas: crate::tokens::CommaTokenCursor<'a>,
    pending: Option<HtmlInputRetainedRange>,
    count: usize,
    validation: InputValidationCursor<'a>,
    temporal_plan: Option<crate::temporal::TemporalInputPlan>,
}
impl<'a> HtmlInputSanitizationCursor<'a> {
    pub fn new(policy: HtmlInputSanitization, units: &'a [u16], mode: HtmlInputEmailMode) -> Self {
        let validation = match policy {
            HtmlInputSanitization::Temporal(format) => InputValidationCursor::Temporal(
                crate::temporal::TemporalValidationCursor::new(units, format.owner(), true),
            ),
            HtmlInputSanitization::Number => {
                InputValidationCursor::Number(crate::numeric::FloatingPointGrammarCursor::new(
                    units,
                    crate::numeric::FloatingPointParseMode::Author,
                ))
            }
            _ => InputValidationCursor::None,
        };
        let step = match policy {
            HtmlInputSanitization::Temporal(_) => SanitizerStep::Temporal,
            HtmlInputSanitization::Number => SanitizerStep::Number,
            HtmlInputSanitization::Email if mode == HtmlInputEmailMode::Multiple => {
                SanitizerStep::Commas
            }
            HtmlInputSanitization::Email | HtmlInputSanitization::StripCrLfAndTrim => {
                SanitizerStep::Leading
            }
            _ => SanitizerStep::Emit,
        };
        Self {
            units,
            policy,
            mode,
            start: 0,
            end: units.len(),
            step,
            commas: crate::tokens::CommaTokenCursor::new(units),
            pending: None,
            count: 0,
            validation,
            temporal_plan: None,
        }
    }
    /// A source-qualified successful local normalization replaces only its
    /// SPACE separator with T. The adapter retains the whole original origin.
    pub fn replacement_unit(&self, position: usize) -> Option<u16> {
        self.temporal_plan
            .and_then(|plan| plan.replacement)
            .filter(|&separator| separator == position)
            .map(|_| 0x54)
    }
    pub fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match self.step {
            SanitizerStep::Done => None,
            SanitizerStep::Number | SanitizerStep::Temporal => Some(
                if match &self.validation {
                    InputValidationCursor::Number(cursor) => cursor.inspects_unit(),
                    InputValidationCursor::Temporal(cursor) => cursor.inspects_unit(),
                    InputValidationCursor::None => unreachable!("validation step"),
                } {
                    HtmlInputRuleWork::InspectValueUnit
                } else {
                    HtmlInputRuleWork::ReadField
                },
            ),
            SanitizerStep::Leading | SanitizerStep::Trailing if self.start < self.end => {
                Some(HtmlInputRuleWork::InspectValueUnit)
            }
            SanitizerStep::Commas if self.commas.inspection().is_some() => {
                Some(HtmlInputRuleWork::InspectValueUnit)
            }
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    pub fn advance(&mut self) -> Option<HtmlInputRetainedRange> {
        match self.step {
            SanitizerStep::Number => {
                let InputValidationCursor::Number(cursor) = &mut self.validation else {
                    unreachable!("Number grammar state")
                };
                cursor.advance();
                if cursor.is_finished() {
                    self.end = if cursor.is_valid() {
                        self.units.len()
                    } else {
                        0
                    };
                    self.step = SanitizerStep::Emit;
                }
            }
            SanitizerStep::Temporal => {
                let InputValidationCursor::Temporal(scanner) = &mut self.validation else {
                    unreachable!("temporal scanner")
                };
                scanner.advance();
                if scanner.finished() {
                    self.temporal_plan = scanner.normalization();
                    self.end = self.temporal_plan.map_or(0, |plan| plan.retained_end);
                    self.step = SanitizerStep::Emit;
                }
            }

            SanitizerStep::Leading => {
                if self.start < self.end && crate::tokens::ascii_whitespace(self.units[self.start])
                {
                    self.start += 1;
                } else {
                    self.step = SanitizerStep::Trailing;
                }
            }
            SanitizerStep::Trailing => {
                if self.start < self.end
                    && crate::tokens::ascii_whitespace(self.units[self.end - 1])
                {
                    self.end -= 1;
                } else {
                    self.step = SanitizerStep::Emit;
                }
            }
            SanitizerStep::Emit => {
                self.step = SanitizerStep::Done;
                return Some(HtmlInputRetainedRange {
                    start: self.start,
                    end: self.end,
                });
            }
            SanitizerStep::Commas => {
                if let Some(range) = self.commas.advance() {
                    self.count += 1;
                    let token = HtmlInputRetainedRange {
                        start: range.start,
                        end: range.end,
                    };
                    if let Some(separator) = range.separator {
                        self.pending = Some(token);
                        self.step = SanitizerStep::Pending;
                        return Some(HtmlInputRetainedRange {
                            start: separator,
                            end: separator + 1,
                        });
                    }
                    return Some(token);
                }
                if self.commas.finished() {
                    self.step = SanitizerStep::Done;
                }
            }
            SanitizerStep::Pending => {
                self.step = SanitizerStep::Commas;
                return self.pending.take();
            }
            SanitizerStep::Done => {}
        }
        None
    }
    /// Final actual produced Email values count, not lenient resplitting of a
    /// joined current string. Single/non-Email operations have no such list.
    pub fn email_value_count(&self) -> Option<usize> {
        if matches!(self.step, SanitizerStep::Done)
            && self.policy == HtmlInputSanitization::Email
            && self.mode == HtmlInputEmailMode::Multiple
        {
            Some(self.count)
        } else {
            None
        }
    }
    pub fn retain_unit(&self, unit: u16) -> bool {
        if self.policy == HtmlInputSanitization::Email && self.mode == HtmlInputEmailMode::Single {
            !matches!(unit, 0x0d | 0x0a)
        } else {
            self.policy.retain_unit(unit)
        }
    }
}

/// Canonical semantic flags, shared by actual parser and future DOM storage.
/// Strings, selection, form identity and resource availability are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputFlags {
    kind: HtmlInputType,
    dirty_value: bool,
    dirty_checkedness: bool,
    checkedness: bool,
    indeterminate: bool,
    user_validity: bool,
}
impl HtmlInputFlags {
    /// Blank creation starts in missing-type Text before ordered attributes.
    pub fn fresh() -> Self {
        Self::from_retained(HtmlInputType::Text, false, false, false, false, false)
    }
    /// Preserve actual retained facts without rerunning creation/reset.
    /// This does not qualify a source, identity, or availability of other fields.
    pub fn from_retained(
        kind: HtmlInputType,
        dirty_value: bool,
        dirty_checkedness: bool,
        checkedness: bool,
        indeterminate: bool,
        user_validity: bool,
    ) -> Self {
        Self {
            kind,
            dirty_value,
            dirty_checkedness,
            checkedness,
            indeterminate,
            user_validity,
        }
    }
    pub fn input_type(self) -> HtmlInputType {
        self.kind
    }
    pub fn dirty_value(self) -> bool {
        self.dirty_value
    }
    pub fn dirty_checkedness(self) -> bool {
        self.dirty_checkedness
    }
    pub fn checkedness(self) -> bool {
        self.checkedness
    }
    pub fn indeterminate(self) -> bool {
        self.indeterminate
    }
    pub fn user_validity(self) -> bool {
        self.user_validity
    }
    pub fn value_attribute_changed(self) -> HtmlInputValueAttributeEffect {
        if self.dirty_value {
            HtmlInputValueAttributeEffect::KeepCurrent
        } else {
            HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
                self.kind.sanitization(),
            )
        }
    }
    pub fn checked_attribute_changed(
        mut self,
        change: HtmlInputCheckedAttributeChange,
    ) -> Option<Self> {
        if self.dirty_checkedness {
            return None;
        }
        self.checkedness = change == HtmlInputCheckedAttributeChange::Added;
        Some(self)
    }
    /// HTML checked setter: current checkedness changes and becomes dirty.
    /// Radio adapters must subsequently apply Checkedness normalization over
    /// the actual updated group; this method owns no tree or peer state.
    pub fn checked_setter(mut self, checked: bool) -> Self {
        self.checkedness = checked;
        self.dirty_checkedness = true;
        self
    }
    /// Whether the source trigger requires peer normalization in updated state.
    /// The adapter supplies actual qualified tree, owner and nonempty-name peers.
    pub fn radio_normalization_required(self, trigger: HtmlRadioTrigger) -> bool {
        match trigger {
            HtmlRadioTrigger::Checkedness
            | HtmlRadioTrigger::Name
            | HtmlRadioTrigger::FormOwner
            | HtmlRadioTrigger::Type
            | HtmlRadioTrigger::Connected => self.kind == HtmlInputType::Radio && self.checkedness,
        }
    }
    /// Source peer normalization changes only checkedness, never dirty state,
    /// default checked presence, indeterminateness or user validity.
    pub fn radio_peer_unchecked(mut self) -> Self {
        self.checkedness = false;
        self
    }
    /// HTML indeterminate setter changes only the actual indeterminateness.
    pub fn indeterminate_setter(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
    /// Source reset's first flag step, before default/value/checked/file effects.
    pub fn reset_begin(mut self) -> Self {
        self.dirty_value = false;
        self.dirty_checkedness = false;
        self.user_validity = false;
        self
    }
    pub fn reset_checkedness(mut self, actual_checked_presence: bool) -> Self {
        self.checkedness = actual_checked_presence;
        self
    }
    /// Actual state transition only. Attribute spelling changes within one state
    /// do not invoke HTML's type-change algorithm.
    pub fn type_changed(mut self, next: HtmlInputType) -> Self {
        self.kind = next;
        self
    }
    /// The dirty-value action after applying SetCurrentFromDefaultAndClearDirty.
    /// Other flags are retained; this is not per-control reset.
    pub fn type_default_installed(mut self) -> Self {
        self.dirty_value = false;
        self
    }
    /// Native Value-mode requirements: install supplied current value first,
    /// then apply the returned dirty flags and sanitize the actual current value.
    /// Default and DefaultOn require the content-attribute path instead.
    /// This is a requirement for an adapter, not event execution.
    pub fn value_setter_begin(self) -> HtmlInputValueSetterStart {
        if self.kind.value_mode() == HtmlInputValueMode::Filename {
            HtmlInputValueSetterStart::SetFilename
        } else if self.kind.value_mode() != HtmlInputValueMode::Value {
            HtmlInputValueSetterStart::SetContentValue
        } else {
            let mut next = self;
            next.dirty_value = true;
            HtmlInputValueSetterStart::SetCurrentValue(next)
        }
    }
    /// Only the flags specified by input cloning overlay fresh/copied attrs.
    pub fn clone_flags_from(mut self, original: Self) -> Self {
        self.dirty_value = original.dirty_value;
        self.dirty_checkedness = original.dirty_checkedness;
        self.checkedness = original.checkedness;
        self.indeterminate = original.indeterminate;
        self
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueAttributeEffect {
    KeepCurrent,
    ReplaceCurrentFromInstalledDefault(HtmlInputValueSanitization),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputCheckedAttributeChange {
    Added,
    Removed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueSetterStart {
    /// Apply input_filename_setter to the proposed value before clearing files.
    SetFilename,
    SetContentValue,
    SetCurrentValue(HtmlInputFlags),
}

/// Source reset order. This is per-control reset, not a form reset event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputResetEffect {
    ClearDirtyAndUserValidity(HtmlInputFlags),
    ReplaceCurrentFromDefault,
    SetCheckedFromPresence,
    EmptySelectedFiles,
    Sanitize(HtmlInputValueSanitization),
}
pub struct HtmlInputReset {
    initial: HtmlInputFlags,
    stage: u8,
}
impl HtmlInputReset {
    pub fn new(initial: HtmlInputFlags) -> Self {
        Self { initial, stage: 0 }
    }
    /// Apply each action to updated actual storage before the next action.
    pub fn next_effect(&mut self) -> Option<HtmlInputResetEffect> {
        let result = match self.stage {
            0 => HtmlInputResetEffect::ClearDirtyAndUserValidity(self.initial.reset_begin()),
            1 => HtmlInputResetEffect::ReplaceCurrentFromDefault,
            2 => HtmlInputResetEffect::SetCheckedFromPresence,
            3 => HtmlInputResetEffect::EmptySelectedFiles,
            4 => HtmlInputResetEffect::Sanitize(self.initial.kind.sanitization()),
            _ => return None,
        };
        self.stage += 1;
        Some(result)
    }
}

/// Source-ordered type-change actions. Apply each action (including nested
/// attribute effects) before requesting the next one; no batch over stale state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputTypeEffect {
    SetContentValueFromCurrent,
    SetCurrentFromDefaultAndClearDirty,
    SetCurrentEmptyForFilename,
    SetRendering,
    SignalTypeChange,
    Sanitize(HtmlInputValueSanitization),
    SetInitialCursorAndNoneDirection,
}
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputTypeChange {
    previous: HtmlInputType,
    next: HtmlInputType,
    nonempty: bool,
    stage: u8,
}
impl HtmlInputTypeChange {
    pub fn new(previous: HtmlInputType, next: HtmlInputType, current_value_len: usize) -> Self {
        Self {
            previous,
            next,
            nonempty: current_value_len != 0,
            stage: if previous == next { 5 } else { 0 },
        }
    }
    pub fn next_effect(&mut self) -> Option<HtmlInputTypeEffect> {
        while self.stage < 5 {
            let stage = self.stage;
            self.stage += 1;
            match stage {
                0 if self.previous.value_mode() == HtmlInputValueMode::Value
                    && matches!(
                        self.next.value_mode(),
                        HtmlInputValueMode::Default | HtmlInputValueMode::DefaultOn
                    )
                    && self.nonempty =>
                {
                    return Some(HtmlInputTypeEffect::SetContentValueFromCurrent);
                }
                0 if self.previous.value_mode() != HtmlInputValueMode::Value
                    && self.next.value_mode() == HtmlInputValueMode::Value =>
                {
                    return Some(HtmlInputTypeEffect::SetCurrentFromDefaultAndClearDirty);
                }
                0 if self.previous.value_mode() != HtmlInputValueMode::Filename
                    && self.next.value_mode() == HtmlInputValueMode::Filename =>
                {
                    return Some(HtmlInputTypeEffect::SetCurrentEmptyForFilename);
                }
                1 => return Some(HtmlInputTypeEffect::SetRendering),
                2 => return Some(HtmlInputTypeEffect::SignalTypeChange),
                3 => return Some(HtmlInputTypeEffect::Sanitize(self.next.sanitization())),
                4 if !self.previous.is_selectable() && self.next.is_selectable() => {
                    return Some(HtmlInputTypeEffect::SetInitialCursorAndNoneDirection);
                }
                _ => {}
            }
        }
        None
    }
}

/// Admission before one fixed cursor action. Adapters map these credits to
/// their own ledger; these cursors neither poll nor hold a controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputRuleWork {
    /// One keyword-row or length metadata decision.
    ReadField,
    /// The actual old/current length metadata pair.
    ReadValueLengths,
    /// One UTF16/ASCII byte pair plus ASCII comparison.
    CompareAsciiUnit,
    /// One pair of actual UTF16 values.
    CompareValueUnits,
    /// One source UTF-16 unit used by a sanitizer boundary decision.
    InspectValueUnit,
}

enum TypeStep {
    Row,
    Length(&'static str, HtmlInputType),
    Unit(&'static str, HtmlInputType, usize),
    Done(HtmlInputType),
}
/// Borrowed recognizer over the authoritative attributes-owned 22-state rows.
/// No trimming, allocation, copied keyword table or whole-string advance.
pub struct HtmlInputTypeCursor<'a> {
    value: Option<&'a DocumentString>,
    row: usize,
    step: TypeStep,
}
impl<'a> HtmlInputTypeCursor<'a> {
    pub fn new(value: Option<&'a DocumentString>) -> Self {
        Self {
            value,
            row: 0,
            step: TypeStep::Row,
        }
    }
    pub fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match self.step {
            TypeStep::Done(_) => None,
            TypeStep::Unit(..) => Some(HtmlInputRuleWork::CompareAsciiUnit),
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    /// At most one row lookup, length decision or UTF-16/ASCII comparison.
    /// The caller admits required_work before calling this method.
    pub fn advance(&mut self) {
        self.step = match self.step {
            TypeStep::Row => {
                if self.value.is_none() {
                    TypeStep::Done(crate::attributes::input_type_default())
                } else if let Some((keyword, state)) =
                    crate::attributes::input_type_keyword(self.row)
                {
                    self.row += 1;
                    TypeStep::Length(keyword, state)
                } else {
                    TypeStep::Done(crate::attributes::input_type_invalid_default())
                }
            }
            TypeStep::Length(keyword, state) => {
                if self.value.expect("present row").len() != keyword.len() {
                    TypeStep::Row
                } else if keyword.is_empty() {
                    TypeStep::Done(state)
                } else {
                    TypeStep::Unit(keyword, state, 0)
                }
            }
            TypeStep::Unit(keyword, state, position) => {
                if !crate::attributes::ascii_keyword_unit(
                    self.value.expect("present row").code_units()[position],
                    keyword.as_bytes()[position],
                ) {
                    TypeStep::Row
                } else if position + 1 == keyword.len() {
                    TypeStep::Done(state)
                } else {
                    TypeStep::Unit(keyword, state, position + 1)
                }
            }
            TypeStep::Done(state) => TypeStep::Done(state),
        };
    }
    pub fn result(&self) -> Option<HtmlInputType> {
        if let TypeStep::Done(state) = self.step {
            Some(state)
        } else {
            None
        }
    }
}

enum ColorSpaceStep {
    Row,
    Length(&'static str, crate::HtmlInputColorSpace),
    Unit(&'static str, crate::HtmlInputColorSpace, usize),
    Done(crate::HtmlInputColorSpace),
}
/// Borrowed recognizer over the authoritative attributes-owned colorspace rows.
/// No trimming, allocation, copied keyword table or whole-string advance.
pub struct HtmlInputColorSpaceCursor<'a> {
    value: Option<&'a DocumentString>,
    row: usize,
    step: ColorSpaceStep,
}
impl<'a> HtmlInputColorSpaceCursor<'a> {
    pub fn new(value: Option<&'a DocumentString>) -> Self {
        Self {
            value,
            row: 0,
            step: ColorSpaceStep::Row,
        }
    }
    pub fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match self.step {
            ColorSpaceStep::Done(_) => None,
            ColorSpaceStep::Unit(..) => Some(HtmlInputRuleWork::CompareAsciiUnit),
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    /// At most one row lookup, length decision or UTF-16/ASCII comparison.
    /// The caller admits required_work before calling this method.
    pub fn advance(&mut self) {
        self.step = match self.step {
            ColorSpaceStep::Row => {
                if self.value.is_none() {
                    ColorSpaceStep::Done(crate::attributes::input_colorspace_default())
                } else if let Some((keyword, state)) =
                    crate::attributes::input_colorspace_keyword(self.row)
                {
                    self.row += 1;
                    ColorSpaceStep::Length(keyword, state)
                } else {
                    ColorSpaceStep::Done(crate::attributes::input_colorspace_invalid_default())
                }
            }
            ColorSpaceStep::Length(keyword, state) => {
                if self.value.expect("present row").len() != keyword.len() {
                    ColorSpaceStep::Row
                } else if keyword.is_empty() {
                    ColorSpaceStep::Done(state)
                } else {
                    ColorSpaceStep::Unit(keyword, state, 0)
                }
            }
            ColorSpaceStep::Unit(keyword, state, position) => {
                if !crate::attributes::ascii_keyword_unit(
                    self.value.expect("present row").code_units()[position],
                    keyword.as_bytes()[position],
                ) {
                    ColorSpaceStep::Row
                } else if position + 1 == keyword.len() {
                    ColorSpaceStep::Done(state)
                } else {
                    ColorSpaceStep::Unit(keyword, state, position + 1)
                }
            }
            ColorSpaceStep::Done(state) => ColorSpaceStep::Done(state),
        };
    }
    pub fn result(&self) -> Option<crate::HtmlInputColorSpace> {
        if let ColorSpaceStep::Done(state) = self.step {
            Some(state)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputValueComparison {
    equal: bool,
}
impl HtmlInputValueComparison {
    pub fn is_equal(self) -> bool {
        self.equal
    }
}
/// One pair per advance; exact code units, not Unicode or semantic equality.
pub struct HtmlInputValueComparisonCursor<'a> {
    old: &'a [u16],
    current: &'a [u16],
    position: usize,
    result: Option<HtmlInputValueComparison>,
    length_pending: bool,
}
impl<'a> HtmlInputValueComparisonCursor<'a> {
    pub fn new(old: &'a [u16], current: &'a [u16]) -> Self {
        Self {
            old,
            current,
            position: 0,
            result: None,
            length_pending: true,
        }
    }
    pub fn required_work(&self) -> Option<HtmlInputRuleWork> {
        if self.result.is_some() {
            None
        } else if self.length_pending {
            Some(HtmlInputRuleWork::ReadValueLengths)
        } else {
            Some(HtmlInputRuleWork::CompareValueUnits)
        }
    }
    pub fn advance(&mut self) {
        if self.result.is_some() {
            return;
        }
        if self.length_pending {
            self.length_pending = false;
            if self.old.len() != self.current.len() {
                self.result = Some(HtmlInputValueComparison { equal: false })
            } else if self.old.is_empty() {
                self.result = Some(HtmlInputValueComparison { equal: true })
            }
        } else if self.old[self.position] != self.current[self.position] {
            self.result = Some(HtmlInputValueComparison { equal: false })
        } else {
            self.position += 1;
            if self.position == self.old.len() {
                self.result = Some(HtmlInputValueComparison { equal: true })
            }
        }
    }
    pub fn result(&self) -> Option<HtmlInputValueComparison> {
        self.result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputSelectionDirection {
    None,
    Forward,
    Backward,
}
/// Qualified actual position. No direction/platform support is inferred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionState {
    Cursor(usize),
    Selection {
        start: usize,
        end: usize,
        direction: HtmlInputSelectionDirection,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputSelection {
    state: SelectionState,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputInvalidSelectionRange {
    start: usize,
    end: usize,
}
impl HtmlInputInvalidSelectionRange {
    pub fn start(self) -> usize {
        self.start
    }
    pub fn end(self) -> usize {
        self.end
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputEmptySelectionSupport {
    Supported,
    Unsupported,
}
impl HtmlInputSelection {
    pub fn cursor(offset: usize) -> Self {
        Self {
            state: SelectionState::Cursor(offset),
        }
    }
    pub fn selection(
        start: usize,
        end: usize,
        direction: HtmlInputSelectionDirection,
    ) -> Result<Self, HtmlInputInvalidSelectionRange> {
        if start > end {
            Err(HtmlInputInvalidSelectionRange { start, end })
        } else {
            Ok(Self {
                state: SelectionState::Selection {
                    start,
                    end,
                    direction,
                },
            })
        }
    }
    pub fn cursor_position(self) -> Option<usize> {
        if let SelectionState::Cursor(position) = self.state {
            Some(position)
        } else {
            None
        }
    }
    pub fn selection_range(self) -> Option<(usize, usize, HtmlInputSelectionDirection)> {
        if let SelectionState::Selection {
            start,
            end,
            direction,
        } = self.state
        {
            Some((start, end, direction))
        } else {
            None
        }
    }
    /// HTML's relevant-value change clamp; fixed work, no string read.
    pub fn clamp(self, actual_value_len: usize, support: HtmlInputEmptySelectionSupport) -> Self {
        match self.state {
            SelectionState::Cursor(offset) => Self::cursor(offset.min(actual_value_len)),
            SelectionState::Selection {
                start,
                end,
                direction,
            } => {
                let start = start.min(actual_value_len);
                let end = end.min(actual_value_len);
                if support == HtmlInputEmptySelectionSupport::Unsupported
                    && start == actual_value_len
                    && end == actual_value_len
                {
                    Self::cursor(actual_value_len)
                } else {
                    Self {
                        state: SelectionState::Selection {
                            start,
                            end,
                            direction,
                        },
                    }
                }
            }
        }
    }
}
/// Additional Value setter effect only; evaluate after actual generic clamping.
/// None-direction application still requires the adapter's real platform facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueSetterFinish {
    NoAdditionalEffect,
    SetEndCursorUnselectAndNoneDirection,
}
pub fn input_value_setter_finish(
    kind: HtmlInputType,
    comparison: HtmlInputValueComparison,
    post_clamp: HtmlInputSelection,
) -> HtmlInputValueSetterFinish {
    if kind.is_selectable() && !comparison.is_equal() && post_clamp.cursor_position().is_some() {
        HtmlInputValueSetterFinish::SetEndCursorUnselectAndNoneDirection
    } else {
        HtmlInputValueSetterFinish::NoAdditionalEffect
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNoneDirectionSupport {
    Supported,
    Unsupported,
}
impl HtmlInputSelectionDirection {
    pub fn resolve_none(self, support: HtmlInputNoneDirectionSupport) -> Self {
        if self == Self::None && support == HtmlInputNoneDirectionSupport::Unsupported {
            Self::Forward
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::control::{ParseControl, ParseStop, WorkCategory};
    use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};

    fn control(work: u64, cancellation: HtmlParseCancellation) -> ParseControl {
        let limits = HtmlParseLimits::builder()
            .source_units(1000)
            .buffered_utf16_units(1000)
            .token_attributes(1000)
            .auxiliary_entries(1000)
            .managed_nodes(1000)
            .managed_edges(1000)
            .managed_attributes(1000)
            .managed_utf16_units(1000)
            .source_map_entries(1000)
            .source_map_payload_units(1000)
            .diagnostics(1000)
            .diagnostic_payload_units(1000)
            .context_effects(1000)
            .open_elements(1000)
            .tree_depth(1000)
            .managed_capacity_bytes(16000)
            .peak_transient_bytes(32000)
            .atomic_layout_bytes(16000)
            .work_credits(work)
            .finish()
            .unwrap();
        ParseControl::new(limits, cancellation)
    }
    fn admit(c: &mut ParseControl, work: HtmlInputRuleWork) -> Result<(), ParseStop> {
        match work {
            HtmlInputRuleWork::ReadField => c.charge(WorkCategory::RecordVisit, 1),
            HtmlInputRuleWork::ReadValueLengths => c.charge(WorkCategory::RecordVisit, 2),
            HtmlInputRuleWork::CompareAsciiUnit => c.charge(WorkCategory::InspectByte, 3),
            HtmlInputRuleWork::CompareValueUnits | HtmlInputRuleWork::InspectValueUnit => {
                c.charge(WorkCategory::InspectByte, 2)
            }
        }
    }
    #[test]
    fn type_exact_sixteen_and_one_short_stop_before_final_pair_and_keep_original() {
        // hidden row+length2; text row+length2; four real pairs*3 =16.
        let raw = DocumentString::from("TeXt");
        for cap in [15, 16] {
            let mut c = control(cap, HtmlParseCancellation::new());
            let mut cursor = HtmlInputTypeCursor::new(Some(&raw));
            while let Some(work) = cursor.required_work() {
                if admit(&mut c, work).is_err() {
                    break;
                }
                cursor.advance();
            }
            if cap == 16 {
                assert_eq!(cursor.result(), Some(HtmlInputType::Text));
                assert_eq!(c.stop(), None);
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    16
                );
            } else {
                assert_eq!(cursor.result(), None);
                assert!(matches!(
                    c.stop(),
                    Some(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!(
                    c.usage().resource(HtmlParseResource::WorkCredits).current,
                    13
                );
                assert_eq!(
                    cursor.required_work(),
                    Some(HtmlInputRuleWork::CompareAsciiUnit)
                );
            }
            assert_eq!(raw.code_units(), [84, 101, 88, 116]);
            assert_eq!(c.usage().actual_capacity_high_water, 0);
        }
    }
    #[test]
    fn comparison_exact_eight_and_one_short_use_actual_controller_without_materialization() {
        let value = [0xd800, 0x41, 0xdfff];
        // Two length metadata reads + three pairs of actual UTF16 units =8.
        for cap in [7, 8] {
            let mut c = control(cap, HtmlParseCancellation::new());
            let mut cursor = HtmlInputValueComparisonCursor::new(&value, &value);
            while let Some(work) = cursor.required_work() {
                if admit(&mut c, work).is_err() {
                    break;
                }
                cursor.advance();
            }
            if cap == 8 {
                assert!(cursor.result().unwrap().is_equal());
                assert_eq!(c.stop(), None);
            } else {
                assert_eq!(cursor.result(), None);
                assert!(matches!(
                    c.stop(),
                    Some(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
            }
            assert_eq!(
                c.usage().resource(HtmlParseResource::WorkCredits).current,
                if cap == 8 { 8 } else { 6 }
            );
            assert_eq!(c.usage().actual_capacity_high_water, 0);
        }
    }
    #[test]
    fn cancellation_before_pair_retains_unfinished_cursor_and_canonical_first_stop() {
        let cancellation = HtmlParseCancellation::new();
        let mut c = control(100, cancellation.clone());
        let old = [1, 2];
        let new = [1, 3];
        let mut cursor = HtmlInputValueComparisonCursor::new(&old, &new);
        admit(&mut c, cursor.required_work().unwrap()).unwrap();
        cursor.advance();
        cancellation.cancel();
        assert_eq!(
            admit(&mut c, cursor.required_work().unwrap()),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(cursor.result(), None);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits).current,
            2
        );
        assert_eq!(
            admit(&mut c, cursor.required_work().unwrap()),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(c.first_stop_usage(), Some(c.usage()));
        assert_eq!(c.usage().actual_capacity_high_water, 0);
    }
}

/// HTML877d #radio-button-group triggers. These are source operations, not host
/// events; Connected requires an actual insertion hook and now-connected proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRadioTrigger {
    Checkedness,
    Name,
    FormOwner,
    Type,
    Connected,
}

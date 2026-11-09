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

use crate::{
    internal::accounting::{HtmlChange, HtmlResource, HtmlWork},
    internal::attributes::HtmlInputType,
    internal::elements::control_values::HtmlTextControlValueSetterFinish,
    internal::values::DocumentString,
};
use surgeist_ledger::{Change, Charge};

impl HtmlInputType {
    /// Whether this Input state uses its current value for auto directionality.
    ///
    /// HTML877d #auto-directionality-form-associated-elements selects these
    /// states. This rule neither resolves directionality nor supplies a current
    /// value; adapters retain actual value facts and their bounded reads.
    pub const fn uses_current_value_for_directionality(self) -> bool {
        matches!(
            self,
            Self::Hidden
                | Self::Text
                | Self::Search
                | Self::Telephone
                | Self::Url
                | Self::Email
                | Self::Password
                | Self::Submit
                | Self::Reset
                | Self::Button
        )
    }
    /// The mode of the `value` IDL attribute that this type state selects: `Default` for Hidden,
    /// Submit, Reset, Button and Image, `DefaultOn` for Checkbox and Radio, `Filename` for File,
    /// and `Value` for every other state. HTML877d #dom-input-value defines the four modes. The
    /// mode decides whether the value reads and writes the control's current value, the `value`
    /// content attribute or the selected-file list.
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
    /// The value sanitization requirement that this type state defines. Range and Color return
    /// their own variants because they need numeric or color facts that a string rule cannot carry.
    /// Number, the five temporal states, Email and Url return their specific string rule; Text,
    /// Search, Telephone and Password strip newlines. A state whose `value` mode is not `Value`
    /// returns `Preserve`, which is the identity rule: the standard defines no sanitization for it.
    /// See HTML877d #value-sanitization-algorithm.
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
/// The mode in which the `value` IDL attribute reads and writes, selected by the `type` state.
/// HTML877d #dom-input-value lists the modes; `HtmlInputType::value_mode` maps each state to one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueMode {
    /// Reads and writes the control's current value. A write also sets the dirty value flag and
    /// runs the state's value sanitization (HTML877d #dom-input-value-value).
    Value,
    /// Reads and writes the `value` content attribute; an absent attribute reads as the empty
    /// string (HTML877d #dom-input-value-default).
    Default,
    /// Reads and writes the `value` content attribute like `Default`, but an absent attribute reads
    /// as the string `on` (HTML877d #dom-input-value-default-on).
    DefaultOn,
    /// Reads from the list of selected files and accepts only the empty string on write, which
    /// empties the list. A nonempty write is refused with `HtmlInputFilenameInvalidState` (HTML877d
    /// #dom-input-value-filename).
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
    /// No file is selected. This is the only established selection; the getter reads it as the
    /// empty string and the setter produces it for an empty write.
    Empty,
}
/// Filename setter refusal for a nonempty proposed value. No files are created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputFilenameInvalidState;
impl std::fmt::Display for HtmlInputFilenameInvalidState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a file input's value can only be set to the empty string")
    }
}
impl std::error::Error for HtmlInputFilenameInvalidState {}
/// The `Filename`-mode setter rule: an empty proposed value empties the selected files, so it
/// returns the `Empty` selection. Any nonempty value is refused with
/// `HtmlInputFilenameInvalidState`, which corresponds to the standard's `InvalidStateError`. The
/// value is a slice of exact UTF-16 code units, and nothing is allocated or retained. See HTML877d
/// #dom-input-value-filename.
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
    /// Fresh Image-state construction starts with the selected zero coordinates.
    pub fn initial() -> Self {
        Self { x: 0, y: 0 }
    }
    /// The x-component of the selected coordinate as an unsigned integer; `initial` yields zero
    /// (HTML877d #concept-input-type-image-coordinate).
    pub fn x(self) -> u32 {
        self.x
    }
    /// The y-component of the selected coordinate as an unsigned integer; `initial` yields zero
    /// (HTML877d #concept-input-type-image-coordinate).
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
    /// A string-level rule that `HtmlInputSanitizationCursor` evaluates on exact UTF-16 code
    /// units. `Email` also needs whether the `multiple` attribute is present
    /// (`HtmlInputEmailMode`).
    /// `HtmlInputSanitization::retain_unit` completes only `Preserve` and `StripCrLf`; the other
    /// rules need trimming, comma processing or whole-value validation, which a per-unit
    /// predicate cannot supply.
    String(HtmlInputSanitization),
    /// The Range sanitization, which needs the control's minimum, maximum, step and value content
    /// and an exact numeric producer. Nothing in this module can supply those facts.
    Range,
    /// The Color sanitization, which needs the dirty or default value source, the `alpha` and
    /// `colorspace` attributes and the CSS used-color operation, none of which this module owns.
    Color,
}
/// Ordered intrinsic Color serialization steps. CSS owns each conversion,
/// numerical operation and spelling; this cursor owns only HTML's sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputColorEffect {
    /// Sets the color's alpha component to fully opaque. It applies when the `alpha` attribute is
    /// not specified.
    ForceOpaqueAlpha,
    /// Converts the color to the given `colorspace` state's space: `srgb` for Limited sRGB and
    /// `display-p3` for Display P3.
    Convert(crate::internal::attributes::input_colorspace::HtmlInputColorSpace),
    /// Rounds each sRGB component toward positive infinity into the range 0 to 255. It applies only
    /// to the Limited sRGB state.
    QuantizeSrgb,
    /// Converts the color with the CSS `color()` function. It applies only to the Limited sRGB
    /// state when the `alpha` attribute is specified.
    ColorFunction,
}
/// The ordered Color serialization steps of HTML877d #serialize-a-color-well-control-color for one
/// control. The cursor yields each step through `next_effect`; CSS performs the conversions and
/// spelling. It is a single-pass state machine, so it is not `Clone`.
#[derive(Debug)]
pub struct HtmlInputColorSerialization {
    alpha: bool,
    space: crate::internal::attributes::input_colorspace::HtmlInputColorSpace,
    step: u8,
}
impl HtmlInputColorSerialization {
    /// Starts the steps for a control whose `alpha` attribute is specified or not (`alpha`) and
    /// whose `colorspace` attribute is in the given state (`space`).
    pub fn new(
        alpha: bool,
        space: crate::internal::attributes::input_colorspace::HtmlInputColorSpace,
    ) -> Self {
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
            2 if self.space == crate::internal::attributes::input_colorspace::HtmlInputColorSpace::LimitedSrgb => {
                Some(HtmlInputColorEffect::QuantizeSrgb)
            }
            3 if self.space == crate::internal::attributes::input_colorspace::HtmlInputColorSpace::LimitedSrgb && self.alpha => {
                Some(HtmlInputColorEffect::ColorFunction)
            }
            _ => None,
        }
    }
    /// Whether the serialization requests CSS's HTML-compatible form: true exactly when `alpha` is
    /// not specified and the `colorspace` state is Limited sRGB. It does not depend on how far the
    /// cursor has advanced.
    pub fn html_compatible(&self) -> bool {
        !self.alpha
            && self.space
                == crate::internal::attributes::input_colorspace::HtmlInputColorSpace::LimitedSrgb
    }
}
/// A string-level value sanitization rule. Each variant names one rule from the `type` states in
/// HTML877d #states-of-the-type-attribute; `HtmlInputSanitizationCursor` evaluates it over UTF-16
/// code units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputSanitization {
    /// Leaves the value unchanged.
    Preserve,
    /// Removes every U+000D and U+000A code unit (strip newlines).
    StripCrLf,
    /// Removes every U+000D and U+000A code unit, then strips leading and trailing ASCII
    /// whitespace.
    StripCrLfAndTrim,
    /// The complete operation requires actual multiple-attribute presence.
    Email,
    /// Strict source-defined temporal grammar; invalid becomes empty.
    Temporal(HtmlInputTemporalFormat),
    /// Replaces a value that is not a valid floating-point number with the empty string. A valid
    /// value is kept unchanged.
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
    /// Keeps a valid date string and otherwise yields the empty string.
    Date,
    /// Keeps a valid month string and otherwise yields the empty string.
    Month,
    /// Keeps a valid week string and otherwise yields the empty string.
    Week,
    /// Keeps a valid time string and otherwise yields the empty string.
    Time,
    /// Normalizes a valid local date and time string and otherwise yields the empty string.
    /// Normalizing replaces a space separator with `T` and writes the time in its shortest form,
    /// dropping zero seconds and trailing zero fraction digits.
    LocalDateTime,
}
impl HtmlInputTemporalFormat {
    fn owner(self) -> crate::internal::microsyntax::temporal::TemporalInputFormat {
        match self {
            Self::Date => crate::internal::microsyntax::temporal::TemporalInputFormat::Date,
            Self::Month => crate::internal::microsyntax::temporal::TemporalInputFormat::Month,
            Self::Week => crate::internal::microsyntax::temporal::TemporalInputFormat::Week,
            Self::Time => crate::internal::microsyntax::temporal::TemporalInputFormat::Time,
            Self::LocalDateTime => {
                crate::internal::microsyntax::temporal::TemporalInputFormat::LocalDateTime
            }
        }
    }
}

/// Actual Email sanitizer mode, selected by specified multiple presence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputEmailMode {
    /// The `multiple` attribute is not specified: the value is one address, with newlines stripped
    /// and leading and trailing ASCII whitespace removed.
    Single,
    /// The `multiple` attribute is specified: the value is a comma-separated list, and each token
    /// is stripped of surrounding ASCII whitespace.
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
    /// The offset of the first retained code unit in the source slice. It never exceeds `end`.
    pub fn start(self) -> usize {
        self.start
    }
    /// The offset one past the last retained code unit in the source slice, so `start..end` is a
    /// valid half-open range of the source. It never exceeds the source length.
    pub fn end(self) -> usize {
        self.end
    }
}
enum SanitizerStep<'a> {
    Temporal(crate::internal::microsyntax::temporal::TemporalValidationCursor<'a>),
    Number(crate::internal::microsyntax::numeric::HtmlFloatingPointGrammarCursor<'a>),
    Leading,
    Trailing,
    Emit,
    Commas,
    Pending,
    Done,
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
    step: SanitizerStep<'a>,
    commas: crate::internal::microsyntax::tokens::CommaTokenCursor<'a>,
    pending: Option<HtmlInputRetainedRange>,
    count: usize,
    temporal_plan: Option<crate::internal::microsyntax::temporal::TemporalInputPlan>,
}
impl std::fmt::Debug for HtmlInputSanitizationCursor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HtmlInputSanitizationCursor")
            .field("policy", &self.policy)
            .field("mode", &self.mode)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish_non_exhaustive()
    }
}
impl<'a> HtmlInputSanitizationCursor<'a> {
    /// Starts the sanitization of `units` under `policy`. `mode` matters only for
    /// `HtmlInputSanitization::Email`, where `Multiple` selects comma splitting. The slice is
    /// borrowed and never copied.
    pub fn new(policy: HtmlInputSanitization, units: &'a [u16], mode: HtmlInputEmailMode) -> Self {
        let step = match policy {
            HtmlInputSanitization::Temporal(format) => SanitizerStep::Temporal(
                crate::internal::microsyntax::temporal::TemporalValidationCursor::new(
                    units,
                    format.owner(),
                    true,
                ),
            ),
            HtmlInputSanitization::Number => SanitizerStep::Number(
                crate::internal::microsyntax::numeric::HtmlFloatingPointGrammarCursor::new(
                    units,
                    crate::internal::microsyntax::numeric::HtmlFloatingPointParseMode::Author,
                ),
            ),
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
            commas: crate::internal::microsyntax::tokens::CommaTokenCursor::new(units),
            pending: None,
            count: 0,
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
    /// The fixed work the next step performs: `InspectValueUnit` when it examines one source
    /// unit, `ReadField` for a state transition, and `None` once the cursor is done.
    fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match &self.step {
            SanitizerStep::Done => None,
            SanitizerStep::Number(cursor) => Some(if cursor.inspects_unit() {
                HtmlInputRuleWork::InspectValueUnit
            } else {
                HtmlInputRuleWork::ReadField
            }),
            SanitizerStep::Temporal(cursor) => Some(if cursor.inspects_unit() {
                HtmlInputRuleWork::InspectValueUnit
            } else {
                HtmlInputRuleWork::ReadField
            }),
            SanitizerStep::Leading | SanitizerStep::Trailing if self.start < self.end => {
                Some(HtmlInputRuleWork::InspectValueUnit)
            }
            SanitizerStep::Commas if self.commas.inspection().is_some() => {
                Some(HtmlInputRuleWork::InspectValueUnit)
            }
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    /// Whether the cursor is done, so that no further step has work to charge.
    pub fn is_complete(&self) -> bool {
        matches!(self.step, SanitizerStep::Done)
    }
    /// Charges the work of the next step, as one schema change, and charges nothing once the
    /// cursor is done. A caller that charges work of its own between this charge and the step
    /// calls this, then `advance_charged`.
    pub fn charge_next<C: Charge<HtmlWork, HtmlResource>>(
        &self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        match self.required_work() {
            Some(work) => charge.charge(&[work.change()]),
            None => Ok(()),
        }
    }
    /// Charges the next step's work, then performs the step. See `advance_charged` for the
    /// result.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        charge: &mut C,
    ) -> Result<Option<HtmlInputRetainedRange>, C::Stop> {
        self.charge_next(charge)?;
        Ok(self.advance_charged())
    }
    /// Performs one step whose work `charge_next` has charged. It returns a retained source
    /// interval when the step emits one and `None` otherwise, including after the cursor is done.
    /// An Email list yields each token and each separating comma as its own interval.
    pub fn advance_charged(&mut self) -> Option<HtmlInputRetainedRange> {
        match &mut self.step {
            SanitizerStep::Number(cursor) => {
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
            SanitizerStep::Temporal(scanner) => {
                scanner.advance();
                if scanner.finished() {
                    self.temporal_plan = scanner.normalization();
                    self.end = self.temporal_plan.map_or(0, |plan| plan.retained_end);
                    self.step = SanitizerStep::Emit;
                }
            }

            SanitizerStep::Leading => {
                if self.start < self.end
                    && crate::internal::microsyntax::tokens::is_ascii_whitespace(
                        self.units[self.start],
                    )
                {
                    self.start += 1;
                } else {
                    self.step = SanitizerStep::Trailing;
                }
            }
            SanitizerStep::Trailing => {
                if self.start < self.end
                    && crate::internal::microsyntax::tokens::is_ascii_whitespace(
                        self.units[self.end - 1],
                    )
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
    /// Whether the code unit survives this cursor's policy. Email in `Single` mode drops U+000D and
    /// U+000A; every other combination defers to `HtmlInputSanitization::retain_unit`.
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
    /// The type state at the time these flags were captured.
    pub fn input_type(self) -> HtmlInputType {
        self.kind
    }
    /// The dirty value flag (HTML877d #concept-fe-dirty): true once the current value no longer
    /// follows the `value` content attribute.
    pub fn dirty_value(self) -> bool {
        self.dirty_value
    }
    /// The dirty checkedness flag (HTML877d #concept-input-checked-dirty-flag): true once the
    /// checkedness no longer follows the `checked` content attribute.
    pub fn dirty_checkedness(self) -> bool {
        self.dirty_checkedness
    }
    /// The current checkedness (HTML877d #concept-fe-checked).
    pub fn checkedness(self) -> bool {
        self.checkedness
    }
    /// The indeterminateness (HTML877d #concept-input-indeterminate). It is independent of the
    /// checkedness.
    pub fn indeterminate(self) -> bool {
        self.indeterminate
    }
    /// The user validity flag, which reset clears and no other rule here changes.
    pub fn user_validity(self) -> bool {
        self.user_validity
    }
    /// The effect of a change to the `value` content attribute: `KeepCurrent` when the dirty value
    /// flag is set, otherwise the current value is replaced from the installed default and
    /// sanitized under this state's rule.
    pub fn value_attribute_changed(self) -> HtmlInputValueAttributeEffect {
        if self.dirty_value {
            HtmlInputValueAttributeEffect::KeepCurrent
        } else {
            HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(
                self.kind.sanitization(),
            )
        }
    }
    /// Applies the effect of adding or removing the `checked` content attribute. It returns the
    /// updated flags, or `None` when the dirty checkedness flag is set and the attribute change has
    /// no effect.
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
    /// Sets the checkedness to `actual_checked_presence`, the presence of the `checked` content
    /// attribute. It is the checkedness step of the reset algorithm and leaves every other flag
    /// unchanged.
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
/// The effect of a change to the `value` content attribute on the current value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueAttributeEffect {
    /// The dirty value flag is set, so the current value is not changed.
    KeepCurrent,
    /// The current value is replaced by the installed default and then sanitized under the given
    /// requirement.
    ReplaceCurrentFromInstalledDefault(HtmlInputValueSanitization),
}
/// Whether the `checked` content attribute was added or removed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputCheckedAttributeChange {
    /// The attribute is now present.
    Added,
    /// The attribute is now absent.
    Removed,
}
/// The first requirement of a `value` IDL attribute write, selected by the mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputValueSetterStart {
    /// Apply input_filename_setter to the proposed value before clearing files.
    SetFilename,
    /// `Default` and `DefaultOn` modes: write the value to the `value` content attribute instead of
    /// the current value.
    SetContentValue,
    /// `Value` mode: install the supplied value as the current value, using these flags (with the
    /// dirty value flag set), then sanitize it.
    SetCurrentValue(HtmlInputFlags),
}

/// Source reset order. This is per-control reset, not a form reset event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputResetEffect {
    /// Clear the dirty value flag, the dirty checkedness flag and the user validity flag, using
    /// these flags.
    ClearDirtyAndUserValidity(HtmlInputFlags),
    /// Set the current value from the installed default (the `value` content attribute or the empty
    /// string).
    ReplaceCurrentFromDefault,
    /// Set the checkedness to the presence of the `checked` content attribute.
    SetCheckedFromPresence,
    /// Empty the list of selected files.
    EmptySelectedFiles,
    /// Run the value sanitization under the given requirement.
    Sanitize(HtmlInputValueSanitization),
}
/// The per-control reset algorithm as ordered effects. It is not a form reset event. A single-pass
/// cursor, so it is not `Clone`.
#[derive(Debug)]
pub struct HtmlInputReset {
    initial: HtmlInputFlags,
    stage: u8,
}
impl HtmlInputReset {
    /// Starts the reset of a control whose flags are `initial`. The type state in `initial` selects
    /// the final sanitization.
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
    /// Set the `value` content attribute to the current value (a Value-mode state became Default or
    /// DefaultOn with a nonempty value).
    SetContentValueFromCurrent,
    /// Set the current value from the `value` content attribute or the empty string, and clear the
    /// dirty value flag (a non-Value mode became Value mode).
    SetCurrentFromDefaultAndClearDirty,
    /// Set the current value to the empty string (a non-Filename mode became Filename mode).
    SetCurrentEmptyForFilename,
    /// Update the control's rendering and behavior to the new state.
    SetRendering,
    /// Signal the type change, which the Radio Button state uses.
    SignalTypeChange,
    /// Run the new state's value sanitization under the given requirement.
    Sanitize(HtmlInputValueSanitization),
    /// Set the text entry cursor to the beginning of the text control and the selection direction
    /// to `none` (the state became selectable and was not before).
    SetInitialCursorAndNoneDirection,
}
/// The ordered effects of the type-change algorithm in HTML877d #input-type-change. The adapter
/// applies each effect, including nested attribute effects, before it asks for the next one.
/// Fixed-size and allocation-free; it holds no consumer references, so every effect still needs
/// the consumer's own operation.
#[derive(Clone, Copy, Debug)]
pub struct HtmlInputTypeChange {
    previous: HtmlInputType,
    next: HtmlInputType,
    nonempty: bool,
    stage: u8,
}
impl HtmlInputTypeChange {
    /// Starts a change from `previous` to `next` for a control whose current value is
    /// `current_value_len` UTF-16 code units long. Equal states produce no effects.
    pub fn new(previous: HtmlInputType, next: HtmlInputType, current_value_len: usize) -> Self {
        Self {
            previous,
            next,
            nonempty: current_value_len != 0,
            stage: if previous == next { 5 } else { 0 },
        }
    }
    /// The next effect in source order, or `None` when the algorithm is complete. The adapter
    /// applies it to the updated state before calling this again.
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

/// Admission before one fixed cursor action, charged as the schema's record
/// visits or inspected bytes; these cursors neither poll nor hold a controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlInputRuleWork {
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

impl HtmlInputRuleWork {
    /// The schema change that admits this work. A length pair reads two records. An ASCII
    /// comparison inspects the unit's two bytes and the literal's one. A value unit comparison or
    /// inspection inspects the unit's two bytes.
    pub(crate) fn change(self) -> HtmlChange {
        match self {
            Self::ReadField => Change::Work(HtmlWork::RecordVisit, 1),
            Self::ReadValueLengths => Change::Work(HtmlWork::RecordVisit, 2),
            Self::CompareAsciiUnit => Change::Work(HtmlWork::InspectByte, 3),
            Self::CompareValueUnits | Self::InspectValueUnit => {
                Change::Work(HtmlWork::InspectByte, 2)
            }
        }
    }
}

#[derive(Debug)]
enum TypeStep<'a> {
    Row,
    Length(&'a DocumentString, &'static str, HtmlInputType),
    Unit(&'a DocumentString, &'static str, HtmlInputType, usize),
    Done(HtmlInputType),
}
/// Borrowed recognizer over the authoritative attributes-owned 22-state rows.
/// No trimming, allocation, copied keyword table or whole-string advance.
#[derive(Debug)]
pub struct HtmlInputTypeCursor<'a> {
    value: Option<&'a DocumentString>,
    row: usize,
    step: TypeStep<'a>,
}
impl<'a> HtmlInputTypeCursor<'a> {
    /// Starts recognition of `value`, the `type` attribute's content, where `None` means the
    /// attribute is absent. The slice is borrowed and nothing is copied.
    pub fn new(value: Option<&'a DocumentString>) -> Self {
        Self {
            value,
            row: 0,
            step: TypeStep::Row,
        }
    }
    /// The fixed work the next step performs, or `None` once `result` is available.
    fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match self.step {
            TypeStep::Done(_) => None,
            TypeStep::Unit(..) => Some(HtmlInputRuleWork::CompareAsciiUnit),
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    /// Charges the next step's work as one schema change and performs at most one row lookup,
    /// length decision or UTF-16/ASCII comparison. A refused charge leaves the cursor as it was.
    /// A completed cursor charges nothing.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        if let Some(work) = self.required_work() {
            charge.charge(&[work.change()])?;
        }
        self.step();
        Ok(())
    }
    fn step(&mut self) {
        self.step = match self.step {
            TypeStep::Row => {
                if let Some(value) = self.value {
                    if let Some((keyword, state)) =
                        crate::internal::attributes::input_type_keyword(self.row)
                    {
                        self.row += 1;
                        TypeStep::Length(value, keyword, state)
                    } else {
                        TypeStep::Done(crate::internal::attributes::input_type_invalid_default())
                    }
                } else {
                    TypeStep::Done(
                        crate::internal::attributes::HtmlInputType::missing_value_default(),
                    )
                }
            }
            TypeStep::Length(value, keyword, state) => {
                if value.len() != keyword.len() {
                    TypeStep::Row
                } else if keyword.is_empty() {
                    TypeStep::Done(state)
                } else {
                    TypeStep::Unit(value, keyword, state, 0)
                }
            }
            TypeStep::Unit(value, keyword, state, position) => {
                if !crate::internal::microsyntax::tokens::ascii_case_insensitive_unit(
                    value.code_units()[position],
                    keyword.as_bytes()[position],
                ) {
                    TypeStep::Row
                } else if position + 1 == keyword.len() {
                    TypeStep::Done(state)
                } else {
                    TypeStep::Unit(value, keyword, state, position + 1)
                }
            }
            TypeStep::Done(state) => TypeStep::Done(state),
        };
    }
    /// The recognized state: the matching keyword's state, the invalid-value default if no keyword
    /// matches, or the missing-value default when the attribute is absent. It is `None` while
    /// recognition is incomplete. Keywords match ASCII case-insensitively over the exact UTF-16
    /// content.
    pub fn result(&self) -> Option<HtmlInputType> {
        if let TypeStep::Done(state) = self.step {
            Some(state)
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum ColorSpaceStep<'a> {
    Row,
    Length(
        &'a DocumentString,
        &'static str,
        crate::internal::attributes::input_colorspace::HtmlInputColorSpace,
    ),
    Unit(
        &'a DocumentString,
        &'static str,
        crate::internal::attributes::input_colorspace::HtmlInputColorSpace,
        usize,
    ),
    Done(crate::internal::attributes::input_colorspace::HtmlInputColorSpace),
}
/// Borrowed recognizer over the authoritative attributes-owned colorspace rows.
/// No trimming, allocation, copied keyword table or whole-string advance.
#[derive(Debug)]
pub struct HtmlInputColorSpaceCursor<'a> {
    value: Option<&'a DocumentString>,
    row: usize,
    step: ColorSpaceStep<'a>,
}
impl<'a> HtmlInputColorSpaceCursor<'a> {
    /// Starts recognition of `value`, the `colorspace` attribute's content, where `None` means the
    /// attribute is absent. The slice is borrowed and nothing is copied.
    pub fn new(value: Option<&'a DocumentString>) -> Self {
        Self {
            value,
            row: 0,
            step: ColorSpaceStep::Row,
        }
    }
    /// The fixed work the next step performs, or `None` once `result` is available.
    fn required_work(&self) -> Option<HtmlInputRuleWork> {
        match self.step {
            ColorSpaceStep::Done(_) => None,
            ColorSpaceStep::Unit(..) => Some(HtmlInputRuleWork::CompareAsciiUnit),
            _ => Some(HtmlInputRuleWork::ReadField),
        }
    }
    /// Charges the next step's work as one schema change and performs at most one row lookup,
    /// length decision or UTF-16/ASCII comparison. A refused charge leaves the cursor as it was.
    /// A completed cursor charges nothing.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        if let Some(work) = self.required_work() {
            charge.charge(&[work.change()])?;
        }
        self.step();
        Ok(())
    }
    fn step(&mut self) {
        self.step = match self.step {
            ColorSpaceStep::Row => {
                if let Some(value) = self.value {
                    if let Some((keyword, state)) =
                        crate::internal::attributes::input_colorspace_keyword(self.row)
                    {
                        self.row += 1;
                        ColorSpaceStep::Length(value, keyword, state)
                    } else {
                        ColorSpaceStep::Done(
                            crate::internal::attributes::input_colorspace_invalid_default(),
                        )
                    }
                } else {
                    ColorSpaceStep::Done(crate::internal::attributes::input_colorspace_default())
                }
            }
            ColorSpaceStep::Length(value, keyword, state) => {
                if value.len() != keyword.len() {
                    ColorSpaceStep::Row
                } else if keyword.is_empty() {
                    ColorSpaceStep::Done(state)
                } else {
                    ColorSpaceStep::Unit(value, keyword, state, 0)
                }
            }
            ColorSpaceStep::Unit(value, keyword, state, position) => {
                if !crate::internal::microsyntax::tokens::ascii_case_insensitive_unit(
                    value.code_units()[position],
                    keyword.as_bytes()[position],
                ) {
                    ColorSpaceStep::Row
                } else if position + 1 == keyword.len() {
                    ColorSpaceStep::Done(state)
                } else {
                    ColorSpaceStep::Unit(value, keyword, state, position + 1)
                }
            }
            ColorSpaceStep::Done(state) => ColorSpaceStep::Done(state),
        };
    }
    /// The recognized color space: the matching keyword's state, the invalid-value default if no
    /// keyword matches, or the missing-value default when the attribute is absent. It is `None`
    /// while recognition is incomplete.
    pub fn result(
        &self,
    ) -> Option<crate::internal::attributes::input_colorspace::HtmlInputColorSpace> {
        if let ColorSpaceStep::Done(state) = self.step {
            Some(state)
        } else {
            None
        }
    }
}

/// The outcome of an exact old-versus-current value comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputValueComparison {
    equal: bool,
}
impl HtmlInputValueComparison {
    /// Whether the two values have identical length and identical UTF-16 code units.
    pub fn is_equal(self) -> bool {
        self.equal
    }
}
/// One pair per advance; exact code units, not Unicode or semantic equality.
#[derive(Debug)]
pub struct HtmlInputValueComparisonCursor<'a> {
    old: &'a [u16],
    current: &'a [u16],
    position: usize,
    result: Option<HtmlInputValueComparison>,
    length_pending: bool,
}
impl<'a> HtmlInputValueComparisonCursor<'a> {
    /// Starts comparing the `old` value with the `current` value. Both are exact UTF-16 code units;
    /// the comparison is neither Unicode nor semantic.
    pub fn new(old: &'a [u16], current: &'a [u16]) -> Self {
        Self {
            old,
            current,
            position: 0,
            result: None,
            length_pending: true,
        }
    }
    /// The fixed work the next step performs, or `None` once `result` is available.
    fn required_work(&self) -> Option<HtmlInputRuleWork> {
        if self.result.is_some() {
            None
        } else if self.length_pending {
            Some(HtmlInputRuleWork::ReadValueLengths)
        } else {
            Some(HtmlInputRuleWork::CompareValueUnits)
        }
    }
    /// Charges the next step's work as one schema change and performs it: the length check, or the
    /// comparison of one code unit pair. A refused charge leaves the cursor as it was. It charges
    /// nothing and does nothing after the result is known.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        &mut self,
        charge: &mut C,
    ) -> Result<(), C::Stop> {
        if let Some(work) = self.required_work() {
            charge.charge(&[work.change()])?;
        }
        self.step();
        Ok(())
    }
    fn step(&mut self) {
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
    /// The comparison outcome, or `None` while the comparison is incomplete.
    pub fn result(&self) -> Option<HtmlInputValueComparison> {
        self.result
    }
}

/// The direction of a text control's selection. HTML877d #set-the-selection-direction defines the
/// `none` value and the platform's choice to support it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputSelectionDirection {
    /// No direction. A platform that does not support it reads it as `Forward` (`resolve_none`).
    None,
    /// The selection's end moves forward from its start.
    Forward,
    /// The selection's start moves backward from its end.
    Backward,
}
/// Position and direction are independent source state. A cursor retains its
/// direction too; a generic value clamp does not reset that direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionPosition {
    Cursor(usize),
    Selection { start: usize, end: usize },
}
/// Actual UTF-16 selection position and independent retained direction.
///
/// This value does not choose platform support. Retained constructors preserve
/// established facts; initial and explicit direction writes consume the actual
/// none-direction support. No DOM identity, events or platform policy is stored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputSelection {
    position: SelectionPosition,
    direction: HtmlInputSelectionDirection,
}
/// The refusal of a selection whose start offset is after its end offset. Both offsets are kept as
/// UTF-16 code unit offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlInputInvalidSelectionRange {
    start: usize,
    end: usize,
}
impl HtmlInputInvalidSelectionRange {
    /// The rejected start offset, in UTF-16 code units.
    pub fn start(self) -> usize {
        self.start
    }
    /// The rejected end offset, in UTF-16 code units. It is less than the start.
    pub fn end(self) -> usize {
        self.end
    }
}
impl std::fmt::Display for HtmlInputInvalidSelectionRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "input selection start {} is after its end {}",
            self.start, self.end
        )
    }
}
impl std::error::Error for HtmlInputInvalidSelectionRange {}
/// Whether the environment supports a selection that is empty at the end of the value. When
/// unsupported, `HtmlInputSelection::clamp` turns such a selection into a cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputEmptySelectionSupport {
    /// An empty selection at the end of the value is kept as a selection.
    Supported,
    /// An empty selection at the end of the value becomes a cursor.
    Unsupported,
}
impl HtmlInputSelection {
    /// Source initial cursor zero with direction resolved from actual support.
    /// This is not a platform default or a selected headless environment.
    pub fn initial(none_support: HtmlInputNoneDirectionSupport) -> Self {
        Self::cursor(
            0,
            HtmlInputSelectionDirection::None.resolve_none(none_support),
        )
    }
    /// Retain an actual cursor and its established direction, without applying
    /// a new platform's policy. Source direction changes use `set_direction`.
    pub fn cursor(offset: usize, direction: HtmlInputSelectionDirection) -> Self {
        Self {
            position: SelectionPosition::Cursor(offset),
            direction,
        }
    }
    /// Retain an actual ordered selection and its established direction.
    /// This is not the event-bearing set-selection-range algorithm and does
    /// not establish that an empty range is supported by an environment.
    pub fn selection(
        start: usize,
        end: usize,
        direction: HtmlInputSelectionDirection,
    ) -> Result<Self, HtmlInputInvalidSelectionRange> {
        if start > end {
            Err(HtmlInputInvalidSelectionRange { start, end })
        } else {
            Ok(Self {
                position: SelectionPosition::Selection { start, end },
                direction,
            })
        }
    }
    /// Actual direction for either a cursor or a selection.
    pub fn direction(self) -> HtmlInputSelectionDirection {
        self.direction
    }
    /// Local set-the-selection-direction step, using actual none support.
    /// Call only for a source direction write; clamping/adoption do not select
    /// this step merely because the current environment's support differs.
    pub fn set_direction(
        mut self,
        requested: HtmlInputSelectionDirection,
        none_support: HtmlInputNoneDirectionSupport,
    ) -> Self {
        self.direction = requested.resolve_none(none_support);
        self
    }
    /// The cursor offset in UTF-16 code units, or `None` when this value is a selection.
    pub fn cursor_position(self) -> Option<usize> {
        if let SelectionPosition::Cursor(position) = self.position {
            Some(position)
        } else {
            None
        }
    }
    /// The start offset, end offset and direction, or `None` when this value is a cursor. Offsets
    /// are UTF-16 code units with `start <= end`.
    pub fn selection_range(self) -> Option<(usize, usize, HtmlInputSelectionDirection)> {
        if let SelectionPosition::Selection { start, end } = self.position {
            Some((start, end, self.direction))
        } else {
            None
        }
    }
    /// HTML's relevant-value change clamp; fixed work, no string read.
    /// Direction is preserved even when a range collapses into a cursor.
    pub fn clamp(
        mut self,
        actual_value_len: usize,
        support: HtmlInputEmptySelectionSupport,
    ) -> Self {
        self.position = match self.position {
            SelectionPosition::Cursor(offset) => {
                SelectionPosition::Cursor(offset.min(actual_value_len))
            }
            SelectionPosition::Selection { start, end } => {
                let start = start.min(actual_value_len);
                let end = end.min(actual_value_len);
                if support == HtmlInputEmptySelectionSupport::Unsupported
                    && start == actual_value_len
                    && end == actual_value_len
                {
                    SelectionPosition::Cursor(actual_value_len)
                } else {
                    SelectionPosition::Selection { start, end }
                }
            }
        };
        self
    }
}
/// Input's additional Value setter effect; evaluate after actual generic clamping.
/// None-direction application still requires the adapter's real platform facts.
pub fn input_value_setter_finish(
    kind: HtmlInputType,
    comparison: HtmlInputValueComparison,
    post_clamp: HtmlInputSelection,
) -> HtmlTextControlValueSetterFinish {
    if kind.is_selectable() && !comparison.is_equal() && post_clamp.cursor_position().is_some() {
        HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
    } else {
        HtmlTextControlValueSetterFinish::NoAdditionalEffect
    }
}

/// Whether the environment supports the `none` selection direction. When unsupported, `none` reads
/// as `Forward`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlInputNoneDirectionSupport {
    /// The `none` direction is kept.
    Supported,
    /// The `none` direction becomes `Forward`.
    Unsupported,
}
impl HtmlInputSelectionDirection {
    /// Applies the environment's support for `none`: `None` becomes `Forward` when unsupported, and
    /// every other direction is returned unchanged.
    pub fn resolve_none(self, support: HtmlInputNoneDirectionSupport) -> Self {
        if self == Self::None && support == HtmlInputNoneDirectionSupport::Unsupported {
            Self::Forward
        } else {
            self
        }
    }
}

/// HTML877d #radio-button-group triggers. These are source operations, not host
/// events; Connected requires an actual insertion hook and now-connected proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRadioTrigger {
    /// The checkedness of a control changed.
    Checkedness,
    /// The `name` attribute of a control changed.
    Name,
    /// The form owner of a control changed.
    FormOwner,
    /// The `type` attribute state of a control changed.
    Type,
    /// A control became connected to a document tree, after an actual insertion hook.
    Connected,
}

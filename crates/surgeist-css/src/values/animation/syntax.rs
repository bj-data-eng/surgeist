use crate::numeric::{
    CssInteger, CssNonNegative, CssNumber, CssNumeric, CssPercentage, CssPositive, CssTime,
};
use crate::rule::keyframes::{CssKeyframesIdent, CssKeyframesString};
use crate::values::common::CssCustomIdent;

/// A checked custom transition property name, retaining unknown names and case.
/// `none` and `all` use their distinct keyword variants instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransitionPropertyName(CssCustomIdent);

impl CssTransitionPropertyName {
    /// Returns the checked name, or `None` when the identifier is `none` or `all` (ASCII case-insensitive), which use [`CssTransitionProperty::None`] and [`CssTransitionProperty::All`] instead.
    #[must_use]
    pub fn try_new(value: CssCustomIdent) -> Option<Self> {
        (!value.as_str().eq_ignore_ascii_case("none")
            && !value.as_str().eq_ignore_ascii_case("all"))
        .then_some(Self(value))
    }

    /// Returns the retained authored property name, preserving unknown names and case.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A non-empty authored duration list.
///
/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDurationList {
    values: Vec<CssNumeric<CssTime, CssNonNegative>>,
}

impl CssDurationList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssNumeric<CssTime, CssNonNegative>>) -> Option<Self> {
        if values.is_empty()
            || values
                .iter()
                .any(|value| value.first_implicit_origin().is_some())
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssNumeric<CssTime, CssNonNegative>>) -> Self {
        Self { values }
    }

    /// Returns the authored non-negative durations in list order.
    #[must_use]
    pub fn values(&self) -> &[CssNumeric<CssTime, CssNonNegative>] {
        &self.values
    }
}

/// A non-empty authored delay list.
///
/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug)]
pub struct CssDelayList {
    values: Vec<CssNumeric<CssTime>>,
}

impl CssDelayList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssNumeric<CssTime>>) -> Option<Self> {
        if values.is_empty()
            || values
                .iter()
                .any(|value| value.first_implicit_origin().is_some())
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssNumeric<CssTime>>) -> Self {
        Self { values }
    }

    /// Returns the authored delays in list order.
    #[must_use]
    pub fn values(&self) -> &[CssNumeric<CssTime>] {
        &self.values
    }
}

/// A keyword-authored easing function, including the two step aliases.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssEasingKeyword {
    /// `ease`: equivalent to `cubic-bezier(0.25, 0.1, 0.25, 1)`.
    Ease,
    /// `linear`: the identity easing function, whose output progress equals its input
    /// progress; the same result as `linear(0, 1)`.
    Linear,
    /// `ease-in`: equivalent to `cubic-bezier(0.42, 0, 1, 1)`.
    EaseIn,
    /// `ease-out`: equivalent to `cubic-bezier(0, 0, 0.58, 1)`.
    EaseOut,
    /// `ease-in-out`: equivalent to `cubic-bezier(0.42, 0, 0.58, 1)`.
    EaseInOut,
    /// `step-start`: computes to `steps(1, start)`.
    StepStart,
    /// `step-end`: computes to `steps(1, end)`.
    StepEnd,
}

/// A checked cubic-bezier x coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCubicBezierX {
    value: CssNumeric<CssNumber>,
}

impl CssCubicBezierX {
    /// Returns the checked coordinate, or `None` when a literal number is outside the range 0 to 1.
    /// Calculations are retained unresolved and are not range-checked.
    #[must_use]
    pub fn try_new(value: CssNumeric<CssNumber>) -> Option<Self> {
        if let Some(component) = value.literal_component() {
            let crate::component::CssComponentValueRef::Token(
                crate::component::CssValueTokenRef::Number(number),
            ) = component.view()
            else {
                unreachable!("checked number literal")
            };
            let decimal =
                crate::serialization::exact_decimal::LexicalDecimal::new(number.representation());
            if !decimal.is_zero() && (decimal.negative || !decimal.absolute_at_most("1", 0)) {
                return None;
            }
        }
        Some(Self { value })
    }

    /// Returns the authored number or calculation.
    #[must_use]
    pub const fn value(&self) -> &CssNumeric<CssNumber> {
        &self.value
    }
}

/// A checked authored `cubic-bezier()` value.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCubicBezier {
    x1: CssCubicBezierX,
    y1: CssNumeric<CssNumber>,
    x2: CssCubicBezierX,
    y2: CssNumeric<CssNumber>,
}

impl CssCubicBezier {
    /// Returns a `cubic-bezier()` value, or `None` when either x coordinate is a literal number outside the range 0 to 1.
    /// The y coordinates are not range-limited.
    #[must_use]
    pub fn try_new(
        x1: CssNumeric<CssNumber>,
        y1: CssNumeric<CssNumber>,
        x2: CssNumeric<CssNumber>,
        y2: CssNumeric<CssNumber>,
    ) -> Option<Self> {
        Some(Self {
            x1: CssCubicBezierX::try_new(x1)?,
            y1,
            x2: CssCubicBezierX::try_new(x2)?,
            y2,
        })
    }

    /// Returns the first control point's x coordinate.
    #[must_use]
    pub const fn x1(&self) -> &CssCubicBezierX {
        &self.x1
    }

    /// Returns the first control point's y coordinate.
    #[must_use]
    pub const fn y1(&self) -> &CssNumeric<CssNumber> {
        &self.y1
    }

    /// Returns the second control point's x coordinate.
    #[must_use]
    pub const fn x2(&self) -> &CssCubicBezierX {
        &self.x2
    }

    /// Returns the second control point's y coordinate.
    #[must_use]
    pub const fn y2(&self) -> &CssNumeric<CssNumber> {
        &self.y2
    }
}

/// One authored `linear()` stop, before contextual input-position fixup.
/// Output is a Number and the contiguous input group contains zero to two Percentages.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearStop {
    output: CssNumeric<CssNumber>,
    inputs: Vec<CssNumeric<CssPercentage>>,
}

impl CssLinearStop {
    /// Preserves signed outputs, omissions, pairs, and out-of-range inputs.
    /// Rejects more than two inputs and numeric graphs retaining implicit closure.
    #[must_use]
    pub fn try_new(
        output: CssNumeric<CssNumber>,
        inputs: Vec<CssNumeric<CssPercentage>>,
    ) -> Option<Self> {
        let stop = Self::from_parser(output, inputs)?;
        stop.has_explicit_numeric_closure().then_some(stop)
    }

    pub(crate) fn from_parser(
        output: CssNumeric<CssNumber>,
        inputs: Vec<CssNumeric<CssPercentage>>,
    ) -> Option<Self> {
        (inputs.len() <= 2).then_some(Self { output, inputs })
    }

    fn has_explicit_numeric_closure(&self) -> bool {
        self.output
            .calculation()
            .is_none_or(|value| value.components().first_implicit_origin().is_none())
            && self.inputs.iter().all(|input| {
                input
                    .calculation()
                    .is_none_or(|value| value.components().first_implicit_origin().is_none())
            })
    }

    /// Returns the authored output value of the stop.
    #[must_use]
    pub const fn output(&self) -> &CssNumeric<CssNumber> {
        &self.output
    }

    /// Returns the authored input positions of the stop, in source order; the list holds zero, one or two percentages.
    #[must_use]
    pub fn inputs(&self) -> &[CssNumeric<CssPercentage>] {
        &self.inputs
    }
}

/// An ordered authored `linear()` function with at least two stops.
/// Omissions, percentage pairs, and symbolic calculations remain unresolved;
/// numeric resolution and input-position fixup belong to later phases.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLinearEasing {
    stops: Vec<CssLinearStop>,
}

impl CssLinearEasing {
    /// Rejects fewer than two authored stops, including a single percentage-pair stop,
    /// and rejects stops whose numeric graphs retain implicit closure.
    #[must_use]
    pub fn try_new(stops: Vec<CssLinearStop>) -> Option<Self> {
        let value = Self::from_parser(stops)?;
        value
            .stops
            .iter()
            .all(CssLinearStop::has_explicit_numeric_closure)
            .then_some(value)
    }

    pub(crate) fn from_parser(stops: Vec<CssLinearStop>) -> Option<Self> {
        (stops.len() >= 2).then_some(Self { stops })
    }

    /// Returns the authored stops in source order; there are always at least two.
    #[must_use]
    pub fn stops(&self) -> &[CssLinearStop] {
        &self.stops
    }
}

/// The optional authored position of a `steps()` easing function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssStepPosition {
    /// `jump-start`: the first rise occurs at input progress value 0.
    JumpStart,
    /// `jump-end`: the last rise occurs at input progress value 1.
    JumpEnd,
    /// `jump-none`: all rises occur within the range (0, 1).
    JumpNone,
    /// `jump-both`: the first rise occurs at input progress value 0 and the last at input
    /// progress value 1.
    JumpBoth,
    /// `start`: behaves as `jump-start`.
    Start,
    /// `end`: behaves as `jump-end`.
    End,
}

/// A checked authored `steps()` value.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSteps {
    count: CssNumeric<CssInteger, CssPositive>,
    position: Option<CssStepPosition>,
}

impl CssSteps {
    /// Returns a `steps()` value, or `None` when the position is `jump-none` and the step count is the literal integer 1.
    #[must_use]
    pub fn try_new(
        count: CssNumeric<CssInteger, CssPositive>,
        position: Option<CssStepPosition>,
    ) -> Option<Self> {
        if matches!(position, Some(CssStepPosition::JumpNone))
            && let Some(literal) = count.literal()
            && literal
                .compare_value(&crate::numeric::CssNumericLiteral::<
                    crate::numeric::CssInteger,
                >::from_i32(1))
                .is_eq()
        {
            None
        } else {
            Some(Self { count, position })
        }
    }

    /// Returns the authored step count.
    #[must_use]
    pub const fn count(&self) -> &CssNumeric<CssInteger, CssPositive> {
        &self.count
    }

    /// Returns the authored step position, or `None` when it was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<CssStepPosition> {
        self.position
    }
}

/// An authored easing function.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssEasing {
    /// A keyword easing function such as `ease` or `step-end`.
    Keyword(CssEasingKeyword),
    /// A `cubic-bezier()` function.
    CubicBezier(CssCubicBezier),
    /// A `linear()` function.
    Linear(CssLinearEasing),
    /// A `steps()` function.
    Steps(CssSteps),
}

/// A non-empty comma-separated list of authored easing functions.
///
/// # Specified serialization
///
/// Serializes the authored easing value using canonical specified syntax. Calculations retain
/// specified-value semantics without timing evaluation or clamping. Aggregates and their numeric
/// children share one serialization context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssEasingList {
    values: Vec<CssEasing>,
}

impl CssEasingList {
    /// Returns the list, or `None` when `values` is empty.
    #[must_use]
    pub fn try_new(values: Vec<CssEasing>) -> Option<Self> {
        (!values.is_empty()).then_some(Self { values })
    }

    /// Returns the authored easing functions in list order.
    #[must_use]
    pub fn values(&self) -> &[CssEasing] {
        &self.values
    }
}

/// One item of a `transition-property` list.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssTransitionProperty {
    /// `all`: all properties are to be transitioned.
    All,
    /// `none`: no property transitions.
    None,
    /// A property name other than `none` and `all`, retained as authored.
    Custom(CssTransitionPropertyName),
}

/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTransitionPropertyList {
    properties: Vec<CssTransitionProperty>,
}

impl CssTransitionPropertyList {
    /// Rejects an empty list or `none` mixed with other items.
    #[must_use]
    pub fn try_new(properties: Vec<CssTransitionProperty>) -> Option<Self> {
        transition_properties_are_valid(properties.iter().map(Some)).then_some(Self { properties })
    }

    /// Returns the authored items in list order.
    #[must_use]
    pub fn properties(&self) -> &[CssTransitionProperty] {
        &self.properties
    }
}

fn transition_properties_are_valid<'a>(
    mut properties: impl ExactSizeIterator<Item = Option<&'a CssTransitionProperty>>,
) -> bool {
    let count = properties.len();
    count > 0
        && (count == 1
            || !properties.any(|property| matches!(property, Some(CssTransitionProperty::None))))
}

/// An authored `animation-name` item, a `<keyframes-name>` or `none`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationName {
    /// `none`: no animation.
    None,
    /// A `<custom-ident>` keyframes name.
    Custom(CssKeyframesIdent),
    /// A `<string>` keyframes name.
    String(CssKeyframesString),
}

/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationNameList {
    names: Vec<CssAnimationName>,
}

impl CssAnimationNameList {
    /// Returns the list, or `None` when `names` is empty.
    #[must_use]
    pub fn try_new(names: Vec<CssAnimationName>) -> Option<Self> {
        if names.is_empty() {
            None
        } else {
            Some(Self::new(names))
        }
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssAnimationName>) -> Self {
        Self { names }
    }

    /// Returns the authored names in list order.
    #[must_use]
    pub fn names(&self) -> &[CssAnimationName] {
        &self.names
    }
}

/// An authored animation iteration count, retaining an exact number or symbolic math.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssAnimationIterationCount {
    /// `infinite`: the animation repeats forever.
    Infinite,
    /// A non-negative `<number>` or calculation giving the repeat count.
    Number(CssNumeric<CssNumber, CssNonNegative>),
}

impl PartialEq for CssAnimationIterationCount {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Infinite, Self::Infinite) => true,
            (Self::Number(left), Self::Number(right)) => left == right,
            _ => false,
        }
    }
}

/// A non-empty list of authored animation iteration counts.
///
/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationIterationCountList {
    values: Vec<CssAnimationIterationCount>,
}

impl CssAnimationIterationCountList {
    /// Returns the list, or `None` when `values` is empty.
    #[must_use]
    pub fn try_new(values: Vec<CssAnimationIterationCount>) -> Option<Self> {
        if values.is_empty() {
            None
        } else {
            Some(Self { values })
        }
    }

    /// Returns the authored iteration counts in list order.
    #[must_use]
    pub fn values(&self) -> &[CssAnimationIterationCount] {
        &self.values
    }
}

/// An authored `animation-direction` item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationDirection {
    /// `normal`: all iterations are played as specified.
    Normal,
    /// `reverse`: all iterations are played in the reverse direction from the way they were specified.
    Reverse,
    /// `alternate`: odd-numbered iterations (counting from 1) play in the normal direction and
    /// even-numbered iterations in the reverse direction.
    Alternate,
    /// `alternate-reverse`: odd-numbered iterations (counting from 1) play in the reverse
    /// direction and even-numbered iterations in the normal direction.
    AlternateReverse,
}

/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationDirectionList {
    directions: Vec<CssAnimationDirection>,
}

impl CssAnimationDirectionList {
    /// Returns the list, or `None` when `directions` is empty.
    #[must_use]
    pub fn try_new(directions: Vec<CssAnimationDirection>) -> Option<Self> {
        if directions.is_empty() {
            None
        } else {
            Some(Self::new(directions))
        }
    }

    #[must_use]
    pub(crate) fn new(directions: Vec<CssAnimationDirection>) -> Self {
        Self { directions }
    }

    /// Returns the authored directions in list order.
    #[must_use]
    pub fn directions(&self) -> &[CssAnimationDirection] {
        &self.directions
    }
}

/// An authored `animation-fill-mode` item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationFillMode {
    /// `none`: the animation has no effect when it is applied but not executing.
    None,
    /// `forwards`: after the animation ends, it applies the property values for the time the
    /// animation ended.
    Forwards,
    /// `backwards`: during the delay, the animation applies the property values of the keyframe
    /// that will start the first iteration.
    Backwards,
    /// `both`: the effects of both `forwards` and `backwards` fill apply.
    Both,
}

/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationFillModeList {
    modes: Vec<CssAnimationFillMode>,
}

impl CssAnimationFillModeList {
    /// Returns the list, or `None` when `modes` is empty.
    #[must_use]
    pub fn try_new(modes: Vec<CssAnimationFillMode>) -> Option<Self> {
        if modes.is_empty() {
            None
        } else {
            Some(Self::new(modes))
        }
    }

    #[must_use]
    pub(crate) fn new(modes: Vec<CssAnimationFillMode>) -> Self {
        Self { modes }
    }

    /// Returns the authored fill modes in list order.
    #[must_use]
    pub fn modes(&self) -> &[CssAnimationFillMode] {
        &self.modes
    }
}

/// An authored `animation-play-state` item.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAnimationPlayState {
    /// `running`: the animation proceeds as normal.
    Running,
    /// `paused`: the animation is paused; it keeps the progress it had made before being paused.
    Paused,
}

/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAnimationPlayStateList {
    states: Vec<CssAnimationPlayState>,
}

impl CssAnimationPlayStateList {
    /// Returns the list, or `None` when `states` is empty.
    #[must_use]
    pub fn try_new(states: Vec<CssAnimationPlayState>) -> Option<Self> {
        if states.is_empty() {
            None
        } else {
            Some(Self::new(states))
        }
    }

    #[must_use]
    pub(crate) fn new(states: Vec<CssAnimationPlayState>) -> Self {
        Self { states }
    }

    /// Returns the authored play states in list order.
    #[must_use]
    pub fn states(&self) -> &[CssAnimationPlayState] {
        &self.states
    }
}

/// An authored transition with distinct duration and delay domains.
#[derive(Clone, Debug)]
pub struct CssTransition {
    property: Option<CssTransitionProperty>,
    duration: Option<CssNumeric<CssTime, CssNonNegative>>,
    delay: Option<CssNumeric<CssTime>>,
    timing_function: Option<CssEasing>,
}

impl CssTransition {
    /// Rejects an empty item or retained recovery in its duration or delay.
    #[must_use]
    pub fn try_new(
        property: Option<CssTransitionProperty>,
        duration: Option<CssNumeric<CssTime, CssNonNegative>>,
        delay: Option<CssNumeric<CssTime>>,
        timing_function: Option<CssEasing>,
    ) -> Option<Self> {
        if duration
            .as_ref()
            .is_some_and(|v| v.first_implicit_origin().is_some())
            || delay
                .as_ref()
                .is_some_and(|v| v.first_implicit_origin().is_some())
        {
            return None;
        }
        Self::from_parser(property, duration, delay, timing_function)
    }
    pub(crate) fn from_parser(
        property: Option<CssTransitionProperty>,
        duration: Option<CssNumeric<CssTime, CssNonNegative>>,
        delay: Option<CssNumeric<CssTime>>,
        timing_function: Option<CssEasing>,
    ) -> Option<Self> {
        if property.is_none() && duration.is_none() && delay.is_none() && timing_function.is_none()
        {
            None
        } else {
            Some(Self {
                property,
                duration,
                delay,
                timing_function,
            })
        }
    }

    /// Returns the authored property, or `None` when the item omitted it.
    #[must_use]
    pub const fn property(&self) -> Option<&CssTransitionProperty> {
        self.property.as_ref()
    }

    /// Returns the authored duration, or `None` when the item omitted it.
    #[must_use]
    pub const fn duration(&self) -> Option<&CssNumeric<CssTime, CssNonNegative>> {
        self.duration.as_ref()
    }

    /// Returns the authored delay, or `None` when the item omitted it.
    #[must_use]
    pub const fn delay(&self) -> Option<&CssNumeric<CssTime>> {
        self.delay.as_ref()
    }

    /// Returns the authored timing function, or `None` when the item omitted it.
    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }
}

/// A non-empty authored transition list.
///
/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTransitionList {
    values: Vec<CssTransition>,
}

impl CssTransitionList {
    /// Rejects an empty list, mixed `none`, or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssTransition>) -> Option<Self> {
        if values.iter().any(|v| {
            v.duration()
                .is_some_and(|d| d.first_implicit_origin().is_some())
                || v.delay()
                    .is_some_and(|d| d.first_implicit_origin().is_some())
        }) {
            None
        } else {
            Self::from_parser(values)
        }
    }

    pub(crate) fn from_parser(values: Vec<CssTransition>) -> Option<Self> {
        transition_properties_are_valid(values.iter().map(CssTransition::property))
            .then_some(Self { values })
    }

    /// Returns the authored transitions in list order.
    #[must_use]
    pub fn values(&self) -> &[CssTransition] {
        &self.values
    }
}

/// An authored animation with distinct timing domains.
#[derive(Clone, Debug)]
pub struct CssAnimation {
    name: Option<CssAnimationName>,
    duration: Option<CssNumeric<CssTime, CssNonNegative>>,
    delay: Option<CssNumeric<CssTime>>,
    timing_function: Option<CssEasing>,
    iteration_count: Option<CssAnimationIterationCount>,
    direction: Option<CssAnimationDirection>,
    fill_mode: Option<CssAnimationFillMode>,
    play_state: Option<CssAnimationPlayState>,
}

/// Checked input components for one authored animation.
#[derive(Clone, Debug, Default)]
pub struct CssAnimationComponents {
    /// The authored `<keyframes-name>` or `none`, or `None` when omitted.
    pub name: Option<CssAnimationName>,
    /// The authored non-negative duration, or `None` when omitted.
    pub duration: Option<CssNumeric<CssTime, CssNonNegative>>,
    /// The authored delay, or `None` when omitted.
    pub delay: Option<CssNumeric<CssTime>>,
    /// The authored easing function, or `None` when omitted.
    pub timing_function: Option<CssEasing>,
    /// The authored iteration count, or `None` when omitted.
    pub iteration_count: Option<CssAnimationIterationCount>,
    /// The authored direction, or `None` when omitted.
    pub direction: Option<CssAnimationDirection>,
    /// The authored fill mode, or `None` when omitted.
    pub fill_mode: Option<CssAnimationFillMode>,
    /// The authored play state, or `None` when omitted.
    pub play_state: Option<CssAnimationPlayState>,
}

impl CssAnimation {
    /// Rejects an empty item or retained recovery in its duration or delay.
    #[must_use]
    pub fn try_new(components: CssAnimationComponents) -> Option<Self> {
        if components
            .duration
            .as_ref()
            .is_some_and(|v| v.first_implicit_origin().is_some())
            || components
                .delay
                .as_ref()
                .is_some_and(|v| v.first_implicit_origin().is_some())
        {
            return None;
        }
        Self::from_parser(components)
    }
    pub(crate) fn from_parser(components: CssAnimationComponents) -> Option<Self> {
        if components.name.is_none()
            && components.duration.is_none()
            && components.delay.is_none()
            && components.timing_function.is_none()
            && components.iteration_count.is_none()
            && components.direction.is_none()
            && components.fill_mode.is_none()
            && components.play_state.is_none()
        {
            None
        } else {
            Some(Self {
                name: components.name,
                duration: components.duration,
                delay: components.delay,
                timing_function: components.timing_function,
                iteration_count: components.iteration_count,
                direction: components.direction,
                fill_mode: components.fill_mode,
                play_state: components.play_state,
            })
        }
    }

    /// Returns the authored name, or `None` when the item omitted it.
    #[must_use]
    pub const fn name(&self) -> Option<&CssAnimationName> {
        self.name.as_ref()
    }

    /// Returns the authored duration, or `None` when the item omitted it.
    #[must_use]
    pub const fn duration(&self) -> Option<&CssNumeric<CssTime, CssNonNegative>> {
        self.duration.as_ref()
    }

    /// Returns the authored delay, or `None` when the item omitted it.
    #[must_use]
    pub const fn delay(&self) -> Option<&CssNumeric<CssTime>> {
        self.delay.as_ref()
    }

    /// Returns the authored timing function, or `None` when the item omitted it.
    #[must_use]
    pub const fn timing_function(&self) -> Option<&CssEasing> {
        self.timing_function.as_ref()
    }

    /// Returns the authored iteration count, or `None` when the item omitted it.
    #[must_use]
    pub const fn iteration_count(&self) -> Option<&CssAnimationIterationCount> {
        self.iteration_count.as_ref()
    }

    /// Returns the authored direction, or `None` when the item omitted it.
    #[must_use]
    pub const fn direction(&self) -> Option<CssAnimationDirection> {
        self.direction
    }

    /// Returns the authored fill mode, or `None` when the item omitted it.
    #[must_use]
    pub const fn fill_mode(&self) -> Option<CssAnimationFillMode> {
        self.fill_mode
    }

    /// Returns the authored play state, or `None` when the item omitted it.
    #[must_use]
    pub const fn play_state(&self) -> Option<CssAnimationPlayState> {
        self.play_state
    }
}

/// A non-empty authored animation list.
///
/// # Specified serialization
///
/// Emits canonical authored values in retained list order. This performs no timing evaluation or
/// contextual list expansion. Numeric children retain their shared domain serialization policy.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAnimationList {
    values: Vec<CssAnimation>,
}

impl CssAnimationList {
    /// Rejects an empty list or retained recovery in any time child.
    #[must_use]
    pub fn try_new(values: Vec<CssAnimation>) -> Option<Self> {
        if values.is_empty()
            || values.iter().any(|v| {
                v.duration()
                    .is_some_and(|d| d.first_implicit_origin().is_some())
                    || v.delay()
                        .is_some_and(|d| d.first_implicit_origin().is_some())
            })
        {
            None
        } else {
            Some(Self { values })
        }
    }

    pub(crate) fn from_parser(values: Vec<CssAnimation>) -> Self {
        Self { values }
    }

    /// Returns the authored animations in list order.
    #[must_use]
    pub fn values(&self) -> &[CssAnimation] {
        &self.values
    }
}

impl PartialEq for CssDelayList {
    fn eq(&self, other: &Self) -> bool {
        self.values.len() == other.values.len()
            && self.values.iter().zip(&other.values).all(|(a, b)| a == b)
    }
}
impl PartialEq for CssTransition {
    fn eq(&self, other: &Self) -> bool {
        self.property == other.property
            && self.duration == other.duration
            && self.delay == other.delay
            && self.timing_function == other.timing_function
    }
}
impl PartialEq for CssAnimation {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.duration == other.duration
            && self.timing_function == other.timing_function
            && self.iteration_count == other.iteration_count
            && self.direction == other.direction
            && self.fill_mode == other.fill_mode
            && self.play_state == other.play_state
            && self.delay == other.delay
    }
}
impl PartialEq for CssAnimationComponents {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.duration == other.duration
            && self.timing_function == other.timing_function
            && self.iteration_count == other.iteration_count
            && self.direction == other.direction
            && self.fill_mode == other.fill_mode
            && self.play_state == other.play_state
            && self.delay == other.delay
    }
}

//! CSS easing contracts.
//!
//! Easing supports linear, cubic-bezier, steps, and CSS `linear()` functions.
//! Piecewise linear easing stores canonical finite control points and supports
//! unrestricted finite evaluation for overshoot and extrapolation.

use core::fmt;

use crate::{EasingInput, NormalizedProgress, NumericError, UnitRatio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingKeyword {
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Easing {
    Linear,
    CubicBezier(CubicBezier),
    Steps(Steps),
    LinearFunction(LinearEasing),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EasedProgress {
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearControlPoint {
    input: f64,
    output: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinearControlPointComponent {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearEasing {
    points: Box<[LinearControlPoint]>,
    normalized_points: Box<[LinearControlPoint]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Steps {
    count: u32,
    position: StepPosition,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingError {
    Numeric(NumericError),
    NonFiniteEasingInput {
        value: f64,
    },
    NonFiniteControlPoint {
        name: &'static str,
        value: f64,
    },
    EmptyLinearFunction,
    NonFiniteLinearControlPoint {
        index: usize,
        component: LinearControlPointComponent,
        value: f64,
    },
    DecreasingLinearControlPointInput {
        index: usize,
        previous_input: f64,
        input: f64,
    },
    NonFiniteNormalizedOutput {
        input: NormalizedProgress,
        value: f64,
    },
    NonFiniteEasedProgress {
        input: EasingInput,
        value: f64,
    },
    InvalidStepCount {
        count: u32,
        position: StepPosition,
    },
}

impl fmt::Display for EasingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Numeric(error) => write!(f, "numeric easing input is invalid: {error}"),
            Self::NonFiniteEasingInput { value } => {
                write!(f, "easing input must be finite, got {value}")
            }
            Self::NonFiniteControlPoint { name, value } => {
                write!(
                    f,
                    "cubic-bezier control point {name} must be finite, got {value}"
                )
            }
            Self::EmptyLinearFunction => {
                f.write_str("linear easing must contain at least one control point")
            }
            Self::NonFiniteLinearControlPoint {
                index,
                component,
                value,
            } => write!(
                f,
                "linear control point {index} {} must be finite, got {value}",
                linear_control_point_component_name(component)
            ),
            Self::DecreasingLinearControlPointInput {
                index,
                previous_input,
                input,
            } => write!(
                f,
                "linear control point {index} input {input} must be greater than or equal to previous input {previous_input}"
            ),
            Self::NonFiniteNormalizedOutput { input, value } => write!(
                f,
                "linear easing normalized output at progress {} must be finite, got {value}",
                input.value()
            ),
            Self::NonFiniteEasedProgress { input, value } => write!(
                f,
                "easing output at input {} must be finite, got {value}",
                input.value()
            ),
            Self::InvalidStepCount { count, position } => write!(
                f,
                "steps count {count} is invalid for {}",
                step_position_name(position)
            ),
        }
    }
}

impl std::error::Error for EasingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Numeric(error) => Some(error),
            _ => None,
        }
    }
}

fn linear_control_point_component_name(component: LinearControlPointComponent) -> &'static str {
    match component {
        LinearControlPointComponent::Input => "input",
        LinearControlPointComponent::Output => "output",
    }
}

fn step_position_name(position: StepPosition) -> &'static str {
    match position {
        StepPosition::JumpStart => "jump-start",
        StepPosition::JumpEnd => "jump-end",
        StepPosition::JumpNone => "jump-none",
        StepPosition::JumpBoth => "jump-both",
    }
}

impl Easing {
    pub const fn linear() -> Self {
        Self::Linear
    }

    pub fn keyword(keyword: EasingKeyword) -> Self {
        match keyword {
            EasingKeyword::Ease => Self::CubicBezier(CubicBezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.25,
                y2: 1.0,
            }),
            EasingKeyword::EaseIn => Self::CubicBezier(CubicBezier {
                x1: 0.42,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            }),
            EasingKeyword::EaseOut => Self::CubicBezier(CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            }),
            EasingKeyword::EaseInOut => Self::CubicBezier(CubicBezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            }),
        }
    }

    pub const fn cubic_bezier(curve: CubicBezier) -> Self {
        Self::CubicBezier(curve)
    }

    pub const fn steps(steps: Steps) -> Self {
        Self::Steps(steps)
    }

    pub const fn linear_function(linear: LinearEasing) -> Self {
        Self::LinearFunction(linear)
    }

    pub fn step_start() -> Self {
        Self::Steps(Steps {
            count: 1,
            position: StepPosition::JumpStart,
        })
    }

    pub fn step_end() -> Self {
        Self::Steps(Steps {
            count: 1,
            position: StepPosition::JumpEnd,
        })
    }

    pub fn evaluate(&self, progress: NormalizedProgress) -> EasedProgress {
        self.evaluate_with_before_flag(progress, false)
    }

    pub fn evaluate_with_before_flag(
        &self,
        progress: NormalizedProgress,
        before_flag: bool,
    ) -> EasedProgress {
        match self {
            Self::Linear => EasedProgress::from_normalized(progress),
            Self::CubicBezier(curve) => curve.evaluate(progress),
            Self::Steps(steps) => steps.evaluate_with_before_flag(progress, before_flag),
            Self::LinearFunction(linear) => linear.evaluate_with_before_flag(progress, before_flag),
        }
    }

    pub fn evaluate_unrestricted(&self, input: EasingInput) -> Result<EasedProgress, EasingError> {
        self.evaluate_unrestricted_with_before_flag(input, false)
    }

    pub fn evaluate_unrestricted_with_before_flag(
        &self,
        input: EasingInput,
        before_flag: bool,
    ) -> Result<EasedProgress, EasingError> {
        match self {
            Self::Linear => EasedProgress::new(input, input.value()),
            Self::CubicBezier(curve) => curve.evaluate_unrestricted(input),
            Self::Steps(steps) => steps.evaluate_unrestricted_with_before_flag(input, before_flag),
            Self::LinearFunction(linear) => {
                linear.evaluate_unrestricted_with_before_flag(input, before_flag)
            }
        }
    }

    pub const fn as_cubic_bezier(&self) -> Option<CubicBezier> {
        match self {
            Self::Linear => None,
            Self::CubicBezier(curve) => Some(*curve),
            Self::Steps(_) => None,
            Self::LinearFunction(_) => None,
        }
    }

    pub const fn as_linear_function(&self) -> Option<&LinearEasing> {
        match self {
            Self::Linear => None,
            Self::CubicBezier(_) => None,
            Self::Steps(_) => None,
            Self::LinearFunction(linear) => Some(linear),
        }
    }
}

impl EasedProgress {
    pub fn new(input: EasingInput, value: f64) -> Result<Self, EasingError> {
        if !value.is_finite() {
            return Err(EasingError::NonFiniteEasedProgress { input, value });
        }

        Ok(Self { value })
    }

    pub const fn from_normalized(progress: NormalizedProgress) -> Self {
        Self {
            value: progress.value(),
        }
    }

    pub const fn value(self) -> f64 {
        self.value
    }
}

impl LinearControlPoint {
    pub const fn new(input: f64, output: f64) -> Self {
        Self { input, output }
    }

    pub const fn input(self) -> f64 {
        self.input
    }

    pub const fn output(self) -> f64 {
        self.output
    }
}

impl LinearEasing {
    pub fn new(points: Vec<LinearControlPoint>) -> Result<Self, EasingError> {
        if points.is_empty() {
            return Err(EasingError::EmptyLinearFunction);
        }

        validate_linear_points(&points)?;

        let boxed_points = points.into_boxed_slice();
        let normalized_points = normalized_linear_projection(&boxed_points)?;

        Ok(Self {
            points: boxed_points,
            normalized_points,
        })
    }

    pub fn points(&self) -> &[LinearControlPoint] {
        &self.points
    }

    pub fn evaluate(&self, progress: NormalizedProgress) -> EasedProgress {
        self.evaluate_with_before_flag(progress, false)
    }

    pub fn evaluate_with_before_flag(
        &self,
        progress: NormalizedProgress,
        before_flag: bool,
    ) -> EasedProgress {
        let value = evaluate_linear_points(
            &self.normalized_points,
            progress.value(),
            before_flag,
            self.points[0].input(),
        );
        debug_assert!(value.is_finite());

        EasedProgress { value }
    }

    pub fn evaluate_unrestricted(&self, input: EasingInput) -> Result<EasedProgress, EasingError> {
        self.evaluate_unrestricted_with_before_flag(input, false)
    }

    pub fn evaluate_unrestricted_with_before_flag(
        &self,
        input: EasingInput,
        before_flag: bool,
    ) -> Result<EasedProgress, EasingError> {
        let value = evaluate_linear_points(
            &self.points,
            input.value(),
            before_flag,
            self.points[0].input(),
        );

        EasedProgress::new(input, value)
    }
}

impl CubicBezier {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<Self, EasingError> {
        let x1 = UnitRatio::new(x1).map_err(EasingError::Numeric)?.value();
        let x2 = UnitRatio::new(x2).map_err(EasingError::Numeric)?.value();

        validate_finite_control_point("y1", y1)?;
        validate_finite_control_point("y2", y2)?;

        Ok(Self { x1, y1, x2, y2 })
    }

    pub const fn control_points(self) -> (f64, f64, f64, f64) {
        (self.x1, self.y1, self.x2, self.y2)
    }

    pub fn evaluate(&self, progress: NormalizedProgress) -> EasedProgress {
        let progress_value = progress.value();

        if progress_value == 0.0 {
            return EasedProgress::from_normalized(progress);
        }

        if progress_value == 1.0 {
            return EasedProgress::from_normalized(progress);
        }

        let t = self.solve_t_for_x(progress_value);
        let value = cubic_output_axis(t, self.y1, self.y2);
        debug_assert!(value.is_finite());

        EasedProgress { value }
    }

    pub fn evaluate_unrestricted(&self, input: EasingInput) -> Result<EasedProgress, EasingError> {
        let value = if input.value() < 0.0 {
            self.start_tangent_output(input.value())
        } else if input.value() > 1.0 {
            self.end_tangent_output(input.value())
        } else {
            self.evaluate(NormalizedProgress::new(input.value()).map_err(EasingError::Numeric)?)
                .value()
        };

        EasedProgress::new(input, value)
    }

    fn solve_t_for_x(&self, x: f64) -> f64 {
        if let Some(t) = self.solve_t_with_newton(x) {
            return t;
        }

        // Retain the original full-interval search when Newton cannot certify
        // the same parameter precision, including curves with flat derivatives.
        let mut lower = 0.0;
        let mut upper = 1.0;

        for _ in 0..32 {
            let midpoint = (lower + upper) * 0.5;
            let midpoint_x = cubic_axis(midpoint, self.x1, self.x2);

            if midpoint_x < x {
                lower = midpoint;
            } else {
                upper = midpoint;
            }
        }

        (lower + upper) * 0.5
    }

    fn solve_t_with_newton(&self, x: f64) -> Option<f64> {
        const MAX_T_WIDTH: f64 = 1.0 / 4_294_967_296.0;
        const MIN_DERIVATIVE: f64 = 1.0e-6;

        let mut lower = 0.0;
        let mut upper = 1.0;
        let mut t = x;

        for _ in 0..8 {
            let current_x = cubic_axis(t, self.x1, self.x2);
            let derivative = cubic_axis_derivative(t, self.x1, self.x2);
            if !current_x.is_finite() || !derivative.is_finite() || derivative < MIN_DERIVATIVE {
                return None;
            }

            let residual = current_x - x;
            if residual.abs() <= derivative * MAX_T_WIDTH {
                let candidate_lower = (t - MAX_T_WIDTH * 0.5).max(0.0);
                let candidate_upper = (t + MAX_T_WIDTH * 0.5).min(1.0);
                // A small x residual alone is not a bound on t error. Accept
                // only a narrow interval certified with the existing x evaluator.
                if candidate_upper - candidate_lower <= MAX_T_WIDTH
                    && cubic_axis(candidate_lower, self.x1, self.x2) <= x
                    && cubic_axis(candidate_upper, self.x1, self.x2) >= x
                {
                    return Some((candidate_lower + candidate_upper) * 0.5);
                }
            }

            if current_x < x {
                lower = t;
            } else {
                upper = t;
            }
            let next_t = t - residual / derivative;
            t = if next_t.is_finite() && next_t > lower && next_t < upper {
                next_t
            } else {
                (lower + upper) * 0.5
            };
        }

        None
    }

    fn start_tangent_output(&self, input: f64) -> f64 {
        if self.x1 > 0.0 {
            input * (self.y1 / self.x1)
        } else if self.x2 > 0.0 {
            input * (self.y2 / self.x2)
        } else {
            0.0
        }
    }

    fn end_tangent_output(&self, input: f64) -> f64 {
        if self.x2 < 1.0 {
            1.0 + ((input - 1.0) * ((1.0 - self.y2) / (1.0 - self.x2)))
        } else if self.x1 < 1.0 {
            1.0 + ((input - 1.0) * ((1.0 - self.y1) / (1.0 - self.x1)))
        } else {
            1.0
        }
    }
}

impl Steps {
    pub fn new(count: u32, position: StepPosition) -> Result<Self, EasingError> {
        match position {
            StepPosition::JumpStart | StepPosition::JumpEnd | StepPosition::JumpBoth
                if count == 0 =>
            {
                Err(EasingError::InvalidStepCount { count, position })
            }
            StepPosition::JumpNone if count < 2 => {
                Err(EasingError::InvalidStepCount { count, position })
            }
            _ => Ok(Self { count, position }),
        }
    }

    pub const fn count(self) -> u32 {
        self.count
    }

    pub const fn position(self) -> StepPosition {
        self.position
    }

    pub fn evaluate(&self, progress: NormalizedProgress) -> EasedProgress {
        self.evaluate_with_before_flag(progress, false)
    }

    pub fn evaluate_with_before_flag(
        &self,
        progress: NormalizedProgress,
        before_flag: bool,
    ) -> EasedProgress {
        let value = self.output_for_input(progress.value(), before_flag);
        debug_assert!(value.is_finite());

        EasedProgress { value }
    }

    pub fn evaluate_unrestricted(&self, input: EasingInput) -> Result<EasedProgress, EasingError> {
        self.evaluate_unrestricted_with_before_flag(input, false)
    }

    pub fn evaluate_unrestricted_with_before_flag(
        &self,
        input: EasingInput,
        before_flag: bool,
    ) -> Result<EasedProgress, EasingError> {
        EasedProgress::new(input, self.output_for_input(input.value(), before_flag))
    }

    fn output_for_input(&self, input: f64, before_flag: bool) -> f64 {
        let count = f64::from(self.count);
        let jumps = self.jump_count();
        let scaled_input = input * count;
        let mut current_step = scaled_input.floor();

        if matches!(
            self.position,
            StepPosition::JumpStart | StepPosition::JumpBoth
        ) {
            current_step += 1.0;
        }

        if before_flag && scaled_input.fract() == 0.0 {
            current_step -= 1.0;
        }

        if input >= 0.0 && current_step < 0.0 {
            current_step = 0.0;
        }

        if input <= 1.0 && current_step > jumps {
            current_step = jumps;
        }

        current_step / jumps
    }

    fn jump_count(&self) -> f64 {
        match self.position {
            StepPosition::JumpStart | StepPosition::JumpEnd => f64::from(self.count),
            StepPosition::JumpNone => f64::from(self.count) - 1.0,
            StepPosition::JumpBoth => f64::from(self.count) + 1.0,
        }
    }
}

fn validate_finite_control_point(name: &'static str, value: f64) -> Result<(), EasingError> {
    if !value.is_finite() {
        return Err(EasingError::NonFiniteControlPoint { name, value });
    }

    Ok(())
}

fn validate_linear_points(points: &[LinearControlPoint]) -> Result<(), EasingError> {
    let mut previous_input = None;

    for (index, point) in points.iter().copied().enumerate() {
        if !point.input().is_finite() {
            return Err(EasingError::NonFiniteLinearControlPoint {
                index,
                component: LinearControlPointComponent::Input,
                value: point.input(),
            });
        }

        if !point.output().is_finite() {
            return Err(EasingError::NonFiniteLinearControlPoint {
                index,
                component: LinearControlPointComponent::Output,
                value: point.output(),
            });
        }

        if let Some(previous_input) = previous_input
            && point.input() < previous_input
        {
            return Err(EasingError::DecreasingLinearControlPointInput {
                index,
                previous_input,
                input: point.input(),
            });
        }

        previous_input = Some(point.input());
    }

    Ok(())
}

fn normalized_linear_projection(
    points: &[LinearControlPoint],
) -> Result<Box<[LinearControlPoint]>, EasingError> {
    let zero = NormalizedProgress::new(0.0).map_err(EasingError::Numeric)?;
    let one = NormalizedProgress::new(1.0).map_err(EasingError::Numeric)?;
    let zero_output = finite_normalized_linear_output(points, zero)?;
    let one_output = finite_normalized_linear_output(points, one)?;
    let mut normalized = Vec::new();

    if points.iter().all(|point| point.input() != 0.0) {
        normalized.push(LinearControlPoint::new(0.0, zero_output));
    }

    normalized.extend(
        points
            .iter()
            .copied()
            .filter(|point| (0.0..=1.0).contains(&point.input())),
    );

    if points.iter().all(|point| point.input() != 1.0) {
        normalized.push(LinearControlPoint::new(1.0, one_output));
    }

    Ok(normalized.into_boxed_slice())
}

fn finite_normalized_linear_output(
    points: &[LinearControlPoint],
    input: NormalizedProgress,
) -> Result<f64, EasingError> {
    let value = evaluate_linear_points(points, input.value(), false, points[0].input());

    if value.is_finite() {
        Ok(value)
    } else {
        Err(EasingError::NonFiniteNormalizedOutput { input, value })
    }
}

fn evaluate_linear_points(
    points: &[LinearControlPoint],
    input: f64,
    before_flag: bool,
    first_control_input: f64,
) -> f64 {
    if points.len() == 1 {
        return points[0].output();
    }

    // Construction validates finite, nondecreasing inputs. Bounds preserve the
    // first/last duplicate distinction without scanning the control points.
    let upper_index = points.partition_point(|point| point.input() < input);
    if upper_index < points.len() && points[upper_index].input() == input {
        let selected = if before_flag && input == first_control_input {
            upper_index
        } else {
            points.partition_point(|point| point.input() <= input) - 1
        };
        return points[selected].output();
    }

    if upper_index == 0 {
        let (start, end) = first_usable_linear_pair(points);
        return affine_linear_output(start, end, input);
    }

    if upper_index == points.len() {
        let (start, end) = last_usable_linear_pair(points);
        return affine_linear_output(start, end, input);
    }

    affine_linear_output(points[upper_index - 1], points[upper_index], input)
}

fn first_usable_linear_pair(
    points: &[LinearControlPoint],
) -> (LinearControlPoint, LinearControlPoint) {
    let start = points[0];
    let first_distinct = points.partition_point(|point| point.input() == start.input());
    (start, points.get(first_distinct).copied().unwrap_or(start))
}

fn last_usable_linear_pair(
    points: &[LinearControlPoint],
) -> (LinearControlPoint, LinearControlPoint) {
    let end = points[points.len() - 1];
    let first_equal = points.partition_point(|point| point.input() < end.input());
    let start = first_equal
        .checked_sub(1)
        .map_or(end, |index| points[index]);
    (start, end)
}

fn affine_linear_output(start: LinearControlPoint, end: LinearControlPoint, input: f64) -> f64 {
    if start.input() == end.input() {
        return end.output();
    }

    let interpolation = linear_interpolation(start.input(), end.input(), input);

    if (0.0..=1.0).contains(&interpolation) {
        finite_lerp(start.output(), end.output(), interpolation)
    } else {
        start.output() + ((end.output() - start.output()) * interpolation)
    }
}

fn linear_interpolation(start: f64, end: f64, input: f64) -> f64 {
    let numerator = input - start;
    let denominator = end - start;

    if numerator.is_finite() && denominator.is_finite() {
        return numerator / denominator;
    }

    let scale = start.abs().max(end.abs()).max(input.abs());

    if scale > 0.0 && scale.is_finite() {
        ((input / scale) - (start / scale)) / ((end / scale) - (start / scale))
    } else {
        numerator / denominator
    }
}

fn cubic_output_axis(t: f64, control_1: f64, control_2: f64) -> f64 {
    let p0 = finite_lerp(0.0, control_1, t);
    let p1 = finite_lerp(control_1, control_2, t);
    let p2 = finite_lerp(control_2, 1.0, t);
    let p3 = finite_lerp(p0, p1, t);
    let p4 = finite_lerp(p1, p2, t);

    finite_lerp(p3, p4, t)
}

fn finite_lerp(start: f64, end: f64, interpolation: f64) -> f64 {
    if interpolation == 0.0 {
        return start;
    }

    if interpolation == 1.0 {
        return end;
    }

    if start == end {
        return start;
    }

    if start.is_sign_positive() == end.is_sign_positive() {
        start + ((end - start) * interpolation)
    } else {
        (start * (1.0 - interpolation)) + (end * interpolation)
    }
}

fn cubic_axis(t: f64, control_1: f64, control_2: f64) -> f64 {
    let inverse_t = 1.0 - t;
    (3.0 * inverse_t * inverse_t * t * control_1)
        + (3.0 * inverse_t * t * t * control_2)
        + (t * t * t)
}

fn cubic_axis_derivative(t: f64, control_1: f64, control_2: f64) -> f64 {
    let inverse_t = 1.0 - t;
    (3.0 * inverse_t * inverse_t * control_1)
        + (6.0 * inverse_t * t * (control_2 - control_1))
        + (3.0 * t * t * (1.0 - control_2))
}

#[cfg(test)]
mod tests {
    use super::{
        CubicBezier, EasedProgress, Easing, EasingError, EasingKeyword, LinearControlPoint,
        LinearControlPointComponent, LinearEasing, StepPosition, Steps,
    };
    use crate::{EasingInput, NormalizedProgress, NumericError, NumericInput};

    #[test]
    fn linear_easing_is_identity() {
        let easing = Easing::linear();

        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.5).unwrap())
                .value(),
            0.5
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            1.0
        );
    }

    #[test]
    fn keyword_easing_maps_to_css_cubic_bezier_control_points() {
        assert_eq!(
            Easing::keyword(EasingKeyword::Ease).as_cubic_bezier(),
            Some(CubicBezier::new(0.25, 0.1, 0.25, 1.0).unwrap())
        );
        assert_eq!(
            Easing::keyword(EasingKeyword::EaseIn).as_cubic_bezier(),
            Some(CubicBezier::new(0.42, 0.0, 1.0, 1.0).unwrap())
        );
        assert_eq!(
            Easing::keyword(EasingKeyword::EaseOut).as_cubic_bezier(),
            Some(CubicBezier::new(0.0, 0.0, 0.58, 1.0).unwrap())
        );
        assert_eq!(
            Easing::keyword(EasingKeyword::EaseInOut).as_cubic_bezier(),
            Some(CubicBezier::new(0.42, 0.0, 0.58, 1.0).unwrap())
        );
    }

    #[test]
    fn cubic_bezier_rejects_non_finite_and_out_of_range_x_control_points() {
        assert_eq!(
            CubicBezier::new(f64::NAN, 0.0, 1.0, 1.0),
            Err(EasingError::Numeric(NumericError::non_finite(
                NumericInput::Ratio
            )))
        );
        assert_eq!(
            CubicBezier::new(-0.01, 0.0, 1.0, 1.0),
            Err(EasingError::Numeric(NumericError::out_of_range(
                NumericInput::Ratio,
                0.0,
                1.0,
                -0.01
            )))
        );
        assert_eq!(
            CubicBezier::new(0.0, 0.0, 1.01, 1.0),
            Err(EasingError::Numeric(NumericError::out_of_range(
                NumericInput::Ratio,
                0.0,
                1.0,
                1.01
            )))
        );
    }

    #[test]
    fn cubic_bezier_rejects_non_finite_y_control_points() {
        assert_eq!(
            CubicBezier::new(0.0, f64::INFINITY, 1.0, 1.0),
            Err(EasingError::NonFiniteControlPoint {
                name: "y1",
                value: f64::INFINITY
            })
        );
        assert_eq!(
            CubicBezier::new(0.0, 0.0, 1.0, f64::NEG_INFINITY),
            Err(EasingError::NonFiniteControlPoint {
                name: "y2",
                value: f64::NEG_INFINITY
            })
        );
    }

    #[test]
    fn eased_progress_rejects_non_finite_values() {
        let input = EasingInput::new(0.5).unwrap();

        assert_eq!(
            EasedProgress::new(input, f64::INFINITY),
            Err(EasingError::NonFiniteEasedProgress {
                input,
                value: f64::INFINITY
            })
        );
    }

    #[test]
    fn linear_function_rejects_empty_non_finite_and_decreasing_points() {
        assert_eq!(
            LinearEasing::new(Vec::new()),
            Err(EasingError::EmptyLinearFunction)
        );
        assert!(matches!(
            LinearEasing::new(vec![linear_point(f64::NAN, 0.0)]),
            Err(EasingError::NonFiniteLinearControlPoint {
                index: 0,
                component: LinearControlPointComponent::Input,
                value
            }) if value.is_nan()
        ));
        assert_eq!(
            LinearEasing::new(vec![linear_point(0.0, f64::INFINITY)]),
            Err(EasingError::NonFiniteLinearControlPoint {
                index: 0,
                component: LinearControlPointComponent::Output,
                value: f64::INFINITY
            })
        );
        assert_eq!(
            LinearEasing::new(vec![linear_point(0.25, 0.0), linear_point(0.0, 1.0)]),
            Err(EasingError::DecreasingLinearControlPointInput {
                index: 1,
                previous_input: 0.25,
                input: 0.0
            })
        );
        assert_eq!(
            EasingInput::new(f64::INFINITY),
            Err(EasingError::NonFiniteEasingInput {
                value: f64::INFINITY
            })
        );
    }

    #[test]
    fn linear_function_rejects_non_finite_normalized_boundary_output() {
        let error = LinearEasing::new(vec![linear_point(1.0, f64::MAX), linear_point(2.0, 0.0)])
            .unwrap_err();

        assert!(matches!(
            error,
            EasingError::NonFiniteNormalizedOutput { input, value }
                if input == NormalizedProgress::new(0.0).unwrap() && value.is_infinite()
        ));
    }

    #[test]
    fn linear_function_single_point_is_constant() {
        let easing = LinearEasing::new(vec![linear_point(0.25, 0.75)]).unwrap();

        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(-10.0).unwrap())
                .unwrap()
                .value(),
            0.75
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.5).unwrap())
                .value(),
            0.75
        );
        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(10.0).unwrap())
                .unwrap()
                .value(),
            0.75
        );
    }

    #[test]
    fn linear_function_interpolates_canonical_points() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, 0.0),
            linear_point(0.5, 1.0),
            linear_point(1.0, 0.0),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            0.5
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.75).unwrap())
                .value(),
            0.5
        );
    }

    #[test]
    fn linear_function_many_knots_preserve_alternating_slopes_and_exact_values() {
        let points = (0..=128)
            .map(|index| linear_point(f64::from(index) / 128.0, f64::from(index % 2)))
            .collect();
        let easing = LinearEasing::new(points).unwrap();

        // Each interval independently rises 0 -> 1 or falls 1 -> 0.
        for index in 0..=128 {
            let input = f64::from(index) / 128.0;
            let expected = f64::from(index % 2);
            assert_eq!(
                easing
                    .evaluate(NormalizedProgress::new(input).unwrap())
                    .value(),
                expected
            );
            assert_eq!(
                easing
                    .evaluate_unrestricted(EasingInput::new(input).unwrap())
                    .unwrap()
                    .value(),
                expected
            );
            if index < 128 {
                let input = input + 1.0 / 512.0;
                let expected = if index % 2 == 0 { 0.25 } else { 0.75 };
                assert_eq!(
                    easing
                        .evaluate(NormalizedProgress::new(input).unwrap())
                        .value(),
                    expected
                );
                assert_eq!(
                    easing
                        .evaluate_unrestricted(EasingInput::new(input).unwrap())
                        .unwrap()
                        .value(),
                    expected
                );
            }
        }
    }

    #[test]
    fn linear_function_duplicate_runs_preserve_signed_zero_and_outer_slopes() {
        let easing = LinearEasing::new(vec![
            linear_point(-0.0, 2.0),
            linear_point(0.0, 4.0),
            linear_point(0.5, 6.0),
            linear_point(0.5, 8.0),
            linear_point(1.0, 10.0),
            linear_point(1.0, 12.0),
        ])
        .unwrap();

        for input in [-0.0, 0.0] {
            for (before_flag, expected) in [(false, 4.0), (true, 2.0)] {
                assert_eq!(
                    easing
                        .evaluate_with_before_flag(
                            NormalizedProgress::new(input).unwrap(),
                            before_flag,
                        )
                        .value(),
                    expected
                );
                assert_eq!(
                    easing
                        .evaluate_unrestricted_with_before_flag(
                            EasingInput::new(input).unwrap(),
                            before_flag,
                        )
                        .unwrap()
                        .value(),
                    expected
                );
            }
        }

        for before_flag in [false, true] {
            for (input, expected) in [(0.25, 5.0), (0.5, 8.0), (0.75, 9.0), (1.0, 12.0)] {
                assert_eq!(
                    easing
                        .evaluate_with_before_flag(
                            NormalizedProgress::new(input).unwrap(),
                            before_flag,
                        )
                        .value(),
                    expected
                );
            }
            // Extrapolation uses the first endpoint below and the last endpoint above.
            for (input, expected) in [(-0.5, -2.0), (1.5, 16.0)] {
                assert_eq!(
                    easing
                        .evaluate_unrestricted_with_before_flag(
                            EasingInput::new(input).unwrap(),
                            before_flag,
                        )
                        .unwrap()
                        .value(),
                    expected
                );
            }
        }
    }

    #[test]
    fn linear_function_projected_boundary_keeps_original_before_flag_origin() {
        let easing = LinearEasing::new(vec![
            linear_point(0.25, 0.25),
            linear_point(0.25, 0.5),
            linear_point(0.75, 1.25),
        ])
        .unwrap();

        for (before_flag, original_first_output) in [(false, 0.5), (true, 0.25)] {
            for (input, expected) in [(0.0, -0.25), (0.25, original_first_output), (1.0, 1.625)] {
                assert_eq!(
                    easing
                        .evaluate_with_before_flag(
                            NormalizedProgress::new(input).unwrap(),
                            before_flag,
                        )
                        .value(),
                    expected
                );
            }
        }

        let first_before_zero = LinearEasing::new(vec![
            linear_point(-1.0, 0.0),
            linear_point(0.0, 1.0),
            linear_point(0.0, 2.0),
            linear_point(1.0, 3.0),
        ])
        .unwrap();
        assert_eq!(
            first_before_zero
                .evaluate_with_before_flag(NormalizedProgress::new(0.0).unwrap(), true)
                .value(),
            2.0
        );
    }

    #[test]
    fn linear_function_all_equal_inputs_select_outer_constants_and_exact_duplicate() {
        let easing = LinearEasing::new(vec![
            linear_point(0.25, 1.0),
            linear_point(0.25, 2.0),
            linear_point(0.25, 3.0),
        ])
        .unwrap();

        for (before_flag, exact_output) in [(false, 3.0), (true, 1.0)] {
            for (input, expected) in [(-1.0, 1.0), (0.25, exact_output), (2.0, 3.0)] {
                assert_eq!(
                    easing
                        .evaluate_unrestricted_with_before_flag(
                            EasingInput::new(input).unwrap(),
                            before_flag,
                        )
                        .unwrap()
                        .value(),
                    expected
                );
            }
            for (input, expected) in [(0.125, 1.0), (0.25, exact_output), (0.5, 3.0)] {
                assert_eq!(
                    easing
                        .evaluate_with_before_flag(
                            NormalizedProgress::new(input).unwrap(),
                            before_flag,
                        )
                        .value(),
                    expected
                );
            }
        }
    }

    #[test]
    fn linear_function_extreme_input_span_retains_scaled_finite_interpolation() {
        let easing = LinearEasing::new(vec![
            linear_point(-f64::MAX, -8.0),
            linear_point(f64::MAX, 8.0),
        ])
        .unwrap();

        for (input, expected) in [(-f64::MAX / 2.0, -4.0), (0.0, 0.0), (f64::MAX / 2.0, 4.0)] {
            assert_eq!(
                easing
                    .evaluate_unrestricted(EasingInput::new(input).unwrap())
                    .unwrap()
                    .value(),
                expected
            );
        }
    }

    #[test]
    fn linear_function_initial_duplicate_without_before_flag_selects_last() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, 0.0),
            linear_point(0.0, 0.4),
            linear_point(1.0, 1.0),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate_with_before_flag(NormalizedProgress::new(0.0).unwrap(), false)
                .value(),
            0.4
        );
    }

    #[test]
    fn linear_function_initial_duplicate_before_flag_selects_first() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, 0.0),
            linear_point(0.0, 0.4),
            linear_point(1.0, 1.0),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate_with_before_flag(NormalizedProgress::new(0.0).unwrap(), true)
                .value(),
            0.0
        );
    }

    #[test]
    fn linear_function_interior_duplicate_without_before_flag_selects_last() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, 0.0),
            linear_point(0.5, 0.25),
            linear_point(0.5, 0.75),
            linear_point(1.0, 1.0),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate_with_before_flag(NormalizedProgress::new(0.5).unwrap(), false)
                .value(),
            0.75
        );
    }

    #[test]
    fn linear_function_interior_duplicate_before_flag_selects_last() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, 0.0),
            linear_point(0.5, 0.25),
            linear_point(0.5, 0.75),
            linear_point(1.0, 1.0),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate_with_before_flag(NormalizedProgress::new(0.5).unwrap(), true)
                .value(),
            0.75
        );
    }

    #[test]
    fn linear_function_extrapolates_below_and_above_range() {
        let easing =
            LinearEasing::new(vec![linear_point(0.25, 0.5), linear_point(0.75, 1.5)]).unwrap();

        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(-0.25).unwrap())
                .unwrap()
                .value(),
            -0.5
        );
        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(1.25).unwrap())
                .unwrap()
                .value(),
            2.5
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            2.0
        );
    }

    #[test]
    fn unrestricted_linear_keyword_is_identity() {
        let easing = Easing::linear();

        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(-0.25).unwrap())
                .unwrap()
                .value(),
            -0.25
        );
        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(1.25).unwrap())
                .unwrap()
                .value(),
            1.25
        );
    }

    #[test]
    fn unrestricted_cubic_bezier_uses_endpoint_tangents() {
        // CSS Easing Level 1 extends cubic-bezier outside [0, 1] using endpoint tangents.
        let easing = Easing::cubic_bezier(CubicBezier::new(0.25, 0.5, 0.75, 0.5).unwrap());

        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(-0.5).unwrap())
                .unwrap()
                .value(),
            -1.0
        );
        assert_eq!(
            easing
                .evaluate_unrestricted(EasingInput::new(1.5).unwrap())
                .unwrap()
                .value(),
            2.0
        );
    }

    #[test]
    fn unrestricted_steps_follow_pinned_level_one_behavior() {
        // CSS Easing Level 1 step easing clamps only while input progress is within [0, 1].
        let jump_end = Easing::steps(Steps::new(4, StepPosition::JumpEnd).unwrap());
        let jump_start = Easing::steps(Steps::new(4, StepPosition::JumpStart).unwrap());

        assert_eq!(
            jump_end
                .evaluate_unrestricted(EasingInput::new(-0.25).unwrap())
                .unwrap()
                .value(),
            -0.25
        );
        assert_eq!(
            jump_end
                .evaluate_unrestricted(EasingInput::new(1.25).unwrap())
                .unwrap()
                .value(),
            1.25
        );
        assert_eq!(
            jump_start
                .evaluate_unrestricted(EasingInput::new(-0.25).unwrap())
                .unwrap()
                .value(),
            0.0
        );
        assert_eq!(
            jump_start
                .evaluate_unrestricted(EasingInput::new(1.25).unwrap())
                .unwrap()
                .value(),
            1.5
        );
    }

    #[test]
    fn unrestricted_easing_overflow_returns_typed_error() {
        let input = EasingInput::new(f64::MAX).unwrap();
        let easing = Easing::cubic_bezier(CubicBezier::new(0.5, 0.0, 0.5, -f64::MAX).unwrap());

        assert_eq!(
            easing.evaluate_unrestricted(input),
            Err(EasingError::NonFiniteEasedProgress {
                input,
                value: f64::INFINITY
            })
        );
    }

    #[test]
    fn normalized_valid_extreme_linear_function_never_panics() {
        let easing = LinearEasing::new(vec![
            linear_point(0.0, f64::MAX),
            linear_point(1.0, f64::MAX),
        ])
        .unwrap();

        assert_eq!(
            easing
                .evaluate(NormalizedProgress::new(0.5).unwrap())
                .value(),
            f64::MAX
        );
    }

    #[test]
    fn normalized_valid_extreme_cubic_bezier_never_panics() {
        let easing = CubicBezier::new(0.25, f64::MAX, 0.75, -f64::MAX).unwrap();
        let output = easing
            .evaluate(NormalizedProgress::new(0.5).unwrap())
            .value();

        assert!(output.is_finite());
    }

    #[test]
    fn steps_constructs_valid_jump_positions() {
        let jump_both = Steps::new(1, StepPosition::JumpBoth).unwrap();
        let jump_none = Steps::new(2, StepPosition::JumpNone).unwrap();

        assert_eq!(jump_both.count(), 1);
        assert_eq!(jump_both.position(), StepPosition::JumpBoth);
        assert_eq!(jump_none.count(), 2);
        assert_eq!(jump_none.position(), StepPosition::JumpNone);
    }

    #[test]
    fn steps_rejects_invalid_counts_for_jump_positions() {
        assert_eq!(
            Steps::new(0, StepPosition::JumpStart),
            Err(EasingError::InvalidStepCount {
                count: 0,
                position: StepPosition::JumpStart
            })
        );
        assert_eq!(
            Steps::new(0, StepPosition::JumpEnd),
            Err(EasingError::InvalidStepCount {
                count: 0,
                position: StepPosition::JumpEnd
            })
        );
        assert_eq!(
            Steps::new(0, StepPosition::JumpBoth),
            Err(EasingError::InvalidStepCount {
                count: 0,
                position: StepPosition::JumpBoth
            })
        );
        assert_eq!(
            Steps::new(1, StepPosition::JumpNone),
            Err(EasingError::InvalidStepCount {
                count: 1,
                position: StepPosition::JumpNone
            })
        );
    }

    #[test]
    fn step_start_and_step_end_aliases_match_steps() {
        assert_eq!(
            Easing::step_start(),
            Easing::steps(Steps::new(1, StepPosition::JumpStart).unwrap())
        );
        assert_eq!(
            Easing::step_end(),
            Easing::steps(Steps::new(1, StepPosition::JumpEnd).unwrap())
        );
    }

    #[test]
    fn easing_evaluate_delegates_to_cubic_bezier_and_steps() {
        let bezier = Easing::cubic_bezier(CubicBezier::new(0.42, 0.0, 0.58, 1.0).unwrap());
        let steps = Easing::steps(Steps::new(4, StepPosition::JumpEnd).unwrap());

        assert_eq!(
            bezier
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            0.25
        );
    }

    #[test]
    fn evaluate_with_before_flag_matches_existing_evaluate_without_before_flag() {
        let progress = NormalizedProgress::new(0.25).unwrap();
        let linear = Easing::linear();
        let bezier = Easing::cubic_bezier(CubicBezier::new(0.25, 0.0, 0.75, 1.0).unwrap());
        let steps = Easing::steps(Steps::new(4, StepPosition::JumpStart).unwrap());

        assert_eq!(
            linear.evaluate_with_before_flag(progress, false),
            linear.evaluate(progress)
        );
        assert_eq!(
            bezier.evaluate_with_before_flag(progress, false),
            bezier.evaluate(progress)
        );
        assert_eq!(
            steps.evaluate_with_before_flag(progress, false),
            steps.evaluate(progress)
        );
    }

    #[test]
    fn before_flag_uses_previous_step_at_exact_boundaries() {
        let jump_start = Easing::steps(Steps::new(4, StepPosition::JumpStart).unwrap());
        let jump_end = Easing::steps(Steps::new(4, StepPosition::JumpEnd).unwrap());
        let jump_both = Easing::steps(Steps::new(4, StepPosition::JumpBoth).unwrap());
        let jump_none = Easing::steps(Steps::new(4, StepPosition::JumpNone).unwrap());

        assert_eq!(
            jump_start
                .evaluate_with_before_flag(NormalizedProgress::new(0.0).unwrap(), true)
                .value(),
            0.0
        );
        assert_eq!(
            jump_start
                .evaluate_with_before_flag(NormalizedProgress::new(0.25).unwrap(), true)
                .value(),
            0.25
        );
        assert_eq!(
            jump_end
                .evaluate_with_before_flag(NormalizedProgress::new(0.25).unwrap(), true)
                .value(),
            0.0
        );
        assert_eq!(
            jump_both
                .evaluate_with_before_flag(NormalizedProgress::new(0.25).unwrap(), true)
                .value(),
            0.2
        );
        assert_eq!(
            jump_none
                .evaluate_with_before_flag(NormalizedProgress::new(0.25).unwrap(), true)
                .value(),
            0.0
        );
    }

    #[test]
    fn jump_end_steps_hold_until_next_boundary() {
        let steps = Steps::new(4, StepPosition::JumpEnd).unwrap();

        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.24).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            0.25
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            1.0
        );
    }

    #[test]
    fn jump_start_steps_jump_at_start_boundary() {
        let steps = Steps::new(4, StepPosition::JumpStart).unwrap();

        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.25
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.24).unwrap())
                .value(),
            0.25
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            0.5
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            1.0
        );
    }

    #[test]
    fn jump_both_steps_include_extra_start_and_end_jumps() {
        let steps = Steps::new(4, StepPosition::JumpBoth).unwrap();

        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.2
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.24).unwrap())
                .value(),
            0.2
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            0.4
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            1.0
        );
    }

    #[test]
    fn jump_none_steps_use_only_interior_boundaries() {
        let steps = Steps::new(4, StepPosition::JumpNone).unwrap();

        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.0).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.24).unwrap())
                .value(),
            0.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(0.25).unwrap())
                .value(),
            1.0 / 3.0
        );
        assert_eq!(
            steps
                .evaluate(NormalizedProgress::new(1.0).unwrap())
                .value(),
            1.0
        );
    }

    #[test]
    fn cubic_bezier_accepts_y_overshoot_when_finite() {
        let curve = CubicBezier::new(0.25, -0.5, 0.75, 1.5).unwrap();

        assert_eq!(curve.control_points(), (0.25, -0.5, 0.75, 1.5));
    }

    #[test]
    fn cubic_bezier_evaluates_endpoints_and_midpoint_monotonically() {
        let curve = CubicBezier::new(0.42, 0.0, 0.58, 1.0).unwrap();

        let start = curve
            .evaluate(NormalizedProgress::new(0.0).unwrap())
            .value();
        let middle = curve
            .evaluate(NormalizedProgress::new(0.5).unwrap())
            .value();
        let end = curve
            .evaluate(NormalizedProgress::new(1.0).unwrap())
            .value();

        assert_eq!(start, 0.0);
        assert!((middle - 0.5).abs() <= 0.000_001);
        assert_eq!(end, 1.0);
    }

    #[test]
    fn cubic_bezier_solves_x_axis_before_evaluating_y_axis() {
        let curve = CubicBezier::new(0.25, 0.0, 0.75, 1.0).unwrap();
        let t = 0.25;
        let input_x = cubic_axis(t, 0.25, 0.75);
        let expected_y = cubic_axis(t, 0.0, 1.0);

        let actual = curve
            .evaluate(NormalizedProgress::new(input_x).unwrap())
            .value();

        assert!((actual - expected_y).abs() <= 0.000_001);
    }

    #[test]
    fn cubic_bezier_analytic_curves_preserve_interior_outputs() {
        // With x controls (0, 0), x = t^3; with (1, 1), x = 1 - (1-t)^3.
        // Output controls (1/3, 2/3) make y = t independently of the x solver.
        for (x1, x2, samples) in [
            (
                0.0,
                0.0,
                [
                    (1.0 / 512.0, 0.125),
                    (1.0 / 64.0, 0.25),
                    (1.0 / 8.0, 0.5),
                    (27.0 / 64.0, 0.75),
                ],
            ),
            (
                1.0,
                1.0,
                [
                    (169.0 / 512.0, 0.125),
                    (37.0 / 64.0, 0.25),
                    (7.0 / 8.0, 0.5),
                    (63.0 / 64.0, 0.75),
                ],
            ),
        ] {
            let curve = CubicBezier::new(x1, 1.0 / 3.0, x2, 2.0 / 3.0).unwrap();
            for (input, expected) in samples {
                let actual = curve
                    .evaluate(NormalizedProgress::new(input).unwrap())
                    .value();
                assert!((actual - expected).abs() <= 1.0e-8);
            }
        }

        // Conversely, x = t and y = t^3.
        let curve = CubicBezier::new(1.0 / 3.0, 0.0, 2.0 / 3.0, 0.0).unwrap();
        for (input, expected) in [
            (0.125, 1.0 / 512.0),
            (0.25, 1.0 / 64.0),
            (0.5, 1.0 / 8.0),
            (0.75, 27.0 / 64.0),
        ] {
            let actual = curve
                .evaluate(NormalizedProgress::new(input).unwrap())
                .value();
            assert!((actual - expected).abs() <= 1.0e-8);
        }
    }

    #[test]
    fn cubic_bezier_keyword_curve_preserves_independently_calculated_samples() {
        let curve = Easing::keyword(EasingKeyword::Ease);
        // At t = 1/4, 1/2, 3/4, the keyword polynomials are
        // x = 3t/4 - 3t^2/4 + t^3 and y = 3t/10 + 12t^2/5 - 17t^3/10.
        for (input, expected) in [(0.15625, 0.1984375), (0.3125, 0.5375), (0.5625, 0.8578125)] {
            let actual = curve
                .evaluate(NormalizedProgress::new(input).unwrap())
                .value();
            assert!((actual - expected).abs() <= 1.0e-8);
        }
    }

    #[test]
    fn cubic_bezier_flat_regions_preserve_identity_and_finite_endpoints() {
        // Identical x/y controls imply y = x, including flat x derivatives.
        for (control_1, control_2) in [(0.0, 0.0), (1.0, 1.0), (1.0, 0.0)] {
            let curve = CubicBezier::new(control_1, control_1, control_2, control_2).unwrap();
            for input in [
                0.0,
                1.0e-12,
                0.5 - 1.0 / 17_179_869_184.0,
                0.5,
                0.5 + 1.0 / 17_179_869_184.0,
                1.0 - 1.0e-12,
                1.0,
            ] {
                let actual = curve
                    .evaluate(NormalizedProgress::new(input).unwrap())
                    .value();
                assert!(actual.is_finite());
                assert!((actual - input).abs() <= 1.0e-8);
                if input == 0.0 || input == 1.0 {
                    assert_eq!(actual, input);
                }
            }
        }
    }

    #[test]
    fn cubic_bezier_extreme_opposite_outputs_preserve_scaled_analytic_values() {
        let curve = CubicBezier::new(1.0 / 3.0, f64::MAX, 2.0 / 3.0, -f64::MAX).unwrap();
        // x = t; y / MAX is 3t(1-t)(1-2t) plus the negligible t^3 / MAX.
        for (input, expected_scaled) in [(0.25, 9.0 / 32.0), (0.5, 0.0), (0.75, -9.0 / 32.0)] {
            let actual = curve
                .evaluate(NormalizedProgress::new(input).unwrap())
                .value();
            assert!(actual.is_finite());
            assert!((actual / f64::MAX - expected_scaled).abs() <= 1.0e-8);
        }
        for input in [0.0, 1.0] {
            assert_eq!(
                curve
                    .evaluate(NormalizedProgress::new(input).unwrap())
                    .value(),
                input
            );
        }
    }

    #[test]
    fn cubic_bezier_preserves_eased_output_overshoot() {
        let curve = CubicBezier::new(0.25, -0.5, 0.75, 1.5).unwrap();
        let t = 0.1;
        let input_x = cubic_axis(t, 0.25, 0.75);
        let expected_y = cubic_axis(t, -0.5, 1.5);

        let actual = curve
            .evaluate(NormalizedProgress::new(input_x).unwrap())
            .value();

        assert!(actual < 0.0);
        assert!((actual - expected_y).abs() <= 0.000_001);
    }

    #[test]
    fn retained_easing_diagnostics_have_display_text() {
        let numeric = EasingError::Numeric(NumericError::non_finite(NumericInput::Ratio));
        assert_eq!(
            numeric.to_string(),
            "numeric easing input is invalid: ratio must be finite"
        );
        assert_eq!(
            std::error::Error::source(&numeric).unwrap().to_string(),
            "ratio must be finite"
        );

        assert_eq!(
            EasingError::NonFiniteEasingInput { value: f64::NAN }.to_string(),
            "easing input must be finite, got NaN"
        );
        assert_eq!(
            EasingError::NonFiniteControlPoint {
                name: "x1",
                value: f64::INFINITY,
            }
            .to_string(),
            "cubic-bezier control point x1 must be finite, got inf"
        );
        assert_eq!(
            EasingError::EmptyLinearFunction.to_string(),
            "linear easing must contain at least one control point"
        );
        assert_eq!(
            EasingError::NonFiniteLinearControlPoint {
                index: 2,
                component: LinearControlPointComponent::Input,
                value: f64::NAN,
            }
            .to_string(),
            "linear control point 2 input must be finite, got NaN"
        );
        assert_eq!(
            EasingError::DecreasingLinearControlPointInput {
                index: 2,
                previous_input: 0.5,
                input: 0.25,
            }
            .to_string(),
            "linear control point 2 input 0.25 must be greater than or equal to previous input 0.5"
        );
        assert_eq!(
            EasingError::NonFiniteNormalizedOutput {
                input: NormalizedProgress::new(0.5).unwrap(),
                value: f64::INFINITY,
            }
            .to_string(),
            "linear easing normalized output at progress 0.5 must be finite, got inf"
        );
        assert_eq!(
            EasingError::NonFiniteEasedProgress {
                input: EasingInput::new(2.0).unwrap(),
                value: f64::INFINITY,
            }
            .to_string(),
            "easing output at input 2 must be finite, got inf"
        );
        assert_eq!(
            EasingError::InvalidStepCount {
                count: 0,
                position: StepPosition::JumpNone,
            }
            .to_string(),
            "steps count 0 is invalid for jump-none"
        );
        assert!(std::error::Error::source(&EasingError::EmptyLinearFunction).is_none());
    }

    fn linear_point(input: f64, output: f64) -> LinearControlPoint {
        LinearControlPoint::new(input, output)
    }

    fn cubic_axis(t: f64, control_1: f64, control_2: f64) -> f64 {
        let inverse_t = 1.0 - t;
        (3.0 * inverse_t * inverse_t * t * control_1)
            + (3.0 * inverse_t * t * t * control_2)
            + (t * t * t)
    }
}

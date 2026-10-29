//! Accepted physical baseline evidence, resolved in the returned border frame.
//!
//! A fresh accumulator belongs to one in-flow pass. Its owning result selects
//! that pass's geometry and consumes the evidence once; none enters a cache.
use crate::geometry::{PhysicalAxis, PhysicalSide};
use crate::group_alignment::{NumericError, add, scalar};
use crate::output::{NamedBaselineDemand, NamedBaselineSet};
use crate::{
    BaselineCoordinateOf, BaselineFactOf, BaselineName, BaselinePairOf, BaselinesOf,
    ComputeOutputOf, FlowAxes, InlineBaselineContext, LayoutScalar, LayoutTextNumericError,
    NamedBaselinesOf, Point, Size,
};

/// Coordinates have already been made relative to the actual used origin.
/// Measured retains its real projection frame, not canonical placement proof.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BaselineFrame {
    Measured {
        used_size: Size<f64>,
        axes: FlowAxes,
    },
    CanonicalLocal {
        axes: FlowAxes,
    },
}

impl BaselineFrame {
    fn axes(self) -> FlowAxes {
        match self {
            Self::Measured { axes, .. } | Self::CanonicalLocal { axes } => axes,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Evidence {
    value: f64,
    axis: PhysicalAxis,
    frame: BaselineFrame,
}

fn component<T: Copy>(point: Point<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => point.x,
        PhysicalAxis::Vertical => point.y,
    }
}

fn extent<T: Copy>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

/// Both operands are required. Cancellation is valid; absorption of either
/// contribution is not. No coordinate snapping belongs to this transition.
fn sum(a: f64, b: f64) -> Result<f64, NumericError> {
    scalar::<f64>(a)?;
    scalar::<f64>(b)?;
    let result = add::<f64>(a, b)?;
    add::<f64>(b, a)?;
    Ok(result)
}

impl Evidence {
    fn new(value: f64, axis: PhysicalAxis, frame: BaselineFrame) -> Result<Self, NumericError> {
        scalar::<f64>(value)?;
        if let BaselineFrame::Measured { used_size, .. } = frame {
            let used = scalar::<f64>(extent(used_size, axis))?;
            if used < 0.0 {
                return Err(NumericError::new(used, LayoutTextNumericError::OutOfRange));
            }
        }
        Ok(Self { value, axis, frame })
    }

    fn resolve<S: LayoutScalar>(self, output_size: Size<S>) -> Result<S, NumericError> {
        let value = match self.frame {
            BaselineFrame::CanonicalLocal { .. } => self.value,
            BaselineFrame::Measured { used_size, axes } => {
                let side = if self.axis == axes.block_axis() {
                    axes.block_start()
                } else {
                    axes.inline_start()
                };
                match side {
                    PhysicalSide::Left | PhysicalSide::Top => self.value,
                    PhysicalSide::Right | PhysicalSide::Bottom => {
                        let change = sum(
                            extent(output_size, self.axis).to_f64(),
                            -extent(used_size, self.axis),
                        )?;
                        sum(self.value, change)?
                    }
                }
            }
        };
        scalar::<S>(value)
    }
}

#[derive(Clone, Copy, Debug)]
enum Fact {
    Unknown,
    Absent,
    Available(Evidence),
}

#[derive(Clone, Copy, Debug)]
struct Pair {
    first: Fact,
    last: Fact,
}

impl Pair {
    const ABSENT: Self = Self {
        first: Fact::Absent,
        last: Fact::Absent,
    };
    const UNKNOWN: Self = Self {
        first: Fact::Unknown,
        last: Fact::Unknown,
    };

    fn record(&mut self, next: Self) {
        if matches!(self.first, Fact::Absent) {
            self.first = next.first;
        }
        if !matches!(next.last, Fact::Absent) {
            self.last = next.last;
        }
    }
}

fn name_index(name: BaselineName) -> usize {
    match name {
        BaselineName::Alphabetic => 0,
        BaselineName::Central => 1,
    }
}

/// Constant-size source-ordered first/last state. Deliberately not Clone:
/// its accepted InFlowResult consumes it at the final output boundary.
#[derive(Debug)]
pub(crate) struct PendingBaselines {
    first: Point<Option<Evidence>>,
    last: Point<Option<Evidence>>,
    named: [Pair; 2],
}

impl PendingBaselines {
    pub(crate) fn new() -> Self {
        Self {
            first: Point::NONE,
            last: Point::NONE,
            named: [Pair::ABSENT; 2],
        }
    }

    fn record(&mut self, next: Self) {
        self.first.x = self.first.x.or(next.first.x);
        self.first.y = self.first.y.or(next.first.y);
        self.last.x = next.last.x.or(self.last.x);
        self.last.y = next.last.y.or(self.last.y);
        for (old, next) in self.named.iter_mut().zip(next.named) {
            old.record(next);
        }
    }

    fn generic(&mut self, first: Option<Evidence>, last: Option<Evidence>) {
        for (point, evidence) in [(&mut self.first, first), (&mut self.last, last)] {
            if let Some(evidence) = evidence {
                match evidence.axis {
                    PhysicalAxis::Horizontal => point.x = Some(evidence),
                    PhysicalAxis::Vertical => point.y = Some(evidence),
                }
            }
        }
    }

    /// Shared boundary for already-physical facts. Native enters directly;
    /// the numeric producer explicitly projects its accepted S logical facts.
    pub(crate) fn line(
        &mut self,
        first: Option<f64>,
        last: Option<f64>,
        frame: BaselineFrame,
        context: InlineBaselineContext,
        first_named: bool,
        last_named: bool,
    ) -> Result<(), NumericError> {
        if first.is_none() && last.is_none() {
            return Ok(());
        }
        let axis = frame.axes().block_axis();
        let first = first
            .map(|value| Evidence::new(value, axis, frame))
            .transpose()?;
        let last = last
            .map(|value| Evidence::new(value, axis, frame))
            .transpose()?;
        let mut next = Self::new();
        next.generic(first, last);
        next.named = [Pair::UNKNOWN; 2];
        if let InlineBaselineContext::Named(name) = context {
            let fact = |value: Option<Evidence>, authority| match value {
                None => Fact::Absent,
                Some(value) if authority => Fact::Available(value),
                Some(_) => Fact::Unknown,
            };
            next.named[name_index(name)] = Pair {
                first: fact(first, first_named),
                last: fact(last, last_named),
            };
        }
        self.record(next);
        Ok(())
    }

    /// The child already resolved its own frame. Its accepted parent placement
    /// creates a new parent-frame coordinate, never a second child rebase.
    pub(crate) fn resolved_child<S: LayoutScalar>(
        &mut self,
        output: ComputeOutputOf<S>,
        child_axes: FlowAxes,
        location: Point<S>,
        parent_frame: BaselineFrame,
    ) -> Result<(), NumericError> {
        let placed = |axis, value: S| {
            let value = sum(component(location, axis).to_f64(), value.to_f64())?;
            Evidence::new(value, axis, parent_frame)
        };
        let baselines = output.baselines();
        let first = baselines
            .first_block_baseline(child_axes)
            .map(|value| placed(value.axis(), value.coordinate()))
            .transpose()?;
        let last = baselines
            .last_block_baseline(child_axes)
            .map(|value| placed(value.axis(), value.coordinate()))
            .transpose()?;
        let mut next = Self::new();
        next.generic(first, last);
        for name in [BaselineName::Alphabetic, BaselineName::Central] {
            let pair = output.named_baselines.get(name);
            let fact = |value: BaselineFactOf<S>| match value {
                BaselineFactOf::Unknown => Ok(Fact::Unknown),
                BaselineFactOf::Absent => Ok(Fact::Absent),
                BaselineFactOf::Available(value) => {
                    placed(value.axis(), value.offset()).map(Fact::Available)
                }
            };
            next.named[name_index(name)] = Pair {
                first: fact(pair.first())?,
                last: fact(pair.last())?,
            };
        }
        self.record(next);
        Ok(())
    }

    pub(crate) fn demand_for_following(&self, demand: NamedBaselineDemand) -> NamedBaselineDemand {
        match demand {
            NamedBaselineDemand::Fact {
                name,
                set: NamedBaselineSet::First,
            } if !matches!(self.named[name_index(name)].first, Fact::Absent) => {
                NamedBaselineDemand::None
            }
            other => other,
        }
    }

    pub(crate) fn resolve<S: LayoutScalar>(
        self,
        output_size: Size<S>,
    ) -> Result<(BaselinesOf<S>, NamedBaselinesOf<S>), NumericError> {
        let point = |point: Point<Option<Evidence>>| -> Result<_, NumericError> {
            Ok(Point::new(
                point
                    .x
                    .map(|value| value.resolve(output_size))
                    .transpose()?,
                point
                    .y
                    .map(|value| value.resolve(output_size))
                    .transpose()?,
            ))
        };
        let generic = BaselinesOf {
            first: point(self.first)?,
            last: point(self.last)?,
        };
        let mut named = NamedBaselinesOf::absent();
        for name in [BaselineName::Alphabetic, BaselineName::Central] {
            let pair = self.named[name_index(name)];
            let fact = |value| -> Result<_, NumericError> {
                Ok(match value {
                    Fact::Unknown => BaselineFactOf::Unknown,
                    Fact::Absent => BaselineFactOf::Absent,
                    Fact::Available(value) => {
                        let coordinate = value.resolve(output_size)?;
                        BaselineFactOf::Available(
                            BaselineCoordinateOf::try_new(value.axis, coordinate)
                                .expect("checked scalar baseline coordinate is finite"),
                        )
                    }
                })
            };
            named = named.with(
                name,
                BaselinePairOf::new(fact(pair.first)?, fact(pair.last)?),
            );
        }
        Ok((generic, named))
    }
}

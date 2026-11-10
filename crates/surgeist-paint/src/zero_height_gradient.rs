//! Native zero-height radial color projection. No replacement axes are created.

use crate::{
    GradientPosition, GradientStopItem, PaintError, PaintOptions, PaintPhase, PaintReport,
    RadialGradientCase, RadialGradientGeometry, ResourceKind, WorkKind,
    accounting::{
        Meter, OperationKind, allocate, apply, bytes, charge, finish, release, setup_error,
    },
    gradient_stops::{
        ColorArity, ColorOccurrence, ColorSourceItem, MultipleColors, ScalarPosition, StopWork,
        UsedGradientRecord, distribute_with, expand_with, fill_run, fixup_with, interior_omitted,
        validate_source,
    },
};
use std::mem::take;
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Selects the ordinary last-color or repeating average semantic case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientRepeat {
    None,
    Repeat,
}

/// Native case or required color-measure representation refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZeroHeightGradientError {
    ExpectedZeroHeight,
    UnrepresentableWeight {
        left_source_index: usize,
        left_position_index: usize,
        right_source_index: usize,
        right_position_index: usize,
    },
}
impl From<ZeroHeightGradientError> for PaintError {
    fn from(value: ZeroHeightGradientError) -> Self {
        Self::ZeroHeightGradient(value)
    }
}

/// One actual color occurrence and its finite nonnegative color-measure weight.
/// The payload is borrowed from the exact source; hints are never colors.
#[derive(Debug)]
pub struct WeightedGradientColor<'a, C> {
    pub source_index: usize,
    pub position_index: usize,
    pub payload: &'a C,
    pub weight: f64,
}

#[derive(Debug)]
struct WeightedColor<'s, C> {
    occurrence: ColorOccurrence<'s, C>,
    weight: f64,
}

/// Borrows one actual weighted occurrence without projecting its fields.
/// A consumer can admit its visit before calling [`Self::color`].
///
/// ```compile_fail
/// use surgeist_paint::WeightedGradientRecord;
/// fn fabricate(authentic: WeightedGradientRecord<'_, ()>) {
///     let _ = WeightedGradientRecord { ..authentic };
/// }
/// ```
#[derive(Debug)]
pub struct WeightedGradientRecord<'a, C> {
    record: &'a WeightedColor<'a, C>,
}
impl<'a, C> WeightedGradientRecord<'a, C> {
    /// Projects this qualified occurrence without source lookup or color math.
    pub fn color(&self) -> WeightedGradientColor<'a, C> {
        WeightedGradientColor {
            source_index: self.record.occurrence.source_index(),
            position_index: self.record.occurrence.position_index(),
            payload: self.record.occurrence.payload(),
            weight: self.record.weight,
        }
    }
}

#[derive(Debug)]
enum Measure<'s, C> {
    Last(WeightedColor<'s, C>),
    Average {
        endpoints: [WeightedColor<'s, C>; 2],
        middle: Vec<WeightedColor<'s, C>>,
    },
}

/// Authenticated whole-image zero-height colors, borrowing their exact source.
/// Ordinary selection emits only the last occurrence at weight one. Repetition
/// emits every expanded color occurrence in order, including zero-weight ones.
/// This is a color measure, not a finite gradient-line position proof.
///
/// The source remains borrowed for the entire result lifetime:
///
/// ```compile_fail
/// use surgeist_paint::{GradientRepeat, GradientStopItem, GradientStopPositions,
///     PaintLimits, PaintOptions, ProfileMode, RadialGradientGeometry,
///     resolve_zero_height_gradient_colors};
/// fn escape(geometry: &RadialGradientGeometry) {
///     let colors = {
///         let source = [GradientStopItem::Stop {
///             payload: String::from("borrowed"), positions: GradientStopPositions::Omitted,
///         }];
///         resolve_zero_height_gradient_colors(geometry, &source, GradientRepeat::None,
///             PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)).result.unwrap()
///     };
///     let _ = colors.colors().next();
/// }
/// ```
///
/// Only the resolver can construct a qualified owner:
///
/// ```compile_fail
/// use surgeist_paint::{GradientStopItem, ZeroHeightGradientColors};
/// fn fabricate<'a>(source: &'a [GradientStopItem<()>], authentic: ZeroHeightGradientColors<'a, ()>) {
///     let _ = ZeroHeightGradientColors { source, ..authentic };
/// }
/// ```
#[derive(Debug)]
pub struct ZeroHeightGradientColors<'a, C> {
    source: &'a [GradientStopItem<C>],
    measure: Measure<'a, C>,
}
impl<'a, C> ZeroHeightGradientColors<'a, C> {
    pub const fn source(&self) -> &'a [GradientStopItem<C>] {
        self.source
    }

    pub fn colors(&self) -> impl ExactSizeIterator<Item = WeightedGradientColor<'_, C>> {
        self.records().map(|record| record.color())
    }

    /// Borrows the selected Last occurrence. `None` means the stored measure is
    /// Average, including its own first and last weighted endpoints.
    pub fn selected_last_color(&self) -> Option<WeightedGradientRecord<'_, C>> {
        match &self.measure {
            Measure::Last(record) => Some(WeightedGradientRecord { record }),
            Measure::Average { .. } => None,
        }
    }

    /// Borrows all real weighted occurrences in order, including zero weights.
    pub fn records(&self) -> impl ExactSizeIterator<Item = WeightedGradientRecord<'_, C>> {
        let (first, middle, last) = match &self.measure {
            Measure::Last(record) => (Some(record), [].iter(), None),
            Measure::Average {
                endpoints: [first, last],
                middle,
            } => (Some(first), middle.iter(), Some(last)),
        };
        PreparedColors {
            first,
            middle,
            last,
        }
        .map(|record| WeightedGradientRecord { record })
    }
}
struct PreparedColors<'r, 's, C> {
    first: Option<&'r WeightedColor<'s, C>>,
    middle: std::slice::Iter<'r, WeightedColor<'s, C>>,
    last: Option<&'r WeightedColor<'s, C>>,
}
impl<'r, 's, C> Iterator for PreparedColors<'r, 's, C> {
    type Item = &'r WeightedColor<'s, C>;
    fn next(&mut self) -> Option<Self::Item> {
        self.first
            .take()
            .or_else(|| self.middle.next())
            .or_else(|| self.last.take())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}
impl<C> ExactSizeIterator for PreparedColors<'_, '_, C> {
    fn len(&self) -> usize {
        usize::from(self.first.is_some()) + self.middle.len() + usize::from(self.last.is_some())
    }
}

/// One complete borrowed color measure or typed refusal with accepted work.
#[derive(Debug)]
pub struct ZeroHeightGradientColorsOutcome<'a, C> {
    pub result: Result<ZeroHeightGradientColors<'a, C>, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves zero-height radial colors using the selected substituted-width
/// limit toward positive infinity. Geometry must be the authentic ZeroHeight
/// case. All raw grammar/components are validated before selecting the last
/// color or the specified stop-only repeating average. No payload is evaluated.
///
/// Hints affect ordering and omitted-color positioning, but have no independent
/// color weight or curve integral. The result cannot outlive its source.
pub fn resolve_zero_height_gradient_colors<'a, C>(
    geometry: &RadialGradientGeometry,
    source: &'a [GradientStopItem<C>],
    repeat: GradientRepeat,
    options: PaintOptions,
) -> ZeroHeightGradientColorsOutcome<'a, C> {
    let mut meter = match options.operation(OperationKind::ZeroHeightGradient) {
        Ok(meter) => meter,
        Err(error) => {
            return ZeroHeightGradientColorsOutcome {
                result: Err(error),
                report: None,
            };
        }
    };
    let root = meter.root();
    let scopes = match setup_scopes(&mut meter) {
        Ok(scopes) => scopes,
        Err(error) => {
            let (result, report) = finish(meter, Err(error));
            return ZeroHeightGradientColorsOutcome { result, report };
        }
    };
    let mut backing = Backing::new();
    let mut fixed_owner = false;
    let result = (|| {
        let validated = meter
            .with_phase(scopes[0], |p| {
                let mut work = StopWork::new(p, scopes[0]);
                if !work.validation(|| matches!(geometry.case(), RadialGradientCase::ZeroHeight))? {
                    return Err(ZeroHeightGradientError::ExpectedZeroHeight.into());
                }
                validate_source(&mut work, source)
            })
            .map_err(setup_error)??;
        let (validated_source, record_count, colors) = validated.into_parts();
        let last = colors.last();
        let prepared = match (repeat, colors) {
            (GradientRepeat::None, _) | (_, ColorArity::One(_)) => PreparedMeasure::Last(last),
            (GradientRepeat::Repeat, ColorArity::Multiple(colors)) => {
                let input = MultipleInput {
                    source: validated_source,
                    colors,
                    record_count,
                };
                PreparedMeasure::Average(input.prepare(&mut meter, scopes, &mut backing)?)
            }
        };
        meter
            .with_phase(scopes[4], |p| {
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<ZeroHeightGradientColors<'_, C>>(1)?,
                    ),
                )?;
                fixed_owner = true;
                if matches!(prepared, PreparedMeasure::Last(_)) {
                    charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                }
                charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                let measure = match prepared {
                    PreparedMeasure::Last(occurrence) => Measure::Last(WeightedColor {
                        occurrence,
                        weight: 1.0,
                    }),
                    PreparedMeasure::Average(endpoints) => Measure::Average {
                        endpoints,
                        middle: take(&mut backing.weights),
                    },
                };
                Ok(ZeroHeightGradientColors { source, measure })
            })
            .map_err(setup_error)?
    })();
    let cleanup = meter
        .with_phase(scopes[5], |p| {
            release(
                p,
                root,
                take(&mut backing.normalized),
                ResourceKind::ScratchBytes,
            )?;
            release(
                p,
                root,
                take(&mut backing.pairs),
                ResourceKind::ScratchBytes,
            )?;
            release(
                p,
                root,
                take(&mut backing.uniform),
                ResourceKind::ScratchBytes,
            )?;
            release(
                p,
                root,
                take(&mut backing.varying),
                ResourceKind::ScratchBytes,
            )?;
            if result.is_err() {
                release(
                    p,
                    root,
                    take(&mut backing.weights),
                    ResourceKind::RetainedResultBytes,
                )?;
                if fixed_owner {
                    apply(
                        p,
                        root,
                        Change::Release(
                            ResourceKind::RetainedResultBytes,
                            bytes::<ZeroHeightGradientColors<'_, C>>(1)?,
                        ),
                    )?;
                }
            }
            Ok(())
        })
        .map_err(setup_error)
        .and_then(|done| done);
    let result = result.and_then(|value| cleanup.map(|()| value));
    let (result, report) = finish(meter, result);
    ZeroHeightGradientColorsOutcome { result, report }
}

fn setup_scopes(meter: &mut Meter) -> Result<[Scope; 6], PaintError> {
    let mut scopes = [meter.root(); 6];
    for (slot, phase) in scopes.iter_mut().zip([
        PaintPhase::GradientValidation,
        PaintPhase::GradientStopResolution,
        PaintPhase::GradientStopFixup,
        PaintPhase::GradientStopDistribution,
        PaintPhase::Publication,
        PaintPhase::Cleanup,
    ]) {
        *slot = meter.child(
            meter.root(),
            phase,
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
    }
    Ok(scopes)
}

enum PreparedMeasure<'s, C> {
    Last(ColorOccurrence<'s, C>),
    Average([WeightedColor<'s, C>; 2]),
}
struct Backing<'s, C> {
    pairs: Vec<UsedGradientRecord<GradientPosition>>,
    normalized: Vec<UsedGradientRecord<Normalized>>,
    uniform: Vec<QualifiedColor<'s, C, ()>>,
    varying: Vec<QualifiedColor<'s, C, Normalized>>,
    weights: Vec<WeightedColor<'s, C>>,
}
impl<C> Backing<'_, C> {
    fn new() -> Self {
        Self {
            pairs: Vec::new(),
            normalized: Vec::new(),
            uniform: Vec::new(),
            varying: Vec::new(),
            weights: Vec::new(),
        }
    }
}

// The only producer first validates this exact source, then expands and fixes
// its own records before constructing the fixed-position owner below.
struct MultipleInput<'s, C> {
    source: &'s [GradientStopItem<C>],
    colors: MultipleColors<'s, C>,
    record_count: usize,
}
impl<'s, C> MultipleInput<'s, C> {
    fn prepare(
        self,
        meter: &mut Meter,
        scopes: [Scope; 6],
        backing: &mut Backing<'s, C>,
    ) -> Result<[WeightedColor<'s, C>; 2], PaintError> {
        let root = meter.root();
        meter
            .with_phase(scopes[1], |p| {
                backing.pairs = allocate(p, root, self.record_count, ResourceKind::ScratchBytes)?;
                expand_with(
                    &mut StopWork::new(p, scopes[1]),
                    self.source,
                    &mut backing.pairs,
                    GradientPosition {
                        fraction: 0.0,
                        length: 0.0,
                    },
                    |_, _, _, position| Ok(position),
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[2], |p| {
                fixup_with(
                    &mut StopWork::new(p, scopes[2]),
                    self.source,
                    &mut backing.pairs,
                    GradientPosition {
                        fraction: 0.0,
                        length: 0.0,
                    },
                    GradientPosition {
                        fraction: 1.0,
                        length: 0.0,
                    },
                    lex_less,
                )
            })
            .map_err(setup_error)??;
        let fixed = FixedInput {
            source: self.source,
            colors: self.colors,
            pairs: &backing.pairs,
        };
        meter
            .with_phase(scopes[3], |p| {
                backing.weights = allocate(
                    p,
                    root,
                    fixed.colors.count() - 2,
                    ResourceKind::RetainedResultBytes,
                )?;
                let mut work = StopWork::new(p, scopes[3]);
                match projection::Projection::for_fixed(&mut work, &fixed)? {
                    projection::Projection::Uniform => {
                        backing.uniform = allocate(
                            work.meter(),
                            root,
                            fixed.colors.count() - 2,
                            ResourceKind::ScratchBytes,
                        )?;
                        let qualified = fixed.qualify_uniform(&mut work, &mut backing.uniform)?;
                        uniform(
                            &mut work,
                            qualified,
                            &mut backing.weights,
                            fixed.colors.count(),
                        )
                    }
                    projection::Projection::Varying(span) => {
                        backing.normalized = allocate(
                            work.meter(),
                            root,
                            self.record_count,
                            ResourceKind::ScratchBytes,
                        )?;
                        span.project(&mut work, &mut backing.normalized)?;
                        distribute_with(
                            &mut work,
                            fixed.source,
                            &mut backing.normalized,
                            fill_run,
                        )?;
                        backing.varying = allocate(
                            work.meter(),
                            root,
                            fixed.colors.count() - 2,
                            ResourceKind::ScratchBytes,
                        )?;
                        let qualified = fixed.qualify_varying(
                            &mut work,
                            &backing.normalized,
                            &mut backing.varying,
                        )?;
                        weighted(&mut work, qualified, &mut backing.weights)
                    }
                }
            })
            .map_err(setup_error)?
    }
}
struct FixedInput<'s, 'r, C> {
    source: &'s [GradientStopItem<C>],
    colors: MultipleColors<'s, C>,
    pairs: &'r [UsedGradientRecord<GradientPosition>],
}
struct QualifiedColor<'s, C, P> {
    occurrence: ColorOccurrence<'s, C>,
    position: P,
}
impl<C, P: Copy> Copy for QualifiedColor<'_, C, P> {}
impl<C, P: Copy> Clone for QualifiedColor<'_, C, P> {
    fn clone(&self) -> Self {
        *self
    }
}
struct QualifiedMultiple<'s, 'r, C, P> {
    first: QualifiedColor<'s, C, P>,
    last: QualifiedColor<'s, C, P>,
    middle: &'r [QualifiedColor<'s, C, P>],
}
impl<'s, C> FixedInput<'s, '_, C> {
    fn qualify_uniform<'r>(
        &self,
        work: &mut StopWork<'_, '_, '_>,
        middle: &'r mut Vec<QualifiedColor<'s, C, ()>>,
    ) -> Result<QualifiedMultiple<'s, 'r, C, ()>, PaintError> {
        let first = work.output(|| QualifiedColor {
            occurrence: self.colors.first(),
            position: (),
        })?;
        // With exactly two colors, validation has already captured the complete
        // payload input. Only additional interior occurrences need a source walk.
        if self.colors.count() > 2 {
            let mut ordinal = 0;
            for source_index in 0..self.source.len() {
                if let ColorSourceItem::Stop { first, second, .. } =
                    work.color_source(self.source, source_index)?
                {
                    for occurrence in std::iter::once(first).chain(second) {
                        if ordinal != 0 && ordinal != self.colors.count() - 1 {
                            work.emit(
                                middle,
                                QualifiedColor {
                                    occurrence,
                                    position: (),
                                },
                            )?;
                        }
                        ordinal += 1;
                    }
                }
            }
        }
        let last = work.output(|| QualifiedColor {
            occurrence: self.colors.last(),
            position: (),
        })?;
        Ok(QualifiedMultiple {
            first,
            last,
            middle,
        })
    }
    fn qualify_varying<'r>(
        &self,
        work: &mut StopWork<'_, '_, '_>,
        normalized: &[UsedGradientRecord<Normalized>],
        middle: &'r mut Vec<QualifiedColor<'s, C, Normalized>>,
    ) -> Result<QualifiedMultiple<'s, 'r, C, Normalized>, PaintError> {
        let first = work.record(normalized, 0)?.position;
        let mut last = work.record(normalized, normalized.len() - 1)?.position;
        let first = work.output(|| QualifiedColor {
            occurrence: self.colors.first(),
            position: first,
        })?;
        if self.colors.count() == 2 {
            // Every interior record is a hint. Both payloads are already
            // authenticated; inspect only the normalized strictness still needed.
            for index in 1..normalized.len() - 1 {
                last.strict |= work.record(normalized, index)?.position.strict;
            }
        } else {
            let mut cursor = 0;
            let mut strict = false;
            for source_index in 0..self.source.len() {
                match work.color_source(self.source, source_index)? {
                    ColorSourceItem::Hint { .. } => {
                        strict |= work.record(normalized, cursor)?.position.strict;
                        cursor += 1;
                    }
                    ColorSourceItem::Stop {
                        first: occurrence,
                        second,
                        ..
                    } => {
                        for occurrence in std::iter::once(occurrence).chain(second) {
                            if cursor == normalized.len() - 1 {
                                last.strict |= strict;
                            } else if cursor != 0 {
                                let mut position = work.record(normalized, cursor)?.position;
                                position.strict |= strict;
                                work.emit(
                                    middle,
                                    QualifiedColor {
                                        occurrence,
                                        position,
                                    },
                                )?;
                            }
                            strict = false;
                            cursor += 1;
                        }
                    }
                }
            }
        }
        let last = work.output(|| QualifiedColor {
            occurrence: self.colors.last(),
            position: last,
        })?;
        Ok(QualifiedMultiple {
            first,
            last,
            middle,
        })
    }
}
// The first adjacent pair is mandatory. The tail is a closed traversal state,
// and can finish only after its required final occurrence has been issued.
enum ColorTail<'s, 'r, C, P> {
    Remaining {
        middle: &'r [QualifiedColor<'s, C, P>],
        last: QualifiedColor<'s, C, P>,
    },
    Done,
}
struct ColorDriver<'s, 'r, C, P> {
    first: QualifiedColor<'s, C, P>,
    second: QualifiedColor<'s, C, P>,
    tail: ColorTail<'s, 'r, C, P>,
}
impl<'s, C, P: Copy> ColorTail<'s, '_, C, P> {
    fn next(
        &mut self,
        work: &mut StopWork<'_, '_, '_>,
    ) -> Result<Option<QualifiedColor<'s, C, P>>, PaintError> {
        match self {
            Self::Done => Ok(None),
            Self::Remaining { middle, last } => match middle.split_first() {
                Some((first, rest)) => {
                    let value = work.visit(|| *first)?;
                    *middle = rest;
                    Ok(Some(value))
                }
                None => {
                    let value = work.visit(|| *last)?;
                    *self = Self::Done;
                    Ok(Some(value))
                }
            },
        }
    }
}
impl<'s, 'r, C, P: Copy> QualifiedMultiple<'s, 'r, C, P> {
    fn adjacent(
        self,
        work: &mut StopWork<'_, '_, '_>,
    ) -> Result<ColorDriver<'s, 'r, C, P>, PaintError> {
        let first = work.visit(|| self.first)?;
        let (second, tail) = match self.middle.split_first() {
            Some((second, rest)) => (
                work.visit(|| *second)?,
                ColorTail::Remaining {
                    middle: rest,
                    last: self.last,
                },
            ),
            None => (work.visit(|| self.last)?, ColorTail::Done),
        };
        Ok(ColorDriver {
            first,
            second,
            tail,
        })
    }
}

fn lex_less(
    work: &mut StopWork<'_, '_, '_>,
    a: GradientPosition,
    b: GradientPosition,
) -> Result<bool, PaintError> {
    if work.numeric(|| a.fraction < b.fraction)? {
        Ok(true)
    } else if work.numeric(|| a.fraction == b.fraction)? {
        work.numeric(|| a.length < b.length)
    } else {
        Ok(false)
    }
}

#[derive(Clone, Copy)]
struct Normalized {
    value: f64,
    // Exact order in the chosen dimension, before floating normalization. For a
    // nonconstant omitted run, every adjacent occurrence is strictly separated.
    strict: bool,
}
impl ScalarPosition for Normalized {
    fn scalar(self) -> f64 {
        self.value
    }
    fn with_scalar(self, value: f64) -> Self {
        Self { value, ..self }
    }
}

fn loss<C, P, Q>(left: QualifiedColor<'_, C, P>, right: QualifiedColor<'_, C, Q>) -> PaintError {
    ZeroHeightGradientError::UnrepresentableWeight {
        left_source_index: left.occurrence.source_index(),
        left_position_index: left.occurrence.position_index(),
        right_source_index: right.occurrence.source_index(),
        right_position_index: right.occurrence.position_index(),
    }
    .into()
}
fn weighted_color<'s, C, P>(
    work: &mut StopWork<'_, '_, '_>,
    color: QualifiedColor<'s, C, P>,
    weight: f64,
) -> Result<WeightedColor<'s, C>, PaintError> {
    work.output(|| WeightedColor {
        occurrence: color.occurrence,
        weight,
    })
}
fn uniform<'s, C>(
    work: &mut StopWork<'_, '_, '_>,
    colors: QualifiedMultiple<'s, '_, C, ()>,
    weights: &mut Vec<WeightedColor<'s, C>>,
    count: usize,
) -> Result<[WeightedColor<'s, C>; 2], PaintError> {
    let denominator = work.numeric(|| (count - 1) as f64)?;
    let full = work.numeric(|| 1.0 / denominator)?;
    let half = work.numeric(|| full * 0.5)?;
    let first = work.visit(|| colors.first)?;
    let first = weighted_color(work, first, half)?;
    for color in colors.middle {
        let color = work.visit(|| *color)?;
        let color = weighted_color(work, color, full)?;
        weights.push(color);
    }
    let last = work.visit(|| colors.last)?;
    Ok([first, weighted_color(work, last, half)?])
}
struct AdjacentColors<'s, C> {
    penultimate: QualifiedColor<'s, C, Normalized>,
    last: QualifiedColor<'s, C, Normalized>,
    incoming: f64,
}
fn interval<C>(
    work: &mut StopWork<'_, '_, '_>,
    left: QualifiedColor<'_, C, Normalized>,
    right: QualifiedColor<'_, C, Normalized>,
) -> Result<f64, PaintError> {
    let distance = work.numeric(|| right.position.value - left.position.value)?;
    if !work.numeric(|| distance.is_finite())? || work.numeric(|| distance < 0.0)? {
        return Err(loss(left, right));
    }
    if right.position.strict && work.numeric(|| distance == 0.0)? {
        return Err(loss(left, right));
    }
    Ok(distance)
}
fn half_interval<C>(
    work: &mut StopWork<'_, '_, '_>,
    distance: f64,
    left: QualifiedColor<'_, C, Normalized>,
    right: QualifiedColor<'_, C, Normalized>,
) -> Result<f64, PaintError> {
    let weight = work.numeric(|| distance * 0.5)?;
    if work.numeric(|| distance > 0.0)? && work.numeric(|| weight == 0.0)? {
        return Err(loss(left, right));
    }
    Ok(weight)
}
fn weighted<'s, C>(
    work: &mut StopWork<'_, '_, '_>,
    colors: QualifiedMultiple<'s, '_, C, Normalized>,
    weights: &mut Vec<WeightedColor<'s, C>>,
) -> Result<[WeightedColor<'s, C>; 2], PaintError> {
    let ColorDriver {
        first,
        second,
        mut tail,
    } = colors.adjacent(work)?;
    let incoming = interval(work, first, second)?;
    let first_weight = half_interval(work, incoming, first, second)?;
    let first_weight = weighted_color(work, first, first_weight)?;
    let mut adjacent = AdjacentColors {
        penultimate: first,
        last: second,
        incoming,
    };
    while let Some(next) = tail.next(work)? {
        let distance = interval(work, adjacent.last, next)?;
        let combined = work.numeric(|| adjacent.incoming + distance)?;
        let weight = half_interval(work, combined, adjacent.last, next)?;
        let color = weighted_color(work, adjacent.last, weight)?;
        weights.push(color);
        adjacent = AdjacentColors {
            penultimate: adjacent.last,
            last: next,
            incoming: distance,
        };
    }
    let last_weight = half_interval(work, adjacent.incoming, adjacent.penultimate, adjacent.last)?;
    Ok([
        first_weight,
        weighted_color(work, adjacent.last, last_weight)?,
    ])
}

// This module owns the numeric construction boundary. A varying span is issued
// only from the finite, ordered endpoints of the fixed input; its interior
// intervals can be issued only from that input's positioned coordinates.
mod projection {
    use super::{
        FixedInput, GradientPosition, GradientStopItem, Normalized, PaintError, StopWork,
        UsedGradientRecord, interior_omitted,
    };

    // Constructed only after this span's own walk excludes interior omissions.
    struct FixedPosition(GradientPosition);

    #[derive(Clone, Copy)]
    enum Axis {
        Fraction,
        Length,
    }
    impl Axis {
        fn selected(self, position: GradientPosition) -> f64 {
            match self {
                Self::Fraction => position.fraction,
                Self::Length => position.length,
            }
        }
    }
    pub(super) enum Projection<'s, 'r, C> {
        Uniform,
        Varying(VaryingSpan<'s, 'r, C>),
    }
    impl<'s, 'r, C> Projection<'s, 'r, C> {
        pub(super) fn for_fixed(
            work: &mut StopWork<'_, '_, '_>,
            fixed: &FixedInput<'s, 'r, C>,
        ) -> Result<Self, PaintError> {
            let first = work.record(fixed.pairs, 0)?.position;
            let last = work.record(fixed.pairs, fixed.pairs.len() - 1)?.position;
            let axis = if work.numeric(|| first.fraction != last.fraction)? {
                Axis::Fraction
            } else if work.numeric(|| first.length != last.length)? {
                Axis::Length
            } else {
                return Ok(Self::Uniform);
            };
            let endpoints = OrderedInterval {
                a: axis.selected(first),
                b: axis.selected(last),
            };
            Ok(Self::Varying(VaryingSpan {
                source: fixed.source,
                pairs: fixed.pairs,
                axis,
                span: endpoints.distance(work)?,
                endpoints,
            }))
        }
    }
    pub(super) struct VaryingSpan<'s, 'r, C> {
        source: &'s [GradientStopItem<C>],
        pairs: &'r [UsedGradientRecord<GradientPosition>],
        axis: Axis,
        endpoints: OrderedInterval,
        span: Distance,
    }
    enum ProjectionInput {
        First,
        Last,
        Interior(OrderedInterval),
    }
    impl<C> VaryingSpan<'_, '_, C> {
        pub(super) fn project(
            &self,
            work: &mut StopWork<'_, '_, '_>,
            normalized: &mut Vec<UsedGradientRecord<Normalized>>,
        ) -> Result<(), PaintError> {
            let source = self.source;
            let pairs = self.pairs;
            let mut previous: Option<(usize, f64)> = None;
            for index in 0..pairs.len() {
                let record = work.record(pairs, index)?;
                work.emit(
                    normalized,
                    UsedGradientRecord {
                        source_index: record.source_index,
                        position_index: record.position_index,
                        position: Normalized {
                            value: 0.0,
                            strict: false,
                        },
                    },
                )?;
                if interior_omitted(work, source, record, index, pairs.len())? {
                    continue;
                }
                let value = self.selected(record.position);
                if let Some((left, previous_value)) = previous {
                    let strict = work.numeric(|| previous_value < value)?;
                    for slot in left + 1..=index {
                        work.record_mut(normalized, slot)?.position.strict = strict;
                    }
                }
                previous = Some((index, value));
                let hint = matches!(
                    work.source(source, record.source_index)?,
                    GradientStopItem::Hint { .. }
                );
                let needed = if hint {
                    let left = work.record(pairs, index - 1)?;
                    if interior_omitted(work, source, left, index - 1, pairs.len())? {
                        true
                    } else {
                        let right = work.record(pairs, index + 1)?;
                        interior_omitted(work, source, right, index + 1, pairs.len())?
                    }
                } else {
                    true
                };
                if needed {
                    let value = self.normalize(work, FixedPosition(record.position))?;
                    work.record_mut(normalized, index)?.position.value = value;
                }
            }
            Ok(())
        }

        fn selected(&self, position: GradientPosition) -> f64 {
            self.axis.selected(position)
        }
        fn input(
            &self,
            work: &mut StopWork<'_, '_, '_>,
            position: FixedPosition,
        ) -> Result<ProjectionInput, PaintError> {
            let value = self.selected(position.0);
            if work.numeric(|| value == self.endpoints.a)? {
                return Ok(ProjectionInput::First);
            }
            if work.numeric(|| value == self.endpoints.b)? {
                return Ok(ProjectionInput::Last);
            }
            Ok(ProjectionInput::Interior(OrderedInterval {
                a: self.endpoints.a,
                b: value,
            }))
        }
        fn normalize(
            &self,
            work: &mut StopWork<'_, '_, '_>,
            position: FixedPosition,
        ) -> Result<f64, PaintError> {
            match self.input(work, position)? {
                ProjectionInput::First => Ok(0.0),
                ProjectionInput::Last => Ok(1.0),
                ProjectionInput::Interior(interval) => {
                    interval.distance(work)?.ratio(work, self.span)
                }
            }
        }
    }
    // Both endpoints are finite and a < b. The fixed owner and its varying span
    // are the sole production issuers, including intervals to interior positions.
    struct OrderedInterval {
        a: f64,
        b: f64,
    }
    impl OrderedInterval {
        fn distance(&self, work: &mut StopWork<'_, '_, '_>) -> Result<Distance, PaintError> {
            // Only the positive finite subtraction (or finite positive half-span)
            // reaches this decoder. Scaling a positive subnormal by 2^52 is
            // always normal, so no recursion or zero-sensitive constructor exists.
            fn decode(work: &mut StopWork<'_, '_, '_>, value: f64) -> Result<Distance, PaintError> {
                let bits = work.numeric(|| value.to_bits())?;
                let raw = work.numeric(|| (bits >> 52) & 0x7ff)?;
                let (bits, raw, bias) = if work.numeric(|| raw == 0)? {
                    let scaled = work.numeric(|| value * 4_503_599_627_370_496.0)?;
                    let bits = work.numeric(|| scaled.to_bits())?;
                    let raw = work.numeric(|| (bits >> 52) & 0x7ff)?;
                    (bits, raw, 1075)
                } else {
                    (bits, raw, 1023)
                };
                let fraction = work.numeric(|| bits & 0x000f_ffff_ffff_ffff)?;
                let normal_bits = work.numeric(|| fraction | 0x3ff0_0000_0000_0000)?;
                let mantissa = work.numeric(|| f64::from_bits(normal_bits))?;
                let raw = work.numeric(|| raw as i32)?;
                let exponent = work.numeric(|| raw - bias)?;
                Ok(Distance { mantissa, exponent })
            }
            let difference = work.numeric(|| self.b - self.a)?;
            if work.numeric(|| difference.is_finite())? {
                decode(work, difference)
            } else {
                let a_half = work.numeric(|| self.a * 0.5)?;
                let b_half = work.numeric(|| self.b * 0.5)?;
                let half = work.numeric(|| b_half - a_half)?;
                let mut result = decode(work, half)?;
                result.exponent = work.numeric(|| result.exponent + 1)?;
                Ok(result)
            }
        }
    }
    /// Positive distances retain their full exponent range, including 1024 for
    /// overflowing whole spans and -1074 for the smallest positive subnormal.
    #[derive(Clone, Copy)]
    struct Distance {
        mantissa: f64,
        exponent: i32,
    }
    impl Distance {
        fn ratio(
            self,
            work: &mut StopWork<'_, '_, '_>,
            denominator: Self,
        ) -> Result<f64, PaintError> {
            let quotient = work.numeric(|| self.mantissa / denominator.mantissa)?;
            let exponent = work.numeric(|| self.exponent - denominator.exponent)?;
            if work.numeric(|| exponent < -1075)? {
                return Ok(0.0);
            }
            // Split the final multiplication only in the subnormal output range.
            // Both factors before that final rounding are normal.
            if work.numeric(|| exponent < -1022)? {
                let first_exponent = work.numeric(|| exponent + 1022)?;
                let factor = power(work, first_exponent)?;
                let partial = work.numeric(|| quotient * factor)?;
                work.numeric(|| partial * f64::MIN_POSITIVE)
            } else {
                let factor = power(work, exponent)?;
                work.numeric(|| quotient * factor)
            }
        }
    }
    fn power(work: &mut StopWork<'_, '_, '_>, exponent: i32) -> Result<f64, PaintError> {
        let biased = work.numeric(|| exponent + 1023)?;
        let biased = work.numeric(|| biased as u64)?;
        let bits = work.numeric(|| biased << 52)?;
        work.numeric(|| f64::from_bits(bits))
    }
    #[cfg(test)]
    pub(super) fn normal_distance(
        work: &mut StopWork<'_, '_, '_>,
    ) -> Result<(f64, i32), PaintError> {
        let value = OrderedInterval { a: 0.0, b: 1.5 }.distance(work)?;
        Ok((value.mantissa, value.exponent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GradientStopPositions, PaintChange, PaintLimits, ProfileMode, RadialGradientSize,
        resolve_radial_gradient,
    };
    use surgeist_layout::{Point, Size};

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }
    fn geometry() -> RadialGradientGeometry {
        resolve_radial_gradient(
            Size::new(40.0, 20.0),
            Some(Point::new(0.0, 0.0)),
            RadialGradientSize::EllipseRadii(Size::new(
                GradientPosition {
                    fraction: 0.0,
                    length: 20.0,
                },
                GradientPosition {
                    fraction: 0.0,
                    length: 0.0,
                },
            )),
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap()
    }
    fn stop(value: f64, fraction: bool) -> GradientStopItem<()> {
        GradientStopItem::Stop {
            payload: (),
            positions: GradientStopPositions::One(if fraction {
                GradientPosition {
                    fraction: value,
                    length: 0.0,
                }
            } else {
                GradientPosition {
                    fraction: 0.0,
                    length: value,
                }
            }),
        }
    }
    fn normal_decode(limits: PaintLimits) -> (Result<(f64, i32), PaintError>, PaintReport) {
        let mut meter = options(limits)
            .operation(OperationKind::ZeroHeightGradient)
            .unwrap();
        let scopes = setup_scopes(&mut meter).unwrap();
        let result = meter
            .with_phase(scopes[3], |p| {
                projection::normal_distance(&mut StopWork::new(p, scopes[3]))
            })
            .unwrap();
        (result, meter.finish().unwrap().unwrap())
    }

    #[test]
    fn normal_distance_decode_admits_each_action_and_preserves_refusal_prefix() {
        let (value, report) = normal_decode(PaintLimits::unlimited());
        let value = value.unwrap();
        assert_eq!(value, (1.5, 0));
        // to_bits, exponent-field extraction, normal/subnormal inspection,
        // fraction mask, normal-bit composition, from_bits, exponent conversion,
        // and bias subtraction: eight decoder actions. Ordered subtraction and
        // finite-span inspection add two, with no record walk.
        assert_eq!(report.work(WorkKind::GradientCoordinate), 10);
        assert_eq!(report.work(WorkKind::GradientStopVisit), 0);
        let (value, report) = normal_decode(
            PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 9),
        );
        let Err(PaintError::Budget(refusal)) = value else {
            panic!("tenth scalar must refuse")
        };
        assert_eq!(refusal.current, 9);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::GradientCoordinate,
                amount: 1,
            }
        );
        assert_eq!(report.work(WorkKind::GradientCoordinate), 9);
        for &resource in ResourceKind::ALL {
            assert_eq!(report.live(resource), 0);
        }
        assert_eq!(normal_decode(PaintLimits::unlimited()).0.unwrap().1, 0);
    }

    #[test]
    fn normalized_omissions_succeed_without_raw_tiny_or_overflowing_coordinates() {
        for (fraction, a, b) in [
            (true, 0.0, f64::from_bits(1)),
            (false, 0.0, f64::from_bits(1)),
            (false, -f64::MAX, f64::MAX),
        ] {
            let source = [
                stop(a, fraction),
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Omitted,
                },
                GradientStopItem::Stop {
                    payload: (),
                    positions: GradientStopPositions::Omitted,
                },
                stop(b, fraction),
            ];
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                GradientRepeat::Repeat,
                options(PaintLimits::unlimited()),
            );
            let colors = outcome.result.unwrap();
            for (actual, expected) in
                colors
                    .colors()
                    .map(|c| c.weight)
                    .zip([1.0 / 6.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0])
            {
                assert!((actual - expected).abs() <= 2.0 * f64::EPSILON);
            }
            assert_eq!(outcome.report.unwrap().live(ResourceKind::ScratchBytes), 0);
        }
    }

    #[test]
    fn two_color_work_counts_actual_qualification_and_weighting_actions() {
        let uniform = [GradientStopItem::Stop {
            payload: (),
            positions: GradientStopPositions::Two(
                GradientPosition {
                    fraction: 0.0,
                    length: 0.0,
                },
                GradientPosition {
                    fraction: 0.0,
                    length: 0.0,
                },
            ),
        }];
        let varying = [stop(0.0, false), stop(10.0, false)];
        for (source, validations, visits, coordinates, outputs) in [
            (uniform.as_slice(), 7, 14, 8, 7),
            (varying.as_slice(), 8, 29, 29, 9),
        ] {
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                source,
                GradientRepeat::Repeat,
                options(PaintLimits::unlimited()),
            );
            assert_eq!(
                outcome
                    .result
                    .unwrap()
                    .colors()
                    .map(|color| color.weight)
                    .collect::<Vec<_>>(),
                [0.5, 0.5]
            );
            let report = outcome.report.unwrap();
            // Uniform: validation/source1; expansion/source1; fixup8 visits,
            // 3 lexicographic comparisons; distribution2 endpoint reads,
            // no source rewalk, 2 qualified reads, 2 dimension comparisons
            // and 3 uniform arithmetic operations. Qualification emits both
            // inline endpoints, weighting emits both endpoints, publication1.
            // Varying: validation/source2; expansion/source2; fixup8 visits,
            // 3 comparisons; distribution2 endpoint reads, 9 projection visits,
            // 2 distribution visits, 2 qualification visits, 2 qualified reads.
            // Its coordinates are 2 dimension + 10 span + 4 projection + 10
            // weighting actions. Expansion2 + normalization2 + qualification2
            // + weighting2 + publication1 gives 9 outputs.
            assert_eq!(report.work(WorkKind::GradientValidation), validations);
            assert_eq!(report.work(WorkKind::GradientStopVisit), visits);
            assert_eq!(report.work(WorkKind::GradientCoordinate), coordinates);
            assert_eq!(report.work(WorkKind::OutputRecord), outputs);
        }
    }

    #[test]
    fn qualification_and_first_weight_refusals_release_both_scratch_and_result_backing() {
        let varying = [stop(0.0, false), stop(10.0, false), stop(40.0, false)];
        let uniform = [stop(0.0, false), stop(0.0, false), stop(0.0, false)];
        // Expansion emits three records. Varying projection emits three more.
        // Qualification then emits its first, middle and final records before
        // any weighted output. These caps refuse at middle, final and weight1.
        for (source, first_cap, expected) in [
            (&uniform, 4, [0.25, 0.5, 0.25]),
            (&varying, 7, [0.125, 0.5, 0.375]),
        ] {
            for cap in first_cap..first_cap + 3 {
                let refused = resolve_zero_height_gradient_colors(
                    &geometry(),
                    source,
                    GradientRepeat::Repeat,
                    options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, cap)),
                );
                let Err(PaintError::Budget(refusal)) = refused.result else {
                    panic!("qualification or first weight must refuse")
                };
                assert_eq!(refusal.current, cap);
                assert_eq!(
                    refusal.change,
                    PaintChange::Work {
                        kind: WorkKind::OutputRecord,
                        amount: 1
                    }
                );
                assert_eq!(refusal.origin, PaintPhase::GradientStopDistribution);
                let report = refused.report.unwrap();
                assert_eq!(report.work(WorkKind::OutputRecord), cap);
                for &resource in ResourceKind::ALL {
                    assert_eq!(report.live(resource), 0);
                }
                let retry = resolve_zero_height_gradient_colors(
                    &geometry(),
                    source,
                    GradientRepeat::Repeat,
                    options(PaintLimits::unlimited()),
                )
                .result
                .unwrap();
                assert_eq!(
                    retry.colors().map(|color| color.weight).collect::<Vec<_>>(),
                    expected
                );
            }
        }
    }

    #[test]
    fn last_and_two_color_results_admit_the_complete_inline_owner_without_a_heap() {
        let source = [stop(0.0, false), stop(10.0, false)];
        let fixed = bytes::<ZeroHeightGradientColors<'_, ()>>(1).unwrap();
        for repeat in [GradientRepeat::None, GradientRepeat::Repeat] {
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                repeat,
                options(
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::RetainedResultBytes, fixed),
                ),
            );
            let result = outcome.result.unwrap();
            if let Measure::Average { middle, .. } = &result.measure {
                assert_eq!(middle.capacity(), 0);
            }
            let report = outcome.report.unwrap();
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), fixed);
            assert_eq!(report.peak(ResourceKind::RetainedResultBytes), fixed);
            assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
            let refused = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                repeat,
                options(
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::RetainedResultBytes, fixed - 1),
                ),
            );
            assert!(matches!(refused.result, Err(PaintError::Budget(_))));
            let report = refused.report.unwrap();
            for &resource in ResourceKind::ALL {
                assert_eq!(report.live(resource), 0);
            }
        }
    }

    #[test]
    fn actual_result_capacity_and_late_publication_refusals_release_every_hold() {
        let source = [stop(0.0, false), stop(10.0, false), stop(40.0, false)];
        let baseline = resolve_zero_height_gradient_colors(
            &geometry(),
            &source,
            GradientRepeat::Repeat,
            options(PaintLimits::unlimited()),
        );
        let report = baseline.report.unwrap();
        let result = baseline.result.unwrap();
        let Measure::Average { middle, .. } = &result.measure else {
            panic!("three-color measure")
        };
        let expected = bytes::<WeightedColor<'_, ()>>(middle.capacity()).unwrap()
            + bytes::<ZeroHeightGradientColors<'_, ()>>(1).unwrap();
        assert_eq!(report.live(ResourceKind::RetainedResultBytes), expected);
        assert!(report.peak(ResourceKind::ScratchBytes) > 0);
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        let work = report.work(WorkKind::OutputRecord);
        for limits in [
            PaintLimits::unlimited()
                .with_resource_limit(ResourceKind::RetainedResultBytes, expected - 1),
            PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, work - 1),
        ] {
            let outcome = resolve_zero_height_gradient_colors(
                &geometry(),
                &source,
                GradientRepeat::Repeat,
                options(limits),
            );
            assert!(matches!(outcome.result, Err(PaintError::Budget(_))));
            let report = outcome.report.unwrap();
            for &resource in ResourceKind::ALL {
                assert_eq!(report.live(resource), 0);
            }
        }
        let retry = resolve_zero_height_gradient_colors(
            &geometry(),
            &source,
            GradientRepeat::Repeat,
            options(PaintLimits::unlimited()),
        )
        .result
        .unwrap();
        assert_eq!(
            retry.colors().map(|c| c.weight).collect::<Vec<_>>(),
            [0.125, 0.5, 0.375]
        );
    }
}

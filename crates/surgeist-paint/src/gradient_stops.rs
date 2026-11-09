use crate::{
    PaintError, PaintOptions, PaintPhase, PaintReport, ResourceKind, WorkKind,
    accounting::{
        Meter, OperationKind, PhaseMeter, allocate, apply, bytes, charge, finish, release,
        setup_error,
    },
};
use std::mem::take;
use surgeist_ledger::{Change, Limits, Scope, Tariffs};

/// Raw, already-computed position contributions. A fraction of 1 means 100%.
/// The operation validates both components, including when its line length is zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientPosition {
    pub length: f64,
    pub fraction: f64,
}

/// Authored omission and positional cardinality retained until used resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientStopPositions {
    Omitted,
    One(GradientPosition),
    Two(GradientPosition, GradientPosition),
}

/// One native source item. Paint never interprets, clones, or validates C.
#[derive(Debug)]
pub enum GradientStopItem<C> {
    Stop {
        payload: C,
        positions: GradientStopPositions,
    },
    Hint {
        position: GradientPosition,
    },
}

/// The invalid scalar contribution in a native position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientPositionComponent {
    Length,
    Fraction,
}

/// Source-indexed semantic or numerical refusal. Hint diagnostics use ordinal 0.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientStopsError {
    InvalidLineLength,
    EmptyList,
    EndpointHint {
        source_index: usize,
    },
    AdjacentHints {
        previous_source_index: usize,
        source_index: usize,
    },
    NonFiniteComponent {
        source_index: usize,
        position_index: usize,
        component: GradientPositionComponent,
    },
    UnrepresentablePosition {
        source_index: usize,
        position_index: usize,
    },
    UnrepresentableDistribution {
        left_source_index: usize,
        right_source_index: usize,
    },
}

/// A resolved item view. Indices identify the exact borrowed native source list.
/// Stop ordinals are 0 for omitted/one positions and 0/1 for two positions.
#[derive(Debug)]
pub enum UsedGradientItem<'a, C> {
    Stop {
        source_index: usize,
        position_index: usize,
        payload: &'a C,
        position: f64,
    },
    Hint {
        source_index: usize,
        position: f64,
    },
}
impl<C> Copy for UsedGradientItem<'_, C> {}
impl<C> Clone for UsedGradientItem<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct UsedGradientRecord<P = f64> {
    pub(crate) source_index: usize,
    pub(crate) position_index: usize,
    pub(crate) position: P,
}

/// Complete finite, nondecreasing used positions borrowing their exact source.
/// Equal, negative and outside-line positions remain distinct. Color semantics
/// and caller traversal after return are outside this operation's report.
#[derive(Debug)]
pub struct GradientStops<'a, C> {
    source: &'a [GradientStopItem<C>],
    line_length: f64,
    records: Vec<UsedGradientRecord>,
}
impl<'a, C> GradientStops<'a, C> {
    pub const fn source(&self) -> &'a [GradientStopItem<C>] {
        self.source
    }
    pub const fn line_length(&self) -> f64 {
        self.line_length
    }
    pub fn items(&self) -> impl ExactSizeIterator<Item = UsedGradientItem<'_, C>> {
        self.records
            .iter()
            .map(|record| match &self.source[record.source_index] {
                GradientStopItem::Stop { payload, .. } => UsedGradientItem::Stop {
                    source_index: record.source_index,
                    position_index: record.position_index,
                    payload,
                    position: record.position,
                },
                GradientStopItem::Hint { .. } => UsedGradientItem::Hint {
                    source_index: record.source_index,
                    position: record.position,
                },
            })
    }
}

/// One complete result or typed refusal, and the operation's accepted prefix.
#[derive(Debug)]
pub struct GradientStopsOutcome<'a, C> {
    pub result: Result<GradientStops<'a, C>, PaintError>,
    pub report: Option<PaintReport>,
}

/// Resolves native stop/hint positions in the physical unit of `line_length`.
/// This does not evaluate authored CSS, interpolate colors, or normalize positions.
/// The borrowed result cannot outlive its source storage.
///
/// ```compile_fail
/// use surgeist_paint::{GradientStopItem, GradientStopPositions, PaintLimits,
///     PaintOptions, ProfileMode, resolve_gradient_stops};
/// let stops = {
///     let source = [GradientStopItem::Stop {
///         payload: (), positions: GradientStopPositions::Omitted,
///     }];
///     resolve_gradient_stops(100.0, &source,
///         PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)).result.unwrap()
/// };
/// let _ = stops.items().count();
/// ```
pub fn resolve_gradient_stops<'a, C>(
    line_length: f64,
    source: &'a [GradientStopItem<C>],
    options: PaintOptions,
) -> GradientStopsOutcome<'a, C> {
    let mut meter = match options.operation(OperationKind::GradientStops) {
        Ok(meter) => meter,
        Err(error) => {
            return GradientStopsOutcome {
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
            return GradientStopsOutcome { result, report };
        }
    };
    let mut records = Vec::new();
    let mut fixed_owner = false;
    let result = (|| {
        let count = meter
            .with_phase(scopes[0], |p| {
                validate(&mut StopWork::new(p, scopes[0]), line_length, source)
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[1], |p| {
                records = allocate(p, root, count, ResourceKind::RetainedResultBytes)?;
                expand(
                    &mut StopWork::new(p, scopes[1]),
                    line_length,
                    source,
                    &mut records,
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[2], |p| {
                fixup(
                    &mut StopWork::new(p, scopes[2]),
                    line_length,
                    source,
                    &mut records,
                )
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[3], |p| {
                let mut work = StopWork::new(p, scopes[3]);
                distribute(&mut work, source, &mut records)?;
                qualify(&mut work, &records)
            })
            .map_err(setup_error)??;
        meter
            .with_phase(scopes[4], |p| {
                apply(
                    p,
                    root,
                    Change::Acquire(
                        ResourceKind::RetainedResultBytes,
                        bytes::<GradientStops<'_, C>>(1)?,
                    ),
                )?;
                fixed_owner = true;
                charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                Ok(GradientStops {
                    source,
                    line_length,
                    records: take(&mut records),
                })
            })
            .map_err(setup_error)?
    })();
    let cleanup = meter
        .with_phase(scopes[5], |p| {
            if result.is_err() {
                release(
                    p,
                    root,
                    take(&mut records),
                    ResourceKind::RetainedResultBytes,
                )?;
                if fixed_owner {
                    apply(
                        p,
                        root,
                        Change::Release(
                            ResourceKind::RetainedResultBytes,
                            bytes::<GradientStops<'_, C>>(1)?,
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
    GradientStopsOutcome { result, report }
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

pub(crate) struct StopWork<'a, 'b, 'operation> {
    meter: &'a mut PhaseMeter<'b, 'operation>,
    scope: Scope,
}
impl<'a, 'b, 'operation> StopWork<'a, 'b, 'operation> {
    pub(crate) fn new(meter: &'a mut PhaseMeter<'b, 'operation>, scope: Scope) -> Self {
        Self { meter, scope }
    }
    pub(crate) fn meter(&mut self) -> &mut PhaseMeter<'b, 'operation> {
        self.meter
    }
    // A validation unit is one scalar's complete domain check, one emptiness
    // check, or one source item's grammar/discriminant validation action.
    pub(crate) fn validation<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientValidation, 1)?;
        Ok(action())
    }
    // A numeric unit is one scalar calculation/conversion/comparison/inspection;
    // FMA is one operation. Repeated inspections charge again. Scalar copies,
    // boolean control and record assembly are not extra numeric operations.
    pub(crate) fn numeric<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientCoordinate, 1)?;
        Ok(action())
    }
    // Each source access or existing record read/write is one actual visit,
    // including separate walks and boundary reinspections. Its constant-size
    // index/ordinal bookkeeping belongs to this unit. No access precedes charge.
    pub(crate) fn source<'s, C>(
        &mut self,
        source: &'s [GradientStopItem<C>],
        index: usize,
    ) -> Result<&'s GradientStopItem<C>, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientStopVisit, 1)?;
        Ok(&source[index])
    }
    pub(crate) fn record<P: Copy>(
        &mut self,
        records: &[UsedGradientRecord<P>],
        index: usize,
    ) -> Result<UsedGradientRecord<P>, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientStopVisit, 1)?;
        Ok(records[index])
    }
    pub(crate) fn write<P>(
        &mut self,
        records: &mut [UsedGradientRecord<P>],
        index: usize,
        position: P,
    ) -> Result<(), PaintError> {
        self.record_mut(records, index)?.position = position;
        Ok(())
    }
    pub(crate) fn record_mut<'r, P>(
        &mut self,
        records: &'r mut [UsedGradientRecord<P>],
        index: usize,
    ) -> Result<&'r mut UsedGradientRecord<P>, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientStopVisit, 1)?;
        Ok(&mut records[index])
    }
    pub(crate) fn emit<P>(
        &mut self,
        records: &mut Vec<UsedGradientRecord<P>>,
        record: UsedGradientRecord<P>,
    ) -> Result<(), PaintError> {
        // One new retained expanded record; later fixup writes are not new records.
        charge(self.meter, self.scope, WorkKind::OutputRecord, 1)?;
        records.push(record);
        Ok(())
    }
}

fn validate<C>(
    work: &mut StopWork<'_, '_, '_>,
    line_length: f64,
    source: &[GradientStopItem<C>],
) -> Result<usize, PaintError> {
    if !work.validation(|| line_length.is_finite() && line_length >= 0.0)? {
        return Err(GradientStopsError::InvalidLineLength.into());
    }
    Ok(validate_source(work, source)?.records)
}

pub(crate) struct ValidatedStops {
    pub(crate) records: usize,
    pub(crate) colors: usize,
    pub(crate) last_ordinal: usize,
}

pub(crate) fn validate_source<C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
) -> Result<ValidatedStops, PaintError> {
    if work.validation(|| source.is_empty())? {
        return Err(GradientStopsError::EmptyList.into());
    }
    let mut count = 0usize;
    let mut previous_hint = false;
    let mut colors = 0usize;
    let mut last_ordinal = 0usize;
    for source_index in 0..source.len() {
        let item = work.source(source, source_index)?;
        previous_hint = work.validation(|| match item {
            GradientStopItem::Hint { .. }
                if source_index == 0 || source_index == source.len() - 1 =>
            {
                Err(GradientStopsError::EndpointHint { source_index })
            }
            GradientStopItem::Hint { .. } if previous_hint => {
                Err(GradientStopsError::AdjacentHints {
                    previous_source_index: source_index - 1,
                    source_index,
                })
            }
            GradientStopItem::Hint { .. } => Ok(true),
            GradientStopItem::Stop { .. } => Ok(false),
        })??;
        let positions = match item {
            GradientStopItem::Stop { positions, .. } => *positions,
            GradientStopItem::Hint { position } => GradientStopPositions::One(*position),
        };
        match positions {
            GradientStopPositions::Omitted => {}
            GradientStopPositions::One(position) => {
                validate_position(work, source_index, 0, position)?
            }
            GradientStopPositions::Two(first, second) => {
                validate_position(work, source_index, 0, first)?;
                validate_position(work, source_index, 1, second)?;
            }
        }
        let amount = match positions {
            GradientStopPositions::Two(..) => 2,
            _ => 1,
        };
        count = count
            .checked_add(amount)
            .ok_or(PaintError::CapacityOverflow)?;
        if !previous_hint {
            colors = colors
                .checked_add(amount)
                .ok_or(PaintError::CapacityOverflow)?;
            last_ordinal = amount - 1;
        }
    }
    Ok(ValidatedStops {
        records: count,
        colors,
        last_ordinal,
    })
}

fn validate_position(
    work: &mut StopWork<'_, '_, '_>,
    source_index: usize,
    position_index: usize,
    position: GradientPosition,
) -> Result<(), PaintError> {
    if !work.validation(|| position.length.is_finite())? {
        return Err(GradientStopsError::NonFiniteComponent {
            source_index,
            position_index,
            component: GradientPositionComponent::Length,
        }
        .into());
    }
    if !work.validation(|| position.fraction.is_finite())? {
        return Err(GradientStopsError::NonFiniteComponent {
            source_index,
            position_index,
            component: GradientPositionComponent::Fraction,
        }
        .into());
    }
    Ok(())
}

fn affine(
    work: &mut StopWork<'_, '_, '_>,
    line_length: f64,
    source_index: usize,
    position_index: usize,
    position: GradientPosition,
) -> Result<f64, PaintError> {
    // FMA preserves a representable final value even if the separate product
    // would overflow before cancellation with the absolute contribution.
    let used = work.numeric(|| position.fraction.mul_add(line_length, position.length))?;
    if !work.numeric(|| used.is_finite())? {
        return Err(GradientStopsError::UnrepresentablePosition {
            source_index,
            position_index,
        }
        .into());
    }
    Ok(used)
}

fn expand<C>(
    work: &mut StopWork<'_, '_, '_>,
    line_length: f64,
    source: &[GradientStopItem<C>],
    records: &mut Vec<UsedGradientRecord>,
) -> Result<(), PaintError> {
    expand_with(
        work,
        source,
        records,
        0.0,
        |work, source_index, ordinal, position| {
            affine(work, line_length, source_index, ordinal, position)
        },
    )
}

pub(crate) fn expand_with<C, P: Copy>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    records: &mut Vec<UsedGradientRecord<P>>,
    omitted: P,
    mut resolve: impl FnMut(
        &mut StopWork<'_, '_, '_>,
        usize,
        usize,
        GradientPosition,
    ) -> Result<P, PaintError>,
) -> Result<(), PaintError> {
    for source_index in 0..source.len() {
        let positions = match work.source(source, source_index)? {
            GradientStopItem::Stop { positions, .. } => *positions,
            GradientStopItem::Hint { position } => GradientStopPositions::One(*position),
        };
        match positions {
            // This zero is unpublished backing state, not a specified position.
            // Later passes recover omission from the exact borrowed source.
            GradientStopPositions::Omitted => work.emit(
                records,
                UsedGradientRecord {
                    source_index,
                    position_index: 0,
                    position: omitted,
                },
            )?,
            GradientStopPositions::One(position) => {
                let position = resolve(work, source_index, 0, position)?;
                work.emit(
                    records,
                    UsedGradientRecord {
                        source_index,
                        position_index: 0,
                        position,
                    },
                )?;
            }
            GradientStopPositions::Two(first, second) => {
                let position = resolve(work, source_index, 0, first)?;
                work.emit(
                    records,
                    UsedGradientRecord {
                        source_index,
                        position_index: 0,
                        position,
                    },
                )?;
                let position = resolve(work, source_index, 1, second)?;
                work.emit(
                    records,
                    UsedGradientRecord {
                        source_index,
                        position_index: 1,
                        position,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) fn originally_omitted<C, P>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    record: UsedGradientRecord<P>,
) -> Result<bool, PaintError> {
    Ok(matches!(
        work.source(source, record.source_index)?,
        GradientStopItem::Stop {
            positions: GradientStopPositions::Omitted,
            ..
        }
    ))
}

pub(crate) fn interior_omitted<C, P>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    record: UsedGradientRecord<P>,
    index: usize,
    length: usize,
) -> Result<bool, PaintError> {
    let omitted = originally_omitted(work, source, record)?;
    Ok(omitted && index != 0 && index != length - 1)
}

fn fixup<C>(
    work: &mut StopWork<'_, '_, '_>,
    line_length: f64,
    source: &[GradientStopItem<C>],
    records: &mut [UsedGradientRecord],
) -> Result<(), PaintError> {
    fixup_with(work, source, records, 0.0, line_length, |work, a, b| {
        work.numeric(|| a < b)
    })
}

pub(crate) fn fixup_with<C, P: Copy>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    records: &mut [UsedGradientRecord<P>],
    first_default: P,
    last_default: P,
    mut less: impl FnMut(&mut StopWork<'_, '_, '_>, P, P) -> Result<bool, PaintError>,
) -> Result<(), PaintError> {
    let first = work.record(records, 0)?;
    if originally_omitted(work, source, first)? {
        work.write(records, 0, first_default)?;
    }
    let last_index = records.len() - 1;
    // A single omitted occurrence already received the first endpoint default.
    if last_index != 0 {
        let last = work.record(records, last_index)?;
        if originally_omitted(work, source, last)? {
            work.write(records, last_index, last_default)?;
        }
    }
    let mut maximum = None;
    for index in 0..records.len() {
        let record = work.record(records, index)?;
        if interior_omitted(work, source, record, index, records.len())? {
            continue;
        }
        let position = if let Some(previous) = maximum {
            if less(work, record.position, previous)? {
                work.write(records, index, previous)?;
                previous
            } else {
                record.position
            }
        } else {
            record.position
        };
        maximum = Some(position);
    }
    Ok(())
}

fn distribute<C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    records: &mut [UsedGradientRecord],
) -> Result<(), PaintError> {
    distribute_with(work, source, records, fill_run)
}

pub(crate) fn distribute_with<C, P: Copy>(
    work: &mut StopWork<'_, '_, '_>,
    source: &[GradientStopItem<C>],
    records: &mut [UsedGradientRecord<P>],
    mut fill: impl FnMut(
        &mut StopWork<'_, '_, '_>,
        &mut [UsedGradientRecord<P>],
        usize,
        usize,
        UsedGradientRecord<P>,
        UsedGradientRecord<P>,
    ) -> Result<(), PaintError>,
) -> Result<(), PaintError> {
    let mut index = 1;
    while index < records.len() {
        let record = work.record(records, index)?;
        if !interior_omitted(work, source, record, index, records.len())? {
            index += 1;
            continue;
        }
        let start = index;
        let left = work.record(records, start - 1)?;
        // Last is already positioned, so every omitted run has a right boundary.
        let right = loop {
            index = index.checked_add(1).ok_or(PaintError::CapacityOverflow)?;
            let candidate = work.record(records, index)?;
            if !interior_omitted(work, source, candidate, index, records.len())? {
                break candidate;
            }
        };
        fill(work, records, start, index, left, right)?;
        // The positioned right boundary is visited again by the outer walk.
    }
    Ok(())
}

pub(crate) trait ScalarPosition: Copy {
    fn scalar(self) -> f64;
    fn with_scalar(self, value: f64) -> Self;
}
impl ScalarPosition for f64 {
    fn scalar(self) -> f64 {
        self
    }
    fn with_scalar(self, value: f64) -> Self {
        value
    }
}

pub(crate) fn fill_run<P: ScalarPosition>(
    work: &mut StopWork<'_, '_, '_>,
    records: &mut [UsedGradientRecord<P>],
    start: usize,
    end: usize,
    left: UsedGradientRecord<P>,
    right: UsedGradientRecord<P>,
) -> Result<(), PaintError> {
    let a = left.position.scalar();
    let b = right.position.scalar();
    if work.numeric(|| a == b)? {
        for index in start..end {
            let record = work.record_mut(records, index)?;
            record.position = record.position.with_scalar(a);
        }
        return Ok(());
    }
    let denominator = (end - start)
        .checked_add(1)
        .ok_or(PaintError::CapacityOverflow)?;
    let denominator = work.numeric(|| denominator as f64)?;
    let negative_a = work.numeric(|| a.is_sign_negative())?;
    let negative_b = work.numeric(|| b.is_sign_negative())?;
    // Same-sign endpoints have a bounded difference. Opposite-sign endpoints
    // use bounded convex terms instead, so -MAX to +MAX needs no finite span.
    let delta = if negative_a == negative_b {
        Some(work.numeric(|| b - a)?)
    } else {
        None
    };
    for index in start..end {
        let record = work.record_mut(records, index)?;
        let ordinal = (index - start)
            .checked_add(1)
            .ok_or(PaintError::CapacityOverflow)?;
        let ordinal = work.numeric(|| ordinal as f64)?;
        let t = work.numeric(|| ordinal / denominator)?;
        let position = if let Some(delta) = delta {
            work.numeric(|| t.mul_add(delta, a))?
        } else {
            let complement = work.numeric(|| 1.0 - t)?;
            let right_part = work.numeric(|| t * b)?;
            work.numeric(|| complement.mul_add(a, right_part))?
        };
        if !work.numeric(|| position.is_finite())? {
            return Err(GradientStopsError::UnrepresentableDistribution {
                left_source_index: left.source_index,
                right_source_index: right.source_index,
            }
            .into());
        }
        record.position = record.position.with_scalar(position);
    }
    Ok(())
}

fn qualify(
    work: &mut StopWork<'_, '_, '_>,
    records: &[UsedGradientRecord],
) -> Result<(), PaintError> {
    let mut previous: Option<UsedGradientRecord> = None;
    for index in 0..records.len() {
        let record = work.record(records, index)?;
        if !work.numeric(|| record.position.is_finite())? {
            return Err(GradientStopsError::UnrepresentableDistribution {
                left_source_index: previous.map_or(record.source_index, |value| value.source_index),
                right_source_index: record.source_index,
            }
            .into());
        }
        if let Some(left) = previous
            && work.numeric(|| record.position < left.position)?
        {
            return Err(GradientStopsError::UnrepresentableDistribution {
                left_source_index: left.source_index,
                right_source_index: record.source_index,
            }
            .into());
        }
        previous = Some(record);
    }
    Ok(())
}

impl From<GradientStopsError> for PaintError {
    fn from(value: GradientStopsError) -> Self {
        Self::GradientStops(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PaintChange, PaintLimits, PaintPhase, ProfileMode};
    use std::mem::size_of;

    fn options(limits: PaintLimits) -> PaintOptions {
        PaintOptions::new(limits, ProfileMode::Summary { timing: false })
    }

    // No payload traits, and no payload bytes even though four output records
    // remain real storage. Two positions borrow the same original payload.
    struct Payload;
    fn source() -> [GradientStopItem<Payload>; 3] {
        [
            GradientStopItem::Stop {
                payload: Payload,
                positions: GradientStopPositions::Two(
                    GradientPosition {
                        length: 0.0,
                        fraction: 0.0,
                    },
                    GradientPosition {
                        length: 0.0,
                        fraction: 0.25,
                    },
                ),
            },
            GradientStopItem::Hint {
                position: GradientPosition {
                    length: 0.0,
                    fraction: 0.5,
                },
            },
            GradientStopItem::Stop {
                payload: Payload,
                positions: GradientStopPositions::Omitted,
            },
        ]
    }

    #[test]
    fn real_capacity_and_fixed_owner_are_retained_without_payload_storage() {
        let source = source();
        let outcome = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
        let plan = outcome.result.unwrap();
        let report = outcome.report.unwrap();
        assert_eq!(size_of::<Payload>(), 0);
        assert_eq!(plan.records.len(), 4);
        assert!(size_of::<UsedGradientRecord>() > 0);
        let backing = (plan.records.capacity() * size_of::<UsedGradientRecord>()) as u64;
        let fixed = size_of::<GradientStops<'_, Payload>>() as u64;
        assert_eq!(
            report.live(ResourceKind::RetainedResultBytes),
            backing + fixed
        );
        assert_eq!(
            report.peak(ResourceKind::RetainedResultBytes),
            backing + fixed
        );
        assert_eq!(report.live(ResourceKind::ScratchBytes), 0);
        assert_eq!(report.peak(ResourceKind::ScratchBytes), 0);
        assert_eq!(report.work(WorkKind::OutputRecord), 5);
        assert_eq!(
            plan.items()
                .map(|item| match item {
                    UsedGradientItem::Stop { position, .. }
                    | UsedGradientItem::Hint { position, .. } => position,
                })
                .collect::<Vec<_>>(),
            vec![0.0, 25.0, 50.0, 100.0]
        );
    }

    #[test]
    fn fixed_owner_and_later_output_refusals_release_all_admitted_custody() {
        let source = source();
        let baseline = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
        let plan = baseline.result.unwrap();
        let backing = (plan.records.capacity() * size_of::<UsedGradientRecord>()) as u64;
        let fixed = size_of::<GradientStops<'_, Payload>>() as u64;
        drop(plan);

        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(
                PaintLimits::unlimited()
                    .with_resource_limit(ResourceKind::RetainedResultBytes, backing),
            ),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("fixed owner must refuse");
        };
        assert_eq!(refusal.current, backing);
        assert_eq!(
            refusal.change,
            PaintChange::Acquire {
                kind: ResourceKind::RetainedResultBytes,
                amount: fixed,
            }
        );
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::OutputRecord), 4);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }

        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 4)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("publication must refuse");
        };
        assert_eq!(refusal.origin, PaintPhase::Publication);
        assert_eq!(refusal.current, 4);
        assert_eq!(
            refusal.change,
            PaintChange::Work {
                kind: WorkKind::OutputRecord,
                amount: 1
            }
        );
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::OutputRecord), 4);
        assert_eq!(
            report.peak(ResourceKind::RetainedResultBytes),
            backing + fixed
        );
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }
    }

    #[test]
    fn single_omission_units_and_refused_prefix_charge_only_actions_that_run() {
        let source = [GradientStopItem::Stop {
            payload: Payload,
            positions: GradientStopPositions::Omitted,
        }];
        let baseline = resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
        assert!(baseline.result.is_ok());
        let report = baseline.report.unwrap();
        // L, list emptiness, item grammar. Visits: validation1 + expansion1 +
        // first record/source/write3 + prefix record/source2 + qualification1.
        // The only numeric inspection is final finiteness; no affine/lerp runs.
        assert_eq!(report.work(WorkKind::GradientValidation), 3);
        assert_eq!(report.work(WorkKind::GradientStopVisit), 8);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 1);
        assert_eq!(report.work(WorkKind::OutputRecord), 2);

        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(PaintLimits::unlimited().with_work_limit(WorkKind::GradientStopVisit, 1)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("expansion visit must refuse");
        };
        assert_eq!(refusal.origin, PaintPhase::GradientStopResolution);
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::GradientValidation), 3);
        assert_eq!(report.work(WorkKind::GradientStopVisit), 1);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 0);
        assert_eq!(report.work(WorkKind::OutputRecord), 0);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }

        let outcome = resolve_gradient_stops(
            100.0,
            &source,
            options(PaintLimits::unlimited().with_work_limit(WorkKind::GradientCoordinate, 0)),
        );
        let Err(PaintError::Budget(refusal)) = outcome.result else {
            panic!("qualification must refuse");
        };
        assert_eq!(refusal.origin, PaintPhase::GradientStopDistribution);
        let report = outcome.report.unwrap();
        assert_eq!(report.work(WorkKind::GradientStopVisit), 8);
        assert_eq!(report.work(WorkKind::GradientCoordinate), 0);
        assert_eq!(report.work(WorkKind::OutputRecord), 1);
        for &kind in ResourceKind::ALL {
            assert_eq!(report.live(kind), 0);
        }
    }
}

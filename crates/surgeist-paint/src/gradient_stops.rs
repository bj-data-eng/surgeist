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

/// The mandatory first color occurrence of a qualified used-stop list.
/// Only Paint's resolver constructs this record; payloads remain source-borrowed.
///
/// ```compile_fail
/// use surgeist_paint::UsedGradientStop;
/// let forged = UsedGradientStop {
///     source_index: 0, position_index: 0, payload: &(), position: 0.0,
/// };
/// ```
#[derive(Debug)]
pub struct UsedGradientStop<'a, C> {
    source_index: usize,
    position_index: usize,
    payload: &'a C,
    position: f64,
}
impl<C> UsedGradientStop<'_, C> {
    pub const fn source_index(&self) -> usize {
        self.source_index
    }
    pub const fn position_index(&self) -> usize {
        self.position_index
    }
    pub const fn payload(&self) -> &C {
        self.payload
    }
    pub const fn position(&self) -> f64 {
        self.position
    }
}

/// A borrowed prepared record. Obtaining a view does not project its fields;
/// consumers can admit a visit before calling [`Self::item`].
///
/// A view cannot be constructed through a raw record or reassociated:
///
/// ```compile_fail
/// use surgeist_paint::GradientRecord;
/// fn fabricate(authentic: GradientRecord<'_, ()>) {
///     let _ = GradientRecord { ..authentic };
/// }
/// ```
///
/// Even with the source alive, a borrowed record cannot outlive its owner:
///
/// ```compile_fail
/// use surgeist_paint::{GradientStopItem, GradientStopPositions, PaintLimits,
///     PaintOptions, ProfileMode, resolve_gradient_stops};
/// let source = [GradientStopItem::Stop { payload: (), positions: GradientStopPositions::Omitted }];
/// let view = {
///     let stops = resolve_gradient_stops(1.0, &source,
///         PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)).result.unwrap();
///     stops.records().next().unwrap()
/// };
/// let _ = view.item();
/// ```
#[derive(Debug)]
pub struct GradientRecord<'a, C> {
    record: PreparedRecord<'a, C>,
}
#[derive(Debug)]
enum PreparedRecord<'a, C> {
    First(&'a UsedGradientStop<'a, C>),
    Tail(&'a UsedGradientItem<'a, C>),
}
impl<'a, C> GradientRecord<'a, C> {
    /// Projects the actual prepared record without source access or validation.
    pub fn item(&self) -> UsedGradientItem<'a, C> {
        match self.record {
            PreparedRecord::First(first) => UsedGradientItem::Stop {
                source_index: first.source_index,
                position_index: first.position_index,
                payload: first.payload,
                position: first.position,
            },
            PreparedRecord::Tail(item) => *item,
        }
    }
}

struct PreparedRecords<'a, C> {
    first: Option<&'a UsedGradientStop<'a, C>>,
    tail: std::slice::Iter<'a, UsedGradientItem<'a, C>>,
}
impl<'a, C> Iterator for PreparedRecords<'a, C> {
    type Item = GradientRecord<'a, C>;
    fn next(&mut self) -> Option<Self::Item> {
        self.first
            .take()
            .map(|first| GradientRecord {
                record: PreparedRecord::First(first),
            })
            .or_else(|| {
                self.tail.next().map(|item| GradientRecord {
                    record: PreparedRecord::Tail(item),
                })
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}
impl<C> ExactSizeIterator for PreparedRecords<'_, C> {
    fn len(&self) -> usize {
        // The tail contains nonzero-sized prepared records in an actual Vec
        // allocation, so its length plus the inline head fits usize.
        usize::from(self.first.is_some()) + self.tail.len()
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
    first: UsedGradientStop<'a, C>,
    tail: Vec<UsedGradientItem<'a, C>>,
}
impl<'a, C> GradientStops<'a, C> {
    pub const fn source(&self) -> &'a [GradientStopItem<C>] {
        self.source
    }
    pub const fn line_length(&self) -> f64 {
        self.line_length
    }
    pub fn items(&self) -> impl ExactSizeIterator<Item = UsedGradientItem<'_, C>> {
        self.records().map(|record| record.item())
    }
    /// Borrows the mandatory first color, before any consumer field inspection.
    ///
    /// ```
    /// use surgeist_paint::{GradientStopItem, GradientStopPositions, PaintLimits,
    ///     PaintOptions, ProfileMode, resolve_gradient_stops};
    /// let source = [GradientStopItem::Stop {
    ///     payload: "first", positions: GradientStopPositions::Omitted,
    /// }];
    /// let stops = resolve_gradient_stops(1.0, &source,
    ///     PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)).result.unwrap();
    /// assert_eq!(*stops.first_stop().payload(), "first");
    /// assert_eq!(stops.first_stop().position(), 0.0);
    /// assert_eq!(stops.tail().len(), 0);
    /// assert_eq!(stops.records().len(), 1);
    /// ```
    pub const fn first_stop(&self) -> &UsedGradientStop<'_, C> {
        &self.first
    }
    /// Borrows the remaining prepared records in expanded source order.
    pub fn tail(&self) -> impl ExactSizeIterator<Item = GradientRecord<'_, C>> {
        PreparedRecords {
            first: None,
            tail: self.tail.iter(),
        }
    }
    /// Borrows every physical record exactly once, including the inline first color.
    pub fn records(&self) -> impl ExactSizeIterator<Item = GradientRecord<'_, C>> {
        PreparedRecords {
            first: Some(&self.first),
            tail: self.tail.iter(),
        }
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
///
/// A result cannot be reassociated with another, equally well-typed source.
///
/// ```compile_fail
/// use surgeist_paint::{GradientStopItem, GradientStopPositions, GradientStops,
///     PaintLimits, PaintOptions, ProfileMode, resolve_gradient_stops};
/// let source = [GradientStopItem::Stop {
///     payload: (), positions: GradientStopPositions::Omitted,
/// }];
/// let replacement = [GradientStopItem::Stop {
///     payload: (), positions: GradientStopPositions::Omitted,
/// }];
/// let authentic = resolve_gradient_stops(100.0, &source,
///     PaintOptions::new(PaintLimits::unlimited(), ProfileMode::Off)).result.unwrap();
/// let reassociated = GradientStops { source: &replacement, ..authentic };
/// let _ = reassociated.items().count();
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
    let scopes = match setup_scopes(&mut meter) {
        Ok(scopes) => scopes,
        Err(error) => {
            let (result, report) = finish(meter, Err(error));
            return GradientStopsOutcome { result, report };
        }
    };
    let result = ordinary::run(&mut meter, scopes, line_length, source);
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
    pub(crate) fn visit<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::GradientStopVisit, 1)?;
        Ok(action())
    }
    pub(crate) fn output<T>(&mut self, action: impl FnOnce() -> T) -> Result<T, PaintError> {
        charge(self.meter, self.scope, WorkKind::OutputRecord, 1)?;
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
    pub(crate) fn emit<T>(&mut self, records: &mut Vec<T>, record: T) -> Result<(), PaintError> {
        // One new staging or prepared record; later fixup writes are not new records.
        charge(self.meter, self.scope, WorkKind::OutputRecord, 1)?;
        records.push(record);
        Ok(())
    }
}

/// A payload association captured only while visiting its actual authored Stop.
#[derive(Debug)]
pub(crate) struct ColorOccurrence<'s, C> {
    source_index: usize,
    ordinal: ColorOrdinal,
    payload: &'s C,
}
#[derive(Clone, Copy, Debug)]
enum ColorOrdinal {
    First,
    Second,
}
impl<C> Copy for ColorOccurrence<'_, C> {}
impl<C> Clone for ColorOccurrence<'_, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'s, C> ColorOccurrence<'s, C> {
    pub(crate) fn source_index(self) -> usize {
        self.source_index
    }
    pub(crate) fn position_index(self) -> usize {
        match self.ordinal {
            ColorOrdinal::First => 0,
            ColorOrdinal::Second => 1,
        }
    }
    pub(crate) fn payload(self) -> &'s C {
        self.payload
    }
}

pub(crate) enum ColorSourceItem<'s, C> {
    Stop {
        first: ColorOccurrence<'s, C>,
        second: Option<ColorOccurrence<'s, C>>,
        positions: GradientStopPositions,
    },
    Hint {
        position: GradientPosition,
    },
}
impl StopWork<'_, '_, '_> {
    pub(crate) fn color_source<'s, C>(
        &mut self,
        source: &'s [GradientStopItem<C>],
        source_index: usize,
    ) -> Result<ColorSourceItem<'s, C>, PaintError> {
        Ok(match self.source(source, source_index)? {
            GradientStopItem::Stop { payload, positions } => {
                let first = ColorOccurrence {
                    source_index,
                    ordinal: ColorOrdinal::First,
                    payload,
                };
                let second = match positions {
                    GradientStopPositions::Two(..) => Some(ColorOccurrence {
                        source_index,
                        ordinal: ColorOrdinal::Second,
                        payload,
                    }),
                    _ => None,
                };
                ColorSourceItem::Stop {
                    first,
                    second,
                    positions: *positions,
                }
            }
            GradientStopItem::Hint { position } => ColorSourceItem::Hint {
                position: *position,
            },
        })
    }
}

pub(crate) enum ColorArity<'s, C> {
    One(ColorOccurrence<'s, C>),
    Multiple(MultipleColors<'s, C>),
}
pub(crate) struct MultipleColors<'s, C> {
    first: ColorOccurrence<'s, C>,
    last: ColorOccurrence<'s, C>,
    count: usize,
}
impl<'s, C> MultipleColors<'s, C> {
    pub(crate) fn first(&self) -> ColorOccurrence<'s, C> {
        self.first
    }
    pub(crate) fn last(&self) -> ColorOccurrence<'s, C> {
        self.last
    }
    pub(crate) fn count(&self) -> usize {
        self.count
    }
}
impl<'s, C> ColorArity<'s, C> {
    fn append(
        self,
        first: ColorOccurrence<'s, C>,
        second: Option<ColorOccurrence<'s, C>>,
    ) -> Result<Self, PaintError> {
        let (original, count) = match self {
            Self::One(original) => (original, 1usize),
            Self::Multiple(colors) => (colors.first, colors.count),
        };
        Ok(Self::Multiple(MultipleColors {
            first: original,
            last: match second {
                Some(last) => last,
                None => first,
            },
            count: count
                .checked_add(if second.is_some() { 2 } else { 1 })
                .ok_or(PaintError::CapacityOverflow)?,
        }))
    }
    pub(crate) fn last(&self) -> ColorOccurrence<'s, C> {
        match self {
            Self::One(occurrence) => *occurrence,
            Self::Multiple(colors) => colors.last,
        }
    }
}

pub(crate) struct ValidatedStops<'s, C> {
    source: &'s [GradientStopItem<C>],
    records: usize,
    colors: ColorArity<'s, C>,
    first: FirstColors<'s, C>,
}

struct FirstColors<'s, C> {
    first: ColorOccurrence<'s, C>,
    second: Option<ColorOccurrence<'s, C>>,
}

impl<'s, C> ValidatedStops<'s, C> {
    pub(crate) fn into_parts(self) -> (&'s [GradientStopItem<C>], usize, ColorArity<'s, C>) {
        (self.source, self.records, self.colors)
    }
}

pub(crate) fn validate_source<'s, C>(
    work: &mut StopWork<'_, '_, '_>,
    source: &'s [GradientStopItem<C>],
) -> Result<ValidatedStops<'s, C>, PaintError> {
    if work.validation(|| source.is_empty())? {
        return Err(GradientStopsError::EmptyList.into());
    }
    let first = work.color_source(source, 0)?;
    let (first, second, positions) = work.validation(|| match first {
        ColorSourceItem::Stop {
            first,
            second,
            positions,
        } => Ok((first, second, positions)),
        ColorSourceItem::Hint { .. } => Err(GradientStopsError::EndpointHint { source_index: 0 }),
    })??;
    validate_positions(work, 0, positions)?;
    let mut count = if second.is_some() { 2usize } else { 1usize };
    let mut colors = match second {
        Some(last) => ColorArity::Multiple(MultipleColors {
            first,
            last,
            count: 2,
        }),
        None => ColorArity::One(first),
    };
    let mut previous_hint = false;
    for source_index in 1..source.len() {
        let item = work.color_source(source, source_index)?;
        previous_hint = work.validation(|| match item {
            ColorSourceItem::Hint { .. } if source_index == source.len() - 1 => {
                Err(GradientStopsError::EndpointHint { source_index })
            }
            ColorSourceItem::Hint { .. } if previous_hint => {
                Err(GradientStopsError::AdjacentHints {
                    previous_source_index: source_index - 1,
                    source_index,
                })
            }
            ColorSourceItem::Hint { .. } => Ok(true),
            ColorSourceItem::Stop { .. } => Ok(false),
        })??;
        let positions = match item {
            ColorSourceItem::Stop { positions, .. } => positions,
            ColorSourceItem::Hint { position } => GradientStopPositions::One(position),
        };
        validate_positions(work, source_index, positions)?;
        let amount = match positions {
            GradientStopPositions::Two(..) => 2,
            _ => 1,
        };
        count = count
            .checked_add(amount)
            .ok_or(PaintError::CapacityOverflow)?;
        if let ColorSourceItem::Stop { first, second, .. } = item {
            colors = colors.append(first, second)?;
        }
    }
    Ok(ValidatedStops {
        source,
        records: count,
        colors,
        first: FirstColors { first, second },
    })
}

fn validate_positions(
    work: &mut StopWork<'_, '_, '_>,
    source_index: usize,
    positions: GradientStopPositions,
) -> Result<(), PaintError> {
    match positions {
        GradientStopPositions::Omitted => Ok(()),
        GradientStopPositions::One(position) => validate_position(work, source_index, 0, position),
        GradientStopPositions::Two(first, second) => {
            validate_position(work, source_index, 0, first)?;
            validate_position(work, source_index, 1, second)
        }
    }
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

// Only this owner can construct ordinary numerical staging or publish its
// prepared output. The parent can request a complete run, not advance phases
// or substitute a source/backing pair.
mod ordinary {
    use super::*;

    struct Construction<'s, C> {
        input: ValidatedStops<'s, C>,
        line_length: f64,
        staging: Vec<UsedGradientRecord>,
        prepared: Vec<UsedGradientItem<'s, C>>,
        fixed_owner: bool,
    }

    impl<'s, C> Construction<'s, C> {
        fn validate(
            work: &mut StopWork<'_, '_, '_>,
            line_length: f64,
            source: &'s [GradientStopItem<C>],
        ) -> Result<Self, PaintError> {
            if !work.validation(|| line_length.is_finite() && line_length >= 0.0)? {
                return Err(GradientStopsError::InvalidLineLength.into());
            }
            Ok(Self {
                input: validate_source(work, source)?,
                line_length,
                staging: Vec::new(),
                prepared: Vec::new(),
                fixed_owner: false,
            })
        }

        fn resolve(
            &mut self,
            meter: &mut Meter,
            scopes: [Scope; 6],
        ) -> Result<GradientStops<'s, C>, PaintError> {
            let root = meter.root();
            meter
                .with_phase(scopes[1], |p| {
                    self.staging =
                        allocate(p, root, self.input.records, ResourceKind::ScratchBytes)?;
                    expand(
                        &mut StopWork::new(p, scopes[1]),
                        self.line_length,
                        self.input.source,
                        &mut self.staging,
                    )
                })
                .map_err(setup_error)??;
            meter
                .with_phase(scopes[2], |p| {
                    fixup(
                        &mut StopWork::new(p, scopes[2]),
                        self.line_length,
                        self.input.source,
                        &mut self.staging,
                    )
                })
                .map_err(setup_error)??;
            meter
                .with_phase(scopes[3], |p| {
                    let mut work = StopWork::new(p, scopes[3]);
                    distribute(&mut work, self.input.source, &mut self.staging)?;
                    qualify(&mut work, &self.staging)
                })
                .map_err(setup_error)??;
            meter
                .with_phase(scopes[4], |p| {
                    let first = self.prepare(&mut StopWork::new(p, scopes[4]), root)?;
                    apply(
                        p,
                        root,
                        Change::Acquire(
                            ResourceKind::RetainedResultBytes,
                            bytes::<GradientStops<'_, C>>(1)?,
                        ),
                    )?;
                    self.fixed_owner = true;
                    charge(p, scopes[4], WorkKind::OutputRecord, 1)?;
                    // Keep both admitted buffers under cleanup ownership until the
                    // final fallible publication action has succeeded.
                    Ok(GradientStops {
                        source: self.input.source,
                        line_length: self.line_length,
                        first,
                        tail: take(&mut self.prepared),
                    })
                })
                .map_err(setup_error)?
        }

        fn prepare(
            &mut self,
            work: &mut StopWork<'_, '_, '_>,
            root: Scope,
        ) -> Result<UsedGradientStop<'s, C>, PaintError> {
            self.prepared = allocate(
                work.meter(),
                root,
                // Validation proves at least one expanded occurrence; this
                // owner alone expands and qualifies that complete backing.
                self.input.records - 1,
                ResourceKind::RetainedResultBytes,
            )?;
            // Validation retained the actual first Stop occurrences. Numerical
            // qualification above completed before any prepared allocation.
            let occurrence = self.input.first.first;
            let position = work.record(&self.staging, 0)?.position;
            let first = work.output(|| UsedGradientStop {
                source_index: occurrence.source_index(),
                position_index: occurrence.position_index(),
                payload: occurrence.payload(),
                position,
            })?;
            let mut index = 1;
            if let Some(second) = self.input.first.second {
                self.prepare_stop(work, index, second)?;
                index += 1;
            }
            for source_index in 1..self.input.source.len() {
                match work.color_source(self.input.source, source_index)? {
                    ColorSourceItem::Stop { first, second, .. } => {
                        self.prepare_stop(work, index, first)?;
                        index += 1;
                        if let Some(second) = second {
                            self.prepare_stop(work, index, second)?;
                            index += 1;
                        }
                    }
                    ColorSourceItem::Hint { .. } => {
                        // source_index names the actual source visit above.
                        let position = work.record(&self.staging, index)?.position;
                        work.emit(
                            &mut self.prepared,
                            UsedGradientItem::Hint {
                                source_index,
                                position,
                            },
                        )?;
                        index += 1;
                    }
                }
            }
            Ok(first)
        }

        fn prepare_stop(
            &mut self,
            work: &mut StopWork<'_, '_, '_>,
            index: usize,
            occurrence: ColorOccurrence<'s, C>,
        ) -> Result<(), PaintError> {
            let position = work.record(&self.staging, index)?.position;
            work.emit(
                &mut self.prepared,
                UsedGradientItem::Stop {
                    source_index: occurrence.source_index(),
                    position_index: occurrence.position_index(),
                    payload: occurrence.payload(),
                    position,
                },
            )
        }

        fn cleanup(
            &mut self,
            p: &mut PhaseMeter<'_, '_>,
            root: Scope,
            failed: bool,
        ) -> Result<(), PaintError> {
            release(p, root, take(&mut self.staging), ResourceKind::ScratchBytes)?;
            if failed {
                release(
                    p,
                    root,
                    take(&mut self.prepared),
                    ResourceKind::RetainedResultBytes,
                )?;
                if self.fixed_owner {
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
        }
    }

    pub(super) fn run<'s, C>(
        meter: &mut Meter,
        scopes: [Scope; 6],
        line_length: f64,
        source: &'s [GradientStopItem<C>],
    ) -> Result<GradientStops<'s, C>, PaintError> {
        let validated = meter
            .with_phase(scopes[0], |p| {
                Construction::validate(&mut StopWork::new(p, scopes[0]), line_length, source)
            })
            .map_err(setup_error)
            .and_then(|value| value);
        let mut owner = match validated {
            Ok(owner) => owner,
            Err(error) => {
                meter.with_phase(scopes[5], |_| ()).map_err(setup_error)?;
                return Err(error);
            }
        };
        let result = owner.resolve(meter, scopes);
        let root = meter.root();
        let cleanup = meter
            .with_phase(scopes[5], |p| owner.cleanup(p, root, result.is_err()))
            .map_err(setup_error)
            .and_then(|done| done);
        result.and_then(|value| cleanup.map(|()| value))
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
            let mut meter = options(PaintLimits::unlimited())
                .operation(OperationKind::GradientStops)
                .unwrap();
            let scopes = setup_scopes(&mut meter).unwrap();
            let root = meter.root();
            let mut owner = meter
                .with_phase(scopes[0], |p| {
                    Construction::validate(&mut StopWork::new(p, scopes[0]), 100.0, &source)
                })
                .unwrap()
                .unwrap();
            let plan = owner.resolve(&mut meter, scopes).unwrap();
            let scratch = (owner.staging.capacity() * size_of::<UsedGradientRecord>()) as u64;
            meter
                .with_phase(scopes[5], |p| owner.cleanup(p, root, false))
                .unwrap()
                .unwrap();
            let (_, report) = finish(meter, Ok(()));
            let report = report.unwrap();
            assert_eq!(size_of::<Payload>(), 0);
            assert_eq!(plan.tail.len(), 3);
            assert!(plan.tail.capacity() >= 3);
            assert_eq!(plan.first.position(), 0.0);
            assert!(size_of::<UsedGradientRecord>() > 0);
            let backing =
                (plan.tail.capacity() * size_of::<UsedGradientItem<'_, Payload>>()) as u64;
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
            assert_eq!(report.peak(ResourceKind::ScratchBytes), scratch);
            assert!(scratch > 0);
            assert_eq!(report.work(WorkKind::OutputRecord), 9);
            // Validation 3 sources; expansion 3 sources; endpoint fixup 5
            // accesses + 4 record/source pairs; distribution 3 pairs + 4
            // qualification records; publication 2 sources + 4 records.
            assert_eq!(
                report.work(WorkKind::GradientStopVisit),
                3 + 3 + 13 + 10 + 6
            );
            // Three positioned occurrences each FMA/finite (6), three prefix
            // comparisons, then four finite and three ordering checks (7).
            assert_eq!(report.work(WorkKind::GradientCoordinate), 6 + 3 + 7);
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
        fn every_source_record_and_emission_frontier_keeps_causal_error_and_retries() {
            let source = source();
            for cap in 0..35 {
                let outcome = resolve_gradient_stops(
                    100.0,
                    &source,
                    options(
                        PaintLimits::unlimited().with_work_limit(WorkKind::GradientStopVisit, cap),
                    ),
                );
                let Err(PaintError::Budget(refusal)) = outcome.result else {
                    panic!("visit must refuse")
                };
                assert_eq!(refusal.current, cap);
                assert_eq!(
                    refusal.change,
                    PaintChange::Work {
                        kind: WorkKind::GradientStopVisit,
                        amount: 1
                    }
                );
                let expected_phase = match cap {
                    0..=2 => PaintPhase::GradientValidation,
                    3..=5 => PaintPhase::GradientStopResolution,
                    6..=18 => PaintPhase::GradientStopFixup,
                    19..=28 => PaintPhase::GradientStopDistribution,
                    _ => PaintPhase::Publication,
                };
                assert_eq!(refusal.origin, expected_phase);
                let report = outcome.report.unwrap();
                assert_eq!(report.work(WorkKind::GradientStopVisit), cap);
                let emissions = match cap {
                    0..=3 => 0,
                    4 => 2,
                    5 => 3,
                    6..=29 => 4,
                    30 => 5,
                    31..=32 => 6,
                    _ => 7,
                };
                assert_eq!(report.work(WorkKind::OutputRecord), emissions);
                for &resource in ResourceKind::ALL {
                    assert_eq!(report.live(resource), 0);
                }
                let retried = resolve_gradient_stops(
                    100.0,
                    &source,
                    options(
                        PaintLimits::unlimited().with_work_limit(WorkKind::GradientStopVisit, 35),
                    ),
                );
                assert_eq!(retried.result.unwrap().items().count(), 4);
            }
            for cap in 0..9 {
                let outcome = resolve_gradient_stops(
                    100.0,
                    &source,
                    options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, cap)),
                );
                let Err(PaintError::Budget(refusal)) = outcome.result else {
                    panic!("emission must refuse")
                };
                assert_eq!(refusal.current, cap);
                assert_eq!(
                    refusal.change,
                    PaintChange::Work {
                        kind: WorkKind::OutputRecord,
                        amount: 1
                    }
                );
                assert_eq!(
                    refusal.origin,
                    if cap < 4 {
                        PaintPhase::GradientStopResolution
                    } else {
                        PaintPhase::Publication
                    }
                );
                let report = outcome.report.unwrap();
                assert_eq!(report.work(WorkKind::OutputRecord), cap);
                for &resource in ResourceKind::ALL {
                    assert_eq!(report.live(resource), 0);
                }
            }
        }

        #[test]
        fn single_inline_head_has_no_prepared_heap_and_charges_actual_fixed_owner() {
            let source = [GradientStopItem::Stop {
                payload: Payload,
                positions: GradientStopPositions::Omitted,
            }];
            let fixed = size_of::<GradientStops<'_, Payload>>() as u64;
            let outcome = resolve_gradient_stops(
                0.0,
                &source,
                options(
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::RetainedResultBytes, fixed),
                ),
            );
            let plan = outcome.result.unwrap();
            assert_eq!(plan.tail.capacity(), 0);
            assert_eq!(plan.first.position(), 0.0);
            let report = outcome.report.unwrap();
            assert_eq!(report.live(ResourceKind::RetainedResultBytes), fixed);
            assert_eq!(report.peak(ResourceKind::RetainedResultBytes), fixed);
            assert_eq!(report.work(WorkKind::GradientStopVisit), 9);
            assert_eq!(report.work(WorkKind::OutputRecord), 3);
            let failed = resolve_gradient_stops(
                0.0,
                &source,
                options(
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::RetainedResultBytes, fixed - 1),
                ),
            );
            let Err(PaintError::Budget(refusal)) = failed.result else {
                panic!("fixed owner must refuse")
            };
            assert_eq!(refusal.current, 0);
            assert_eq!(
                refusal.change,
                PaintChange::Acquire {
                    kind: ResourceKind::RetainedResultBytes,
                    amount: fixed
                }
            );
            let report = failed.report.unwrap();
            assert_eq!(report.work(WorkKind::OutputRecord), 2);
            for &resource in ResourceKind::ALL {
                assert_eq!(report.live(resource), 0);
            }
        }

        #[test]
        fn fixed_owner_and_later_output_refusals_release_all_admitted_custody() {
            let source = source();
            let baseline =
                resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
            let plan = baseline.result.unwrap();
            let backing =
                (plan.tail.capacity() * size_of::<UsedGradientItem<'_, Payload>>()) as u64;
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
            assert_eq!(report.work(WorkKind::OutputRecord), 8);
            for &kind in ResourceKind::ALL {
                assert_eq!(report.live(kind), 0);
            }

            let outcome = resolve_gradient_stops(
                100.0,
                &source,
                options(PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 8)),
            );
            let Err(PaintError::Budget(refusal)) = outcome.result else {
                panic!("publication must refuse");
            };
            assert_eq!(refusal.origin, PaintPhase::Publication);
            assert_eq!(refusal.current, 8);
            assert_eq!(
                refusal.change,
                PaintChange::Work {
                    kind: WorkKind::OutputRecord,
                    amount: 1
                }
            );
            let report = outcome.report.unwrap();
            assert_eq!(report.work(WorkKind::OutputRecord), 8);
            assert_eq!(
                report.peak(ResourceKind::RetainedResultBytes),
                backing + fixed
            );
            for &kind in ResourceKind::ALL {
                assert_eq!(report.live(kind), 0);
            }
        }

        #[test]
        fn prepared_allocation_and_partial_capture_release_both_backings_for_retry() {
            let source = source();
            let baseline =
                resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
            let plan = baseline.result.unwrap();
            let report = baseline.report.unwrap();
            let scratch = report.peak(ResourceKind::ScratchBytes);
            let backing =
                (plan.tail.capacity() * size_of::<UsedGradientItem<'_, Payload>>()) as u64;
            let fixed = size_of::<GradientStops<'_, Payload>>() as u64;
            let publication_visits = report
                .phases()
                .find(|phase| phase.phase() == PaintPhase::Publication)
                .unwrap()
                .work(WorkKind::GradientStopVisit);
            // Source zero is retained from validation: two later source visits
            // and four reads of qualified numerical positions.
            assert_eq!(publication_visits, 2 + 4);
            let before_capture = report.work(WorkKind::GradientStopVisit) - publication_visits;
            // Byte custody is admitted at the root; capture work is admitted
            // at Publication. Refusal origins follow those actual scopes.
            for (limits, phase, outputs, retained_peak) in [
                (
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::ScratchBytes, scratch - 1),
                    PaintPhase::Operation,
                    0,
                    0,
                ),
                (
                    PaintLimits::unlimited()
                        .with_resource_limit(ResourceKind::RetainedResultBytes, 0),
                    PaintPhase::Operation,
                    4,
                    0,
                ),
                (
                    PaintLimits::unlimited().with_work_limit(WorkKind::OutputRecord, 5),
                    PaintPhase::Publication,
                    5,
                    backing,
                ),
                // The first position read is accepted; the second numeric read
                // refuses after the inline head has been emitted.
                (
                    PaintLimits::unlimited()
                        .with_work_limit(WorkKind::GradientStopVisit, before_capture + 1),
                    PaintPhase::Publication,
                    5,
                    backing,
                ),
            ] {
                let failed = resolve_gradient_stops(100.0, &source, options(limits));
                let Err(PaintError::Budget(refusal)) = failed.result else {
                    panic!("expected staging/preparation refusal");
                };
                assert_eq!(refusal.origin, phase);
                let failed_report = failed.report.unwrap();
                assert_eq!(failed_report.work(WorkKind::OutputRecord), outputs);
                assert_eq!(
                    failed_report.peak(ResourceKind::RetainedResultBytes),
                    retained_peak
                );
                for &kind in ResourceKind::ALL {
                    assert_eq!(failed_report.live(kind), 0);
                }
                let retry = resolve_gradient_stops(
                    100.0,
                    &source,
                    options(
                        PaintLimits::unlimited()
                            .with_resource_limit(ResourceKind::ScratchBytes, scratch)
                            .with_resource_limit(
                                ResourceKind::RetainedResultBytes,
                                backing + fixed,
                            ),
                    ),
                );
                let retried = retry.result.unwrap();
                assert_eq!(retried.items().count(), 4);
                assert_eq!(retry.report.unwrap().live(ResourceKind::ScratchBytes), 0);
            }
        }

        #[test]
        fn single_omission_units_and_refused_prefix_charge_only_actions_that_run() {
            let source = [GradientStopItem::Stop {
                payload: Payload,
                positions: GradientStopPositions::Omitted,
            }];
            let baseline =
                resolve_gradient_stops(100.0, &source, options(PaintLimits::unlimited()));
            assert!(baseline.result.is_ok());
            let report = baseline.report.unwrap();
            // L, list emptiness, item grammar. Visits: validation1 + expansion1 +
            // first record/source/write3 + prefix record/source2 + qualification1.
            // Preparation reuses the validated occurrence and reads its numeric position.
            // The only numeric inspection is final finiteness; no affine/lerp runs.
            assert_eq!(report.work(WorkKind::GradientValidation), 3);
            assert_eq!(report.work(WorkKind::GradientStopVisit), 9);
            assert_eq!(report.work(WorkKind::GradientCoordinate), 1);
            assert_eq!(report.work(WorkKind::OutputRecord), 3);

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
}

//! Source admission and serialization for HTML's exact live Input stepping.

use super::super::{StepInput, StepRead};
use super::*;
use std::{cmp::Ordering, fmt, fmt::Write};
use surgeist_html::{
    attributes::{
        HtmlInputType,
        input_bounds::{HtmlInputNumericConstraints, select_input_numeric_constraints},
    },
    bounded::{
        input::{
            HtmlInputTemporalStepError, HtmlInputTemporalStepMeter, HtmlInputTemporalStepOutcome,
            decode_input_temporal_step, input_temporal_step_parameters,
            input_temporal_step_register_bytes, serialize_input_temporal_step,
        },
        numeric::{
            HtmlFiniteDecimalExpansionCursor, HtmlIntegerUnitStepError, HtmlNaturalDigits,
            HtmlNumericMaterializationError, HtmlRangeScratchRole,
        },
    },
    elements::input::{
        HtmlInputStepBounds, HtmlInputStepCoordinate, HtmlInputStepDirection, HtmlInputStepError,
        HtmlInputStepOperand,
    },
    microsyntax::numeric::HtmlSignedDecimal,
};

pub(in crate::internal::store::input_processing) fn run<F>(
    meter: &mut InputProcessing,
    input: StepInput,
    source: Option<&Formation>,
    read: F,
    cancel: &AtomicBool,
) -> Result<Option<InputValue>, DomError>
where
    F: FnMut(StepRead) -> Result<Option<InputValue>, DomError>,
{
    let fixed = size_of::<Session<'_, F>>();
    meter.fixed(fixed, fixed, source, cancel)?;
    let mut session = Session {
        meter,
        input,
        source,
        cancel,
        read,
        storage: Storage::new(),
    };
    let raw = (session.read)(StepRead::Step)?;
    let step = super::step(
        session.meter,
        &mut session.storage,
        raw.as_ref().map(|raw| &raw.value),
        source,
        cancel,
    )?;
    drop(raw);
    if step == HtmlInputNumericStep::Any {
        return Err(DomError::InputStepInvalidState);
    }
    if matches!(
        session.input.kind,
        HtmlInputType::Number | HtmlInputType::Range
    ) {
        session.binary64(step)
    } else {
        session.temporal(step)
    }
}

struct Session<'a, F> {
    meter: &'a mut InputProcessing,
    input: StepInput,
    source: Option<&'a Formation>,
    cancel: &'a AtomicBool,
    read: F,
    storage: Storage,
}

struct Expansion {
    digits: Vec<u8>,
    scale: usize,
    negative: bool,
}

impl Expansion {
    fn operand(
        &self,
        meter: &mut InputProcessing,
        source: Option<&Formation>,
        cancel: &AtomicBool,
    ) -> Result<HtmlInputStepOperand<'_>, DomError> {
        let work = HtmlNaturalDigits::validation_work_bound(self.digits.len())
            .ok_or(DomError::RepresentationExhausted)?;
        meter.charge_source_work(work, source, cancel)?;
        let digits = HtmlNaturalDigits::try_little(&self.digits)
            .map_err(|_| DomError::InputProcessingProtocol)?;
        Ok(HtmlInputStepOperand::coefficient(
            digits,
            self.scale,
            self.negative,
        ))
    }
}

impl<F: FnMut(StepRead) -> Result<Option<InputValue>, DomError>> Session<'_, F> {
    fn temporal(&mut self, step: HtmlInputNumericStep) -> Result<Option<InputValue>, DomError> {
        let kind = self.input.kind;
        let parameters =
            input_temporal_step_parameters(kind).ok_or(DomError::InputStepInvalidState)?;
        let minimum = self.temporal_content(StepRead::Minimum)?;
        let maximum = self.temporal_content(StepRead::Maximum)?;
        let min = minimum.as_ref().map(HtmlInputStepOperand::decimal);
        let max = maximum.as_ref().map(HtmlInputStepOperand::decimal);
        if let (Some(min), Some(max)) = (min, max)
            && min.compare(max, || self.grid_tick()).map_err(grid_error)? == Ordering::Greater
        {
            return Ok(None);
        }
        let default = if min.is_none() {
            self.temporal_content(StepRead::Default)?
        } else {
            None
        };
        let default = default.as_ref().map(HtmlInputStepOperand::decimal);
        let fallback;
        let base = match min.or(default) {
            Some(base) => base,
            None => {
                // The pinned temporal fallback constants are exact binary64 integers.
                let base = HtmlFloatingPointNumber::try_from(parameters.default_base as f64)
                    .map_err(|_| DomError::InputProcessingProtocol)?;
                fallback = self.expand(base)?;
                self.operand(&fallback)?
            }
        };
        let allowed = match step {
            HtmlInputNumericStep::Value(value) if value.value() > 0.0 => value,
            _ => HtmlFloatingPointNumber::try_from(f64::from(parameters.default_step))
                .map_err(|_| DomError::InputProcessingProtocol)?,
        };
        let allowed = self.expand(allowed)?;
        let allowed = self.operand(&allowed)?;
        let mut scaled = self.grid_storage()?;
        self.scratch(&mut scaled, allowed.multiplication_cells())?;
        let step = allowed
            .multiply(parameters.scale, &mut scaled, || self.grid_tick())
            .map_err(grid_error)?;
        self.admit_grid_registers()?;
        let bounds = HtmlInputStepBounds {
            minimum: min,
            maximum: max,
            base,
            step: step.operand(),
        };
        let mut scratch = self.grid_storage()?;
        self.scratch(&mut scratch, bounds.qualification_cells())?;
        let Some(grid) = bounds
            .qualify(&mut scratch, || self.grid_tick())
            .map_err(grid_error)?
        else {
            return Ok(None);
        };
        let current = self.temporal_content(StepRead::Current)?;
        let current = current.as_ref().map(HtmlInputStepOperand::decimal);
        self.scratch(&mut scratch, grid.scratch_cells(current))?;
        let Some(projected) = grid
            .project(
                current,
                self.input.direction,
                self.input.count,
                &mut scratch,
                || self.grid_tick(),
            )
            .map_err(grid_error)?
        else {
            return Ok(None);
        };
        let header = size_of::<Vec<u16>>();
        self.meter.fixed(header, header, self.source, self.cancel)?;
        let units = self.with_temporal_meter(|meter| {
            serialize_input_temporal_step(kind, projected, usize::MAX, meter)
        })?;
        self.meter
            .output(units, None, self.source, self.cancel)
            .map(Some)
    }

    fn temporal_content(
        &mut self,
        position: StepRead,
    ) -> Result<Option<HtmlSignedDecimal>, DomError> {
        let raw = (self.read)(position)?;
        let kind = self.input.kind;
        self.with_temporal_meter(|meter| {
            decode_input_temporal_step(
                kind,
                raw.as_ref().map(|value| value.value.code_units()),
                usize::MAX,
                meter,
            )
        })
    }

    fn with_temporal_meter<T>(
        &mut self,
        perform: impl FnOnce(&mut TemporalMeter<'_>) -> HtmlInputTemporalStepOutcome<T, DomError>,
    ) -> Result<T, DomError> {
        let fixed = sum(&[
            size_of::<TemporalMeter<'_>>(),
            input_temporal_step_register_bytes(),
        ])?;
        self.meter.fixed(fixed, fixed, self.source, self.cancel)?;
        let (result, _, _) = perform(&mut TemporalMeter {
            meter: self.meter,
            source: self.source,
            cancel: self.cancel,
        })
        .into_parts();
        self.meter.release(fixed, metadata(fixed), self.source);
        result.map_err(temporal_error)
    }

    fn admit_grid_registers(&mut self) -> Result<(), DomError> {
        let bytes = HtmlInputStepBounds::register_bytes();
        self.meter.fixed(bytes, bytes, self.source, self.cancel)
    }

    fn binary64(&mut self, step: HtmlInputNumericStep) -> Result<Option<InputValue>, DomError> {
        let minimum = self.numeric_content(StepRead::Minimum)?;
        let maximum = self.numeric_content(StepRead::Maximum)?;
        let kind = self.input.kind;
        let limits = select_input_numeric_constraints(
            minimum,
            maximum,
            step,
            HtmlInputNumericContent::Absent,
            kind,
        )
        .ok_or(DomError::InputStepInvalidState)?;
        self.grid_tick()?;
        if let (Some(min), Some(max)) = (limits.minimum(), limits.maximum())
            && min.value() > max.value()
        {
            return Ok(None);
        }
        let limits = if matches!(minimum, HtmlInputNumericContent::Value(_)) {
            limits
        } else {
            let default = self.numeric_content(StepRead::Default)?;
            select_input_numeric_constraints(minimum, maximum, step, default, kind)
                .ok_or(DomError::InputStepInvalidState)?
        };
        let minimum = limits
            .minimum()
            .map(|number| self.expand(number))
            .transpose()?;
        let maximum = limits
            .maximum()
            .map(|number| self.expand(number))
            .transpose()?;
        let base = self.expand(limits.step_base())?;
        let step = self.expand(
            limits
                .allowed_step()
                .ok_or(DomError::InputStepInvalidState)?,
        )?;
        self.admit_grid_registers()?;
        let bounds = HtmlInputStepBounds {
            minimum: minimum
                .as_ref()
                .map(|value| self.operand(value))
                .transpose()?,
            maximum: maximum
                .as_ref()
                .map(|value| self.operand(value))
                .transpose()?,
            base: self.operand(&base)?,
            step: self.operand(&step)?,
        };
        let mut scratch = self.grid_storage()?;
        self.scratch(&mut scratch, bounds.qualification_cells())?;
        let Some(grid) = bounds
            .qualify(&mut scratch, || self.grid_tick())
            .map_err(grid_error)?
        else {
            return Ok(None);
        };
        let current = match self.numeric_content(StepRead::Current)? {
            HtmlInputNumericContent::Value(value) => Some(value),
            _ => None,
        };
        let expanded = current.map(|value| self.expand(value)).transpose()?;
        let operand = expanded
            .as_ref()
            .map(|value| self.operand(value))
            .transpose()?;
        self.scratch(&mut scratch, grid.scratch_cells(operand))?;
        let Some(projected) = grid
            .project(
                operand,
                self.input.direction,
                self.input.count,
                &mut scratch,
                || self.grid_tick(),
            )
            .map_err(grid_error)?
        else {
            return Ok(None);
        };
        self.serialize_binary64(projected, &limits, current)
    }

    fn serialize_binary64(
        &mut self,
        projected: HtmlInputStepCoordinate<'_>,
        limits: &HtmlInputNumericConstraints,
        current: Option<HtmlFloatingPointNumber>,
    ) -> Result<Option<InputValue>, DomError> {
        let raw = self.projected_text(projected)?;
        let rounded = parse(
            self.meter,
            &mut self.storage,
            &raw,
            HtmlFloatingPointParseMode::Recovery,
            self.source,
            self.cancel,
        );
        let payload = super::super::output_payload(&raw)?;
        drop(raw);
        self.meter.release(payload.bytes, payload, self.source);
        let rounded = rounded?;
        let HtmlInputRangeValue::Finite(rounded) = rounded else {
            return Err(DomError::InputNumeric(HtmlRangeNumericError::Overflow));
        };
        let units = if self.input.kind == HtmlInputType::Number {
            let number = HtmlFloatingPointNumber::try_from(rounded.value())
                .map_err(|_| DomError::InputProcessingProtocol)?;
            self.plain_number(number)?
        } else {
            let Some(value) = self.range_value(projected, rounded, limits, current)? else {
                return Ok(None);
            };
            best(
                self.meter,
                &mut self.storage,
                value,
                self.source,
                self.cancel,
            )?
        };
        self.meter
            .output(units, None, self.source, self.cancel)
            .map(Some)
    }

    fn range_value(
        &mut self,
        projected: HtmlInputStepCoordinate<'_>,
        rounded: HtmlInputRangeNumber,
        limits: &HtmlInputNumericConstraints,
        current: Option<HtmlFloatingPointNumber>,
    ) -> Result<Option<HtmlInputRangeNumber>, DomError> {
        let mut minimum = limits
            .minimum()
            .ok_or(DomError::InputProcessingProtocol)?
            .value();
        let mut maximum = limits
            .maximum()
            .ok_or(DomError::InputProcessingProtocol)?
            .value();
        let prior = current.map_or(0.0, |value| value.value());
        match self.input.direction {
            HtmlInputStepDirection::Up => minimum = minimum.max(prior),
            HtmlInputStepDirection::Down => maximum = maximum.min(prior),
        }
        let approximate = self.expand(qualified_number(rounded.value())?)?;
        let target = projected.operand();
        let position = target
            .compare(self.operand(&approximate)?, || self.grid_tick())
            .map_err(grid_error)?;
        if position == Ordering::Equal {
            return self.range_lattice(rounded, minimum, maximum, limits);
        }
        // Nearest/even gives a bounding pair of adjacent binary64 values.
        // Search each side independently and compare to the exact projection.
        let (lower_edge, upper_edge) = if position == Ordering::Less {
            (rounded.value().next_down(), rounded.value())
        } else {
            (rounded.value(), rounded.value().next_up())
        };
        let lower = self.range_lattice(rounded, minimum, maximum.min(lower_edge), limits)?;
        let upper = self.range_lattice(rounded, minimum.max(upper_edge), maximum, limits)?;
        let (Some(lower), Some(upper)) = (lower, upper) else {
            return Ok(lower.or(upper));
        };
        let lower_exact = self.expand(qualified_number(lower.value())?)?;
        let upper_exact = self.expand(qualified_number(upper.value())?)?;
        let order = target
            .compare_midpoint(
                self.operand(&lower_exact)?,
                self.operand(&upper_exact)?,
                || self.grid_tick(),
            )
            .map_err(grid_error)?;
        Ok(Some(if order == Ordering::Less {
            lower
        } else {
            upper
        }))
    }

    fn range_lattice(
        &mut self,
        current: HtmlInputRangeNumber,
        minimum: f64,
        maximum: f64,
        limits: &HtmlInputNumericConstraints,
    ) -> Result<Option<HtmlInputRangeNumber>, DomError> {
        if minimum > maximum {
            return Ok(None);
        }
        let minimum = HtmlInputRangeNumber::from(qualified_number(minimum)?);
        let maximum = HtmlInputRangeNumber::from(qualified_number(maximum)?);
        let allowed = limits
            .allowed_step()
            .ok_or(DomError::InputStepInvalidState)?;
        lattice(
            self.meter,
            &mut self.storage,
            LatticeInput {
                current,
                minimum,
                maximum: Some(maximum),
                base: limits.step_base().into(),
                step: allowed.into(),
            },
            self.source,
            self.cancel,
        )
    }

    fn projected_text(&mut self, value: HtmlInputStepCoordinate<'_>) -> Result<Vec<u16>, DomError> {
        let significant = value.coefficient();
        let scale = value.scale();
        let digits = significant.len().max(
            scale
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
        );
        let length = sum(&[
            digits,
            usize::from(value.is_negative()),
            usize::from(scale != 0),
        ])?;
        let mut units = self.utf16_storage(length)?;
        if value.is_negative() {
            self.push_unit(&mut units, u16::from(b'-'))?;
        }
        for index in (0..digits).rev() {
            if index + 1 == scale {
                self.push_unit(&mut units, u16::from(b'.'))?;
            }
            let digit = significant.get(index).copied().unwrap_or(0);
            self.push_unit(&mut units, u16::from(b'0' + digit))?;
        }
        Ok(units)
    }

    fn utf16_storage(&mut self, length: usize) -> Result<Vec<u16>, DomError> {
        let header = size_of::<Vec<u16>>();
        self.meter.fixed(header, header, self.source, self.cancel)?;
        let mut units = Vec::new();
        self.meter.reserve(
            &mut units,
            length,
            BankKind::Utf16,
            self.source,
            self.cancel,
        )?;
        Ok(units)
    }

    fn push_unit(&mut self, output: &mut Vec<u16>, unit: u16) -> Result<(), DomError> {
        if output.len() == output.capacity() {
            return Err(DomError::InputProcessingProtocol);
        }
        self.meter
            .charge(1 + size_of::<u16>(), self.source, self.cancel)?;
        output.push(unit);
        Ok(())
    }

    fn plain_number(&mut self, number: HtmlFloatingPointNumber) -> Result<Vec<u16>, DomError> {
        let fixed = size_of::<PlainDecimal>();
        self.meter.fixed(fixed, fixed, self.source, self.cancel)?;
        let result = (|| {
            // The finite binary64 plain decimal format has a fixed scalar bound.
            self.meter
                .charge_source_work(1024, self.source, self.cancel)?;
            let mut spelling = PlainDecimal {
                units: [0; 327],
                length: 0,
            };
            write!(spelling, "{number}").map_err(|_| DomError::InputProcessingProtocol)?;
            let mut units = self.utf16_storage(spelling.length)?;
            for &unit in spelling.units.iter().take(spelling.length) {
                self.push_unit(&mut units, unit)?;
            }
            Ok(units)
        })();
        self.meter.release(fixed, metadata(fixed), self.source);
        result
    }

    fn operand<'digits>(
        &mut self,
        value: &'digits Expansion,
    ) -> Result<HtmlInputStepOperand<'digits>, DomError> {
        value.operand(self.meter, self.source, self.cancel)
    }

    fn grid_storage(&mut self) -> Result<Vec<u8>, DomError> {
        let header = size_of::<Vec<u8>>();
        self.meter.fixed(header, header, self.source, self.cancel)?;
        Ok(Vec::new())
    }

    fn numeric_content(&mut self, read: StepRead) -> Result<HtmlInputNumericContent, DomError> {
        let raw = (self.read)(read)?;
        content(
            self.meter,
            &mut self.storage,
            raw.as_ref().map(|raw| &raw.value),
            self.source,
            self.cancel,
        )
    }

    fn expand(&mut self, value: HtmlFloatingPointNumber) -> Result<Expansion, DomError> {
        let Self {
            meter,
            storage,
            source,
            cancel,
            ..
        } = self;
        let layout = HtmlFiniteDecimalExpansionCursor::layout();
        let fixed = frame(layout)?;
        meter.fixed(fixed, fixed, *source, cancel)?;
        let result = (|| {
            let mut cursor = HtmlFiniteDecimalExpansionCursor::new(value, storage.headers())
                .map_err(DomError::InputNumeric)?;
            loop {
                meter.inspect(*source, cancel)?;
                let Some(request) = cursor.request() else {
                    break;
                };
                let reply = storage.service(meter, &[], request, layout, *source, cancel)?;
                cursor.reply(reply).map_err(DomError::InputNumeric)?;
            }
            if cursor.headers() != storage.headers() {
                return Err(DomError::InputProcessingProtocol);
            }
            let (negative, scale) = cursor.result().ok_or(DomError::InputProcessingProtocol)?;
            let output = size_of::<Expansion>();
            meter.fixed(output, output, *source, cancel)?;
            let digits =
                std::mem::take(&mut storage.banks[HtmlRangeScratchRole::Coefficient.index()]);
            Ok(Expansion {
                digits,
                scale,
                negative,
            })
        })();
        meter.release(fixed, metadata(fixed), *source);
        result
    }

    fn scratch(&mut self, scratch: &mut Vec<u8>, length: Option<usize>) -> Result<(), DomError> {
        let length = length.ok_or(DomError::RepresentationExhausted)?;
        self.meter
            .reserve(scratch, length, BankKind::Bytes, self.source, self.cancel)?;
        for _ in scratch.len()..length {
            self.meter.charge(1, self.source, self.cancel)?;
            scratch.push(0);
        }
        Ok(())
    }

    fn grid_tick(&mut self) -> Result<(), DomError> {
        self.meter.charge(1, self.source, self.cancel)
    }
}

fn qualified_number(value: f64) -> Result<HtmlFloatingPointNumber, DomError> {
    HtmlFloatingPointNumber::try_from(value).map_err(|_| DomError::InputProcessingProtocol)
}

struct PlainDecimal {
    units: [u16; 327],
    length: usize,
}

impl fmt::Write for PlainDecimal {
    fn write_str(&mut self, spelling: &str) -> fmt::Result {
        if !spelling.is_ascii() {
            return Err(fmt::Error);
        }
        let end = self.length.checked_add(spelling.len()).ok_or(fmt::Error)?;
        let destination = self.units.get_mut(self.length..end).ok_or(fmt::Error)?;
        for (unit, byte) in destination.iter_mut().zip(spelling.bytes()) {
            *unit = u16::from(byte);
        }
        self.length = end;
        Ok(())
    }
}

struct TemporalMeter<'a> {
    meter: &'a mut InputProcessing,
    source: Option<&'a Formation>,
    cancel: &'a AtomicBool,
}

impl HtmlInputTemporalStepMeter for TemporalMeter<'_> {
    type Stop = DomError;

    fn work(&mut self, units: usize) -> Result<(), Self::Stop> {
        self.meter
            .charge_source_work(units, self.source, self.cancel)
    }

    fn reserve(&mut self, bytes: usize, units: usize, _: usize) -> Result<(), Self::Stop> {
        self.meter
            .acquire(bytes, Payload { bytes, units }, self.source, self.cancel)
    }

    fn observe_allocation(&mut self, _: usize, _: usize) -> Result<(), Self::Stop> {
        // Native Input processing already holds the live reservation.
        Ok(())
    }

    fn release(&mut self, bytes: usize, units: usize, _: usize) {
        self.meter
            .release(bytes, Payload { bytes, units }, self.source);
    }
}

fn temporal_error(error: HtmlInputTemporalStepError<DomError>) -> DomError {
    match error {
        HtmlInputTemporalStepError::Admission(error) => error,
        HtmlInputTemporalStepError::Inapplicable => DomError::InputStepInvalidState,
        HtmlInputTemporalStepError::Materialization(HtmlNumericMaterializationError::Boundary(
            HtmlIntegerUnitStepError::AllocationFailed { .. },
        )) => DomError::AllocationFailed,
        HtmlInputTemporalStepError::Materialization(HtmlNumericMaterializationError::Digits(_)) => {
            DomError::InputProcessingProtocol
        }
        _ => DomError::RepresentationExhausted,
    }
}

fn grid_error(error: HtmlInputStepError<DomError>) -> DomError {
    match error {
        HtmlInputStepError::NonPositiveStep => DomError::InputStepInvalidState,
        HtmlInputStepError::Workspace => DomError::RepresentationExhausted,
        HtmlInputStepError::Work(error) => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::store::DomResource;

    fn step(
        meter: &mut InputProcessing,
        kind: HtmlInputType,
        initial: &str,
    ) -> Result<Option<InputValue>, DomError> {
        let value = DocumentString::from(initial);
        meter.step(
            StepInput {
                kind,
                direction: HtmlInputStepDirection::Up,
                count: 1,
            },
            None,
            |read| {
                Ok(match read {
                    StepRead::Current | StepRead::Default => Some(InputValue {
                        value: value.clone(),
                        payload: None,
                    }),
                    _ => None,
                })
            },
            &AtomicBool::new(false),
        )
    }

    #[test]
    fn exact_scratch_and_work_boundaries_release_the_original_session() {
        for (kind, initial, expected) in [
            (HtmlInputType::Number, "7.25", "8.25"),
            (HtmlInputType::Time, "23:59", "00:00"),
        ] {
            let mut observation = InputProcessing::new(1_000_000, 100_000_000);
            let value = step(&mut observation, kind, initial).unwrap().unwrap();
            assert_eq!(value.value, DocumentString::from(expected));
            assert_eq!(observation.live, 0);
            let mut exact = InputProcessing::new(observation.peak, observation.work);
            let value = step(&mut exact, kind, initial).unwrap().unwrap();
            assert_eq!(value.value, DocumentString::from(expected));
            let work_limit = observation.work - 1;
            let mut short_work = InputProcessing::new(observation.peak, work_limit);
            assert!(matches!(step(&mut short_work, kind, initial),
                Err(DomError::InputProcessingWorkLimit { limit }) if limit == work_limit));
            assert_eq!(short_work.live, 0);
            let scratch_limit = observation.peak - 1;
            let mut short_scratch = InputProcessing::new(scratch_limit, observation.work);
            assert!(matches!(step(&mut short_scratch, kind, initial),
                Err(DomError::ResourceLimit {
                    resource: DomResource::InputProcessingScratch, limit
                }) if limit == scratch_limit));
            assert_eq!(short_scratch.live, 0);
        }
    }
}

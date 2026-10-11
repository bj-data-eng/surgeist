//! Captured time datetime selection and source-ordered exact interpretation.
//! HTML owns the temporal syntax and numeric primitives; this consumer owns
//! admission through the existing semantic session. No host zone is inferred.
//! Recognition is bounded by admitted UTF-16 length; numeric operations are
//! bounded by admitted digit length. The session polls between these atomic
//! operations, not during each instruction or at the parser's fixed quantum.
//! Observed variable-buffer capacity is cumulative, including dropped buffers
//! and later failures. Shared immutable input backing and fixed wrappers are
//! outside that accounting.

use super::*;
use crate::numeric::{
    NumericMaterializationError, NumericMaterializationFailure, NumericMaterialized,
};
use crate::temporal::{
    BorrowedDate, BorrowedDurationPlan, BorrowedGlobalDateTime, BorrowedTemporalPlan, BorrowedYear,
    TemporalAlternative, TemporalPhase, recognize_time_alternative,
    temporal_recognition_work_bound,
};
use crate::{
    HtmlDate, HtmlMachineReadableTime, HtmlNonNegativeDecimal, HtmlNonNegativeInteger,
    HtmlTemporalError, HtmlTimeElementValue, HtmlValueReport, HtmlYear,
};
use std::num::NonZeroU32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementValueOrigin {
    DatetimeAttribute,
    ChildText,
}

/// The independent author restriction on absent-datetime element descendants.
/// This does not certify that the datetime value conveys human-readable meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementDescendantAuthorState {
    NotRequired,
    Conforming,
    Nonconforming(HtmlCaptureSubjectId),
}

/// Actual datetime selection, lexical attempts and descendant author observation.
/// Borrowed accessors retain exact raw spelling and selecting capture evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct HtmlTimeElementReadback {
    // Exactly one fallibly admitted record, with no successful empty/multi value.
    data: Vec<HtmlTimeElementReadbackData>,
}
#[derive(Clone, Debug, PartialEq)]
struct HtmlTimeElementReadbackData {
    report: HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
    datetime_attribute: Option<Attribute>,
    origin: HtmlTimeElementValueOrigin,
    descendants: HtmlTimeElementDescendantAuthorState,
}
impl HtmlTimeElementReadback {
    pub fn value_report(&self) -> &HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime> {
        &self.data[0].report
    }
    pub fn datetime_attribute(&self) -> Option<&Attribute> {
        self.data[0].datetime_attribute.as_ref()
    }
    pub fn origin(&self) -> HtmlTimeElementValueOrigin {
        self.data[0].origin
    }
    pub fn descendant_author_state(&self) -> &HtmlTimeElementDescendantAuthorState {
        &self.data[0].descendants
    }
}

struct TimeElementAttempts {
    author: Result<HtmlTimeElementValue, HtmlTemporalError>,
    recovery: Result<HtmlMachineReadableTime, HtmlTemporalError>,
}

impl HtmlSemanticSession<'_> {
    pub(super) fn time_element(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTimeElementReadback>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Time) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let datetime_attribute = self.attribute(element, "datetime")?;
        let (raw, origin, descendants) = if let Some(attribute) = datetime_attribute {
            self.charge(1)?;
            (
                attribute.value().clone(),
                HtmlTimeElementValueOrigin::DatetimeAttribute,
                HtmlTimeElementDescendantAuthorState::NotRequired,
            )
        } else {
            let raw = match self.control_text_content(subject, HtmlControlTextOrigin::ChildText)? {
                HtmlSemanticValue::Known(raw) => raw,
                HtmlSemanticValue::Missing(missing) => {
                    return Ok(HtmlSemanticValue::Missing(missing));
                }
                HtmlSemanticValue::Inapplicable => return Ok(HtmlSemanticValue::Inapplicable),
                HtmlSemanticValue::Unsupported(reason) => {
                    return Ok(HtmlSemanticValue::Unsupported(reason));
                }
            };
            let node = self.node(subject)?;
            self.charge(1)?;
            let HtmlCapturedChildren::Complete(mut children) = node.children() else {
                unreachable!("the same immutable child sequence was complete for text selection")
            };
            let mut descendants = HtmlTimeElementDescendantAuthorState::Conforming;
            loop {
                self.charge(1)?;
                let Some(child) = children.next() else {
                    break;
                };
                if matches!(self.node(&child)?.kind(), HtmlCapturedNodeKind::Element(_)) {
                    // Checked capture edges forbid Document/Fragment children;
                    // permitted non-elements cannot contain ordinary children.
                    descendants = HtmlTimeElementDescendantAuthorState::Nonconforming(child);
                    break;
                }
            }
            (raw, HtmlTimeElementValueOrigin::ChildText, descendants)
        };
        self.charge(1)?;
        self.interpreter
            .admit_attribute_raw(Some(&raw))
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        let attempts = self.time_element_attempts(&raw)?;
        self.charge(1)?;
        let report = self
            .interpreter
            .time_element_report_from_attempts(&raw, attempts.author, attempts.recovery)
            .map_err(|error| self.stop(HtmlSemanticFailureKind::Attribute(error)))?;
        let mut data = Vec::new();
        self.reserve_nested_output(&mut data, 1)?;
        self.charge(1)?;
        data.push(HtmlTimeElementReadbackData {
            report,
            datetime_attribute: datetime_attribute.cloned(),
            origin,
            descendants,
        });
        Ok(HtmlSemanticValue::Known(HtmlTimeElementReadback { data }))
    }

    fn time_element_attempts(
        &mut self,
        raw: &DocumentString,
    ) -> Result<TimeElementAttempts, HtmlSemanticFailure> {
        let mut author = Err(HtmlTemporalError::InvalidSyntax);
        for alternative in TemporalAlternative::ALL {
            self.time_work(temporal_recognition_work_bound(raw.len()))?;
            match recognize_time_alternative(raw.code_units(), TemporalPhase::Author, alternative) {
                Ok(plan) => {
                    author = Ok(self.time_materialize_author(plan)?);
                    break;
                }
                Err(error) => author = Err(error),
            }
        }
        let mut recovery = Err(HtmlTemporalError::InvalidSyntax);
        for alternative in TemporalAlternative::ALL {
            self.time_work(temporal_recognition_work_bound(raw.len()))?;
            match recognize_time_alternative(raw.code_units(), TemporalPhase::Recovery, alternative)
            {
                Ok(plan) => {
                    recovery = Ok(self.time_materialize_recovery(plan)?);
                    break;
                }
                Err(error) => recovery = Err(error),
            }
        }
        self.poll()?;
        Ok(TimeElementAttempts { author, recovery })
    }

    fn time_materialize_author(
        &mut self,
        plan: BorrowedTemporalPlan<'_>,
    ) -> Result<HtmlTimeElementValue, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(match plan {
            BorrowedTemporalPlan::Month(plan) => {
                HtmlTimeElementValue::Month(plan.assemble(self.time_year(plan.year())?))
            }
            BorrowedTemporalPlan::Date(plan) => HtmlTimeElementValue::Date(self.time_date(plan)?),
            BorrowedTemporalPlan::YearlessDate(value) => HtmlTimeElementValue::YearlessDate(value),
            BorrowedTemporalPlan::Time(plan) => HtmlTimeElementValue::Time(plan.assemble_author()),
            BorrowedTemporalPlan::LocalDateTime(plan) => HtmlTimeElementValue::LocalDateTime(
                plan.assemble_author(self.time_date(plan.date())?),
            ),
            BorrowedTemporalPlan::TimeZoneOffset(value) => {
                HtmlTimeElementValue::TimeZoneOffset(value)
            }
            BorrowedTemporalPlan::GlobalDateTime(plan) => HtmlTimeElementValue::GlobalDateTime(
                plan.assemble_author(self.time_date(plan.local().date())?),
            ),
            BorrowedTemporalPlan::Week(plan) => {
                HtmlTimeElementValue::Week(plan.assemble(self.time_year(plan.year())?))
            }
            BorrowedTemporalPlan::Year(plan) => HtmlTimeElementValue::Year(self.time_year(plan)?),
            BorrowedTemporalPlan::Duration(plan) => {
                let (seconds, millisecond) = self.time_duration_author(plan)?;
                HtmlTimeElementValue::Duration(plan.assemble_author(seconds, millisecond))
            }
        })
    }

    fn time_materialize_recovery(
        &mut self,
        plan: BorrowedTemporalPlan<'_>,
    ) -> Result<HtmlMachineReadableTime, HtmlSemanticFailure> {
        self.charge(1)?;
        Ok(match plan {
            BorrowedTemporalPlan::Month(plan) => {
                HtmlMachineReadableTime::Month(plan.assemble(self.time_year(plan.year())?))
            }
            BorrowedTemporalPlan::Date(plan) => {
                HtmlMachineReadableTime::Date(self.time_date(plan)?)
            }
            BorrowedTemporalPlan::YearlessDate(value) => {
                HtmlMachineReadableTime::YearlessDate(value)
            }
            BorrowedTemporalPlan::Time(plan) => {
                HtmlMachineReadableTime::Time(plan.assemble_recovery(
                    self.time_decimal(plan.second_digits(), plan.fraction_digits())?,
                ))
            }
            BorrowedTemporalPlan::LocalDateTime(plan) => {
                let date = self.time_date(plan.date())?;
                let second =
                    self.time_decimal(plan.time().second_digits(), plan.time().fraction_digits())?;
                HtmlMachineReadableTime::LocalDateTime(plan.assemble_recovery(date, second))
            }
            BorrowedTemporalPlan::TimeZoneOffset(value) => {
                HtmlMachineReadableTime::TimeZoneOffset(value)
            }
            BorrowedTemporalPlan::GlobalDateTime(plan) => self.time_global(plan)?,
            BorrowedTemporalPlan::Week(plan) => {
                HtmlMachineReadableTime::Week(plan.assemble(self.time_year(plan.year())?))
            }
            BorrowedTemporalPlan::Year(plan) => {
                HtmlMachineReadableTime::Year(self.time_year(plan)?)
            }
            BorrowedTemporalPlan::Duration(plan) => HtmlMachineReadableTime::Duration(
                plan.assemble_recovery(self.time_duration_recovery(plan)?),
            ),
        })
    }

    fn time_year(&mut self, plan: BorrowedYear<'_>) -> Result<HtmlYear, HtmlSemanticFailure> {
        let value = self.time_integer(plan.digits())?;
        self.charge(1)?;
        Ok(plan.assemble(value))
    }
    fn time_date(&mut self, plan: BorrowedDate<'_>) -> Result<HtmlDate, HtmlSemanticFailure> {
        let year = self.time_year(plan.year())?;
        self.charge(1)?;
        Ok(plan.assemble(year))
    }
    fn time_integer(
        &mut self,
        digits: &[u16],
    ) -> Result<HtmlNonNegativeInteger, HtmlSemanticFailure> {
        self.time_work(HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
            digits.len(),
        ))?;
        let result = HtmlNonNegativeInteger::try_from_borrowed_digits(
            digits,
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        self.time_numeric_result(result)
    }
    fn time_constant(&mut self, value: u32) -> Result<HtmlNonNegativeInteger, HtmlSemanticFailure> {
        self.time_work(HtmlNonNegativeInteger::from_u32_work_bound())?;
        let result = HtmlNonNegativeInteger::try_from_u32(
            value,
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        self.time_numeric_result(result)
    }
    fn time_decimal(
        &mut self,
        integer: &[u16],
        fraction: &[u16],
    ) -> Result<HtmlNonNegativeDecimal, HtmlSemanticFailure> {
        self.time_work(HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
            integer.len(),
            fraction.len(),
        ))?;
        let result = HtmlNonNegativeDecimal::try_from_borrowed_digits(
            integer,
            fraction,
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        self.time_numeric_result(result)
    }

    fn time_global(
        &mut self,
        plan: BorrowedGlobalDateTime<'_>,
    ) -> Result<HtmlMachineReadableTime, HtmlSemanticFailure> {
        let local_plan = plan.local();
        let date_plan = local_plan.date();
        let year = self.time_integer(date_plan.year().digits())?;
        // The local year remains in the output while arithmetic consumes another
        // admitted exact magnitude; no native-value Clone is hidden here.
        self.time_work(year.copy_work_bound())?;
        let result = year.try_copy(self.limits.integer_digits, self.remaining_allocated_bytes());
        let local_year = self.time_numeric_result(result)?;
        self.charge(1)?;
        let date = date_plan.assemble(date_plan.year().assemble(local_year));
        self.time_work(year.decremented_work_bound())?;
        let result =
            year.try_decremented(self.limits.integer_digits, self.remaining_allocated_bytes());
        let preceding = self
            .time_numeric_result(result)?
            .expect("recognized HTML year is positive");
        self.time_work(preceding.quotient_remainder_work_bound())?;
        let result = preceding.try_quotient_remainder(
            NonZeroU32::new(400).expect("400 is nonzero"),
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        let (cycles, _) = self.time_numeric_result(result)?;
        self.time_work(cycles.multiplied_by_work_bound())?;
        let result = cycles.try_multiplied_by(
            146097,
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        let cycle_days = self.time_numeric_result(result)?;
        // Temporal owner bounds its fixed Gregorian month visits independently.
        self.charge(256)?;
        let within_cycle = self.time_constant(date_plan.within_cycle_days())?;
        self.time_work(cycle_days.added_to_work_bound(&within_cycle))?;
        let result = cycle_days.try_added_to(
            &within_cycle,
            self.limits.integer_digits,
            self.remaining_allocated_bytes(),
        );
        let ordinal = self.time_numeric_result(result)?;
        self.charge(1)?;
        let mut day = HtmlInteger::from_magnitude(ordinal, false);
        self.charge(1)?;
        let adjustment = plan.utc_day_adjustment();
        if adjustment != 0 {
            self.time_work(day.materialized_unit_step_work_bound())?;
            let result = day.try_materialized_unit_step(
                if adjustment < 0 {
                    IntegerUnitStep::Decrement
                } else {
                    IntegerUnitStep::Increment
                },
                self.limits.integer_digits,
                self.remaining_allocated_bytes(),
            );
            day = self.time_numeric_result(result)?;
        }
        let seconds = self.time_decimal(
            local_plan.time().second_digits(),
            local_plan.time().fraction_digits(),
        )?;
        self.time_work(seconds.copy_work_bound())?;
        let result = seconds.try_copy(self.limits.integer_digits, self.remaining_allocated_bytes());
        let utc_seconds = self.time_numeric_result(result)?;
        self.charge(1)?;
        let local = local_plan.assemble_recovery(date, seconds);
        self.charge(1)?;
        Ok(HtmlMachineReadableTime::GlobalDateTime(
            plan.assemble_recovery(local, day, utc_seconds),
        ))
    }

    fn time_duration_author(
        &mut self,
        plan: BorrowedDurationPlan<'_>,
    ) -> Result<(HtmlNonNegativeInteger, u16), HtmlSemanticFailure> {
        let mut seconds = self.time_constant(0)?;
        let mut millisecond = 0;
        let mut cursor = plan.cursor();
        loop {
            self.time_work(cursor.next_work_bound())?;
            let component = cursor
                .next_component()
                .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            let Some(component) = component else {
                break;
            };
            let Some(scale) = component.scale() else {
                continue;
            };
            let quantity = self.time_integer(component.integer_digits())?;
            self.time_work(quantity.multiplied_by_work_bound())?;
            let result = quantity.try_multiplied_by(
                scale,
                self.limits.integer_digits,
                self.remaining_allocated_bytes(),
            );
            let scaled = self.time_numeric_result(result)?;
            self.time_work(seconds.added_to_work_bound(&scaled))?;
            let result = seconds.try_added_to(
                &scaled,
                self.limits.integer_digits,
                self.remaining_allocated_bytes(),
            );
            seconds = self.time_numeric_result(result)?;
            if scale == 1 {
                millisecond = component.millisecond();
            }
        }
        Ok((seconds, millisecond))
    }
    fn time_duration_recovery(
        &mut self,
        plan: BorrowedDurationPlan<'_>,
    ) -> Result<HtmlNonNegativeDecimal, HtmlSemanticFailure> {
        let mut seconds: HtmlNonNegativeDecimal = self.time_constant(0)?.into();
        let mut cursor = plan.cursor();
        loop {
            self.time_work(cursor.next_work_bound())?;
            let component = cursor
                .next_component()
                .map_err(|_| self.stop(HtmlSemanticFailureKind::IntrinsicUnavailable))?;
            let Some(component) = component else {
                break;
            };
            let Some(scale) = component.scale() else {
                continue;
            };
            let quantity =
                self.time_decimal(component.integer_digits(), component.fraction_digits())?;
            self.time_work(quantity.multiplied_by_work_bound())?;
            let result = quantity.try_multiplied_by(
                scale,
                self.limits.integer_digits,
                self.remaining_allocated_bytes(),
            );
            let scaled = self.time_numeric_result(result)?;
            self.time_work(seconds.added_to_work_bound(&scaled))?;
            let result = seconds.try_added_to(
                &scaled,
                self.limits.integer_digits,
                self.remaining_allocated_bytes(),
            );
            seconds = self.time_numeric_result(result)?;
        }
        Ok(seconds)
    }

    fn time_work(&mut self, bound: Option<usize>) -> Result<(), HtmlSemanticFailure> {
        let bound =
            bound.ok_or_else(|| self.stop(HtmlSemanticFailureKind::RepresentationExhausted))?;
        self.charge(bound)
    }

    /// These operations can allocate several buffers and fail after an earlier
    /// successful allocation. Record cumulative observed bytes before mapping
    /// failure or polling; the ordinal single-allocation adapter is not reused.
    fn time_numeric_result<T>(
        &mut self,
        result: Result<NumericMaterialized<T>, NumericMaterializationFailure>,
    ) -> Result<T, HtmlSemanticFailure> {
        let baseline = self.usage.allocated_bytes;
        let observed = match &result {
            Ok(value) => value.capacity_bytes,
            Err(error) => error.capacity_bytes,
        };
        self.record_allocated_bytes(observed)?;
        let value = result
            .map_err(|failure| {
                let kind = match failure.kind {
                    NumericMaterializationError::Digits(_) => {
                        HtmlSemanticFailureKind::IntrinsicUnavailable
                    }
                    NumericMaterializationError::Boundary(error) => match error {
                        IntegerUnitStepError::RepresentationExhausted => {
                            HtmlSemanticFailureKind::RepresentationExhausted
                        }
                        IntegerUnitStepError::AllocationFailed { .. } => {
                            HtmlSemanticFailureKind::AllocationFailed
                        }
                        IntegerUnitStepError::DigitLimitExceeded { limit, required } => {
                            HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::IntegerDigits,
                                limit,
                                required,
                            }
                        }
                        IntegerUnitStepError::RequestedCapacityLimitExceeded {
                            required, ..
                        } => match baseline.checked_add(required) {
                            Some(required) => HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required,
                            },
                            None => HtmlSemanticFailureKind::RepresentationExhausted,
                        },
                        IntegerUnitStepError::ActualCapacityLimitExceeded { .. } => {
                            HtmlSemanticFailureKind::LimitExceeded {
                                resource: HtmlSemanticResource::AllocatedBytes,
                                limit: self.limits.allocated_bytes,
                                required: self.usage.allocated_bytes,
                            }
                        }
                    },
                };
                self.stop(kind)
            })?
            .value;
        self.poll()?;
        Ok(value)
    }
}

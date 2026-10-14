//! Retained time-element recognition and admitted exact materialization.
use super::*;
use crate::numeric::{
    IntegerUnitStep, NumericMaterializationError, NumericMaterializationFailure,
    NumericMaterialized,
};
use crate::{HtmlValueAdmissionError, HtmlValueInterpreter, HtmlValueReport};

/// A refused owner transition or latched processing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlTimeElementInterpretationError {
    NotReady,
    Consumed,
    RepresentationExhausted,
    Processing(NumericMaterializationError),
    InvalidRecognizedPlan,
}
impl std::fmt::Display for HtmlTimeElementInterpretationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "time-element interpretation: {self:?}")
    }
}
impl std::error::Error for HtmlTimeElementInterpretationError {}

/// The next operation's role; no borrowed temporal plan crosses this boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementInterpretationActionKind {
    Recognition,
    Materialization,
    Assembly,
    Poll,
    ReportAssembly,
}
/// Work to admit before execution and whether the operation can allocate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTimeElementInterpretationLayout {
    work: usize,
    kind: HtmlTimeElementInterpretationActionKind,
}
impl HtmlTimeElementInterpretationLayout {
    /// Original semantic work credits. A zero-credit Assembly completes an
    /// already admitted numeric operation after its post-operation poll.
    pub fn work(self) -> usize {
        self.work
    }
    pub fn kind(self) -> HtmlTimeElementInterpretationActionKind {
        self.kind
    }
    pub fn can_allocate(self) -> bool {
        self.kind == HtmlTimeElementInterpretationActionKind::Materialization
    }
}
/// Actual capacity is observed before interpreting the outcome or polling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTimeElementInterpretationStep {
    capacity: usize,
    outcome: Result<(), HtmlTimeElementInterpretationError>,
    poll_after: bool,
}
impl HtmlTimeElementInterpretationStep {
    /// Capacity observed by this action, including on failure.
    pub fn capacity_bytes(self) -> usize {
        self.capacity
    }
    pub fn outcome(self) -> Result<(), HtmlTimeElementInterpretationError> {
        self.outcome
    }
    /// Poll after a successful outcome and capacity observation, before any
    /// following assembly. Failed outcomes retain their original failure.
    pub fn poll_after(self) -> bool {
        self.poll_after
    }
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    Recognize,
    Begin,
    YearRead,
    YearAssemble,
    DateAssemble,
    Second,
    SecondAssemble,
    Global(GlobalStage),
    Duration(DurationStage),
    Poll,
    Report,
    Done,
}
#[derive(Clone, Copy, Debug)]
enum GlobalStage {
    Year,
    CopyYear,
    Date,
    PrecedingYear,
    Cycles,
    CycleDays,
    WithinCycle,
    WithinCycleNumber,
    Ordinal,
    Day,
    Adjustment,
    AdjustDay,
    Seconds,
    CopySeconds,
    Local,
    Finish,
}
impl GlobalStage {
    fn next(self) -> Option<Self> {
        match self {
            Self::Year => Some(Self::CopyYear),
            Self::CopyYear => Some(Self::Date),
            Self::Date => Some(Self::PrecedingYear),
            Self::PrecedingYear => Some(Self::Cycles),
            Self::Cycles => Some(Self::CycleDays),
            Self::CycleDays => Some(Self::WithinCycle),
            Self::WithinCycle => Some(Self::WithinCycleNumber),
            Self::WithinCycleNumber => Some(Self::Ordinal),
            Self::Ordinal => Some(Self::Day),
            Self::Day => Some(Self::Adjustment),
            Self::Adjustment => Some(Self::AdjustDay),
            Self::AdjustDay => Some(Self::Seconds),
            Self::Seconds => Some(Self::CopySeconds),
            Self::CopySeconds => Some(Self::Local),
            Self::Local => Some(Self::Finish),
            Self::Finish => None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
enum DurationStage {
    Zero,
    Initialize,
    Next,
    Quantity,
    Scale,
    Accumulate,
    Advance,
}
impl DurationStage {
    fn next(self) -> Self {
        match self {
            Self::Zero => Self::Initialize,
            Self::Initialize => Self::Next,
            Self::Next => Self::Quantity,
            Self::Quantity => Self::Scale,
            Self::Scale => Self::Accumulate,
            Self::Accumulate => Self::Advance,
            Self::Advance => Self::Next,
        }
    }
}
type Report = HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>;

/// One exact borrowed operand and its unpublished recognition/materialization.
/// Construction admits only the raw length; no grammar or allocation runs.
pub struct HtmlTimeElementInterpretation<'raw> {
    raw: &'raw DocumentString,
    interpreter: HtmlValueInterpreter,
    phase: TemporalPhase,
    alternative: usize,
    stage: Stage,
    plan: Option<BorrowedTemporalPlan<'raw>>,
    author: Result<HtmlTimeElementValue, HtmlTemporalError>,
    recovery: Result<HtmlMachineReadableTime, HtmlTemporalError>,
    integers: [Option<HtmlNonNegativeInteger>; 5],
    decimals: [Option<HtmlNonNegativeDecimal>; 3],
    year: Option<HtmlYear>,
    date: Option<HtmlDate>,
    day: Option<HtmlInteger>,
    local: Option<HtmlParsedLocalDateTime>,
    duration: Option<BorrowedDurationCursor<'raw>>,
    component: Option<BorrowedDurationComponent<'raw>>,
    millisecond: u16,
    within_cycle: u32,
    adjustment: i8,
    report: Option<Report>,
    failure: Option<HtmlTimeElementInterpretationError>,
}
impl<'raw> HtmlTimeElementInterpretation<'raw> {
    /// Admits raw UTF-16 length without recognition, cloning or allocation.
    pub fn new(
        raw: &'raw DocumentString,
        interpreter: HtmlValueInterpreter,
    ) -> Result<Self, HtmlValueAdmissionError> {
        interpreter.admit_attribute_raw(Some(raw))?;
        Ok(Self {
            raw,
            interpreter,
            phase: TemporalPhase::Author,
            alternative: 0,
            stage: Stage::Recognize,
            plan: None,
            author: Err(HtmlTemporalError::InvalidSyntax),
            recovery: Err(HtmlTemporalError::InvalidSyntax),
            integers: std::array::from_fn(|_| None),
            decimals: std::array::from_fn(|_| None),
            year: None,
            date: None,
            day: None,
            local: None,
            duration: None,
            component: None,
            millisecond: 0,
            within_cycle: 0,
            adjustment: 0,
            report: None,
            failure: None,
        })
    }
    /// Borrows the exact original operand, including malformed UTF-16.
    pub fn raw(&self) -> &'raw DocumentString {
        self.raw
    }
    /// Drops unpublished values and returns the original operand.
    /// Previously observed allocations receive no resource credit.
    pub fn abort(self) -> &'raw DocumentString {
        self.raw
    }
    /// Describes an exclusive next action before executing its work.
    pub fn next_action(
        &mut self,
    ) -> Result<
        Option<HtmlTimeElementInterpretationAction<'_, 'raw>>,
        HtmlTimeElementInterpretationError,
    > {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if matches!(self.stage, Stage::Done) {
            return Ok(None);
        }
        let Some(layout) = self.layout() else {
            self.failure = Some(HtmlTimeElementInterpretationError::RepresentationExhausted);
            return Err(HtmlTimeElementInterpretationError::RepresentationExhausted);
        };
        Ok(Some(HtmlTimeElementInterpretationAction {
            owner: self,
            layout,
        }))
    }
    /// Publishes only a completed report; every refusal retains Some.
    pub fn try_finish(
        owner: &mut Option<Self>,
    ) -> Result<
        HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
        HtmlTimeElementInterpretationError,
    > {
        let current = owner
            .as_ref()
            .ok_or(HtmlTimeElementInterpretationError::Consumed)?;
        if let Some(error) = current.failure {
            return Err(error);
        }
        if !matches!(current.stage, Stage::Done) {
            return Err(HtmlTimeElementInterpretationError::NotReady);
        }
        Ok(owner
            .take()
            .expect("checked owner")
            .report
            .expect("completed admitted report assembly"))
    }
    fn integer(&self, index: usize) -> &HtmlNonNegativeInteger {
        self.integers[index]
            .as_ref()
            .expect("materialized integer register")
    }
    fn decimal(&self, index: usize) -> &HtmlNonNegativeDecimal {
        self.decimals[index]
            .as_ref()
            .expect("materialized decimal register")
    }
    fn plan(&self) -> BorrowedTemporalPlan<'raw> {
        self.plan.expect("recognized plan")
    }
    fn year_plan(&self) -> BorrowedYear<'raw> {
        match self.plan() {
            BorrowedTemporalPlan::Month(p) => p.year(),
            BorrowedTemporalPlan::Date(p) => p.year(),
            BorrowedTemporalPlan::LocalDateTime(p) => p.date().year(),
            BorrowedTemporalPlan::GlobalDateTime(p) => p.local().date().year(),
            BorrowedTemporalPlan::Week(p) => p.year(),
            BorrowedTemporalPlan::Year(p) => p,
            _ => unreachable!("year materialization branch"),
        }
    }
    fn date_plan(&self) -> BorrowedDate<'raw> {
        match self.plan() {
            BorrowedTemporalPlan::Date(p) => p,
            BorrowedTemporalPlan::LocalDateTime(p) => p.date(),
            BorrowedTemporalPlan::GlobalDateTime(p) => p.local().date(),
            _ => unreachable!("date materialization branch"),
        }
    }
    fn time_plan(&self) -> BorrowedTime<'raw> {
        match self.plan() {
            BorrowedTemporalPlan::Time(p) => p,
            BorrowedTemporalPlan::LocalDateTime(p) => p.time(),
            BorrowedTemporalPlan::GlobalDateTime(p) => p.local().time(),
            _ => unreachable!("time materialization branch"),
        }
    }
    fn global_plan(&self) -> BorrowedGlobalDateTime<'raw> {
        let BorrowedTemporalPlan::GlobalDateTime(p) = self.plan() else {
            unreachable!("global materialization branch")
        };
        p
    }
    fn layout(&self) -> Option<HtmlTimeElementInterpretationLayout> {
        use HtmlTimeElementInterpretationActionKind as K;
        let (kind, work) = match self.stage {
            Stage::SecondAssemble => (K::Assembly, 0),
            Stage::Recognize => (
                K::Recognition,
                temporal_recognition_work_bound(self.raw.len())?,
            ),
            Stage::YearRead => (
                K::Materialization,
                HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                    self.year_plan().digits().len(),
                )?,
            ),
            Stage::Second => {
                let p = self.time_plan();
                (
                    K::Materialization,
                    HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
                        p.second_digits().len(),
                        p.fraction_digits().len(),
                    )?,
                )
            }
            Stage::Global(step) => match step {
                GlobalStage::Year => (
                    K::Materialization,
                    HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                        self.date_plan().year().digits().len(),
                    )?,
                ),
                GlobalStage::CopyYear => (K::Materialization, self.integer(0).copy_work_bound()?),
                GlobalStage::PrecedingYear => (
                    K::Materialization,
                    self.integer(0).decremented_work_bound()?,
                ),
                GlobalStage::Cycles => (
                    K::Materialization,
                    self.integer(2).quotient_remainder_work_bound()?,
                ),
                GlobalStage::CycleDays => (
                    K::Materialization,
                    self.integer(3).multiplied_by_work_bound()?,
                ),
                GlobalStage::WithinCycle => (K::Assembly, 256),
                GlobalStage::WithinCycleNumber => (
                    K::Materialization,
                    HtmlNonNegativeInteger::from_u32_work_bound()?,
                ),
                GlobalStage::Ordinal => (
                    K::Materialization,
                    self.integer(4).added_to_work_bound(self.integer(1))?,
                ),
                GlobalStage::AdjustDay => (
                    K::Materialization,
                    self.day
                        .as_ref()
                        .expect("day register")
                        .materialized_unit_step_work_bound()?,
                ),
                GlobalStage::Seconds => {
                    let p = self.time_plan();
                    (
                        K::Materialization,
                        HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
                            p.second_digits().len(),
                            p.fraction_digits().len(),
                        )?,
                    )
                }
                GlobalStage::CopySeconds => {
                    (K::Materialization, self.decimal(0).copy_work_bound()?)
                }
                _ => (K::Assembly, 1),
            },
            Stage::Duration(step) => match step {
                DurationStage::Initialize | DurationStage::Advance => (K::Assembly, 0),
                DurationStage::Zero => (
                    K::Materialization,
                    HtmlNonNegativeInteger::from_u32_work_bound()?,
                ),
                DurationStage::Next => (
                    K::Recognition,
                    self.duration
                        .as_ref()
                        .expect("duration cursor")
                        .next_work_bound()?,
                ),
                DurationStage::Quantity => {
                    let p = self.component.expect("duration component");
                    (
                        K::Materialization,
                        if self.phase == TemporalPhase::Author {
                            HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                                p.integer_digits().len(),
                            )?
                        } else {
                            HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
                                p.integer_digits().len(),
                                p.fraction_digits().len(),
                            )?
                        },
                    )
                }
                DurationStage::Scale => (
                    K::Materialization,
                    if self.phase == TemporalPhase::Author {
                        self.integer(1).multiplied_by_work_bound()?
                    } else {
                        self.decimal(1).multiplied_by_work_bound()?
                    },
                ),
                DurationStage::Accumulate => (
                    K::Materialization,
                    if self.phase == TemporalPhase::Author {
                        self.integer(0).added_to_work_bound(self.integer(2))?
                    } else {
                        self.decimal(0).added_to_work_bound(self.decimal(2))?
                    },
                ),
            },
            Stage::Poll => (K::Poll, 0),
            Stage::Report => (K::ReportAssembly, 1),
            Stage::Done => return None,
            _ => (K::Assembly, 1),
        };
        Some(HtmlTimeElementInterpretationLayout { work, kind })
    }
    fn complete_phase(&mut self) {
        self.integers = std::array::from_fn(|_| None);
        self.decimals = std::array::from_fn(|_| None);
        self.year = None;
        self.date = None;
        self.day = None;
        self.local = None;
        self.duration = None;
        self.component = None;
        self.plan = None;
        if self.phase == TemporalPhase::Author {
            self.phase = TemporalPhase::Recovery;
            self.alternative = 0;
            self.stage = Stage::Recognize;
        } else {
            self.stage = Stage::Poll;
        }
    }
    fn author(&mut self, value: HtmlTimeElementValue) {
        self.author = Ok(value);
        self.complete_phase();
    }
    fn recovery(&mut self, value: HtmlMachineReadableTime) {
        self.recovery = Ok(value);
        self.complete_phase();
    }
    fn advance(
        &mut self,
        digits: usize,
        bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlTimeElementInterpretationError> {
        match self.stage {
            Stage::Recognize => match recognize_time_alternative(
                self.raw.code_units(),
                self.phase,
                TemporalAlternative::ALL[self.alternative],
            ) {
                Ok(plan) => {
                    self.plan = Some(plan);
                    self.stage = Stage::Begin;
                }
                Err(error) => {
                    if self.phase == TemporalPhase::Author {
                        self.author = Err(error);
                    } else {
                        self.recovery = Err(error);
                    }
                    self.alternative += 1;
                    if self.alternative == TemporalAlternative::ALL.len() {
                        self.complete_phase();
                    }
                }
            },
            Stage::Begin => match self.plan() {
                BorrowedTemporalPlan::YearlessDate(value) => {
                    if self.phase == TemporalPhase::Author {
                        self.author(HtmlTimeElementValue::YearlessDate(value));
                    } else {
                        self.recovery(HtmlMachineReadableTime::YearlessDate(value));
                    }
                }
                BorrowedTemporalPlan::TimeZoneOffset(value) => {
                    if self.phase == TemporalPhase::Author {
                        self.author(HtmlTimeElementValue::TimeZoneOffset(value));
                    } else {
                        self.recovery(HtmlMachineReadableTime::TimeZoneOffset(value));
                    }
                }
                BorrowedTemporalPlan::Time(value) => {
                    if self.phase == TemporalPhase::Author {
                        self.author(HtmlTimeElementValue::Time(value.assemble_author()));
                    } else {
                        self.stage = Stage::Second;
                    }
                }
                BorrowedTemporalPlan::GlobalDateTime(_)
                    if self.phase == TemporalPhase::Recovery =>
                {
                    self.stage = Stage::Global(GlobalStage::Year)
                }
                BorrowedTemporalPlan::Duration(_) => {
                    self.stage = Stage::Duration(DurationStage::Zero);
                }
                _ => self.stage = Stage::YearRead,
            },
            Stage::YearRead => {
                self.integers[0] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_borrowed_digits(
                        self.year_plan().digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?);
                self.stage = Stage::YearAssemble;
            }
            Stage::YearAssemble => {
                let year = self
                    .year_plan()
                    .assemble(self.integers[0].take().expect("year magnitude"));
                match self.plan() {
                    BorrowedTemporalPlan::Month(p) => {
                        if self.phase == TemporalPhase::Author {
                            self.author(HtmlTimeElementValue::Month(p.assemble(year)));
                        } else {
                            self.recovery(HtmlMachineReadableTime::Month(p.assemble(year)));
                        }
                    }
                    BorrowedTemporalPlan::Week(p) => {
                        if self.phase == TemporalPhase::Author {
                            self.author(HtmlTimeElementValue::Week(p.assemble(year)));
                        } else {
                            self.recovery(HtmlMachineReadableTime::Week(p.assemble(year)));
                        }
                    }
                    BorrowedTemporalPlan::Year(_) => {
                        if self.phase == TemporalPhase::Author {
                            self.author(HtmlTimeElementValue::Year(year));
                        } else {
                            self.recovery(HtmlMachineReadableTime::Year(year));
                        }
                    }
                    _ => {
                        self.year = Some(year);
                        self.stage = Stage::DateAssemble;
                    }
                }
            }
            Stage::DateAssemble => {
                let date = self.date_plan().assemble(self.year.take().expect("year"));
                match self.plan() {
                    BorrowedTemporalPlan::Date(_) => {
                        if self.phase == TemporalPhase::Author {
                            self.author(HtmlTimeElementValue::Date(date));
                        } else {
                            self.recovery(HtmlMachineReadableTime::Date(date));
                        }
                    }
                    BorrowedTemporalPlan::LocalDateTime(p) => {
                        if self.phase == TemporalPhase::Author {
                            self.author(HtmlTimeElementValue::LocalDateTime(
                                p.assemble_author(date),
                            ));
                        } else {
                            self.date = Some(date);
                            self.stage = Stage::Second;
                        }
                    }
                    BorrowedTemporalPlan::GlobalDateTime(p) => self.author(
                        HtmlTimeElementValue::GlobalDateTime(p.assemble_author(date)),
                    ),
                    _ => unreachable!("date branch"),
                }
            }
            Stage::Second => {
                let p = self.time_plan();
                self.decimals[0] = Some(materialized(
                    HtmlNonNegativeDecimal::try_from_borrowed_digits(
                        p.second_digits(),
                        p.fraction_digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?);
                self.stage = Stage::SecondAssemble;
            }
            Stage::SecondAssemble => {
                let seconds = self.decimals[0].take().expect("recovery seconds");
                match self.plan() {
                    BorrowedTemporalPlan::Time(p) => {
                        self.recovery(HtmlMachineReadableTime::Time(p.assemble_recovery(seconds)))
                    }
                    BorrowedTemporalPlan::LocalDateTime(p) => {
                        let date = self.date.take().expect("local date");
                        self.recovery(HtmlMachineReadableTime::LocalDateTime(
                            p.assemble_recovery(date, seconds),
                        ));
                    }
                    _ => unreachable!("time branch"),
                }
            }
            Stage::Global(step) => self.advance_global(step, digits, bytes, observed)?,
            Stage::Duration(step) => self.advance_duration(step, digits, bytes, observed)?,
            Stage::Poll => self.stage = Stage::Report,
            Stage::Report => {
                let author =
                    std::mem::replace(&mut self.author, Err(HtmlTemporalError::InvalidSyntax));
                let recovery =
                    std::mem::replace(&mut self.recovery, Err(HtmlTemporalError::InvalidSyntax));
                self.report = Some(
                    self.interpreter
                        .time_element_report_from_attempts(self.raw, author, recovery)
                        .expect("same interpreter and raw admitted at construction"),
                );
                self.stage = Stage::Done;
            }
            Stage::Done => unreachable!("completed owner issues no action"),
        }
        Ok(())
    }
    fn advance_global(
        &mut self,
        step: GlobalStage,
        digits: usize,
        bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlTimeElementInterpretationError> {
        let p = self.global_plan();
        match step {
            GlobalStage::Year => {
                self.integers[0] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_borrowed_digits(
                        p.local().date().year().digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?)
            }
            GlobalStage::CopyYear => {
                self.integers[1] = Some(materialized(
                    self.integer(0).try_copy(digits, bytes),
                    observed,
                )?)
            }
            GlobalStage::Date => {
                self.date = Some(
                    p.local().date().assemble(
                        p.local()
                            .date()
                            .year()
                            .assemble(self.integers[1].take().expect("copied local year")),
                    ),
                )
            }
            GlobalStage::PrecedingYear => {
                self.integers[2] = Some(
                    materialized(self.integer(0).try_decremented(digits, bytes), observed)?
                        .expect("positive recognized year"),
                )
            }
            GlobalStage::Cycles => {
                self.integers[3] = Some(
                    materialized(
                        self.integer(2).try_quotient_remainder(
                            NonZeroU32::new(400).expect("nonzero divisor"),
                            digits,
                            bytes,
                        ),
                        observed,
                    )?
                    .0,
                )
            }
            GlobalStage::CycleDays => {
                self.integers[4] = Some(materialized(
                    self.integer(3).try_multiplied_by(146097, digits, bytes),
                    observed,
                )?)
            }
            GlobalStage::WithinCycle => self.within_cycle = p.local().date().within_cycle_days(),
            GlobalStage::WithinCycleNumber => {
                self.integers[1] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_u32(self.within_cycle, digits, bytes),
                    observed,
                )?)
            }
            GlobalStage::Ordinal => {
                self.integers[2] = Some(materialized(
                    self.integer(4).try_added_to(self.integer(1), digits, bytes),
                    observed,
                )?)
            }
            GlobalStage::Day => {
                self.day = Some(HtmlInteger::from_magnitude(
                    self.integers[2].take().expect("ordinal"),
                    false,
                ))
            }
            GlobalStage::Adjustment => {
                self.adjustment = p.utc_day_adjustment();
                if self.adjustment == 0 {
                    self.stage = Stage::Global(GlobalStage::Seconds);
                    return Ok(());
                }
            }
            GlobalStage::AdjustDay => {
                let direction = if self.adjustment < 0 {
                    IntegerUnitStep::Decrement
                } else {
                    IntegerUnitStep::Increment
                };
                self.day = Some(materialized(
                    self.day
                        .as_ref()
                        .expect("day")
                        .try_materialized_unit_step(direction, digits, bytes),
                    observed,
                )?);
            }
            GlobalStage::Seconds => {
                self.decimals[0] = Some(materialized(
                    HtmlNonNegativeDecimal::try_from_borrowed_digits(
                        p.local().time().second_digits(),
                        p.local().time().fraction_digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?)
            }
            GlobalStage::CopySeconds => {
                self.decimals[1] = Some(materialized(
                    self.decimal(0).try_copy(digits, bytes),
                    observed,
                )?)
            }
            GlobalStage::Local => {
                self.local = Some(p.local().assemble_recovery(
                    self.date.take().expect("local date"),
                    self.decimals[0].take().expect("local seconds"),
                ))
            }
            GlobalStage::Finish => {
                let value = p.assemble_recovery(
                    self.local.take().expect("local value"),
                    self.day.take().expect("UTC day"),
                    self.decimals[1].take().expect("UTC seconds"),
                );
                self.recovery(HtmlMachineReadableTime::GlobalDateTime(value));
                return Ok(());
            }
        }
        self.stage = Stage::Global(
            step.next()
                .expect("finishing stage returns before its successor"),
        );
        Ok(())
    }
    fn advance_duration(
        &mut self,
        step: DurationStage,
        digits: usize,
        bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlTimeElementInterpretationError> {
        match step {
            DurationStage::Zero => {
                let zero = materialized(
                    HtmlNonNegativeInteger::try_from_u32(0, digits, bytes),
                    observed,
                )?;
                if self.phase == TemporalPhase::Author {
                    self.integers[0] = Some(zero);
                } else {
                    self.decimals[0] = Some(zero.into());
                }
            }
            DurationStage::Initialize => {
                let BorrowedTemporalPlan::Duration(plan) = self.plan() else {
                    unreachable!("duration plan")
                };
                self.duration = Some(plan.cursor());
            }
            DurationStage::Next => {
                self.component = self
                    .duration
                    .as_mut()
                    .expect("duration cursor")
                    .next_component()
                    .map_err(|_| HtmlTimeElementInterpretationError::InvalidRecognizedPlan)?;
                let Some(component) = self.component else {
                    let BorrowedTemporalPlan::Duration(p) = self.plan() else {
                        unreachable!("duration plan")
                    };
                    if self.phase == TemporalPhase::Author {
                        let value = p.assemble_author(
                            self.integers[0].take().expect("accumulated seconds"),
                            self.millisecond,
                        );
                        self.author(HtmlTimeElementValue::Duration(value));
                    } else {
                        let value = p.assemble_recovery(
                            self.decimals[0].take().expect("accumulated seconds"),
                        );
                        self.recovery(HtmlMachineReadableTime::Duration(value));
                    }
                    return Ok(());
                };
                if component.scale().is_none() {
                    return Ok(());
                }
            }
            DurationStage::Quantity => {
                let p = self.component.expect("duration component");
                if self.phase == TemporalPhase::Author {
                    self.integers[1] = Some(materialized(
                        HtmlNonNegativeInteger::try_from_borrowed_digits(
                            p.integer_digits(),
                            digits,
                            bytes,
                        ),
                        observed,
                    )?);
                } else {
                    self.decimals[1] = Some(materialized(
                        HtmlNonNegativeDecimal::try_from_borrowed_digits(
                            p.integer_digits(),
                            p.fraction_digits(),
                            digits,
                            bytes,
                        ),
                        observed,
                    )?);
                }
            }
            DurationStage::Scale => {
                let scale = self
                    .component
                    .expect("duration component")
                    .scale()
                    .expect("numeric duration component");
                if self.phase == TemporalPhase::Author {
                    self.integers[2] = Some(materialized(
                        self.integer(1).try_multiplied_by(scale, digits, bytes),
                        observed,
                    )?);
                } else {
                    self.decimals[2] = Some(materialized(
                        self.decimal(1).try_multiplied_by(scale, digits, bytes),
                        observed,
                    )?);
                }
            }
            DurationStage::Accumulate => {
                if self.phase == TemporalPhase::Author {
                    self.integers[0] = Some(materialized(
                        self.integer(0).try_added_to(self.integer(2), digits, bytes),
                        observed,
                    )?);
                } else {
                    self.decimals[0] = Some(materialized(
                        self.decimal(0).try_added_to(self.decimal(2), digits, bytes),
                        observed,
                    )?);
                }
            }
            DurationStage::Advance => {
                if self.phase == TemporalPhase::Author {
                    let p = self.component.expect("component");
                    if p.scale() == Some(1) {
                        self.millisecond = p.millisecond();
                    }
                }
                self.integers[1] = None;
                self.integers[2] = None;
                self.decimals[1] = None;
                self.decimals[2] = None;
                self.stage = Stage::Duration(DurationStage::Next);
                return Ok(());
            }
        }
        self.stage = Stage::Duration(step.next());
        Ok(())
    }
}
fn materialized<T>(
    result: Result<NumericMaterialized<T>, NumericMaterializationFailure>,
    observed: &mut usize,
) -> Result<T, HtmlTimeElementInterpretationError> {
    match result {
        Ok(value) => {
            *observed = value.capacity_bytes;
            Ok(value.value)
        }
        Err(error) => {
            *observed = error.capacity_bytes;
            Err(HtmlTimeElementInterpretationError::Processing(error.kind))
        }
    }
}
/// Dropping the guard refuses admission without changing its retained owner.
pub struct HtmlTimeElementInterpretationAction<'step, 'raw> {
    owner: &'step mut HtmlTimeElementInterpretation<'raw>,
    layout: HtmlTimeElementInterpretationLayout,
}
impl HtmlTimeElementInterpretationAction<'_, '_> {
    pub fn layout(&self) -> HtmlTimeElementInterpretationLayout {
        self.layout
    }
    pub fn advance(
        self,
        max_integer_digits: usize,
        max_allocation_bytes: usize,
    ) -> HtmlTimeElementInterpretationStep {
        let mut capacity = 0;
        let outcome = self
            .owner
            .advance(max_integer_digits, max_allocation_bytes, &mut capacity);
        if let Err(error) = outcome {
            self.owner.failure = Some(error);
        }
        HtmlTimeElementInterpretationStep {
            capacity,
            outcome,
            poll_after: self.layout.can_allocate(),
        }
    }
}

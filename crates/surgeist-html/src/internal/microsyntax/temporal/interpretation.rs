//! Retained time-element recognition and admitted exact materialization.
use super::*;
use crate::internal::accounting::{HtmlChange, HtmlResource, HtmlWork, wide};
use crate::internal::microsyntax::numeric::{
    HtmlIntegerUnitStep, HtmlNumericMaterializationError, HtmlNumericMaterializationFailure,
    HtmlNumericMaterialized,
    decimal_total::{HtmlDecimalTotal, HtmlDecimalTotalError},
};
use crate::internal::values::finish::HtmlFinishFailure;
use crate::{
    internal::values::interpretation::HtmlValueAdmissionError,
    internal::values::interpretation::HtmlValueInterpreter,
    internal::values::interpretation::HtmlValueReport,
};
use surgeist_ledger::{Change, Charge};

/// A refused owner transition or latched processing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTimeElementInterpretationError {
    kind: HtmlTimeElementInterpretationErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HtmlTimeElementInterpretationErrorKind {
    NotReady,
    RepresentationExhausted,
    Processing(HtmlNumericMaterializationError),
    InvalidRecognizedPlan,
}

impl HtmlTimeElementInterpretationErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlTimeElementInterpretationError {
        HtmlTimeElementInterpretationError { kind: self }
    }
}

impl From<HtmlTimeElementInterpretationErrorKind> for HtmlTimeElementInterpretationError {
    fn from(kind: HtmlTimeElementInterpretationErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlTimeElementInterpretationError {
    /// Whether the cause is: time-element interpretation has an unfinished step.
    pub fn is_not_ready(&self) -> bool {
        matches!(self.kind, HtmlTimeElementInterpretationErrorKind::NotReady)
    }
    /// Whether the cause is: time-element interpretation size cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(
            self.kind,
            HtmlTimeElementInterpretationErrorKind::RepresentationExhausted
        )
    }
    /// Whether the cause is: recognized time-element plan does not match its value.
    pub fn is_invalid_recognized_plan(&self) -> bool {
        matches!(
            self.kind,
            HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan
        )
    }
    /// The value-processing failure that stopped the interpretation, if that was the cause.
    pub fn processing_failure(&self) -> Option<HtmlNumericMaterializationError> {
        match self.kind {
            HtmlTimeElementInterpretationErrorKind::Processing(error) => Some(error),
            _ => None,
        }
    }
}

impl std::fmt::Display for HtmlTimeElementInterpretationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            HtmlTimeElementInterpretationErrorKind::NotReady => {
                f.write_str("time-element interpretation has an unfinished step")
            }
            HtmlTimeElementInterpretationErrorKind::RepresentationExhausted => f.write_str(
                "time-element interpretation size cannot be represented on this platform",
            ),
            HtmlTimeElementInterpretationErrorKind::Processing(_) => {
                f.write_str("time-element value processing failed")
            }
            HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan => {
                f.write_str("recognized time-element plan does not match its value")
            }
        }
    }
}

impl std::error::Error for HtmlTimeElementInterpretationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            HtmlTimeElementInterpretationErrorKind::Processing(error) => Some(error),
            _ => None,
        }
    }
}

/// The next operation's role; no borrowed temporal plan crosses this boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTimeElementInterpretationActionKind {
    /// Recognizing which of the time element's alternatives the raw value matches, or, for a
    /// duration, scanning its next component. Each attempt is charged `16 * length + 256`, where
    /// `length` is the raw length for the value match and, for a duration component, the units
    /// that the step consumes: the T markers and whitespace before the component, the
    /// component, and the whitespace after it (or the trailing whitespace of the last step).
    /// It allocates nothing.
    Recognition,
    /// Building exact numeric digits from borrowed digits or by arithmetic. It is the only kind
    /// that allocates, and the caller supplies the limits for it.
    Materialization,
    /// Combining already admitted numeric values into a temporal value. It allocates nothing; a
    /// zero-credit assembly completes a numeric operation whose work was already admitted.
    Assembly,
    /// A boundary with no work credit between the end of value processing and report assembly, at
    /// which the caller can poll before continuing.
    Poll,
    /// Assembling the published report from the authored value and the processed value.
    ReportAssembly,
}
/// The role of the next operation and whether it can allocate. Its work is charged by
/// [`HtmlTimeElementInterpretationAction::advance`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlTimeElementInterpretationLayout {
    work: usize,
    kind: HtmlTimeElementInterpretationActionKind,
}
impl HtmlTimeElementInterpretationLayout {
    /// The schema change that admits this action's work, or `None` for a zero-credit Assembly:
    /// it completes a numeric operation whose work was already admitted, so it adds no charge.
    fn change(self) -> Option<HtmlChange> {
        (self.work != 0 || self.kind != HtmlTimeElementInterpretationActionKind::Assembly)
            .then(|| Change::Work(HtmlWork::SemanticStep, wide(self.work)))
    }
    /// The role of the next operation.
    pub fn kind(self) -> HtmlTimeElementInterpretationActionKind {
        self.kind
    }
    /// Whether the operation can allocate, which only a `Materialization` can. Admit its allocation
    /// limits before executing it.
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
    /// The result of the action: `Ok` when it completed, or the failure it ended with. The failure
    /// also stays with the interpretation, which refuses further actions.
    pub fn outcome(self) -> Result<(), HtmlTimeElementInterpretationError> {
        self.outcome
    }
    /// Poll after a successful outcome and capacity observation, before any
    /// following assembly. Failed outcomes retain their original failure.
    pub fn poll_after(self) -> bool {
        self.poll_after
    }
}
type Plan<'raw> = BorrowedTemporalPlan<'raw>;
/// Where the interpretation is, with the data that only that stage holds: the recognized plan,
/// the assembled year, date, day or local value, and the duration cursor and component. The
/// shared numeric scratch registers are the one thing a stage does not own.
#[derive(Debug)]
enum Stage<'raw> {
    Recognize,
    Begin(Plan<'raw>),
    YearRead(Plan<'raw>),
    YearAssemble(Plan<'raw>),
    /// A year was assembled for a plan that also has a date.
    DateAssemble {
        plan: Plan<'raw>,
        year: HtmlYear,
    },
    /// Recovery of a time: its seconds are read next.
    TimeSecond(BorrowedTime<'raw>),
    TimeSecondAssemble(BorrowedTime<'raw>),
    /// Recovery of a local date and time, whose date is assembled.
    LocalSecond {
        plan: BorrowedLocalDateTime<'raw>,
        date: HtmlDate,
    },
    LocalSecondAssemble {
        plan: BorrowedLocalDateTime<'raw>,
        date: HtmlDate,
    },
    Global(GlobalStage<'raw>),
    Duration(DurationStage<'raw>),
    Poll,
    Report,
    /// The published report, which only `finish` takes.
    Done(Report),
}
type Global<'raw> = BorrowedGlobalDateTime<'raw>;
/// A global date and time whose date is assembled.
#[derive(Debug)]
struct Dated<'raw> {
    plan: Global<'raw>,
    date: HtmlDate,
}
/// A global date and time with its date and its UTC day.
#[derive(Debug)]
struct Dayed<'raw> {
    plan: Global<'raw>,
    date: HtmlDate,
    day: HtmlInteger,
}
/// A global date and time with its assembled local value and UTC day.
#[derive(Debug)]
struct Localized<'raw> {
    plan: Global<'raw>,
    local: HtmlParsedLocalDateTime,
    day: HtmlInteger,
}
#[derive(Debug)]
enum GlobalStage<'raw> {
    Year(Global<'raw>),
    CopyYear(Global<'raw>),
    Date(Global<'raw>),
    PrecedingYear(Dated<'raw>),
    Cycles(Dated<'raw>),
    CycleDays(Dated<'raw>),
    WithinCycle(Dated<'raw>),
    WithinCycleNumber(Dated<'raw>),
    Ordinal(Dated<'raw>),
    Day(Dated<'raw>),
    Adjustment(Dayed<'raw>),
    AdjustDay(Dayed<'raw>),
    Seconds(Dayed<'raw>),
    CopySeconds(Dayed<'raw>),
    Local(Dayed<'raw>),
    /// `Finish` publishes the report and never resumes at a successor.
    Finish(Localized<'raw>),
}
/// A duration component whose quantity is being scaled and accumulated.
#[derive(Debug)]
struct Running<'raw> {
    plan: BorrowedDurationPlan<'raw>,
    cursor: BorrowedDurationCursor<'raw>,
    component: BorrowedDurationComponent<'raw>,
}
#[derive(Debug)]
enum DurationStage<'raw> {
    /// The authored total starts at an allocated zero; the recovery total starts empty.
    Zero(BorrowedDurationPlan<'raw>),
    Initialize(BorrowedDurationPlan<'raw>),
    Next {
        plan: BorrowedDurationPlan<'raw>,
        cursor: BorrowedDurationCursor<'raw>,
    },
    Quantity(Running<'raw>),
    Scale(Running<'raw>),
    Accumulate(Running<'raw>),
    Advance(Running<'raw>),
    /// The recovery total is read out once every component is added.
    Finish(BorrowedDurationPlan<'raw>),
}
type Report = HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>;

/// One exact borrowed operand and its unpublished recognition/materialization.
/// Construction admits only the raw length; no grammar or allocation runs.
///
/// When no alternative matches, each phase retains the error of the last
/// alternative tried (duration), exactly as
/// [`HtmlValueInterpreter::time_element_value`] reports: §4.5.14 (elements
/// capture lines 6855-6877) defines no failure order across alternatives.
#[derive(Debug)]
pub struct HtmlTimeElementInterpretation<'raw> {
    raw: &'raw DocumentString,
    interpreter: HtmlValueInterpreter,
    phase: TemporalPhase,
    alternative: usize,
    stage: Stage<'raw>,
    author: Result<HtmlTimeElementValue, HtmlTemporalError>,
    recovery: Result<HtmlMachineReadableTime, HtmlTemporalError>,
    // Scratch slots that the stages share: which slot a stage reads or fills depends on the
    // stage and on the author or recovery phase, in the order that `layout` charges, so they
    // belong to no single stage.
    integers: [Option<HtmlNonNegativeInteger>; 5],
    decimals: [Option<HtmlNonNegativeDecimal>; 3],
    /// The recovery duration's running total, which no scratch slot holds: adding a component
    /// visits that component's digits, not the total's.
    total: HtmlDecimalTotal,
    millisecond: u16,
    within_cycle: u32,
    adjustment: i8,
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
            author: Err(HtmlTemporalError::InvalidSyntax),
            recovery: Err(HtmlTemporalError::InvalidSyntax),
            integers: std::array::from_fn(|_| None),
            decimals: std::array::from_fn(|_| None),
            total: HtmlDecimalTotal::default(),
            millisecond: 0,
            within_cycle: 0,
            adjustment: 0,
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
        if matches!(self.stage, Stage::Done(_)) {
            return Ok(None);
        }
        let layout = match self.layout() {
            Ok(layout) => layout,
            Err(error) => {
                self.failure = Some(error);
                return Err(error);
            }
        };
        Ok(Some(HtmlTimeElementInterpretationAction {
            owner: self,
            layout,
        }))
    }
    /// Consumes the interpretation and publishes its completed report. A latched failure or an
    /// unfinished step refuses the finish and hands the interpretation back unchanged.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn finish(
        self,
    ) -> Result<
        HtmlValueReport<HtmlTimeElementValue, HtmlMachineReadableTime>,
        HtmlFinishFailure<Self, HtmlTimeElementInterpretationError>,
    > {
        if let Some(error) = self.failure {
            return Err(HtmlFinishFailure::new(self, error));
        }
        match self {
            Self {
                stage: Stage::Done(report),
                ..
            } => Ok(report),
            other => Err(HtmlFinishFailure::new(
                other,
                HtmlTimeElementInterpretationErrorKind::NotReady,
            )),
        }
    }
    // The registers are filled by the stage that precedes each reader, in the order that
    // `layout` charges; a register that a stage finds missing reports the invalid-recognized-plan
    // cause instead of asserting.
    fn integer(
        &self,
        index: usize,
    ) -> Result<&HtmlNonNegativeInteger, HtmlTimeElementInterpretationError> {
        self.integers[index]
            .as_ref()
            .ok_or(HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan.error())
    }
    fn decimal(
        &self,
        index: usize,
    ) -> Result<&HtmlNonNegativeDecimal, HtmlTimeElementInterpretationError> {
        self.decimals[index]
            .as_ref()
            .ok_or(HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan.error())
    }
    fn layout(
        &self,
    ) -> Result<HtmlTimeElementInterpretationLayout, HtmlTimeElementInterpretationError> {
        use HtmlTimeElementInterpretationActionKind as K;
        let (kind, work) = match &self.stage {
            Stage::TimeSecondAssemble(_) | Stage::LocalSecondAssemble { .. } => (K::Assembly, 0),
            Stage::Recognize => (
                K::Recognition,
                exhausted(temporal_recognition_work_bound(self.raw.len()))?,
            ),
            Stage::YearRead(plan) => (
                K::Materialization,
                exhausted(HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                    year_of(*plan)?.digits().len(),
                ))?,
            ),
            Stage::TimeSecond(time) => (K::Materialization, seconds_bound(*time)?),
            Stage::LocalSecond { plan, .. } => (K::Materialization, seconds_bound(plan.time())?),
            Stage::Global(step) => match step {
                GlobalStage::Year(plan) => (
                    K::Materialization,
                    exhausted(HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                        plan.local().date().year().digits().len(),
                    ))?,
                ),
                GlobalStage::CopyYear(_) => (
                    K::Materialization,
                    exhausted(self.integer(0)?.copy_work_bound())?,
                ),
                GlobalStage::PrecedingYear(_) => (
                    K::Materialization,
                    exhausted(self.integer(0)?.decremented_work_bound())?,
                ),
                GlobalStage::Cycles(_) => (
                    K::Materialization,
                    exhausted(self.integer(2)?.quotient_remainder_work_bound())?,
                ),
                GlobalStage::CycleDays(_) => (
                    K::Materialization,
                    exhausted(self.integer(3)?.multiplied_by_work_bound())?,
                ),
                GlobalStage::WithinCycle(_) => (K::Assembly, 256),
                GlobalStage::WithinCycleNumber(_) => (
                    K::Materialization,
                    exhausted(HtmlNonNegativeInteger::from_u32_work_bound())?,
                ),
                GlobalStage::Ordinal(_) => (
                    K::Materialization,
                    exhausted(self.integer(4)?.added_to_work_bound(self.integer(1)?))?,
                ),
                GlobalStage::AdjustDay(v) => (
                    K::Materialization,
                    exhausted(v.day.materialized_unit_step_work_bound())?,
                ),
                GlobalStage::Seconds(v) => {
                    (K::Materialization, seconds_bound(v.plan.local().time())?)
                }
                GlobalStage::CopySeconds(_) => (
                    K::Materialization,
                    exhausted(self.decimal(0)?.copy_work_bound())?,
                ),
                _ => (K::Assembly, 1),
            },
            Stage::Duration(step) => match step {
                DurationStage::Initialize(_) | DurationStage::Advance(_) => (K::Assembly, 0),
                DurationStage::Zero(_) => (
                    K::Materialization,
                    exhausted(HtmlNonNegativeInteger::from_u32_work_bound())?,
                ),
                DurationStage::Next { cursor, .. } => {
                    (K::Recognition, exhausted(cursor.next_work_bound())?)
                }
                DurationStage::Quantity(r) => {
                    let p = r.component;
                    (
                        K::Materialization,
                        exhausted(if self.phase == TemporalPhase::Author {
                            HtmlNonNegativeInteger::from_borrowed_digits_work_bound(
                                p.integer_digits().len(),
                            )
                        } else {
                            HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
                                p.integer_digits().len(),
                                p.fraction_digits().len(),
                            )
                        })?,
                    )
                }
                DurationStage::Scale(_) => (
                    K::Materialization,
                    exhausted(if self.phase == TemporalPhase::Author {
                        self.integer(1)?.multiplied_by_work_bound()
                    } else {
                        self.decimal(1)?.multiplied_by_work_bound()
                    })?,
                ),
                DurationStage::Accumulate(_) => (
                    K::Materialization,
                    exhausted(if self.phase == TemporalPhase::Author {
                        self.integer(0)?.added_to_work_bound(self.integer(2)?)
                    } else if self.total.carry_pending() {
                        self.total.carried_work_bound()
                    } else {
                        self.total.added_work_bound(self.decimal(2)?)
                    })?,
                ),
                DurationStage::Finish(_) => (
                    K::Materialization,
                    exhausted(self.total.finished_work_bound())?,
                ),
            },
            Stage::Poll => (K::Poll, 0),
            Stage::Report => (K::ReportAssembly, 1),
            // A finished interpretation issues no action.
            Stage::Done(_) => return Err(invalid()),
            Stage::Begin(_) | Stage::YearAssemble(_) | Stage::DateAssemble { .. } => {
                (K::Assembly, 1)
            }
        };
        Ok(HtmlTimeElementInterpretationLayout { work, kind })
    }
    fn complete_phase(&mut self) {
        self.integers = std::array::from_fn(|_| None);
        self.decimals = std::array::from_fn(|_| None);
        self.total = HtmlDecimalTotal::default();
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
        // The stage is taken so that its payload moves into the next stage. A failure latches in
        // the caller, after which no stage is read again, so the placeholder left by an early
        // error is never observed.
        match std::mem::replace(&mut self.stage, Stage::Poll) {
            Stage::Recognize => {
                self.stage = Stage::Recognize;
                match recognize_time_alternative(
                    self.raw.code_units(),
                    self.phase,
                    TemporalAlternative::ALL[self.alternative],
                ) {
                    Ok(plan) => self.stage = Stage::Begin(plan),
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
                }
            }
            Stage::Begin(plan) => match plan {
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
                        self.stage = Stage::TimeSecond(value);
                    }
                }
                BorrowedTemporalPlan::GlobalDateTime(global)
                    if self.phase == TemporalPhase::Recovery =>
                {
                    self.stage = Stage::Global(GlobalStage::Year(global))
                }
                BorrowedTemporalPlan::Duration(plan) => {
                    // The recovery total starts empty, which needs no allocation.
                    self.stage = Stage::Duration(if self.phase == TemporalPhase::Author {
                        DurationStage::Zero(plan)
                    } else {
                        DurationStage::Initialize(plan)
                    });
                }
                plan => self.stage = Stage::YearRead(plan),
            },
            Stage::YearRead(plan) => {
                self.integers[0] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_borrowed_digits(
                        year_of(plan)?.digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?);
                self.stage = Stage::YearAssemble(plan);
            }
            Stage::YearAssemble(plan) => {
                let year = year_of(plan)?
                    .assemble(self.integers[0].take().ok_or_else(invalid)?)
                    .map_err(|_| invalid())?;
                match plan {
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
                    plan => self.stage = Stage::DateAssemble { plan, year },
                }
            }
            Stage::DateAssemble { plan, year } => {
                let date = date_of(plan)?.assemble(year);
                match plan {
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
                            self.stage = Stage::LocalSecond { plan: p, date };
                        }
                    }
                    BorrowedTemporalPlan::GlobalDateTime(p) => self.author(
                        HtmlTimeElementValue::GlobalDateTime(p.assemble_author(date)),
                    ),
                    _ => return Err(invalid()),
                }
            }
            Stage::TimeSecond(time) => {
                self.decimals[0] = Some(read_seconds(time, digits, bytes, observed)?);
                self.stage = Stage::TimeSecondAssemble(time);
            }
            Stage::LocalSecond { plan, date } => {
                self.decimals[0] = Some(read_seconds(plan.time(), digits, bytes, observed)?);
                self.stage = Stage::LocalSecondAssemble { plan, date };
            }
            Stage::TimeSecondAssemble(time) => {
                let seconds = self.decimals[0].take().ok_or_else(invalid)?;
                self.recovery(HtmlMachineReadableTime::Time(
                    time.assemble_recovery(seconds),
                ));
            }
            Stage::LocalSecondAssemble { plan, date } => {
                let seconds = self.decimals[0].take().ok_or_else(invalid)?;
                self.recovery(HtmlMachineReadableTime::LocalDateTime(
                    plan.assemble_recovery(date, seconds),
                ));
            }
            Stage::Global(step) => self.advance_global(step, digits, bytes, observed)?,
            Stage::Duration(step) => self.advance_duration(step, digits, bytes, observed)?,
            Stage::Poll => self.stage = Stage::Report,
            Stage::Report => {
                let author =
                    std::mem::replace(&mut self.author, Err(HtmlTemporalError::InvalidSyntax));
                let recovery =
                    std::mem::replace(&mut self.recovery, Err(HtmlTemporalError::InvalidSyntax));
                // The same interpreter admitted this raw value at construction.
                let report = self
                    .interpreter
                    .time_element_report_from_attempts(self.raw, author, recovery)
                    .map_err(|_| invalid())?;
                self.stage = Stage::Done(report);
            }
            Stage::Done(report) => {
                self.stage = Stage::Done(report);
                return Err(invalid());
            }
        }
        Ok(())
    }
    fn advance_global(
        &mut self,
        step: GlobalStage<'raw>,
        digits: usize,
        bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlTimeElementInterpretationError> {
        let next = match step {
            GlobalStage::Year(plan) => {
                self.integers[0] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_borrowed_digits(
                        plan.local().date().year().digits(),
                        digits,
                        bytes,
                    ),
                    observed,
                )?);
                GlobalStage::CopyYear(plan)
            }
            GlobalStage::CopyYear(plan) => {
                self.integers[1] = Some(materialized(
                    self.integer(0)?.try_copy(digits, bytes),
                    observed,
                )?);
                GlobalStage::Date(plan)
            }
            GlobalStage::Date(plan) => {
                let local_date = plan.local().date();
                let date = local_date.assemble(
                    local_date
                        .year()
                        .assemble(self.integers[1].take().ok_or_else(invalid)?)
                        .map_err(|_| invalid())?,
                );
                GlobalStage::PrecedingYear(Dated { plan, date })
            }
            GlobalStage::PrecedingYear(v) => {
                self.integers[2] = Some(
                    materialized(self.integer(0)?.try_decremented(digits, bytes), observed)?
                        .ok_or_else(invalid)?,
                );
                GlobalStage::Cycles(v)
            }
            GlobalStage::Cycles(v) => {
                self.integers[3] = Some(
                    materialized(
                        self.integer(2)?
                            .try_quotient_remainder(GREGORIAN_CYCLE, digits, bytes),
                        observed,
                    )?
                    .0,
                );
                GlobalStage::CycleDays(v)
            }
            GlobalStage::CycleDays(v) => {
                self.integers[4] = Some(materialized(
                    self.integer(3)?.try_multiplied_by(146097, digits, bytes),
                    observed,
                )?);
                GlobalStage::WithinCycle(v)
            }
            GlobalStage::WithinCycle(v) => {
                self.within_cycle = v.plan.local().date().within_cycle_days();
                GlobalStage::WithinCycleNumber(v)
            }
            GlobalStage::WithinCycleNumber(v) => {
                self.integers[1] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_u32(self.within_cycle, digits, bytes),
                    observed,
                )?);
                GlobalStage::Ordinal(v)
            }
            GlobalStage::Ordinal(v) => {
                self.integers[2] = Some(materialized(
                    self.integer(4)?
                        .try_added_to(self.integer(1)?, digits, bytes),
                    observed,
                )?);
                GlobalStage::Day(v)
            }
            GlobalStage::Day(v) => {
                let day = HtmlInteger::from_magnitude(
                    self.integers[2].take().ok_or_else(invalid)?,
                    false,
                );
                GlobalStage::Adjustment(Dayed {
                    plan: v.plan,
                    date: v.date,
                    day,
                })
            }
            GlobalStage::Adjustment(v) => {
                self.adjustment = v.plan.utc_day_adjustment();
                if self.adjustment == 0 {
                    GlobalStage::Seconds(v)
                } else {
                    GlobalStage::AdjustDay(v)
                }
            }
            GlobalStage::AdjustDay(mut v) => {
                let direction = if self.adjustment < 0 {
                    HtmlIntegerUnitStep::Decrement
                } else {
                    HtmlIntegerUnitStep::Increment
                };
                v.day = materialized(
                    v.day.try_materialized_unit_step(direction, digits, bytes),
                    observed,
                )?;
                GlobalStage::Seconds(v)
            }
            GlobalStage::Seconds(v) => {
                self.decimals[0] = Some(read_seconds(
                    v.plan.local().time(),
                    digits,
                    bytes,
                    observed,
                )?);
                GlobalStage::CopySeconds(v)
            }
            GlobalStage::CopySeconds(v) => {
                self.decimals[1] = Some(materialized(
                    self.decimal(0)?.try_copy(digits, bytes),
                    observed,
                )?);
                GlobalStage::Local(v)
            }
            GlobalStage::Local(v) => {
                let local = v
                    .plan
                    .local()
                    .assemble_recovery(v.date, self.decimals[0].take().ok_or_else(invalid)?);
                GlobalStage::Finish(Localized {
                    plan: v.plan,
                    local,
                    day: v.day,
                })
            }
            GlobalStage::Finish(v) => {
                let value = v.plan.assemble_recovery(
                    v.local,
                    v.day,
                    self.decimals[1].take().ok_or_else(invalid)?,
                );
                self.recovery(HtmlMachineReadableTime::GlobalDateTime(value));
                return Ok(());
            }
        };
        self.stage = Stage::Global(next);
        Ok(())
    }
    fn advance_duration(
        &mut self,
        step: DurationStage<'raw>,
        digits: usize,
        bytes: usize,
        observed: &mut usize,
    ) -> Result<(), HtmlTimeElementInterpretationError> {
        let next = match step {
            DurationStage::Zero(plan) => {
                self.integers[0] = Some(materialized(
                    HtmlNonNegativeInteger::try_from_u32(0, digits, bytes),
                    observed,
                )?);
                DurationStage::Initialize(plan)
            }
            DurationStage::Initialize(plan) => DurationStage::Next {
                plan,
                cursor: plan.cursor(),
            },
            DurationStage::Next { plan, mut cursor } => {
                let Some(component) = cursor
                    .next_component()
                    .map_err(|_| HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan)?
                else {
                    if self.phase == TemporalPhase::Author {
                        let value = plan.assemble_author(
                            self.integers[0].take().ok_or_else(invalid)?,
                            self.millisecond,
                        );
                        self.author(HtmlTimeElementValue::Duration(value));
                    } else {
                        self.stage = Stage::Duration(DurationStage::Finish(plan));
                    }
                    return Ok(());
                };
                if component.scale().is_none() {
                    DurationStage::Next { plan, cursor }
                } else {
                    DurationStage::Quantity(Running {
                        plan,
                        cursor,
                        component,
                    })
                }
            }
            DurationStage::Quantity(running) => {
                let p = running.component;
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
                DurationStage::Scale(running)
            }
            DurationStage::Scale(running) => {
                let scale = running.component.scale().ok_or_else(invalid)?;
                if self.phase == TemporalPhase::Author {
                    self.integers[2] = Some(materialized(
                        self.integer(1)?.try_multiplied_by(scale, digits, bytes),
                        observed,
                    )?);
                } else {
                    self.decimals[2] = Some(materialized(
                        self.decimal(1)?.try_multiplied_by(scale, digits, bytes),
                        observed,
                    )?);
                }
                DurationStage::Accumulate(running)
            }
            DurationStage::Accumulate(running) => {
                if self.phase == TemporalPhase::Author {
                    self.integers[0] = Some(materialized(
                        self.integer(0)?
                            .try_added_to(self.integer(2)?, digits, bytes),
                        observed,
                    )?);
                    DurationStage::Advance(running)
                } else {
                    // A carry too long for one step stays pending, and the next step walks on.
                    let result = if self.total.carry_pending() {
                        self.total.try_carried(digits, bytes)
                    } else {
                        let addend = self.decimals[2].as_ref().ok_or_else(invalid)?;
                        self.total.try_added_to(addend, digits, bytes)
                    };
                    total_step(result, observed)?;
                    if self.total.carry_pending() {
                        DurationStage::Accumulate(running)
                    } else {
                        DurationStage::Advance(running)
                    }
                }
            }
            DurationStage::Finish(plan) => {
                let seconds = total_step(self.total.try_finished(digits, bytes), observed)?;
                let value = plan.assemble_recovery(seconds);
                self.recovery(HtmlMachineReadableTime::Duration(value));
                return Ok(());
            }
            DurationStage::Advance(running) => {
                if self.phase == TemporalPhase::Author && running.component.scale() == Some(1) {
                    self.millisecond = running.component.millisecond();
                }
                self.integers[1] = None;
                self.integers[2] = None;
                self.decimals[1] = None;
                self.decimals[2] = None;
                DurationStage::Next {
                    plan: running.plan,
                    cursor: running.cursor,
                }
            }
        };
        self.stage = Stage::Duration(next);
        Ok(())
    }
}
/// The recognized plan is not the kind that the stage reads, or a register it reads is missing.
fn invalid() -> HtmlTimeElementInterpretationError {
    HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan.error()
}
/// A work bound that cannot be represented on this platform.
fn exhausted<T>(bound: Option<T>) -> Result<T, HtmlTimeElementInterpretationError> {
    bound.ok_or(HtmlTimeElementInterpretationErrorKind::RepresentationExhausted.error())
}
fn year_of<'raw>(
    plan: Plan<'raw>,
) -> Result<BorrowedYear<'raw>, HtmlTimeElementInterpretationError> {
    Ok(match plan {
        BorrowedTemporalPlan::Month(p) => p.year(),
        BorrowedTemporalPlan::Date(p) => p.year(),
        BorrowedTemporalPlan::LocalDateTime(p) => p.date().year(),
        BorrowedTemporalPlan::GlobalDateTime(p) => p.local().date().year(),
        BorrowedTemporalPlan::Week(p) => p.year(),
        BorrowedTemporalPlan::Year(p) => p,
        _ => return Err(invalid()),
    })
}
fn date_of<'raw>(
    plan: Plan<'raw>,
) -> Result<BorrowedDate<'raw>, HtmlTimeElementInterpretationError> {
    Ok(match plan {
        BorrowedTemporalPlan::Date(p) => p,
        BorrowedTemporalPlan::LocalDateTime(p) => p.date(),
        BorrowedTemporalPlan::GlobalDateTime(p) => p.local().date(),
        _ => return Err(invalid()),
    })
}
/// The work bound of materializing a time's seconds.
fn seconds_bound(time: BorrowedTime<'_>) -> Result<usize, HtmlTimeElementInterpretationError> {
    exhausted(HtmlNonNegativeDecimal::from_borrowed_digits_work_bound(
        time.second_digits().len(),
        time.fraction_digits().len(),
    ))
}
fn read_seconds(
    time: BorrowedTime<'_>,
    digits: usize,
    bytes: usize,
    observed: &mut usize,
) -> Result<HtmlNonNegativeDecimal, HtmlTimeElementInterpretationError> {
    materialized(
        HtmlNonNegativeDecimal::try_from_borrowed_digits(
            time.second_digits(),
            time.fraction_digits(),
            digits,
            bytes,
        ),
        observed,
    )
}
/// A step of the recovery total. The stage order never runs a step the total refuses, so a
/// refusal reports the invalid-recognized-plan cause.
fn total_step<T>(
    result: Result<HtmlNumericMaterialized<T>, HtmlDecimalTotalError>,
    observed: &mut usize,
) -> Result<T, HtmlTimeElementInterpretationError> {
    match result {
        Ok(value) => materialized(Ok(value), observed),
        Err(HtmlDecimalTotalError::Materialization(failure)) => {
            materialized(Err(failure), observed)
        }
        Err(HtmlDecimalTotalError::CarryPending | HtmlDecimalTotalError::NoCarryPending) => {
            Err(invalid())
        }
    }
}
fn materialized<T>(
    result: Result<HtmlNumericMaterialized<T>, HtmlNumericMaterializationFailure>,
    observed: &mut usize,
) -> Result<T, HtmlTimeElementInterpretationError> {
    match result {
        Ok(value) => {
            *observed = value.capacity_bytes;
            Ok(value.value)
        }
        Err(error) => {
            *observed = error.capacity_bytes;
            Err(HtmlTimeElementInterpretationErrorKind::Processing(error.kind).error())
        }
    }
}
/// Dropping the guard refuses admission without changing its retained owner.
#[derive(Debug)]
pub struct HtmlTimeElementInterpretationAction<'step, 'raw> {
    owner: &'step mut HtmlTimeElementInterpretation<'raw>,
    layout: HtmlTimeElementInterpretationLayout,
}
impl HtmlTimeElementInterpretationAction<'_, '_> {
    /// The kind of the action and whether it can allocate, known before it runs.
    pub fn layout(&self) -> HtmlTimeElementInterpretationLayout {
        self.layout
    }
    /// Charges the action's work as one change of semantic steps, then executes it. A zero-credit
    /// Assembly charges nothing. A refused charge leaves the interpretation at the same action.
    /// `max_integer_digits` and `max_allocation_bytes` bound any numeric allocation the action
    /// makes. The step reports the capacity actually observed, even on failure, the outcome, and
    /// whether to poll afterward.
    pub fn advance<C: Charge<HtmlWork, HtmlResource>>(
        self,
        charge: &mut C,
        max_integer_digits: usize,
        max_allocation_bytes: usize,
    ) -> Result<HtmlTimeElementInterpretationStep, C::Stop> {
        if let Some(change) = self.layout.change() {
            charge.charge(&[change])?;
        }
        let mut capacity = 0;
        let outcome = self
            .owner
            .advance(max_integer_digits, max_allocation_bytes, &mut capacity);
        if let Err(error) = outcome {
            self.owner.failure = Some(error);
        }
        Ok(HtmlTimeElementInterpretationStep {
            capacity,
            outcome,
            poll_after: self.layout.can_allocate(),
        })
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlTimeElementInterpretationError) -> bool;
        let cases: Vec<(HtmlTimeElementInterpretationError, &str, Predicate)> = vec![
            (
                HtmlTimeElementInterpretationErrorKind::NotReady.error(),
                "time-element interpretation has an unfinished step",
                HtmlTimeElementInterpretationError::is_not_ready,
            ),
            (
                HtmlTimeElementInterpretationErrorKind::RepresentationExhausted.error(),
                "time-element interpretation size cannot be represented on this platform",
                HtmlTimeElementInterpretationError::is_representation_exhausted,
            ),
            (
                HtmlTimeElementInterpretationErrorKind::Processing(
                    HtmlNumericMaterializationError::Digits(
                        crate::internal::microsyntax::numeric::HtmlDecimalDigitsError::Empty,
                    ),
                )
                .error(),
                "time-element value processing failed",
                |e| e.processing_failure().is_some(),
            ),
            (
                HtmlTimeElementInterpretationErrorKind::InvalidRecognizedPlan.error(),
                "recognized time-element plan does not match its value",
                HtmlTimeElementInterpretationError::is_invalid_recognized_plan,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}

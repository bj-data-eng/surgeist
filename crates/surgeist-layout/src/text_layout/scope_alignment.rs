//! Actual captured-scope facts and native selected allocation.

use super::TextParagraphBinding;
use crate::{
    BaselineOffsetOf, BaselineOffsetResolutionError, DefaultScalar, LayoutScalar,
    LengthPercentageOf, NonNegativeFiniteOf, PercentageBasisOf, VerticalAlignOf,
};
use std::{mem::size_of, sync::Arc};
use surgeist_text::{
    CapturedScope, PreparedText, ScopeAlignmentBoundary, ScopeAlignmentError,
    ScopeAlignmentLimitKind, ScopeAlignmentLimits,
};

/// One actual scope's resolved alignment, supplied before Layout computation.
///
/// This does not authenticate an external computed Style or create a box. Its
/// captured identity and early formatting fact are checked against the real flow.
#[derive(Clone, Debug, PartialEq)]
pub struct TextScopeAlignmentInputOf<S: LayoutScalar = DefaultScalar> {
    pub scope: CapturedScope,
    pub alignment: VerticalAlignOf<S>,
}

pub type TextScopeAlignmentInput = TextScopeAlignmentInputOf<DefaultScalar>;

/// Called only after the actual normalized scope and nominated own box have
/// been proved. Equality checks consistency; it never establishes ownership.
pub(super) fn intrinsic_alignment<S: LayoutScalar>(
    row: &TextScopeAlignmentInputOf<S>,
    alignment: surgeist_text::LineBoxAlignment,
) -> surgeist_text::Result<surgeist_text::LineBoxAlignment> {
    use surgeist_text::LineBoxAlignment as A;
    let agrees = match (alignment, row.alignment) {
        (A::Baseline, _) => true,
        (A::Top, VerticalAlignOf::Top) | (A::Bottom, VerticalAlignOf::Bottom) => true,
        (A::Shift(actual), VerticalAlignOf::Shift(expected)) => {
            actual.get() == expected.get().to_f64()
        }
        _ => false,
    };
    if !agrees {
        return Err(refusal(ScopeAlignmentError::BoundaryMismatch {
            scope: row.scope.capture_ordinal(),
        }));
    }
    Ok(A::Baseline)
}

pub(crate) fn nominated_endpoint<'a, Node: Copy + Eq, S: LayoutScalar>(
    resources: &'a ScopeAlignmentResources<S>,
    associations: &super::capture::ScopeAssociations<Node>,
    node: Node,
    occurrence: &surgeist_text::PreparedControl,
) -> surgeist_text::Result<Option<&'a TextScopeAlignmentInputOf<S>>> {
    let Some(control) = occurrence.captured_control() else {
        return Ok(None);
    };
    let Some(limits) = control.scope().scope_alignment_limits() else {
        return Ok(None);
    };
    for row in resources.snapshot().rows() {
        ScopeAlignmentFacts::visit(resources, limits, 1)?;
        ScopeAlignmentFacts::visit(
            resources,
            limits,
            associations.captured_binding_work(&row.scope),
        )?;
        let Some(super::TextScopeAssociation::Inline { start, end }) =
            associations.captured_binding(&row.scope)
        else {
            continue;
        };
        if node == start && row.scope.entry_control().as_ref() == Some(&control)
            || node == end && row.scope.exit_control().as_ref() == Some(&control)
        {
            return Ok(Some(row));
        }
    }
    Ok(None)
}

impl<S: LayoutScalar> TextScopeAlignmentInputOf<S> {
    /// Normalize against this actual scope's own retained used line height.
    pub fn from_length_percentage(
        prepared: &PreparedText,
        scope: CapturedScope,
        amount: LengthPercentageOf<S>,
    ) -> surgeist_text::Result<Self> {
        require_prepared_scope(prepared, &scope)?;
        if amount.percent_fraction() == S::ZERO {
            let offset =
                BaselineOffsetOf::from_length_percentage(amount, PercentageBasisOf::Missing)
                    .map_err(|error| {
                        normalization_error(
                            error,
                            scope.capture_ordinal(),
                            "scope absolute baseline offset",
                            amount.absolute_px().to_f64(),
                        )
                    })?;
            return Ok(Self {
                scope,
                alignment: VerticalAlignOf::Shift(offset),
            });
        }
        let own = prepared.scope_metrics(&scope)?;
        let height = scalar_context::<S>(own.line_height())?;
        let height = NonNegativeFiniteOf::new(height).map_err(|_| {
            numeric(
                own.line_height(),
                surgeist_text::TextNumericError::OutOfRange,
            )
        })?;
        let offset =
            BaselineOffsetOf::from_length_percentage(amount, PercentageBasisOf::Definite(height))
                .map_err(|error| {
                normalization_error(
                    error,
                    scope.capture_ordinal(),
                    "scope own line height",
                    own.line_height(),
                )
            })?;
        Ok(Self {
            scope,
            alignment: VerticalAlignOf::Shift(offset),
        })
    }

    /// Native subscript policy using the actual captured parent's used size.
    pub fn subscript(prepared: &PreparedText, scope: CapturedScope) -> surgeist_text::Result<Self> {
        Self::script(prepared, scope, false)
    }

    /// Native superscript policy using the actual captured parent's used size.
    pub fn superscript(
        prepared: &PreparedText,
        scope: CapturedScope,
    ) -> surgeist_text::Result<Self> {
        Self::script(prepared, scope, true)
    }

    fn script(
        prepared: &PreparedText,
        scope: CapturedScope,
        superscript: bool,
    ) -> surgeist_text::Result<Self> {
        require_prepared_scope(prepared, &scope)?;
        let parent = scope
            .parent()
            .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
        let actual = prepared.scope_metrics(&parent)?;
        let size = scalar_context::<S>(f64::from(actual.font_size()))?;
        let size = NonNegativeFiniteOf::new(size).map_err(|_| {
            numeric(
                f64::from(actual.font_size()),
                surgeist_text::TextNumericError::OutOfRange,
            )
        })?;
        let offset = if superscript {
            BaselineOffsetOf::superscript(size)
        } else {
            BaselineOffsetOf::subscript(size)
        }
        .map_err(|error| {
            normalization_error(
                error,
                scope.capture_ordinal(),
                "scope parent used font size",
                f64::from(actual.font_size()),
            )
        })?;
        Ok(Self {
            scope,
            alignment: VerticalAlignOf::Shift(offset),
        })
    }
}

fn require_prepared_scope(
    prepared: &PreparedText,
    scope: &CapturedScope,
) -> surgeist_text::Result<()> {
    let captured = prepared
        .captured_paragraph()
        .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
    if !captured
        .enter_position()
        .same_capture(&scope.enter_position())
    {
        return Err(refusal(ScopeAlignmentError::ForeignCapture));
    }
    if captured.domain() != scope.domain() || scope.role() != surgeist_text::ScopeRole::Inline {
        return Err(refusal(ScopeAlignmentError::WrongDomain));
    }
    if !captured
        .scope_intersections()
        .any(|intersection| intersection.scope() == *scope)
    {
        return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
            scope: scope.capture_ordinal(),
        }));
    }
    Ok(())
}

fn scalar_context<S: LayoutScalar>(value: f64) -> surgeist_text::Result<S> {
    if !value.is_finite() {
        return Err(numeric(value, surgeist_text::TextNumericError::NonFinite));
    }
    let result = S::from_f64(value);
    if !result.is_finite() {
        return Err(numeric(value, surgeist_text::TextNumericError::OutOfRange));
    }
    if value != 0.0 && result == S::ZERO {
        return Err(numeric(value, surgeist_text::TextNumericError::NonZeroLost));
    }
    Ok(result)
}

fn normalization_error(
    error: BaselineOffsetResolutionError,
    scope: usize,
    field: &'static str,
    input: f64,
) -> surgeist_text::Error {
    let category = match error {
        BaselineOffsetResolutionError::MissingOwnLineHeight => {
            return refusal(ScopeAlignmentError::MissingMetrics { scope });
        }
        BaselineOffsetResolutionError::NonFinite => {
            if input.is_finite() {
                surgeist_text::TextNumericError::OutOfRange
            } else {
                surgeist_text::TextNumericError::NonFinite
            }
        }
        BaselineOffsetResolutionError::NonZeroLost => surgeist_text::TextNumericError::NonZeroLost,
    };
    // This enum reports the operation, not a failed numeric result. Retain its
    // genuine named input instead of fabricating a zero result.
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::InvalidTextNumeric,
        "scope baseline normalization refused",
    )
    .with_detail(surgeist_text::ErrorDetail::InvalidTextNumeric {
        field,
        phase: surgeist_text::TextNumericPhase::Selection,
        category,
        value: input,
    })
}

pub(super) fn refusal(reason: ScopeAlignmentError) -> surgeist_text::Error {
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::ScopeAlignmentRefused,
        "captured scope alignment refused",
    )
    .with_detail(surgeist_text::ErrorDetail::ScopeAlignment { reason })
}

fn numeric(value: f64, category: surgeist_text::TextNumericError) -> surgeist_text::Error {
    surgeist_text::Error::new(
        surgeist_text::ErrorCode::InvalidTextNumeric,
        "scope alignment numeric operation refused",
    )
    .with_detail(surgeist_text::ErrorDetail::InvalidTextNumeric {
        field: "scope alignment",
        phase: surgeist_text::TextNumericPhase::Selection,
        category,
        value,
    })
}

/// One immutable scalar-typed fact table. It retains no guessed font or position.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ScopeAlignmentSnapshot<S: LayoutScalar> {
    rows: Vec<TextScopeAlignmentInputOf<S>>,
    retention: ScopeCensus,
}

/// The fact table stays provisional while its actual native association proof
/// admits feature-owned descriptors and copies. Native graph/ExpectedFlow
/// storage remains with the existing source-validation owner.
#[derive(Debug)]
pub(crate) struct ScopeAlignmentAdmission<S: LayoutScalar> {
    rows: Vec<TextScopeAlignmentInputOf<S>>,
    work: AdmissionWork,
}

impl<S: LayoutScalar> ScopeAlignmentAdmission<S> {
    pub(super) fn capture_parts(
        &mut self,
    ) -> (&[TextScopeAlignmentInputOf<S>], Option<&mut AdmissionWork>) {
        let active = !self.rows.is_empty();
        (&self.rows, active.then_some(&mut self.work))
    }

    pub(super) fn work(&mut self) -> &mut AdmissionWork {
        &mut self.work
    }

    pub(crate) fn freeze(self) -> Arc<ScopeAlignmentSnapshot<S>> {
        Arc::new(ScopeAlignmentSnapshot {
            rows: self.rows,
            retention: self.work.retained,
        })
    }
}

impl<S: LayoutScalar> ScopeAlignmentSnapshot<S> {
    pub(crate) fn admit<Node>(
        rows: Vec<TextScopeAlignmentInputOf<S>>,
        paragraphs: &[TextParagraphBinding<Node>],
    ) -> surgeist_text::Result<ScopeAlignmentAdmission<S>> {
        let rows = if rows.is_empty() { Vec::new() } else { rows };
        let mut work = AdmissionWork::new(&rows)?;
        let retention = Self::admit_storage(&rows, paragraphs, &mut work)?;
        for (index, row) in rows.iter().enumerate() {
            work.visit()?;
            let ordinal = row.scope.capture_ordinal();
            for previous in &rows[..index] {
                work.visit()?;
                if previous.scope == row.scope {
                    return Err(refusal(ScopeAlignmentError::DuplicateAllocation {
                        scope: ordinal,
                    }));
                }
            }
            if row.scope.role() != surgeist_text::ScopeRole::Inline {
                return Err(refusal(ScopeAlignmentError::WrongDomain));
            }
            let class = row
                .scope
                .alignment_boundary()
                .ok_or_else(|| refusal(ScopeAlignmentError::MissingBoundary { scope: ordinal }))?;
            if !matches!(
                (class, row.alignment),
                (ScopeAlignmentBoundary::Initial, VerticalAlignOf::Baseline)
                    | (
                        ScopeAlignmentBoundary::NonInitial,
                        VerticalAlignOf::Top | VerticalAlignOf::Bottom | VerticalAlignOf::Shift(_)
                    )
            ) {
                return Err(refusal(ScopeAlignmentError::BoundaryMismatch {
                    scope: ordinal,
                }));
            }
            let mut same_capture = false;
            let mut same_domain = false;
            let mut represented = false;
            for paragraph in paragraphs {
                work.visit()?;
                let Some(captured) = paragraph.prepared.captured_paragraph() else {
                    continue;
                };
                if !captured
                    .enter_position()
                    .same_capture(&row.scope.enter_position())
                {
                    continue;
                }
                same_capture = true;
                if captured.domain() != row.scope.domain() {
                    continue;
                }
                same_domain = true;
                let mut intersects = false;
                for intersection in captured.scope_intersections() {
                    work.visit()?;
                    if intersection.scope() == row.scope {
                        intersects = true;
                        break;
                    }
                }
                if !intersects {
                    continue;
                }
                represented = true;
            }
            if !same_capture {
                return Err(refusal(
                    if paragraphs
                        .iter()
                        .any(|p| p.prepared.captured_paragraph().is_some())
                    {
                        ScopeAlignmentError::ForeignCapture
                    } else {
                        ScopeAlignmentError::MissingCapture
                    },
                ));
            }
            if !same_domain {
                return Err(refusal(ScopeAlignmentError::WrongDomain));
            }
            if !represented {
                return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
                    scope: ordinal,
                }));
            }
        }
        // Actual selected handles demand complete rows during allocation.
        // Dormant potential scopes do not force font or allocation demand.
        work.base(retention)?;
        Ok(ScopeAlignmentAdmission { rows, work })
    }

    fn admit_storage<Node>(
        rows: &Vec<TextScopeAlignmentInputOf<S>>,
        paragraphs: &[TextParagraphBinding<Node>],
        work: &mut AdmissionWork,
    ) -> surgeist_text::Result<ScopeCensus> {
        let active = !rows.is_empty()
            || paragraphs
                .iter()
                .any(|p| p.prepared.scope_alignment_retention().is_some());
        if !active {
            return Ok(ScopeCensus::default());
        }
        let own_bytes = rows
            .capacity()
            .checked_mul(size_of::<TextScopeAlignmentInputOf<S>>())
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        for (index, paragraph) in paragraphs.iter().enumerate() {
            let Some(retained) = paragraph.prepared.scope_alignment_retention() else {
                continue;
            };
            let captured = paragraph
                .prepared
                .captured_paragraph()
                .ok_or_else(|| refusal(ScopeAlignmentError::MissingCapture))?;
            let limits = captured
                .domain()
                .scope_alignment_limits()
                .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
            work.admit_input(
                limits,
                ScopeCensus::from_retention(retained),
                custody::input_headers::<S>(),
            )?;
            work.limit = work.limit.min(limits.structural_work());
            work.visit()?;
            let mut repeated = false;
            for old in &paragraphs[..index] {
                work.visit()?;
                if old.prepared == paragraph.prepared {
                    repeated = true;
                    break;
                }
            }
            if repeated {
                continue;
            }
            let records = retained
                .records()
                .checked_add(rows.len())
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
            check(ScopeAlignmentLimitKind::Records, records, limits.records())?;
            let bytes = retained
                .payload_bytes()
                .checked_add(own_bytes)
                .and_then(|bytes| bytes.checked_add(custody::input_headers::<S>()))
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
            check(
                ScopeAlignmentLimitKind::PayloadBytes,
                bytes,
                limits.payload_bytes(),
            )?;
        }
        Ok(ScopeCensus {
            records: rows.len(),
            bytes: own_bytes,
        })
    }

    pub(crate) fn rows(&self) -> &[TextScopeAlignmentInputOf<S>] {
        &self.rows
    }
}

/// Admission counts actual table, paragraph and intersection visits before
/// executing them. All scopes participating in this one table bound its work.
#[derive(Debug)]
pub(super) struct AdmissionWork {
    count: usize,
    limit: usize,
    records_limit: usize,
    bytes_limit: usize,
    materialized: usize,
    live: ScopeCensus,
    retained: ScopeCensus,
}
impl AdmissionWork {
    fn new<S: LayoutScalar>(rows: &[TextScopeAlignmentInputOf<S>]) -> surgeist_text::Result<Self> {
        let mut work = Self {
            count: 0,
            limit: usize::MAX,
            records_limit: usize::MAX,
            bytes_limit: usize::MAX,
            materialized: 0,
            live: ScopeCensus::default(),
            retained: ScopeCensus::default(),
        };
        for row in rows {
            if row.scope.role() != surgeist_text::ScopeRole::Inline {
                return Err(refusal(ScopeAlignmentError::WrongDomain));
            }
            if row.scope.alignment_boundary().is_none() {
                return Err(refusal(ScopeAlignmentError::MissingBoundary {
                    scope: row.scope.capture_ordinal(),
                }));
            }
            let limits = row
                .scope
                .scope_alignment_limits()
                .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
            work.limit = work.limit.min(limits.structural_work());
            work.records_limit = work.records_limit.min(limits.records());
            work.bytes_limit = work.bytes_limit.min(limits.payload_bytes());
            work.visit()?;
        }
        Ok(work)
    }
    fn visit(&mut self) -> surgeist_text::Result<()> {
        self.visits(1)
    }
    pub(super) fn visits(&mut self, count: usize) -> surgeist_text::Result<()> {
        self.count = self
            .count
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::StructuralWork))?;
        check(
            ScopeAlignmentLimitKind::StructuralWork,
            self.count,
            self.limit,
        )
    }

    fn admit_input(
        &mut self,
        limits: ScopeAlignmentLimits,
        retained: ScopeCensus,
        headers: usize,
    ) -> surgeist_text::Result<()> {
        check(
            ScopeAlignmentLimitKind::Records,
            retained.records,
            limits.records(),
        )?;
        let bytes = retained
            .bytes
            .checked_add(headers)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            bytes,
            limits.payload_bytes(),
        )?;
        self.records_limit = self.records_limit.min(limits.records() - retained.records);
        self.bytes_limit = self.bytes_limit.min(limits.payload_bytes() - bytes);
        Ok(())
    }

    fn base(&mut self, census: ScopeCensus) -> surgeist_text::Result<()> {
        self.check_live(census)?;
        self.live = census;
        self.retained = census;
        self.materialized = census.records;
        Ok(())
    }

    fn check_live(&self, census: ScopeCensus) -> surgeist_text::Result<()> {
        check(
            ScopeAlignmentLimitKind::Records,
            census.records,
            self.records_limit,
        )?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            census.bytes,
            self.bytes_limit,
        )
    }

    fn materialize(&mut self, count: usize) -> surgeist_text::Result<()> {
        let next = self
            .materialized
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        check(ScopeAlignmentLimitKind::Records, next, self.records_limit)?;
        self.materialized = next;
        Ok(())
    }

    fn census<T>(values: &Vec<T>) -> surgeist_text::Result<ScopeCensus> {
        Ok(ScopeCensus {
            records: values.len(),
            bytes: values
                .capacity()
                .checked_mul(size_of::<T>())
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
        })
    }

    fn replace_census(
        &mut self,
        old: ScopeCensus,
        next: ScopeCensus,
        retained: bool,
    ) -> surgeist_text::Result<()> {
        let removed = ScopeCensus {
            records: self
                .live
                .records
                .checked_sub(old.records)
                .expect("admitted scope rows"),
            bytes: self
                .live
                .bytes
                .checked_sub(old.bytes)
                .expect("admitted scope capacity"),
        };
        let live = removed.add(next)?;
        self.check_live(live)?;
        if retained {
            self.retained = ScopeCensus {
                records: self
                    .retained
                    .records
                    .checked_sub(old.records)
                    .expect("retained scope rows"),
                bytes: self
                    .retained
                    .bytes
                    .checked_sub(old.bytes)
                    .expect("retained scope capacity"),
            }
            .add(next)?;
        }
        self.live = live;
        Ok(())
    }

    /// Pin an existing canonical vector when the new feature first needs its
    /// complete lookup/proof inventory. This is custody, not another fact copy.
    pub(super) fn pin<T>(
        &mut self,
        values: &Vec<T>,
        held: &mut ScopeCensus,
        retained: bool,
    ) -> surgeist_text::Result<()> {
        let next = Self::census(values)?;
        if *held == next {
            return Ok(());
        }
        self.materialize(next.records.saturating_sub(held.records))?;
        self.replace_census(*held, next, retained)?;
        *held = next;
        Ok(())
    }

    pub(super) fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
        held: &mut ScopeCensus,
        retained: bool,
    ) -> surgeist_text::Result<()> {
        self.pin(values, held, retained)?;
        let required = values
            .len()
            .checked_add(1)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        if required > values.capacity() {
            let requested = ScopeCensus {
                records: required,
                bytes: required
                    .checked_mul(size_of::<T>())
                    .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
            };
            self.check_live(self.live.add(requested)?)?;
            self.materialize(required)?;
            self.visits(values.len())?;
            let mut replacement = Vec::new();
            replacement.try_reserve_exact(required).map_err(|_| {
                refusal(ScopeAlignmentError::Allocation {
                    kind: ScopeAlignmentLimitKind::PayloadBytes,
                })
            })?;
            let capacity = Self::census(&replacement)?.bytes;
            self.check_live(self.live.add(ScopeCensus {
                records: required,
                bytes: capacity,
            })?)?;
            replacement.append(values);
            replacement.push(value);
            let old = std::mem::replace(values, replacement);
            drop(old);
            let next = Self::census(values)?;
            self.replace_census(*held, next, retained)?;
            *held = next;
        } else {
            self.materialize(1)?;
            let next = ScopeCensus {
                records: required,
                bytes: held.bytes,
            };
            self.replace_census(*held, next, retained)?;
            values.push(value);
            *held = next;
        }
        Ok(())
    }

    pub(super) fn release(&mut self, held: ScopeCensus) {
        self.live.records = self
            .live
            .records
            .checked_sub(held.records)
            .expect("scope scratch rows");
        self.live.bytes = self
            .live
            .bytes
            .checked_sub(held.bytes)
            .expect("scope scratch capacity");
    }

    /// These adapter rows contain fixed-size source handles, not nested buffers.
    pub(super) fn copy<T: Clone>(
        &mut self,
        source: &[T],
    ) -> surgeist_text::Result<(Vec<T>, ScopeCensus)> {
        self.copy_map(source, Clone::clone)
    }

    pub(super) fn copy_map<T, U>(
        &mut self,
        source: &[T],
        map: impl FnMut(&T) -> U,
    ) -> surgeist_text::Result<(Vec<U>, ScopeCensus)> {
        self.visits(source.len())?;
        self.materialize(source.len())?;
        let requested = ScopeCensus {
            records: source.len(),
            bytes: source
                .len()
                .checked_mul(size_of::<U>())
                .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
        };
        self.check_live(self.live.add(requested)?)?;
        let mut values = Vec::new();
        values.try_reserve_exact(source.len()).map_err(|_| {
            refusal(ScopeAlignmentError::Allocation {
                kind: ScopeAlignmentLimitKind::PayloadBytes,
            })
        })?;
        let census = ScopeCensus {
            records: source.len(),
            bytes: Self::census(&values)?.bytes,
        };
        self.check_live(self.live.add(census)?)?;
        values.extend(source.iter().map(map));
        self.live = self.live.add(census)?;
        Ok((values, census))
    }

    pub(super) fn retain_existing(&mut self, held: ScopeCensus) -> surgeist_text::Result<()> {
        self.retained = self.retained.add(held)?;
        Ok(())
    }
}

fn check(
    kind: ScopeAlignmentLimitKind,
    required: usize,
    limit: usize,
) -> surgeist_text::Result<()> {
    if required > limit {
        return Err(refusal(ScopeAlignmentError::Limit {
            kind,
            required,
            limit,
        }));
    }
    Ok(())
}
pub(super) fn overflow(kind: ScopeAlignmentLimitKind) -> surgeist_text::Error {
    refusal(ScopeAlignmentError::Overflow { kind })
}

mod custody;
pub(super) use custody::ScopeCensus;
pub(crate) use custody::{ScopeAlignmentResources, ScopeLease, ScopeOperation};

fn reserve_buffer<T>(
    limits: ScopeAlignmentLimits,
    external: ScopeCensus,
    live: ScopeCensus,
    count: usize,
    header: usize,
) -> surgeist_text::Result<(Vec<T>, ScopeCensus)> {
    let records = external
        .records
        .checked_add(live.records)
        .and_then(|rows| rows.checked_add(count))
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
    check(ScopeAlignmentLimitKind::Records, records, limits.records())?;
    let bytes = count
        .checked_mul(size_of::<T>())
        .and_then(|bytes| bytes.checked_add(header))
        .and_then(|bytes| bytes.checked_add(live.bytes))
        .and_then(|bytes| bytes.checked_add(external.bytes))
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
    check(
        ScopeAlignmentLimitKind::PayloadBytes,
        bytes,
        limits.payload_bytes(),
    )?;
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| {
        refusal(ScopeAlignmentError::Allocation {
            kind: ScopeAlignmentLimitKind::PayloadBytes,
        })
    })?;
    let census = ScopeCensus {
        records: count,
        bytes: values
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(header))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
    };
    let total = live
        .add(census)?
        .bytes
        .checked_add(external.bytes)
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
    check(
        ScopeAlignmentLimitKind::PayloadBytes,
        total,
        limits.payload_bytes(),
    )?;
    Ok((values, census))
}

pub(crate) fn allocation_buffer<T>(
    facts: &dyn ScopeAlignmentFacts,
    operation: &ScopeOperation<'_>,
    count: usize,
) -> surgeist_text::Result<(Vec<T>, ScopeLease)> {
    let retained = operation.input_union()?;
    facts.materialize(operation.limits(), retained.records, count)?;
    let (values, census) =
        reserve_buffer::<T>(operation.limits(), retained, facts.retention(), count, 0)?;
    let lease = facts.retain_payload(census)?;
    Ok((values, lease))
}

/// The native fitter consumes f64 facts without erasing the scalar ownership of
/// the admitted snapshot or narrowing font/reference positions through f32.
pub(crate) trait ScopeAlignmentFacts {
    fn retention(&self) -> ScopeCensus;
    fn materialize(
        &self,
        limits: ScopeAlignmentLimits,
        input_records: usize,
        count: usize,
    ) -> surgeist_text::Result<()>;
    fn visit(&self, limits: ScopeAlignmentLimits, count: usize) -> surgeist_text::Result<()>;
    fn retain_payload(&self, census: ScopeCensus) -> surgeist_text::Result<ScopeLease>;
    fn begin<'a>(
        &'a self,
        input: surgeist_text::ScopeAlignmentCensusInput<'a>,
        outputs: usize,
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>>;
    fn alignment(
        &self,
        scope: &CapturedScope,
        visit: &mut dyn FnMut() -> surgeist_text::Result<()>,
    ) -> surgeist_text::Result<VerticalAlignOf<f64>>;
}
impl<S: LayoutScalar> ScopeAlignmentFacts for ScopeAlignmentResources<S> {
    fn retention(&self) -> ScopeCensus {
        self.census()
    }
    fn materialize(
        &self,
        limits: ScopeAlignmentLimits,
        input_records: usize,
        count: usize,
    ) -> surgeist_text::Result<()> {
        ScopeAlignmentResources::materialize(self, limits, input_records, count)
    }
    fn visit(&self, limits: ScopeAlignmentLimits, count: usize) -> surgeist_text::Result<()> {
        ScopeAlignmentResources::visit(self, limits, count)
    }
    fn retain_payload(&self, census: ScopeCensus) -> surgeist_text::Result<ScopeLease> {
        self.lease(census)
    }
    fn begin<'a>(
        &'a self,
        input: surgeist_text::ScopeAlignmentCensusInput<'a>,
        outputs: usize,
    ) -> surgeist_text::Result<Option<ScopeOperation<'a>>> {
        ScopeAlignmentResources::begin(self, input, outputs)
    }
    fn alignment(
        &self,
        scope: &CapturedScope,
        visit: &mut dyn FnMut() -> surgeist_text::Result<()>,
    ) -> surgeist_text::Result<VerticalAlignOf<f64>> {
        let mut found = None;
        for row in &self.snapshot().rows {
            visit()?;
            if &row.scope == scope {
                found = Some(row);
                break;
            }
        }
        let row = found.ok_or_else(|| {
            refusal(ScopeAlignmentError::MissingAllocation {
                scope: scope.capture_ordinal(),
            })
        })?;
        Ok(match row.alignment {
            VerticalAlignOf::Baseline => VerticalAlignOf::Baseline,
            VerticalAlignOf::Top => VerticalAlignOf::Top,
            VerticalAlignOf::Bottom => VerticalAlignOf::Bottom,
            VerticalAlignOf::Shift(offset) => VerticalAlignOf::Shift(
                BaselineOffsetOf::try_new(offset.get().to_f64()).map_err(|_| {
                    numeric(
                        offset.get().to_f64(),
                        surgeist_text::TextNumericError::NonFinite,
                    )
                })?,
            ),
        })
    }
}

use super::fit::FitError;
use crate::inline::{InlineLineMetricGroupsOf, InlineMetricContributionOf};
use surgeist_text::{
    LineBoxAlignment, ScopeAllocation, ScopeAllocations, SelectedLine, SelectedScope,
};

struct ScopeNode {
    scope: SelectedScope,
    parent: Option<usize>,
    pending: usize,
    alignment: VerticalAlignOf<f64>,
    own: f64,
    cumulative: f64,
    metrics: Option<InlineMetricContributionOf<f64>>,
}

struct AllocationBudget<'a> {
    facts: &'a dyn ScopeAlignmentFacts,
    input_records: usize,
    limits: ScopeAlignmentLimits,
    records: usize,
    bytes: usize,
}
impl<'a> AllocationBudget<'a> {
    fn new(
        retained: ScopeCensus,
        scope: &CapturedScope,
        facts: &'a dyn ScopeAlignmentFacts,
    ) -> surgeist_text::Result<Self> {
        let limits = scope
            .scope_alignment_limits()
            .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
        let live = facts.retention();
        let records = retained
            .records
            .checked_add(live.records)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        let bytes = retained
            .bytes
            .checked_add(live.bytes)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        check(ScopeAlignmentLimitKind::Records, records, limits.records())?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            bytes,
            limits.payload_bytes(),
        )?;
        Ok(Self {
            facts,
            input_records: retained.records,
            limits,
            records,
            bytes,
        })
    }
    fn work(&mut self, count: usize) -> surgeist_text::Result<()> {
        self.facts.visit(self.limits, count)
    }
    fn buffer<T>(&mut self, count: usize) -> surgeist_text::Result<Vec<T>> {
        self.facts
            .materialize(self.limits, self.input_records, count)?;
        let records = self
            .records
            .checked_add(count)
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::Records))?;
        check(
            ScopeAlignmentLimitKind::Records,
            records,
            self.limits.records(),
        )?;
        let requested = count
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(self.bytes))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            requested,
            self.limits.payload_bytes(),
        )?;
        let mut values = Vec::new();
        values.try_reserve_exact(count).map_err(|_| {
            refusal(ScopeAlignmentError::Allocation {
                kind: ScopeAlignmentLimitKind::PayloadBytes,
            })
        })?;
        self.bytes = values
            .capacity()
            .checked_mul(size_of::<T>())
            .and_then(|bytes| bytes.checked_add(self.bytes))
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?;
        check(
            ScopeAlignmentLimitKind::PayloadBytes,
            self.bytes,
            self.limits.payload_bytes(),
        )?;
        self.records = records;
        Ok(values)
    }
}

/// Allocate from exact selected custody before the first geometry publication.
pub(crate) enum ScopeAllocationInput<'a> {
    Selected(&'a SelectedLine),
    Finalized(&'a surgeist_text::FinalizedLine),
}
pub(crate) struct AllocatedScopes {
    allocations: ScopeAllocations,
    lease: Option<ScopeLease>,
}
impl AllocatedScopes {
    pub(crate) fn allocations(&self) -> &ScopeAllocations {
        &self.allocations
    }
    /// The vector becomes actual Text-operation input at this handoff. Its
    /// capacity must no longer also be charged as external Layout storage.
    pub(crate) fn into_text(self) -> ScopeAllocations {
        let Self { allocations, lease } = self;
        drop(lease);
        allocations
    }
}
pub(crate) fn allocate(
    input: ScopeAllocationInput<'_>,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<AllocatedScopes, FitError> {
    let (selected, product) = match input {
        ScopeAllocationInput::Selected(line) => (
            line.clone(),
            surgeist_text::ScopeAlignmentProduct::Selected(line),
        ),
        ScopeAllocationInput::Finalized(line) => (
            line.selected_line(),
            surgeist_text::ScopeAlignmentProduct::Finalized(line),
        ),
    };
    let mut selected_scopes = selected.scopes();
    let Some(first) = selected_scopes.next() else {
        return Ok(AllocatedScopes {
            allocations: ScopeAllocations::None,
            lease: None,
        });
    };
    let operation = facts
        .begin(
            surgeist_text::ScopeAlignmentCensusInput::Product(product),
            0,
        )?
        .ok_or_else(|| {
            refusal(ScopeAlignmentError::MissingMetrics {
                scope: first.captured_scope().capture_ordinal(),
            })
        })?;
    let mut budget =
        AllocationBudget::new(operation.input_union()?, &first.captured_scope(), facts)?;
    let count = selected.scopes().len();
    let mut nodes = budget.buffer::<ScopeNode>(count)?;
    for scope in std::iter::once(first).chain(selected_scopes) {
        // A node unit includes its fixed queue/emission/frame-qualification
        // bookkeeping. Repeated association, child and key scans are separate.
        budget.work(1)?;
        let captured = scope.captured_scope();
        let alignment = facts.alignment(&captured, &mut || budget.work(1))?;
        let own = match alignment {
            VerticalAlignOf::Baseline => matching(&scope)?,
            VerticalAlignOf::Shift(offset) => sum(matching(&scope)?, offset.get())?,
            VerticalAlignOf::Top | VerticalAlignOf::Bottom => 0.0,
        };
        nodes.push(ScopeNode {
            scope,
            parent: None,
            pending: 0,
            alignment,
            own,
            cumulative: 0.0,
            metrics: None,
        });
    }
    for index in 0..nodes.len() {
        budget.work(1)?;
        let captured = nodes[index].scope.captured_scope();
        let Some(parent) = captured.parent() else {
            return Err(refusal(ScopeAlignmentError::MissingCapture).into());
        };
        if parent.domain() != captured.domain() {
            return Err(refusal(ScopeAlignmentError::WrongDomain).into());
        }
        if parent.role() == surgeist_text::ScopeRole::Inline {
            let mut actual = None;
            for (candidate, node) in nodes.iter().enumerate() {
                budget.work(1)?;
                if node.scope.captured_scope() == parent {
                    actual = Some(candidate);
                    break;
                }
            }
            let actual = actual.ok_or_else(|| {
                refusal(if parent.alignment_boundary().is_none() {
                    ScopeAlignmentError::MissingBoundary {
                        scope: parent.capture_ordinal(),
                    }
                } else {
                    ScopeAlignmentError::MissingAllocation {
                        scope: parent.capture_ordinal(),
                    }
                })
            })?;
            nodes[index].parent = Some(actual);
            nodes[actual].pending += 1;
        }
    }
    let mut ready = budget.buffer::<usize>(count)?;
    for (index, node) in nodes.iter().enumerate() {
        if node.pending == 0 {
            ready.push(index);
        }
    }
    let mut completed = 0;
    let mut root = InlineLineMetricGroupsOf::<f64>::typographic();
    let mut root_present = false;
    while let Some(index) = ready.pop() {
        budget.work(1)?;
        let mut groups = InlineLineMetricGroupsOf::<f64>::typographic();
        let mut present = false;
        for contribution in nodes[index].scope.direct_metric_contributions() {
            budget.work(1)?;
            if matches!(
                contribution.alignment,
                LineBoxAlignment::Top | LineBoxAlignment::Bottom
            ) {
                // Edge-aligned leaf subtrees are independent of this lexical
                // ancestor's aligned subtree and meet containing line edges.
                include(
                    &mut root,
                    contribution.before,
                    contribution.after,
                    text_alignment(contribution.alignment)?,
                )?;
                root_present = true;
            } else {
                include(
                    &mut groups,
                    contribution.before,
                    contribution.after,
                    text_alignment(contribution.alignment)?,
                )?;
                present = true;
            }
        }
        for child in &nodes {
            budget.work(1)?;
            if child.parent != Some(index) {
                continue;
            }
            if matches!(
                child.alignment,
                VerticalAlignOf::Top | VerticalAlignOf::Bottom
            ) {
                continue;
            }
            if let Some(metrics) = child.metrics {
                include(
                    &mut groups,
                    metrics.baseline,
                    metrics.after_baseline,
                    shifted(child.own)?,
                )?;
                present = true;
            }
        }
        nodes[index].metrics = if present { Some(finish(groups)?) } else { None };
        completed += 1;
        if let Some(parent) = nodes[index].parent {
            nodes[parent].pending -= 1;
            if nodes[parent].pending == 0 {
                ready.push(parent);
            }
        }
    }
    if completed != nodes.len() {
        return Err(refusal(ScopeAlignmentError::WrongDomain).into());
    }
    for contribution in selected.unscoped_metric_contributions() {
        budget.work(1)?;
        include(
            &mut root,
            contribution.before,
            contribution.after,
            text_alignment(contribution.alignment)?,
        )?;
        root_present = true;
    }
    for node in &nodes {
        budget.work(1)?;
        if node.parent.is_some()
            && !matches!(
                node.alignment,
                VerticalAlignOf::Top | VerticalAlignOf::Bottom
            )
        {
            continue;
        }
        if let Some(metrics) = node.metrics {
            let alignment = match node.alignment {
                VerticalAlignOf::Baseline | VerticalAlignOf::Shift(_) => shifted(node.own)?,
                alignment => alignment,
            };
            include(
                &mut root,
                metrics.baseline,
                metrics.after_baseline,
                alignment,
            )?;
            root_present = true;
        }
    }
    let envelope = if root_present {
        Some(finish(root)?)
    } else {
        None
    };
    ready.clear();
    for (index, node) in nodes.iter().enumerate() {
        if node.parent.is_none() {
            ready.push(index);
        }
    }
    while let Some(index) = ready.pop() {
        let parent = nodes[index]
            .parent
            .map_or(0.0, |parent| nodes[parent].cumulative);
        let node = &mut nodes[index];
        // Empty aligned subtrees supply a coordinate only. They do not add a
        // metric group, root strut or line extent.
        let local = node.metrics.unwrap_or(InlineMetricContributionOf {
            baseline: 0.0,
            after_baseline: 0.0,
        });
        let edges = envelope.unwrap_or(InlineMetricContributionOf {
            baseline: 0.0,
            after_baseline: 0.0,
        });
        node.cumulative = match node.alignment {
            VerticalAlignOf::Top => sum(edges.baseline, -local.baseline)?,
            VerticalAlignOf::Bottom => sum(local.after_baseline, -edges.after_baseline)?,
            _ => sum(parent, node.own)?,
        };
        if matches!(
            node.alignment,
            VerticalAlignOf::Top | VerticalAlignOf::Bottom
        ) {
            node.own = sum(node.cumulative, -parent)?;
        }
        for (child, node) in nodes.iter().enumerate() {
            budget.work(1)?;
            if node.parent == Some(index) {
                ready.push(child);
            }
        }
    }
    let mut allocations = budget.buffer::<ScopeAllocation>(count)?;
    for node in nodes {
        allocations.push(ScopeAllocation {
            scope: node.scope,
            displacement: surgeist_text::BaselineShift::try_new(node.own)?,
        });
    }
    let census = ScopeCensus {
        records: allocations.len(),
        bytes: allocations
            .capacity()
            .checked_mul(size_of::<ScopeAllocation>())
            .ok_or_else(|| overflow(ScopeAlignmentLimitKind::PayloadBytes))?,
    };
    let lease = facts.retain_payload(census)?;
    Ok(AllocatedScopes {
        allocations: ScopeAllocations::Complete(allocations),
        lease: Some(lease),
    })
}

fn matching(scope: &SelectedScope) -> surgeist_text::Result<f64> {
    let actual = scope.captured_scope();
    let own = scope.metrics();
    if own.captured_scope() != actual {
        return Err(refusal(ScopeAlignmentError::ForeignCapture));
    }
    let parent = scope.parent_metrics().ok_or_else(|| {
        refusal(ScopeAlignmentError::MissingMetrics {
            scope: actual.capture_ordinal(),
        })
    })?;
    if actual.parent().as_ref() != Some(&parent.captured_scope()) {
        return Err(refusal(ScopeAlignmentError::UnrepresentedOwnership {
            scope: actual.capture_ordinal(),
        }));
    }
    if own.orientation() != parent.orientation()
        || own.common_baseline_reference() != parent.common_baseline_reference()
    {
        return Err(refusal(ScopeAlignmentError::UnsupportedReference {
            scope: actual.capture_ordinal(),
        }));
    }
    let reference = parent.font_reference();
    if own.font_reference().axis != reference.axis {
        return Err(refusal(ScopeAlignmentError::UnsupportedReference {
            scope: actual.capture_ordinal(),
        }));
    }
    let before = parent
        .matching_coordinate(reference.baseline)
        .ok_or_else(|| {
            refusal(ScopeAlignmentError::UnsupportedReference {
                scope: parent.captured_scope().capture_ordinal(),
            })
        })?;
    let after = own.matching_coordinate(reference.baseline).ok_or_else(|| {
        refusal(ScopeAlignmentError::UnsupportedReference {
            scope: actual.capture_ordinal(),
        })
    })?;
    sum(before, -after)
}

fn text_alignment(alignment: LineBoxAlignment) -> surgeist_text::Result<VerticalAlignOf<f64>> {
    Ok(match alignment {
        LineBoxAlignment::Baseline => VerticalAlignOf::Baseline,
        LineBoxAlignment::Top => VerticalAlignOf::Top,
        LineBoxAlignment::Bottom => VerticalAlignOf::Bottom,
        LineBoxAlignment::Shift(offset) => shifted(offset.get())?,
    })
}
fn shifted(value: f64) -> surgeist_text::Result<VerticalAlignOf<f64>> {
    Ok(VerticalAlignOf::Shift(
        BaselineOffsetOf::try_new(value)
            .map_err(|_| numeric(value, surgeist_text::TextNumericError::NonFinite))?,
    ))
}
fn include(
    groups: &mut InlineLineMetricGroupsOf<f64>,
    before: f64,
    after: f64,
    alignment: VerticalAlignOf<f64>,
) -> surgeist_text::Result<()> {
    let _ = sum(before, after)?;
    groups.include(
        alignment,
        InlineMetricContributionOf {
            baseline: before,
            after_baseline: after,
        },
    );
    Ok(())
}
fn finish(
    groups: InlineLineMetricGroupsOf<f64>,
) -> surgeist_text::Result<InlineMetricContributionOf<f64>> {
    let metrics = groups.checked().map_err(|error| {
        numeric(
            error.value(),
            match error.reason() {
                crate::LayoutTextNumericError::NonFinite => {
                    surgeist_text::TextNumericError::NonFinite
                }
                crate::LayoutTextNumericError::OutOfRange => {
                    surgeist_text::TextNumericError::OutOfRange
                }
                crate::LayoutTextNumericError::NonZeroLost => {
                    surgeist_text::TextNumericError::NonZeroLost
                }
            },
        )
    })?;
    let _ = sum(metrics.baseline, metrics.after_baseline)?;
    Ok(metrics)
}
fn sum(base: f64, delta: f64) -> surgeist_text::Result<f64> {
    let value = base + delta;
    if !value.is_finite() {
        return Err(numeric(value, surgeist_text::TextNumericError::NonFinite));
    }
    if (delta != 0.0 && value == base) || (base != 0.0 && value == delta) {
        return Err(numeric(delta, surgeist_text::TextNumericError::NonZeroLost));
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

/// Deterministic allocation proof; cloning shares the actual payload and lease.
#[derive(Clone, Debug, Default)]
pub(crate) struct ScopeBasis(Option<Arc<ScopeProof>>);
#[derive(Debug)]
struct ScopeProof {
    rows: Vec<surgeist_text::ScopeFrameAllocation>,
    _lease: ScopeLease,
}
impl PartialEq for ScopeBasis {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().map(|proof| &proof.rows) == other.0.as_ref().map(|proof| &proof.rows)
    }
}

/// The canonical paragraph's per-line handles. Cloning a pending paragraph or
/// packet shares this exact buffer and its lease, rather than copying an
/// unaccounted feature-owned Vec. None records actual all-absent frame facts.
#[derive(Clone, Debug, Default)]
pub(crate) struct ScopeBases(Option<Arc<ScopeBasesData>>);
#[derive(Debug)]
struct ScopeBasesData {
    rows: Vec<ScopeBasis>,
    _lease: ScopeLease,
}
impl ScopeBases {
    pub(crate) fn from_lines<S: LayoutScalar>(
        lines: &[super::fit::FittedLine],
        resources: &ScopeAlignmentResources<S>,
    ) -> surgeist_text::Result<Self> {
        let limits = resources.key_limits()?;
        if let Some(limits) = limits {
            // This scan establishes actual absence or identifies the genuine
            // selected product used for simultaneous Text/Layout custody.
            resources.visit(limits, lines.len())?;
        }
        let Some(line) = lines.iter().find(|line| line.scopes.0.is_some()) else {
            return Ok(Self::default());
        };
        let first = line
            .scopes
            .0
            .as_ref()
            .and_then(|proof| proof.rows.first())
            .expect("a canonical scope proof retains at least one real frame");
        let work_limits = limits.ok_or_else(|| {
            refusal(ScopeAlignmentError::MissingAllocation {
                scope: first.scope.capture_ordinal(),
            })
        })?;
        let limits = first
            .scope
            .scope_alignment_limits()
            .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
        let operation = resources
            .begin(
                surgeist_text::ScopeAlignmentCensusInput::Product(
                    surgeist_text::ScopeAlignmentProduct::Finalized(&line.finalized),
                ),
                0,
            )?
            .ok_or_else(|| {
                refusal(ScopeAlignmentError::MissingMetrics {
                    scope: first.scope.capture_ordinal(),
                })
            })?;
        let retained = operation.input_union()?;
        resources.materialize(limits, retained.records, lines.len())?;
        resources.visit(work_limits, lines.len())?;
        let (mut rows, census) = reserve_buffer::<ScopeBasis>(
            limits,
            retained,
            ScopeAlignmentFacts::retention(resources),
            lines.len(),
            size_of::<ScopeBasesData>(),
        )?;
        rows.extend(lines.iter().map(|line| line.scopes.clone()));
        let lease = resources.lease(census)?;
        Ok(Self(Some(Arc::new(ScopeBasesData {
            rows,
            _lease: lease,
        }))))
    }

    pub(crate) fn iter(&self, lines: usize) -> impl Iterator<Item = ScopeBasis> + '_ {
        if let Some(data) = &self.0 {
            assert_eq!(
                data.rows.len(),
                lines,
                "scope handles cover every canonical line"
            );
        }
        (0..lines).map(|index| match &self.0 {
            Some(data) => data.rows[index].clone(),
            None => ScopeBasis::default(),
        })
    }
}

impl ScopeBasis {
    pub(crate) fn comparison_rows(&self) -> usize {
        self.0.as_ref().map_or(0, |proof| proof.rows.len())
    }

    pub(crate) fn admit_comparisons<'a>(
        &self,
        facts: &dyn ScopeAlignmentFacts,
        others: impl Iterator<Item = &'a ScopeBasis>,
    ) -> surgeist_text::Result<()> {
        let Some(proof) = &self.0 else {
            return Ok(());
        };
        let Some(first) = proof.rows.first() else {
            return Ok(());
        };
        admit_comparison_rows(
            &first.scope,
            proof.rows.len(),
            others.map(ScopeBasis::comparison_rows),
            facts,
        )
    }
}

/// Pure Eq callbacks cannot return a resource refusal. Admit a conservative
/// bound before entering the stable native history: one descriptor inspection
/// per key and, for equal lengths, every possible canonical frame comparison.
fn admit_comparison_rows(
    scope: &CapturedScope,
    rows: usize,
    others: impl Iterator<Item = usize>,
    facts: &dyn ScopeAlignmentFacts,
) -> surgeist_text::Result<()> {
    let limits = scope
        .scope_alignment_limits()
        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    for other in others {
        admit_key_comparison(limits, rows, other, facts)?;
    }
    Ok(())
}

/// One descriptor equality plus the entire row equality that equal lengths
/// permit. This is a conservative admission bound, not observed execution work.
pub(crate) fn admit_key_comparison(
    limits: ScopeAlignmentLimits,
    rows: usize,
    other: usize,
    facts: &dyn ScopeAlignmentFacts,
) -> surgeist_text::Result<()> {
    let work = 1usize
        .checked_add(if rows == other { rows } else { 0 })
        .ok_or_else(|| overflow(ScopeAlignmentLimitKind::StructuralWork))?;
    facts.visit(limits, work)
}

pub(crate) fn admit_frame_comparisons(
    line: &surgeist_text::FinalizedLine,
    others: impl Iterator<Item = usize>,
    facts: &dyn ScopeAlignmentFacts,
) -> surgeist_text::Result<()> {
    let Some(first) = line.scope_allocations().first() else {
        return Ok(());
    };
    admit_comparison_rows(&first.scope, line.scope_allocations().len(), others, facts)
}

pub(crate) fn basis(
    line: &surgeist_text::FinalizedLine,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<ScopeBasis, FitError> {
    let rows = line.scope_allocations();
    let Some(first) = rows.first() else {
        return Ok(ScopeBasis::default());
    };
    let limits = first
        .scope
        .scope_alignment_limits()
        .ok_or_else(|| refusal(ScopeAlignmentError::LimitsRequired))?;
    let operation = facts
        .begin(
            surgeist_text::ScopeAlignmentCensusInput::Product(
                surgeist_text::ScopeAlignmentProduct::Finalized(line),
            ),
            0,
        )?
        .ok_or_else(|| {
            refusal(ScopeAlignmentError::MissingMetrics {
                scope: first.scope.capture_ordinal(),
            })
        })?;
    let retained = operation.input_union()?;
    facts.materialize(limits, retained.records, rows.len())?;
    facts.visit(limits, rows.len())?;
    let (mut values, census) = reserve_buffer::<surgeist_text::ScopeFrameAllocation>(
        limits,
        retained,
        facts.retention(),
        rows.len(),
        size_of::<ScopeProof>(),
    )?;
    values.extend_from_slice(rows);
    let lease = facts.retain_payload(census)?;
    Ok(ScopeBasis(Some(Arc::new(ScopeProof {
        rows: values,
        _lease: lease,
    }))))
}

pub(crate) fn matches(
    line: &surgeist_text::FinalizedLine,
    allocations: &ScopeAllocations,
    facts: &dyn ScopeAlignmentFacts,
) -> Result<bool, FitError> {
    if let ScopeAllocations::Complete(rows) = allocations {
        admit_frame_comparisons(line, std::iter::once(rows.len()), facts)?;
    }
    Ok(match allocations {
        ScopeAllocations::None => line.scope_allocations().is_empty(),
        ScopeAllocations::Complete(rows) => {
            rows.len() == line.scope_allocations().len()
                && rows
                    .iter()
                    .zip(line.scope_allocations())
                    .all(|(allocation, actual)| {
                        allocation.scope.captured_scope() == actual.scope
                            && allocation.displacement == actual.displacement
                    })
        }
    })
}

#[cfg(test)]
mod numeric_tests {
    use super::*;

    #[test]
    fn scope_numeric_signed_parent_and_child_envelopes() {
        // Parent occupies [-8, 2]. Child occupies [-6, 4] before shifting.
        // +3 gives [-9, 1], so the union is [-9, 2]; -3 gives [-3, 7],
        // so that union is [-8, 7]. These are native typographic coordinates.
        for (offset, before, after) in [(3.0, 9.0, 2.0), (-3.0, 8.0, 7.0)] {
            let mut groups = InlineLineMetricGroupsOf::<f64>::typographic();
            include(&mut groups, 8.0, 2.0, VerticalAlignOf::Baseline).unwrap();
            include(&mut groups, 6.0, 4.0, shifted(offset).unwrap()).unwrap();
            let actual = finish(groups).unwrap();
            assert_eq!((actual.baseline, actual.after_baseline), (before, after));
        }
    }

    #[test]
    fn scope_numeric_absent_parent_does_not_add_a_zero_strut() {
        // Moving the sole [-6,4] child up20 leaves [-26,-16], still height10.
        // The common baseline lies outside that occupied interval.
        let mut groups = InlineLineMetricGroupsOf::<f64>::typographic();
        include(&mut groups, 6.0, 4.0, shifted(20.0).unwrap()).unwrap();
        let actual = finish(groups).unwrap();
        assert_eq!((actual.baseline, actual.after_baseline), (26.0, -16.0));
        assert_eq!(actual.baseline + actual.after_baseline, 10.0);
    }

    #[test]
    fn scope_numeric_edge_group_preserves_native_baseline_tie() {
        // The baseline component is [-8,2]. A separate20-high top leaf
        // extends under to12; it supplies no new font baseline witness.
        let mut groups = InlineLineMetricGroupsOf::<f64>::typographic();
        include(&mut groups, 8.0, 2.0, VerticalAlignOf::Baseline).unwrap();
        include(&mut groups, 6.0, 14.0, VerticalAlignOf::Top).unwrap();
        let actual = finish(groups).unwrap();
        assert_eq!((actual.baseline, actual.after_baseline), (8.0, 12.0));
    }

    #[test]
    fn scope_numeric_sum_preserves_cancellation_and_refuses_absorption() {
        assert_eq!(sum(20.0, -20.0).unwrap().to_bits(), 0.0_f64.to_bits());
        assert!(sum(f64::MAX, f64::MAX).is_err());
        assert!(sum(1.0e100, 1.0).is_err());
        assert!(sum(1.0, 1.0e100).is_err());
    }

    fn scalar_lane<S: LayoutScalar>() {
        assert_eq!(scalar_context::<S>(3.0).unwrap(), S::from_f64(3.0));
        assert_eq!(scalar_context::<S>(-3.0).unwrap(), S::from_f64(-3.0));
        let error = scalar_context::<S>(f64::INFINITY).unwrap_err();
        assert!(
            matches!(error.detail(), Some(surgeist_text::ErrorDetail::InvalidTextNumeric {
            category: surgeist_text::TextNumericError::NonFinite, value, ..
        }) if *value == f64::INFINITY)
        );
    }

    #[test]
    fn scope_numeric_scalar_context_preserves_f32_categories() {
        scalar_lane::<f32>();
        for (input, category) in [
            (f64::MAX, surgeist_text::TextNumericError::OutOfRange),
            (
                f64::MIN_POSITIVE,
                surgeist_text::TextNumericError::NonZeroLost,
            ),
        ] {
            let error = scalar_context::<f32>(input).unwrap_err();
            assert!(
                matches!(error.detail(), Some(surgeist_text::ErrorDetail::InvalidTextNumeric {
                category: actual, value, ..
            }) if *actual == category && *value == input)
            );
        }
    }

    #[test]
    fn scope_numeric_scalar_context_preserves_f64_categories() {
        scalar_lane::<f64>();
        assert_eq!(scalar_context::<f64>(f64::MAX).unwrap(), f64::MAX);
        assert_eq!(
            scalar_context::<f64>(f64::MIN_POSITIVE).unwrap(),
            f64::MIN_POSITIVE
        );
    }
}

//! Controlled UTF16 property input over the original CSS syntax owners.
use std::cell::Cell;
use std::mem::{size_of, size_of_val};
use std::{cell::RefCell, sync::Arc};

use crate::{CssComponentValueLimits, CssUtf16InputLimits};

/// One complete original UTF16 input and its checked authored color-scheme
/// declaration. Global and substitution-dependent values remain symbolic.
#[derive(Debug)]
pub struct CssColorSchemeParseReport {
    source: crate::CssParseSource,
    result: Result<crate::CssDeclaration, crate::CssPropertyValueError>,
    retained: CssUtf16ValueStorage,
}
impl CssColorSchemeParseReport {
    #[must_use]
    pub const fn source(&self) -> &crate::CssParseSource {
        &self.source
    }
    pub const fn result(&self) -> &Result<crate::CssDeclaration, crate::CssPropertyValueError> {
        &self.result
    }
    /// Exact immutable producer receipt, including the original Vec capacity,
    /// source graph, report and actual checked output allocations.
    #[must_use]
    pub const fn retained_storage(&self) -> CssUtf16ValueStorage {
        self.retained
    }

    /// Compares meaningful captured input and checked contents. Fresh source and
    /// declaration occurrences are deliberately not semantic differences.
    pub fn equivalent_with<E>(
        &self,
        other: &Self,
        mut admit: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, crate::CssRetainedPayloadError<E>> {
        use crate::CssRetainedPayloadError;
        admit().map_err(CssRetainedPayloadError::admission)?;
        if self.source.context() != other.source.context() {
            return Ok(false);
        }
        admit().map_err(CssRetainedPayloadError::admission)?;
        let left = self.source.source().original_utf16().expect("UTF16 report");
        admit().map_err(CssRetainedPayloadError::admission)?;
        let right = other
            .source
            .source()
            .original_utf16()
            .expect("UTF16 report");
        admit().map_err(CssRetainedPayloadError::admission)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for index in 0..left.len() {
            admit().map_err(CssRetainedPayloadError::admission)?;
            let a = left[index];
            admit().map_err(CssRetainedPayloadError::admission)?;
            if a != right[index] {
                return Ok(false);
            }
        }
        admit().map_err(CssRetainedPayloadError::admission)?;
        match (&self.result, &other.result) {
            (Ok(left), Ok(right)) => {
                if !crate::component_values::equivalent_in_same_input_with(
                    left.value_components(),
                    right.value_components(),
                    &mut admit,
                )? {
                    return Ok(false);
                }
                equivalent_bodies(left, right, &mut admit)
            }
            // These diagnostics are issued privately by the same deterministic
            // language owner over the complete equal input/context above.
            // Their source occurrences differ, but their checked rejection is
            // the same. No consumer-supplied diagnostic can enter this report.
            (Err(left), Err(right)) => Ok(left.admission_kind() == right.admission_kind()),
            _ => Ok(false),
        }
    }
}

impl crate::CssRetainedSources for CssColorSchemeParseReport {
    fn visit_retained_sources(
        &self,
        sources: &mut crate::CssSourceRetention,
    ) -> Result<(), crate::CssSourceRetentionError> {
        // All component and diagnostic origins are issued from this exact
        // snapshot; retaining the complete source also covers empty input.
        sources.source(self.source.source())
    }
}

fn text_equal<E>(
    left: &str,
    right: &str,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, crate::CssRetainedPayloadError<E>> {
    use crate::CssRetainedPayloadError;
    admit().map_err(CssRetainedPayloadError::admission)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for index in 0..left.len() {
        admit().map_err(CssRetainedPayloadError::admission)?;
        let a = left.as_bytes()[index];
        admit().map_err(CssRetainedPayloadError::admission)?;
        if a != right.as_bytes()[index] {
            return Ok(false);
        }
    }
    Ok(true)
}

fn equivalent_bodies<E>(
    left: &crate::CssDeclaration,
    right: &crate::CssDeclaration,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, crate::CssRetainedPayloadError<E>> {
    use crate::{
        CssKnownDeclaredValueRef as Value, CssKnownPropertyValueRef as Property,
        CssRetainedPayloadError,
    };
    admit().map_err(CssRetainedPayloadError::admission)?;
    let left = left.known().expect("checked color-scheme");
    admit().map_err(CssRetainedPayloadError::admission)?;
    let right = right.known().expect("checked color-scheme");
    match (left.declared_value(), right.declared_value()) {
        (Value::Global(a), Value::Global(b)) => Ok(a == b),
        (Value::SubstitutionDependent(a), Value::SubstitutionDependent(b)) => {
            text_equal(a.as_css(), b.as_css(), admit)
        }
        (Value::Property(Property::ColorScheme(a)), Value::Property(Property::ColorScheme(b))) => {
            if !text_equal(a.as_css(), b.as_css(), admit)? {
                return Ok(false);
            }
            admit().map_err(CssRetainedPayloadError::admission)?;
            if a.value().only() != b.value().only()
                || a.value().schemes().len() != b.value().schemes().len()
            {
                return Ok(false);
            }
            for (a, b) in a.value().schemes().iter().zip(b.value().schemes()) {
                admit().map_err(CssRetainedPayloadError::admission)?;
                match (a, b) {
                    (crate::CssColorSchemeKeyword::Light, crate::CssColorSchemeKeyword::Light)
                    | (crate::CssColorSchemeKeyword::Dark, crate::CssColorSchemeKeyword::Dark) => {}
                    (
                        crate::CssColorSchemeKeyword::Custom(a),
                        crate::CssColorSchemeKeyword::Custom(b),
                    ) => {
                        if !text_equal(a.as_str(), b.as_str(), admit)? {
                            return Ok(false);
                        }
                    }
                    _ => return Ok(false),
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

impl crate::CssColorScheme {
    /// Preprocesses the actual shared UTF16 input and checks the complete
    /// `color-scheme` authored property grammar at normal importance.
    ///
    /// The parser supplies the grammar context and the component limits that
    /// charge the lexical input; `limits` is the rest of the operation envelope.
    ///
    /// Every input, lexical, component and grammar owner shares this cumulative
    /// admission. Prospective live observations precede allocation; release is
    /// infallible and follows actual drops. Successful return transfers exactly
    /// `report.retained_storage()` to the caller. Resource refusal returns no
    /// partial report and preserves the original admission error.
    ///
    /// # Errors
    ///
    /// Returns the exhausted resource, the failed allocation, the capacity
    /// overflow or the caller's own admission error; no partial report escapes.
    pub fn parse_utf16<E>(
        parser: &crate::CssParser,
        original: Arc<Vec<u16>>,
        limits: CssUtf16PropertyValueLimits,
        admit: impl FnMut(CssUtf16ValueAdmission) -> Result<(), E>,
        release: impl FnMut(CssUtf16ValueStorage),
    ) -> Result<CssColorSchemeParseReport, crate::CssUtf16InputError<E>> {
        let admit = RefCell::new(admit);
        let release = RefCell::new(release);
        let cause = RefCell::new(None);
        let admitted = |event| match (admit.borrow_mut())(event) {
            Ok(()) => Ok(()),
            Err(error) => {
                *cause.borrow_mut() = Some(error);
                Err(())
            }
        };
        let released = |usage| (release.borrow_mut())(usage);
        let control = Control::new(limits, &admitted, &released);
        let fixed = size_of::<Control<'_>>()
            + size_of_val(&admit)
            + size_of_val(&release)
            + size_of_val(&cause)
            + size_of_val(&admitted)
            + size_of_val(&released)
            + size_of::<CssColorSchemeParseReport>();
        let outcome = if control.add(fixed).is_err() {
            drop(original);
            Err(crate::CssUtf16InputError::interrupted(()))
        } else {
            parse_operation(
                parser.context(),
                parser.component_limits(),
                original,
                limits,
                &control,
            )
        };
        let failure = control.failure();
        let outcome = match outcome {
            Ok(report) => match control.transfer(report.retained) {
                Ok(()) => Ok(report),
                Err(()) => Err(crate::CssUtf16InputError::interrupted(())),
            },
            Err(error) => Err(error),
        };
        let failure = control.failure().or(failure);
        drop(control);
        match outcome {
            Ok(report) => {
                // Actual control/register custody has now ended. The caller
                // receives exactly the immutable result payload, not scratch.
                released(report.retained);
                Ok(report)
            }
            Err(error) => {
                use crate::CssUtf16InputError as Error;
                match failure {
                    Some(Failure::Admission) => Err(Error::interrupted(
                        cause.borrow_mut().take().expect("original admission error"),
                    )),
                    Some(Failure::Limit(resource)) => Err(Error::limit(resource)),
                    Some(Failure::Allocation) => Err(Error::allocation()),
                    Some(Failure::CapacityOverflow) => Err(Error::capacity_overflow()),
                    None => Err(error
                        .map_interruption(|()| unreachable!("controlled interruption is latched"))),
                }
            }
        }
    }
}

fn parse_operation(
    context: crate::CssParserContext,
    syntax: CssComponentValueLimits,
    original: Arc<Vec<u16>>,
    limits: CssUtf16PropertyValueLimits,
    control: &Control<'_>,
) -> Result<CssColorSchemeParseReport, crate::CssUtf16InputError<()>> {
    use crate::CssUtf16InputError as Error;
    let refused = |()| Error::interrupted(());
    let baseline = control.live().bytes;
    let source = crate::utf16_input::prepare_utf16(
        original,
        limits.input,
        || control.step(),
        |bytes, units| {
            let total = baseline.checked_add(bytes).ok_or_else(|| {
                let _ = control.fail::<()>(Failure::CapacityOverflow);
            })?;
            control.set_live(total, units)
        },
        |bytes, units| control.reconciled(baseline + bytes, units),
    )?;
    let result = match crate::component_values::parse_controlled_snapshot(&source, syntax, control)
    {
        Err(error) => {
            if control.failure().is_some() {
                return Err(Error::interrupted(()));
            }
            control.component_resource(error.kind).map_err(refused)?;
            Err(crate::CssPropertyValueError::admission(error))
        }
        Ok(components) => {
            match crate::parser::parse_color_scheme_value_controlled(
                &components,
                context,
                &source,
                control,
            )
            .map_err(refused)?
            {
                Err(error) => Err(error),
                Ok(body) => {
                    control
                        .add(crate::CssDeclaration::constructed_allocation_bytes())
                        .map_err(refused)?;
                    Ok(crate::CssDeclaration::new_constructed(
                        context,
                        body,
                        crate::CssImportance::Normal,
                        components,
                    ))
                }
            }
        }
    };
    control.step().map_err(refused)?;
    let source_bytes = source
        .payload_bytes(true)
        .ok_or_else(Error::capacity_overflow)?;
    let output_bytes = match &result {
        Ok(declaration) => {
            let components = crate::component_values::retained_backing_bytes_with(
                declaration.value_components(),
                &mut || control.step(),
            )
            .map_err(|error| payload_failure(error, control))?;
            let body = body_backing(declaration, &mut || control.step())
                .map_err(|error| payload_failure(error, control))?;
            crate::CssDeclaration::constructed_allocation_bytes()
                .checked_add(components)
                .and_then(|bytes| bytes.checked_add(body))
                .ok_or_else(Error::capacity_overflow)?
        }
        Err(error) => error
            .retained_backing_bytes_with(&mut || control.step())
            .map_err(|error| payload_failure(error, control))?,
    };
    let bytes = source_bytes
        .checked_add(size_of::<CssColorSchemeParseReport>())
        .and_then(|bytes| bytes.checked_add(output_bytes))
        .ok_or_else(Error::capacity_overflow)?;
    let units = source.original_utf16().expect("UTF16 source").capacity();
    Ok(CssColorSchemeParseReport {
        source: crate::CssParseSource::new(source, context),
        result,
        retained: CssUtf16ValueStorage {
            bytes,
            utf16_units: units,
        },
    })
}

fn payload_failure(
    error: crate::CssRetainedPayloadError<()>,
    control: &Control<'_>,
) -> crate::CssUtf16InputError<()> {
    match error.into_admission() {
        Ok(()) => crate::CssUtf16InputError::interrupted(()),
        Err(_) => {
            let _ = control.fail::<()>(Failure::CapacityOverflow);
            crate::CssUtf16InputError::capacity_overflow()
        }
    }
}

fn body_backing<E>(
    declaration: &crate::CssDeclaration,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, crate::CssRetainedPayloadError<E>> {
    use crate::{
        CssKnownDeclaredValueRef as Value, CssKnownPropertyValueRef as Property,
        CssRetainedPayloadError,
    };
    admit().map_err(CssRetainedPayloadError::admission)?;
    match declaration
        .known()
        .expect("checked color-scheme")
        .declared_value()
    {
        Value::Global(_) => Ok(0),
        Value::SubstitutionDependent(value) => {
            admit().map_err(CssRetainedPayloadError::admission)?;
            Ok(value.backing_capacity())
        }
        Value::Property(Property::ColorScheme(value)) => value.retained_backing_bytes_with(admit),
        _ => unreachable!("producer fixes color-scheme grammar"),
    }
}

/// Independent preparation policy and the complete operation envelope. The
/// component limits that charge the lexical input come from the
/// [`crate::CssParser`] that drives the operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUtf16PropertyValueLimits {
    pub input: CssUtf16InputLimits,
    pub max_work: usize,
    pub max_live_bytes: usize,
    pub max_retained_bytes: usize,
}

/// Actual owned payload, including fixed output/source headers. Opaque allocator
/// and Arc control-block overhead is outside the repository payload tariff.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CssUtf16ValueStorage {
    bytes: usize,
    utf16_units: usize,
}
impl CssUtf16ValueStorage {
    #[must_use]
    pub const fn bytes(self) -> usize {
        self.bytes
    }
    /// Original UTF16 Vec capacity; normalized UTF8 and decoded names add bytes.
    #[must_use]
    pub const fn utf16_units(self) -> usize {
        self.utf16_units
    }
}

/// One actual observation or prospective/current source-inclusive live extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssUtf16ValueAdmission {
    Work,
    Live(CssUtf16ValueStorage),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Failure {
    Limit(crate::CssUtf16Resource),
    Allocation,
    CapacityOverflow,
    Admission,
}

/// One original caller admission, shared by every called CSS owner. Generic
/// caller errors remain in the front's original closure; no parser copies them.
pub(crate) struct Control<'a> {
    limits: CssUtf16PropertyValueLimits,
    admit: &'a dyn Fn(CssUtf16ValueAdmission) -> Result<(), ()>,
    release: &'a dyn Fn(CssUtf16ValueStorage),
    work: Cell<usize>,
    live: Cell<CssUtf16ValueStorage>,
    failure: Cell<Option<Failure>>,
    transferred: Cell<bool>,
}
impl<'a> Control<'a> {
    pub(crate) fn new(
        limits: CssUtf16PropertyValueLimits,
        admit: &'a dyn Fn(CssUtf16ValueAdmission) -> Result<(), ()>,
        release: &'a dyn Fn(CssUtf16ValueStorage),
    ) -> Self {
        Self {
            limits,
            admit,
            release,
            work: Cell::new(0),
            live: Cell::new(CssUtf16ValueStorage::default()),
            failure: Cell::new(None),
            transferred: Cell::new(false),
        }
    }
    pub(crate) fn failure(&self) -> Option<Failure> {
        self.failure.get()
    }
    pub(crate) fn fail<T>(&self, failure: Failure) -> Result<T, ()> {
        if self.failure.get().is_none() {
            self.failure.set(Some(failure));
        }
        Err(())
    }
    pub(crate) fn component_resource(
        &self,
        kind: crate::CssComponentValueErrorKind,
    ) -> Result<(), ()> {
        use crate::CssComponentValueErrorKind as Kind;
        match kind {
            Kind::ByteLimit => self.fail(Failure::Limit(crate::CssUtf16Resource::ComponentBytes)),
            Kind::ComponentLimit => {
                self.fail(Failure::Limit(crate::CssUtf16Resource::ComponentCount))
            }
            Kind::NestingLimit => {
                self.fail(Failure::Limit(crate::CssUtf16Resource::ComponentDepth))
            }
            Kind::CapacityOverflow => self.fail(Failure::CapacityOverflow),
            _ => Ok(()),
        }
    }
    fn observe(&self, action: CssUtf16ValueAdmission) -> Result<(), ()> {
        if self.failure.get().is_some() {
            return Err(());
        }
        let Some(work) = self.work.get().checked_add(1) else {
            return self.fail(Failure::CapacityOverflow);
        };
        if work > self.limits.max_work {
            return self.fail(Failure::Limit(crate::CssUtf16Resource::PropertyWork));
        }
        self.work.set(work);
        if (self.admit)(action).is_err() {
            return self.fail(Failure::Admission);
        }
        Ok(())
    }
    pub(crate) fn step(&self) -> Result<(), ()> {
        self.observe(CssUtf16ValueAdmission::Work)
    }
    pub(crate) fn live(&self) -> CssUtf16ValueStorage {
        self.live.get()
    }
    pub(crate) fn set_live(&self, bytes: usize, utf16_units: usize) -> Result<(), ()> {
        if bytes > self.limits.max_live_bytes {
            return self.fail(Failure::Limit(crate::CssUtf16Resource::PropertyLiveBytes));
        }
        let usage = CssUtf16ValueStorage { bytes, utf16_units };
        self.observe(CssUtf16ValueAdmission::Live(usage))?;
        self.live.set(usage);
        Ok(())
    }
    pub(crate) fn add(&self, bytes: usize) -> Result<(), ()> {
        let Some(total) = self.live.get().bytes.checked_add(bytes) else {
            return self.fail(Failure::CapacityOverflow);
        };
        self.set_live(total, self.live.get().utf16_units)
    }
    /// Called only after the corresponding allocation/custody actually ended.
    /// It never observes cancellation or permits cleanup refusal.
    pub(crate) fn released(&self, bytes: usize) {
        let current = self.live.get();
        let usage = CssUtf16ValueStorage {
            bytes: current.bytes.checked_sub(bytes).expect("owned CSS release"),
            utf16_units: current.utf16_units,
        };
        self.live.set(usage);
        (self.release)(usage);
    }
    pub(crate) fn reconciled(&self, bytes: usize, utf16_units: usize) {
        assert!(
            bytes <= self.live.get().bytes,
            "only ended CSS custody is released"
        );
        let usage = CssUtf16ValueStorage { bytes, utf16_units };
        self.live.set(usage);
        (self.release)(usage);
    }
    pub(crate) fn reserve<T>(&self, values: &mut Vec<T>, additional: usize) -> Result<(), ()> {
        self.step()?;
        let Some(required) = values.len().checked_add(additional) else {
            return self.fail(Failure::CapacityOverflow);
        };
        if required <= values.capacity() {
            return Ok(());
        }
        let Some(old) = values.capacity().checked_mul(size_of::<T>()) else {
            return self.fail(Failure::CapacityOverflow);
        };
        let target = required.max(values.capacity().saturating_mul(2)).max(4);
        let Some(planned) = target.checked_mul(size_of::<T>()) else {
            return self.fail(Failure::CapacityOverflow);
        };
        struct Growth<T> {
            next: Vec<T>,
            previous: std::vec::IntoIter<T>,
        }
        let header = size_of::<Growth<T>>();
        let Some(prospective) = planned.checked_add(header) else {
            return self.fail(Failure::CapacityOverflow);
        };
        // The actual old bank stays live while a distinct new bank is formed.
        // No opaque allocator reallocation hides the simultaneous old/new cost
        // or the cancellable movement of retained elements.
        self.add(prospective)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(target).is_err() {
            self.released(prospective);
            return self.fail(Failure::Allocation);
        }
        let Some(actual) = next.capacity().checked_mul(size_of::<T>()) else {
            return self.fail(Failure::CapacityOverflow);
        };
        if actual > planned && self.add(actual - planned).is_err() {
            drop(next);
            self.released(prospective);
            return Err(());
        }
        let mut growth = Growth {
            next,
            previous: std::mem::take(values).into_iter(),
        };
        while let Some(value) = growth.previous.next() {
            if self.step().is_err() {
                drop(value);
                drop(growth);
                self.released(old + actual + header);
                return Err(());
            }
            growth.next.push(value);
        }
        let Growth { next, previous } = growth;
        drop(previous);
        *values = next;
        self.released(old + header);
        Ok(())
    }
    pub(crate) fn string(&self, value: &str) -> Result<String, ()> {
        self.step()?;
        self.add(value.len())?;
        let mut out = String::new();
        if out.try_reserve_exact(value.len()).is_err() {
            self.released(value.len());
            return self.fail(Failure::Allocation);
        }
        if out.capacity() > value.len() {
            self.add(out.capacity() - value.len())?;
        }
        for point in value.chars() {
            if self.step().is_err() {
                let capacity = out.capacity();
                drop(out);
                self.released(capacity);
                return Err(());
            }
            out.push(point);
        }
        Ok(out)
    }
    pub(crate) fn boxed(&self, value: &str) -> Result<Box<str>, ()> {
        let out = self.string(value)?;
        let capacity = out.capacity();
        // The old String may coexist with the shrink destination allocation.
        if capacity != out.len() {
            self.add(out.len())?;
        }
        let result = out.into_boxed_str();
        if capacity != result.len() {
            self.released(capacity);
        }
        Ok(result)
    }
    pub(crate) fn transfer(&self, storage: CssUtf16ValueStorage) -> Result<(), ()> {
        self.step()?;
        if storage.bytes > self.limits.max_retained_bytes {
            return self.fail(Failure::Limit(
                crate::CssUtf16Resource::PropertyRetainedBytes,
            ));
        }
        assert!(
            storage.bytes <= self.live.get().bytes,
            "retained CSS has live custody"
        );
        self.live.set(storage);
        self.transferred.set(true);
        Ok(())
    }
}
impl Drop for Control<'_> {
    fn drop(&mut self) {
        if !self.transferred.get() {
            // The front declares this owner before its actual payload banks;
            // those banks have dropped before this infallible final release.
            (self.release)(CssUtf16ValueStorage::default());
        }
    }
}

//! Controlled UTF16 property input over the original CSS syntax owners.
use crate::source::{CssUtf16PropertyValueLimits, CssUtf16ValueAdmission, CssUtf16ValueStorage};
use std::cell::Cell;
use std::mem::{size_of, size_of_val};
use std::{cell::RefCell, sync::Arc};

use crate::component::CssComponentValueLimits;
use surgeist_css_parser::limits::{self, Meter, Refused};

/// One complete original UTF16 input and its checked authored color-scheme
/// declaration. Global and substitution-dependent values remain symbolic.
#[derive(Debug)]
pub struct CssColorSchemeParseReport {
    source: crate::source::CssParseSource,
    result: Result<crate::declaration::CssDeclaration, crate::property::CssPropertyValueError>,
    retained: CssUtf16ValueStorage,
}
impl CssColorSchemeParseReport {
    /// Returns the original captured source of the complete UTF16 input.
    #[must_use]
    pub const fn source(&self) -> &crate::source::CssParseSource {
        &self.source
    }
    /// Returns the checked declaration, or the property-value error produced for the input.
    pub const fn result(
        &self,
    ) -> &Result<crate::declaration::CssDeclaration, crate::property::CssPropertyValueError> {
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
    ) -> Result<bool, crate::source::CssRetainedPayloadError<E>> {
        use crate::source::CssRetainedPayloadError;
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
                if !crate::component::values::retained::equivalent_in_same_input_with(
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

impl crate::source::CssRetainedSources for CssColorSchemeParseReport {
    fn visit_retained_sources(
        &self,
        sources: &mut crate::source::CssSourceRetention,
    ) -> Result<(), crate::source::CssSourceRetentionError> {
        // All component and diagnostic origins are issued from this exact
        // snapshot; retaining the complete source also covers empty input.
        sources.source(self.source.source())
    }
}

fn text_equal<E>(
    left: &str,
    right: &str,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, crate::source::CssRetainedPayloadError<E>> {
    use crate::source::CssRetainedPayloadError;
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
    left: &crate::declaration::CssDeclaration,
    right: &crate::declaration::CssDeclaration,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, crate::source::CssRetainedPayloadError<E>> {
    use crate::property::CssKnownDeclaredValueRef as Value;
    use crate::property::CssKnownPropertyValueRef as Property;
    use crate::source::CssRetainedPayloadError;
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
                    (
                        crate::values::color_adjust::CssColorSchemeKeyword::Light,
                        crate::values::color_adjust::CssColorSchemeKeyword::Light,
                    )
                    | (
                        crate::values::color_adjust::CssColorSchemeKeyword::Dark,
                        crate::values::color_adjust::CssColorSchemeKeyword::Dark,
                    ) => {}
                    (
                        crate::values::color_adjust::CssColorSchemeKeyword::Custom(a),
                        crate::values::color_adjust::CssColorSchemeKeyword::Custom(b),
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

impl crate::values::color_adjust::CssColorScheme {
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
    ) -> Result<CssColorSchemeParseReport, crate::source::CssUtf16InputError<E>> {
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
            Err(crate::source::CssUtf16InputError::interrupted(()))
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
                Err(()) => Err(crate::source::CssUtf16InputError::interrupted(())),
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
                use crate::source::CssUtf16InputError as Error;
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
) -> Result<CssColorSchemeParseReport, crate::source::CssUtf16InputError<()>> {
    use crate::source::CssUtf16InputError as Error;
    let refused = |()| Error::interrupted(());
    let baseline = control.live().bytes;
    let source = crate::source::utf16_input::prepare_utf16(
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
    let result = match crate::component::values::parse::bounded::parse_controlled_snapshot(
        &source, syntax, control,
    ) {
        Err(error) => {
            if control.failure().is_some() {
                return Err(Error::interrupted(()));
            }
            control.component_resource(error.kind).map_err(refused)?;
            Err(crate::property::CssPropertyValueError::admission(error))
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
                        .add(crate::declaration::CssDeclaration::constructed_allocation_bytes())
                        .map_err(refused)?;
                    Ok(crate::declaration::CssDeclaration::new_constructed(
                        context,
                        body,
                        crate::declaration::CssImportance::Normal,
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
            let components = crate::component::values::retained::retained_backing_bytes_with(
                declaration.value_components(),
                &mut || control.step(),
            )
            .map_err(|error| payload_failure(error, control))?;
            let body = body_backing(declaration, &mut || control.step())
                .map_err(|error| payload_failure(error, control))?;
            crate::declaration::CssDeclaration::constructed_allocation_bytes()
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
        source: crate::source::CssParseSource::new(source, context),
        result,
        retained: CssUtf16ValueStorage {
            bytes,
            utf16_units: units,
        },
    })
}

fn payload_failure(
    error: crate::source::CssRetainedPayloadError<()>,
    control: &Control<'_>,
) -> crate::source::CssUtf16InputError<()> {
    match error.into_admission() {
        Ok(()) => crate::source::CssUtf16InputError::interrupted(()),
        Err(_) => {
            let _ = control.fail::<()>(Failure::CapacityOverflow);
            crate::source::CssUtf16InputError::capacity_overflow()
        }
    }
}

fn body_backing<E>(
    declaration: &crate::declaration::CssDeclaration,
    admit: &mut impl FnMut() -> Result<(), E>,
) -> Result<usize, crate::source::CssRetainedPayloadError<E>> {
    use crate::property::CssKnownDeclaredValueRef as Value;
    use crate::property::CssKnownPropertyValueRef as Property;
    use crate::source::CssRetainedPayloadError;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Failure {
    Limit(crate::source::CssUtf16Resource),
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
        kind: crate::component::values::CssComponentValueErrorKind,
    ) -> Result<(), ()> {
        use crate::component::values::CssComponentValueErrorKind as Kind;
        match kind {
            Kind::ByteLimit => self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::ComponentBytes,
            )),
            Kind::ComponentLimit => self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::ComponentCount,
            )),
            Kind::NestingLimit => self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::ComponentDepth,
            )),
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
            return self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::PropertyWork,
            ));
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
            return self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::PropertyLiveBytes,
            ));
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
    /// Makes room for `additional` elements under the parser crate's growth model.
    pub(crate) fn reserve<T>(&self, values: &mut Vec<T>, additional: usize) -> Result<(), ()> {
        limits::grow(self, values, additional).map_err(|Refused| ())
    }
    /// Copies `value` into a new `String` of exactly its length under the parser
    /// crate's copy model.
    pub(crate) fn string(&self, value: &str) -> Result<String, ()> {
        limits::copy_str(self, value).map_err(|Refused| ())
    }
    /// Copies `value` into a new `Box<str>` under the parser crate's copy model.
    pub(crate) fn boxed(&self, value: &str) -> Result<Box<str>, ()> {
        limits::copy_boxed(self, value).map_err(|Refused| ())
    }
    pub(crate) fn transfer(&self, storage: CssUtf16ValueStorage) -> Result<(), ()> {
        self.step()?;
        if storage.bytes > self.limits.max_retained_bytes {
            return self.fail(Failure::Limit(
                crate::source::CssUtf16Resource::PropertyRetainedBytes,
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
/// The charged-work and live-storage contract of the parser crate, backed by this
/// admission. The parser crate's growth and copy helpers charge it exactly as
/// `reserve`, `string` and `boxed` charged this control before they moved there.
impl Meter for Control<'_> {
    fn step(&self) -> Result<(), Refused> {
        Control::step(self).map_err(|()| Refused)
    }
    fn add(&self, bytes: usize) -> Result<(), Refused> {
        Control::add(self, bytes).map_err(|()| Refused)
    }
    fn released(&self, bytes: usize) {
        Control::released(self, bytes);
    }
    fn note(&self, failure: limits::Failure) -> Refused {
        let failure = match failure {
            limits::Failure::Allocation => Failure::Allocation,
            limits::Failure::CapacityOverflow => Failure::CapacityOverflow,
            _ => unreachable!("the parser crate reports only allocation and overflow failures"),
        };
        let _ = self.fail::<()>(failure);
        Refused
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

#[cfg(test)]
mod frozen_meter_tests;

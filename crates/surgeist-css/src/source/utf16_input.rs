//! General UTF16 CSS input, CSS Syntax preprocessing and provenance.
use crate::CssParserContext;
use crate::source::CssSourceSnapshot;
use crate::tokenization::bounded::{InputCursor, InputKind, Request};
use std::sync::Arc;

/// Owner-issued complete CSS parse input and its actual parser context.
#[derive(Clone, Debug)]
pub struct CssParseSource {
    source: CssSourceSnapshot,
    context: CssParserContext,
}
impl CssParseSource {
    pub(crate) fn new(source: CssSourceSnapshot, context: CssParserContext) -> Self {
        Self { source, context }
    }
    #[must_use]
    pub const fn source(&self) -> &CssSourceSnapshot {
        &self.source
    }
    #[must_use]
    pub const fn context(&self) -> CssParserContext {
        self.context
    }
}

pub(crate) struct Utf16Source {
    pub(crate) original: Arc<Vec<u16>>,
    pub(crate) map: Vec<(usize, usize)>,
}

/// Independent finite preparation limits. Grammar execution retains its existing
/// resource limits; polling during grammar execution is not provided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUtf16InputLimits {
    /// Logical original UTF16 extent; original spare backing is charged below.
    pub max_input_units: usize,
    /// Logical output bytes and actual normalized String capacity.
    pub max_output_bytes: usize,
    /// Actual normalized-boundary map backing slots.
    pub max_map_entries: usize,
    /// Actual source-position checkpoint backing slots.
    pub max_checkpoints: usize,
    /// Fixed decoder/checkpoint preparation registers; grammar scratch is separate.
    pub max_scratch_bytes: usize,
    /// Aggregate original-unit, text, map and checkpoint backing, excluding
    /// fixed headers, allocator overhead and the existing grammar's scratch.
    pub max_retained_bytes: usize,
    /// Cursor requests/acknowledgements, actual preparation buffer moves,
    /// checkpoint characters and pre/post grammar observations. The initial
    /// interruption poll is not an action.
    pub max_work: usize,
}
impl Default for CssUtf16InputLimits {
    fn default() -> Self {
        Self {
            max_input_units: 16 * 1024 * 1024,
            max_output_bytes: 16 * 1024 * 1024,
            max_map_entries: 16 * 1024 * 1024,
            max_checkpoints: 262_145,
            max_scratch_bytes: 1024,
            max_retained_bytes: 512 * 1024 * 1024,
            max_work: 128 * 1024 * 1024,
        }
    }
}

/// A finite resource of UTF-16 CSS input preparation or property admission that can
/// refuse the input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssUtf16Resource {
    /// [`CssUtf16InputLimits::max_input_units`]: original UTF-16 units.
    InputUnits,
    /// [`CssUtf16InputLimits::max_output_bytes`]: normalized UTF-8 bytes.
    OutputBytes,
    /// [`CssUtf16InputLimits::max_map_entries`]: source map entries.
    MapEntries,
    /// [`CssUtf16InputLimits::max_checkpoints`]: source checkpoints.
    Checkpoints,
    /// [`CssUtf16InputLimits::max_scratch_bytes`]: preparation scratch bytes.
    ScratchBytes,
    /// [`CssUtf16InputLimits::max_retained_bytes`]: retained source bytes.
    RetainedBytes,
    /// [`CssUtf16InputLimits::max_work`]: preparation work.
    Work,
    /// `CssUtf16PropertyValueLimits::max_work`: property input work.
    PropertyWork,
    /// `CssUtf16PropertyValueLimits::max_live_bytes`: property input live bytes.
    PropertyLiveBytes,
    /// `CssUtf16PropertyValueLimits::max_retained_bytes`: property input retained bytes.
    PropertyRetainedBytes,
    /// The property's component byte limit.
    ComponentBytes,
    /// The property's component count limit.
    ComponentCount,
    /// The property's component depth limit.
    ComponentDepth,
}
impl CssUtf16Resource {
    /// Returns the resource's stable description, as error messages name it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputUnits => "original UTF16 units",
            Self::OutputBytes => "normalized UTF8 bytes",
            Self::MapEntries => "source map entries",
            Self::Checkpoints => "source checkpoints",
            Self::ScratchBytes => "preparation scratch bytes",
            Self::RetainedBytes => "retained source bytes",
            Self::Work => "preparation work",
            Self::PropertyWork => "property input work",
            Self::PropertyLiveBytes => "property input live bytes",
            Self::PropertyRetainedBytes => "property input retained bytes",
            Self::ComponentBytes => "property component bytes",
            Self::ComponentCount => "property component count",
            Self::ComponentDepth => "property component depth",
        }
    }
}

/// Atomic refusal of UTF-16 CSS input: no partial source or parse report is returned.
///
/// A consumer's interruption is returned unchanged through [`Self::into_interruption`].
/// Consumers whose admission callbacks prepare UTF-16 input build refusals with
/// [`Self::limit`], [`Self::allocation`], [`Self::capacity_overflow`] and
/// [`Self::interrupted`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUtf16InputError<E> {
    kind: Utf16InputErrorKind<E>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Utf16InputErrorKind<E> {
    Limit(CssUtf16Resource),
    Allocation,
    CapacityOverflow,
    Interrupted(E),
}
impl<E> CssUtf16InputError<E> {
    /// A refusal because `resource` is exhausted.
    #[must_use]
    pub const fn limit(resource: CssUtf16Resource) -> Self {
        Self {
            kind: Utf16InputErrorKind::Limit(resource),
        }
    }
    /// A refusal because storage could not be allocated.
    #[must_use]
    pub const fn allocation() -> Self {
        Self {
            kind: Utf16InputErrorKind::Allocation,
        }
    }
    /// A refusal because a size calculation cannot be represented.
    #[must_use]
    pub const fn capacity_overflow() -> Self {
        Self {
            kind: Utf16InputErrorKind::CapacityOverflow,
        }
    }
    /// The consumer's poll or admission callback interrupted preparation with `error`.
    #[must_use]
    pub const fn interrupted(error: E) -> Self {
        Self {
            kind: Utf16InputErrorKind::Interrupted(error),
        }
    }
    /// Returns the consumer's interruption, or this error for a CSS-owned refusal.
    ///
    /// # Errors
    ///
    /// Returns `self` unchanged when CSS refused the input itself.
    pub fn into_interruption(self) -> Result<E, Self> {
        match self.kind {
            Utf16InputErrorKind::Interrupted(error) => Ok(error),
            kind => Err(Self { kind }),
        }
    }
    /// Returns the consumer's interruption.
    #[must_use]
    pub const fn interruption(&self) -> Option<&E> {
        match &self.kind {
            Utf16InputErrorKind::Interrupted(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the exhausted resource of a limit refusal.
    #[must_use]
    pub const fn exhausted_limit(&self) -> Option<CssUtf16Resource> {
        match self.kind {
            Utf16InputErrorKind::Limit(resource) => Some(resource),
            _ => None,
        }
    }
    /// Returns whether storage could not be allocated.
    #[must_use]
    pub const fn is_allocation_failure(&self) -> bool {
        matches!(self.kind, Utf16InputErrorKind::Allocation)
    }
    /// Returns whether a size calculation cannot be represented.
    #[must_use]
    pub const fn is_capacity_overflow(&self) -> bool {
        matches!(self.kind, Utf16InputErrorKind::CapacityOverflow)
    }
    /// Maps the consumer's interruption, keeping every CSS-owned refusal.
    pub(crate) fn map_interruption<F>(self, map: impl FnOnce(E) -> F) -> CssUtf16InputError<F> {
        CssUtf16InputError {
            kind: match self.kind {
                Utf16InputErrorKind::Limit(resource) => Utf16InputErrorKind::Limit(resource),
                Utf16InputErrorKind::Allocation => Utf16InputErrorKind::Allocation,
                Utf16InputErrorKind::CapacityOverflow => Utf16InputErrorKind::CapacityOverflow,
                Utf16InputErrorKind::Interrupted(error) => {
                    Utf16InputErrorKind::Interrupted(map(error))
                }
            },
        }
    }
}
impl<E: std::fmt::Display> std::fmt::Display for CssUtf16InputError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            Utf16InputErrorKind::Limit(resource) => {
                write!(f, "CSS UTF16 {} limit exceeded", resource.as_str())
            }
            Utf16InputErrorKind::Allocation => f.write_str("CSS UTF16 input allocation failed"),
            Utf16InputErrorKind::CapacityOverflow => {
                f.write_str("CSS UTF16 input capacity overflow")
            }
            Utf16InputErrorKind::Interrupted(error) => {
                write!(f, "CSS UTF16 input interrupted: {error}")
            }
        }
    }
}
impl<E: std::fmt::Debug + std::fmt::Display> std::error::Error for CssUtf16InputError<E> {}

pub(crate) fn check<E>(
    actual: usize,
    maximum: usize,
    resource: CssUtf16Resource,
) -> Result<(), CssUtf16InputError<E>> {
    if actual > maximum {
        Err(CssUtf16InputError::limit(resource))
    } else {
        Ok(())
    }
}

pub(crate) fn parse_utf16<E, T>(
    original: Arc<Vec<u16>>,
    limits: CssUtf16InputLimits,
    poll: impl FnMut() -> Result<(), E>,
    parse: impl FnOnce(&CssSourceSnapshot) -> T,
) -> Result<T, CssUtf16InputError<E>> {
    with_prepared(
        original,
        limits,
        poll,
        |_, _| Ok(()),
        |_, _| {},
        |source, control| {
            control.step()?;
            let report = parse(&source);
            control.step()?;
            Ok(report)
        },
    )
}

/// Reuses the original UTF16 decoder and original source owner. Live/release
/// callbacks describe this preparation alone, including original spare backing,
/// actual fixed payload headers and simultaneous replacement storage.
pub(crate) fn prepare_utf16<E>(
    original: Arc<Vec<u16>>,
    limits: CssUtf16InputLimits,
    poll: impl FnMut() -> Result<(), E>,
    live: impl FnMut(usize, usize) -> Result<(), E>,
    release: impl FnMut(usize, usize),
) -> Result<CssSourceSnapshot, CssUtf16InputError<E>> {
    with_prepared(original, limits, poll, live, release, |source, _| {
        Ok(source)
    })
}

pub(crate) struct Preparation<'a, E> {
    limits: CssUtf16InputLimits,
    poll: &'a mut dyn FnMut() -> Result<(), E>,
    live: &'a mut dyn FnMut(usize, usize) -> Result<(), E>,
    release: &'a mut dyn FnMut(usize, usize),
    work: usize,
    bytes: usize,
    units: usize,
}
impl<E> Preparation<'_, E> {
    pub(crate) fn step(&mut self) -> Result<(), CssUtf16InputError<E>> {
        (self.poll)().map_err(CssUtf16InputError::interrupted)?;
        self.work = self
            .work
            .checked_add(1)
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        check(self.work, self.limits.max_work, CssUtf16Resource::Work)
    }
    fn set_live(&mut self, bytes: usize) -> Result<(), CssUtf16InputError<E>> {
        (self.live)(bytes, self.units).map_err(CssUtf16InputError::interrupted)?;
        self.bytes = bytes;
        Ok(())
    }
    pub(crate) fn add(&mut self, bytes: usize) -> Result<(), CssUtf16InputError<E>> {
        self.set_live(
            self.bytes
                .checked_add(bytes)
                .ok_or_else(CssUtf16InputError::capacity_overflow)?,
        )
    }
    pub(crate) fn released(&mut self, bytes: usize) {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("actual preparation release");
        (self.release)(self.bytes, self.units);
    }
    fn release_to(&mut self, bytes: usize, units: usize) {
        assert!(bytes <= self.bytes);
        self.bytes = bytes;
        self.units = units;
        (self.release)(bytes, units);
    }
    pub(crate) fn grow_vec<T>(
        &mut self,
        values: &mut Vec<T>,
        capacity: usize,
    ) -> Result<(), CssUtf16InputError<E>> {
        if capacity <= values.capacity() {
            return Ok(());
        }
        let old = values
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        let planned = capacity
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        let header = std::mem::size_of::<Vec<T>>() + std::mem::size_of::<std::vec::IntoIter<T>>();
        let prospective = planned
            .checked_add(header)
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        self.add(prospective)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(capacity).is_err() {
            drop(next);
            self.released(prospective);
            return Err(CssUtf16InputError::allocation());
        }
        let actual = next
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        if actual > planned
            && let Err(error) = self.add(actual - planned)
        {
            drop(next);
            self.released(prospective);
            return Err(error);
        }
        let mut previous = std::mem::take(values).into_iter();
        while let Some(value) = previous.next() {
            if let Err(error) = self.step() {
                drop(value);
                drop(previous);
                drop(next);
                self.released(old + actual + header);
                return Err(error);
            }
            next.push(value);
        }
        drop(previous);
        *values = next;
        self.released(old + header);
        Ok(())
    }
    fn grow_string(
        &mut self,
        value: &mut String,
        capacity: usize,
    ) -> Result<(), CssUtf16InputError<E>> {
        if capacity <= value.capacity() {
            return Ok(());
        }
        let header = std::mem::size_of::<String>();
        let prospective = capacity
            .checked_add(header)
            .ok_or_else(CssUtf16InputError::capacity_overflow)?;
        self.add(prospective)?;
        let mut next = String::new();
        if next.try_reserve_exact(capacity).is_err() {
            drop(next);
            self.released(prospective);
            return Err(CssUtf16InputError::allocation());
        }
        let actual = next.capacity();
        if actual > capacity
            && let Err(error) = self.add(actual - capacity)
        {
            drop(next);
            self.released(prospective);
            return Err(error);
        }
        for point in value.chars() {
            if let Err(error) = self.step() {
                drop(next);
                self.released(actual + header);
                return Err(error);
            }
            next.push(point);
        }
        let old = std::mem::replace(value, next);
        let old_capacity = old.capacity();
        drop(old);
        self.released(old_capacity + header);
        Ok(())
    }
}

fn with_prepared<E, T>(
    original: Arc<Vec<u16>>,
    limits: CssUtf16InputLimits,
    mut poll: impl FnMut() -> Result<(), E>,
    mut live: impl FnMut(usize, usize) -> Result<(), E>,
    mut release: impl FnMut(usize, usize),
    finish: impl FnOnce(CssSourceSnapshot, &mut Preparation<'_, E>) -> Result<T, CssUtf16InputError<E>>,
) -> Result<T, CssUtf16InputError<E>> {
    poll().map_err(CssUtf16InputError::interrupted)?;
    let (result, retained) = {
        let mut control = Preparation {
            limits,
            poll: &mut poll,
            live: &mut live,
            release: &mut release,
            work: 0,
            bytes: 0,
            units: original.capacity(),
        };
        let mut retained = (0, 0);
        let result = (|| {
            let source = prepare_owned(original, limits, &mut control)?;
            // Decoder/buffer/checkpoint temporaries have actually left scope;
            // the preparation control itself still lives through finish.
            let bytes = source
                .payload_bytes(true)
                .ok_or_else(CssUtf16InputError::capacity_overflow)?;
            let units = source
                .original_utf16()
                .expect("original UTF16 owner")
                .capacity();
            let live = bytes
                .checked_add(std::mem::size_of::<Preparation<'_, E>>())
                .ok_or_else(CssUtf16InputError::capacity_overflow)?;
            control.release_to(live, units);
            retained = (bytes, units);
            finish(source, &mut control)
        })();
        (result, retained)
    };
    // The control and every failed-call payload have actually left scope.
    if result.is_err() {
        release(0, 0);
    } else {
        release(retained.0, retained.1);
    }
    result
}

fn prepare_owned<E>(
    original: Arc<Vec<u16>>,
    limits: CssUtf16InputLimits,
    control: &mut Preparation<'_, E>,
) -> Result<CssSourceSnapshot, CssUtf16InputError<E>> {
    check(
        original.len(),
        limits.max_input_units,
        CssUtf16Resource::InputUnits,
    )?;
    let original_bytes = original
        .capacity()
        .checked_mul(2)
        .ok_or_else(CssUtf16InputError::capacity_overflow)?;
    check(
        original_bytes,
        limits.max_retained_bytes,
        CssUtf16Resource::RetainedBytes,
    )?;
    check(
        std::mem::size_of::<InputCursor>()
            + std::mem::size_of::<crate::source::CssSourcePosition>(),
        limits.max_scratch_bytes,
        CssUtf16Resource::ScratchBytes,
    )?;
    let mut cursor = InputCursor::new(InputKind::Utf16, original.len());
    let mut text = String::new();
    let mut map = Vec::new();
    let checkpoints = Vec::new();
    let position = crate::source::CssSourcePosition::from_byte_offset_in("", 0);
    let headers = std::mem::size_of::<InputCursor>()
        + std::mem::size_of::<String>()
        + std::mem::size_of::<Vec<(usize, usize)>>()
        + std::mem::size_of::<Vec<crate::source::CssSourcePosition>>()
        + std::mem::size_of::<crate::source::CssSourcePosition>()
        + std::mem::size_of::<Vec<u16>>()
        + std::mem::size_of::<Arc<Vec<u16>>>()
        + std::mem::size_of::<Preparation<'_, E>>();
    control.set_live(
        original_bytes
            .checked_add(headers)
            .ok_or_else(CssUtf16InputError::capacity_overflow)?,
    )?;
    loop {
        control.step()?;
        match cursor.request() {
            Request::Read { index } => cursor.read(original[index]).expect("UTF16 cursor protocol"),
            Request::HeldUnit => cursor.process_held().expect("UTF16 held-unit protocol"),
            Request::Emit(scalar) => {
                let end = text
                    .len()
                    .checked_add(scalar.value.len_utf8())
                    .ok_or_else(CssUtf16InputError::capacity_overflow)?;
                check(end, limits.max_output_bytes, CssUtf16Resource::OutputBytes)?;
                check(
                    map.len()
                        .checked_add(1)
                        .ok_or_else(CssUtf16InputError::capacity_overflow)?,
                    limits.max_map_entries,
                    CssUtf16Resource::MapEntries,
                )?;
                let text_capacity = if end > text.capacity() {
                    text.capacity()
                        .saturating_mul(2)
                        .max(end)
                        .min(limits.max_output_bytes)
                } else {
                    text.capacity()
                };
                let map_capacity = if map.len() == map.capacity() {
                    map.capacity()
                        .saturating_mul(2)
                        .max(1)
                        .min(limits.max_map_entries)
                } else {
                    map.capacity()
                };
                let backing = original_bytes
                    .checked_add(text_capacity)
                    .and_then(|v| {
                        map_capacity
                            .checked_mul(std::mem::size_of::<(usize, usize)>())
                            .and_then(|m| v.checked_add(m))
                    })
                    .ok_or_else(CssUtf16InputError::capacity_overflow)?;
                check(
                    backing,
                    limits.max_retained_bytes,
                    CssUtf16Resource::RetainedBytes,
                )?;
                control.grow_string(&mut text, text_capacity)?;
                control.grow_vec(&mut map, map_capacity)?;
                check(
                    text.capacity(),
                    limits.max_output_bytes,
                    CssUtf16Resource::OutputBytes,
                )?;
                check(
                    map.capacity(),
                    limits.max_map_entries,
                    CssUtf16Resource::MapEntries,
                )?;
                let backing = original_bytes
                    .checked_add(text.capacity())
                    .and_then(|v| {
                        map.capacity()
                            .checked_mul(std::mem::size_of::<(usize, usize)>())
                            .and_then(|m| v.checked_add(m))
                    })
                    .ok_or_else(CssUtf16InputError::capacity_overflow)?;
                check(
                    backing,
                    limits.max_retained_bytes,
                    CssUtf16Resource::RetainedBytes,
                )?;
                map.push((text.len(), scalar.start));
                text.push(scalar.value);
                cursor.emitted().expect("UTF16 emit protocol");
            }
            Request::End => {
                if cursor.end().expect("UTF16 EOF protocol") {
                    break;
                }
            }
        }
    }
    let other_bytes = original_bytes
        .checked_add(
            map.capacity()
                .checked_mul(std::mem::size_of::<(usize, usize)>())
                .ok_or_else(CssUtf16InputError::capacity_overflow)?,
        )
        .ok_or_else(CssUtf16InputError::capacity_overflow)?;
    let source = CssSourceSnapshot::try_from_preprocessed(
        text,
        checkpoints,
        position,
        limits,
        other_bytes,
        control,
    )?
    .attach_utf16(Utf16Source { original, map })
    .ok_or_else(CssUtf16InputError::capacity_overflow)?;
    control.released(
        std::mem::size_of::<Vec<(usize, usize)>>() + std::mem::size_of::<Arc<Vec<u16>>>(),
    );
    check(
        source.storage_usage().retained_bytes(),
        limits.max_retained_bytes,
        CssUtf16Resource::RetainedBytes,
    )?;
    Ok(source)
}

#[cfg(test)]
mod preparation_tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::mem::size_of;
    use std::rc::Rc;

    #[test]
    fn shared_preparation_retains_original_spare_backing_mapping_and_one_source_identity() {
        let mut raw = Vec::with_capacity(64);
        raw.extend([13, 10, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00]);
        let raw = Arc::new(raw);
        let extents = RefCell::new(Vec::new());
        let source = prepare_utf16(
            raw.clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
            |bytes, units| {
                extents.borrow_mut().push((bytes, units));
                Ok(())
            },
            |bytes, units| {
                extents.borrow_mut().push((bytes, units));
            },
        )
        .unwrap();
        assert!(Arc::ptr_eq(source.original_utf16().unwrap(), &raw));
        assert_eq!(source.as_str(), "\n\n�😀�a�");
        assert_eq!(source.preprocessed_scalar_count(), Some(7));
        for (bytes, units) in [
            (0..1, 0..2),
            (1..2, 2..3),
            (2..5, 3..4),
            (5..9, 4..6),
            (9..12, 6..7),
            (12..13, 7..8),
            (13..16, 8..9),
            (16..16, 9..9),
        ] {
            assert_eq!(source.original_utf16_range(bytes), Some(units));
        }
        assert_eq!(
            extents.borrow().last().copied(),
            Some((source.payload_bytes(true).unwrap(), raw.capacity(),))
        );
        assert!(
            extents
                .borrow()
                .iter()
                .all(|(_, units)| *units == raw.capacity())
        );
        let clone = source.clone();
        assert!(clone.same_snapshot(&source));
        drop(source);
        assert!(Arc::ptr_eq(clone.original_utf16().unwrap(), &raw));
        assert_eq!(clone.preprocessed_scalar_count(), Some(7));
    }

    #[test]
    fn actual_vec_replacement_refuses_one_short_before_growth_then_admits_old_new_overlap() {
        let mut values = Vec::with_capacity(2);
        values.extend([11_u64, 22]);
        let old_capacity = values.capacity();
        let baseline = size_of::<Vec<u64>>() + old_capacity * size_of::<u64>();
        // The original Vec/header survives alongside the actual new Vec and
        // old IntoIter during movement. These are independent physical fields.
        let growth_header = size_of::<Vec<u64>>() + size_of::<std::vec::IntoIter<u64>>();
        let requested = old_capacity + 1;
        let peak = baseline + growth_header + requested * size_of::<u64>();
        let allowance = Cell::new(peak - 1);
        let extents = RefCell::new(Vec::new());
        let mut poll = || Ok::<_, &'static str>(());
        let mut live = |bytes, units| {
            extents.borrow_mut().push((bytes, units));
            if bytes > allowance.get() {
                Err("live refusal")
            } else {
                Ok(())
            }
        };
        let mut release = |bytes, units| {
            extents.borrow_mut().push((bytes, units));
        };
        let mut control = Preparation {
            limits: CssUtf16InputLimits::default(),
            poll: &mut poll,
            live: &mut live,
            release: &mut release,
            work: 0,
            bytes: 0,
            units: 0,
        };
        control.set_live(baseline).unwrap();
        let original_pointer = values.as_ptr();
        assert_eq!(
            control.grow_vec(&mut values, requested),
            Err(CssUtf16InputError::interrupted("live refusal"))
        );
        assert_eq!(values.as_ptr(), original_pointer);
        assert_eq!(values.capacity(), old_capacity);
        assert_eq!(values, [11, 22]);
        assert_eq!(control.bytes, baseline);
        allowance.set(peak);
        control.grow_vec(&mut values, requested).unwrap();
        assert_eq!(values, [11, 22]);
        assert_eq!(extents.borrow()[1].0, peak);
        assert_eq!(
            control.bytes,
            size_of::<Vec<u64>>() + values.capacity() * size_of::<u64>()
        );
        drop(values);
        control.release_to(0, 0);
        assert_eq!(extents.borrow().last().copied(), Some((0, 0)));
    }

    #[test]
    fn interrupted_vec_move_drops_both_actual_banks_before_infallible_release() {
        struct Item(Rc<Cell<usize>>);
        impl Drop for Item {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let dropped = Rc::new(Cell::new(0));
        let mut values = Vec::with_capacity(3);
        for _ in 0..3 {
            values.push(Item(dropped.clone()));
        }
        let baseline = size_of::<Vec<Item>>() + values.capacity() * size_of::<Item>();
        let calls = Cell::new(0);
        let mut poll = || {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                Err("movement canceled")
            } else {
                Ok(())
            }
        };
        let mut live = |_, _| Ok(());
        let mut release = |bytes, _| {
            if bytes == size_of::<Vec<Item>>() {
                assert_eq!(
                    dropped.get(),
                    3,
                    "old and replacement items dropped before release"
                );
            }
        };
        let mut control = Preparation {
            limits: CssUtf16InputLimits::default(),
            poll: &mut poll,
            live: &mut live,
            release: &mut release,
            work: 0,
            bytes: 0,
            units: 0,
        };
        control.set_live(baseline).unwrap();
        assert_eq!(
            control.grow_vec(&mut values, 4),
            Err(CssUtf16InputError::interrupted("movement canceled"))
        );
        assert!(values.is_empty());
        assert_eq!(values.capacity(), 0);
        assert_eq!(control.bytes, size_of::<Vec<Item>>());
        assert_eq!(dropped.get(), 3);
    }

    #[test]
    fn preparation_poll_and_live_refusals_drop_original_custody_then_release_zero() {
        let raw = Arc::new("a".repeat(130).encode_utf16().collect::<Vec<_>>());
        let calls = Cell::new(0);
        drop(
            prepare_utf16(
                raw.clone(),
                CssUtf16InputLimits::default(),
                || {
                    calls.set(calls.get() + 1);
                    Ok::<_, usize>(())
                },
                |_, _| Ok(()),
                |_, _| {},
            )
            .unwrap(),
        );
        for stop in [2, calls.get() / 2, calls.get()] {
            let polled = Cell::new(0);
            let released = Cell::new(None);
            let result = prepare_utf16(
                raw.clone(),
                CssUtf16InputLimits::default(),
                || {
                    polled.set(polled.get() + 1);
                    if polled.get() == stop {
                        Err(stop)
                    } else {
                        Ok(())
                    }
                },
                |_, _| Ok(()),
                |bytes, units| {
                    if bytes == 0 {
                        assert_eq!(Arc::strong_count(&raw), 1);
                    }
                    released.set(Some((bytes, units)));
                },
            );
            assert!(matches!(result, Err(ref error) if error.interruption() == Some(&stop)));
            assert_eq!(polled.get(), stop);
            assert_eq!(released.get(), Some((0, 0)));
            assert_eq!(Arc::strong_count(&raw), 1);
        }
        let observations = Cell::new(0);
        let released = Cell::new(None);
        let result = prepare_utf16(
            raw.clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
            |_, _| {
                observations.set(observations.get() + 1);
                if observations.get() == 2 {
                    Err(())
                } else {
                    Ok(())
                }
            },
            |bytes, units| {
                if bytes == 0 {
                    assert_eq!(Arc::strong_count(&raw), 1);
                }
                released.set(Some((bytes, units)));
            },
        );
        assert!(matches!(result, Err(error) if error.interruption() == Some(&())));
        assert_eq!(observations.get(), 2);
        assert_eq!(released.get(), Some((0, 0)));
    }
}

//! Section 3.3 over UTF-16 units: preprocessing a whole input into normalized
//! text and the map that keeps every scalar's original units.

use std::fmt;
use std::mem::size_of;
use std::ops::Range;

use crate::limits::{Failure, Meter};

use super::cursor::{InputCursor, InputKind, InputRequest};

/// The size limits and the work limit of [`preprocess_utf16`].
///
/// Each limit is independent. The defaults are finite and generous: sixteen
/// mebi-units of input, output and map entries, 1024 bytes of scratch, half a
/// gibibyte retained and 128 mebi-steps of work.
///
/// ```
/// use surgeist_css_parser::input::PreprocessLimits;
///
/// let limits = PreprocessLimits::default().with_max_output_bytes(64);
/// assert_eq!(limits.max_output_bytes(), 64);
/// assert_eq!(limits.max_input_units(), 16 * 1024 * 1024);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PreprocessLimits {
    input_units: usize,
    output_bytes: usize,
    map_entries: usize,
    scratch_bytes: usize,
    retained_bytes: usize,
    work: usize,
}

impl PreprocessLimits {
    /// The bytes of fixed registers that [`preprocess_utf16`] holds while it
    /// runs and checks against [`max_scratch_bytes`](Self::max_scratch_bytes):
    /// the size of the [`InputCursor`](super::InputCursor).
    pub const SCRATCH_BYTES: usize = size_of::<InputCursor>();

    /// Returns the greatest number of input units.
    #[must_use]
    pub const fn max_input_units(self) -> usize {
        self.input_units
    }

    /// Returns the greatest number of normalized UTF-8 bytes, and the greatest
    /// capacity of the normalized text.
    #[must_use]
    pub const fn max_output_bytes(self) -> usize {
        self.output_bytes
    }

    /// Returns the greatest number of source-map entries, and the greatest
    /// capacity of the source map, which has one entry per scalar.
    #[must_use]
    pub const fn max_map_entries(self) -> usize {
        self.map_entries
    }

    /// Returns the greatest number of bytes of fixed scratch registers.
    #[must_use]
    pub const fn max_scratch_bytes(self) -> usize {
        self.scratch_bytes
    }

    /// Returns the greatest number of bytes the result may retain: the
    /// capacity of the normalized text plus that of the source map (a host
    /// that also retains the input subtracts it from its own limit).
    #[must_use]
    pub const fn max_retained_bytes(self) -> usize {
        self.retained_bytes
    }

    /// Returns the greatest number of steps of work: cursor requests,
    /// acknowledgements, and the elements and scalars moved when storage
    /// grows.
    #[must_use]
    pub const fn max_work(self) -> usize {
        self.work
    }

    /// Returns these limits with another input limit.
    #[must_use]
    pub const fn with_max_input_units(mut self, units: usize) -> Self {
        self.input_units = units;
        self
    }

    /// Returns these limits with another output limit.
    #[must_use]
    pub const fn with_max_output_bytes(mut self, bytes: usize) -> Self {
        self.output_bytes = bytes;
        self
    }

    /// Returns these limits with another source-map limit.
    #[must_use]
    pub const fn with_max_map_entries(mut self, entries: usize) -> Self {
        self.map_entries = entries;
        self
    }

    /// Returns these limits with another scratch limit.
    #[must_use]
    pub const fn with_max_scratch_bytes(mut self, bytes: usize) -> Self {
        self.scratch_bytes = bytes;
        self
    }

    /// Returns these limits with another retained-bytes limit.
    #[must_use]
    pub const fn with_max_retained_bytes(mut self, bytes: usize) -> Self {
        self.retained_bytes = bytes;
        self
    }

    /// Returns these limits with another work limit.
    #[must_use]
    pub const fn with_max_work(mut self, steps: usize) -> Self {
        self.work = steps;
        self
    }
}

impl Default for PreprocessLimits {
    fn default() -> Self {
        Self {
            input_units: 16 * 1024 * 1024,
            output_bytes: 16 * 1024 * 1024,
            map_entries: 16 * 1024 * 1024,
            scratch_bytes: 1024,
            retained_bytes: 512 * 1024 * 1024,
            work: 128 * 1024 * 1024,
        }
    }
}

/// The limit of [`PreprocessLimits`] that a preprocessing exceeded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PreprocessResource {
    /// [`PreprocessLimits::max_input_units`]: the UTF-16 units of the input.
    InputUnits,
    /// [`PreprocessLimits::max_output_bytes`]: the normalized UTF-8 bytes.
    OutputBytes,
    /// [`PreprocessLimits::max_map_entries`]: the source-map entries.
    MapEntries,
    /// [`PreprocessLimits::max_scratch_bytes`]: the fixed scratch registers.
    ScratchBytes,
    /// [`PreprocessLimits::max_retained_bytes`]: the retained text and map.
    RetainedBytes,
    /// [`PreprocessLimits::max_work`]: the steps of work.
    Work,
}

impl PreprocessResource {
    /// Returns the resource's stable description, as error messages name it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputUnits => "input UTF-16 units",
            Self::OutputBytes => "normalized UTF-8 bytes",
            Self::MapEntries => "source map entries",
            Self::ScratchBytes => "preprocessing scratch bytes",
            Self::RetainedBytes => "retained text and map bytes",
            Self::Work => "preprocessing work",
        }
    }
}

/// A preprocessing failed as a whole: no text and no map are returned, and
/// everything it charged to the [`Meter`] has been released.
///
/// Ask what happened with the `is_*` predicates; the cause is private so that
/// a new one is not a breaking change.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PreprocessError {
    kind: PreprocessErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PreprocessErrorKind {
    Limit(PreprocessResource),
    Allocation,
    CapacityOverflow,
    Refused,
}

impl PreprocessError {
    const fn new(kind: PreprocessErrorKind) -> Self {
        Self { kind }
    }

    kind_predicates! { kind;
        /// Whether a limit of [`PreprocessLimits`] was exceeded; see
        /// [`exhausted_limit`](Self::exhausted_limit).
        is_limit => PreprocessErrorKind::Limit(_),
        /// Whether the allocator could not provide storage. The meter has been
        /// told with [`Failure::Allocation`].
        is_allocation_failure => PreprocessErrorKind::Allocation,
        /// Whether a size cannot be represented. The meter has been told with
        /// [`Failure::CapacityOverflow`].
        is_capacity_overflow => PreprocessErrorKind::CapacityOverflow,
        /// Whether the [`Meter`] refused an action or storage; the host holds
        /// the reason.
        is_refused => PreprocessErrorKind::Refused,
    }

    /// Returns the limit that was exceeded, for a limit failure.
    #[must_use]
    pub const fn exhausted_limit(&self) -> Option<PreprocessResource> {
        match self.kind {
            PreprocessErrorKind::Limit(resource) => Some(resource),
            _ => None,
        }
    }
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            PreprocessErrorKind::Limit(resource) => {
                write!(
                    formatter,
                    "UTF-16 preprocessing {} limit exceeded",
                    resource.as_str()
                )
            }
            PreprocessErrorKind::Allocation => {
                formatter.write_str("UTF-16 preprocessing allocation failed")
            }
            PreprocessErrorKind::CapacityOverflow => {
                formatter.write_str("UTF-16 preprocessing capacity overflow")
            }
            PreprocessErrorKind::Refused => {
                formatter.write_str("UTF-16 preprocessing was refused by the meter")
            }
        }
    }
}

impl std::error::Error for PreprocessError {}

/// The end of one scalar, in the normalized text and in the original units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Boundary {
    normalized: usize,
    original: usize,
}

/// Where each scalar of preprocessed text came from in the original UTF-16
/// units.
///
/// The map has one entry per scalar (16 bytes each), so any scalar boundary of
/// the normalized text maps back exactly. Positions in the normalized text are
/// byte offsets; positions in the original are UTF-16 unit indexes. It holds
/// nothing else, so it is no larger than the vector of its entries: each entry
/// records where its scalar ends, which makes the start of the text, the length
/// of the text and the number of original units the only values it does not
/// store (the first is always 0, the others are the last entry's).
///
/// ```
/// use surgeist_css_parser::input::{preprocess_utf16, PreprocessLimits};
/// use surgeist_css_parser::limits::{Failure, Meter, Refused};
///
/// struct Free;
/// impl Meter for Free {
///     fn step(&self) -> Result<(), Refused> { Ok(()) }
///     fn add(&self, _: usize) -> Result<(), Refused> { Ok(()) }
///     fn released(&self, _: usize) {}
///     fn note(&self, _: Failure) -> Refused { Refused }
/// }
///
/// // "a", CR LF, "b": the pair is one LF made of two original units.
/// let units: Vec<u16> = "a\r\nb".encode_utf16().collect();
/// let result = preprocess_utf16(&units, &PreprocessLimits::default(), &Free)?;
/// assert_eq!(result.text(), "a\nb");
/// assert_eq!(result.map().original_utf16(1..2), Some(1..3));
/// # Ok::<(), surgeist_css_parser::input::PreprocessError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMap {
    boundaries: Vec<Boundary>,
}

impl SourceMap {
    /// Maps a half-open range of the normalized text to the half-open range of
    /// original UTF-16 units that produced it.
    ///
    /// Both ends must be scalar boundaries of the normalized text, which
    /// includes its length. The range of one scalar covers every original unit
    /// the scalar replaced: both units of a surrogate pair and both of a CR LF
    /// pair. Returns `None` for a range that ends before it starts, ends
    /// outside the text or has an end inside a scalar.
    #[must_use]
    pub fn original_utf16(&self, normalized: Range<usize>) -> Option<Range<usize>> {
        if normalized.start > normalized.end {
            return None;
        }
        let boundary = |offset: usize| {
            if offset == 0 {
                return Some(0);
            }
            self.boundaries
                .binary_search_by_key(&offset, |entry| entry.normalized)
                .ok()
                .map(|index| self.boundaries[index].original)
        };
        Some(boundary(normalized.start)?..boundary(normalized.end)?)
    }

    /// Returns the number of scalars in the normalized text.
    #[must_use]
    pub fn scalar_count(&self) -> usize {
        self.boundaries.len()
    }

    /// Returns the number of UTF-16 units in the original input.
    #[must_use]
    pub fn original_units(&self) -> usize {
        self.boundaries.last().map_or(0, |entry| entry.original)
    }

    /// Returns the number of entries the map has room for without growing.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.boundaries.capacity()
    }

    /// Returns the bytes of the entries' storage, which is the capacity times
    /// the 16 bytes of an entry. This is the map's share of
    /// [`Preprocessed::charged_bytes`], without the header of the map's vector.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        self.boundaries.capacity() * size_of::<Boundary>()
    }
}

/// UTF-16 input after section 3.3: normalized UTF-8 text and the map back to
/// the original units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Preprocessed {
    text: String,
    map: SourceMap,
}

impl Preprocessed {
    /// Returns the normalized text: a CR, an FF and a CR LF pair are each one
    /// LF, and a NUL and every unpaired surrogate unit are U+FFFD.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the map from the normalized text to the original units.
    #[must_use]
    pub const fn map(&self) -> &SourceMap {
        &self.map
    }

    /// Splits the result into its normalized text and its map.
    #[must_use]
    pub fn into_parts(self) -> (String, SourceMap) {
        (self.text, self.map)
    }

    /// Returns the capacity of the normalized text in bytes.
    #[must_use]
    pub fn text_capacity(&self) -> usize {
        self.text.capacity()
    }

    /// Returns the bytes this value holds charged to the [`Meter`] that
    /// [`preprocess_utf16`] ran against: the capacity of the text and of the
    /// map, and one header each. A host that drops the value returns exactly
    /// this amount.
    #[must_use]
    pub fn charged_bytes(&self) -> usize {
        self.text.capacity()
            + size_of::<String>()
            + self.map.capacity_bytes()
            + size_of::<Vec<Boundary>>()
    }
}

/// Preprocesses UTF-16 `units` as section 3.3 says, charging `meter`.
///
/// Each scalar is read from the units (a surrogate pair is one scalar, an
/// unpaired surrogate unit is U+FFFD), filtered (CR, FF and CR LF become LF, NUL
/// becomes U+FFFD), appended to the normalized text and entered in the
/// [`SourceMap`]. The work, the storage and every growth are charged to the
/// meter as they happen, and each limit of `limits` is checked before the
/// storage that would exceed it exists.
///
/// # Charging
///
/// The sequence of meter calls is:
///
/// 1. [`add`](Meter::add) the bytes of the fixed registers: the cursor, and
///    the headers of the text and the map.
/// 2. For each cursor request, one [`step`](Meter::step). When a scalar needs
///    room, before it is stored, the text grows and then the map. A growth
///    takes no step of its own; it charges the new bank beside the old one while
///    the old contents move: an [`add`](Meter::add) of the new bank and a
///    header, an [`add`](Meter::add) of any capacity the allocator gave beyond
///    the plan, one [`step`](Meter::step) per element moved (per scalar for the
///    text, per entry for the map), and a [`released`](Meter::released) of the
///    old bank and header. The text grows to its required length or twice its
///    capacity, whichever is greater, and the map to twice its capacity or one
///    entry, each no further than its limit.
/// 3. [`released`](Meter::released) the bytes of the cursor.
///
/// This is the growth model of the preparation loop this function was moved
/// from, not that of [`grow`](crate::limits::grow): that helper takes a step
/// when it starts and never plans a bank under four elements, so charging
/// with it would change the work and bytes that a host compares.
///
/// On success the meter holds [`Preprocessed::charged_bytes`]; the host
/// releases that amount when it drops the result. On failure everything has
/// been released.
///
/// # Errors
///
/// Returns [`PreprocessError`] when a limit is exceeded, the meter refuses, a
/// size cannot be represented or the allocator fails. No text or map is
/// returned.
///
/// ```
/// use surgeist_css_parser::input::{preprocess_utf16, PreprocessLimits};
/// use surgeist_css_parser::limits::{Failure, Meter, Refused};
///
/// struct Free;
/// impl Meter for Free {
///     fn step(&self) -> Result<(), Refused> { Ok(()) }
///     fn add(&self, _: usize) -> Result<(), Refused> { Ok(()) }
///     fn released(&self, _: usize) {}
///     fn note(&self, _: Failure) -> Refused { Refused }
/// }
///
/// // A lone high surrogate, a NUL and a form feed.
/// let result = preprocess_utf16(&[0xd800, 0, 0x0c], &PreprocessLimits::default(), &Free)?;
/// assert_eq!(result.text(), "\u{fffd}\u{fffd}\n");
/// # Ok::<(), surgeist_css_parser::input::PreprocessError>(())
/// ```
pub fn preprocess_utf16(
    units: &[u16],
    limits: &PreprocessLimits,
    meter: &dyn Meter,
) -> Result<Preprocessed, PreprocessError> {
    let mut run = Run {
        meter,
        limits,
        work: 0,
        bytes: 0,
    };
    match run.preprocess(units) {
        Ok(result) => Ok(result),
        Err(error) => {
            // The text and the map have dropped; return what remains charged.
            run.release_all();
            Err(error)
        }
    }
}

/// One preprocessing: the meter, the limits, the work done and the live bytes
/// charged so far.
struct Run<'a> {
    meter: &'a dyn Meter,
    limits: &'a PreprocessLimits,
    work: usize,
    bytes: usize,
}

impl Run<'_> {
    fn limit(resource: PreprocessResource) -> PreprocessError {
        PreprocessError::new(PreprocessErrorKind::Limit(resource))
    }

    fn overflow(&self) -> PreprocessError {
        let _ = self.meter.note(Failure::CapacityOverflow);
        PreprocessError::new(PreprocessErrorKind::CapacityOverflow)
    }

    fn allocation(&self) -> PreprocessError {
        let _ = self.meter.note(Failure::Allocation);
        PreprocessError::new(PreprocessErrorKind::Allocation)
    }

    fn refused() -> PreprocessError {
        PreprocessError::new(PreprocessErrorKind::Refused)
    }

    fn check(
        actual: usize,
        maximum: usize,
        resource: PreprocessResource,
    ) -> Result<(), PreprocessError> {
        if actual > maximum {
            Err(Self::limit(resource))
        } else {
            Ok(())
        }
    }

    /// One admitted action, which is also one step of work.
    fn step(&mut self) -> Result<(), PreprocessError> {
        self.meter.step().map_err(|_| Self::refused())?;
        let Some(work) = self.work.checked_add(1) else {
            return Err(self.overflow());
        };
        self.work = work;
        Self::check(work, self.limits.work, PreprocessResource::Work)
    }

    /// Charges `bytes` more live bytes.
    fn add(&mut self, bytes: usize) -> Result<(), PreprocessError> {
        let Some(total) = self.bytes.checked_add(bytes) else {
            return Err(self.overflow());
        };
        self.meter.add(bytes).map_err(|_| Self::refused())?;
        self.bytes = total;
        Ok(())
    }

    /// Returns `bytes` of live bytes that have ended.
    fn released(&mut self, bytes: usize) {
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .expect("the preprocessing releases only what it charged");
        self.meter.released(bytes);
    }

    /// Returns everything still charged, on a failure.
    fn release_all(&mut self) {
        if self.bytes > 0 {
            self.meter.released(self.bytes);
            self.bytes = 0;
        }
    }

    /// Grows `values` to room for exactly `capacity` elements, if it has less:
    /// the new bank is charged beside the old one, the elements move one step
    /// each, and the old bank is returned.
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, capacity: usize) -> Result<(), PreprocessError> {
        if capacity <= values.capacity() {
            return Ok(());
        }
        let Some(old) = values.capacity().checked_mul(size_of::<T>()) else {
            return Err(self.overflow());
        };
        let Some(planned) = capacity.checked_mul(size_of::<T>()) else {
            return Err(self.overflow());
        };
        let header = size_of::<Vec<T>>() + size_of::<std::vec::IntoIter<T>>();
        let Some(prospective) = planned.checked_add(header) else {
            return Err(self.overflow());
        };
        self.add(prospective)?;
        let mut next = Vec::new();
        if next.try_reserve_exact(capacity).is_err() {
            drop(next);
            self.released(prospective);
            return Err(self.allocation());
        }
        let Some(actual) = next.capacity().checked_mul(size_of::<T>()) else {
            drop(next);
            self.released(prospective);
            return Err(self.overflow());
        };
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

    /// Grows `value` to room for exactly `capacity` bytes, if it has less,
    /// the same way, moving one step per scalar.
    fn grow_string(&mut self, value: &mut String, capacity: usize) -> Result<(), PreprocessError> {
        if capacity <= value.capacity() {
            return Ok(());
        }
        let header = size_of::<String>();
        let Some(prospective) = capacity.checked_add(header) else {
            return Err(self.overflow());
        };
        self.add(prospective)?;
        let mut next = String::new();
        if next.try_reserve_exact(capacity).is_err() {
            drop(next);
            self.released(prospective);
            return Err(self.allocation());
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

    fn preprocess(&mut self, units: &[u16]) -> Result<Preprocessed, PreprocessError> {
        let limits = *self.limits;
        Self::check(
            units.len(),
            limits.input_units,
            PreprocessResource::InputUnits,
        )?;
        Self::check(
            PreprocessLimits::SCRATCH_BYTES,
            limits.scratch_bytes,
            PreprocessResource::ScratchBytes,
        )?;
        let mut cursor = InputCursor::new(InputKind::Utf16, units.len());
        let mut text = String::new();
        let mut map: Vec<Boundary> = Vec::new();
        self.add(size_of::<InputCursor>() + size_of::<String>() + size_of::<Vec<Boundary>>())?;
        loop {
            self.step()?;
            match cursor.request() {
                InputRequest::Read { index } => cursor
                    .read(units[index])
                    .expect("a UTF-16 cursor accepts every unit"),
                InputRequest::HeldUnit => cursor
                    .process_held()
                    .expect("a held unit is processed once"),
                InputRequest::Emit(scalar) => {
                    let Some(end) = text.len().checked_add(scalar.value.len_utf8()) else {
                        return Err(self.overflow());
                    };
                    Self::check(end, limits.output_bytes, PreprocessResource::OutputBytes)?;
                    let Some(entries) = map.len().checked_add(1) else {
                        return Err(self.overflow());
                    };
                    Self::check(entries, limits.map_entries, PreprocessResource::MapEntries)?;
                    let text_capacity = if end > text.capacity() {
                        text.capacity()
                            .saturating_mul(2)
                            .max(end)
                            .min(limits.output_bytes)
                    } else {
                        text.capacity()
                    };
                    let map_capacity = if map.len() == map.capacity() {
                        map.capacity()
                            .saturating_mul(2)
                            .max(1)
                            .min(limits.map_entries)
                    } else {
                        map.capacity()
                    };
                    self.check_retained(text_capacity, map_capacity)?;
                    self.grow_string(&mut text, text_capacity)?;
                    self.grow_vec(&mut map, map_capacity)?;
                    Self::check(
                        text.capacity(),
                        limits.output_bytes,
                        PreprocessResource::OutputBytes,
                    )?;
                    Self::check(
                        map.capacity(),
                        limits.map_entries,
                        PreprocessResource::MapEntries,
                    )?;
                    self.check_retained(text.capacity(), map.capacity())?;
                    text.push(scalar.value);
                    map.push(Boundary {
                        normalized: text.len(),
                        original: scalar.end,
                    });
                    cursor
                        .emitted()
                        .expect("the emitted scalar was the pending request");
                }
                InputRequest::End => {
                    if cursor.end().expect("the end was the pending request") {
                        break;
                    }
                }
            }
        }
        self.released(size_of::<InputCursor>());
        Ok(Preprocessed {
            text,
            map: SourceMap { boundaries: map },
        })
    }

    /// Checks the bytes that a text and a map of these capacities retain.
    fn check_retained(
        &self,
        text_capacity: usize,
        map_capacity: usize,
    ) -> Result<(), PreprocessError> {
        let backing = map_capacity
            .checked_mul(size_of::<Boundary>())
            .and_then(|map| map.checked_add(text_capacity));
        let Some(backing) = backing else {
            return Err(self.overflow());
        };
        Self::check(
            backing,
            self.limits.retained_bytes,
            PreprocessResource::RetainedBytes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::Refused;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// What a ledger runs after a release, with the live total.
    type ReleaseCheck = Box<dyn Fn(usize)>;

    /// A meter that tracks the live total, records it after every change and
    /// notes, and refuses an `add` that would take the total past a ceiling or
    /// a `step` from a chosen call on.
    struct Ledger {
        live: Cell<usize>,
        ceiling: Cell<usize>,
        levels: RefCell<Vec<usize>>,
        notes: RefCell<Vec<Failure>>,
        steps: Cell<usize>,
        refuse_step: Cell<usize>,
        on_release: RefCell<Option<ReleaseCheck>>,
    }

    impl Ledger {
        fn new() -> Self {
            Self {
                live: Cell::new(0),
                ceiling: Cell::new(usize::MAX),
                levels: RefCell::new(Vec::new()),
                notes: RefCell::new(Vec::new()),
                steps: Cell::new(0),
                refuse_step: Cell::new(usize::MAX),
                on_release: RefCell::new(None),
            }
        }
    }

    impl Meter for Ledger {
        fn step(&self) -> Result<(), Refused> {
            self.steps.set(self.steps.get() + 1);
            if self.steps.get() >= self.refuse_step.get() {
                Err(Refused)
            } else {
                Ok(())
            }
        }

        fn add(&self, bytes: usize) -> Result<(), Refused> {
            let total = self.live.get() + bytes;
            self.levels.borrow_mut().push(total);
            if total > self.ceiling.get() {
                return Err(Refused);
            }
            self.live.set(total);
            Ok(())
        }

        fn released(&self, bytes: usize) {
            self.live.set(self.live.get() - bytes);
            self.levels.borrow_mut().push(self.live.get());
            if let Some(check) = &*self.on_release.borrow() {
                check(self.live.get());
            }
        }

        fn note(&self, failure: Failure) -> Refused {
            self.notes.borrow_mut().push(failure);
            Refused
        }
    }

    fn run<'a>(meter: &'a Ledger, limits: &'a PreprocessLimits) -> Run<'a> {
        Run {
            meter,
            limits,
            work: 0,
            bytes: 0,
        }
    }

    #[test]
    fn a_vector_replacement_refuses_one_short_before_growth_then_admits_old_new_overlap() {
        let limits = PreprocessLimits::default();
        let mut values = Vec::with_capacity(2);
        values.extend([11_u64, 22]);
        let old_capacity = values.capacity();
        let baseline = size_of::<Vec<u64>>() + old_capacity * size_of::<u64>();
        // The original Vec survives alongside the new Vec and the old IntoIter
        // while the elements move: independent physical fields.
        let growth_header = size_of::<Vec<u64>>() + size_of::<std::vec::IntoIter<u64>>();
        let requested = old_capacity + 1;
        let peak = baseline + growth_header + requested * size_of::<u64>();
        let meter = Ledger::new();
        meter.ceiling.set(peak - 1);
        let mut run = run(&meter, &limits);
        run.add(baseline).unwrap();
        let pointer = values.as_ptr();
        assert!(
            run.grow_vec(&mut values, requested)
                .unwrap_err()
                .is_refused()
        );
        assert_eq!(values.as_ptr(), pointer);
        assert_eq!(values.capacity(), old_capacity);
        assert_eq!(values, [11, 22]);
        assert_eq!(run.bytes, baseline);
        meter.ceiling.set(peak);
        run.grow_vec(&mut values, requested).unwrap();
        assert_eq!(values, [11, 22]);
        // The refused attempt asked for the peak; the admitted one reached it.
        assert_eq!(meter.levels.borrow()[1], peak);
        assert_eq!(
            run.bytes,
            size_of::<Vec<u64>>() + values.capacity() * size_of::<u64>()
        );
        drop(values);
        run.release_all();
        assert_eq!(meter.live.get(), 0);
    }

    #[test]
    fn an_interrupted_vector_move_drops_both_banks_before_the_infallible_release() {
        struct Item(Rc<Cell<usize>>);
        impl Drop for Item {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }
        let limits = PreprocessLimits::default();
        let dropped = Rc::new(Cell::new(0));
        let mut values = Vec::with_capacity(3);
        for _ in 0..3 {
            values.push(Item(dropped.clone()));
        }
        let baseline = size_of::<Vec<Item>>() + values.capacity() * size_of::<Item>();
        let meter = Ledger::new();
        // The second step of the movement is refused.
        meter.refuse_step.set(2);
        let seen = dropped.clone();
        *meter.on_release.borrow_mut() = Some(Box::new(move |live| {
            if live == size_of::<Vec<Item>>() {
                assert_eq!(
                    seen.get(),
                    3,
                    "old and new items dropped before the release"
                );
            }
        }));
        let mut run = run(&meter, &limits);
        run.add(baseline).unwrap();
        assert!(run.grow_vec(&mut values, 4).unwrap_err().is_refused());
        assert!(values.is_empty());
        assert_eq!(values.capacity(), 0);
        assert_eq!(run.bytes, size_of::<Vec<Item>>());
        assert_eq!(dropped.get(), 3);
    }

    #[test]
    fn an_interrupted_string_move_drops_the_new_bank_and_keeps_the_old_one_charged() {
        let limits = PreprocessLimits::default();
        let meter = Ledger::new();
        let mut run = run(&meter, &limits);
        let mut text = String::new();
        run.grow_string(&mut text, 4).unwrap();
        text.push_str("abcd");
        let before = run.bytes;
        // The first step of the movement is refused, so the new bank goes and
        // the old one, still owned by the caller, stays charged.
        meter.refuse_step.set(meter.steps.get() + 1);
        assert!(run.grow_string(&mut text, 8).unwrap_err().is_refused());
        assert_eq!(text, "abcd");
        assert_eq!(run.bytes, before);
        assert_eq!(meter.live.get(), before);
    }

    #[test]
    fn unrepresentable_sizes_are_reported_to_the_meter_and_as_overflow() {
        let limits = PreprocessLimits::default();
        let meter = Ledger::new();
        let mut run = run(&meter, &limits);
        // The work counter cannot count one more step.
        run.work = usize::MAX;
        let error = run.step().unwrap_err();
        assert!(error.is_capacity_overflow());
        assert_eq!(error.to_string(), "UTF-16 preprocessing capacity overflow");
        run.work = 0;
        // The live total cannot take one more byte, and nothing is asked of
        // the meter.
        run.bytes = usize::MAX;
        let levels = meter.levels.borrow().len();
        assert!(run.add(1).unwrap_err().is_capacity_overflow());
        assert_eq!(meter.levels.borrow().len(), levels);
        run.bytes = 0;
        // The retained backing of a text and a map overflows usize.
        assert!(
            run.check_retained(usize::MAX, 2)
                .unwrap_err()
                .is_capacity_overflow()
        );
        assert!(
            run.check_retained(1, usize::MAX)
                .unwrap_err()
                .is_capacity_overflow()
        );
        // A planned bank larger than the address space.
        let mut values: Vec<u64> = Vec::new();
        assert!(
            run.grow_vec(&mut values, usize::MAX)
                .unwrap_err()
                .is_capacity_overflow()
        );
        assert!(values.is_empty());
        let mut text = String::new();
        assert!(
            run.grow_string(&mut text, usize::MAX)
                .unwrap_err()
                .is_capacity_overflow()
        );
        assert_eq!(
            *meter.notes.borrow(),
            [Failure::CapacityOverflow; 6],
            "every overflow told the meter"
        );
    }

    #[test]
    fn an_allocation_that_cannot_be_made_is_reported_as_an_allocation_failure() {
        // A bank of isize::MAX bytes passes the size arithmetic and is refused
        // by the allocator's capacity check without any allocation.
        let limits = PreprocessLimits::default();
        let meter = Ledger::new();
        let mut run = run(&meter, &limits);
        let mut text = String::new();
        let error = run.grow_string(&mut text, isize::MAX as usize).unwrap_err();
        assert!(error.is_allocation_failure());
        assert_eq!(error.to_string(), "UTF-16 preprocessing allocation failed");
        assert_eq!(*meter.notes.borrow(), [Failure::Allocation]);
        // The charge for the bank that never existed has been returned.
        assert_eq!(run.bytes, 0);
        assert_eq!(meter.live.get(), 0);
    }
}

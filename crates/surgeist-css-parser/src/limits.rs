//! Build limits, the metering contract, and the helpers that charge it.
//!
//! A [`Limits`] value bounds what a build may produce. A [`Meter`] is how a
//! host admits each unit of work and each byte of live storage a build asks
//! for: the crate reports what it is about to do and the host decides whether
//! to allow it. [`grow`], [`copy_str`] and [`copy_boxed`] are the crate's
//! charged allocations. They define one growth and copy model, with the
//! sequence of meter calls spelled out below, so every consumer of the crate
//! charges storage the same way.
//!
//! # Charging model
//!
//! - One *step* is one admitted action. Each helper takes a step when it
//!   starts, one step for every element or scalar value it moves or copies,
//!   and no others.
//! - *Live bytes* are charged with [`Meter::add`] before the storage exists
//!   and returned with [`Meter::released`] after it has ended.
//! - While a vector is moved into a larger bank, the old bank and the new bank
//!   are both live. The old bank was already charged by whoever created it, so
//!   [`grow`] charges only the new one, and returns the old one once it has
//!   ended.
//! - A refusal is final for the operation: the helper stops and returns
//!   [`Refused`]. The host keeps its own reason for refusing; [`Refused`]
//!   carries none. Each helper's documentation lists exactly what it returns
//!   to the meter on each refusal path. Where it returns nothing (a refused
//!   excess charge in [`copy_str`], or a refused shrink charge in
//!   [`copy_boxed`]), the host is expected to end the whole build, as it does
//!   for any refusal.

use std::fmt;

/// The depth, node and byte limits of a build.
///
/// Building a document is iterative, so depth does not threaten the stack
/// during the build. Consumers of the built document may recurse, so the
/// default depth is finite. The node and byte limits are unbounded by default;
/// a caller that feeds the crate untrusted text sets them.
///
/// ```
/// use surgeist_css_parser::limits::Limits;
///
/// let limits = Limits::new(32, 10_000, 1 << 20);
/// assert_eq!(limits.max_depth(), 32);
/// assert_eq!(Limits::default().max_depth(), 256);
/// assert_eq!(Limits::default().max_nodes(), usize::MAX);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Limits {
    max_depth: u32,
    max_nodes: usize,
    max_bytes: usize,
}

/// The depth of the default [`Limits`]: deep enough for any stylesheet written
/// by hand, shallow enough for a recursive consumer to survive.
const DEFAULT_MAX_DEPTH: u32 = 256;

impl Limits {
    /// Creates limits of at most `max_depth` open blocks and functions at once,
    /// `max_nodes` nodes and `max_bytes` bytes of retained text.
    #[must_use]
    pub const fn new(max_depth: u32, max_nodes: usize, max_bytes: usize) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_bytes,
        }
    }

    /// Returns the greatest number of blocks and functions that may be open at
    /// once.
    #[must_use]
    pub const fn max_depth(self) -> u32 {
        self.max_depth
    }

    /// Returns the greatest number of nodes a build may hold.
    #[must_use]
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// Returns the greatest number of bytes of retained text a build may hold.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

impl Default for Limits {
    /// A depth of 256 and no node or byte limit.
    fn default() -> Self {
        Self::new(DEFAULT_MAX_DEPTH, usize::MAX, usize::MAX)
    }
}

/// What a build does when input nests deeper than [`Limits::max_depth`].
///
/// The default is to fail the whole build.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum DepthExcess {
    /// Refuse the whole build.
    #[default]
    Fail,
    /// Keep a `{` block that nests too deep as one opaque node and continue.
    ///
    /// Only a `{` block is denied this way, and the depth is counted as every
    /// open block or function, including any base depth the build started
    /// with. A `(` block, a `[` block or a function that nests past the limit
    /// still fails the build with the nesting limit.
    DenyBlock,
}

/// Which resource failure a [`Meter`] is asked to record.
///
/// The helpers in this module cannot represent these failures in their
/// [`Refused`] result, so they tell the meter, which records the first one as
/// the reason its host reports.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum Failure {
    /// The allocator could not provide the requested storage.
    Allocation,
    /// A length, capacity or byte count could not be represented.
    CapacityOverflow,
}

/// A host's refusal of an action or a storage request.
///
/// The refusing [`Meter`] keeps the reason, such as a limit it enforces, a
/// cancellation or a recorded [`Failure`]. `Refused` only says that the
/// operation must stop.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Refused;

impl fmt::Display for Refused {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the meter refused the action")
    }
}

impl std::error::Error for Refused {}

/// The host side of charged work and live storage.
///
/// A meter takes `&self` and is expected to use interior mutability, because a
/// build and every stream over it share one meter. The host enforces its own
/// limits and keeps its own reason for a refusal.
///
/// # Refusal
///
/// A refusal is final for whoever asked. A [`Document`](crate::Document) built
/// with a meter latches the first refusal made to a stream over it: every later
/// metered step through the document fails without asking the meter again, and
/// [`Document::is_refused`](crate::Document::is_refused) says so. A host
/// therefore need not remember that it refused, and a meter that would admit
/// the next call (a host that refused once, for instance, to interrupt a
/// grammar) cannot make a grammar see a half-refused document. Code that
/// calls the meter directly, [`grow`] for example, is expected to stop at a
/// refusal.
///
/// Implementations must be consistent with the charging model in the
/// [module documentation](self#charging-model): [`released`](Self::released)
/// is only ever called with bytes that were previously admitted through
/// [`add`](Self::add) and have since ended.
pub trait Meter {
    /// Admits one action.
    ///
    /// # Errors
    ///
    /// Returns [`Refused`] when the host does not allow the action.
    fn step(&self) -> Result<(), Refused>;

    /// Admits `bytes` more bytes of live storage.
    ///
    /// # Errors
    ///
    /// Returns [`Refused`] when the host does not allow the storage, including
    /// when the total would not be representable.
    fn add(&self, bytes: usize) -> Result<(), Refused>;

    /// Records that `bytes` bytes of admitted storage have ended.
    ///
    /// This is infallible: a host cannot refuse to be told that storage is
    /// gone, so cleanup always completes.
    fn released(&self, bytes: usize);

    /// Records `failure` as the reason for the refusal about to be returned,
    /// unless the host has already recorded one, and returns the [`Refused`]
    /// to return.
    fn note(&self, failure: Failure) -> Refused;
}

/// Makes room in `values` for `additional` more elements, charging `meter`.
///
/// This is the crate's one vector-growth model. If the vector already has room
/// for the elements, only the opening step is taken. Otherwise the new
/// capacity is the greatest of the required length, twice the old capacity and
/// four; a new bank of exactly that capacity is reserved fallibly; every
/// element moves into it, one step each; and the old bank ends. The old and
/// new banks are live at the same time, and no reallocation hides that cost.
///
/// The sequence of meter calls is:
///
/// 1. [`step`](Meter::step).
/// 2. If the vector has room, stop. Otherwise [`add`](Meter::add) the bytes of
///    the planned new bank plus the size of the move's bookkeeping state, a
///    vector and a vector iterator.
/// 3. Reserve the new bank. If that fails, [`released`](Meter::released) the
///    bytes just added and [`note`](Meter::note) [`Failure::Allocation`].
/// 4. If the bank came out larger than planned, [`add`](Meter::add) the
///    excess.
/// 5. One [`step`](Meter::step) for each element moved.
/// 6. [`released`](Meter::released) the bytes of the old bank and the
///    bookkeeping state.
///
/// A length or byte count that cannot be represented is reported through
/// [`note`](Meter::note) with [`Failure::CapacityOverflow`].
///
/// # Errors
///
/// Returns [`Refused`] when the meter refuses an action or storage, or when a
/// size overflows or the allocator fails. If the refusal comes before elements
/// start to move, `values` is unchanged and everything this call charged has
/// been released. If it comes while elements move, the moved and unmoved
/// elements and both banks have been dropped, `values` is left empty, and the
/// release covers the old bank as well as the new bank and the bookkeeping
/// state, because the old bank's elements ended with it.
///
/// ```
/// use surgeist_css_parser::limits::{grow, Meter, Refused, Failure};
/// use std::cell::Cell;
///
/// struct Counting(Cell<usize>);
/// impl Meter for Counting {
///     fn step(&self) -> Result<(), Refused> { self.0.set(self.0.get() + 1); Ok(()) }
///     fn add(&self, _: usize) -> Result<(), Refused> { Ok(()) }
///     fn released(&self, _: usize) {}
///     fn note(&self, _: Failure) -> Refused { Refused }
/// }
///
/// let meter = Counting(Cell::new(0));
/// let mut values = vec![1, 2, 3];
/// grow(&meter, &mut values, 1).unwrap();
/// assert!(values.capacity() >= 4);
/// assert_eq!(values, [1, 2, 3]);
/// ```
pub fn grow<T>(meter: &dyn Meter, values: &mut Vec<T>, additional: usize) -> Result<(), Refused> {
    meter.step()?;
    let Some(required) = values.len().checked_add(additional) else {
        return Err(meter.note(Failure::CapacityOverflow));
    };
    if required <= values.capacity() {
        return Ok(());
    }
    let Some(old) = values.capacity().checked_mul(size_of::<T>()) else {
        return Err(meter.note(Failure::CapacityOverflow));
    };
    let target = required.max(values.capacity().saturating_mul(2)).max(4);
    let Some(planned) = target.checked_mul(size_of::<T>()) else {
        return Err(meter.note(Failure::CapacityOverflow));
    };
    struct Growth<T> {
        next: Vec<T>,
        previous: std::vec::IntoIter<T>,
    }
    let header = size_of::<Growth<T>>();
    let Some(prospective) = planned.checked_add(header) else {
        return Err(meter.note(Failure::CapacityOverflow));
    };
    meter.add(prospective)?;
    let mut next = Vec::new();
    if next.try_reserve_exact(target).is_err() {
        meter.released(prospective);
        return Err(meter.note(Failure::Allocation));
    }
    let Some(actual) = next.capacity().checked_mul(size_of::<T>()) else {
        return Err(meter.note(Failure::CapacityOverflow));
    };
    if actual > planned
        && let Err(refused) = meter.add(actual - planned)
    {
        drop(next);
        meter.released(prospective);
        return Err(refused);
    }
    let mut growth = Growth {
        next,
        previous: std::mem::take(values).into_iter(),
    };
    while let Some(value) = growth.previous.next() {
        if let Err(refused) = meter.step() {
            drop(value);
            drop(growth);
            meter.released(old + actual + header);
            return Err(refused);
        }
        growth.next.push(value);
    }
    let Growth { next, previous } = growth;
    drop(previous);
    *values = next;
    meter.released(old + header);
    Ok(())
}

/// Copies `value` into a new [`String`] of exactly its length, charging
/// `meter` one step per scalar value.
///
/// The sequence of meter calls is:
///
/// 1. [`step`](Meter::step).
/// 2. [`add`](Meter::add) the length of `value` in bytes.
/// 3. Reserve exactly that capacity. If that fails,
///    [`released`](Meter::released) the bytes just added and
///    [`note`](Meter::note) [`Failure::Allocation`].
/// 4. If the capacity came out larger than the length,
///    [`add`](Meter::add) the excess.
/// 5. One [`step`](Meter::step) for each `char` of `value`. If one is refused,
///    the partial copy is dropped and its capacity
///    [`released`](Meter::released).
///
/// A refusal of step 2's add or step 4's add returns nothing to the meter.
///
/// # Errors
///
/// Returns [`Refused`] when the meter refuses an action or storage or the
/// allocator fails. No partial string is returned.
///
/// ```
/// use surgeist_css_parser::limits::{copy_str, Meter, Refused, Failure};
///
/// struct Free;
/// impl Meter for Free {
///     fn step(&self) -> Result<(), Refused> { Ok(()) }
///     fn add(&self, _: usize) -> Result<(), Refused> { Ok(()) }
///     fn released(&self, _: usize) {}
///     fn note(&self, _: Failure) -> Refused { Refused }
/// }
///
/// let copy = copy_str(&Free, "naïve").unwrap();
/// assert_eq!(copy, "naïve");
/// assert_eq!(copy.capacity(), "naïve".len());
/// ```
pub fn copy_str(meter: &dyn Meter, value: &str) -> Result<String, Refused> {
    meter.step()?;
    meter.add(value.len())?;
    let mut out = String::new();
    if out.try_reserve_exact(value.len()).is_err() {
        meter.released(value.len());
        return Err(meter.note(Failure::Allocation));
    }
    if out.capacity() > value.len() {
        meter.add(out.capacity() - value.len())?;
    }
    for point in value.chars() {
        if let Err(refused) = meter.step() {
            let capacity = out.capacity();
            drop(out);
            meter.released(capacity);
            return Err(refused);
        }
        out.push(point);
    }
    Ok(out)
}

/// Copies `value` into a new [`Box<str>`], charging `meter` as [`copy_str`]
/// does.
///
/// After the copy, if the string's capacity differs from its length, the
/// shrink to a boxed string is charged as a second bank that coexists with the
/// first: [`add`](Meter::add) the length, then
/// [`released`](Meter::released) the old capacity.
///
/// # Errors
///
/// Returns [`Refused`] in the same cases as [`copy_str`], and when the meter
/// refuses the storage for the shrink.
///
/// ```
/// use surgeist_css_parser::limits::{copy_boxed, Meter, Refused, Failure};
///
/// struct Free;
/// impl Meter for Free {
///     fn step(&self) -> Result<(), Refused> { Ok(()) }
///     fn add(&self, _: usize) -> Result<(), Refused> { Ok(()) }
///     fn released(&self, _: usize) {}
///     fn note(&self, _: Failure) -> Refused { Refused }
/// }
///
/// assert_eq!(&*copy_boxed(&Free, "red").unwrap(), "red");
/// ```
pub fn copy_boxed(meter: &dyn Meter, value: &str) -> Result<Box<str>, Refused> {
    let out = copy_str(meter, value)?;
    let capacity = out.capacity();
    if capacity != out.len() {
        meter.add(out.len())?;
    }
    let result = out.into_boxed_str();
    if capacity != result.len() {
        meter.released(capacity);
    }
    Ok(result)
}

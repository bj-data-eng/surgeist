//! Checked work and resource ledgers for synchronous operations.
//!
//! Choose work and resource dimensions for an operation. Each scope can use a
//! different denomination and define its tariff to the parent. Usage in a scope
//! includes its descendants in that scope's units.

#![forbid(unsafe_code)]

use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

/// A work dimension in an operation's schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Work<const N: usize>(usize);

impl<const N: usize> Work<N> {
    /// Selects an index below the schema's work dimension count.
    pub const fn new(index: usize) -> Option<Self> {
        if index < N { Some(Self(index)) } else { None }
    }
}

/// A live resource dimension in an operation's schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resource<const N: usize>(usize);

impl<const N: usize> Resource<N> {
    /// Selects an index below the schema's resource dimension count.
    pub const fn new(index: usize) -> Option<Self> {
        if index < N { Some(Self(index)) } else { None }
    }
}

/// Per-dimension caps in the scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits<const W: usize, const R: usize> {
    work: [u64; W],
    live: [u64; R],
}

impl<const W: usize, const R: usize> Limits<W, R> {
    pub const fn new(work: [u64; W], live: [u64; R]) -> Self {
        Self { work, live }
    }

    /// Removes configured caps. Arithmetic remains checked.
    pub const fn unlimited() -> Self {
        Self::new([u64::MAX; W], [u64::MAX; R])
    }
}

/// Maps one dimension to an ancestor dimension with an integral scale.
/// A fixed scale makes arbitrary partial releases additive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tariff<D> {
    target: D,
    scale: u64,
}

impl<D> Tariff<D> {
    pub const fn new(target: D, scale: u64) -> Self {
        Self { target, scale }
    }
}

/// One mapping per local dimension, indexed in schema order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tariffs<const W: usize, const R: usize> {
    work: [Tariff<Work<W>>; W],
    resources: [Tariff<Resource<R>>; R],
}

impl<const W: usize, const R: usize> Tariffs<W, R> {
    pub const fn new(work: [Tariff<Work<W>>; W], resources: [Tariff<Resource<R>>; R]) -> Self {
        Self { work, resources }
    }

    /// Explicitly keeps the same dimensions and units in the parent.
    pub fn identity() -> Self {
        Self::new(
            std::array::from_fn(|i| Tariff::new(Work(i), 1)),
            std::array::from_fn(|i| Tariff::new(Resource(i), 1)),
        )
    }

    fn project(&self, change: Change<W, R>) -> Option<Change<W, R>> {
        Some(match change {
            Change::Work(dimension, amount) => {
                let tariff = self.work[dimension.0];
                Change::Work(tariff.target, amount.checked_mul(tariff.scale)?)
            }
            Change::Acquire(dimension, amount) => {
                let tariff = self.resources[dimension.0];
                Change::Acquire(tariff.target, amount.checked_mul(tariff.scale)?)
            }
            Change::Release(dimension, amount) => {
                let tariff = self.resources[dimension.0];
                Change::Release(tariff.target, amount.checked_mul(tariff.scale)?)
            }
        })
    }
}

/// One ordered ledger change, expressed in the originating scope's units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Change<const W: usize, const R: usize> {
    Work(Work<W>, u64),
    Acquire(Resource<R>, u64),
    Release(Resource<R>, u64),
}

/// An operation-owned scope. Its identity stays valid for the operation's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Scope {
    operation: u64,
    index: usize,
}

/// Inclusive accepted usage in one scope's own units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage<const W: usize, const R: usize> {
    work: [u64; W],
    live: [u64; R],
    peak: [u64; R],
}

impl<const W: usize, const R: usize> Usage<W, R> {
    const ZERO: Self = Self {
        work: [0; W],
        live: [0; R],
        peak: [0; R],
    };

    pub const fn work(&self, dimension: Work<W>) -> u64 {
        self.work[dimension.0]
    }

    pub const fn live(&self, dimension: Resource<R>) -> u64 {
        self.live[dimension.0]
    }

    pub const fn peak(&self, dimension: Resource<R>) -> u64 {
        self.peak[dimension.0]
    }

    fn current(&self, change: Change<W, R>) -> u64 {
        match change {
            Change::Work(dimension, _) => self.work(dimension),
            Change::Acquire(dimension, _) | Change::Release(dimension, _) => self.live(dimension),
        }
    }

    fn stage(&mut self, limits: &Limits<W, R>, change: Change<W, R>) -> Result<(), RefusalReason> {
        match change {
            Change::Work(dimension, amount) => {
                let current = &mut self.work[dimension.0];
                *current = increase(*current, amount, limits.work[dimension.0])?;
            }
            Change::Acquire(dimension, amount) => {
                let current = &mut self.live[dimension.0];
                *current = increase(*current, amount, limits.live[dimension.0])?;
                let peak = &mut self.peak[dimension.0];
                *peak = (*peak).max(*current);
            }
            Change::Release(dimension, amount) => {
                let current = &mut self.live[dimension.0];
                *current = current
                    .checked_sub(amount)
                    .ok_or(RefusalReason::Underflow)?;
            }
        }
        Ok(())
    }
}

fn increase(current: u64, amount: u64, limit: u64) -> Result<u64, RefusalReason> {
    let next = current.checked_add(amount).ok_or(RefusalReason::Overflow)?;
    if next > limit {
        return Err(RefusalReason::Limit { limit });
    }
    Ok(next)
}

/// Why an ordered change was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefusalReason {
    Overflow,
    Underflow,
    Limit { limit: u64 },
}

/// The first refused item. Current usage includes the provisional prefix of the
/// batch and, for a custody underflow, describes only the origin's direct hold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Refusal<const W: usize, const R: usize> {
    pub scope: Scope,
    pub item: usize,
    pub change: Change<W, R>,
    pub current: u64,
    pub reason: RefusalReason,
}

/// A ledger admission or scope management failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<const W: usize, const R: usize> {
    Allocation,
    IdentityExhausted,
    ForeignScope,
    ScopeLimit { limit: usize },
    Refused(Refusal<W, R>),
}

impl<const W: usize, const R: usize> fmt::Display for Error<W, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation => f.write_str("could not allocate ledger scope storage"),
            Self::IdentityExhausted => f.write_str("ledger operation identities exhausted"),
            Self::ForeignScope => f.write_str("scope belongs to another operation"),
            Self::ScopeLimit { limit } => write!(f, "ledger scope limit of {limit} reached"),
            Self::Refused(refusal) => write!(
                f,
                "ledger change {} refused: {:?}",
                refusal.item, refusal.reason
            ),
        }
    }
}

impl<const W: usize, const R: usize> std::error::Error for Error<W, R> {}

struct Parent<const W: usize, const R: usize> {
    index: usize,
    tariffs: Tariffs<W, R>,
}

struct ScopeEntry<const W: usize, const R: usize> {
    name: &'static str,
    parent: Option<Parent<W, R>>,
    limits: Limits<W, R>,
    usage: Usage<W, R>,
    staged: Usage<W, R>,
    direct: [u64; R],
}

impl<const W: usize, const R: usize> ScopeEntry<W, R> {
    const fn new(name: &'static str, parent: Option<Parent<W, R>>, limits: Limits<W, R>) -> Self {
        Self {
            name,
            parent,
            limits,
            usage: Usage::ZERO,
            staged: Usage::ZERO,
            direct: [0; R],
        }
    }

    fn parent_index(&self) -> Option<usize> {
        self.parent.as_ref().map(|parent| parent.index)
    }
}

static NEXT_OPERATION: AtomicU64 = AtomicU64::new(0);

/// The ledger and scope tree for one synchronous operation.
///
/// Scopes are retained until this ledger is dropped. Only the originating scope
/// can release its direct custody, even when that scope has children. Consumers
/// own cancellation and terminal state and can issue cleanup after refusal.
pub struct Operation<const W: usize, const R: usize> {
    identity: u64,
    max_scopes: usize,
    scopes: Vec<ScopeEntry<W, R>>,
}

impl<const W: usize, const R: usize> Operation<W, R> {
    /// Creates an operation. The scope bound includes the root.
    pub fn new(
        name: &'static str,
        limits: Limits<W, R>,
        max_scopes: usize,
    ) -> Result<Self, Error<W, R>> {
        if max_scopes == 0 {
            return Err(Error::ScopeLimit { limit: 0 });
        }
        let mut scopes = Vec::new();
        scopes.try_reserve(1).map_err(|_| Error::Allocation)?;
        let identity = NEXT_OPERATION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::IdentityExhausted)?;
        scopes.push(ScopeEntry::new(name, None, limits));
        Ok(Self {
            identity,
            max_scopes,
            scopes,
        })
    }

    pub const fn root(&self) -> Scope {
        self.scope(0)
    }

    /// Opens a child with its own caps and explicit projection into its parent.
    pub fn child(
        &mut self,
        parent: Scope,
        name: &'static str,
        limits: Limits<W, R>,
        tariffs: Tariffs<W, R>,
    ) -> Result<Scope, Error<W, R>> {
        let parent = self.index(parent)?;
        let index = self.scopes.len();
        if index == self.max_scopes {
            return Err(Error::ScopeLimit {
                limit: self.max_scopes,
            });
        }
        self.scopes.try_reserve(1).map_err(|_| Error::Allocation)?;
        self.scopes.push(ScopeEntry::new(
            name,
            Some(Parent {
                index: parent,
                tariffs,
            }),
            limits,
        ));
        Ok(self.scope(index))
    }

    pub fn usage(&self, scope: Scope) -> Result<&Usage<W, R>, Error<W, R>> {
        Ok(&self.scopes[self.index(scope)?].usage)
    }

    pub fn scope_name(&self, scope: Scope) -> Result<&'static str, Error<W, R>> {
        Ok(self.scopes[self.index(scope)?].name)
    }

    pub fn apply(&mut self, scope: Scope, change: Change<W, R>) -> Result<(), Error<W, R>> {
        self.apply_batch(scope, &[change])
    }

    /// Admits a batch atomically, checking items in input order and each item
    /// from the origin towards the root. A scope is checked before applying its
    /// tariff to reach the next scope. Intermediate accepted live usage counts
    /// towards peak, including acquire/release pairs for temporary resources.
    ///
    /// Any refusal discards the entire batch. Earlier batches are retained.
    /// Callers can issue a separate release after refusal or cancellation.
    pub fn apply_batch(
        &mut self,
        scope: Scope,
        changes: &[Change<W, R>],
    ) -> Result<(), Error<W, R>> {
        let origin = self.index(scope)?;
        if changes.is_empty() {
            return Ok(());
        }
        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let entry = &mut self.scopes[index];
            entry.staged = entry.usage;
            cursor = entry.parent_index();
        }

        let mut direct = self.scopes[origin].direct;
        for (item, &change) in changes.iter().enumerate() {
            if let Change::Release(dimension, amount) = change {
                let current = &mut direct[dimension.0];
                *current = current.checked_sub(amount).ok_or(Error::Refused(Refusal {
                    scope,
                    item,
                    change,
                    current: *current,
                    reason: RefusalReason::Underflow,
                }))?;
            }

            let mut index = origin;
            let mut projected = change;
            loop {
                let entry = &mut self.scopes[index];
                let current = entry.staged.current(projected);
                let refusal = |reason| {
                    Error::Refused(Refusal {
                        scope: Scope {
                            operation: scope.operation,
                            index,
                        },
                        item,
                        change: projected,
                        current,
                        reason,
                    })
                };
                entry
                    .staged
                    .stage(&entry.limits, projected)
                    .map_err(refusal)?;
                let Some(parent) = entry.parent.as_ref() else {
                    break;
                };
                projected = parent
                    .tariffs
                    .project(projected)
                    .ok_or_else(|| refusal(RefusalReason::Overflow))?;
                index = parent.index;
            }

            if let Change::Acquire(dimension, amount) = change {
                // Direct holdings are a subset of the admitted inclusive total.
                direct[dimension.0] += amount;
            }
        }

        let mut cursor = Some(origin);
        while let Some(index) = cursor {
            let entry = &mut self.scopes[index];
            entry.usage = entry.staged;
            cursor = entry.parent_index();
        }
        self.scopes[origin].direct = direct;
        Ok(())
    }

    const fn scope(&self, index: usize) -> Scope {
        Scope {
            operation: self.identity,
            index,
        }
    }

    fn index(&self, scope: Scope) -> Result<usize, Error<W, R>> {
        if scope.operation == self.identity {
            Ok(scope.index)
        } else {
            Err(Error::ForeignScope)
        }
    }
}

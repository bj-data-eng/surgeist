//! Engine-neutral observations and failure custody for native host adapters.

use std::cell::OnceCell;
use std::error::Error as Cause;
use std::fmt;
use std::rc::{Rc, Weak};

use crate::owner::OwnerState;
use crate::{Error, RealmKey};

/// Opaque identity of an observed realm in its original process owner.
/// It does not retain the context, grant entry, or identify a native Document.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RealmId(pub(crate) RealmKey);

/// An engine Realm observation with weak process liveness.
/// The host establishes native attachment, incumbent settings, and entry
/// authority separately.
#[derive(Clone)]
pub struct RealmAuthority {
    pub(crate) key: RealmKey,
    pub(crate) owner: Weak<OwnerState>,
}

impl fmt::Debug for RealmAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealmAuthority")
            .field("identity", &self.identity())
            .finish_non_exhaustive()
    }
}

impl RealmAuthority {
    pub fn identity(&self) -> RealmId {
        RealmId(self.key)
    }

    /// Checks the original Rust owner without entering the SDK. This does not
    /// establish that an unrooted context is still reachable by the engine.
    pub fn check_owner_active(&self) -> Result<(), Error> {
        self.owner
            .upgrade()
            .ok_or(Error::ExpiredIdentity)?
            .available()
    }
}

trait StoredCause {
    fn source(&self) -> Option<&(dyn Cause + 'static)>;
}

struct PreparedCause<E, Storage> {
    // Error payload is destroyed before its backing debit is released.
    cause: OnceCell<E>,
    _storage: Storage,
}

impl<E: Cause + 'static, Storage> StoredCause for PreparedCause<E, Storage> {
    fn source(&self) -> Option<&(dyn Cause + 'static)> {
        self.cause.get().map(|cause| cause as &dyn Cause)
    }
}

/// One immutable native cause, frozen in preallocated custody before crossing
/// the engine boundary. Cloning retains the same cause without allocating.
/// Returning it as [`Error::HostFailure`] from a callback reports a terminal
/// refusal for its original process. Other processes refuse the foreign error.
#[derive(Clone)]
pub struct HostFailure {
    pub(crate) owner: u64,
    cause: Rc<dyn StoredCause>,
}

impl HostFailure {
    /// Returns the original typed cause, with no diagnostic conversion.
    pub fn cause<E: Cause + 'static>(&self) -> Option<&E> {
        self.cause.source()?.downcast_ref()
    }
}

impl fmt::Debug for HostFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HostFailure").finish_non_exhaustive()
    }
}
impl fmt::Display for HostFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("native host refused execution")
    }
}
impl Cause for HostFailure {
    fn source(&self) -> Option<&(dyn Cause + 'static)> {
        self.cause.source()
    }
}

/// Prepared storage for the first native error of one process owner. A slot
/// and its frozen failure retain custody independently of engine handles.
/// Supply causes and storage guards that carry no Process or live native
/// authority. Prepare before accepting effects and retain adapter custody
/// until SDK frames end. Cause and guard destruction follow ordinary Rust rules.
pub struct HostFailureSlot<E, Storage> {
    authority: RealmAuthority,
    cause: Rc<PreparedCause<E, Storage>>,
}

/// Refusal to record a native cause. The attempted cause remains caller-owned.
#[derive(Debug)]
pub enum RecordFailure<E> {
    /// The original process is unavailable. Its first error, if any, is intact.
    Inactive { error: Error, cause: E },
    /// Another cause already occupies the slot. It cannot be replaced.
    Occupied { first: HostFailure, cause: E },
}

impl<E: Cause + 'static, Storage: 'static> HostFailureSlot<E, Storage> {
    pub(crate) fn owner_id(&self) -> u64 {
        self.authority.key.owner
    }

    /// Requested Rust target body for admission, excluding Rc metadata.
    pub const fn storage_bytes() -> usize {
        std::mem::size_of::<PreparedCause<E, Storage>>()
    }

    /// Prepare the target with its already admitted storage guard. The guard
    /// remains live until the last slot/failure token releases the cause.
    pub fn new(authority: &RealmAuthority, storage: Storage) -> Result<Self, Error> {
        authority.check_owner_active()?;
        Ok(Self {
            authority: authority.clone(),
            cause: Rc::new(PreparedCause {
                cause: OnceCell::new(),
                _storage: storage,
            }),
        })
    }

    /// Move the original cause into the prepared slot. Neither storing the
    /// cause nor issuing its token allocates, enters the SDK, or drops a
    /// rejected cause. Only a filled slot can issue a failure token.
    pub fn record(&self, cause: E) -> Result<HostFailure, RecordFailure<E>> {
        let eligible = self
            .authority
            .owner
            .upgrade()
            .ok_or(Error::ExpiredIdentity)
            .and_then(|owner| owner.check_host_failure_report());
        if let Err(error) = eligible {
            return Err(RecordFailure::Inactive { error, cause });
        }
        match self.cause.cause.set(cause) {
            Ok(()) => Ok(self.failure()),
            Err(cause) => Err(RecordFailure::Occupied {
                first: self.failure(),
                cause,
            }),
        }
    }

    fn failure(&self) -> HostFailure {
        HostFailure {
            owner: self.authority.key.owner,
            cause: self.cause.clone(),
        }
    }
}

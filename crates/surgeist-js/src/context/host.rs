//! Short, identity-preserving leases of context access for host code.

use std::cell::RefCell;

use super::{Context, ContextAccess};
use crate::{JsResult, error::EngineError};

/// A borrowed context holder that lends one short access capability at a time.
///
/// Hosts can retain a shared reference to this holder across asynchronous work,
/// then use [`Self::with`] for each synchronous engine operation. Each lease
/// releases its private cell borrow before calling host code and restores the
/// original capability on return or unwinding. Another request to this holder
/// while a lease is active returns [`EngineError::ContextAccessBusy`]. Reentering
/// the engine through the currently lent capability remains supported.
///
/// This holder neither owns nor moves the underlying [`Context`]. It preserves
/// any active execution and does not create a budget or select unbounded access.
///
/// ```
/// use surgeist_js::{Context, Source};
///
/// let mut owner = Context::default();
/// let host = owner.host_access();
/// let value = host.with(|mut access| access.eval(Source::from_bytes("40 + 2")))?;
/// assert_eq!(value.as_number(), Some(42.0));
/// # Ok::<(), surgeist_js::JsError>(())
/// ```
///
/// The lent capability cannot escape through the callback's result:
///
/// ```compile_fail
/// use surgeist_js::Context;
///
/// let mut owner = Context::default();
/// let host = owner.host_access();
/// let escaped = host.with(|access| Ok(access));
/// ```
///
/// Converting it to another holder cannot extend that lease either:
///
/// ```compile_fail
/// use surgeist_js::Context;
///
/// let mut owner = Context::default();
/// let host = owner.host_access();
/// let escaped = host.with(|access| Ok(access.into_host_access()));
/// ```
///
/// A future retaining the lent capability cannot escape to suspend its lease:
///
/// ```compile_fail
/// use surgeist_js::{Context, Source};
///
/// let mut owner = Context::default();
/// let host = owner.host_access();
/// let escaped = host.with(|mut access| Ok(async move {
///     access.eval(Source::from_bytes("42"))
/// }));
/// ```
#[must_use]
pub struct HostContext<'owner> {
    access: RefCell<Option<ContextAccess<'owner>>>,
}

impl std::fmt::Debug for HostContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HostContext")
            .finish_non_exhaustive()
    }
}

impl Context {
    /// Borrows this context for short, exclusive host access leases.
    ///
    /// The returned holder reserves the context for its lifetime. It does not
    /// transfer ownership or change the context's active execution.
    #[inline]
    pub fn host_access(&mut self) -> HostContext<'_> {
        HostContext {
            access: RefCell::new(Some(self.access())),
        }
    }
}

impl<'owner> ContextAccess<'owner> {
    /// Moves this capability into a holder with the same owner lifetime.
    ///
    /// Moving the capability does not move or replace the underlying context.
    #[inline]
    pub fn into_host_access(self) -> HostContext<'owner> {
        HostContext {
            access: RefCell::new(Some(self)),
        }
    }
}

impl HostContext<'_> {
    /// Lends a shorter context capability for one synchronous host callback.
    ///
    /// The private cell is not borrowed during the callback. Its original
    /// capability is restored on return or Rust unwinding, even if the callback
    /// moves or replaces its local capability binding. The callback's result
    /// cannot retain the lent capability or a future borrowing it.
    ///
    /// # Errors
    /// Returns the uncatchable [`EngineError::ContextAccessBusy`] before invoking
    /// the callback if another lease is active. This availability rejection
    /// performs no ledger admission and does not replace a retained host stop.
    /// Otherwise, returns the callback's result unchanged.
    pub fn with<T, F>(&self, run: F) -> JsResult<T>
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
    {
        let access = self
            .access
            .try_borrow_mut()
            .map_err(|_| EngineError::ContextAccessBusy)?
            .take()
            .ok_or(EngineError::ContextAccessBusy)?;
        let mut lease = AccessLease {
            holder: self,
            access: Some(access),
        };
        run(lease.access.as_mut().expect("lease owns access").reborrow())
    }
}

struct AccessLease<'holder, 'owner> {
    holder: &'holder HostContext<'owner>,
    access: Option<ContextAccess<'owner>>,
}

impl Drop for AccessLease<'_, '_> {
    fn drop(&mut self) {
        let mut slot = self.holder.access.borrow_mut();
        assert!(slot.is_none(), "a lease restores its original empty holder");
        *slot = self.access.take();
    }
}

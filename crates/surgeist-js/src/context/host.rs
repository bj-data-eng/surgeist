//! Short, identity-preserving leases of context access for host code.

use std::cell::Cell;

use super::{Context, ContextAccess};
use crate::{JsResult, error::EngineError};

/// A borrowed context holder that lends one short access capability at a time.
///
/// Hosts can retain a shared reference to this holder across asynchronous work,
/// then use [`Self::with`] for each synchronous engine operation. Each lease
/// transfers the protected capability before calling host code and restores the
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
/// let mut owner = Context::builder().build().unwrap();
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
/// let mut owner = Context::builder().build().unwrap();
/// let host = owner.host_access();
/// let escaped = host.with(|access| Ok(access));
/// ```
///
/// Converting it to another holder cannot extend that lease either:
///
/// ```compile_fail
/// use surgeist_js::Context;
///
/// let mut owner = Context::builder().build().unwrap();
/// let host = owner.host_access();
/// let escaped = host.with(|access| Ok(access.into_host_access()));
/// ```
///
/// A future retaining the lent capability cannot escape to suspend its lease:
///
/// ```compile_fail
/// use surgeist_js::{Context, Source};
///
/// let mut owner = Context::builder().build().unwrap();
/// let host = owner.host_access();
/// let escaped = host.with(|mut access| Ok(async move {
///     access.eval(Source::from_bytes("42"))
/// }));
/// ```
#[must_use]
pub struct HostContext<'owner> {
    access: Cell<Option<ContextAccess<'owner>>>,
    fault: Cell<Option<crate::job::JobFault>>,
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
            access: Cell::new(Some(self.access())),
            fault: Cell::new(None),
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
            access: Cell::new(Some(self)),
            fault: Cell::new(None),
        }
    }
}

impl HostContext<'_> {
    /// Lends a shorter context capability for one synchronous host callback.
    ///
    /// The private slot holds no capability during the callback. Its original
    /// capability is restored on return or Rust unwinding, even if the callback
    /// moves or replaces its local capability binding. The callback's result
    /// cannot retain the lent capability or a future borrowing it.
    ///
    /// # Errors
    /// Returns the uncatchable [`EngineError::ContextAccessBusy`] before invoking
    /// the callback if another lease is active. This availability rejection
    /// performs no ledger admission and does not replace a retained host stop.
    /// Otherwise, returns the callback's result unchanged unless protected
    /// custody faults. A retained operation stop takes precedence over that fault.
    pub fn with<T, F>(&self, run: F) -> JsResult<T>
    where
        F: for<'call> FnOnce(ContextAccess<'call>) -> JsResult<T>,
    {
        match self.with_owned((), |(), access| run(access)) {
            Ok(result) => {
                if let Some(fault) = self.fault.get() {
                    self.with_job_cleanup(|access| access.into_owner().check_execution_stop())??;
                    if !result
                        .as_ref()
                        .is_err_and(|error| error.as_engine().is_some())
                    {
                        return Err(fault.into());
                    }
                }
                result
            }
            Err(((), error)) => Err(error),
        }
    }

    /// Acquires access or returns the original unconsumed input and its error.
    /// No callback runs on failure and the input is neither retained nor disposed.
    ///
    /// # Examples
    ///
    /// An acquired callback receives its input and a short context capability:
    ///
    /// ```
    /// use surgeist_js::{Context, Source};
    ///
    /// let mut owner = Context::builder().build().unwrap();
    /// let holder = owner.host_access();
    /// let answer = holder
    ///     .with_owned("40 + 2", |source, mut access| {
    ///         access.eval(Source::from_bytes(source))
    ///     })
    ///     .map_err(|(_, error)| error)??;
    /// assert_eq!(answer.as_number(), Some(42.0));
    /// # Ok::<(), surgeist_js::JsError>(())
    /// ```
    ///
    /// The lent capability cannot be returned from `with_owned`:
    ///
    /// ```compile_fail
    /// use surgeist_js::Context;
    ///
    /// let mut owner = Context::builder().build().unwrap();
    /// let holder = owner.host_access();
    /// let escaped = holder.with_owned((), |(), access| access).unwrap();
    /// ```
    ///
    /// A future capturing the lent capability cannot be returned either:
    ///
    /// ```compile_fail
    /// use surgeist_js::{Context, Source};
    ///
    /// let mut owner = Context::builder().build().unwrap();
    /// let holder = owner.host_access();
    /// let escaped = holder
    ///     .with_owned((), |(), mut access| async move {
    ///         access.eval(Source::from_bytes("42"))
    ///     })
    ///     .unwrap();
    /// ```
    pub fn with_owned<I, T, F>(&self, input: I, run: F) -> Result<T, (I, crate::JsError)>
    where
        F: for<'call> FnOnce(I, ContextAccess<'call>) -> T,
    {
        self.loan(input, run, LeaseKind::Entry)
    }

    pub(crate) fn fault(&self) -> Option<crate::job::JobFault> {
        self.fault.get()
    }

    fn record_fault(&self, cause: crate::job::JobFaultCause) -> crate::job::JobFault {
        let fault = self.fault.get().unwrap_or_else(|| {
            crate::job::JobFault::new(crate::job::JobFaultOwner::HostContext, cause)
        });
        self.fault.set(Some(fault));
        fault
    }

    // Mandatory restoration has no guest dispatch. It must use the original
    // capability even when ordinary access has been quarantined by a fault.
    pub(crate) fn with_job_cleanup<T>(
        &self,
        run: impl for<'call> FnOnce(ContextAccess<'call>) -> T,
    ) -> Result<T, crate::job::JobFault> {
        self.loan((), |(), access| run(access), LeaseKind::Cleanup)
            .map_err(|((), _)| {
                self.fault.get().unwrap_or_else(|| {
                    self.record_fault(crate::job::JobFaultCause::StateInconsistency)
                })
            })
    }

    fn loan<I, T>(
        &self,
        input: I,
        run: impl for<'call> FnOnce(I, ContextAccess<'call>) -> T,
        kind: LeaseKind,
    ) -> Result<T, (I, crate::JsError)> {
        let Some(access) = self.access.take() else {
            let error = match kind {
                LeaseKind::Entry => EngineError::ContextAccessBusy.into(),
                LeaseKind::Cleanup => self
                    .record_fault(crate::job::JobFaultCause::StateInconsistency)
                    .into(),
            };
            return Err((input, error));
        };
        let mut lease = AccessLease {
            holder: self,
            access: Some(access),
        };
        if matches!(kind, LeaseKind::Entry)
            && let Some(fault) = self.fault.get()
        {
            let error = match lease.access.as_mut() {
                Some(access) => access
                    .reborrow()
                    .into_owner()
                    .check_execution_stop()
                    .err()
                    .unwrap_or_else(|| fault.into()),
                None => self
                    .record_fault(crate::job::JobFaultCause::StateInconsistency)
                    .into(),
            };
            return Err((input, error));
        }
        match lease.access.as_mut() {
            Some(access) => Ok(run(input, access.reborrow())),
            None => Err((
                input,
                self.record_fault(crate::job::JobFaultCause::StateInconsistency)
                    .into(),
            )),
        }
    }
}

#[derive(Clone, Copy)]
enum LeaseKind {
    Entry,
    Cleanup,
}

struct AccessLease<'holder, 'owner> {
    holder: &'holder HostContext<'owner>,
    access: Option<ContextAccess<'owner>>,
}

impl Drop for AccessLease<'_, '_> {
    fn drop(&mut self) {
        let Some(original) = self.access.take() else {
            self.holder
                .record_fault(crate::job::JobFaultCause::StateInconsistency);
            return;
        };
        let displaced = self.holder.access.replace(Some(original));
        if displaced.is_some() {
            self.holder
                .record_fault(crate::job::JobFaultCause::StateInconsistency);
        }
        // The unexpected capability leaves scope only after original restoration.
    }
}

#[cfg(test)]
use crate::registry::UserEvent;
use crate::{Id, registry::Proxy};
use raw_window_handle::HandleError;
use std::{
    fmt,
    sync::{Arc, Mutex, Weak},
};

#[derive(Clone, Debug)]
pub(crate) struct ReleaseNotifier {
    proxy: Proxy,
}

impl ReleaseNotifier {
    pub(crate) fn new(proxy: Proxy) -> Self {
        Self { proxy }
    }

    fn notify(&self, id: Id) {
        let _ = self.proxy.send_release(id);
    }
}

pub(crate) struct Lease<R> {
    inner: Arc<LeaseInner<R>>,
}

/// Non-owning access to a shared native lease.
///
/// The closing owner keeps this token so it can invalidate externally retained
/// handles without extending the native resource lifetime.
pub(crate) struct LeaseWeak<R> {
    inner: Weak<LeaseInner<R>>,
}

impl<R> Clone for Lease<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<R> Clone for LeaseWeak<R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct LeaseInner<R> {
    id: Id,
    state: Mutex<LeaseState>,
    resource: Option<R>,
    notifier: ReleaseNotifier,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LeaseState {
    Live,
    Closing,
    Destroyed,
    LoopExited,
}

impl LeaseState {
    const fn owns_resource(self) -> bool {
        matches!(self, Self::Live | Self::Closing)
    }
}

impl<R> fmt::Debug for Lease<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let resource_available = self
            .inner
            .state
            .lock()
            .expect("native handle lease state mutex must not be poisoned")
            .owns_resource();
        f.debug_struct("Lease")
            .field("id", &self.inner.id)
            .field("resource_available", &resource_available)
            .finish()
    }
}

impl<R> Drop for LeaseInner<R> {
    fn drop(&mut self) {
        drop(self.resource.take());
        self.notifier.notify(self.id);
    }
}

impl<R> Lease<R> {
    pub(crate) fn new(id: Id, resource: R, notifier: ReleaseNotifier) -> Self {
        Self {
            inner: Arc::new(LeaseInner {
                id,
                state: Mutex::new(LeaseState::Live),
                resource: Some(resource),
                notifier,
            }),
        }
    }

    pub(crate) fn with_resource<'a, T>(
        &'a self,
        access: impl FnOnce(&'a R) -> T,
    ) -> Result<T, HandleError> {
        if !self.resource_is_available() {
            return Err(HandleError::Unavailable);
        }
        let resource = self
            .inner
            .resource
            .as_ref()
            .expect("a resource-owning native handle lease retains its resource");
        Ok(access(resource))
    }

    pub(crate) fn mark_closing(&self) {
        let mut state = self
            .inner
            .state
            .lock()
            .expect("native handle lease state mutex must not be poisoned");
        if *state == LeaseState::Live {
            *state = LeaseState::Closing;
        }
    }

    pub(crate) fn mark_destroyed(&self) {
        mark_terminal(&self.inner.state, LeaseState::Destroyed);
    }

    pub(crate) fn mark_loop_exited(&self) {
        mark_terminal(&self.inner.state, LeaseState::LoopExited);
    }

    pub(crate) fn id(&self) -> Id {
        self.inner.id
    }

    fn resource_is_available(&self) -> bool {
        self.inner
            .state
            .lock()
            .expect("native handle lease state mutex must not be poisoned")
            .owns_resource()
    }

    pub(crate) fn downgrade(&self) -> LeaseWeak<R> {
        LeaseWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl<R> LeaseWeak<R> {
    /// Returns whether all strong resource owners have released the lease.
    pub(crate) fn is_released(&self) -> bool {
        self.inner.strong_count() == 0
    }

    /// Invalidates every remaining external wrapper after native destruction.
    pub(crate) fn mark_destroyed(&self) {
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        mark_terminal(&inner.state, LeaseState::Destroyed);
    }

    /// Invalidates every remaining external wrapper once the event loop exits.
    pub(crate) fn mark_loop_exited(&self) {
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        mark_terminal(&inner.state, LeaseState::LoopExited);
    }
}

fn mark_terminal(state: &Mutex<LeaseState>, terminal: LeaseState) {
    let mut state = state
        .lock()
        .expect("native handle lease state mutex must not be poisoned");
    if matches!(*state, LeaseState::Live | LeaseState::Closing) {
        *state = terminal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ProxyQueue;
    use std::{cell::Cell, sync::Arc};

    const ID: Id = Id::from_u64(81);

    #[derive(Debug)]
    struct FakeResource {
        raw_accesses: Cell<u8>,
        cleanup_accesses: Cell<u8>,
    }

    impl FakeResource {
        fn raw_access(&self) -> Result<&'static str, HandleError> {
            self.raw_accesses.set(self.raw_accesses.get() + 1);
            Ok("fake-raw-resource")
        }

        fn cleanup(&self) {
            self.cleanup_accesses.set(self.cleanup_accesses.get() + 1);
        }
    }

    fn raw_access(lease: &Lease<FakeResource>) -> Result<&'static str, HandleError> {
        lease.with_resource(FakeResource::raw_access)?
    }

    fn lease() -> (Lease<FakeResource>, Arc<ProxyQueue>) {
        let queue = Arc::new(ProxyQueue::new());
        let notifier = ReleaseNotifier::new(Proxy::with_queue(queue.clone()));
        (
            Lease::new(
                ID,
                FakeResource {
                    raw_accesses: Cell::new(0),
                    cleanup_accesses: Cell::new(0),
                },
                notifier,
            ),
            queue,
        )
    }

    #[test]
    fn lease_allows_raw_access_only_while_resource_owning() {
        let (lease, _) = lease();
        let clone = lease.clone();

        assert!(matches!(raw_access(&lease), Ok("fake-raw-resource")));
        lease.mark_closing();
        assert!(matches!(raw_access(&clone), Ok("fake-raw-resource")));
        assert!(matches!(
            clone.with_resource(|resource| resource.raw_accesses.get()),
            Ok(2)
        ));

        lease.mark_destroyed();
        assert!(matches!(raw_access(&clone), Err(HandleError::Unavailable)));
    }

    #[test]
    fn lease_last_owner_drop_sends_one_release_notification() {
        let (lease, queue) = lease();
        let weak = lease.downgrade();
        let clone = lease.clone();

        drop(lease);
        assert!(queue.pop().is_none());
        assert!(!weak.is_released());

        drop(clone);
        assert!(weak.is_released());
        let UserEvent::HandleReleased(release) = queue
            .pop()
            .expect("the final lease owner must send one release notification")
        else {
            panic!("the final lease owner must send a release notification");
        };
        assert_eq!(release.id(), ID);
        assert!(queue.pop().is_none());
    }

    #[test]
    fn lease_destroyed_and_loop_exited_states_are_terminal() {
        let (destroyed, _) = lease();
        destroyed.mark_destroyed();
        destroyed.mark_closing();
        destroyed.mark_loop_exited();
        assert!(matches!(
            raw_access(&destroyed),
            Err(HandleError::Unavailable)
        ));

        let (loop_exited, _) = lease();
        let loop_exited_clone = loop_exited.clone();
        loop_exited.mark_loop_exited();
        loop_exited.mark_closing();
        loop_exited.mark_destroyed();
        assert!(matches!(
            raw_access(&loop_exited_clone),
            Err(HandleError::Unavailable)
        ));
    }

    #[test]
    fn lease_closing_retains_the_same_resource_for_cleanup() {
        let (lease, _) = lease();

        lease.mark_closing();
        assert!(matches!(
            lease.with_resource(|resource| resource.cleanup()),
            Ok(())
        ));
        assert!(matches!(
            lease.with_resource(|resource| resource.cleanup_accesses.get()),
            Ok(1)
        ));
    }

    #[test]
    fn lease_ignores_release_notification_failure() {
        let (lease, queue) = lease();
        queue.close();

        drop(lease);
        assert!(queue.pop().is_none());
    }
}

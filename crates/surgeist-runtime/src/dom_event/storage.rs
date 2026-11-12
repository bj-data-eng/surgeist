use super::{DomEventDispatchError, DomEventDom, DomEventResource};
use std::cell::Cell;
use std::mem::size_of;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use surgeist_dom::{DomError, DomEventPayloadRetention, DomOwnerId};

/// Finite shared storage and cumulative work for a session's Event objects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventStorageLimits {
    /// Actual fixed targets, UTF-16 backing and replacement overlap.
    pub storage_bytes: usize,
    /// Cumulative name comparisons, copying and object preparation work.
    pub work: u64,
}

struct StorageCounter {
    owner: Option<DomOwnerId>,
    limits: DomEventStorageLimits,
    bytes: Cell<usize>,
    work: Cell<u64>,
    _retained: Option<DomEventPayloadRetention>,
}

/// Accounting only. The last handle frees its counter target before releasing
/// its canonical receipt, independently of any execution owner.
#[derive(Clone)]
pub(crate) struct EventStorage(Option<Rc<StorageCounter>>);

impl Drop for EventStorage {
    fn drop(&mut self) {
        drop(self.0.take().and_then(Rc::into_inner));
    }
}

impl EventStorage {
    pub(crate) fn validate(limits: DomEventStorageLimits) -> Result<(), DomEventDispatchError> {
        if limits.storage_bytes < size_of::<StorageCounter>()
            || limits.storage_bytes > isize::MAX as usize
        {
            return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
        }
        Ok(())
    }

    pub(crate) fn session(
        dom: &mut DomEventDom<'_>,
        limits: DomEventStorageLimits,
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        Self::validate(limits)?;
        let retained =
            dom.raw_mut()
                .retain_event_payload(size_of::<StorageCounter>(), 0, cancel)?;
        Ok(Self(Some(Rc::new(StorageCounter {
            owner: Some(dom.owner()),
            limits,
            bytes: Cell::new(size_of::<StorageCounter>()),
            work: Cell::new(0),
            _retained: Some(retained),
        }))))
    }

    pub(crate) fn registry(limits: DomEventStorageLimits) -> Result<Self, DomEventDispatchError> {
        Self::validate(limits)?;
        Ok(Self(Some(Rc::new(StorageCounter {
            owner: None,
            limits,
            bytes: Cell::new(size_of::<StorageCounter>()),
            work: Cell::new(0),
            _retained: None,
        }))))
    }

    fn counter(&self) -> Result<&StorageCounter, DomEventDispatchError> {
        self.0.as_deref().ok_or(DomEventDispatchError::Abandoned)
    }

    pub(crate) fn step(
        &self,
        amount: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomEventDispatchError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(DomError::Cancelled.into());
        }
        let counter = self.counter()?;
        let amount = u64::try_from(amount)
            .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Work))?;
        let next = counter
            .work
            .get()
            .checked_add(amount)
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Work))?;
        counter.work.set(next);
        if next > counter.limits.work {
            return Err(DomEventDispatchError::Admission(DomEventResource::Work));
        }
        Ok(())
    }

    pub(crate) fn reserve(
        &self,
        dom: &mut DomEventDom<'_>,
        full: usize,
        source: usize,
        units: usize,
        cancel: &AtomicBool,
    ) -> Result<EventAllocation, DomEventDispatchError> {
        let counter = self.counter()?;
        if counter.owner.is_some_and(|owner| owner != dom.owner()) {
            return Err(DomError::ForeignOwner.into());
        }
        let next = counter
            .bytes
            .get()
            .checked_add(full)
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        if next > counter.limits.storage_bytes {
            return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
        }
        let retained = dom.raw_mut().retain_event_payload(source, units, cancel)?;
        counter.bytes.set(next);
        Ok(EventAllocation {
            storage: self.clone(),
            full,
            retained,
        })
    }
}

pub(crate) struct EventAllocation {
    storage: EventStorage,
    full: usize,
    retained: DomEventPayloadRetention,
}

impl Drop for EventAllocation {
    fn drop(&mut self) {
        if let Some(counter) = self.storage.0.as_ref() {
            counter.bytes.set(counter.bytes.get() - self.full);
        }
    }
}

impl EventAllocation {
    pub(crate) fn split(
        &mut self,
        full: usize,
        source: usize,
        units: usize,
    ) -> Result<Self, DomError> {
        let remainder = self
            .full
            .checked_sub(full)
            .ok_or(DomError::RepresentationExhausted)?;
        let retained = self.retained.split(source, units)?;
        self.full = remainder;
        Ok(Self {
            storage: self.storage.clone(),
            full,
            retained,
        })
    }

    pub(crate) fn absorb(&mut self, mut other: Self) -> Result<(), DomError> {
        let same = match (&self.storage.0, &other.storage.0) {
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            _ => false,
        };
        if !same {
            return Err(DomError::ForeignOwner);
        }
        let full = self
            .full
            .checked_add(other.full)
            .ok_or(DomError::RepresentationExhausted)?;
        let source = other.retained.retained_bytes();
        let units = other.retained.retained_units();
        let retained = other.retained.split(source, units)?;
        self.retained
            .absorb(retained)
            .map_err(|_| DomError::ForeignOwner)?;
        self.full = full;
        other.full = 0;
        Ok(())
    }

    pub(crate) fn shrink(&mut self, full: usize, source: usize) -> Result<(), DomError> {
        let released = self
            .full
            .checked_sub(full)
            .ok_or(DomError::RepresentationExhausted)?;
        let source_released = self
            .retained
            .retained_bytes()
            .checked_sub(source)
            .ok_or(DomError::RepresentationExhausted)?;
        drop(self.split(released, source_released, 0)?);
        Ok(())
    }
}

pub(super) struct TextBlock {
    units: Vec<u16>,
}
pub(super) struct TextCustody {
    _allocation: EventAllocation,
}

#[derive(Clone)]
pub(super) enum EventText {
    Static(&'static [u16]),
    // The units and fixed target drop before their shared accounting receipt.
    Owned {
        block: Rc<TextBlock>,
        _custody: Rc<TextCustody>,
    },
}

/// One exact immutable UTF-16 event type. Clones preserve its admitted backing.
#[derive(Clone)]
pub struct DomEventType(EventText);

impl DomEventType {
    pub(super) const EMPTY: Self = Self(EventText::Static(&[]));

    /// Exact `load` spelling.
    pub const LOAD: Self = Self(EventText::Static(&[108, 111, 97, 100]));
    /// Exact `error` spelling.
    pub const ERROR: Self = Self(EventText::Static(&[101, 114, 114, 111, 114]));
    /// Exact `toggle` spelling.
    pub const TOGGLE: Self = Self(EventText::Static(&[116, 111, 103, 103, 108, 101]));
    /// Exact `beforetoggle` spelling.
    pub const BEFORE_TOGGLE: Self = Self(EventText::Static(&[
        98, 101, 102, 111, 114, 101, 116, 111, 103, 103, 108, 101,
    ]));
    /// Exact `close` spelling.
    pub const CLOSE: Self = Self(EventText::Static(&[99, 108, 111, 115, 101]));
    /// Exact `cancel` spelling.
    pub const CANCEL: Self = Self(EventText::Static(&[99, 97, 110, 99, 101, 108]));
    #[cfg(feature = "document-scripting")]
    pub(super) const CSP_VIOLATION: Self = Self(EventText::Static(&[
        115, 101, 99, 117, 114, 105, 116, 121, 112, 111, 108, 105, 99, 121, 118, 105, 111, 108, 97,
        116, 105, 111, 110,
    ]));

    /// Exact code units, including isolated surrogates and NULs.
    pub fn code_units(&self) -> &[u16] {
        self.0.code_units()
    }

    pub(crate) fn retain(
        storage: &EventStorage,
        dom: &mut DomEventDom<'_>,
        units: &[u16],
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        for candidate in [
            Self::LOAD,
            Self::ERROR,
            Self::TOGGLE,
            Self::BEFORE_TOGGLE,
            Self::CLOSE,
            Self::CANCEL,
        ] {
            if equal_units(units, candidate.code_units(), || storage.step(1, cancel))? {
                return Ok(candidate);
            }
        }
        EventText::retain(storage, dom, units, cancel).map(Self)
    }
}

impl EventText {
    pub(super) fn code_units(&self) -> &[u16] {
        match self {
            Self::Static(units) => units,
            Self::Owned { block, .. } => &block.units,
        }
    }

    pub(super) fn retain(
        storage: &EventStorage,
        dom: &mut DomEventDom<'_>,
        units: &[u16],
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        storage.step(units.len(), cancel)?;
        let bytes = size_of::<TextBlock>()
            .checked_add(size_of::<TextCustody>())
            .and_then(|headers| {
                units
                    .len()
                    .checked_mul(size_of::<u16>())
                    .and_then(|bytes| headers.checked_add(bytes))
            })
            .ok_or(DomEventDispatchError::Admission(DomEventResource::Storage))?;
        let allocation = storage.reserve(dom, bytes, bytes, units.len(), cancel)?;
        let mut copy = Vec::new();
        copy.try_reserve_exact(units.len())
            .map_err(|_| DomEventDispatchError::Admission(DomEventResource::Allocation))?;
        if copy.capacity() != units.len() {
            return Err(DomEventDispatchError::Admission(DomEventResource::Storage));
        }
        copy.extend_from_slice(units);
        Ok(Self::Owned {
            block: Rc::new(TextBlock { units: copy }),
            _custody: Rc::new(TextCustody {
                _allocation: allocation,
            }),
        })
    }
}

impl PartialEq for EventText {
    fn eq(&self, other: &Self) -> bool {
        self.code_units() == other.code_units()
    }
}
impl Eq for EventText {}
impl std::fmt::Debug for EventText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.code_units().fmt(f)
    }
}

impl AsRef<[u16]> for DomEventType {
    fn as_ref(&self) -> &[u16] {
        self.code_units()
    }
}
impl PartialEq for DomEventType {
    fn eq(&self, other: &Self) -> bool {
        self.code_units() == other.code_units()
    }
}
impl Eq for DomEventType {}
impl std::fmt::Debug for DomEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DomEventType")
            .field(&self.code_units())
            .finish()
    }
}

pub(crate) fn equal_units<E>(
    left: &[u16],
    right: &[u16],
    mut step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    step()?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        step()?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "document-scripting")]
    #[test]
    fn toggle_payload_releases_the_first_state_if_the_second_state_refuses() {
        use super::super::{AuthoredToggleInit, DomEventInit, DomToggleEvent};

        let mut owner = super::super::tests::dom();
        let mut dom = DomEventDom::borrow(&mut owner);
        let cancel = AtomicBool::new(false);
        let old = [0xd800, 0];
        let new = [0xdc00];
        let init = AuthoredToggleInit {
            event: DomEventInit::default(),
            old_state: &old,
            new_state: &new,
            source: None,
        };
        let header = size_of::<StorageCounter>();
        let required = 2 * (size_of::<TextBlock>() + size_of::<TextCustody>())
            + size_of_val(&old)
            + size_of_val(&new);
        let limits = |storage_bytes| DomEventStorageLimits {
            storage_bytes,
            work: 100,
        };
        let short =
            EventStorage::session(&mut dom, limits(header + required - 1), &cancel).unwrap();
        assert!(matches!(
            DomToggleEvent::retain(&short, &mut dom, &init, &cancel),
            Err(DomEventDispatchError::Admission(DomEventResource::Storage))
        ));
        assert_eq!(short.counter().unwrap().work.get(), 3);
        assert_eq!(short.counter().unwrap().bytes.get(), header);
        drop(short);
        let exact = EventStorage::session(&mut dom, limits(header + required), &cancel).unwrap();
        let payload = DomToggleEvent::retain(&exact, &mut dom, &init, &cancel).unwrap();
        assert_eq!(payload.old_state(), old);
        assert_eq!(payload.new_state(), new);
        assert_eq!(exact.counter().unwrap().bytes.get(), header + required);
        let escaped = payload.clone();
        drop(payload);
        assert_eq!(exact.counter().unwrap().bytes.get(), header + required);
        drop(escaped);
        assert_eq!(exact.counter().unwrap().bytes.get(), header);
    }

    #[test]
    fn exact_type_admission_retains_canonical_bytes_and_units_through_escaped_clones() {
        let mut owner = super::super::tests::dom();
        let mut dom = DomEventDom::borrow(&mut owner);
        let cancel = AtomicBool::new(false);
        let input = [0xd800, 0, 0xdc01];
        let headers = size_of::<TextBlock>() + size_of::<TextCustody>();
        let header = size_of::<StorageCounter>();
        let bytes = headers + size_of_val(&input);
        let limits = |storage_bytes| DomEventStorageLimits {
            storage_bytes,
            work: 100,
        };
        let short = EventStorage::session(&mut dom, limits(header + bytes - 1), &cancel).unwrap();
        assert!(matches!(
            DomEventType::retain(&short, &mut dom, &input, &cancel),
            Err(DomEventDispatchError::Admission(DomEventResource::Storage))
        ));
        assert_eq!(short.counter().unwrap().bytes.get(), header);
        drop(short);

        let exact = EventStorage::session(&mut dom, limits(header + bytes), &cancel).unwrap();
        let name = DomEventType::retain(&exact, &mut dom, &input, &cancel).unwrap();
        let EventText::Owned {
            block,
            _custody: custody,
        } = &name.0
        else {
            panic!("independently retained exact input expected");
        };
        let actual = size_of_val(block.as_ref())
            + size_of_val(custody.as_ref())
            + block.units.capacity() * size_of::<u16>();
        assert_eq!(actual, bytes);
        assert_eq!(custody._allocation.retained.retained_bytes(), actual);
        assert_eq!(custody._allocation.retained.retained_units(), input.len());
        assert_eq!(name.code_units(), input);

        let escaped = name.clone();
        drop(name);
        assert!(matches!(
            DomEventType::retain(&exact, &mut dom, &input, &cancel),
            Err(DomEventDispatchError::Admission(DomEventResource::Storage))
        ));
        assert_eq!(escaped.code_units(), input);
        assert_eq!(exact.counter().unwrap().bytes.get(), header + actual);
        drop(escaped);
        assert_eq!(exact.counter().unwrap().bytes.get(), header);
        assert_eq!(
            DomEventType::retain(&exact, &mut dom, &input, &cancel)
                .unwrap()
                .code_units(),
            input
        );
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    ))]
    #[test]
    fn ordinary_dispatch_exact_formation_and_escaped_allocations() {
        use super::super::{
            ContinuationObservation, ContinuationState, DispatchFunding, DomDispatchCustody,
            DomEventCallback, DomEventCallbackError, DomEventComposedPath, DomEventDispatchLimits,
            DomEventDispatchOutcome, DomEventDispatcher, DomEventInit, DomEventListenerFailure,
            DomEventListenerFailureKind, DomEventListenerOptions, DomEventRegistrationId,
            DomEventRegistryLimits, DomEventTarget, EventContinuation, EventPath, Funding, NodeId,
            PathFunding, PathItem, Slot, tests::hosted_style_parent_tests,
        };
        let cancel = AtomicBool::new(false);
        let mut session = hosted_style_parent_tests::hosted_session();
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
        let html = dom
            .snapshot()
            .node(top.document().node())
            .unwrap()
            .children()[0];
        let target = dom.snapshot().node(html).unwrap().children()[1];
        let event = hosted
            .create_event(
                &mut dom,
                top.global(),
                DomEventType::CLOSE.code_units(),
                DomEventInit::default(),
                &cancel,
            )
            .unwrap();

        let dispatch_limits = DomEventDispatchLimits {
            path_items: 4,
            snapshot_listeners: 1,
            failure_records: 1,
            callbacks: 1,
            work: 10_000,
        };
        type HeldPath = Option<DomEventComposedPath>;
        let header = size_of::<StorageCounter>();
        let registration = size_of::<Slot<HeldPath>>();
        let index = size_of::<ContinuationObservation>();
        let fixed = size_of::<EventContinuation>() + size_of::<ContinuationState>();
        let path_items = 4 * size_of::<PathItem>();
        let path = size_of::<EventPath>() + path_items;
        let snapshot =
            size_of::<Vec<DomEventRegistrationId>>() + size_of::<DomEventRegistrationId>();
        let failure = size_of::<DomEventListenerFailure>();
        let gather_peak = size_of::<Vec<PathItem>>()
            + path_items
            + size_of::<Vec<NodeId>>()
            + 4 * size_of::<NodeId>();
        let failure_peak = path + size_of::<Vec<DomEventListenerFailure>>() + failure;
        let invoke_peak = path + snapshot + failure;
        let scope = DomDispatchCustody::maximum_retained_bytes(4).unwrap();
        let peak = header
            + registration
            + index
            + fixed
            + gather_peak.max(failure_peak).max(invoke_peak)
            + scope;

        let mut run = |registry_bytes, admit: bool| {
            let mut registry = DomEventDispatcher::<HeldPath>::new(DomEventRegistryLimits {
                registrations: 1,
                pending_events: 1,
                nested_dispatches: 1,
                storage_bytes: registry_bytes,
                work: 10_000,
            })
            .unwrap();
            let id = registry
                .add_listener(
                    &mut dom,
                    target,
                    DomEventType::CLOSE,
                    DomEventCallback::new(|_, _, held: &mut HeldPath, event| {
                        *held = Some(event.composed_path());
                        Err(DomEventCallbackError { code: 19 })
                    }),
                    DomEventListenerOptions::default(),
                )
                .unwrap();
            let mut invocation = registry
                .prepare_dispatch(&mut dom, target, &event, &cancel)
                .unwrap();
            let mut held = None;
            let result =
                invocation.advance(&mut dom, &mut registry, &mut held, dispatch_limits, &cancel);
            let storage = registry.storage.clone();
            if !admit {
                assert!(matches!(
                    result,
                    Err(DomEventDispatchError::Admission(DomEventResource::Storage))
                ));
                assert!(held.is_none());
                assert_eq!(event.target(), None);
                assert_eq!(
                    storage.counter().unwrap().bytes.get(),
                    header + registration + index + fixed
                );
                return None;
            }
            let DomEventDispatchOutcome::Complete(receipt) = result.unwrap() else {
                panic!("actual ordinary dispatch must complete");
            };
            assert_eq!(receipt.callbacks(), 1);
            assert_eq!(
                receipt.failures(),
                [DomEventListenerFailure {
                    registration: id,
                    current_target: DomEventTarget::Node(target),
                    kind: DomEventListenerFailureKind::Reported(DomEventCallbackError { code: 19 }),
                }]
            );
            drop(invocation);
            drop(registry);
            Some((storage, receipt, held.unwrap()))
        };

        assert!(run(peak - 1, false).is_none());
        let (storage, receipt, mut escaped) = run(peak, true).unwrap();
        let backing = escaped.path.as_ref().unwrap();
        assert_eq!(backing.items.capacity(), 4);
        let PathFunding::Owned {
            _retained: path_receipt,
        } = &backing._funding
        else {
            panic!("ordinary path owns its canonical backing");
        };
        assert_eq!(path_receipt.retained.retained_bytes(), path);
        let Some(DispatchFunding(Funding::Owned(failure_receipt))) = &receipt.funding else {
            panic!("ordinary completion owns its canonical failure backing");
        };
        assert_eq!(failure_receipt.retained.retained_bytes(), failure);
        assert_eq!(
            storage.counter().unwrap().bytes.get(),
            header + fixed + path + failure
        );
        drop(event);
        assert_eq!(
            storage.counter().unwrap().bytes.get(),
            header + path + failure
        );
        drop(receipt);
        assert_eq!(storage.counter().unwrap().bytes.get(), header + path);
        assert_eq!(
            (&mut escaped).collect::<Vec<_>>(),
            [
                target.into(),
                html.into(),
                top.document().node().into(),
                top.global().into()
            ]
        );
        drop(escaped);
        assert_eq!(storage.counter().unwrap().bytes.get(), header);
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    ))]
    #[test]
    fn standalone_identity_is_charged_until_its_last_observation() {
        use super::super::{
            DomEventConstructionError, DomEventDispatcher, DomEventRegistryLimits, DomEventTarget,
            tests::hosted_style_parent_tests,
        };

        let cancel = AtomicBool::new(false);
        let mut session = hosted_style_parent_tests::hosted_session();
        let mut hosted = session.hosted_execution();
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let global = hosted
            .create_top_level(&mut dom, host, &cancel)
            .unwrap()
            .global();
        let registry = |storage_bytes| {
            DomEventDispatcher::<()>::new(DomEventRegistryLimits {
                registrations: 0,
                pending_events: 0,
                nested_dispatches: 0,
                storage_bytes,
                work: 100,
            })
            .unwrap()
        };
        let probe = registry(16_000_000);
        let actual = {
            let target = hosted
                .create_event_target(&mut dom, &probe, global, &cancel)
                .unwrap();
            target.inspect_allocations().0
        };
        let header = size_of::<StorageCounter>();
        assert_eq!(probe.storage.counter().unwrap().bytes.get(), header);
        drop(probe);

        let short = registry(header + actual - 1);
        assert!(matches!(
            hosted.create_event_target(&mut dom, &short, global, &cancel),
            Err(DomEventConstructionError::Event(
                DomEventDispatchError::Admission(DomEventResource::Storage)
            ))
        ));
        assert_eq!(short.storage.counter().unwrap().bytes.get(), header);

        let exact = registry(header + actual);
        let target = hosted
            .create_event_target(&mut dom, &exact, global, &cancel)
            .unwrap();
        let (bytes, backing) = target.inspect_allocations();
        assert_eq!(bytes, actual);
        assert_eq!(backing.retained.retained_bytes(), actual);
        let escaped = DomEventTarget::Standalone(target.clone());
        drop(target);
        let storage = exact.storage.clone();
        drop(exact);
        assert_eq!(storage.counter().unwrap().bytes.get(), header + actual);
        drop(escaped);
        assert_eq!(storage.counter().unwrap().bytes.get(), header);
    }
}

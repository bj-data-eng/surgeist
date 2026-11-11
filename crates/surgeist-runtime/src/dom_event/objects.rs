use super::*;
use surgeist_dom::{DomHostedGlobalId, DomHostedSourceId};

/// Independent flags accepted by the Event constructor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomEventInit {
    /// Invoke eligible ancestor listeners during the bubbling pass.
    pub bubbles: bool,
    /// Permit the Event's canceled flag to be set.
    pub cancelable: bool,
    /// Cross the originating ShadowRoot's host boundary.
    pub composed: bool,
}

/// Original authenticated Global and source-domain moments for one Event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomEventCreationObservation {
    pub(crate) global: DomHostedGlobalId,
    pub(crate) source: DomHostedSourceId,
    pub(crate) origin_microseconds: u64,
    pub(crate) occurrence_microseconds: u64,
}
impl DomEventCreationObservation {
    /// Actual Global selected when this Event was constructed.
    pub fn global(&self) -> DomHostedGlobalId {
        self.global
    }
    /// Native monotonic clock domain shared by that Global's session.
    pub fn clock_source(&self) -> DomHostedSourceId {
        self.source
    }
    /// Original navigation-start origin, coarsened in its source domain.
    pub fn time_origin_microseconds(&self) -> u64 {
        self.origin_microseconds
    }
    /// Actual occurrence moment, independently coarsened in the same domain.
    pub fn occurrence_microseconds(&self) -> u64 {
        self.occurrence_microseconds
    }
    /// Signed duration from origin to occurrence, preserving earlier moments.
    pub fn relative_microseconds(&self) -> i128 {
        i128::from(self.occurrence_microseconds) - i128::from(self.origin_microseconds)
    }
    /// The signed coarse duration projected into the standard millisecond unit.
    pub fn milliseconds(&self) -> f64 {
        self.relative_microseconds() as f64 / 1000.0
    }
}

/// Why a selected producer cannot supply this Event's creation timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomEventTimestampUnavailable {
    /// The event has no available authenticated creation clock.
    NoAuthenticatedClock,
    /// The original authenticated clock refused its one firing observation.
    Provider(surgeist_dom::DomSourceEventClockFailure),
}
impl std::fmt::Display for DomEventTimestampUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use surgeist_dom::DomSourceEventClockFailure as Failure;
        f.write_str(match self {
            Self::NoAuthenticatedClock => "authenticated Event creation clock unavailable",
            Self::Provider(Failure::Unavailable) => "original Event creation clock unavailable",
            Self::Provider(Failure::Regressed) => "original Event creation clock regressed",
            Self::Provider(Failure::Overflow) => "original Event creation clock overflowed",
        })
    }
}
impl std::error::Error for DomEventTimestampUnavailable {}

/// Event and EventTarget construction preserves the actual hosted or storage refusal.
#[derive(Debug)]
pub enum DomEventConstructionError {
    /// Actual creation Global, Source owner or native clock refused observation.
    Hosted(crate::DomHostedRuntimeError),
    /// Event or target storage, or exact type construction was refused.
    Event(DomEventDispatchError),
    /// The legacy constructor supports only the base Event interface aliases.
    UnsupportedInterface,
    /// The receiving Document has no native relevant Global.
    MissingRelevantGlobal,
}
impl std::fmt::Display for DomEventConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hosted(error) => error.fmt(f),
            Self::Event(error) => error.fmt(f),
            Self::UnsupportedInterface => f.write_str("unsupported legacy Event interface"),
            Self::MissingRelevantGlobal => f.write_str("Document has no native relevant Global"),
        }
    }
}
impl std::error::Error for DomEventConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Hosted(error) => Some(error),
            Self::Event(error) => Some(error),
            Self::UnsupportedInterface | Self::MissingRelevantGlobal => None,
        }
    }
}
impl From<DomEventDispatchError> for DomEventConstructionError {
    fn from(error: DomEventDispatchError) -> Self {
        Self::Event(error)
    }
}
impl From<crate::DomHostedRuntimeError> for DomEventConstructionError {
    fn from(error: crate::DomHostedRuntimeError) -> Self {
        Self::Hosted(error)
    }
}

struct InitializedEvent {
    event_type: DomEventType,
    init: DomEventInit,
}

impl NativeDomEvent {
    pub(crate) fn authored(
        dom: &mut DomEventDom<'_>,
        storage: EventStorage,
        creation: crate::dom_lifecycle::EventCreation,
        event_type: &[u16],
        init: DomEventInit,
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        storage.step(1, cancel)?;
        let event_type = DomEventType::retain(&storage, dom, event_type, cancel)?;
        Self::allocate(
            dom,
            storage,
            creation,
            Some(InitializedEvent { event_type, init }),
            cancel,
        )
    }

    pub(crate) fn legacy(
        dom: &mut DomEventDom<'_>,
        storage: EventStorage,
        creation: crate::dom_lifecycle::EventCreation,
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        storage.step(1, cancel)?;
        Self::allocate(dom, storage, creation, None, cancel)
    }

    fn allocate(
        dom: &mut DomEventDom<'_>,
        storage: EventStorage,
        creation: crate::dom_lifecycle::EventCreation,
        initialized: Option<InitializedEvent>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomEventDispatchError> {
        let bytes = EventBodyCustody::event_bytes();
        let allocation = storage.reserve(dom, bytes, bytes, 0, cancel)?;
        let backing = Rc::new(BodyCustody {
            _storage: BodyStorage::Owned {
                _retained: allocation,
            },
        });
        let (event_type, init) = initialized.map_or_else(
            || (None, DomEventInit::default()),
            |event| (Some(event.event_type), event.init),
        );
        Ok(Self(
            Rc::new(Event {
                owner: dom.owner(),
                event_type: RefCell::new(event_type),
                init: Cell::new(init),
                trusted: Cell::new(false),
                creation: Ok(creation.into_observation()),
                toggle: None,
                storage,
                target: RefCell::new(None),
                stop: Cell::new(false),
                immediate: Cell::new(false),
                canceled: Cell::new(false),
                continuation: Cell::new(None),
            }),
            backing,
        ))
    }

    /// Actual original creation observation. Existing selected sources can lack
    /// this capability independently of their valid delivery authority.
    pub fn creation(&self) -> Option<&DomEventCreationObservation> {
        self.0.creation.as_ref().ok()
    }
    /// Original signed coarse relative time in milliseconds.
    pub fn time_stamp(&self) -> Result<f64, DomEventTimestampUnavailable> {
        self.0
            .creation
            .as_ref()
            .map(DomEventCreationObservation::milliseconds)
            .map_err(|error| *error)
    }
    /// Actual construction trust, reset by ordinary dispatch and initialization.
    pub fn is_trusted(&self) -> bool {
        self.0.trusted.get()
    }
    /// Legacy alias of the current shadow-adjusted target.
    pub fn src_element(&self) -> Option<DomEventTarget> {
        self.target()
    }
    /// Legacy observation of the stop-propagation flag.
    pub fn cancel_bubble(&self) -> bool {
        self.0.stop.get()
    }
    /// Setting true stops propagation. Setting false preserves an existing stop.
    pub fn set_cancel_bubble(&self, value: bool) {
        if value {
            self.stop_propagation();
        }
    }
    /// Legacy inverse of the sticky canceled flag.
    pub fn return_value(&self) -> bool {
        !self.default_prevented()
    }
    /// Setting false requests cancellation. Setting true preserves cancellation.
    pub fn set_return_value(&self, value: bool) {
        if !value {
            self.prevent_default();
        }
    }

    /// Legacy initialization is inert during an active invocation. Successful
    /// idle initialization preserves composed and the original creation clock.
    pub fn init_event(
        &self,
        dom: &mut DomEventDom<'_>,
        event_type: &[u16],
        bubbles: bool,
        cancelable: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomEventDispatchError> {
        if self.is_dispatching() {
            return Ok(());
        }
        if dom.owner() != self.0.owner {
            return Err(DomError::ForeignOwner.into());
        }
        let event_type = DomEventType::retain(&self.0.storage, dom, event_type, cancel)?;
        let composed = self.0.init.get().composed;
        self.0.init.set(DomEventInit {
            bubbles,
            cancelable,
            composed,
        });
        self.0.trusted.set(false);
        self.0.target.replace(None);
        self.0.stop.set(false);
        self.0.immediate.set(false);
        self.0.canceled.set(false);
        self.0.event_type.replace(Some(event_type));
        Ok(())
    }

    pub(crate) fn check_dispatch_state(&self) -> Result<(), DomEventDispatchError> {
        if self.is_dispatching() {
            return Err(DomEventDispatchError::AlreadyDispatching);
        }
        if self.0.event_type.borrow().is_none() {
            return Err(DomEventDispatchError::Uninitialized);
        }
        Ok(())
    }

    pub(crate) fn is_dispatching(&self) -> bool {
        self.continuation()
            .is_some_and(|invocation| invocation.lifetime.get() != InvocationLifetime::Idle)
    }
}

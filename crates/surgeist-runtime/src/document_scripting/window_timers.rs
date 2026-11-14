//! Native Window timer tasks for the callable branch of HTML timers.

use std::time::Duration;

use surgeist_bindings_js::{
    Arguments, Exception, FunctionMetadata, IntegerKind, IntegerMode, IntegerValue,
    RetainedCallbackObject, Value,
};
use surgeist_dom::DomHostedSourceId;

use super::*;
use crate::dom_lifecycle::WindowTimerMoment;

/// Finite limits for the attached Window's callable timer profile.
#[derive(Clone, Copy, Debug)]
pub struct WindowTimerLimits {
    pending: usize,
    arguments: u16,
    storage_bytes: usize,
}

impl WindowTimerLimits {
    /// Bound simultaneous registrations, extra arguments per timer and live
    /// adapter backing. Storage is reserved from the attachment's overall
    /// limit; native Ledger receipts cover the actual backing in use.
    pub const fn new(pending: usize, arguments: u16, storage_bytes: usize) -> Self {
        Self {
            pending,
            arguments,
            storage_bytes,
        }
    }

    pub(super) fn reserved_storage(self) -> Option<usize> {
        self.storage_bytes.checked_add(WindowTimers::fixed_bytes())
    }
}

/// Result of servicing at most one task in the attached Window.
#[derive(Debug)]
pub enum WindowTimerProgress {
    /// There are no pending timer tasks.
    Idle,
    /// No task is due at the native observation. The duration is measured
    /// from that observation, in the Window's own monotonic clock.
    Waiting(Duration),
    /// The exact timer completed and the microtask queue was checkpointed.
    Fired {
        /// Positive ID returned by the original registration.
        id: i32,
        /// The original ordinary thrown value, if any.
        exception: Option<Exception>,
    },
    /// The timer callback completed before its microtask checkpoint refused.
    CheckpointRefused {
        /// ID of the consumed timer.
        id: i32,
        /// Original ordinary thrown value, if the callback threw.
        exception: Option<Exception>,
        /// Original checkpoint refusal.
        error: DocumentScriptError,
    },
}

struct Backing {
    used: Cell<usize>,
    _canonical: DomHostedRuntimeRetention,
}

struct Debit {
    bytes: usize,
    backing: Rc<Backing>,
    _canonical: DomHostedRuntimeRetention,
}

impl Drop for Debit {
    fn drop(&mut self) {
        self.backing.used.set(self.backing.used.get() - self.bytes);
    }
}

struct Registration {
    next: Option<Box<Registration>>,
    id: i32,
    source: DomHostedSourceId,
    deadline: Duration,
    nesting: u8,
    // Private dense array holding the exact callback and supplied arguments.
    call: RetainedCallbackObject,
    arguments: usize,
    _backing: Debit,
}

#[derive(Default)]
struct Queue {
    first: Option<Box<Registration>>,
    count: usize,
    last_id: i32,
}

impl Queue {
    fn insertion_point(
        &mut self,
        deadline: Duration,
        mut step: impl FnMut() -> Result<(), DocumentScriptNativeError>,
    ) -> Result<&mut Option<Box<Registration>>, DocumentScriptNativeError> {
        let mut link = &mut self.first;
        while let Some(record) = link.as_ref() {
            step()?;
            // A new ID follows all existing timers with the same deadline.
            if deadline < record.deadline {
                break;
            }
            match link {
                Some(record) => link = &mut record.next,
                None => break,
            }
        }
        Ok(link)
    }

    fn remove(
        &mut self,
        id: i32,
        mut step: impl FnMut() -> Result<(), DocumentScriptNativeError>,
    ) -> Result<Option<Box<Registration>>, DocumentScriptNativeError> {
        let mut link = &mut self.first;
        loop {
            let Some(record) = link.as_ref() else {
                return Ok(None);
            };
            step()?;
            if record.id == id {
                let Some(mut removed) = link.take() else {
                    return Err(DocumentScriptNativeError::Revoked);
                };
                *link = removed.next.take();
                self.count -= 1;
                return Ok(Some(removed));
            }
            let Some(record) = link.as_mut() else {
                return Err(DocumentScriptNativeError::Revoked);
            };
            link = &mut record.next;
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        while let Some(mut record) = self.first.take() {
            self.first = record.next.take();
        }
    }
}

pub(super) struct WindowTimers {
    limits: WindowTimerLimits,
    queue: RefCell<Queue>,
    nesting: Cell<u8>,
    backing: Rc<Backing>,
}

struct Capture {
    method: Method,
    timers: Weak<WindowTimers>,
    native: Weak<NativeOwner>,
    _backing: Rc<Backing>,
}

#[derive(Clone, Copy)]
enum Method {
    Set,
    Clear,
}

impl WindowTimers {
    pub(super) const fn fixed_bytes() -> usize {
        size_of::<Self>() + size_of::<Backing>() + 2 * size_of::<Capture>()
    }

    pub(super) fn new(limits: WindowTimerLimits, backing: DomHostedRuntimeRetention) -> Rc<Self> {
        Rc::new(Self {
            limits,
            queue: RefCell::new(Queue::default()),
            nesting: Cell::new(0),
            backing: Rc::new(Backing {
                used: Cell::new(0),
                _canonical: backing,
            }),
        })
    }

    pub(super) fn install(
        self: &Rc<Self>,
        owner: &Rc<NativeOwner>,
        scope: &mut Scope<'_>,
    ) -> Result<(), Error> {
        let global = scope.global()?;
        for (method, name, length, maximum_arguments) in [
            (
                Method::Set,
                &b"setTimeout".map(u16::from)[..],
                1,
                // Capture one extra to refuse an unsupported arity explicitly.
                u32::from(self.limits.arguments) + 3,
            ),
            (Method::Clear, &b"clearTimeout".map(u16::from)[..], 0, 1),
        ] {
            let capture = Capture {
                method,
                timers: Rc::downgrade(self),
                native: Rc::downgrade(owner),
                _backing: self.backing.clone(),
            };
            let callback = Callback::new(move |scope, args| capture.invoke(scope, args));
            let function = scope.bounded_function(
                &callback,
                FunctionMetadata { name, length },
                maximum_arguments,
            )?;
            scope.define_new_property(
                &global,
                PropertyKey::String(name),
                PropertyDescriptor::Data {
                    value: &function,
                    writable: true,
                    enumerable: true,
                    configurable: true,
                },
            )?;
        }
        Ok(())
    }

    fn reserve(
        &self,
        owner: &NativeOwner,
        bytes: usize,
    ) -> Result<Debit, DocumentScriptNativeError> {
        let total = self
            .backing
            .used
            .get()
            .checked_add(bytes)
            .filter(|total| *total <= self.limits.storage_bytes)
            .ok_or(DocumentScriptNativeError::StorageLimit)?;
        let mut session = owner
            .session
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let canonical = session.hosted_execution().retain_script_payload(
            &mut session.dom(),
            bytes,
            &owner.cancellation,
        )?;
        self.backing.used.set(total);
        Ok(Debit {
            bytes,
            backing: self.backing.clone(),
            _canonical: canonical,
        })
    }

    fn schedule<'scope>(
        &self,
        owner: &NativeOwner,
        attachment: &Attachment,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, DocumentScriptError> {
        charge(owner, arguments.len().max(1))?;
        let Some(handler) = arguments.get(0) else {
            return type_error(scope, &b"setTimeout requires a callable".map(u16::from));
        };
        if !scope.is_callable(handler)? {
            return type_error(
                scope,
                &b"Only callable Window timers are supported".map(u16::from),
            );
        }
        if scope
            .callback_object(handler)?
            .associated_realm()
            .identity()
            != attachment.realm.identity()
        {
            return Err(DocumentScriptNativeError::ForeignPermit.into());
        }
        // Delay conversion can run guest code and change the queue. Hold no
        // native loan or queue borrow across it.
        let delay = long(scope, arguments.get(1))?;
        scope.check_active()?;
        let count = arguments.len().saturating_sub(2);
        if count > usize::from(self.limits.arguments) {
            return Err(DocumentScriptNativeError::RepresentationExhausted.into());
        }

        let nesting = self.nesting.get();
        let delay = if nesting > 5 {
            delay.max(4)
        } else {
            delay.max(0)
        };
        let moment = sample(owner, attachment)?;
        let deadline = moment
            .elapsed()
            .checked_add(Duration::from_millis(
                u64::try_from(delay)
                    .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?,
            ))
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let id = {
            let queue = self
                .queue
                .try_borrow()
                .map_err(|_| DocumentScriptNativeError::Busy)?;
            if queue.count >= self.limits.pending {
                return Err(DocumentScriptNativeError::StorageLimit.into());
            }
            queue
                .last_id
                .checked_add(1)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?
        };
        let result = scope.primitive(Primitive::Number(f64::from(id)))?;
        let mut queue = self
            .queue
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let insertion = queue.insertion_point(deadline, || charge(owner, 1))?;
        let record_backing = self.reserve(owner, size_of::<Registration>())?;
        let (_, mut values) = arguments.into_parts();
        if values.len() > 1 {
            values.remove(1);
        }
        let array = scope.array(&values)?;
        drop(values);
        let call = scope.callback_object(&array)?;
        let call = scope.retain_callback_object(call, |bytes| {
            self.reserve(owner, bytes)
                .map_err(DocumentScriptError::from)
        })?;
        let next = insertion.take();
        *insertion = Some(Box::new(Registration {
            next,
            id,
            source: moment.source(),
            deadline,
            nesting: nesting.saturating_add(1),
            call,
            arguments: count,
            _backing: record_backing,
        }));
        queue.count += 1;
        queue.last_id = id;
        Ok(Completion::Return(result))
    }

    fn clear<'scope>(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, DocumentScriptError> {
        charge(owner, 1)?;
        let id = long(scope, arguments.get(0))?;
        scope.check_active()?;
        if id <= 0 {
            return Ok(Completion::Undefined);
        }
        let record = self
            .queue
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .remove(id, || charge(owner, 1))?;
        if let Some(record) = record {
            scope.release_callback_object(record.call)?;
        }
        Ok(Completion::Undefined)
    }

    pub(super) fn discard(&self, process: &mut Process) {
        // Terminal cleanup cannot depend on remaining work or guest entry.
        if let Ok(mut queue) = self.queue.try_borrow_mut() {
            while let Some(mut record) = queue.first.take() {
                queue.first = record.next.take();
                let _ = process.release_callback_object(record.call);
            }
            queue.count = 0;
        }
    }
}

impl Capture {
    fn invoke<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let owner = self.native.upgrade().ok_or(Error::ExpiredIdentity)?;
        let timers = self.timers.upgrade().ok_or(Error::ExpiredIdentity)?;
        let attachment = owner.attachment.get().ok_or(Error::ExpiredIdentity)?;
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&b"Not a Window object".map(u16::from))?,
            ));
        }
        if scope.realm_authority()?.identity() != attachment.realm.identity() {
            return refuse(
                scope,
                &attachment.failures,
                DocumentScriptNativeError::ForeignPermit,
            );
        }
        let result = match self.method {
            Method::Set => timers.schedule(&owner, attachment, scope, arguments),
            Method::Clear => timers.clear(&owner, scope, arguments),
        };
        match result {
            Ok(completion) => Ok(completion),
            Err(DocumentScriptError::Script(error)) => Err(error),
            Err(DocumentScriptError::Native(error)) => refuse(scope, &attachment.failures, error),
        }
    }
}

fn sample(
    owner: &NativeOwner,
    attachment: &Attachment,
) -> Result<WindowTimerMoment, DocumentScriptNativeError> {
    let mut session = owner
        .session
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let hosted = session.hosted_execution();
    Ok(hosted.sample_window_timer(
        &session.dom(),
        attachment.target.global(),
        &owner.cancellation,
    )?)
}

fn charge(owner: &NativeOwner, units: usize) -> Result<(), DocumentScriptNativeError> {
    let mut session = owner
        .session
        .try_borrow_mut()
        .map_err(|_| DocumentScriptNativeError::Busy)?;
    let hosted = session.hosted_execution();
    Ok(hosted.admit_script_work(&session.dom(), units, &owner.cancellation)?)
}

fn long<'scope>(scope: &mut Scope<'scope>, value: Option<&Value<'scope>>) -> Result<i32, Error> {
    let Some(value) = value else {
        return Ok(0);
    };
    match scope.integer(value, IntegerKind::Long, IntegerMode::Wrap)? {
        IntegerValue::Long(value) => Ok(value),
        _ => Err(Error::IntegerOutOfRange),
    }
}

fn type_error<'scope>(
    scope: &mut Scope<'scope>,
    message: &[u16],
) -> Result<Completion<'scope>, DocumentScriptError> {
    Ok(Completion::Throw(scope.type_error(message)?))
}

#[derive(Clone, Copy)]
struct Due {
    id: i32,
    source: DomHostedSourceId,
    deadline: Duration,
    arguments: usize,
}

impl WindowTimers {
    fn earliest(&self, owner: &NativeOwner) -> Result<Option<Due>, DocumentScriptNativeError> {
        let queue = self
            .queue
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?;
        let Some(record) = &queue.first else {
            return Ok(None);
        };
        charge(owner, 1)?;
        Ok(Some(Due {
            id: record.id,
            source: record.source,
            deadline: record.deadline,
            arguments: record.arguments,
        }))
    }

    fn fire(
        &self,
        owner: &NativeOwner,
        scope: &mut Scope<'_>,
        due: Due,
    ) -> Result<Option<Exception>, DocumentScriptError> {
        charge(owner, due.arguments.max(1))?;
        let arguments_bytes = due
            .arguments
            .checked_mul(size_of::<Value<'_>>())
            .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        let temporary = self.reserve(owner, arguments_bytes)?;
        let mut args = Vec::new();
        args.try_reserve_exact(due.arguments)
            .map_err(|_| Error::Allocation)?;

        let record = self
            .queue
            .try_borrow_mut()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .remove(due.id, || charge(owner, 1))?
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let values = scope.open_callback_object(&record.call);
        let released = scope.release_callback_object(record.call);
        let values = values?;
        released?;
        let callback = element(scope, &values, 0)?;
        for index in 1..=due.arguments {
            let index = u32::try_from(index)
                .map_err(|_| DocumentScriptNativeError::RepresentationExhausted)?;
            args.push(element(scope, &values, index)?);
        }
        let receiver = scope.global()?;
        let nesting = Nesting {
            depth: &self.nesting,
            previous: self.nesting.replace(record.nesting),
        };
        let result = scope.call(&callback, &receiver, &args);
        drop(nesting);
        drop(args);
        drop(temporary);
        match result {
            Ok(_) => Ok(None),
            Err(Error::Exception(thrown)) => Ok(Some(thrown)),
            Err(error) => Err(error.into()),
        }
    }
}

struct Nesting<'a> {
    depth: &'a Cell<u8>,
    previous: u8,
}

impl Drop for Nesting<'_> {
    fn drop(&mut self) {
        self.depth.set(self.previous);
    }
}

fn element<'scope>(
    scope: &mut Scope<'scope>,
    array: &Value<'scope>,
    mut index: u32,
) -> Result<Value<'scope>, Error> {
    let mut units = [0_u16; 10];
    let mut digits = 0;
    for unit in units.iter_mut().rev() {
        *unit = u16::try_from(index % 10).map_err(|_| Error::ArrayTooLong)? + u16::from(b'0');
        digits += 1;
        index /= 10;
        if index == 0 {
            break;
        }
    }
    let key = units
        .get(units.len() - digits..)
        .ok_or(Error::ArrayTooLong)?;
    scope.get(array, PropertyKey::String(key))
}

impl DocumentScriptHost {
    /// Observe the attached Window's native clock and service at most one due
    /// callback, then run the microtask checkpoint for that task. The host
    /// supplies task opportunities. This call never waits for a future timer.
    pub fn service_window_timer(
        &mut self,
        permit: &DocumentScriptPermit,
    ) -> Result<WindowTimerProgress, DocumentScriptError> {
        self.check_permit(permit)?;
        self.observe()?;
        let attachment = self
            .native
            .attachment
            .get()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let engine = self
            .engine
            .as_mut()
            .ok_or(DocumentScriptNativeError::Revoked)?;
        let timers = engine
            .window_timers
            .as_ref()
            .ok_or(DocumentScriptNativeError::WindowTimersDisabled)?;
        let Some(due) = timers.earliest(&self.native)? else {
            return Ok(WindowTimerProgress::Idle);
        };
        let now = sample(&self.native, attachment)?;
        if due.source != now.source() {
            return Err(DocumentScriptNativeError::Hosted(DomHostedRuntimeError::Protocol).into());
        }
        if let Some(wait) = due.deadline.checked_sub(now.elapsed())
            && !wait.is_zero()
        {
            return Ok(WindowTimerProgress::Waiting(wait));
        }
        let result = engine.process.execution().in_realm(&engine.realm, |scope| {
            Ok(timers.fire(&self.native, scope, due))
        });
        engine.settle_engine_cleanup(&attachment.realm);
        let exception = result??;
        match self.checkpoint() {
            Ok(()) => Ok(WindowTimerProgress::Fired {
                id: due.id,
                exception,
            }),
            Err(error) => Ok(WindowTimerProgress::CheckpointRefused {
                id: due.id,
                exception,
                error,
            }),
        }
    }
}

#[cfg(test)]
pub(super) mod tests;

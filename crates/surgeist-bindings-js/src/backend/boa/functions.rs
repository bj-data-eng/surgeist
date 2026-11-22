use super::{Entered, Handle, Operations, State};

use std::any::Any as CallbackPanicPayload;
use std::cell::{Cell as CallbackCell, RefCell as CallbackRefCell};
use std::collections::BTreeMap as CallbackMap;
use std::marker::PhantomData as CallbackMarker;
use std::panic::{AssertUnwindSafe as CallbackUnwind, catch_unwind as catch_callback_unwind};
use std::rc::{Rc as CallbackRc, Weak as CallbackWeak};

use boa_engine::{
    Context as CallbackContext, JsError as BoaJsError, JsObject as BoaJsObject,
    JsResult as BoaJsResult, JsString as BoaJsString, JsValue as BoaJsValue,
    NativeFunction as BoaNativeFunction,
    error::PanicError as BoaPanicError,
    object::{FunctionObjectBuilder as BoaFunctionBuilder, WeakJsObject},
    property::PropertyDescriptor as BoaPropertyDescriptor,
};

use crate::preparation::{
    ArgumentValues as CallbackValues, Guard as CallbackGuard, Request as CallbackRequest,
};

pub(super) enum BoaCallable {
    Function {
        callback: crate::Callback,
        maximum_arguments: Option<u32>,
    },
    Constructor(crate::ConstructorCallback),
}

struct BoaCallableLease {
    realm: crate::RealmKey,
    body: BoaCallable,
}

struct BoaCallableEntry {
    retired: CallbackRc<CallbackCell<bool>>,
    lease: CallbackRc<BoaCallableLease>,
    constructor: Option<WeakJsObject>,
    _backing: Option<CallbackGuard>,
}

struct BoaCallbackFrame {
    next: Option<Box<Self>>,
    payload: Option<Box<dyn CallbackPanicPayload + Send>>,
    _backing: Option<CallbackGuard>,
}

#[derive(Default)]
struct BoaCallbackFrames(CallbackCell<Option<Box<BoaCallbackFrame>>>);

impl BoaCallbackFrames {
    fn push(&self, mut frame: Box<BoaCallbackFrame>) {
        frame.next = self.0.take();
        self.0.set(Some(frame));
    }

    fn drain(&self) {
        while let Some(mut frame) = self.0.take() {
            self.0.set(frame.next.take());
            drop(frame);
        }
    }
}

impl Drop for BoaCallbackFrames {
    fn drop(&mut self) {
        self.drain();
    }
}

pub(super) struct BoaCallables {
    next: CallbackCell<u64>,
    entries: CallbackRefCell<CallbackMap<u64, BoaCallableEntry>>,
    emergency: CallbackCell<Option<Box<BoaCallbackFrame>>>,
    returned_frames: BoaCallbackFrames,
    panic_frames: BoaCallbackFrames,
}

impl BoaCallables {
    pub(super) const EMERGENCY_BYTES: usize = size_of::<BoaCallbackFrame>();

    pub(super) fn new() -> Self {
        Self {
            next: CallbackCell::new(0),
            entries: CallbackRefCell::new(CallbackMap::new()),
            emergency: CallbackCell::new(Some(Box::new(BoaCallbackFrame {
                next: None,
                payload: None,
                _backing: None,
            }))),
            returned_frames: BoaCallbackFrames::default(),
            panic_frames: BoaCallbackFrames::default(),
        }
    }

    pub(super) fn is_constructor(
        &self,
        object: &BoaJsObject,
        realm: crate::RealmKey,
    ) -> Result<bool, crate::Error> {
        Ok(self
            .entries
            .try_borrow()
            .map_err(|_| crate::Error::RegistryConflict)?
            .values()
            .any(|entry| {
                entry.lease.realm == realm
                    && entry
                        .constructor
                        .as_ref()
                        .and_then(WeakJsObject::upgrade)
                        .as_ref()
                        == Some(object)
            }))
    }

    pub(super) fn reap(&self) -> Result<(), crate::Error> {
        loop {
            let retired = self
                .entries
                .try_borrow_mut()
                .map_err(|_| crate::Error::RegistryConflict)?
                .extract_if(.., |_, entry| entry.retired.get())
                .next();
            let Some((_, entry)) = retired else {
                break;
            };
            drop(entry);
        }
        self.returned_frames.drain();
        Ok(())
    }

    pub(super) fn drain(&self) -> Result<(), crate::Error> {
        let entries = std::mem::take(
            &mut *self
                .entries
                .try_borrow_mut()
                .map_err(|_| crate::Error::RegistryConflict)?,
        );
        drop(entries);
        self.returned_frames.drain();
        self.panic_frames.drain();
        Ok(())
    }
}

// GC owns only the registration token. A weak Rust pointer and a boolean
// signal need no tracing, and finalization only sets the signal. Callback
// bodies and release guards stay Rust-owned until an outer boundary reaps them.
#[derive(boa_gc::Trace)]
struct BoaCallableToken {
    id: u64,
    #[unsafe_ignore_trace]
    state: CallbackWeak<State>,
    #[unsafe_ignore_trace]
    retired: CallbackRc<CallbackCell<bool>>,
}

impl boa_gc::Finalize for BoaCallableToken {
    fn finalize(&self) {
        self.retired.set(true);
    }
}

impl Entered<'_> {
    pub(super) fn register_boa_callable(
        &mut self,
        body: BoaCallable,
        metadata: Option<crate::FunctionMetadata<'_>>,
    ) -> Result<crate::Root, crate::Error> {
        self.check_active()?;
        let state = self.state.clone();
        let id = state
            .callables
            .next
            .get()
            .checked_add(1)
            .ok_or(crate::Error::IdentityExhausted)?;
        state.callables.next.set(id);
        let backing = admit_boa_callback(
            &state,
            CallbackRequest::CallableRegistration {
                bytes: size_of::<BoaCallableLease>()
                    + size_of::<CallbackCell<bool>>()
                    + size_of::<BoaCallableToken>()
                    + size_of::<(u64, BoaCallableEntry)>(),
            },
        )?;
        let retired = CallbackRc::new(CallbackCell::new(false));
        let native = BoaNativeFunction::from_copy_closure_with_captures(
            invoke_boa_callable,
            BoaCallableToken {
                id,
                state: CallbackRc::downgrade(&state),
                retired: retired.clone(),
            },
        );
        let is_constructor = matches!(body, BoaCallable::Constructor(_));
        let root = self.attempt_value(crate::ExceptionPhase::Coerce, |context| {
            let builder =
                BoaFunctionBuilder::new(context.realm(), native).constructor(is_constructor);
            let builder = match metadata {
                Some(metadata) => builder
                    .name(BoaJsString::from(metadata.name))
                    .length(metadata.length as usize),
                None => builder,
            };
            let function = builder.build();
            if is_constructor {
                let prototype = BoaJsObject::with_object_proto(context.intrinsics());
                prototype.define_property_or_throw(
                    boa_engine::js_string!("constructor"),
                    BoaPropertyDescriptor::builder()
                        .value(function.clone())
                        .writable(true)
                        .enumerable(false)
                        .configurable(true),
                    context,
                )?;
                function.define_property_or_throw(
                    boa_engine::js_string!("prototype"),
                    BoaPropertyDescriptor::builder()
                        .value(prototype)
                        .writable(true)
                        .enumerable(false)
                        .configurable(false),
                    context,
                )?;
            }
            Ok(function.into())
        })?;
        let constructor = if is_constructor {
            Some(self.object_value(&root)?.downgrade())
        } else {
            None
        };
        state
            .callables
            .entries
            .try_borrow_mut()
            .map_err(|_| state.owner.fail(crate::Error::RegistryConflict))?
            .insert(
                id,
                BoaCallableEntry {
                    retired,
                    constructor,
                    lease: CallbackRc::new(BoaCallableLease {
                        realm: self.realm,
                        body,
                    }),
                    _backing: backing,
                },
            );
        Ok(root)
    }

    fn retain_boa_callback_input<'scope>(
        &self,
        value: BoaJsValue,
    ) -> Result<crate::Value<'scope>, crate::Error> {
        let backing = admit_boa_callback(
            &self.state,
            CallbackRequest::IncomingRoot {
                bytes: State::root_storage_bytes(),
            },
        )?;
        let root = self
            .state
            .retain_funded(self.realm, Handle::Value(value), backing)?;
        Ok(crate::Value {
            root,
            invariant: CallbackMarker,
        })
    }

    fn retain_boa_callback_arguments<'scope>(
        &self,
        args: &[BoaJsValue],
        maximum_arguments: Option<u32>,
    ) -> Result<CallbackValues<'scope>, crate::Error> {
        let count = maximum_arguments.map_or(args.len(), |limit| args.len().min(limit as usize));
        let bytes = count
            .checked_mul(size_of::<crate::Value<'_>>())
            .ok_or(crate::Error::Allocation)?;
        let backing = if count == 0 {
            None
        } else {
            admit_boa_callback(&self.state, CallbackRequest::IncomingBuffer { bytes })?
        };
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| crate::Error::Allocation)?;
        for value in args.iter().take(count) {
            values.push(self.retain_boa_callback_input(value.clone())?);
        }
        Ok(CallbackValues { values, backing })
    }
}

fn admit_boa_callback(
    state: &State,
    request: CallbackRequest,
) -> Result<Option<CallbackGuard>, crate::Error> {
    state
        .owner
        .preparation
        .get()
        .map(|account| account.admit(request))
        .transpose()
}

fn boa_host_abort() -> BoaJsError {
    // This uncatchable marker only stops Boa. OwnerState retains the cause.
    BoaPanicError::new("").into()
}

fn invoke_boa_callable(
    receiver_or_target: &BoaJsValue,
    args: &[BoaJsValue],
    token: &BoaCallableToken,
    context: &mut CallbackContext,
) -> BoaJsResult<BoaJsValue> {
    let state = token.state.upgrade().ok_or_else(boa_host_abort)?;
    state.owner.available().map_err(|_| boa_host_abort())?;
    let Some(mut frame) = state.callables.emergency.take() else {
        state.owner.fail(crate::Error::RegistryConflict);
        return Err(boa_host_abort());
    };
    let mut admitted = false;
    let result = catch_callback_unwind(CallbackUnwind(|| {
        let backing = admit_boa_callback(
            &state,
            CallbackRequest::CallbackFrame {
                bytes: size_of::<BoaCallbackFrame>(),
            },
        );
        let result = backing.and_then(|backing| {
            state.callables.emergency.set(Some(std::mem::replace(
                &mut frame,
                Box::new(BoaCallbackFrame {
                    next: None,
                    payload: None,
                    _backing: backing,
                }),
            )));
            admitted = true;
            run_boa_callable(&state, token, receiver_or_target, args, context)
        });
        match result {
            Ok(value) => Ok(value),
            Err(crate::Error::Exception(exception)) => match state.value(exception.value()) {
                Ok(value) => Err(BoaJsError::from_opaque(value)),
                Err(error) => {
                    state.owner.fail(error);
                    Err(boa_host_abort())
                }
            },
            Err(error) => {
                state.owner.fail(error);
                Err(boa_host_abort())
            }
        }
    }));
    match result {
        Ok(result) => {
            if admitted {
                state.callables.returned_frames.push(frame);
            } else {
                state.callables.emergency.set(Some(frame));
            }
            result
        }
        Err(payload) => {
            frame.payload = Some(payload);
            state.callables.panic_frames.push(frame);
            if let Err(error) = state
                .owner
                .finalize::<()>(Err(crate::Error::CallbackPanicked))
            {
                state.owner.fail(error);
            }
            Err(boa_host_abort())
        }
    }
}

fn run_boa_callable(
    state: &CallbackRc<State>,
    token: &BoaCallableToken,
    receiver_or_target: &BoaJsValue,
    args: &[BoaJsValue],
    context: &mut CallbackContext,
) -> Result<BoaJsValue, crate::Error> {
    let boundary = crate::owner::Boundary::enter(state.owner.clone())?;
    let result = (|| {
        let lease = state
            .callables
            .entries
            .try_borrow()
            .map_err(|_| crate::Error::RegistryConflict)?
            .get(&token.id)
            .filter(|entry| !entry.retired.get())
            .map(|entry| entry.lease.clone())
            .ok_or(crate::Error::ExpiredIdentity)?;
        let realm = state.current_key(context)?;
        if realm != lease.realm {
            return Err(crate::Error::ForeignOwner);
        }
        let mut entered = Entered {
            context,
            state: state.clone(),
            realm,
        };
        match &lease.body {
            BoaCallable::Function {
                callback,
                maximum_arguments,
            } => {
                let global = entered.context.global_object();
                let receiver = if receiver_or_target.is_null_or_undefined()
                    || receiver_or_target
                        .as_object()
                        .is_some_and(|receiver| receiver == global)
                {
                    // Global environment record access can reach the inner
                    // Window. Native interface code observes its WindowProxy.
                    entered.context.global_this()
                } else {
                    // Boa ToObject only refuses null or undefined, handled above.
                    receiver_or_target
                        .to_object(entered.context)
                        .map_err(|_| crate::Error::BackendUnavailable)?
                };
                let receiver = entered.retain_boa_callback_input(receiver.into())?;
                let values = entered.retain_boa_callback_arguments(args, *maximum_arguments)?;
                match (callback.0)(
                    &mut crate::Scope {
                        operations: &mut entered,
                        invariant: CallbackMarker,
                    },
                    crate::Arguments { receiver, values },
                )? {
                    crate::Completion::Undefined => Ok(BoaJsValue::undefined()),
                    crate::Completion::Return(value) => state.value(&value.root),
                    crate::Completion::Throw(value) => Err(boa_callback_throw(value.root)),
                }
            }
            BoaCallable::Constructor(callback) => {
                // Boa passes undefined on ordinary calls to native constructors.
                // Only [[Construct]] supplies its original NewTarget here.
                if receiver_or_target.is_undefined() {
                    let error =
                        entered.type_error(&b"Constructor requires 'new'".map(u16::from))?;
                    return Err(boa_callback_throw(error));
                }
                let values =
                    entered.retain_boa_callback_arguments(args, callback.0.argument_limit)?;
                let new_target = entered.retain_boa_callback_input(receiver_or_target.clone())?;
                match (callback.0.action)(
                    &mut crate::Scope {
                        operations: &mut entered,
                        invariant: CallbackMarker,
                    },
                    crate::ConstructorArguments {
                        values,
                        new_target,
                        fallback: callback.0.fallback.clone(),
                    },
                )? {
                    crate::ConstructorCompletion::Return(value) => {
                        let value = state.value(&value.root)?;
                        if value.is_object() {
                            Ok(value)
                        } else {
                            Err(crate::Error::ExpectedObject)
                        }
                    }
                    crate::ConstructorCompletion::Throw(value) => {
                        Err(boa_callback_throw(value.root))
                    }
                }
            }
        }
    })();
    boundary.finish(result)
}

fn boa_callback_throw(value: crate::Root) -> crate::Error {
    crate::Error::Exception(crate::Exception {
        phase: crate::ExceptionPhase::Invoke,
        value,
        rule: None,
    })
}

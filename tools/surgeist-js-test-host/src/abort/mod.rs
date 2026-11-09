//! `AbortController` and `AbortSignal` Web API implementations.

use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use surgeist_js::class::Class;
use surgeist_js::gc::heap::Trace as HeapTrace;
use surgeist_js::job::GenericJob;
use surgeist_js::native_function::{CapturedObject, CapturedValue, NativeCapture};
use surgeist_js::object::builtins::JsFunction;
use surgeist_js::realm::Realm;
use surgeist_js::{
    Context, ContextAccess, JsData, JsError, JsNativeError, JsObject, JsResult, JsString, JsValue,
    boa_class, boa_module, js_error, js_string,
};

#[cfg(test)]
mod tests;

/// Cancellation token for cooperative abort.
#[derive(Debug, Clone)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    /// Cancel the token.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Returns `true` if cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

fn make_abort_error(mut context: ContextAccess<'_>) -> JsValue {
    let obj = JsNativeError::error()
        .with_message("signal is aborted without reason")
        .into_opaque(context.reborrow())
        .expect("the abort error contains only local data");
    obj.set(
        js_string!("name"),
        js_string!("AbortError"),
        false,
        context.reborrow(),
    )
    .ok();
    obj.into()
}

/// The JavaScript `AbortSignal` class.
#[derive(Debug, Clone, JsData, HeapTrace)]
pub struct JsAbortSignal {
    #[heap_trace(skip)]
    aborted: Cell<bool>,
    reason: RefCell<Option<CapturedValue>>,
    listeners: RefCell<Vec<AbortEventListener>>,
    #[heap_trace(skip)]
    cancel_token: CancellationToken,
}

#[derive(Debug, Clone, HeapTrace)]
struct AbortEventListener {
    event_type: JsString,
    callback: CapturedObject,
}

impl Default for JsAbortSignal {
    fn default() -> Self {
        Self {
            aborted: Cell::new(false),
            reason: RefCell::default(),
            listeners: RefCell::default(),
            cancel_token: CancellationToken::new(),
        }
    }
}

impl JsAbortSignal {
    /// # Errors
    ///
    /// Returns the first listener publication error after completing the
    /// committed abort and attempting every listener publication.
    pub fn signal_abort(&self, reason: JsValue, mut context: ContextAccess<'_>) -> JsResult<()> {
        if self.aborted.get() {
            return Ok(());
        }
        self.aborted.set(true);
        *self.reason.borrow_mut() = Some(reason.store());

        let abort = js_string!("abort");
        let listeners: Vec<JsFunction> = self
            .listeners
            .borrow()
            .iter()
            .filter(|listener| listener.event_type == abort)
            .map(|listener| JsFunction::restore(&listener.callback))
            .collect();

        let realm = context.realm().clone();
        let mut publication_error = None;
        for listener in listeners {
            let publication = context.enqueue_job(
                GenericJob::new(
                    move |context| {
                        listener.call(&JsValue::undefined(), &[], context)?;
                        Ok(JsValue::undefined())
                    },
                    realm.clone(),
                )
                .into(),
            );
            let result = publication
                .map_err(surgeist_js::job::JobPublicationError::into_error)
                .and_then(surgeist_js::job::JobAcceptance::into_result);
            if let Err(error) = result {
                if publication_error.is_none() {
                    publication_error = Some(error);
                }
            }
        }

        self.cancel_token.cancel();

        publication_error.map_or(Ok(()), Err)
    }

    /// Returns `true` if this signal has been aborted.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.aborted.get()
    }

    /// Returns the abort reason.
    pub fn abort_reason(&self, mut context: ContextAccess<'_>) -> JsValue {
        if !self.aborted.get() {
            return JsValue::undefined();
        }
        let reason = self.reason.borrow().as_ref().map(JsValue::restore);
        reason.unwrap_or_else(|| make_abort_error(context.reborrow()))
    }

    /// Returns the cancellation token.
    #[must_use]
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}

#[boa_class(rename = "AbortSignal")]
#[boa(rename_all = "camelCase")]
impl JsAbortSignal {
    #[boa(constructor)]
    fn constructor() -> JsResult<Self> {
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }

    #[boa(getter)]
    fn aborted(&self) -> bool {
        self.aborted.get()
    }

    #[boa(getter)]
    fn reason(&self, mut context: ContextAccess<'_>) -> JsValue {
        self.abort_reason(context.reborrow())
    }

    fn throw_if_aborted(&self, mut context: ContextAccess<'_>) -> JsResult<()> {
        if self.aborted.get() {
            Err(JsError::from_opaque(self.abort_reason(context.reborrow())))
        } else {
            Ok(())
        }
    }

    fn add_event_listener(
        &self,
        event_type: JsString,
        callback: JsFunction,
        _context: ContextAccess<'_>,
    ) {
        {
            let listeners = self.listeners.borrow();
            if listeners.iter().any(|listener| {
                listener.event_type == event_type
                    && JsObject::equals(&JsFunction::restore(&listener.callback), &callback)
            }) {
                return;
            }
        }
        self.listeners.borrow_mut().push(AbortEventListener {
            event_type,
            callback: callback.store(),
        });
    }

    fn remove_event_listener(&self, event_type: JsString, callback: JsFunction) {
        self.listeners.borrow_mut().retain(|listener| {
            listener.event_type != event_type
                || !JsObject::equals(&JsFunction::restore(&listener.callback), &callback)
        });
    }
}

/// The JavaScript `AbortController` class.
#[derive(Debug, Clone, JsData, HeapTrace)]
pub struct JsAbortController {
    signal: CapturedObject,
}

#[boa_class(rename = "AbortController")]
#[boa(rename_all = "camelCase")]
impl JsAbortController {
    #[boa(constructor)]
    fn constructor(mut context: ContextAccess<'_>) -> JsResult<Self> {
        let signal_obj = Class::from_data(JsAbortSignal::default(), context.reborrow())?;
        Ok(Self {
            signal: signal_obj.store(),
        })
    }

    #[boa(getter)]
    fn signal(&self) -> JsObject {
        JsObject::restore(&self.signal)
    }

    fn abort(&self, reason: Option<JsValue>, mut context: ContextAccess<'_>) -> JsResult<()> {
        let abort_reason = reason.unwrap_or_else(|| make_abort_error(context.reborrow()));

        let signal_obj = self.signal();
        let Some(signal) = signal_obj.downcast_ref::<JsAbortSignal>() else {
            return Err(js_error!(TypeError: "AbortController: invalid signal object"));
        };
        signal.signal_abort(abort_reason, context.reborrow())
    }
}

/// `AbortController` and `AbortSignal` module.
#[boa_module]
pub mod js_module {
    type JsAbortController = super::JsAbortController;
    type JsAbortSignal = super::JsAbortSignal;
}

/// # Errors
/// Returns an error if registration fails.
pub fn register(realm: Option<Realm>, context: &mut Context) -> JsResult<()> {
    js_module::boa_register(realm, context.access())
}

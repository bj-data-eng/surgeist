use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use bus::BusReader;
#[cfg(feature = "annex-b")]
use surgeist_js::builtins::is_html_dda::IsHTMLDDA;
use surgeist_js::{
    Context, ContextAccess, JsArgs, JsNativeError, JsResult, JsValue, Source,
    builtins::array_buffer::SharedArrayBuffer,
    gc::heap::Trace,
    js_string,
    native_function::{NativeCapture, NativeFunction},
    object::{
        JsObject, ObjectInitializer,
        builtins::{JsArrayBuffer, JsSharedArrayBuffer},
    },
    property::Attribute,
};

/// Result of a worker thread execution.
#[derive(Debug)]
pub enum WorkerResult {
    /// The worker completed successfully.
    Ok,
    /// The worker returned an error.
    Err(String),
    /// The worker panicked.
    Panic(String),
}

type WorkerHandle = JoinHandle<Result<(), String>>;

/// Handles for worker threads spawned by `$262.agent.start()`.
#[derive(Debug, Clone)]
pub struct WorkerHandles(Rc<RefCell<Vec<WorkerHandle>>>);

impl Default for WorkerHandles {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerHandles {
    /// Creates a new empty set of worker handles.
    #[must_use]
    pub fn new() -> Self {
        Self(Rc::default())
    }

    /// Joins all worker threads and returns their results.
    #[allow(clippy::print_stderr)]
    pub fn join_all(&mut self) -> Vec<WorkerResult> {
        let handles = std::mem::take(&mut *self.0.borrow_mut());

        handles
            .into_iter()
            .map(|h| {
                let result = h.join();

                match result {
                    Ok(Ok(())) => WorkerResult::Ok,
                    Ok(Err(msg)) => {
                        eprintln!("Detected error on worker thread: {msg}");
                        WorkerResult::Err(msg)
                    }
                    Err(e) => {
                        let msg = e
                            .downcast_ref::<&str>()
                            .map(|&s| String::from(s))
                            .unwrap_or_default();
                        eprintln!("Detected panic on worker thread: {msg}");
                        WorkerResult::Panic(msg)
                    }
                }
            })
            .collect()
    }
}

impl Drop for WorkerHandles {
    fn drop(&mut self) {
        self.join_all();
    }
}

/// Creates the object `$262` in the context.
///
/// # Panics
///
/// Panics if any of the expected global properties cannot be defined.
pub fn register_js262(
    handles: WorkerHandles,
    console: bool,
    mut context: ContextAccess<'_>,
) -> JsResult<JsObject> {
    let global_obj = context.global_object();

    let agent = agent_obj(handles, console, context.reborrow())?;

    let js262 = ObjectInitializer::new(context.reborrow())
        .function(
            NativeFunction::from_fn_ptr(create_realm),
            js_string!("createRealm"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(detach_array_buffer),
            js_string!("detachArrayBuffer"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(eval_script),
            js_string!("evalScript"),
            1,
        )
        .function(NativeFunction::from_fn_ptr(gc), js_string!("gc"), 0)
        .property(
            js_string!("global"),
            global_obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .property(
            js_string!("agent"),
            agent,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .build()?;

    #[cfg(feature = "annex-b")]
    js262.create_data_property_or_throw(
        js_string!("IsHTMLDDA"),
        JsObject::from_proto_and_data(context.runtime(), None, IsHTMLDDA)?,
        context.reborrow(),
    )?;

    context.register_global_property(
        js_string!("$262"),
        js262.clone(),
        Attribute::WRITABLE | Attribute::CONFIGURABLE,
    )?;

    Ok(js262)
}

/// The `$262.createRealm()` function.
///
/// Creates a new ECMAScript Realm, defines this API on the new realm's global object, and
/// returns the `$262` property of the new realm's global object.
fn create_realm(_: &JsValue, _: &[JsValue], mut context: ContextAccess<'_>) -> JsResult<JsValue> {
    let mut realm = context.create_realm()?;

    realm = context.enter_realm(realm)?;
    let js262 = register_js262(WorkerHandles::new(), false, context.reborrow());
    context.enter_realm(realm)?;

    js262.map(JsValue::from)
}

/// The `$262.detachArrayBuffer()` function.
///
/// Implements the `DetachArrayBuffer` abstract operation.
fn detach_array_buffer(_: &JsValue, args: &[JsValue], _: ContextAccess<'_>) -> JsResult<JsValue> {
    fn type_err() -> JsNativeError {
        JsNativeError::typ().with_message("The provided object was not an ArrayBuffer")
    }

    // 1. Assert: IsSharedArrayBuffer(arrayBuffer) is false.
    let object = args
        .first()
        .and_then(JsValue::as_object)
        .ok_or_else(type_err)?;
    let array_buffer = JsArrayBuffer::from_object(object).map_err(|_| type_err())?;

    // 2. If key is not present, set key to undefined.
    let key = args.get_or_undefined(1);

    // 3. If SameValue(arrayBuffer.[[ArrayBufferDetachKey]], key) is false, throw a TypeError exception.
    // 4. Set arrayBuffer.[[ArrayBufferData]] to null.
    // 5. Set arrayBuffer.[[ArrayBufferByteLength]] to 0.
    array_buffer.detach_if_attached(key)?;

    // 6. Return NormalCompletion(null).
    Ok(JsValue::null())
}

/// The `$262.evalScript()` function.
///
/// Accepts a string value as its first argument and executes it as an ECMAScript script.
fn eval_script(
    _this: &JsValue,
    args: &[JsValue],
    mut context: ContextAccess<'_>,
) -> JsResult<JsValue> {
    args.first().and_then(JsValue::as_string).map_or_else(
        || Ok(JsValue::undefined()),
        |source_text| context.eval(Source::from_bytes(&source_text.to_std_string_escaped())),
    )
}

/// The `$262.gc()` function.
///
/// Wraps the host's garbage collection invocation mechanism, if such a capability exists.
/// Must throw an exception if no capability exists. This is necessary for testing the
/// semantics of any feature that relies on garbage collection, e.g. the `WeakRef` API.
fn gc(_this: &JsValue, _: &[JsValue], context: ContextAccess<'_>) -> JsResult<JsValue> {
    context
        .runtime()
        .collect()
        .map_err(|error| JsNativeError::error().with_message(error.to_string()))?;
    Ok(JsValue::undefined())
}

/// The `$262.agent.sleep()` function.
fn sleep(_: &JsValue, args: &[JsValue], mut context: ContextAccess<'_>) -> JsResult<JsValue> {
    let ms = args.get_or_undefined(0).to_number(context.reborrow())? / 1000.0;
    std::thread::sleep(Duration::from_secs_f64(ms));
    Ok(JsValue::undefined())
}

/// The `$262.agent.monotonicNow()` function.
#[allow(clippy::unnecessary_wraps)]
fn monotonic_now(_: &JsValue, _: &[JsValue], context: ContextAccess<'_>) -> JsResult<JsValue> {
    #[allow(clippy::cast_precision_loss)]
    Ok(JsValue::from(
        context.clock().now().millis_since_epoch() as f64
    ))
}

// Agent transports only carry shared byte buffers, UTF-16 reports, and
// native worker handles. They never hold objects from a JavaScript heap.
#[derive(Clone, Trace)]
struct AgentBus(#[heap_trace(skip)] Rc<RefCell<bus::Bus<SharedArrayBuffer>>>);

#[derive(Trace)]
struct AgentStart {
    bus: AgentBus,
    #[heap_trace(skip)]
    reports: Sender<Vec<u16>>,
    #[heap_trace(skip)]
    handles: WorkerHandles,
    console: bool,
}

#[derive(Trace)]
struct AgentReports(#[heap_trace(skip)] Receiver<Vec<u16>>);

#[derive(Trace)]
struct WorkerBroadcasts(#[heap_trace(skip)] RefCell<BusReader<SharedArrayBuffer>>);

#[derive(Trace)]
struct WorkerReports(#[heap_trace(skip)] Sender<Vec<u16>>);

#[derive(Trace)]
struct AgentCapture<T: Trace> {
    state: Rc<T>,
}

impl<T: Trace> AgentCapture<T> {
    fn new(state: T) -> Self {
        Self {
            state: Rc::new(state),
        }
    }
}

impl<T: Trace> NativeCapture for AgentCapture<T> {
    type Stored = Self;

    fn store(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }

    fn restore(stored: &Self) -> Result<Self, surgeist_js::error::RootingError> {
        Ok(stored.store())
    }
}

/// Initializes the `$262.agent` object in the main agent.
fn agent_obj(
    handles: WorkerHandles,
    console: bool,
    mut context: ContextAccess<'_>,
) -> JsResult<JsObject> {
    let bus = AgentBus(Rc::new(RefCell::new(bus::Bus::new(1))));

    let (reports_tx, reports_rx) = mpsc::channel();

    let start = NativeFunction::from_fn_with_captures(
        |_, args, state, mut context| {
            let state = &state.state;
            let script = args
                .get_or_undefined(0)
                .to_string(context.reborrow())?
                .to_std_string()
                .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;

            let rx = state.bus.0.borrow_mut().add_rx();
            let tx = state.reports.clone();
            let console = state.console;

            state
                .handles
                .0
                .borrow_mut()
                .push(std::thread::spawn(move || {
                    let context = &mut Context::builder()
                        .can_block(true)
                        .build()
                        .map_err(|e| e.to_string())?;
                    register_js262_worker(rx, tx, context.access()).map_err(|e| e.to_string())?;

                    if console {
                        let console =
                            crate::Console::init(context.access()).map_err(|e| e.to_string())?;

                        context
                            .register_global_property(
                                crate::Console::NAME,
                                console,
                                Attribute::all(),
                            )
                            .expect("the console builtin shouldn't exist");
                    }

                    let src = Source::from_bytes(&script);

                    context.eval(src).map_err(|e| e.to_string())?;
                    context.run_jobs().map_err(|e| e.to_string())?;

                    Ok(())
                }));

            Ok(JsValue::undefined())
        },
        AgentCapture::new(AgentStart {
            bus: bus.clone(),
            reports: reports_tx,
            handles,
            console,
        }),
    );

    // Test262 does not use the second numeric argument.
    let broadcast = NativeFunction::from_fn_with_captures(
        |_, args, bus, _| {
            let buffer = args.get_or_undefined(0).as_object().ok_or_else(|| {
                JsNativeError::typ().with_message("argument was not a shared array")
            })?;
            let buffer = buffer
                .downcast_ref::<SharedArrayBuffer>()?
                .ok_or_else(|| {
                    JsNativeError::typ().with_message("argument was not a shared array")
                })?
                .clone();

            bus.state.0.borrow_mut().broadcast(buffer);

            Ok(JsValue::undefined())
        },
        AgentCapture::new(bus),
    );

    let get_report = NativeFunction::from_fn_with_captures(
        |_, _, reports, _| {
            let Ok(msg) = reports.state.0.try_recv() else {
                return Ok(JsValue::null());
            };

            Ok(js_string!(&msg[..]).into())
        },
        AgentCapture::new(AgentReports(reports_rx)),
    );

    ObjectInitializer::new(context.reborrow())
        .function(start, js_string!("start"), 1)
        .function(broadcast, js_string!("broadcast"), 2)
        .function(get_report, js_string!("getReport"), 0)
        .function(NativeFunction::from_fn_ptr(sleep), js_string!("sleep"), 1)
        .function(
            NativeFunction::from_fn_ptr(monotonic_now),
            js_string!("monotonicNow"),
            0,
        )
        .build()
}

/// Initializes the `$262` object in a worker agent.
fn register_js262_worker(
    rx: BusReader<SharedArrayBuffer>,
    tx: Sender<Vec<u16>>,
    mut context: ContextAccess<'_>,
) -> JsResult<()> {
    // Test262 does not use the second numeric argument.
    let receive_broadcast = NativeFunction::from_fn_with_captures(
        |_, args, broadcasts, mut context| {
            let array = broadcasts.state.0.borrow_mut().recv().map_err(|err| {
                JsNativeError::typ().with_message(format!("failed to receive buffer: {err}"))
            })?;

            let callable = args
                .get_or_undefined(0)
                .as_callable()
                .ok_or_else(|| JsNativeError::typ().with_message("argument is not callable"))?;

            let buffer = JsSharedArrayBuffer::from_buffer(array, context.reborrow());
            callable.call(&JsValue::undefined(), &[buffer.into()], context.reborrow())
        },
        AgentCapture::new(WorkerBroadcasts(RefCell::new(rx))),
    );

    let report = NativeFunction::from_fn_with_captures(
        |_, args, reports, mut context| {
            let string = args
                .get_or_undefined(0)
                .to_string(context.reborrow())?
                .to_vec();
            reports
                .state
                .0
                .send(string)
                .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
            Ok(JsValue::undefined())
        },
        AgentCapture::new(WorkerReports(tx)),
    );

    let agent = ObjectInitializer::new(context.reborrow())
        .function(receive_broadcast, js_string!("receiveBroadcast"), 1)
        .function(report, js_string!("report"), 1)
        .function(NativeFunction::from_fn_ptr(sleep), js_string!("sleep"), 1)
        // Don't need to signal leaving, the main thread will join with the worker
        // threads anyways.
        .function(
            NativeFunction::from_fn_ptr(|_, _, _| Ok(JsValue::undefined())),
            js_string!("leaving"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(monotonic_now),
            js_string!("monotonicNow"),
            0,
        )
        .build()?;

    let js262 = ObjectInitializer::new(context.reborrow())
        .property(
            js_string!("agent"),
            agent,
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .build()?;

    context.register_global_property(
        js_string!("$262"),
        js262,
        Attribute::WRITABLE | Attribute::CONFIGURABLE,
    )
}

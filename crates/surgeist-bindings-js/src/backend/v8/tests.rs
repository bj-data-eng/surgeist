//! SDK/native lifetime qualification against the real production engine.
//!
//! The Counter is a private host fixture, not a production binding API. Its
//! observable mutation and drop receipts qualify the SDK operations required
//! by native bindings, while Engine owns contexts, guest roots and execution.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use super::Engine;
use crate::backend::{Engine as EngineApi, Operations};
use crate::{Error, ExceptionPhase, Realm};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NativeId(u32);

#[derive(Clone, Default)]
struct Observations {
    native_drops: Rc<Cell<usize>>,
    wrapper_finalizers: Rc<Cell<usize>>,
    host_drops: Rc<Cell<usize>>,
}

struct Counter {
    id: NativeId,
    value: Cell<i32>,
    observations: Observations,
}

impl Drop for Counter {
    fn drop(&mut self) {
        self.observations
            .native_drops
            .set(self.observations.native_drops.get() + 1);
    }
}

impl std::fmt::Display for NativeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

struct State {
    next_id: Cell<u32>,
    native: RefCell<BTreeMap<NativeId, Weak<Counter>>>,
    wrappers: RefCell<BTreeMap<NativeId, Rc<v8::Weak<v8::Object>>>>,
    callback: RefCell<Option<v8::Global<v8::Function>>>,
    rooted: RefCell<Option<crate::Root>>,
    realm: Realm,
    observations: Observations,
}

impl Drop for State {
    fn drop(&mut self) {
        self.observations
            .host_drops
            .set(self.observations.host_drops.get() + 1);
    }
}

struct Host {
    engine: Engine,
    realm: Realm,
}

impl Host {
    fn new(observations: Observations) -> Result<Self, String> {
        let mut engine = Engine::bootstrap().map_err(|error| error.to_string())?;
        let realm = engine.create_realm().map_err(|error| error.to_string())?;
        engine.isolate.set_slot(Rc::new(State {
            next_id: Cell::new(0),
            native: RefCell::default(),
            wrappers: RefCell::default(),
            callback: RefCell::default(),
            rooted: RefCell::default(),
            realm: realm.clone(),
            observations,
        }));
        let mut host = Self { engine, realm };
        let realm = host.realm.clone();
        host.install(&realm)?;
        Ok(host)
    }

    fn install(&mut self, realm: &Realm) -> Result<(), String> {
        self.engine
            .with_context(realm, |entered| {
                entered.attempt(
                    ExceptionPhase::PropertyWrite,
                    |scope, _roots, _realm, _intrinsics| {
                        let context = scope.get_current_context();
                        for (name, function) in [
                            ("getCounter", v8::Function::new(scope, get_counter)),
                            ("retainCallback", v8::Function::new(scope, retain_callback)),
                            ("fireCallback", v8::Function::new(scope, fire_callback)),
                            ("clearCallback", v8::Function::new(scope, clear_callback)),
                            ("retainRooted", v8::Function::new(scope, retain_rooted)),
                            ("rootedCall", v8::Function::new(scope, rooted_call)),
                            ("clearRooted", v8::Function::new(scope, clear_rooted)),
                        ] {
                            let key =
                                v8::String::new(scope, name).ok_or(Error::BackendUnavailable)?;
                            let value = function.ok_or(Error::BackendUnavailable)?;
                            if context.global(scope).set(scope, key.into(), value.into())
                                != Some(true)
                            {
                                return Err(Error::PropertyRejected);
                            }
                        }
                        Ok(())
                    },
                )
            })
            .map_err(|error| error.to_string())
    }

    fn state(&self) -> Result<Rc<State>, String> {
        self.engine
            .isolate
            .get_slot::<Rc<State>>()
            .cloned()
            .ok_or_else(|| "host is closed".into())
    }

    fn counter(&mut self, value: i32) -> Result<Rc<Counter>, String> {
        let state = self.state()?;
        let next = state
            .next_id
            .get()
            .checked_add(1)
            .ok_or("native identity exhausted")?;
        let id = NativeId(next);
        let counter = Rc::new(Counter {
            id,
            value: Cell::new(value),
            observations: state.observations.clone(),
        });
        state
            .native
            .try_borrow_mut()
            .map_err(|_| "native registry already borrowed")?
            .insert(id, Rc::downgrade(&counter));
        state.next_id.set(next);
        Ok(counter)
    }

    fn eval(&mut self, source: &str) -> Result<String, String> {
        let realm = self.realm.clone();
        self.eval_in(&realm, source)
    }

    fn eval_in(&mut self, realm: &Realm, source: &str) -> Result<String, String> {
        let source: Vec<_> = source.encode_utf16().collect();
        let result = self.engine.with_context(realm, |entered| {
            let root = entered.evaluate(&source)?;
            entered.to_utf16(&root)
        });
        let result = match result {
            Ok(units) => Ok(String::from_utf16_lossy(&units)),
            Err(Error::Exception(exception)) => {
                // Explicit diagnostic conversion belongs to this fixture. Core
                // exception capture itself never coerces the thrown value.
                let message = self
                    .engine
                    .with_context(realm, |entered| entered.to_utf16(exception.value()))
                    .map_err(|error| error.to_string());
                Err(message
                    .map(|units| String::from_utf16_lossy(&units))
                    .unwrap_or_else(|error| error))
            }
            Err(error) => Err(error.to_string()),
        };
        // Drop every transient Core root before asking the SDK to collect.
        self.engine.reap();
        result
    }

    fn collect(&mut self) -> Result<(), String> {
        self.engine.reap();
        self.engine
            .isolate
            .request_garbage_collection_for_testing(v8::GarbageCollectionType::Full);
        let state = self.state()?;
        state
            .wrappers
            .try_borrow_mut()
            .map_err(|_| "wrapper registry already borrowed")?
            .retain(|_, weak| !weak.is_empty());
        state
            .native
            .try_borrow_mut()
            .map_err(|_| "native registry already borrowed")?
            .retain(|_, native| native.strong_count() != 0);
        Ok(())
    }

    fn checkpoint(&mut self) -> Result<(), String> {
        self.engine.checkpoint().map_err(|error| error.to_string())
    }

    fn callback_is_live(&self) -> Result<bool, String> {
        Ok(self
            .state()?
            .callback
            .try_borrow()
            .map_err(|_| "callback already borrowed")?
            .is_some())
    }

    fn cache_len(&self) -> Result<usize, String> {
        Ok(self
            .state()?
            .wrappers
            .try_borrow()
            .map_err(|_| "wrapper registry already borrowed")?
            .len())
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        if let Some(state) = self.engine.isolate.remove_slot::<Rc<State>>() {
            // No callbacks are active at an Engine boundary. Drop typed slot,
            // Global and Weak handles before production Engine destruction.
            // No checkpoint is performed here: queued jobs remain abandoned.
            let mut state = Rc::try_unwrap(state)
                .unwrap_or_else(|_| panic!("host state escaped an engine boundary"));
            state.callback.get_mut().take();
            state.wrappers.get_mut().clear();
            state.native.get_mut().clear();
        }
    }
}

fn state(scope: &v8::PinScope) -> Result<Rc<State>, &'static str> {
    production_roots(scope)?;
    scope
        .get_slot::<Rc<State>>()
        .cloned()
        .ok_or("host is closed")
}

fn throw(scope: &mut v8::PinScope, message: &str) {
    // A sticky host failure cannot be translated to a catchable guest error.
    // Termination may still await a V8 interrupt check, so decline this entry.
    if production_roots(scope).is_err() {
        return;
    }
    if let Some(message) = v8::String::new(scope, message) {
        let exception = v8::Exception::type_error(scope, message);
        scope.throw_exception(exception);
    }
}

fn get_counter(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    match wrap_counter(scope, args.get(0)) {
        Ok(object) => result.set(object.into()),
        Err(message) => throw(scope, message),
    }
}

fn wrap_counter<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    id: v8::Local<v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, &'static str> {
    let id = NativeId(
        v8::Local::<v8::Uint32>::try_from(id)
            .map_err(|_| "expected a native identity")?
            .value(),
    );
    let state = state(scope)?;
    // Clone out of the table before any operation that may run the collector.
    let cached = state
        .wrappers
        .try_borrow()
        .map_err(|_| "wrapper registry already borrowed")?
        .get(&id)
        .cloned();
    if let Some(object) = cached.and_then(|weak| weak.to_local(scope)) {
        return Ok(object);
    }
    let native = state
        .native
        .try_borrow()
        .map_err(|_| "native registry already borrowed")?
        .get(&id)
        .and_then(Weak::upgrade)
        .ok_or("native object has expired")?;
    let template = v8::ObjectTemplate::new(scope);
    if !template.set_internal_field_count(1) {
        return Err("wrapper layout rejected");
    }
    let key = v8::String::new(scope, "value").ok_or("accessor name allocation")?;
    let getter = v8::FunctionTemplate::new(scope, value_getter);
    template.set_accessor_property(key.into(), Some(getter), None, v8::PropertyAttribute::NONE);
    let key = v8::String::new(scope, "add").ok_or("method name allocation")?;
    let method = v8::FunctionTemplate::new(scope, add);
    template.set(key.into(), method.into());
    let wrapper = template.new_instance(scope).ok_or("wrapper allocation")?;
    let identity = v8::Integer::new_from_unsigned(scope, id.0);
    if !wrapper.set_internal_field(0, identity.into()) {
        return Err("wrapper identity installation");
    }
    let observations = state.observations.clone();
    let weak = v8::Weak::with_guaranteed_finalizer(
        scope,
        wrapper,
        Box::new(move || {
            observations
                .wrapper_finalizers
                .set(observations.wrapper_finalizers.get() + 1);
            drop(native);
        }),
    );
    state
        .wrappers
        .try_borrow_mut()
        .map_err(|_| "wrapper registry already borrowed")?
        .insert(id, Rc::new(weak));
    Ok(wrapper)
}

fn receiver(
    scope: &mut v8::PinScope,
    object: v8::Local<v8::Object>,
) -> Result<Rc<Counter>, &'static str> {
    let field = object
        .get_internal_field(scope, 0)
        .ok_or("invalid Counter receiver")?;
    let value = v8::Local::<v8::Value>::try_from(field).map_err(|_| "invalid Counter identity")?;
    let id = NativeId(
        v8::Local::<v8::Uint32>::try_from(value)
            .map_err(|_| "invalid Counter identity")?
            .value(),
    );
    state(scope)?
        .native
        .try_borrow()
        .map_err(|_| "native registry already borrowed")?
        .get(&id)
        .and_then(Weak::upgrade)
        .ok_or("native object has expired")
}

fn value_getter(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    match receiver(scope, args.this()) {
        Ok(native) => result.set_int32(native.value.get()),
        Err(message) => throw(scope, message),
    }
}

fn add(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, mut result: v8::ReturnValue) {
    let native = match receiver(scope, args.this()) {
        Ok(native) => native,
        Err(message) => return throw(scope, message),
    };
    // Coercion can call JavaScript, including this same native object.
    // No registry borrow or mutable native borrow is held across it.
    let Some(delta) = args.get(0).int32_value(scope) else {
        return;
    };
    if production_roots(scope).is_err() {
        return;
    }
    let Some(value) = native.value.get().checked_add(delta) else {
        return throw(scope, "counter overflow");
    };
    native.value.set(value);
    result.set_int32(value);
}

fn retain_callback(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _result: v8::ReturnValue,
) {
    let callback = match v8::Local::<v8::Function>::try_from(args.get(0)) {
        Ok(callback) => callback,
        Err(_) => return throw(scope, "expected a callback"),
    };
    let state = match state(scope) {
        Ok(state) => state,
        Err(message) => return throw(scope, message),
    };
    let root = v8::Global::new(scope, callback);
    match state.callback.try_borrow_mut() {
        Ok(mut retained) => *retained = Some(root),
        Err(_) => throw(scope, "callback already borrowed"),
    };
}

fn fire_callback(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    let state = match state(scope) {
        Ok(state) => state,
        Err(message) => return throw(scope, message),
    };
    let callback = match state.callback.try_borrow() {
        Ok(retained) => retained.clone(),
        Err(_) => return throw(scope, "callback already borrowed"),
    };
    let Some(callback) = callback else {
        return throw(scope, "no retained callback");
    };
    let callback = v8::Local::new(scope, &callback);
    let this = v8::undefined(scope);
    if let Some(value) = callback.call(scope, this.into(), &[args.get(0)]) {
        result.set(value);
    }
}

fn clear_callback(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    _result: v8::ReturnValue,
) {
    match state(scope) {
        Ok(state) => {
            match state.callback.try_borrow_mut() {
                Ok(mut retained) => {
                    retained.take();
                }
                Err(_) => throw(scope, "callback already borrowed"),
            };
        }
        Err(message) => throw(scope, message),
    }
}

fn production_roots(scope: &v8::PinScope) -> Result<Rc<super::Roots>, &'static str> {
    let roots = scope
        .get_slot::<Weak<super::Roots>>()
        .and_then(Weak::upgrade)
        .ok_or("execution owner is closed")?;
    roots.available().map_err(|_| "execution owner failed")?;
    Ok(roots)
}

fn retain_rooted(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    _result: v8::ReturnValue,
) {
    if v8::Local::<v8::Function>::try_from(args.get(0)).is_err() {
        return throw(scope, "expected a function");
    }
    let state = match state(scope) {
        Ok(value) => value,
        Err(message) => return throw(scope, message),
    };
    let roots = match production_roots(scope) {
        Ok(value) => value,
        Err(message) => return throw(scope, message),
    };
    let root = match roots.retain(scope, state.realm.0.key, args.get(0)) {
        Ok(root) => root,
        Err(Error::RegistryConflict) => return,
        Err(_) => return throw(scope, "root retention failed"),
    };
    match state.rooted.try_borrow_mut() {
        Ok(mut retained) => *retained = Some(root),
        Err(_) => throw(scope, "callback slot is borrowed"),
    };
}

fn rooted_call(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut result: v8::ReturnValue,
) {
    let state = match state(scope) {
        Ok(value) => value,
        Err(message) => return throw(scope, message),
    };
    let root = match state.rooted.try_borrow() {
        Ok(retained) => retained.clone(),
        Err(_) => return throw(scope, "callback slot is borrowed"),
    };
    let Some(root) = root else {
        return throw(scope, "no rooted callback");
    };
    let roots = match production_roots(scope) {
        Ok(value) => value,
        Err(message) => return throw(scope, message),
    };
    let function = match roots.local(scope, &root) {
        Ok(value) => value,
        Err(Error::RegistryConflict) => return,
        Err(_) => return throw(scope, "rooted callback is unavailable"),
    };
    let function = match v8::Local::<v8::Function>::try_from(function) {
        Ok(value) => value,
        Err(_) => return throw(scope, "rooted value is not callable"),
    };
    let receiver = v8::undefined(scope);
    let Some(value) = function.call(scope, receiver.into(), &[args.get(0)]) else {
        return;
    };
    // Exercise the SAME production registry for nested read and result rooting.
    // No registry or callback-slot borrow crosses this invocation or collection.
    let returned = match roots.retain(scope, root.0.realm, value) {
        Ok(value) => value,
        Err(Error::RegistryConflict) => return,
        Err(_) => return throw(scope, "result root retention failed"),
    };
    match roots.local(scope, &returned) {
        Ok(value) => result.set(value),
        Err(Error::RegistryConflict) => (),
        Err(_) => throw(scope, "result root is unavailable"),
    }
}

fn clear_rooted(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    _result: v8::ReturnValue,
) {
    let state = match state(scope) {
        Ok(value) => value,
        Err(message) => return throw(scope, message),
    };
    match state.rooted.try_borrow_mut() {
        Ok(mut retained) => {
            retained.take();
        }
        Err(_) => throw(scope, "callback slot is borrowed"),
    };
}

// V8's process platform is pinned to its first entry thread. Keep every native
// qualification case on this one test thread, with sequential isolate owners.
#[test]
fn native_sdk_lifetimes_callbacks_and_teardown() -> TestResult {
    v8::V8::set_flags_from_string("--expose-gc");
    // Scalar validation exercises the declared embedding boundary without an
    // enormous element allocation or a fabricated slice.
    assert!(crate::backend::validate_array_length(16_777_216).is_ok());
    assert!(matches!(
        crate::backend::validate_array_length(16_777_217),
        Err(Error::ArrayTooLong)
    ));

    coerced_string_copy_is_exact_noncoercive_and_owner_checked()?;
    repeated_native_access_preserves_javascript_identity_and_native_mutation()?;
    live_javascript_wrapper_keeps_native_alive_and_garbage_collection_releases_it()?;
    collected_wrapper_can_be_recreated_for_a_still_owned_native()?;
    retained_callback_survives_scope_exit_and_gc_then_releases_captured_native()?;
    callback_can_reenter_bindings_and_clear_itself_without_a_borrow_panic()?;
    coercion_reentry_observes_the_current_native_value()?;
    invalid_receivers_and_callback_registration_fail_without_corrupting_state()?;
    callback_exception_reaches_the_host_and_later_execution_still_works()?;
    promise_callbacks_run_at_an_explicit_host_checkpoint()?;
    host_teardown_releases_roots_and_native_resources_after_script_failure()?;
    replacing_a_callback_releases_the_previous_closure_and_its_native_capture()?;
    expired_native_identity_is_rejected_after_wrapper_collection()?;
    inheriting_from_a_wrapper_does_not_grant_a_native_method_receiver()?;
    repeated_host_teardown_discards_queued_jobs_without_executing_them()?;
    distinct_realms_share_native_state_without_sharing_globals()?;
    invalid_native_identity_types_are_rejected()?;
    nested_callback_scopes_share_production_roots_and_release_self_cleared_custody()?;
    registry_failure_stays_terminal_across_guest_catches_and_native_coercion()?;
    cleanup_failure_uses_the_same_owner_and_preserves_the_first_refusal()?;
    Ok(())
}

fn cleanup_failure_uses_the_same_owner_and_preserves_the_first_refusal() -> TestResult {
    struct FailureOnDrop(Rc<crate::owner::OwnerState>);
    impl Drop for FailureOnDrop {
        fn drop(&mut self) {
            // The destructor owns no SDK handle or root registry. This is the
            // same pure owner transition that accounting cleanup will use.
            self.0.fail(Error::RegistryConflict);
        }
    }

    for source in [
        "try { cleanup() } catch (_) {} a.add(99)",
        "a.add({valueOf() { try { cleanup() } catch (_) {} return 99 }})",
    ] {
        let mut host = Host::new(Observations::default())?;
        let native = host.counter(0)?;
        host.eval(&format!(
            "globalThis.a = getCounter({}); a.add(1)",
            native.id
        ))?;
        let owner = host.engine.roots.owner.clone();
        let callback = crate::Callback::new(move |_, _| {
            drop(FailureOnDrop(owner.clone()));
            Ok(crate::Completion::Undefined)
        });
        host.engine.with_context(&host.realm, |entered| {
            let function = entered.function(&callback)?;
            let global = entered.global()?;
            entered.set(
                &global,
                crate::backend::PropertyKey::String(&"cleanup".encode_utf16().collect::<Vec<_>>()),
                &function,
            )
        })?;
        assert!(matches!(
            host.engine.with_context(&host.realm, |entered| entered
                .evaluate(&source.encode_utf16().collect::<Vec<_>>())),
            Err(Error::RegistryConflict)
        ));
        assert!(matches!(
            host.engine.create_realm(),
            Err(Error::RegistryConflict)
        ));
        assert!(matches!(
            host.engine.checkpoint(),
            Err(Error::RegistryConflict)
        ));
        host.engine.memory_pressure();
        assert_eq!(native.value.get(), 1);
    }

    {
        let mut host = Host::new(Observations::default())?;
        let native = host.counter(0)?;
        host.eval(&format!(
            "globalThis.a = getCounter({}); a.add(1)",
            native.id
        ))?;
        // A returned value can be destroyed after its entered scope ends.
        // The next host entry still observes its SDK-free cleanup failure.
        drop(FailureOnDrop(host.engine.roots.owner.clone()));
        assert!(matches!(
            host.engine.with_context(&host.realm, |entered| entered
                .evaluate(&"a.add(99)".encode_utf16().collect::<Vec<_>>())),
            Err(Error::RegistryConflict)
        ));
        assert_eq!(native.value.get(), 1);
    }

    let owner = {
        let mut host = Host::new(Observations::default())?;
        let owner = host.engine.roots.owner.clone();
        host.engine.roots.fail(Error::ExpectedObject);
        drop(FailureOnDrop(owner.clone()));
        assert!(matches!(
            host.engine.create_realm(),
            Err(Error::ExpectedObject)
        ));
        assert!(matches!(
            host.engine.checkpoint(),
            Err(Error::ExpectedObject)
        ));
        owner
    };
    // Closing the process preserves its original error even if an observer
    // temporarily retains the SDK-free state itself.
    assert!(matches!(owner.available(), Err(Error::ExpectedObject)));
    drop(owner);

    let owner = {
        let host = Host::new(Observations::default())?;
        host.engine.roots.owner.clone()
    };
    assert!(matches!(owner.available(), Err(Error::BackendUnavailable)));
    let weak = Rc::downgrade(&owner);
    drop(owner);
    assert!(weak.upgrade().is_none());
    Ok(())
}

fn coerced_string_copy_is_exact_noncoercive_and_owner_checked() -> TestResult {
    let original = [0, 0xd800, 0x78, 0xdc00, 0xd83d, 0xde00];
    let retained = {
        let mut host = Host::new(Observations::default())?;
        host.engine.with_context(&host.realm, |entered| {
            let input = entered.evaluate(
                &"globalThis.stringCoercions = 0; globalThis.stringInput = { [Symbol.toPrimitive](hint) { stringCoercions++; if (hint !== 'string') throw 1; return '\\0\\uD800x\\uDC00\\uD83D\\uDE00'; } }; stringInput"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?;
            let string = entered.coerce_string(&input)?;
            assert_eq!(string.len(), original.len());
            drop(input);
            entered.evaluate(
                &"stringInput[Symbol.toPrimitive] = () => { throw 2; }; stringInput = null; gc()"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?;
            // Caller-owned storage can be admitted after coercion and reused.
            // Neither copy consults the original object or performs ToString.
            for _ in 0..2 {
                let mut output = vec![0xffff; string.len()];
                entered.copy_string(&string, &mut output)?;
                assert_eq!(output, original);
            }
            let calls = entered.evaluate(&"stringCoercions".encode_utf16().collect::<Vec<_>>())?;
            assert_eq!(entered.to_utf16(&calls)?, [0x31]);
            let empty = entered.evaluate(&"''".encode_utf16().collect::<Vec<_>>())?;
            let empty = entered.coerce_string(&empty)?;
            assert_eq!(empty.len(), 0);
            entered.copy_string(&empty, &mut [])?;
            Ok(string)
        })?
    };

    let mut host = Host::new(Observations::default())?;
    host.engine.with_context(&host.realm, |entered| {
        let mut output = [0xffff; 6];
        assert!(matches!(
            entered.copy_string(&retained, &mut output),
            Err(Error::ForeignOwner)
        ));
        assert_eq!(output, [0xffff; 6]);
        Ok(())
    })?;
    Ok(())
}

fn nested_callback_scopes_share_production_roots_and_release_self_cleared_custody() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations.clone())?;
    let native = host.counter(0)?;
    let weak = Rc::downgrade(&native);
    host.eval(&format!("(() => {{ const a = getCounter({}); retainRooted(n => {{ if (n === 0) {{ clearRooted(); gc(); return a.add(3); }} return rootedCall(n - 1) + 1; }}); }})()", native.id))?;
    drop(native);
    host.collect()?;
    assert!(weak.upgrade().is_some());
    assert_eq!(host.eval("rootedCall(5)")?, "8");
    assert_eq!(
        weak.upgrade()
            .ok_or("live callback lost native custody")?
            .value
            .get(),
        3
    );
    host.collect()?;
    assert!(weak.upgrade().is_none());
    assert_eq!(observations.native_drops.get(), 1);
    Ok(())
}

fn registry_failure_stays_terminal_across_guest_catches_and_native_coercion() -> TestResult {
    // Qualify both a later native entry and an earlier entry whose coercion
    // encounters the failure. Neither may mutate after the sticky refusal.
    for source in [
        "try { rootedCall(0) } catch (_) {} a.add(1)",
        "a.add({ valueOf() { try { rootedCall(0) } catch (_) {} return 1; } })",
    ] {
        registry_failure_rejects_native_work(source)?;
    }
    Ok(())
}

fn registry_failure_rejects_native_work(source: &str) -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(0)?;
    host.eval("retainRooted(() => 7)")?;
    host.eval(&format!("globalThis.a = getCounter({})", native.id))?;
    let roots = host.engine.roots.clone();
    let held = roots.entries.try_borrow_mut()?;
    let source: Vec<_> = source.encode_utf16().collect();
    assert!(matches!(
        host.engine
            .with_context(&host.realm, |entered| entered.evaluate(&source)),
        Err(Error::RegistryConflict)
    ));
    assert!(matches!(
        host.engine.create_realm(),
        Err(Error::RegistryConflict)
    ));
    assert!(matches!(
        host.engine.checkpoint(),
        Err(Error::RegistryConflict)
    ));
    assert_eq!(native.value.get(), 0);
    drop(held);
    assert!(matches!(
        host.engine.with_context(&host.realm, |entered| entered
            .evaluate(&"42".encode_utf16().collect::<Vec<_>>())),
        Err(Error::RegistryConflict)
    ));
    Ok(())
}

fn repeated_native_access_preserves_javascript_identity_and_native_mutation() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations)?;
    let native = host.counter(10)?;
    let other = host.counter(20)?;
    assert_eq!(
        host.eval(&format!(
            "globalThis.a = getCounter({}); globalThis.b = getCounter({}); a === getCounter({}) && a !== b",
            native.id, other.id, native.id
        ))?,
        "true"
    );
    assert_eq!(host.eval("a.add(7); a.value")?, "17");
    assert_eq!(native.value.get(), 17);
    host.collect()?;
    assert_eq!(
        host.eval(&format!(
            "a === getCounter({}) && b === getCounter({}) && a !== b && a.value === 17",
            native.id, other.id
        ))?,
        "true"
    );
    native.value.set(23);
    assert_eq!(host.eval("a.value")?, "23");
    Ok(())
}

fn live_javascript_wrapper_keeps_native_alive_and_garbage_collection_releases_it() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations.clone())?;
    let native = host.counter(5)?;
    host.eval(&format!(
        "globalThis.a = getCounter({}); a.self = a",
        native.id
    ))?;
    let weak = Rc::downgrade(&native);
    drop(native);
    host.collect()?;
    assert!(weak.upgrade().is_some());
    assert_eq!(host.eval("a.add(2)")?, "7");
    assert_eq!(observations.native_drops.get(), 0);
    host.eval("a = null")?;
    host.collect()?;
    assert!(weak.upgrade().is_none());
    assert_eq!(observations.native_drops.get(), 1);
    assert_eq!(observations.wrapper_finalizers.get(), 1);
    assert_eq!(host.cache_len()?, 0);
    Ok(())
}

fn collected_wrapper_can_be_recreated_for_a_still_owned_native() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations.clone())?;
    let native = host.counter(12)?;
    host.eval(&format!("getCounter({}).value", native.id))?;
    host.collect()?;
    assert_eq!(observations.wrapper_finalizers.get(), 1);
    assert_eq!(observations.native_drops.get(), 0);
    assert_eq!(
        host.eval(&format!("getCounter({}).value", native.id))?,
        "12"
    );
    host.collect()?;
    assert_eq!(observations.wrapper_finalizers.get(), 2);
    Ok(())
}

fn retained_callback_survives_scope_exit_and_gc_then_releases_captured_native() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations.clone())?;
    let native = host.counter(3)?;
    let weak = Rc::downgrade(&native);
    host.eval(&format!(
        "(() => {{ const node = getCounter({}); retainCallback(delta => node.add(delta)); }})()",
        native.id
    ))?;
    drop(native);
    host.collect()?;
    assert!(host.callback_is_live()?);
    assert_eq!(host.eval("fireCallback(9)")?, "12");
    host.eval("clearCallback()")?;
    host.collect()?;
    assert!(!host.callback_is_live()?);
    assert!(weak.upgrade().is_none());
    assert_eq!(observations.native_drops.get(), 1);
    Ok(())
}

fn callback_can_reenter_bindings_and_clear_itself_without_a_borrow_panic() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(4)?;
    host.eval(&format!(
        "retainCallback(delta => {{ clearCallback(); return getCounter({}).add(delta); }})",
        native.id
    ))?;
    assert_eq!(host.eval("fireCallback(6)")?, "10");
    assert_eq!(native.value.get(), 10);
    assert!(!host.callback_is_live()?);
    Ok(())
}

fn coercion_reentry_observes_the_current_native_value() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(1)?;
    host.eval(&format!("globalThis.a = getCounter({})", native.id))?;
    assert_eq!(
        host.eval("a.add({ valueOf() { a.add(10); return 2; } })")?,
        "13"
    );
    assert_eq!(native.value.get(), 13);
    assert_eq!(host.eval("try { a.add({ valueOf() { throw new Error('coercion failed'); } }); false } catch (e) { e.message === 'coercion failed' && a.value === 13 }")?, "true");
    Ok(())
}

fn invalid_receivers_and_callback_registration_fail_without_corrupting_state() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(8)?;
    host.eval(&format!(
        "globalThis.a = getCounter({}); retainCallback(x => x + 1)",
        native.id
    ))?;
    assert_eq!(
        host.eval("try { a.add.call({}, 1); false } catch (e) { e instanceof TypeError }")?,
        "true"
    );
    assert_eq!(
        host.eval("try { retainCallback(42); false } catch (e) { e instanceof TypeError }")?,
        "true"
    );
    assert_eq!(host.eval("fireCallback(4)")?, "5");
    assert_eq!(host.eval("[null, undefined, 7, true, 'x'].every(receiver => { try { a.add.call(receiver, 1); return false; } catch (error) { return error instanceof TypeError; } })")?, "true");
    assert_eq!(native.value.get(), 8);
    Ok(())
}

fn callback_exception_reaches_the_host_and_later_execution_still_works() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    host.eval("retainCallback(() => { clearCallback(); throw new Error('callback failed'); })")?;
    let error = host
        .eval("fireCallback(0)")
        .err()
        .ok_or("expected callback error")?;
    assert!(error.contains("callback failed"));
    assert!(!host.callback_is_live()?);
    assert_eq!(host.eval("2 + 3")?, "5");
    Ok(())
}

fn promise_callbacks_run_at_an_explicit_host_checkpoint() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(2)?;
    host.eval(&format!(
        "globalThis.a = getCounter({}); Promise.resolve().then(() => a.add(5)); 'queued'",
        native.id
    ))?;
    assert_eq!(native.value.get(), 2);
    host.checkpoint()?;
    assert_eq!(native.value.get(), 7);
    assert_eq!(host.eval("a.value")?, "7");
    Ok(())
}

fn host_teardown_releases_roots_and_native_resources_after_script_failure() -> TestResult {
    let observations = Observations::default();
    let weak = {
        let mut host = Host::new(observations.clone())?;
        let native = host.counter(1)?;
        host.eval(&format!(
            "globalThis.a = getCounter({}); retainCallback(() => a.value)",
            native.id
        ))?;
        let weak = Rc::downgrade(&native);
        drop(native);
        assert!(host.eval("throw new Error('unfinished document')").is_err());
        assert!(weak.upgrade().is_some());
        weak
    };
    assert!(weak.upgrade().is_none());
    assert_eq!(observations.native_drops.get(), 1);
    assert_eq!(observations.host_drops.get(), 1);
    Ok(())
}

fn replacing_a_callback_releases_the_previous_closure_and_its_native_capture() -> TestResult {
    let observations = Observations::default();
    let mut host = Host::new(observations.clone())?;
    let native = host.counter(4)?;
    let weak = Rc::downgrade(&native);
    host.eval(&format!(
        "(() => {{ const a = getCounter({}); retainCallback(() => a.value); }})()",
        native.id
    ))?;
    drop(native);
    host.collect()?;
    assert!(weak.upgrade().is_some());
    host.eval("retainCallback(x => x * 2)")?;
    host.collect()?;
    assert!(weak.upgrade().is_none());
    assert_eq!(observations.native_drops.get(), 1);
    assert_eq!(host.eval("fireCallback(6)")?, "12");
    Ok(())
}

fn expired_native_identity_is_rejected_after_wrapper_collection() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(0)?;
    let id = native.id;
    host.eval(&format!("getCounter({id}).value"))?;
    drop(native);
    host.collect()?;
    assert_eq!(
        host.eval(&format!(
            "try {{ getCounter({id}); false }} catch (e) {{ e instanceof TypeError }}"
        ))?,
        "true"
    );
    Ok(())
}

fn inheriting_from_a_wrapper_does_not_grant_a_native_method_receiver() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(9)?;
    host.eval(&format!("globalThis.a = getCounter({})", native.id))?;
    assert_eq!(
        host.eval("try { Object.create(a).add(1); false } catch (e) { e instanceof TypeError }")?,
        "true"
    );
    assert_eq!(
        host.eval("try { Object.create(a).value; false } catch (e) { e instanceof TypeError }")?,
        "true"
    );
    assert_eq!(
        host.eval("try { Object.getOwnPropertyDescriptor(a, 'value').get.call({}); false } catch (e) { e instanceof TypeError }")?,
        "true"
    );
    assert_eq!(native.value.get(), 9);
    Ok(())
}

fn repeated_host_teardown_discards_queued_jobs_without_executing_them() -> TestResult {
    let observations = Observations::default();
    for count in 1..=12 {
        let native = {
            let mut host = Host::new(observations.clone())?;
            let native = host.counter(5)?;
            host.eval(&format!("globalThis.a = getCounter({}); Promise.resolve().then(() => a.add(1)); retainCallback(() => a.value)", native.id))?;
            assert_eq!(native.value.get(), 5);
            native
        };
        assert_eq!(native.value.get(), 5);
        assert_eq!(observations.host_drops.get(), count);
        drop(native);
        assert_eq!(observations.native_drops.get(), count);
    }
    Ok(())
}

fn distinct_realms_share_native_state_without_sharing_globals() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let first = host.realm.clone();
    let second = host.engine.create_realm()?;
    host.install(&second)?;
    let native = host.counter(6)?;
    host.eval_in(
        &first,
        &format!("globalThis.a = getCounter({}); a.add(1)", native.id),
    )?;
    assert_eq!(host.eval_in(&second, "typeof a")?, "undefined");
    assert_eq!(
        host.eval_in(&second, &format!("getCounter({}).add(2)", native.id))?,
        "9"
    );
    assert_eq!(host.eval_in(&first, "a.value")?, "9");
    assert_eq!(native.value.get(), 9);
    Ok(())
}

fn invalid_native_identity_types_are_rejected() -> TestResult {
    let mut host = Host::new(Observations::default())?;
    let native = host.counter(3)?;
    assert_eq!(host.eval("[-1, 1.5, 4294967296, '1', null, {}, undefined].every(id => { try { getCounter(id); return false; } catch (error) { return error instanceof TypeError; } })")?, "true");
    assert_eq!(host.eval(&format!("getCounter({}).value", native.id))?, "3");
    assert_eq!(native.value.get(), 3);
    Ok(())
}

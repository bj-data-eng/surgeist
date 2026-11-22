//! Safe JavaScript execution for native Surgeist bindings.
//!
//! [`Process`] owns one thread-affine engine. Its borrowed [`Execution`] creates
//! realms and enters a [`Scope`]; values leave that scope only through explicit
//! [`Scope::retain`]. Persistent roots contain identities, never engine handles.
//!
//! A scoped value cannot escape:
//! ```compile_fail
//! use surgeist_bindings_js::Process;
//! let mut process = Process::bootstrap().unwrap();
//! let mut execution = process.execution();
//! let realm = execution.create_realm().unwrap();
//! let escaped = execution.in_realm(&realm, |scope| scope.evaluate(&[49]));
//! ```
//!
//! There cannot be two simultaneous execution borrows:
//! ```compile_fail,E0499
//! use surgeist_bindings_js::Process;
//! let mut process = Process::bootstrap().unwrap();
//! let mut first = process.execution();
//! let mut second = process.execution();
//! first.create_realm().unwrap();
//! second.create_realm().unwrap();
//! ```
//!
//! An execution owner cannot move to a pre-existing thread:
//! ```compile_fail,E0277
//! use surgeist_bindings_js::Process;
//! let process = Process::bootstrap().unwrap();
//! std::thread::spawn(move || drop(process));
//! ```
//!
//! Scoped values from unrelated entry scopes cannot be mixed:
//! ```compile_fail
//! use surgeist_bindings_js::Scope;
//! fn cross_scope<'a, 'b>(left: &mut Scope<'a>, right: &mut Scope<'b>) {
//!     let value = left.string(&[65]).unwrap();
//!     right.to_utf16(&value).unwrap();
//! }
//! ```
//!
//! Explicit rooting is the supported way to retain a value:
//! ```no_run
//! use surgeist_bindings_js::{Error, Process};
//! # fn example() -> Result<(), Error> {
//! let mut process = Process::bootstrap()?;
//! let mut execution = process.execution();
//! let realm = execution.create_realm()?;
//! let root = execution.in_realm(&realm, |scope| {
//!     let value = scope.string(&[0xd800])?;
//!     Ok::<_, Error>(scope.retain(&value))
//! })?;
//! execution.in_realm(&realm, |scope| {
//!     let value = scope.import(&root)?;
//!     assert_eq!(scope.to_utf16(&value)?, [0xd800]);
//!     Ok::<_, Error>(())
//! })?;
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![cfg_attr(
    not(test),
    deny(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented,
        clippy::unreachable,
        clippy::unwrap_used
    )
)]

mod backend;
mod callback_object;
pub mod conversion;
pub mod css;
pub mod cssom;
mod custom_elements;
pub mod declarations;
pub mod dom_exception;
mod error;
mod host;
mod idl;
mod native_object;
mod owner;
pub mod preparation;
mod promise;
mod trusted_types;

use std::rc::Rc;
use std::{any::Any, marker::PhantomData};

pub use callback_object::{CallbackObject, RetainedCallbackObject};
pub use custom_elements::{CustomElementRegistryIdentity, PublishedCustomElementRegistry};
pub use error::{Error, Exception, ExceptionPhase};
pub use host::{HostFailure, HostFailureSlot, RealmAuthority, RealmId, RecordFailure};
pub use idl::{ByteString, DomString, IntegerKind, IntegerMode, IntegerValue, UsvString};
pub use native_object::{
    HtmlAllCollectionCallbacks, NativeNamedPropertyCallbacks, NativeObject, NativeObjectToken,
    NativeProjectionFunding, NativeProjectionPrototype, PreparedNativeProjection,
};
use preparation::{Admitted, ArgumentValues};
pub use promise::PromiseResolver;
pub use trusted_types::{
    TrustedAttributeValue, TrustedPolicyCallback, TrustedType, TrustedTypeData, TrustedTypeKind,
};

/// Host scalar values that can be constructed without evaluating guest source.
/// Strings are constructed separately through [`Scope::string`].
#[derive(Clone, Copy, Debug)]
pub enum Primitive {
    /// The unique JavaScript undefined value.
    Undefined,
    /// The unique JavaScript null value.
    Null,
    /// A JavaScript Boolean with the supplied truth value.
    Boolean(bool),
    /// A JavaScript Number, preserving finite values, infinities and signed zero.
    /// NaN represents JavaScript NaN without a distinct payload guarantee.
    Number(f64),
}

/// ECMAScript language type of a value, determined without guest coercion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    /// The unique undefined value.
    Undefined,
    /// The unique null value, distinct from Object.
    Null,
    /// A Boolean primitive.
    Boolean,
    /// A Number primitive, including NaN and infinities.
    Number,
    /// A BigInt primitive.
    BigInt,
    /// A String primitive.
    String,
    /// A Symbol primitive.
    Symbol,
    /// An object, including functions, callable proxies and boxed primitives.
    Object,
}

/// The unique thread-affine JavaScript execution owner.
///
/// Bootstrap on the worker's entry thread. Later bootstrap is allowed on that
/// thread after the prior owner is dropped. Other threads and duplicate owners
/// receive typed errors before engine entry. Dropping this token releases its
/// context, roots, and pending jobs.
pub struct Process {
    engine: Box<dyn backend::Engine>,
    // Engine records and SDK slots are destroyed before their setup backing.
    _setup_backing: Option<Box<dyn std::any::Any>>,
    thread_affinity: PhantomData<Rc<()>>,
}

impl Process {
    /// Initializes the backend and acquires the unique execution owner.
    pub fn bootstrap() -> Result<Self, Error> {
        Ok(Self {
            engine: Box::new(backend::boa::Engine::bootstrap()?),
            _setup_backing: None,
            thread_affinity: PhantomData,
        })
    }

    /// Admit this process's Rust setup before constructing its backend records.
    /// Thread and singleton validation precede funding. The supplied ordinary
    /// backing is destroyed after the backend; allocator and SDK storage are
    /// excluded. This does not install a subsequent preparation policy.
    pub fn bootstrap_funded<B: 'static, E: From<Error>>(
        fund: impl FnOnce(preparation::Request) -> Result<B, E>,
    ) -> Result<Self, E> {
        let (engine, backing) = backend::boa::Engine::bootstrap_funded(fund)?;
        Ok(Self {
            engine: Box::new(engine),
            _setup_backing: Some(Box::new(backing)),
            thread_affinity: PhantomData,
        })
    }

    /// Borrows this heap exclusively. Forgetting the borrow cannot leak a
    /// second execution owner: the original token retains custody.
    pub fn execution(&mut self) -> Execution<'_> {
        Execution {
            engine: self.engine.as_mut(),
        }
    }

    /// Whether Rust preparation has a caller admission policy. Without one,
    /// preparation is explicitly unaccounted. Profiling remains caller-owned.
    pub fn has_preparation_admission(&self) -> bool {
        self.engine.owner_state().preparation.get().is_some()
    }

    /// Secondary conversion cleanup observations, including after terminal
    /// refusal or release of the original account. This query enters no SDK
    /// frame and never changes the immutable first error.
    pub fn conversion_cleanup_status(&self) -> conversion::CleanupStatus {
        self.engine.owner_state().cleanup_status()
    }

    /// Requested Rust identity, context and shared custody records for one
    /// realm, excluding allocator/SDK storage and the supplied backing body.
    /// Funded creation adds that body's size before calling the funder.
    pub fn realm_storage_bytes(&self) -> usize {
        self.engine.realm_storage_bytes()
    }

    /// Requested Rust root record bodies for one managed object, excluding
    /// allocator and SDK storage. An installed preparation policy admits these
    /// bodies and its retention value before materialization.
    pub fn managed_object_storage_bytes(&self) -> usize {
        self.engine.managed_object_storage_bytes()
    }

    /// Releases the managed holder without entering guest code, including
    /// after this process has refused further execution.
    pub fn release_managed_object(&mut self, object: ManagedObject) -> Result<(), Error> {
        self.engine.release_managed_object(object.holder)
    }
}

/// A sample of Boa's thread-local garbage collector accounting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct HeapObservation {
    /// Shallow bytes in garbage-collected boxes and ephemerons, including
    /// unreachable boxes awaiting collection.
    pub used_bytes: usize,
}

/// Exclusive access to an execution owner's heap.
pub struct Execution<'process> {
    engine: &'process mut dyn backend::Engine,
}

/// A Window-shaped realm during host installation. Only a completed
/// installation yields the ordinary execution capability for this realm.
pub struct PendingWindowRealm(Realm);

/// Host custody of a native browsing context's guest WindowProxy.
pub struct ChildWindow {
    _realm: Realm,
    proxy: Root,
}

impl ChildWindow {
    /// Opens this context's stable WindowProxy in the current execution owner.
    pub fn open<'scope>(&self, scope: &mut Scope<'scope>) -> Result<Value<'scope>, Error> {
        scope.import(&self.proxy)
    }
}

impl Drop for Execution<'_> {
    fn drop(&mut self) {
        self.engine.reap();
    }
}

impl Execution<'_> {
    /// Creates a distinct global environment in this heap.
    pub fn create_realm(&mut self) -> Result<Realm, Error> {
        self.engine.create_realm()
    }

    /// Prepare the attached Window and its immutable prototype chain without
    /// making the realm available for ordinary execution.
    pub fn prepare_window_realm(&mut self) -> Result<PendingWindowRealm, Error> {
        self.engine
            .create_realm_profile(backend::RealmProfile::Window)
            .map(PendingWindowRealm)
    }

    /// Enter a private Window during host installation.
    pub fn in_pending_window_realm<T>(
        &mut self,
        pending: &PendingWindowRealm,
        install: impl for<'scope> FnOnce(&mut Scope<'scope>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.in_realm(&pending.0, install)
    }

    /// Bind the installed EventTarget constructor to the Window interface.
    /// The installer receives the reserved EventTarget prototype and must
    /// return the same-realm constructor it installed on the global. A failed
    /// installation does not yield a realm capability.
    pub fn finish_window_realm<T, E: From<Error>>(
        &mut self,
        pending: PendingWindowRealm,
        install: impl for<'scope> FnOnce(
            &mut Scope<'scope>,
            &Value<'scope>,
        ) -> Result<(Value<'scope>, T), E>,
    ) -> Result<(Realm, T), E> {
        let result =
            self.in_pending_window_realm(&pending, |scope| scope.assemble_window(install))??;
        Ok((pending.0, result))
    }

    /// Admit initial realm setup using ordinary owned backing before a policy
    /// is installed. Existing policies are refused before the funder is called;
    /// later realms use [`Self::create_realm`] and its installed admission.
    /// Backing follows both external realm tokens and guest-retained contexts.
    pub fn create_realm_funded<B: 'static, E: From<Error>>(
        &mut self,
        fund: impl FnOnce(preparation::Request) -> Result<B, E>,
    ) -> Result<Realm, E> {
        self.create_funded_realm(backend::RealmProfile::Plain, fund)
    }

    /// Admit the initial Window template realm before establishing a guest policy.
    pub fn prepare_window_realm_funded<B: 'static, E: From<Error>>(
        &mut self,
        fund: impl FnOnce(preparation::Request) -> Result<B, E>,
    ) -> Result<PendingWindowRealm, E> {
        self.create_funded_realm(backend::RealmProfile::Window, fund)
            .map(PendingWindowRealm)
    }

    fn create_funded_realm<B: 'static, E: From<Error>>(
        &mut self,
        profile: backend::RealmProfile,
        fund: impl FnOnce(preparation::Request) -> Result<B, E>,
    ) -> Result<Realm, E> {
        self.engine.prepare_realm()?;
        if self.engine.owner_state().preparation.get().is_some() {
            return Err(Error::PreparationAlreadyAdmitted.into());
        }
        let bytes = self
            .engine
            .realm_storage_bytes()
            .checked_add(size_of::<B>())
            .ok_or(Error::Allocation)?;
        let backing = fund(preparation::Request::Realm { bytes })?;
        Ok(self
            .engine
            .create_realm_with_backing(profile, Some(RealmBacking::new(backing)))?)
    }

    /// Runs fallible Rust binding operations in a validated realm. The chosen
    /// result includes local cleanup and post-SDK capture reaping. A guest
    /// exception remains catchable unless cleanup itself fatally fails; that
    /// double failure retains the original exception in its terminal report.
    /// The result cannot contain a scoped value; retain [`Root`]s explicitly.
    pub fn in_realm<T>(
        &mut self,
        realm: &Realm,
        action: impl for<'scope> FnOnce(&mut Scope<'scope>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let boundary = owner::Boundary::enter(self.engine.owner_state())?;
        let mut action = Some(action);
        let mut result = None;
        let entered = self.engine.with_realm(realm, &mut |operations| {
            if let Some(action) = action.take() {
                result = Some(action(&mut Scope {
                    operations,
                    invariant: PhantomData,
                })?);
            }
            Ok(())
        });
        let chosen = match entered {
            Ok(()) => result.take().ok_or(Error::BackendUnavailable),
            Err(error) => {
                drop(result.take());
                Err(error)
            }
        };
        self.engine.reap();
        boundary.finish(chosen)
    }

    /// Runs this heap's shared explicit microtask checkpoint. Evaluating a
    /// script does not implicitly run jobs. Jobs can reference several realms;
    /// dropping a realm token does not cancel them. This SDK checkpoint does
    /// not surface rejected promises; host rejection reporting is separate.
    pub fn checkpoint(&mut self) -> Result<(), Error> {
        self.engine.checkpoint()
    }

    /// Samples the thread-local collector without running a collection.
    /// This counts shallow collector allocations across Boa contexts on the
    /// thread. Separate buffers and process memory are outside the sample.
    pub fn observe_heap(&mut self) -> Result<HeapObservation, Error> {
        self.engine.observe_heap()
    }

    /// Advises the engine of memory pressure and reaps abandoned Rust roots.
    /// This is not a promise that every unreachable object will be collected.
    pub fn memory_pressure(&mut self) {
        self.engine.memory_pressure();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RealmKey {
    owner: u64,
    index: u64,
}

#[derive(Debug)]
pub(crate) struct RealmIdentity {
    key: RealmKey,
    _backing: Option<Rc<RealmBacking>>,
}

/// Shared by external tokens, backend context records and the SDK context slot.
/// It deliberately retains no realm identity or execution authority.
pub(crate) struct RealmBacking {
    _backing: Box<dyn std::any::Any>,
}

impl RealmBacking {
    fn new<B: 'static>(backing: B) -> Rc<Self> {
        Rc::new(Self {
            _backing: Box::new(backing),
        })
    }
}

impl std::fmt::Debug for RealmBacking {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RealmBacking")
            .finish_non_exhaustive()
    }
}

/// Identity of a global environment. Clones denote the same realm.
///
/// Tokens are thread-affine, may outlive their process, and are checked before
/// engine use. A token from an old or different owner is refused. The last
/// token's release makes the external context root eligible for reaping;
/// guest functions and other heap references can still keep that realm alive.
#[derive(Clone, Debug)]
pub struct Realm(Rc<RealmIdentity>);

#[derive(Debug)]
pub(crate) struct RootIdentity {
    realm: RealmKey,
    index: u64,
    _backing: Option<Rc<RootBacking>>,
}

/// Shared Rust custody, without an SDK handle, identity or admission policy.
pub(crate) struct RootBacking {
    _preparation: preparation::Guard,
}

impl RootBacking {
    fn new(preparation: preparation::Guard) -> Rc<Self> {
        Rc::new(Self {
            _preparation: preparation,
        })
    }
}

impl std::fmt::Debug for RootBacking {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RootBacking")
            .finish_non_exhaustive()
    }
}

/// Explicitly retained heap value. Cloning retains the same root identity.
///
/// Only the backend owns the actual heap root. Dropping the last token makes
/// that root eligible for release at the next boundary; dropping the process
/// releases it immediately, even if this token survives. Import checks owner
/// identity before accessing the engine. Accepted Rust preparation backing stays
/// live until both this identity and the backend record retire. Clones share one
/// admission. This token is neither Send nor Sync.
#[derive(Clone, Debug)]
pub struct Root(Rc<RootIdentity>);

/// Exclusive native custody of one host object's managed heap holder.
/// The holder keeps the object and its creation context alive. Native callback
/// registrations must not capture this custody.
pub struct ManagedObject {
    holder: Root,
}

/// A value confined to one Rust binding scope.
pub struct Value<'scope> {
    root: Root,
    invariant: PhantomData<&'scope mut &'scope ()>,
}

/// Shared native function behavior. Clones share the same immutable callable;
/// recursive invocation never holds a mutable borrow of the closure.
///
/// Callback values cannot escape without explicit retention:
/// ```compile_fail
/// use surgeist_bindings_js::{Callback, Completion, Value};
/// use std::{cell::RefCell, rc::Rc};
/// let escaped: Rc<RefCell<Option<Value<'static>>>> = Rc::new(RefCell::new(None));
/// let callback = Callback::new(move |_, arguments| {
///     let (_, mut values) = arguments.into_parts();
///     escaped.replace(values.pop());
///     Ok(Completion::Undefined)
/// });
/// ```
#[derive(Clone)]
pub struct Callback(Rc<CallbackBody>);

type CallbackBody = dyn for<'scope> Fn(&mut Scope<'scope>, Arguments<'scope>) -> Result<Completion<'scope>, Error>
    + 'static;

impl Callback {
    /// Creates a recursive, thread-affine native callable. Captures must be
    /// owned; retain guest values explicitly as [`Root`]s. Guest failures use
    /// [`Completion::Throw`] or `Error::Exception` with the original value.
    /// Any other returned error is a sticky host refusal: it terminates this
    /// owner, bypasses guest catch handlers, and overrides later completions.
    /// A callback panic likewise fails closed as [`Error::CallbackPanicked`].
    /// Its payload stays in owner custody until teardown outside callback
    /// frames; user destructors still follow ordinary Rust drop behavior.
    pub fn new(
        action: impl for<'scope> Fn(
            &mut Scope<'scope>,
            Arguments<'scope>,
        ) -> Result<Completion<'scope>, Error>
        + 'static,
    ) -> Self {
        Self(Rc::new(action))
    }
}

/// Exact declaration metadata for a native function.
#[derive(Clone, Copy)]
pub struct FunctionMetadata<'name> {
    /// Exact UTF-16 name, including unmatched surrogates.
    pub name: &'name [u16],
    /// Exact nonnegative own length metadata, preserving the full u32 range.
    pub length: u32,
}

type PrototypeResolver =
    Rc<dyn for<'scope> Fn(&mut Scope<'scope>) -> Result<Value<'scope>, Error> + 'static>;
type ConstructorBody = dyn for<'scope> Fn(
        &mut Scope<'scope>,
        ConstructorArguments<'scope>,
    ) -> Result<ConstructorCompletion<'scope>, Error>
    + 'static;

struct ConstructorBehavior {
    fallback: PrototypeResolver,
    action: Box<ConstructorBody>,
    argument_limit: Option<u32>,
}

/// Shared declared-constructor behavior with a realm-specific prototype resolver.
/// Ordinary calls throw TypeError before the handler runs. Construction requires
/// an explicitly returned object or an original guest throw; there is no implicit
/// acceptance of an engine-allocated receiver or native platform wrapper.
#[derive(Clone)]
pub struct ConstructorCallback(Rc<ConstructorBehavior>);

impl ConstructorCallback {
    /// Requested Rust body for the shared bounded-constructor behavior wrapper.
    /// Caller action/fallback closure bodies are accounted by their own owner.
    pub const fn behavior_storage_bytes() -> usize {
        size_of::<ConstructorBehavior>()
    }

    /// The handler runs in the constructor's creation realm. It converts its
    /// arguments, then consumes [`ConstructorArguments::select_prototype`] before
    /// native action. If NewTarget.prototype is not an object, `fallback` runs
    /// in NewTarget's authenticated function realm and returns that realm's
    /// registered same-interface prototype. Native-global authentication remains
    /// the binding adapter's responsibility; this API does not supply a DOM.
    pub fn new(
        fallback: impl for<'scope> Fn(&mut Scope<'scope>) -> Result<Value<'scope>, Error> + 'static,
        action: impl for<'scope> Fn(
            &mut Scope<'scope>,
            ConstructorArguments<'scope>,
        ) -> Result<ConstructorCompletion<'scope>, Error>
        + 'static,
    ) -> Self {
        Self::with_argument_limit(None, fallback, action)
    }

    /// Like [`Self::new`], with at most `maximum_arguments` supplied values
    /// captured for the handler. This bound is independent of the declared
    /// JavaScript [`FunctionMetadata::length`].
    pub fn bounded(
        maximum_arguments: u32,
        fallback: impl for<'scope> Fn(&mut Scope<'scope>) -> Result<Value<'scope>, Error> + 'static,
        action: impl for<'scope> Fn(
            &mut Scope<'scope>,
            ConstructorArguments<'scope>,
        ) -> Result<ConstructorCompletion<'scope>, Error>
        + 'static,
    ) -> Self {
        Self::with_argument_limit(Some(maximum_arguments), fallback, action)
    }

    fn with_argument_limit(
        argument_limit: Option<u32>,
        fallback: impl for<'scope> Fn(&mut Scope<'scope>) -> Result<Value<'scope>, Error> + 'static,
        action: impl for<'scope> Fn(
            &mut Scope<'scope>,
            ConstructorArguments<'scope>,
        ) -> Result<ConstructorCompletion<'scope>, Error>
        + 'static,
    ) -> Self {
        Self(Rc::new(ConstructorBehavior {
            fallback: Rc::new(fallback),
            action: Box::new(action),
            argument_limit,
        }))
    }
}

/// Supplied constructor arguments and authentic NewTarget. No platform receiver
/// has been allocated by this boundary. Perform argument conversions before
/// selecting the prototype, as required by the installed interface algorithm.
///
/// NewTarget cannot escape its scope:
/// ```compile_fail
/// use surgeist_bindings_js::{ConstructorCallback, ConstructorCompletion, Value};
/// use std::{cell::RefCell, rc::Rc};
/// let escaped: Rc<RefCell<Option<Value<'static>>>> = Rc::new(RefCell::new(None));
/// let constructor = ConstructorCallback::new(
///     |scope| scope.object(surgeist_bindings_js::ObjectPrototype::Default),
///     move |_, arguments| {
///         let (_, target) = arguments.into_parts();
///         escaped.replace(Some(target));
///         Err(surgeist_bindings_js::Error::BackendUnavailable)
///     },
/// );
/// ```
pub struct ConstructorArguments<'scope> {
    values: ArgumentValues<'scope>,
    new_target: Value<'scope>,
    fallback: PrototypeResolver,
}

impl<'scope> ConstructorArguments<'scope> {
    /// Number of captured supplied values, including explicit undefined.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Whether no argument was supplied.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    /// An omitted argument is None; explicit undefined remains a supplied value.
    pub fn get(&self, index: usize) -> Option<&Value<'scope>> {
        self.values.get(index)
    }
    /// Authentic NewTarget supplied by construction, without coercion.
    pub fn new_target(&self) -> &Value<'scope> {
        &self.new_target
    }
    /// Moves the values and authentic NewTarget out without selecting a prototype.
    pub fn into_parts(self) -> (ArgumentValues<'scope>, Value<'scope>) {
        (self.values, self.new_target)
    }
    /// Reads the original NewTarget.prototype once, preserving abrupt completion.
    /// Consumption prevents repeating this helper on the same argument bundle.
    /// The returned prototype is distinct from native creation-realm authority.
    ///
    /// The transition cannot be performed twice on one argument bundle:
    /// ```compile_fail
    /// use surgeist_bindings_js::{ConstructorCallback, ConstructorCompletion, ObjectPrototype};
    /// let constructor = ConstructorCallback::new(
    ///     |scope| scope.object(ObjectPrototype::Default),
    ///     |scope, arguments| {
    ///         let _first = arguments.select_prototype(scope)?;
    ///         let _second = arguments.select_prototype(scope)?;
    ///         Ok(ConstructorCompletion::Return(scope.object(ObjectPrototype::Default)?))
    ///     },
    /// );
    /// ```
    pub fn select_prototype(
        self,
        scope: &mut Scope<'scope>,
    ) -> Result<ConstructorInvocation<'scope>, Error> {
        let root = scope
            .operations
            .constructor_prototype(&self.new_target.root, &self.fallback)?;
        let prototype = scope.value(root);
        Ok(ConstructorInvocation {
            values: self.values,
            new_target: self.new_target,
            prototype,
        })
    }
}

/// Constructor inputs after one prototype-selection transition. The handler
/// remains in its original creation realm even when the selected prototype and
/// NewTarget belong to another realm.
pub struct ConstructorInvocation<'scope> {
    values: ArgumentValues<'scope>,
    new_target: Value<'scope>,
    prototype: Value<'scope>,
}

impl<'scope> ConstructorInvocation<'scope> {
    /// Number of captured supplied values, including explicit undefined.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Whether no argument was supplied.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&Value<'scope>> {
        self.values.get(index)
    }
    /// Authentic NewTarget supplied by construction, without coercion.
    pub fn new_target(&self) -> &Value<'scope> {
        &self.new_target
    }
    /// The already-selected prototype, with no further guest property read.
    pub fn prototype(&self) -> &Value<'scope> {
        &self.prototype
    }
    pub fn into_parts(self) -> (ArgumentValues<'scope>, Value<'scope>, Value<'scope>) {
        (self.values, self.new_target, self.prototype)
    }
}

/// Explicit completion of a declared constructor. Return is checked to contain
/// an object; a primitive is a terminal host refusal, not a default receiver.
pub enum ConstructorCompletion<'scope> {
    /// Return an explicitly created object, checked before SDK completion.
    Return(Value<'scope>),
    /// Throw the exact supplied guest value.
    Throw(Value<'scope>),
}

/// Receiver and supplied arguments branded to the current callback scope.
/// An omitted argument is `None`; an explicit `undefined` is a present value.
///
/// Arguments cannot escape their callback:
/// ```compile_fail
/// use surgeist_bindings_js::{Callback, Completion};
/// use std::{cell::RefCell, rc::Rc};
/// use surgeist_bindings_js::Arguments;
/// let escaped: Rc<RefCell<Option<Arguments<'static>>>> = Rc::new(RefCell::new(None));
/// let callback = Callback::new(move |_, arguments| {
///     escaped.replace(Some(arguments));
///     Ok(Completion::Undefined)
/// });
/// ```
pub struct Arguments<'scope> {
    receiver: Value<'scope>,
    values: ArgumentValues<'scope>,
}

impl<'scope> Arguments<'scope> {
    /// The SDK-normalized receiver: null and undefined become the function's
    /// creation-realm global, and primitives are boxed. Objects retain identity.
    /// This is not an unboxed ECMAScript this value. Interface adapters must
    /// validate their own native brand even for globals and boxed primitives.
    pub fn receiver(&self) -> &Value<'scope> {
        &self.receiver
    }
    /// Number of supplied arguments, including explicit undefined values.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    /// Whether no argument was supplied.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    /// Borrows an argument without manufacturing an omitted value.
    pub fn get(&self, index: usize) -> Option<&Value<'scope>> {
        self.values.get(index)
    }
    /// Moves the receiver and arguments out while preserving their scope brand.
    pub fn into_parts(self) -> (Value<'scope>, ArgumentValues<'scope>) {
        (self.receiver, self.values)
    }
}

/// Guest completion of a native callback. Throw preserves the exact value.
///
/// A completion cannot cross a scope boundary:
/// ```compile_fail
/// use surgeist_bindings_js::{Completion, Execution, Realm};
/// fn escape(execution: &mut Execution<'_>, realm: &Realm) {
///     let completion = execution.in_realm(realm, |scope| {
///         Ok(Completion::Return(scope.string(&[65]).unwrap()))
///     });
/// }
/// ```
pub enum Completion<'scope> {
    /// Return the JavaScript undefined value.
    Undefined,
    /// Return a value from this callback scope.
    Return(Value<'scope>),
    /// Throw a value from this callback scope.
    Throw(Value<'scope>),
}

/// A property name already expressed as exact UTF-16 or a scoped String/Symbol.
/// Dynamic values are checked without ToPropertyKey or guest coercion.
#[derive(Clone, Copy)]
pub enum PropertyKey<'key, 'scope> {
    /// Exact code units, including unmatched surrogates and embedded NUL.
    String(&'key [u16]),
    /// An existing String or Symbol. Other language types are refused.
    Value(&'key Value<'scope>),
}

impl<'key, 'scope> PropertyKey<'key, 'scope> {
    fn input(self) -> backend::PropertyKey<'key> {
        match self {
            Self::String(units) => backend::PropertyKey::String(units),
            Self::Value(value) => backend::PropertyKey::Value(&value.root),
        }
    }
}

/// Well-known symbol identities supplied by the engine, without global lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WellKnownSymbol {
    Iterator,
    ToStringTag,
    Unscopables,
}

/// A specific realm-pristine intrinsic needed by binding algorithms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Intrinsic {
    ArrayEntries,
    ArrayKeys,
    ArrayValues,
    ArrayForEach,
}

/// The complete shape of an own property. Data and accessor fields cannot mix.
/// `None` and explicit undefined accessor values both specify an absent getter
/// or setter; every other accessor must be callable.
///
/// Borrowed property data cannot escape its scope:
/// ```compile_fail
/// use surgeist_bindings_js::{Execution, Realm, PropertyDescriptor};
/// fn escape(execution: &mut Execution<'_>, realm: &Realm) {
///     let descriptor = execution.in_realm(realm, |scope| {
///         let value = scope.string(&[65]).unwrap();
///         Ok(PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true })
///     });
/// }
/// ```
#[derive(Clone, Copy)]
pub enum PropertyDescriptor<'value, 'scope> {
    Data {
        value: &'value Value<'scope>,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: Option<&'value Value<'scope>>,
        set: Option<&'value Value<'scope>>,
        enumerable: bool,
        configurable: bool,
    },
}

impl<'value, 'scope> PropertyDescriptor<'value, 'scope> {
    fn input(self) -> backend::PropertyDescriptor<'value> {
        match self {
            Self::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => backend::PropertyDescriptor::Data {
                value: &value.root,
                writable,
                enumerable,
                configurable,
            },
            Self::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => backend::PropertyDescriptor::Accessor {
                get: get.map(|value| &value.root),
                set: set.map(|value| &value.root),
                enumerable,
                configurable,
            },
        }
    }
}

/// Explicit prototype selection for object construction and mutation.
#[derive(Clone, Copy)]
pub enum ObjectPrototype<'value, 'scope> {
    /// The selected realm's original Object.prototype.
    Default,
    /// No prototype.
    Null,
    /// An existing object; primitives, including an implicit null, are refused.
    Object(&'value Value<'scope>),
}

impl<'value, 'scope> ObjectPrototype<'value, 'scope> {
    fn input(self) -> backend::ObjectPrototype<'value> {
        match self {
            Self::Default => backend::ObjectPrototype::Default,
            Self::Null => backend::ObjectPrototype::Null,
            Self::Object(value) => backend::ObjectPrototype::Object(&value.root),
        }
    }
}

/// Binding operations, independent of the concrete engine's handle types.
pub struct Scope<'scope> {
    operations: &'scope mut dyn backend::Operations,
    invariant: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope> Scope<'scope> {
    /// Install the native reader for a pending Window's named properties.
    /// The callback receives one exact string name, with an undefined result
    /// representing an unsupported name. The reader is never exposed to script.
    ///
    /// # Errors
    ///
    /// Returns an error outside pending Window installation, for a duplicate
    /// installation, or if the callback cannot be admitted.
    pub fn install_window_name_reader(&mut self, reader: &Callback) -> Result<(), Error> {
        let reader = self.fixed_arity_function(
            reader,
            FunctionMetadata {
                name: &[],
                length: 1,
            },
        )?;
        self.operations.install_window_name_reader(&reader.root)
    }

    /// Install the pending WindowProxy's live child reader. With no argument
    /// it returns the native child count. With an index, it returns that
    /// child's WindowProxy or undefined. A second argument requests a boolean
    /// membership check. The reader is never exposed to script.
    pub fn install_window_index_reader(&mut self, reader: &Callback) -> Result<(), Error> {
        let reader = self.fixed_arity_function(
            reader,
            FunctionMetadata {
                name: &[],
                length: 2,
            },
        )?;
        self.operations.install_window_index_reader(&reader.root)
    }

    /// Create a child Window for a native context whose active settings the
    /// host has qualified as same-origin with the current Window. The installer
    /// establishes the reserved EventTarget prototype before publication.
    pub fn create_same_origin_window<T, E: From<Error>>(
        &mut self,
        payload: &Rc<impl Any>,
        install: impl for<'child> FnOnce(
            &mut Scope<'child>,
            &Value<'child>,
        ) -> Result<(Value<'child>, T), E>,
    ) -> Result<(ChildWindow, T), E> {
        let erased: Rc<dyn Any> = payload.clone();
        let child = self
            .operations
            .prepare_same_origin_window(Rc::downgrade(&erased))?;
        let mut install = Some(install);
        let mut result = None;
        self.operations
            .install_child_window(&child, &mut |operations| {
                let install = install.take().ok_or(Error::RegistryConflict)?;
                let mut scope = Scope {
                    operations,
                    invariant: PhantomData,
                };
                result = Some(match scope.assemble_window(install)? {
                    Ok(value) => Ok((scope.global()?.root, value)),
                    Err(error) => Err(error),
                });
                Ok(())
            })?;
        let (proxy, value) = result.ok_or(Error::RegistryConflict)??;
        Ok((
            ChildWindow {
                _realm: child,
                proxy,
            },
            value,
        ))
    }

    /// Resolve host data only for an authentic child WindowProxy in this heap.
    /// The child realm holds a weak reference to its host payload.
    pub fn child_window_payload<T: Any>(
        &self,
        value: &Value<'scope>,
    ) -> Result<Option<Rc<T>>, Error> {
        Ok(self
            .operations
            .child_window_payload(&value.root)?
            .and_then(|data| data.downcast().ok()))
    }

    fn assemble_window<T, E: From<Error>>(
        &mut self,
        install: impl FnOnce(&mut Scope<'scope>, &Value<'scope>) -> Result<(Value<'scope>, T), E>,
    ) -> Result<Result<T, E>, Error> {
        let scope = self;
        let interface = scope.operations.window_interface()?;
        let window_prototype = scope.import(&interface.prototype)?;
        let prototype = scope.import(&interface.event_target_prototype)?;
        let (parent, value) = match install(scope, &prototype) {
            Ok(result) => result,
            Err(error) => return Ok(Err(error)),
        };
        scope
            .operations
            .qualify_window_event_target(&parent.root, &prototype.root)?;
        let global = scope.global()?;
        let prototype_key = &b"prototype".map(u16::from);
        let name = &b"Window".map(u16::from);
        let constructor = ConstructorCallback::bounded(
            0,
            // The interface always refuses before consulting NewTarget.
            |_| Err(Error::BackendUnavailable),
            |scope, _| {
                Ok(ConstructorCompletion::Throw(
                    scope.type_error(&b"Illegal constructor".map(u16::from))?,
                ))
            },
        );
        let window = scope.constructor(&constructor, FunctionMetadata { name, length: 0 })?;
        scope.set_prototype(&window, ObjectPrototype::Object(&parent))?;
        scope.define_property(
            &window,
            PropertyKey::String(prototype_key),
            PropertyDescriptor::Data {
                value: &window_prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        scope.define_new_property(
            &window_prototype,
            PropertyKey::String(&b"constructor".map(u16::from)),
            PropertyDescriptor::Data {
                value: &window,
                writable: true,
                enumerable: false,
                configurable: true,
            },
        )?;
        scope.define_new_property(
            &global,
            PropertyKey::String(name),
            PropertyDescriptor::Data {
                value: &window,
                writable: true,
                enumerable: false,
                configurable: true,
            },
        )?;
        Ok(Ok(value))
    }

    /// Allocates one host object in the entered creation realm, with its
    /// original Object prototype and a private managed holder.
    pub fn managed_object(&mut self) -> Result<ManagedObject, Error> {
        self.operations
            .managed_object()
            .map_err(|error| self.operations.refuse(error))
            .map(|holder| ManagedObject { holder })
    }

    /// Reopens the exact projected object, including from a different realm
    /// of the same owner. The holder itself is never exposed to the guest.
    pub fn open_managed_object(&mut self, object: &ManagedObject) -> Result<Value<'scope>, Error> {
        let root = self.operations.open_managed_object(&object.holder)?;
        Ok(self.value(root))
    }

    /// Observes the actual selected scope realm. In a native callback this
    /// is its creation realm; it says nothing about a guest caller, incumbent
    /// settings, a native Global, or permission to access a Document.
    pub fn realm_authority(&self) -> Result<RealmAuthority, Error> {
        self.check_active()?;
        let owner = self.operations.owner_state();
        Ok(RealmAuthority {
            key: self.operations.realm_key(),
            owner: Rc::downgrade(&owner),
        })
    }

    /// Latches a prepared native refusal in this process and requests engine
    /// termination. The first existing refusal wins. This does not throw a
    /// catchable guest value or promise immediate stack unwinding.
    /// Pending cleanup can still stop guest work while this method records the
    /// selected native cause at the active boundary.
    pub fn refuse_host(&self, failure: &HostFailure) -> Error {
        if failure.owner != self.operations.owner_state().id() {
            return match self.check_active() {
                Ok(()) => self.operations.refuse(Error::ForeignOwner),
                Err(first) => first,
            };
        }
        if let Err(first) = self.operations.check_host_failure_report() {
            return first;
        }
        self.operations.refuse(Error::HostFailure(failure.clone()))
    }

    /// Run completion steps after a native operation has already failed.
    /// Successful steps leave the operation's ordinary error for the caller
    /// to return. If the steps refuse or terminate execution, the earlier
    /// native cause becomes the terminal error. Use an empty, funded
    /// failure slot for this invocation.
    pub fn complete_after_host_failure<T, E, Cause, Storage>(
        &mut self,
        slot: &HostFailureSlot<Cause, Storage>,
        cause: Cause,
        complete: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Result<T, Error>
    where
        Cause: std::error::Error + 'static,
        Storage: 'static,
    {
        self.check_active()?;
        let owner = self.operations.owner_state();
        if slot.owner_id() != owner.id() {
            return Err(Error::ForeignOwner);
        }
        let prior = slot.record(cause).map_err(|failure| match failure {
            RecordFailure::Inactive { error, .. } => error,
            RecordFailure::Occupied { .. } => Error::RegistryConflict,
        })?;
        let result = complete(self);
        match result {
            Ok(value) if self.check_active().is_ok() => Ok(value),
            _ => {
                owner.select_prior_host_failure(prior.clone());
                Err(self.operations.refuse(Error::HostFailure(prior)))
            }
        }
    }

    /// Refuses native continuation after a terminal host failure or execution
    /// termination. Native adapters must call this after guest reentry and
    /// before making their own externally observable mutations.
    pub fn check_active(&self) -> Result<(), Error> {
        self.operations.check_active()
    }

    /// Creates a new callable-only JavaScript function in this realm. Its
    /// native capture survives as long as its guest function remains reachable.
    /// Callback clones share Rust behavior; each function has distinct guest
    /// identity and preserves its own creation realm.
    pub fn function(&mut self, callback: &Callback) -> Result<Value<'scope>, Error> {
        let root = self.operations.function(callback, None)?;
        Ok(self.value(root))
    }

    /// Append an existing JavaScript function to this heap's microtask queue.
    /// It runs with no arguments at an explicit checkpoint, in queue order
    /// with Promise jobs. Native callbacks should report ordinary guest
    /// exceptions within the callback. Terminal host refusal still stops
    /// the owning heap and is returned by the checkpoint.
    pub fn enqueue_microtask(&mut self, function: &Value<'scope>) -> Result<(), Error> {
        self.operations.enqueue_microtask(&function.root)
    }

    /// Creates a callable with declared name and length. The native handler
    /// receives only the supplied arguments up to that length.
    pub fn fixed_arity_function(
        &mut self,
        callback: &Callback,
        metadata: FunctionMetadata<'_>,
    ) -> Result<Value<'scope>, Error> {
        self.bounded_function(callback, metadata, metadata.length)
    }

    /// Creates a callable whose captured argument prefix is independent of
    /// its declared JavaScript length. Explicit undefined remains supplied.
    pub fn bounded_function(
        &mut self,
        callback: &Callback,
        metadata: FunctionMetadata<'_>,
        maximum_arguments: u32,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.function(
            callback,
            Some(backend::DeclaredCallable {
                metadata,
                maximum_arguments,
            }),
        )?;
        Ok(self.value(root))
    }

    /// Creates a declared constructor with exact name/length metadata. Calls
    /// without construction throw TypeError without invoking its handler.
    pub fn constructor(
        &mut self,
        callback: &ConstructorCallback,
        metadata: FunctionMetadata<'_>,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.constructor(callback, metadata)?;
        Ok(self.value(root))
    }

    /// Creates a TypeError for an explicitly specified guest validation rule.
    /// Returning a host [`Error`] from a callback instead fails the owner closed.
    pub fn type_error(&mut self, message: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.operations.type_error(message)?;
        Ok(self.value(root))
    }

    /// Creates a standard DOMException in the entered realm, using its
    /// privately installed interface prototype and the exact UTF-16 message.
    /// The entered realm's account admits both outbound string copies.
    /// Returns [`Error::DomExceptionUnavailable`] if this realm has no installation.
    pub fn dom_exception(
        &mut self,
        name: dom_exception::DomExceptionName,
        message: &[u16],
    ) -> Result<Value<'scope>, Error> {
        dom_exception::create_native(self, name, message)
    }

    fn value(&self, root: Root) -> Value<'scope> {
        Value {
            root,
            invariant: PhantomData,
        }
    }

    /// Evaluates a classic script from exact UTF-16 code units. Compilation and
    /// execution exceptions retain their original thrown value without calling
    /// guest coercion code. This call does not run a microtask checkpoint.
    pub fn evaluate(&mut self, source: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.operations.evaluate(source)?;
        Ok(self.value(root))
    }

    /// Creates an exact JavaScript string, including unpaired surrogates.
    pub fn string(&mut self, units: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.operations.string(units)?;
        Ok(self.value(root))
    }

    /// Constructs a scoped scalar directly, without guest evaluation or coercion.
    pub fn primitive(&mut self, primitive: Primitive) -> Result<Value<'scope>, Error> {
        let root = self.operations.primitive(primitive)?;
        Ok(self.value(root))
    }

    /// Returns the ECMAScript language type without reading guest properties or
    /// invoking coercion hooks. Functions remain [`ValueKind::Object`].
    pub fn kind(&mut self, value: &Value<'scope>) -> Result<ValueKind, Error> {
        self.operations.kind(&value.root)
    }

    /// Tests ECMAScript IsCallable without invoking guest hooks. A true result
    /// means a `[[Call]]` internal method exists, not that calling will succeed:
    /// class constructors and revoked callable proxies are callable and throw
    /// when called. Callability is separate from [`ValueKind::Object`].
    pub fn is_callable(&mut self, value: &Value<'scope>) -> Result<bool, Error> {
        self.operations.is_callable(&value.root)
    }

    /// Tests ECMAScript IsConstructor without reading properties or invoking
    /// guest hooks. A revoked constructible proxy still has `[[Construct]]`.
    pub fn is_constructor(&mut self, value: &Value<'scope>) -> Result<bool, Error> {
        self.operations.is_constructor(&value.root)
    }

    /// Construct an exact guest object with no arguments, using the supplied
    /// constructor as NewTarget and preserving the original thrown value.
    pub fn construct_without_arguments(
        &mut self,
        constructor: &Value<'scope>,
    ) -> Result<Value<'scope>, Error> {
        let root = self
            .operations
            .construct_without_arguments(&constructor.root)?;
        Ok(self.value(root))
    }

    /// Gets this realm's global object.
    pub fn global(&mut self) -> Result<Value<'scope>, Error> {
        let root = self.operations.global()?;
        Ok(self.value(root))
    }

    /// Reads a String/Symbol property. Getters may run guest code and throw.
    /// Invalid dynamic keys produce [`Error::ExpectedPropertyKey`] without coercion.
    pub fn get(
        &mut self,
        object: &Value<'scope>,
        key: PropertyKey<'_, 'scope>,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.get(&object.root, key.input())?;
        Ok(self.value(root))
    }

    /// Sets a String/Symbol property. Setters may run guest code and throw. A false
    /// internal Set result is reported as [`Error::PropertyRejected`].
    pub fn set(
        &mut self,
        object: &Value<'scope>,
        key: PropertyKey<'_, 'scope>,
        value: &Value<'scope>,
    ) -> Result<(), Error> {
        self.operations.set(&object.root, key.input(), &value.root)
    }

    /// Defines one complete own property. Proxy traps can reenter or throw;
    /// Boolean rejection is [`Error::PropertyRejected`].
    pub fn define_property(
        &mut self,
        object: &Value<'scope>,
        key: PropertyKey<'_, 'scope>,
        descriptor: PropertyDescriptor<'_, 'scope>,
    ) -> Result<(), Error> {
        self.operations.define_property(
            &object.root,
            key.input(),
            descriptor.input(),
            backend::PropertyDefinition::AllowExisting,
        )
    }

    /// Defines a complete own property after checking that the key has no own
    /// property. An existing own property returns [`Error::PropertyRejected`].
    /// On proxies the probe and definition are separate observable steps.
    /// Either trap can throw, and reentrant guest code can change the object
    /// between the two steps.
    pub fn define_new_property(
        &mut self,
        object: &Value<'scope>,
        key: PropertyKey<'_, 'scope>,
        descriptor: PropertyDescriptor<'_, 'scope>,
    ) -> Result<(), Error> {
        self.operations.define_property(
            &object.root,
            key.input(),
            descriptor.input(),
            backend::PropertyDefinition::RequireAbsent,
        )
    }

    /// Creates an empty object with an explicit prototype selection.
    pub fn object(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
    ) -> Result<Value<'scope>, Error> {
        let root = self.operations.object(prototype.input())?;
        Ok(self.value(root))
    }

    /// Changes an object's prototype. Cycles, nonextensibility and proxy refusal
    /// produce [`Error::PropertyRejected`]; trap exceptions preserve their value.
    pub fn set_prototype(
        &mut self,
        object: &Value<'scope>,
        prototype: ObjectPrototype<'_, 'scope>,
    ) -> Result<(), Error> {
        self.operations
            .set_prototype(&object.root, prototype.input())
    }

    /// Creates a scalar with backing supplied from the caller's current native
    /// context. Funding completes before construction, which runs no guest code.
    pub fn primitive_funded<T: preparation::Retention>(
        &mut self,
        primitive: Primitive,
        controller: &preparation::Controller<T>,
        fund: impl FnOnce(preparation::Request) -> Result<T, Error>,
    ) -> Result<Value<'scope>, Error> {
        let backing = self.fund_root(controller, fund)?;
        let root = self.operations.primitive_funded(primitive, backing)?;
        Ok(self.value(root))
    }

    /// Copies a known UTF-16 payload without guest entry, using owned backing
    /// supplied from the caller's current native context.
    pub fn string_funded<T: preparation::Retention>(
        &mut self,
        units: &[u16],
        controller: &preparation::Controller<T>,
        fund: impl FnOnce(preparation::Request) -> Result<T, Error>,
    ) -> Result<Value<'scope>, Error> {
        self.operations.validate_string_length(units.len())?;
        let backing = self.fund_root(controller, fund)?;
        let root = self.operations.string_funded(units, backing)?;
        Ok(self.value(root))
    }

    fn fund_root<T: preparation::Retention>(
        &self,
        controller: &preparation::Controller<T>,
        fund: impl FnOnce(preparation::Request) -> Result<T, Error>,
    ) -> Result<preparation::Guard, Error> {
        let owner = self.operations.owner_state();
        controller.check_owner(&owner)?;
        controller.fund(
            preparation::Request::RootRecord {
                bytes: self.operations.root_storage_bytes(),
            },
            fund,
        )
    }

    /// Creates a dense array in this realm, preserving value identity and order.
    /// Every supplied value creates an own index, including explicit undefined.
    /// The embedding policy allows at most 16,777,216 elements, the minimum
    /// capacity of supported pinned SDK build modes; this is not a JS limit.
    pub fn array(&mut self, values: &[Value<'scope>]) -> Result<Value<'scope>, Error> {
        self.check_active()?;
        backend::validate_array_length(values.len())?;
        let owner = self.operations.owner_state();
        let mut roots = preparation::buffer::<Root>(
            &owner,
            preparation::BufferKind::OutboundRoots,
            values.len(),
        )?;
        for value in values {
            preparation::charge(
                &owner,
                preparation::Work::BufferElement(preparation::BufferKind::OutboundRoots),
            )?;
            self.operations.validate_root(&value.root)?;
            roots.value.push(value.root.clone());
        }
        let root = self.operations.array(&roots)?;
        Ok(self.value(root))
    }

    /// Retrieves a well-known Symbol identity without consulting user globals.
    pub fn well_known_symbol(&mut self, symbol: WellKnownSymbol) -> Result<Value<'scope>, Error> {
        let root = self.operations.well_known_symbol(symbol)?;
        Ok(self.value(root))
    }

    /// Retrieves a specifically captured pristine intrinsic from this realm.
    pub fn intrinsic(&mut self, intrinsic: Intrinsic) -> Result<Value<'scope>, Error> {
        let root = self.operations.intrinsic(intrinsic)?;
        Ok(self.value(root))
    }

    /// Calls a function with an explicit receiver and arguments. Callability,
    /// ownership and argument bounds are checked before engine invocation.
    pub fn call(
        &mut self,
        function: &Value<'scope>,
        receiver: &Value<'scope>,
        arguments: &[Value<'scope>],
    ) -> Result<Value<'scope>, Error> {
        self.call_with_prefix(function, receiver, &[], arguments)
    }

    /// Call a function and acknowledge entry after all backend preparation.
    /// The acknowledgment runs once before the guest function can run. If
    /// acknowledgment refuses, the guest function is never entered.
    /// The acknowledgment is for native bookkeeping and must not reenter
    /// the engine.
    pub fn call_with_entry(
        &mut self,
        function: &Value<'scope>,
        receiver: &Value<'scope>,
        arguments: &[Value<'scope>],
        enter: impl FnOnce() -> Result<(), Error>,
    ) -> Result<Value<'scope>, Error> {
        let mut enter = Some(enter);
        self.call_with_entry_and_prefix(function, receiver, &[], arguments, &mut || {
            enter.take().ok_or(Error::BackendUnavailable)?()
        })
    }

    pub(crate) fn call_with_prefix(
        &mut self,
        function: &Value<'scope>,
        receiver: &Value<'scope>,
        prefix: &[Value<'scope>],
        arguments: &[Value<'scope>],
    ) -> Result<Value<'scope>, Error> {
        self.call_with_entry_and_prefix(function, receiver, prefix, arguments, &mut || Ok(()))
    }

    fn call_with_entry_and_prefix(
        &mut self,
        function: &Value<'scope>,
        receiver: &Value<'scope>,
        prefix: &[Value<'scope>],
        arguments: &[Value<'scope>],
        enter: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<Value<'scope>, Error> {
        self.check_active()?;
        let length = prefix
            .len()
            .checked_add(arguments.len())
            .ok_or(Error::TooManyArguments)?;
        if length > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        self.operations.validate_root(&function.root)?;
        self.operations.validate_root(&receiver.root)?;
        let owner = self.operations.owner_state();
        let mut roots =
            preparation::buffer::<Root>(&owner, preparation::BufferKind::OutboundRoots, length)?;
        for value in prefix.iter().chain(arguments) {
            preparation::charge(
                &owner,
                preparation::Work::BufferElement(preparation::BufferKind::OutboundRoots),
            )?;
            self.operations.validate_root(&value.root)?;
            roots.value.push(value.root.clone());
        }
        let root = self
            .operations
            .call(&function.root, &receiver.root, &roots, enter)?;
        Ok(self.value(root))
    }

    /// Performs JavaScript ToString, returning exact UTF-16 code units. Unlike
    /// exception capture, this explicit conversion may run guest code.
    pub fn to_utf16(&mut self, value: &Value<'scope>) -> Result<Admitted<Vec<u16>>, Error> {
        self.operations.to_utf16(&value.root)
    }

    /// Converts to Web IDL boolean through noncoercive ECMAScript ToBoolean.
    /// Objects are truthy without invoking their conversion hooks.
    pub fn boolean(&mut self, value: &Value<'scope>) -> Result<bool, Error> {
        self.operations.to_boolean(&value.root)
    }

    /// Converts to ordinary Web IDL DOMString through one ECMAScript ToString.
    /// Preserves exact UTF-16, including unmatched surrogates. Null becomes
    /// `"null"`; no LegacyNullToEmptyString extended attribute is applied.
    pub fn dom_string(&mut self, value: &Value<'scope>) -> Result<Admitted<DomString>, Error> {
        self.to_utf16(value).map(|units| units.map(DomString::new))
    }

    /// Converts to Web IDL USVString through one ToString, replacing each
    /// unmatched surrogate with U+FFFD and retaining valid surrogate pairs.
    pub fn usv_string(&mut self, value: &Value<'scope>) -> Result<Admitted<UsvString>, Error> {
        let units = self.to_utf16(value)?;
        preparation::charge(
            &self.operations.owner_state(),
            preparation::Work::CodeUnits { units: units.len() },
        )?;
        Ok(units.map(UsvString::normalize))
    }

    /// Converts to Web IDL ByteString through one ToString. Code units above
    /// 255 produce [`Error::ByteStringOutOfRange`], corresponding to TypeError
    /// in an installed interface; no UTF-8 encoding or byte truncation occurs.
    pub fn byte_string(&mut self, value: &Value<'scope>) -> Result<Admitted<ByteString>, Error> {
        let units = self.to_utf16(value)?;
        let owner = self.operations.owner_state();
        let mut backing = None;
        let result = ByteString::convert_with(units.value, |step| match step {
            idl::string::ByteStringStep::Inspect => preparation::charge(
                &owner,
                preparation::Work::BufferElement(preparation::BufferKind::Utf16),
            ),
            idl::string::ByteStringStep::Allocate(elements) => {
                backing = owner
                    .preparation
                    .get()
                    .filter(|_| elements != 0)
                    .map(|account| {
                        account.admit(preparation::Request::Buffer {
                            kind: preparation::BufferKind::Bytes,
                            bytes: elements,
                            elements,
                        })
                    })
                    .transpose()?;
                Ok(())
            }
            idl::string::ByteStringStep::Copy(units) => {
                preparation::charge(&owner, preparation::Work::CodeUnits { units })
            }
        });
        drop(units.backing);
        let value = result?;
        Ok(Admitted { value, backing })
    }

    /// Converts to finite Web IDL float through one ECMAScript ToNumber and
    /// binary32 rounding to nearest with ties to even. Nonfinite values and
    /// rounding overflow produce [`Error::FloatOutOfRange`] (IDL TypeError).
    /// Negative zero and negative underflow retain their sign.
    pub fn float(&mut self, value: &Value<'scope>) -> Result<f32, Error> {
        let number = self.operations.to_number(&value.root)?;
        idl::float(number).map_err(|_| Error::FloatOutOfRange)
    }

    /// Converts to unrestricted Web IDL float through one ToNumber, allowing
    /// infinities and rounding overflow. NaN is canonicalized to 0x7fc00000.
    pub fn unrestricted_float(&mut self, value: &Value<'scope>) -> Result<f32, Error> {
        let number = self.operations.to_number(&value.root)?;
        Ok(idl::unrestricted_float(number))
    }

    /// Converts to finite Web IDL double through one ECMAScript ToNumber.
    /// Nonfinite values produce [`Error::FloatOutOfRange`] (IDL TypeError).
    pub fn double(&mut self, value: &Value<'scope>) -> Result<f64, Error> {
        let number = self.operations.to_number(&value.root)?;
        idl::double(number).map_err(|_| Error::FloatOutOfRange)
    }

    /// Converts to unrestricted Web IDL double through one ToNumber, retaining
    /// infinities and signed zero. NaN is canonicalized to 0x7ff8000000000000.
    pub fn unrestricted_double(&mut self, value: &Value<'scope>) -> Result<f64, Error> {
        let number = self.operations.to_number(&value.root)?;
        Ok(idl::unrestricted_double(number))
    }

    /// Converts through ECMAScript ToNumber and the selected Web IDL integer
    /// algorithm. Guest coercion runs once and may throw. EnforceRange failure
    /// is [`Error::IntegerOutOfRange`], corresponding to a JavaScript TypeError
    /// when used by an installed interface. The result retains its declared type.
    pub fn integer(
        &mut self,
        value: &Value<'scope>,
        kind: IntegerKind,
        mode: IntegerMode,
    ) -> Result<IntegerValue, Error> {
        let number = self.operations.to_number(&value.root)?;
        idl::convert(number, kind, mode).map_err(|_| Error::IntegerOutOfRange)
    }

    /// JavaScript strict equality, without coercion.
    pub fn strict_equals(
        &mut self,
        left: &Value<'scope>,
        right: &Value<'scope>,
    ) -> Result<bool, Error> {
        self.operations.strict_equals(&left.root, &right.root)
    }

    /// Keeps this value alive beyond the current scope.
    pub fn retain(&self, value: &Value<'scope>) -> Root {
        value.root.clone()
    }

    /// Reopens a retained value from this heap, including another realm's
    /// value. Foreign owners are refused before any SDK handle conversion.
    pub fn import(&mut self, root: &Root) -> Result<Value<'scope>, Error> {
        self.operations.validate_root(root)?;
        Ok(self.value(root.clone()))
    }

    /// Capture a host-created exception value for reporting an invocation.
    pub fn invocation_exception(&self, value: &Value<'scope>) -> Exception {
        Exception {
            phase: ExceptionPhase::Invoke,
            value: self.retain(value),
            rule: None,
        }
    }
}

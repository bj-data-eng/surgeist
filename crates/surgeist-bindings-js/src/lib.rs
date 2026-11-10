//! Safe JavaScript execution for native Surgeist bindings.
//!
//! [`Process`] owns one thread-affine engine. Its borrowed [`Execution`] creates
//! realms and enters a [`Scope`]; values leave that scope only through explicit
//! [`Scope::retain`]. Persistent roots contain identities, never engine handles.
//!
//! A scoped value cannot escape:
//! ```compile_fail
//! use surgeist_bindings_js::{EngineKind, Process};
//! let mut process = Process::bootstrap(EngineKind::V8).unwrap();
//! let mut execution = process.execution();
//! let realm = execution.create_realm().unwrap();
//! let escaped = execution.in_realm(&realm, |scope| scope.evaluate(&[49]));
//! ```
//!
//! There cannot be two simultaneous execution borrows:
//! ```compile_fail,E0499
//! use surgeist_bindings_js::{EngineKind, Process};
//! let mut process = Process::bootstrap(EngineKind::V8).unwrap();
//! let mut first = process.execution();
//! let mut second = process.execution();
//! first.create_realm().unwrap();
//! second.create_realm().unwrap();
//! ```
//!
//! An execution owner cannot move to a pre-existing thread:
//! ```compile_fail,E0277
//! use surgeist_bindings_js::{EngineKind, Process};
//! let process = Process::bootstrap(EngineKind::V8).unwrap();
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
//! use surgeist_bindings_js::{EngineKind, Error, Process};
//! # fn example() -> Result<(), Error> {
//! let mut process = Process::bootstrap(EngineKind::V8)?;
//! let mut execution = process.execution();
//! let realm = execution.create_realm()?;
//! let root = execution.in_realm(&realm, |scope| {
//!     let value = scope.string(&[0xd800])?;
//!     Ok::<_, Error>(scope.retain(&value))
//! })??;
//! execution.in_realm(&realm, |scope| {
//!     let value = scope.import(&root)?;
//!     assert_eq!(scope.to_utf16(&value)?, [0xd800]);
//!     Ok::<_, Error>(())
//! })??;
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
pub mod declarations;
mod error;
mod idl;
mod owner;

use std::marker::PhantomData;
use std::rc::Rc;

pub use error::{Error, Exception, ExceptionPhase};
pub use idl::{ByteString, DomString, IntegerKind, IntegerMode, IntegerValue, UsvString};

/// Engine selected before creating any realm or heap values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EngineKind {
    /// External rusty_v8, pinned by the package manifest.
    V8,
}

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

/// One process-lived platform and one exclusively owned, thread-affine heap.
///
/// Bootstrap on the worker's entry thread. Later bootstrap is allowed only on
/// that same thread, after the prior owner is dropped. Other threads and a
/// duplicate live owner receive typed errors before any engine entry. Do not
/// independently initialize or use the same V8 dependency outside this owner.
/// Platform disposal is deliberately excluded; dropping this token releases its
/// heap, roots and pending jobs, while the platform remains process-lived.
pub struct Process {
    engine: Box<dyn backend::Engine>,
    thread_affinity: PhantomData<Rc<()>>,
}

impl Process {
    /// Initializes the selected backend and acquires its unique execution owner.
    pub fn bootstrap(kind: EngineKind) -> Result<Self, Error> {
        let engine: Box<dyn backend::Engine> = match kind {
            EngineKind::V8 => Box::new(backend::v8::Engine::bootstrap()?),
        };
        Ok(Self {
            engine,
            thread_affinity: PhantomData,
        })
    }

    /// Borrows this heap exclusively. Forgetting the borrow cannot leak a
    /// second isolate: this process token continues to own the original heap.
    pub fn execution(&mut self) -> Execution<'_> {
        Execution {
            engine: self.engine.as_mut(),
        }
    }
}

/// Exclusive access to an execution owner's heap.
pub struct Execution<'process> {
    engine: &'process mut dyn backend::Engine,
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

    /// Runs Rust binding operations in a validated realm. The result cannot
    /// contain a scoped value; explicitly retained [`Root`]s may escape.
    pub fn in_realm<T>(
        &mut self,
        realm: &Realm,
        action: impl for<'scope> FnOnce(&mut Scope<'scope>) -> T,
    ) -> Result<T, Error> {
        let mut action = Some(action);
        let mut result = None;
        self.engine.with_realm(realm, &mut |operations| {
            if let Some(action) = action.take() {
                result = Some(action(&mut Scope {
                    operations,
                    invariant: PhantomData,
                }));
            }
        })?;
        self.engine.reap();
        result.ok_or(Error::BackendUnavailable)
    }

    /// Runs this heap's shared explicit microtask checkpoint. Evaluating a
    /// script does not implicitly run jobs. Jobs can reference several realms;
    /// dropping a realm token does not cancel them. This SDK checkpoint does
    /// not surface rejected promises; host rejection reporting is separate.
    pub fn checkpoint(&mut self) -> Result<(), Error> {
        self.engine.checkpoint()
    }

    /// Advises the engine of memory pressure and reaps abandoned Rust roots.
    /// This is not a promise that every unreachable object will be collected.
    pub fn memory_pressure(&mut self) {
        self.engine.memory_pressure();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RealmKey {
    owner: u64,
    index: u64,
}

#[derive(Debug)]
pub(crate) struct RealmIdentity {
    key: RealmKey,
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
}

/// Explicitly retained heap value. Cloning retains the same root identity.
///
/// Only the backend owns the actual heap root. Dropping the last token makes
/// that root eligible for release at the next boundary; dropping the process
/// releases it immediately, even if this token survives. Import checks owner
/// identity before accessing the engine. This token is neither Send nor Sync.
#[derive(Clone, Debug)]
pub struct Root(Rc<RootIdentity>);

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

/// Exact declaration metadata for a native constructor function.
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
}

/// Shared declared-constructor behavior with a realm-specific prototype resolver.
/// Ordinary calls throw TypeError before the handler runs. Construction requires
/// an explicitly returned object or an original guest throw; there is no implicit
/// acceptance of an engine-allocated receiver or native platform wrapper.
#[derive(Clone)]
pub struct ConstructorCallback(Rc<ConstructorBehavior>);

impl ConstructorCallback {
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
        Self(Rc::new(ConstructorBehavior {
            fallback: Rc::new(fallback),
            action: Box::new(action),
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
    values: Vec<Value<'scope>>,
    new_target: Value<'scope>,
    fallback: PrototypeResolver,
}

impl<'scope> ConstructorArguments<'scope> {
    /// Number of supplied values, including explicit undefined.
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
    pub fn into_parts(self) -> (Vec<Value<'scope>>, Value<'scope>) {
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
    values: Vec<Value<'scope>>,
    new_target: Value<'scope>,
    prototype: Value<'scope>,
}

impl<'scope> ConstructorInvocation<'scope> {
    /// Number of supplied values, including explicit undefined.
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
    pub fn into_parts(self) -> (Vec<Value<'scope>>, Value<'scope>, Value<'scope>) {
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
    values: Vec<Value<'scope>>,
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
    pub fn into_parts(self) -> (Value<'scope>, Vec<Value<'scope>>) {
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
///         Completion::Return(scope.string(&[65]).unwrap())
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
    ArrayValues,
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
///         PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true }
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
        let root = self.operations.function(callback)?;
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
        self.operations
            .define_property(&object.root, key.input(), descriptor.input())
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

    /// Creates a dense array in this realm, preserving value identity and order.
    /// Every supplied value creates an own index, including explicit undefined.
    /// The embedding policy allows at most 16,777,216 elements, the minimum
    /// capacity of supported pinned SDK build modes; this is not a JS limit.
    pub fn array(&mut self, values: &[Value<'scope>]) -> Result<Value<'scope>, Error> {
        self.check_active()?;
        backend::validate_array_length(values.len())?;
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(values.len())
            .map_err(|_| Error::Allocation)?;
        roots.extend(values.iter().map(|value| value.root.clone()));
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
        self.check_active()?;
        if arguments.len() > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(arguments.len())
            .map_err(|_| Error::Allocation)?;
        roots.extend(arguments.iter().map(|value| value.root.clone()));
        let root = self
            .operations
            .call(&function.root, &receiver.root, &roots)?;
        Ok(self.value(root))
    }

    /// Performs JavaScript ToString, returning exact UTF-16 code units. Unlike
    /// exception capture, this explicit conversion may run guest code.
    pub fn to_utf16(&mut self, value: &Value<'scope>) -> Result<Vec<u16>, Error> {
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
    pub fn dom_string(&mut self, value: &Value<'scope>) -> Result<DomString, Error> {
        self.to_utf16(value).map(DomString::new)
    }

    /// Converts to Web IDL USVString through one ToString, replacing each
    /// unmatched surrogate with U+FFFD and retaining valid surrogate pairs.
    pub fn usv_string(&mut self, value: &Value<'scope>) -> Result<UsvString, Error> {
        self.to_utf16(value).map(UsvString::normalize)
    }

    /// Converts to Web IDL ByteString through one ToString. Code units above
    /// 255 produce [`Error::ByteStringOutOfRange`], corresponding to TypeError
    /// in an installed interface; no UTF-8 encoding or byte truncation occurs.
    pub fn byte_string(&mut self, value: &Value<'scope>) -> Result<ByteString, Error> {
        ByteString::convert(self.to_utf16(value)?)
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
}

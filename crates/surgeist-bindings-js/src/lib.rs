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
mod error;
mod idl;

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

    /// Gets this realm's global object.
    pub fn global(&mut self) -> Result<Value<'scope>, Error> {
        let root = self.operations.global()?;
        Ok(self.value(root))
    }

    /// Reads a named property. Getters may run guest code and throw.
    pub fn get(&mut self, object: &Value<'scope>, name: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.operations.get(&object.root, name)?;
        Ok(self.value(root))
    }

    /// Sets a named property. Setters may run guest code and throw. A false
    /// internal Set result is reported as [`Error::PropertyRejected`].
    pub fn set(
        &mut self,
        object: &Value<'scope>,
        name: &[u16],
        value: &Value<'scope>,
    ) -> Result<(), Error> {
        self.operations.set(&object.root, name, &value.root)
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

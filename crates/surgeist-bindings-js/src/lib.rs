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

use std::marker::PhantomData;
use std::rc::Rc;

pub use error::{Error, Exception, ExceptionPhase};

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
        self.engine.validate_realm(realm)?;
        let result = action(&mut Scope {
            engine: self.engine,
            realm: realm.clone(),
            invariant: PhantomData,
        });
        self.engine.reap();
        Ok(result)
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

#[derive(Debug)]
pub(crate) struct RealmIdentity {
    owner: u64,
    index: u64,
}

/// Identity of a global environment. Clones denote the same realm.
///
/// Tokens are thread-affine, may outlive their process, and are checked before
/// engine use. A token from an old or different owner is refused.
#[derive(Clone, Debug)]
pub struct Realm(Rc<RealmIdentity>);

#[derive(Debug)]
pub(crate) struct RootIdentity {
    realm: Realm,
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

/// Binding operations, independent of the concrete engine's handle types.
pub struct Scope<'scope> {
    engine: &'scope mut dyn backend::Engine,
    realm: Realm,
    invariant: PhantomData<&'scope mut &'scope ()>,
}

impl<'scope> Scope<'scope> {
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
        let root = self.engine.evaluate(&self.realm, source)?;
        Ok(self.value(root))
    }

    /// Creates an exact JavaScript string, including unpaired surrogates.
    pub fn string(&mut self, units: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.engine.string(&self.realm, units)?;
        Ok(self.value(root))
    }

    /// Gets this realm's global object.
    pub fn global(&mut self) -> Result<Value<'scope>, Error> {
        let root = self.engine.global(&self.realm)?;
        Ok(self.value(root))
    }

    /// Reads a named property. Getters may run guest code and throw.
    pub fn get(&mut self, object: &Value<'scope>, name: &[u16]) -> Result<Value<'scope>, Error> {
        let root = self.engine.get(&self.realm, &object.root, name)?;
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
        self.engine
            .set(&self.realm, &object.root, name, &value.root)
    }

    /// Calls a function with an explicit receiver and arguments. Callability,
    /// ownership and argument bounds are checked before engine invocation.
    pub fn call(
        &mut self,
        function: &Value<'scope>,
        receiver: &Value<'scope>,
        arguments: &[Value<'scope>],
    ) -> Result<Value<'scope>, Error> {
        if arguments.len() > i32::MAX as usize {
            return Err(Error::TooManyArguments);
        }
        let mut roots = Vec::new();
        roots
            .try_reserve_exact(arguments.len())
            .map_err(|_| Error::Allocation)?;
        roots.extend(arguments.iter().map(|value| value.root.clone()));
        let root = self
            .engine
            .call(&self.realm, &function.root, &receiver.root, &roots)?;
        Ok(self.value(root))
    }

    /// Performs JavaScript ToString, returning exact UTF-16 code units. Unlike
    /// exception capture, this explicit conversion may run guest code.
    pub fn to_utf16(&mut self, value: &Value<'scope>) -> Result<Vec<u16>, Error> {
        self.engine.to_utf16(&self.realm, &value.root)
    }

    /// JavaScript strict equality, without coercion.
    pub fn strict_equals(
        &mut self,
        left: &Value<'scope>,
        right: &Value<'scope>,
    ) -> Result<bool, Error> {
        self.engine.strict_equals(&left.root, &right.root)
    }

    /// Keeps this value alive beyond the current scope.
    pub fn retain(&self, value: &Value<'scope>) -> Root {
        value.root.clone()
    }

    /// Reopens a retained value from this heap, including another realm's
    /// value. Foreign owners are refused before any SDK handle conversion.
    pub fn import(&mut self, root: &Root) -> Result<Value<'scope>, Error> {
        self.engine.validate_root(root)?;
        Ok(self.value(root.clone()))
    }
}

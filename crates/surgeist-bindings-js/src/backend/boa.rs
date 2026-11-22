//! The Boa adapter owns the only guest heap used by this execution owner.

use std::{
    any::Any,
    cell::{Cell, OnceCell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use boa_engine::{
    Context, JsError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    builtins::promise::ResolvingFunctions,
    context::{HostHooks, intrinsics::Intrinsics},
    module::IdleModuleLoader,
    object::{
        builtins::JsPromise,
        host::{WindowOrigin, immutable_prototype, window_properties, window_proxy},
    },
    property::PropertyDescriptor as BoaDescriptor,
    realm::{Realm as BoaRealm, WeakRealm},
};

use crate::{
    Error, Exception, ExceptionPhase, HeapObservation, Realm, RealmBacking, RealmIdentity,
    RealmKey, Root, RootBacking, RootIdentity,
    callback_object::CallbackObjectIdentity,
    owner::{BackendLease, OwnerState},
    preparation::{Guard, Request},
};

use super::{NativeProjectionSlot, Operations, RealmProfile, WindowInterface};

mod callback_objects;
mod cssom;
mod dom_exception;
mod functions;
mod native;
mod operations;
#[cfg(test)]
mod tests;

#[derive(Clone)]
enum Handle {
    Value(JsValue),
    Promise(JsPromise, ResolvingFunctions),
    Managed(JsObject, BoaRealm),
}

struct RootRecord {
    identity: Weak<RootIdentity>,
    handle: OnceCell<Handle>,
    _backing: Option<Rc<RootBacking>>,
}

struct PreparedRoot {
    root: Root,
    record: Rc<RootRecord>,
    table: Weak<State>,
}

impl PreparedRoot {
    fn finish(self, handle: Handle) -> Result<Root, Error> {
        self.record
            .handle
            .set(handle)
            .map_err(|_| Error::RegistryConflict)?;
        Ok(self.root.clone())
    }
}

impl Drop for PreparedRoot {
    fn drop(&mut self) {
        if self.record.handle.get().is_none()
            && let Some(state) = self.table.upgrade()
        {
            let _ = state.discard_root(&self.root);
        }
    }
}

#[derive(Clone)]
struct WindowObjects {
    prototype: JsObject,
    event_target: JsObject,
    names: JsObject,
}

struct RealmRecord {
    identity: Weak<RealmIdentity>,
    external: Option<BoaRealm>,
    realm: WeakRealm,
    window: Option<WindowObjects>,
    child_payload: Option<Weak<dyn Any>>,
    _backing: Option<Rc<RealmBacking>>,
}

struct State {
    self_weak: Weak<State>,
    owner: Rc<OwnerState>,
    roots: RefCell<BTreeMap<u64, Rc<RootRecord>>>,
    realms: RefCell<BTreeMap<u64, RealmRecord>>,
    next_root: Cell<u64>,
    next_realm: Cell<u64>,
    globals: Rc<Globals>,
    callbacks: callback_objects::CallbackObjects,
    native: native::NativeObjects,
    callables: functions::BoaCallables,
    dom_exceptions: dom_exception::DomExceptions,
}

impl State {
    fn new(owner: u64, globals: Rc<Globals>) -> Rc<Self> {
        Rc::new_cyclic(|weak| Self {
            self_weak: weak.clone(),
            owner: Rc::new(OwnerState::new(owner)),
            roots: RefCell::new(BTreeMap::new()),
            realms: RefCell::new(BTreeMap::new()),
            next_root: Cell::new(0),
            next_realm: Cell::new(0),
            globals,
            callbacks: Default::default(),
            native: Default::default(),
            callables: functions::BoaCallables::new(),
            dom_exceptions: Default::default(),
        })
    }

    fn create_realm(
        &self,
        context: &mut Context,
        origin: Option<WindowOrigin>,
        backing: Option<Rc<RealmBacking>>,
        child_payload: Option<Weak<dyn Any>>,
    ) -> Result<Realm, Error> {
        self.owner.available()?;
        let index = self
            .next_realm
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let window = origin.is_some();
        let prior = context.realm().clone();
        self.globals.origin.set(origin);
        let realm = context.create_realm();
        self.globals.origin.set(None);
        self.globals.prepared.set(None);
        context.enter_realm(prior);
        let realm = realm.map_err(|_| Error::BackendUnavailable)?;
        let key = RealmKey {
            owner: self.owner.id(),
            index,
        };
        let token = Realm(Rc::new(RealmIdentity {
            key,
            _backing: backing.clone(),
        }));
        let window = {
            let frame = EnterRealm::new(context, realm.clone());
            let global = frame.context.global_object();
            if window {
                let prototype = frame
                    .context
                    .intrinsics()
                    .constructors()
                    .object()
                    .prototype();
                let root_shape = frame.context.root_shape();
                let event_target = immutable_prototype(root_shape, Some(prototype));
                let names = window_properties(root_shape, global.clone(), event_target.clone());
                let prototype = immutable_prototype(root_shape, Some(names.clone()));
                prototype.insert_property(
                    JsSymbol::to_string_tag(),
                    BoaDescriptor::builder()
                        .value(JsString::from("Window"))
                        .writable(false)
                        .enumerable(false)
                        .configurable(true),
                );
                if !global.set_prototype(Some(prototype.clone())) {
                    return Err(Error::PropertyRejected);
                }
                Some(WindowObjects {
                    prototype,
                    event_target,
                    names,
                })
            } else {
                None
            }
        };
        self.realms
            .try_borrow_mut()
            .map_err(|_| self.conflict())?
            .insert(
                index,
                RealmRecord {
                    identity: Rc::downgrade(&token.0),
                    realm: realm.downgrade(),
                    external: Some(realm),
                    window,
                    child_payload,
                    _backing: backing,
                },
            );
        self.next_realm.set(index);
        Ok(token)
    }

    fn conflict(&self) -> Error {
        self.owner.fail(Error::RegistryConflict)
    }

    fn realm_storage_bytes() -> usize {
        size_of::<RealmIdentity>()
            + size_of::<RealmRecord>()
            + size_of::<(u64, RealmRecord)>()
            + size_of::<RealmBacking>()
    }

    fn prepare(&self, realm: RealmKey) -> Result<PreparedRoot, Error> {
        self.owner.available()?;
        if realm.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let backing = self
            .owner
            .preparation
            .get()
            .map(|account| {
                account.admit(Request::RootRecord {
                    bytes: Self::root_storage_bytes(),
                })
            })
            .transpose()?;
        self.prepare_funded(realm, backing)
    }

    fn prepare_funded(
        &self,
        realm: RealmKey,
        backing: Option<Guard>,
    ) -> Result<PreparedRoot, Error> {
        self.owner.available()?;
        if realm.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        let index = self
            .next_root
            .get()
            .checked_add(1)
            .ok_or(Error::IdentityExhausted)?;
        let backing = backing.map(RootBacking::new);
        let root = Root(Rc::new(RootIdentity {
            realm,
            index,
            _backing: backing.clone(),
        }));
        let record = Rc::new(RootRecord {
            identity: Rc::downgrade(&root.0),
            handle: OnceCell::new(),
            _backing: backing,
        });
        self.roots
            .try_borrow_mut()
            .map_err(|_| self.conflict())?
            .insert(index, record.clone());
        self.next_root.set(index);
        Ok(PreparedRoot {
            root,
            record,
            table: self.self_weak.clone(),
        })
    }

    fn root_storage_bytes() -> usize {
        size_of::<RootIdentity>()
            + size_of::<RootRecord>()
            + size_of::<(u64, Rc<RootRecord>)>()
            + size_of::<RootBacking>()
    }

    fn retain(&self, realm: RealmKey, handle: Handle) -> Result<Root, Error> {
        self.prepare(realm)?.finish(handle)
    }

    fn retain_funded(
        &self,
        realm: RealmKey,
        handle: Handle,
        backing: Option<Guard>,
    ) -> Result<Root, Error> {
        match backing {
            Some(backing) => self.prepare_funded(realm, Some(backing)),
            None => self.prepare(realm),
        }?
        .finish(handle)
    }

    fn entry(&self, root: &Root) -> Result<Rc<RootRecord>, Error> {
        if root.0.realm.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        self.roots
            .try_borrow()
            .map_err(|_| self.conflict())?
            .get(&root.0.index)
            .filter(|entry| entry.identity.ptr_eq(&Rc::downgrade(&root.0)))
            .cloned()
            .ok_or(Error::ExpiredIdentity)
    }

    fn discard_root(&self, root: &Root) -> Result<(), Error> {
        self.entry(root)?;
        let removed = self
            .roots
            .try_borrow_mut()
            .map_err(|_| self.conflict())?
            .remove(&root.0.index);
        drop(removed);
        Ok(())
    }

    fn value(&self, root: &Root) -> Result<JsValue, Error> {
        match self.entry(root)?.handle.get() {
            Some(Handle::Value(value)) => Ok(value.clone()),
            _ => Err(Error::ExpectedObject),
        }
    }

    fn promise(&self, root: &Root) -> Result<(JsPromise, ResolvingFunctions), Error> {
        match self.entry(root)?.handle.get() {
            Some(Handle::Promise(promise, resolver)) => Ok((promise.clone(), resolver.clone())),
            _ => Err(Error::ExpectedObject),
        }
    }

    fn lookup_realm(&self, key: RealmKey) -> Result<BoaRealm, Error> {
        if key.owner != self.owner.id() {
            return Err(Error::ForeignOwner);
        }
        self.realms
            .try_borrow()
            .map_err(|_| self.conflict())?
            .get(&key.index)
            .and_then(|record| record.realm.upgrade())
            .ok_or(Error::ExpiredIdentity)
    }

    fn realm_key(&self, realm: &BoaRealm) -> Result<RealmKey, Error> {
        self.realms
            .try_borrow()
            .map_err(|_| self.conflict())?
            .iter()
            .find(|(_, record)| record.realm.upgrade().as_ref() == Some(realm))
            .map(|(&index, _)| RealmKey {
                owner: self.owner.id(),
                index,
            })
            .ok_or(Error::CallbackRealmUnavailable)
    }

    fn current_key(&self, context: &Context) -> Result<RealmKey, Error> {
        self.realm_key(context.realm())
    }

    fn reap(&self) {
        loop {
            let retired = {
                let Ok(mut roots) = self.roots.try_borrow_mut() else {
                    self.conflict();
                    return;
                };
                let index = roots.iter().find_map(|(&index, entry)| {
                    (entry.identity.strong_count() == 0).then_some(index)
                });
                index.and_then(|index| roots.remove(&index))
            };
            if retired.is_none() {
                break;
            }
            drop(retired);
        }
        self.callbacks.reap(self);
        self.native.reap(self);
        self.dom_exceptions.reap(self);
        if let Err(error) = self.callables.reap() {
            self.owner.fail(error);
        }
        if let Ok(mut realms) = self.realms.try_borrow_mut() {
            realms.retain(|_, entry| {
                if entry.identity.strong_count() == 0 {
                    entry.external = None;
                    entry.window = None;
                }
                entry.realm.upgrade().is_some()
            });
        } else {
            self.conflict();
        }
        self.owner.reap_conversion_cleanup();
    }
}

#[derive(Default)]
struct Globals {
    origin: Cell<Option<WindowOrigin>>,
    prepared: Cell<Option<JsObject>>,
}

impl HostHooks for Globals {
    fn create_global_object(&self, intrinsics: &Intrinsics) -> JsObject {
        if let Some(origin) = self.origin.take() {
            let window = immutable_prototype(
                intrinsics.root_shape(),
                Some(intrinsics.constructors().object().prototype()),
            );
            self.prepared.set(Some(window_proxy(
                intrinsics.root_shape(),
                window.clone(),
                origin,
            )));
            window
        } else {
            JsObject::with_object_proto(intrinsics)
        }
    }

    fn create_global_this(&self, _: &Intrinsics) -> Option<JsObject> {
        self.prepared.take()
    }
}

pub(crate) struct Engine {
    state: Rc<State>,
    context: Context,
    _lease: BackendLease,
}

impl Engine {
    pub(crate) fn bootstrap() -> Result<Self, Error> {
        Self::bootstrap_inner(|| Ok::<_, Error>(())).map(|(engine, ())| engine)
    }

    pub(crate) fn bootstrap_funded<B: 'static, E: From<Error>>(
        admit: impl FnOnce(Request) -> Result<B, E>,
    ) -> Result<(Self, B), E> {
        Self::bootstrap_inner(|| {
            let bytes = size_of::<Self>()
                .checked_add(
                    size_of::<State>()
                        + size_of::<OwnerState>()
                        + size_of::<Globals>()
                        + functions::BoaCallables::EMERGENCY_BYTES,
                )
                .and_then(|bytes| bytes.checked_add(size_of::<B>()))
                .ok_or(Error::Allocation)?;
            admit(Request::Process { bytes })
        })
    }

    fn bootstrap_inner<B, E: From<Error>>(
        acquire: impl FnOnce() -> Result<B, E>,
    ) -> Result<(Self, B), E> {
        let lease = BackendLease::acquire()?;
        let backing = acquire()?;
        let owner = crate::owner::next_backend_id()?;
        let globals = Rc::new(Globals::default());
        let context = Context::builder()
            .host_hooks(globals.clone())
            .module_loader(Rc::new(IdleModuleLoader))
            .build()
            .map_err(|_| Error::BackendUnavailable)?;
        Ok((
            Self {
                state: State::new(owner, globals),
                context,
                _lease: lease,
            },
            backing,
        ))
    }
}

impl super::Engine for Engine {
    fn owner_state(&self) -> Rc<OwnerState> {
        self.state.owner.clone()
    }

    fn observe_heap(&mut self) -> Result<HeapObservation, Error> {
        self.state.owner.available()?;
        let sample = boa_gc::heap_statistics().ok_or(Error::BackendUnavailable)?;
        Ok(HeapObservation {
            used_bytes: sample.bytes_allocated,
        })
    }

    fn prepare_realm(&mut self) -> Result<(), Error> {
        self.state.owner.available()
    }

    fn create_realm_with_backing(
        &mut self,
        profile: RealmProfile,
        backing: Option<Rc<RealmBacking>>,
    ) -> Result<Realm, Error> {
        let origin = matches!(profile, RealmProfile::Window).then(WindowOrigin::default);
        self.state
            .create_realm(&mut self.context, origin, backing, None)
    }

    fn with_realm(
        &mut self,
        token: &Realm,
        action: &mut dyn FnMut(&mut dyn Operations) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.state.owner.available()?;
        let key = token.0.key;
        let realm = self.state.lookup_realm(key)?;
        let frame = EnterRealm::new(&mut self.context, realm);
        action(&mut Entered {
            context: &mut *frame.context,
            state: self.state.clone(),
            realm: key,
        })
    }

    fn checkpoint(&mut self) -> Result<(), Error> {
        self.state.owner.available()?;
        let result = self.context.run_jobs();
        // The host job ends even when there were no queued microtasks.
        self.context.clear_kept_objects();
        self.state.reap();
        self.state.owner.available()?;
        result.map_err(|_| Error::BackendUnavailable)
    }

    fn memory_pressure(&mut self) {
        if self.state.owner.available().is_ok() {
            self.state.reap();
            boa_gc::force_collect();
            self.state.reap();
        }
    }

    fn realm_storage_bytes(&self) -> usize {
        State::realm_storage_bytes()
    }

    fn managed_object_storage_bytes(&self) -> usize {
        State::root_storage_bytes()
    }

    fn release_managed_object(&mut self, holder: Root) -> Result<(), Error> {
        if !matches!(
            self.state.entry(&holder)?.handle.get(),
            Some(Handle::Managed(..))
        ) {
            return Err(self.state.conflict());
        }
        self.state.discard_root(&holder)
    }

    fn release_callback_object(
        &mut self,
        holder: &Rc<CallbackObjectIdentity>,
    ) -> Result<(), Error> {
        self.state.callbacks.release(&self.state, holder)
    }

    fn reap(&mut self) {
        self.state.reap();
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.state.owner.close();
        if let Err(error) = self.state.callables.drain() {
            self.state.owner.fail(error);
        }
    }
}

struct EnterRealm<'context> {
    context: &'context mut Context,
    prior: Option<BoaRealm>,
}

impl<'context> EnterRealm<'context> {
    fn new(context: &'context mut Context, realm: BoaRealm) -> Self {
        let prior = context.enter_realm(realm);
        Self {
            context,
            prior: Some(prior),
        }
    }
}

impl Drop for EnterRealm<'_> {
    fn drop(&mut self) {
        if let Some(prior) = self.prior.take() {
            self.context.enter_realm(prior);
        }
    }
}

struct Entered<'a> {
    context: &'a mut Context,
    state: Rc<State>,
    realm: RealmKey,
}

impl Entered<'_> {
    fn new_handle(
        &mut self,
        make: impl FnOnce(&mut Context) -> Result<Handle, Error>,
    ) -> Result<Root, Error> {
        let prepared = self.state.prepare(self.realm)?;
        prepared.finish(make(self.context)?)
    }

    fn root(&self, value: JsValue) -> Result<Root, Error> {
        self.state.retain(self.realm, Handle::Value(value))
    }

    fn funded(&self, value: JsValue, backing: Guard) -> Result<Root, Error> {
        self.state
            .retain_funded(self.realm, Handle::Value(value), Some(backing))
    }

    fn value(&self, root: &Root) -> Result<JsValue, Error> {
        self.check_active()?;
        self.state.value(root)
    }

    fn object_value(&self, root: &Root) -> Result<JsObject, Error> {
        self.value(root)?.as_object().ok_or(Error::ExpectedObject)
    }

    fn capture<T>(
        &mut self,
        phase: ExceptionPhase,
        error: JsError,
        prepared: PreparedRoot,
    ) -> Result<T, Error> {
        self.state.owner.available()?;
        let value = error
            .into_opaque(self.context)
            .map_err(|_| Error::BackendUnavailable)?;
        Err(Error::Exception(Exception {
            phase,
            value: prepared.finish(Handle::Value(value))?,
            rule: None,
        }))
    }

    fn attempt<T>(
        &mut self,
        phase: ExceptionPhase,
        action: impl FnOnce(&mut Context) -> JsResult<T>,
    ) -> Result<T, Error> {
        let prepared = self.state.prepare(self.realm)?;
        let result = action(self.context);
        self.state.owner.available()?;
        match result {
            Ok(result) => Ok(result),
            Err(error) => self.capture(phase, error, prepared),
        }
    }

    fn attempt_value(
        &mut self,
        phase: ExceptionPhase,
        action: impl FnOnce(&mut Context) -> JsResult<JsValue>,
    ) -> Result<Root, Error> {
        let prepared = self.state.prepare(self.realm)?;
        self.finish_value(prepared, phase, action)
    }

    fn finish_value(
        &mut self,
        prepared: PreparedRoot,
        phase: ExceptionPhase,
        action: impl FnOnce(&mut Context) -> JsResult<JsValue>,
    ) -> Result<Root, Error> {
        let result = action(self.context);
        self.state.owner.available()?;
        match result {
            Ok(result) => prepared.finish(Handle::Value(result)),
            Err(error) => self.capture(phase, error, prepared),
        }
    }
}

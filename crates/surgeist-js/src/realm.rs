//! Boa's implementation of ECMAScript's `Realm Records`
//!
//! Conceptually, a realm consists of a set of intrinsic objects, an ECMAScript global environment,
//! all of the ECMAScript code that is loaded within the scope of that global environment,
//! and other associated state and resources.
//!
//! A realm is represented in this implementation as a Realm struct with the fields specified from the spec.

use crate::ContextAccess;
use std::{
    any::TypeId,
    cell::{Cell, Ref, RefCell},
};

use crate::{
    HostDefined, JsNativeError, JsObject, JsResult, JsString,
    class::{Class, ClassConstructor},
    context::{
        HostHooks,
        intrinsics::{Intrinsics, StandardConstructor},
    },
    environments::{DeclarativeEnvironment, DeclarativeEnvironmentEdge, DeclarativeEnvironmentRef},
    module::{Module, ModuleEdge},
    object::{
        NativeObject, ObjectEdge,
        shape::{RootShape, SharedShape},
    },
};
use rustc_hash::FxHashMap;
use surgeist_js::JsValue;
use surgeist_js::property::{Attribute, PropertyDescriptor, PropertyKey};
use surgeist_js_ast::scope::Scope;
use surgeist_js_gc::heap::{RecordRef, Rooted, Trace as HeapTrace};

/// Representation of a Realm.
///
/// In the specification these are called Realm Records.
#[derive(Clone)]
pub struct Realm {
    inner: Rooted<RealmRecord>,
}

impl crate::native_function::NativeCapture for Realm {
    type Stored = crate::native_function::CapturedRealm;

    fn store(&self) -> Self::Stored {
        crate::native_function::CapturedRealm(self.edge())
    }

    fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
        use crate::error::RootingError;

        let inner = stored
            .0
            .0
            .upgrade()
            .map_err(RootingError::Access)?
            .ok_or(RootingError::Expired)?;
        Ok(Self { inner })
    }
}

impl Eq for Realm {}

impl PartialEq for Realm {
    fn eq(&self, other: &Self) -> bool {
        self.inner.handle() == other.inner.handle()
    }
}

impl std::fmt::Debug for Realm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Realm")
            .field("intrinsics", &self.inner.intrinsics)
            .field("environment", &self.inner.environment)
            .field("global_object", &self.global_object())
            .field("global_this", &self.global_this())
            .finish()
    }
}

#[derive(Clone, Debug, HeapTrace)]
pub(crate) struct RealmEdge(RecordRef<RealmRecord>);

impl RealmEdge {
    fn root(&self) -> Realm {
        Realm {
            inner: self
                .0
                .upgrade()
                .expect("realm heap is available")
                .expect("live owner retains its realm"),
        }
    }
}

#[derive(HeapTrace)]
pub(crate) struct RealmRecord {
    intrinsics: Intrinsics,

    /// The global declarative environment of this realm.
    environment: DeclarativeEnvironmentEdge,

    /// The global scope of this realm.
    /// This is directly related to the global declarative environment.
    // Safety: Nothing in `Scope` needs tracing.
    #[heap_trace(skip)]
    scope: Scope,

    global_object: ObjectEdge,
    global_this: ObjectEdge,
    // The high half of each template-site key counts sites within a parse.
    #[heap_trace(skip)]
    parser_identifier: Cell<u32>,
    template_map: RefCell<FxHashMap<u64, ObjectEdge>>,
    loaded_modules: RefCell<FxHashMap<JsString, ModuleEdge>>,
    host_classes: RefCell<FxHashMap<TypeId, StandardConstructor>>,

    host_defined: RefCell<HostDefined>,
}

impl std::fmt::Debug for RealmRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealmRecord").finish_non_exhaustive()
    }
}

/// Keeps intrinsic shells alive until the realm owns their stored edges.
#[derive(Default)]
pub(crate) struct BootstrapRoots {
    objects: Vec<JsObject>,
    shapes: Vec<SharedShape>,
}

impl BootstrapRoots {
    pub(crate) fn retain(&mut self, object: JsObject) -> ObjectEdge {
        let edge = object.edge();
        self.objects.push(object);
        edge
    }

    pub(crate) fn retain_shape(&mut self, shape: SharedShape) {
        self.shapes.push(shape);
    }
}

impl Realm {
    pub(crate) fn runtime(&self) -> crate::Runtime {
        crate::Runtime {
            heap: self.inner.runtime().clone(),
        }
    }

    pub(crate) fn from_edge(inner: RealmEdge) -> Self {
        inner.root()
    }

    /// Resolves a function's realm, reusing this root when the target is already active.
    #[inline]
    pub(crate) fn resolve_call_realm(&self, target: Option<&RealmEdge>) -> Self {
        match target {
            Some(edge) if edge.0.handle() != self.inner.handle() => edge.root(),
            _ => self.clone(),
        }
    }

    pub(crate) fn edge(&self) -> RealmEdge {
        RealmEdge(self.inner.record_ref())
    }

    /// Create a new [`Realm`].
    #[inline]
    pub fn create(hooks: &dyn HostHooks, root_shape: &RootShape) -> JsResult<Self> {
        let mut roots = BootstrapRoots::default();
        let intrinsics = Intrinsics::uninit(root_shape, &mut roots).ok_or_else(|| {
            JsNativeError::typ().with_message("failed to create the realm intrinsics")
        })?;

        let global_object = hooks.create_global_object(&intrinsics);
        let global_this = hooks
            .create_global_this(&intrinsics)
            .unwrap_or_else(|| global_object.clone());
        if ![&global_object, &global_this]
            .into_iter()
            .all(|object| root_shape.runtime().same_heap(&object.runtime()))
        {
            return Err(JsNativeError::typ()
                .with_message("host global objects must belong to the realm runtime")
                .into());
        }
        let environment = root_shape
            .runtime()
            .heap
            .allocate(DeclarativeEnvironment::global())
            .expect("global environment has valid heap edges");
        let scope = Scope::new_global();

        let realm = Self {
            inner: root_shape
                .runtime()
                .heap
                .allocate(RealmRecord {
                    intrinsics,
                    environment: environment.record_ref(),
                    scope,
                    global_object: roots.retain(global_object),
                    global_this: roots.retain(global_this),
                    parser_identifier: Cell::new(0),
                    template_map: RefCell::default(),
                    loaded_modules: RefCell::default(),
                    host_classes: RefCell::default(),
                    host_defined: RefCell::default(),
                })
                .expect("realm has valid heap edges"),
        };

        drop(roots);
        realm.initialize();

        Ok(realm)
    }

    /// Gets the intrinsics of this `Realm`.
    #[inline]
    #[must_use]
    pub fn intrinsics(&self) -> &Intrinsics {
        &self.inner.intrinsics
    }

    /// Returns an immutable reference to the [`ECMAScript specification`][spec] defined
    /// [`\[\[\HostDefined]\]`][`HostDefined`] field of the [`Realm`].
    ///
    /// [spec]: https://tc39.es/ecma262/#table-realm-record-fields
    ///
    /// # Panics
    ///
    /// Panics if [`HostDefined`] field is mutably borrowed.
    #[inline]
    #[must_use]
    pub fn host_defined(&self) -> Ref<'_, HostDefined> {
        self.inner.host_defined.borrow()
    }

    /// Installs a host value for this realm, replacing the previous value of its type.
    ///
    /// Any subsequent interior mutation of the value must preserve the trace
    /// contract and keep its stored edges live in this realm's runtime.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the host data is borrowed or the heap rejects
    /// an edge reported by the value.
    pub fn insert_host_defined<T: NativeObject>(&self, value: T) -> JsResult<()> {
        self.runtime()
            .heap
            .with_pending_edges(value, |value| {
                let previous = self
                    .inner
                    .host_defined
                    .try_borrow_mut()
                    .map_err(|_| JsNativeError::typ().with_message("realm host data is borrowed"))?
                    .insert(value);
                drop(previous);
                Ok(())
            })
            .map_err(|error| JsNativeError::typ().with_message(error.to_string()))?
    }

    /// Checks if this `Realm` has the class `C` registered into its class map.
    #[must_use]
    pub fn has_class<C: Class>(&self) -> bool {
        self.inner
            .host_classes
            .borrow()
            .contains_key(&TypeId::of::<C>())
    }

    /// Gets the constructor and prototype of the class `C` if it is registered in the class map.
    #[must_use]
    pub fn get_class<C: Class>(&self) -> Option<ClassConstructor> {
        self.inner
            .host_classes
            .borrow()
            .get(&TypeId::of::<C>())
            .map(ClassConstructor::from_stored)
    }

    pub(crate) fn environment(&self) -> DeclarativeEnvironmentRef {
        self.inner
            .environment
            .upgrade()
            .expect("environment heap is available")
            .expect("live realm retains its environment")
    }

    /// Returns the scope of this realm.
    #[must_use]
    pub fn scope(&self) -> &Scope {
        &self.inner.scope
    }

    pub(crate) fn global_object(&self) -> JsObject {
        JsObject::from_inner(self.inner.global_object.clone())
    }

    pub(crate) fn global_this(&self) -> JsObject {
        JsObject::from_inner(self.inner.global_this.clone())
    }

    pub(crate) fn insert_loaded_module(&self, specifier: JsString, module: &Module) {
        let mut loaded_modules = self.inner.loaded_modules.borrow_mut();
        let edge = loaded_modules
            .entry(specifier)
            .or_insert_with(|| module.edge());
        debug_assert_eq!(edge, &module.edge());
    }

    /// Prepares global storage without publishing new environment slots.
    pub(crate) fn prepare_global_env(
        &self,
        binding_count: u32,
    ) -> JsResult<PreparedGlobalEnvironment> {
        let environment = self.environment();
        let global = environment
            .kind()
            .as_global()
            .expect("realm owns a global environment");
        let extends = global.prepare_bindings(binding_count as usize)?;
        Ok(PreparedGlobalEnvironment {
            realm: self.clone(),
            binding_count: binding_count as usize,
            extends,
        })
    }

    /// Resizes the number of bindings on the global environment.
    pub(crate) fn resize_global_env(&self) {
        let binding_number = self.scope().num_bindings();
        let environment = self.environment();
        let env = environment
            .kind()
            .as_global()
            .expect("Realm should only store global environments");
        env.extend_bindings(binding_number as usize);
    }

    pub(crate) fn next_parser_identifier(&self) -> JsResult<u32> {
        let next = self
            .inner
            .parser_identifier
            .get()
            .checked_add(1)
            .ok_or_else(|| {
                JsNativeError::range().with_message("realm parser identifiers exhausted")
            })?;
        self.inner.parser_identifier.set(next);
        Ok(next)
    }

    pub(crate) fn push_template(&self, site: u64, template: JsObject) {
        self.inner
            .template_map
            .borrow_mut()
            .insert(site, template.into_record());
    }

    pub(crate) fn lookup_template(&self, site: u64) -> Option<JsObject> {
        self.inner
            .template_map
            .borrow()
            .get(&site)
            .cloned()
            .map(JsObject::from_inner)
    }

    /// Register a property on the global object of this realm.
    ///
    /// It will return an error if the property is already defined.
    pub fn register_property<K, V>(
        &self,
        key: K,
        value: V,
        attribute: Attribute,
        context: ContextAccess<'_>,
    ) -> JsResult<()>
    where
        K: Into<PropertyKey>,
        V: Into<JsValue>,
    {
        let context = context.into_owner();
        self.global_object().define_property_or_throw(
            key,
            PropertyDescriptor::builder()
                .value(value)
                .writable(attribute.writable())
                .enumerable(attribute.enumerable())
                .configurable(attribute.configurable()),
            context.access(),
        )?;
        Ok(())
    }

    /// Register a class `C` in this realm.
    pub fn register_class<C: Class>(&self, spec: ClassConstructor) -> JsResult<()> {
        let runtime = self.runtime();
        spec.constructor().check_runtime(&runtime)?;
        spec.prototype().check_runtime(&runtime)?;
        self.inner
            .host_classes
            .borrow_mut()
            .insert(TypeId::of::<C>(), spec.stored());
        Ok(())
    }

    /// Unregister a class `C` in this realm.
    #[must_use]
    pub fn unregister_class<C: Class>(&self) -> Option<ClassConstructor> {
        self.inner
            .host_classes
            .borrow_mut()
            .remove(&TypeId::of::<C>())
            .as_ref()
            .map(ClassConstructor::from_stored)
    }

    pub(crate) fn identity(&self) -> surgeist_js_gc::heap::ObjectId {
        self.inner.handle().erase()
    }
}

/// Capacity prepared for an admitted, callback-free global declaration transition.
pub(crate) struct PreparedGlobalEnvironment {
    realm: Realm,
    binding_count: usize,
    extends: bool,
}

impl PreparedGlobalEnvironment {
    pub(crate) fn creates_bindings(&self) -> bool {
        self.extends
    }

    pub(crate) fn install(self) {
        let environment = self.realm.environment();
        environment
            .kind()
            .as_global()
            .expect("realm owns a global environment")
            .install_prepared_bindings(self.binding_count);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Context, JsNativeError, JsValue, Source,
        builtins::dataview::DataView,
        object::builtins::{JsArrayBuffer, JsDataView},
    };

    #[test]
    fn builtin_function_slots_are_ordered_and_isolated_between_realms() {
        let mut context = Context::default();
        let source = "Array.prototype.slice";
        let first = context.eval(Source::from_bytes(source)).unwrap();
        let second_realm = context.create_realm().unwrap();
        let first_realm = context.enter_realm(second_realm).unwrap();
        let second = context.eval(Source::from_bytes(source)).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "Array.prototype.slice.name === 'slice' && Array.prototype.slice.length === 2",
                ))
                .unwrap(),
            JsValue::from(true),
        );
        context
            .eval(Source::from_bytes(
                "Object.defineProperty(Array.prototype.slice, 'name', {value: 'changed'})",
            ))
            .unwrap();

        context.enter_realm(first_realm).unwrap();
        assert_eq!(
            context
                .eval(Source::from_bytes("Array.prototype.slice.name === 'slice'"))
                .unwrap(),
            JsValue::from(true),
        );
    }

    #[test]
    fn realm_host_data_retains_local_edges_and_rejects_foreign_edges() {
        let mut owner = Context::default();
        let other = Context::default();
        let buffer = JsArrayBuffer::new(8, owner.access()).unwrap();
        let buffer_edge = buffer.edge();
        let view = JsDataView::from_js_array_buffer(buffer, None, None, owner.access()).unwrap();
        let view_edge = view.edge();
        let payload = view.borrow().data().clone();

        let error = other
            .realm()
            .insert_host_defined(payload.clone())
            .unwrap_err();
        assert!(error.as_native().is_some_and(JsNativeError::is_type));
        assert!(other.realm().host_defined().get::<DataView>().is_none());
        other.runtime().collect().unwrap();

        owner.realm().insert_host_defined(payload).unwrap();
        drop(view);
        owner.runtime().collect().unwrap();
        assert!(!view_edge.is_live().unwrap());
        assert!(buffer_edge.is_live().unwrap());
    }
}

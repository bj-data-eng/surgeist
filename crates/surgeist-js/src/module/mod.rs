//! Boa's implementation of the ECMAScript's module system.
//!
//! This module contains the [`Module`] type, which represents an [**Abstract Module Record**][module],
//! a [`ModuleLoader`] trait for custom module loader implementations, and [`SimpleModuleLoader`],
//! the default `ModuleLoader` for [`Context`] which can be used for most simple usecases.
//!
//! Every module roughly follows the same lifecycle:
//! - Parse using [`Module::parse`].
//! - Load all its dependencies using [`Module::load`].
//! - Link its dependencies together using [`Module::link`].
//! - Evaluate the module and its dependencies using [`Module::evaluate`].
//!
//! The [`ModuleLoader`] trait allows customizing the "load" step on the lifecycle
//! of a module, which allows doing things like fetching modules from urls, having multiple
//! "modpaths" from where to import modules, or using Rust futures to avoid blocking the main thread
//! on loads.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-modules
//! [module]: https://tc39.es/ecma262/#sec-abstract-module-records

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rustc_hash::FxHashSet;

use surgeist_js::js_string;
use surgeist_js::property::PropertyKey;
use surgeist_js::value::TryFromJs;
use surgeist_js_gc::heap::{RecordRef, Rooted, Trace as HeapTrace};
use surgeist_js_interner::Interner;
use surgeist_js_parser::Source;
use surgeist_js_parser::source::ReadChar;

pub use loader::*;
pub use namespace::ModuleNamespace;
pub(crate) use source::ModuleActivation;
use source::SourceTextModule;
pub use synthetic::{SyntheticModule, SyntheticModuleInitializer};

use crate::object::TypedJsFunction;
use crate::spanned_source_text::SourceText;
use crate::{
    Context, ContextAccess, HostDefined, JsError, JsNativeError, JsResult, JsString, JsValue,
    NativeFunction, builtins,
    builtins::promise::{PromiseCapability, PromiseState},
    environments::DeclarativeEnvironmentRef,
    object::{JsObject, JsPromise, ObjectEdge},
    realm::{Realm, RealmEdge},
};

pub(crate) mod lifecycle;
/// Real guest completion callbacks owned before source evaluation can suspend.
pub(crate) struct NativeEvaluationBridge {
    pub(crate) target: builtins::promise::PromiseCapability,
    pub(crate) fulfilled: crate::object::JsFunction,
    pub(crate) rejected: crate::object::JsFunction,
}

mod loader;
pub(crate) mod metadata;
mod namespace;
mod source;
mod synthetic;

/// Import attribute.
///
/// [spec]: https://tc39.es/ecma262/#table-importattribute-fields
#[derive(Debug, Clone, PartialEq, Eq, Hash, HeapTrace)]
pub struct ImportAttribute {
    key: JsString,
    value: JsString,
}

impl ImportAttribute {
    /// Creates a new import attribute.
    #[must_use]
    pub fn new(key: JsString, value: JsString) -> Self {
        Self { key, value }
    }

    /// Gets the attribute key.
    #[must_use]
    pub fn key(&self) -> &JsString {
        &self.key
    }

    /// Gets the attribute value.
    #[must_use]
    pub fn value(&self) -> &JsString {
        &self.value
    }
}

/// A module request with optional import attributes.
///
/// Represents a module specifier and its associated import attributes.
/// According to the [ECMAScript specification][spec], the module cache key
/// should be (referrer, specifier, attributes).
///
/// [spec]: https://tc39.es/ecma262/#sec-modulerequest-record
#[derive(Debug, Clone, PartialEq, Eq, Hash, HeapTrace)]
pub struct ModuleRequest {
    specifier: JsString,
    #[heap_trace(skip)]
    attributes: Rc<Vec<ImportAttribute>>,
}

impl ModuleRequest {
    /// Creates a new module request from a specifier and attributes.
    #[must_use]
    pub fn new(specifier: JsString, mut attributes: Box<[ImportAttribute]>) -> Self {
        // Sort attributes by key to ensure canonical cache keys.
        attributes.sort_unstable_by(|k1, k2| k1.key.cmp(&k2.key));
        Self {
            specifier,
            attributes: Rc::new(attributes.into_vec()),
        }
    }

    /// Creates a new module request from only a specifier with no attributes.
    #[must_use]
    pub fn from_specifier(specifier: JsString) -> Self {
        Self {
            specifier,
            attributes: Rc::default(),
        }
    }

    /// Gets the module specifier.
    #[must_use]
    pub fn specifier(&self) -> &JsString {
        &self.specifier
    }

    /// Gets the import attributes as key-value pairs.
    #[must_use]
    pub fn attributes(&self) -> &[ImportAttribute] {
        &self.attributes
    }

    /// Gets the value of a specific attribute by key.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<&JsString> {
        self.attributes
            .iter()
            .find(|attr| attr.key == key)
            .map(|attr| &attr.value)
    }
}

/// ECMAScript's [**Abstract module record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
#[derive(Clone)]
pub struct Module {
    inner: Rooted<ModuleRepr>,
}

impl std::fmt::Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("realm", &self.inner.realm)
            .field("namespace", &self.inner.namespace)
            .field("kind", &self.inner.kind)
            .finish()
    }
}

/// Strong edge to a module owned by the safe runtime graph.
#[derive(Clone, Debug, Eq, PartialEq, HeapTrace)]
pub(crate) struct ModuleEdge(RecordRef<ModuleRepr>);

/// Native callback capture of a module allocation.
#[derive(Debug, HeapTrace)]
pub struct CapturedModule(pub(crate) ModuleEdge);

impl crate::native_function::NativeCapture for Module {
    type Stored = CapturedModule;

    fn store(&self) -> Self::Stored {
        CapturedModule(self.edge())
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

#[derive(HeapTrace)]
struct ModuleRepr {
    realm: RealmEdge,
    namespace: RefCell<Option<ObjectEdge>>,
    kind: ModuleKind,
    host_defined: HostDefined,
    #[heap_trace(skip)]
    path: Option<PathBuf>,
}

/// The kind of a [`Module`].
#[derive(Debug, HeapTrace)]
pub(crate) enum ModuleKind {
    /// A [**Source Text Module Record**](https://tc39.es/ecma262/#sec-source-text-module-records)
    SourceText(Box<SourceTextModule>),
    /// A [**Synthetic Module Record**](https://tc39.es/proposal-json-modules/#sec-synthetic-module-records)
    Synthetic(Box<SyntheticModule>),
}

impl ModuleKind {
    /// Returns the inner `SourceTextModule`.
    pub(crate) fn as_source_text(&self) -> Option<&SourceTextModule> {
        match self {
            ModuleKind::SourceText(src) => Some(src),
            ModuleKind::Synthetic(_) => None,
        }
    }
}

/// Return value of the [`Module::resolve_export`] operation.
///
/// Indicates how to access a specific export in a module.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedBinding {
    module: Module,
    binding_name: BindingName,
}

/// The local name of the resolved binding within its containing module.
///
/// Note that a resolved binding can resolve to a single binding inside a module (`export var a = 1"`)
/// or to a whole module namespace (`export * as ns from "mod.js"`).
#[derive(Debug, Clone, HeapTrace)]
pub(crate) enum BindingName {
    /// A local binding.
    Name(JsString),
    /// The whole namespace of the containing module.
    Namespace,
}

#[derive(Debug, HeapTrace)]
struct StoredResolvedBinding {
    module: ModuleEdge,
    binding_name: BindingName,
}

impl StoredResolvedBinding {
    fn to_binding(&self) -> ResolvedBinding {
        ResolvedBinding {
            module: Module::from_edge(self.module.clone()),
            binding_name: self.binding_name.clone(),
        }
    }
}

impl ResolvedBinding {
    fn stored(&self) -> StoredResolvedBinding {
        StoredResolvedBinding {
            module: self.module.edge(),
            binding_name: self.binding_name.clone(),
        }
    }

    /// Gets the module from which the export resolved.
    pub(crate) const fn module(&self) -> &Module {
        &self.module
    }

    /// Consumes `self` and returns the module from which the export resolved.
    pub(crate) fn into_module(self) -> Module {
        self.module
    }

    /// Gets a reference to the binding associated with the resolved export.
    pub(crate) const fn binding_name(&self) -> &BindingName {
        &self.binding_name
    }
}

#[derive(Debug, Clone)]
struct GraphLoadingState {
    capability: PromiseCapability,
    loading: Cell<bool>,
    pending_modules: Cell<usize>,
    visited: RefCell<HashSet<Module>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ResolveExportError {
    NotFound,
    Ambiguous,
}

impl Module {
    pub(crate) fn from_edge(edge: ModuleEdge) -> Self {
        Self {
            inner: edge
                .0
                .upgrade()
                .expect("module heap is available")
                .expect("live owner retains its module"),
        }
    }

    pub(crate) fn edge(&self) -> ModuleEdge {
        ModuleEdge(self.inner.record_ref())
    }

    fn resolve_realm(realm: Option<Realm>, context: &Context) -> JsResult<Realm> {
        let realm = realm.unwrap_or_else(|| context.realm().clone());
        if !context.runtime().same_heap(&realm.runtime()) {
            return Err(JsNativeError::typ()
                .with_message("module realm belongs to a different runtime")
                .into());
        }
        Ok(realm)
    }

    fn allocate(
        realm: &Realm,
        kind: ModuleKind,
        path: Option<PathBuf>,
        context: &Context,
    ) -> JsResult<Self> {
        let inner = context
            .runtime()
            .heap
            .allocate(ModuleRepr {
                realm: realm.edge(),
                namespace: RefCell::default(),
                kind,
                host_defined: HostDefined::default(),
                path,
            })
            .map_err(|rejected| JsNativeError::typ().with_message(rejected.reason.to_string()))?;
        Ok(Self { inner })
    }

    /// Abstract operation [`ParseModule ( sourceText, realm, hostDefined )`][spec].
    ///
    /// Parses the provided `src` as an ECMAScript module.
    ///
    /// Contexts from the module's runtime can link and evaluate it using its original
    /// source and realm.
    /// Parsing retains syntax without publishing bindings or spending shared scope
    /// identities. Scope analysis and compilation occur during linking.
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or the heap rejects the module. An explicit realm
    /// from another runtime returns a `TypeError` before parsing begins.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parsemodule
    pub fn parse<R: ReadChar>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        context.check_execution_stop()?;
        let realm = Self::resolve_realm(realm, context)?;
        let path = context
            .with_module_metadata(|context| metadata::Control::new(context).path(src.path()))?;
        let identifier = realm.next_parser_identifier()?;
        let mut interner = Interner::new();
        let (module, source) = context.parse_source(src, |mut parser| {
            parser.set_identifier(identifier);
            Ok(parser.parse_module_syntax_with_source(&mut interner)?)
        })?;
        context.with_module_metadata(|context| {
            let mut control = metadata::Control::new(context);
            control.header::<surgeist_js_ast::SourceText>()?;
            let source_text = SourceText::new(source);
            let source_path = control.path(path.as_deref())?;
            let (source, roots) =
                SourceTextModule::new(module, interner, source_text, source_path, &mut control)?;
            control.header::<ModuleRepr>()?;
            control.admit(metadata::Work::Publication)?;
            let module = Self::allocate(
                &realm,
                ModuleKind::SourceText(Box::new(source)),
                path,
                control.context(),
            )?;
            drop(roots);
            Ok(module)
        })
    }

    /// Abstract operation [`CreateSyntheticModule ( exportNames, evaluationSteps, realm )`][spec].
    ///
    /// Creates a new Synthetic Module from its list of exported names, its evaluation steps and
    /// optionally a root realm.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the realm or initializer captures belong to another runtime,
    /// or the heap rejects the module.
    ///
    /// [spec]: https://tc39.es/proposal-json-modules/#sec-createsyntheticmodule
    #[inline]
    pub fn synthetic(
        export_names: &[JsString],
        evaluation_steps: SyntheticModuleInitializer,
        path: Option<PathBuf>,
        realm: Option<Realm>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        context.check_execution_stop()?;
        let realm = Self::resolve_realm(realm, context)?;
        context.with_module_metadata(|context| {
            let mut control = metadata::Control::new(context);
            let mut names = metadata::Index::default();
            for name in export_names {
                control.admit(metadata::Work::Visit)?;
                let name = control.copy(name)?;
                names.insert(name, (), &mut control)?;
            }
            Self::publish_synthetic(names, &evaluation_steps, path, &realm, &mut control)
        })
    }

    fn publish_synthetic(
        names: metadata::Index<JsString>,
        initializer: &SyntheticModuleInitializer,
        path: Option<PathBuf>,
        realm: &Realm,
        control: &mut metadata::Control<'_>,
    ) -> JsResult<Self> {
        let synthetic = SyntheticModule::new(names, initializer, control)?;
        control.header::<ModuleRepr>()?;
        control.admit(metadata::Work::Publication)?;
        Self::allocate(
            realm,
            ModuleKind::Synthetic(Box::new(synthetic)),
            path,
            control.context(),
        )
    }

    /// Create a [`Module`] from a `JsValue`, exporting that value as the default export.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the value belongs to another runtime or the heap rejects the module.
    pub fn from_value_as_default(value: JsValue, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        context.check_execution_stop()?;
        context.with_module_metadata(|context| {
            let realm = context.realm().clone();
            let mut control = metadata::Control::new(context);
            let mut names = metadata::Index::default();
            names.insert(js_string!("default"), (), &mut control)?;
            let initializer = SyntheticModuleInitializer::prepare(
                |module, value, _| module.set_export(&js_string!("default"), value.clone()),
                value,
                &mut control,
            )?;
            Self::publish_synthetic(names, &initializer, None, &realm, &mut control)
        })
    }

    /// Create a module that exports a single JSON value as the default export, from its
    /// JSON string.
    ///
    /// # Specification
    /// This is a custom extension to the ECMAScript specification. The current proposal
    /// for JSON modules is being considered in <https://github.com/tc39/proposal-json-modules>
    /// and might differ from this implementation.
    ///
    /// This method is provided as a convenience for hosts to create JSON modules.
    ///
    /// # Errors
    /// This will return an error if the JSON string is invalid or cannot be converted.
    pub fn parse_json(json: JsString, context: ContextAccess<'_>) -> JsResult<Self> {
        let context = context.into_owner();
        context.check_execution_stop()?;
        let value = builtins::Json::parse(&JsValue::undefined(), &[json.into()], context)?;
        Self::from_value_as_default(value, context.access())
    }

    /// Gets the realm of this `Module`.
    #[inline]
    #[must_use]
    pub fn realm(&self) -> Realm {
        Realm::from_edge(self.inner.realm.clone())
    }

    /// Returns the [`ECMAScript specification`][spec] defined [`\[\[HostDefined\]\]`][`HostDefined`] field of the [`Module`].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-abstract-module-records
    #[inline]
    #[must_use]
    pub fn host_defined(&self) -> &HostDefined {
        &self.inner.host_defined
    }

    /// Gets the kind of this `Module`.
    pub(crate) fn kind(&self) -> &ModuleKind {
        &self.inner.kind
    }

    /// Gets the declarative environment of this `Module`.
    pub(crate) fn environment(&self) -> Option<DeclarativeEnvironmentRef> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.environment(),
            ModuleKind::Synthetic(syn) => syn.environment(),
        }
    }

    /// Abstract method [`LoadRequestedModules ( [ hostDefined ] )`][spec].
    ///
    /// Prepares the module for linking by loading all its module dependencies. Returns a `JsPromise`
    /// that will resolve when the loading process either completes or fails.
    ///
    /// # Errors
    /// Returns host errors from preparing the promise and retaining its loading jobs.
    /// Module loading failures continue to reject the returned promise.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[inline]
    pub fn load(&self, context: ContextAccess<'_>) -> JsResult<JsPromise> {
        let context = context.into_owner();
        if let Err(error) = self.check_runtime(context) {
            return JsPromise::reject(error, context.access());
        }
        match self.kind() {
            ModuleKind::SourceText(_) => {
                // Concrete method [`LoadRequestedModules ( [ hostDefined ] )`][spec].
                //
                // [spec]: https://tc39.es/ecma262/#sec-LoadRequestedModules
                // 1. If hostDefined is not present, let hostDefined be empty.

                // 2. Let pc be ! NewPromiseCapability(%Promise%).
                let pc = PromiseCapability::new(
                    &context.intrinsics().constructors().promise().constructor(),
                    context,
                )?;

                // 4. Perform InnerModuleLoading(state, module).
                self.inner_load(
                    // 3. Let state be the GraphLoadingState Record {
                    //     [[IsLoading]]: true, [[PendingModulesCount]]: 1, [[Visited]]: « »,
                    //     [[PromiseCapability]]: pc, [[HostDefined]]: hostDefined
                    // }.
                    &Rc::new(GraphLoadingState {
                        capability: pc.clone(),
                        loading: Cell::new(true),
                        pending_modules: Cell::new(1),
                        visited: RefCell::default(),
                    }),
                    context,
                )?;

                // 5. Return pc.[[Promise]].
                JsPromise::from_object(pc.promise().clone())
            }
            ModuleKind::Synthetic(_) => SyntheticModule::load(context),
        }
    }

    /// Abstract operation [`InnerModuleLoading`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLoading
    fn inner_load(&self, state: &Rc<GraphLoadingState>, context: &mut Context) -> JsResult<()> {
        // 1. Assert: state.[[IsLoading]] is true.
        assert!(state.loading.get());

        if let ModuleKind::SourceText(src) = self.kind() {
            // continues on `inner_load
            src.inner_load(self, state, context)?;
            if !state.loading.get() {
                return Ok(());
            }
        }

        // 3. Assert: state.[[PendingModulesCount]] ≥ 1.
        assert!(state.pending_modules.get() >= 1);

        // 4. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] - 1.
        state.pending_modules.set(state.pending_modules.get() - 1);
        // 5. If state.[[PendingModulesCount]] = 0, then

        if state.pending_modules.get() == 0 {
            // a. Set state.[[IsLoading]] to false.
            state.loading.set(false);
            // b. For each Cyclic Module Record loaded of state.[[Visited]], do
            //    i. If loaded.[[Status]] is new, set loaded.[[Status]] to unlinked.
            // By default, all modules start on `unlinked`.

            // c. Perform ! Call(state.[[PromiseCapability]].[[Resolve]], undefined, « undefined »).
            state
                .capability
                .resolve()
                .call(&JsValue::undefined(), &[], context.access())?;
        }
        // 6. Return unused.
        Ok(())
    }

    /// Abstract method [`GetExportedNames([exportStarSet])`][spec].
    ///
    /// Returns a list of all the names exported from this module.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    fn get_exported_names(&self, export_star_set: &mut Vec<Module>) -> FxHashSet<JsString> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.get_exported_names(self, export_star_set),
            ModuleKind::Synthetic(synth) => synth.get_exported_names(),
        }
    }

    /// Abstract method [`ResolveExport(exportName [, resolveSet])`][spec].
    ///
    /// Returns the corresponding local binding of a binding exported by this module.
    /// The spec requires that this operation must be idempotent; calling this multiple times
    /// with the same `export_name` and `resolve_set` should always return the same result.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::mutable_key_type)]
    pub(crate) fn resolve_export(
        &self,
        export_name: &JsString,
        resolve_set: &mut FxHashSet<(Self, JsString)>,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.resolve_export(self, export_name, resolve_set),
            ModuleKind::Synthetic(synth) => synth.resolve_export(self, export_name),
        }
    }

    /// Abstract method [`Link() `][spec].
    ///
    /// Prepares this module for evaluation by resolving all its module dependencies and initializing
    /// its environment.
    /// Newly linked dependencies prepare privately as one group. A preparation
    /// failure leaves new modules unlinked, retains source syntax for retry, and
    /// preserves previously linked modules.
    /// Linking creates no guest frame; invocation limits apply when code executes.
    ///
    /// # Note
    ///
    /// This must only be called if the [`JsPromise`] returned by [`Module::load`] has fulfilled.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[allow(clippy::missing_panics_doc)]
    #[inline]
    pub fn link(&self, context: ContextAccess<'_>) -> JsResult<()> {
        let context = context.into_owner();
        self.check_runtime(context)?;
        source::link(self, context)
    }

    /// Abstract method [`Evaluate()`][spec].
    ///
    /// Evaluates this module, returning a promise for the result of the evaluation of this module
    /// and its dependencies.
    /// If the promise is rejected, hosts are expected to handle the promise rejection and rethrow
    /// the evaluation error.
    ///
    /// When an engine failure invalidates an entered source-module evaluation,
    /// that instance becomes terminal. The initiating call returns its original
    /// engine error; later calls
    /// return [`crate::error::EngineError::ModuleInterrupted`] with the immutable
    /// interruption identity and typed original cause. Increasing a later budget
    /// cannot retry the instance. An already-issued guest promise stays pending;
    /// native promise waits observe the interruption instead.
    /// Unstarted deferred jobs and valid continuation transfers retain their
    /// custody when an enclosing operation stops.
    ///
    /// # Note
    ///
    /// This must only be called if the [`Module::link`] method finished successfully.
    ///
    /// [spec]: https://tc39.es/ecma262/#table-abstract-methods-of-module-records
    #[inline]
    pub fn evaluate(&self, context: ContextAccess<'_>) -> JsResult<JsPromise> {
        let context = context.into_owner();
        self.check_runtime(context)?;
        match self.kind() {
            ModuleKind::SourceText(src) => src.evaluate(self, None, context),
            ModuleKind::Synthetic(synth) => synth.evaluate(self, context),
        }
    }

    pub(crate) fn evaluate_for_native(
        &self,
        bridge: &NativeEvaluationBridge,
        context: &mut Context,
    ) -> JsResult<JsPromise> {
        self.check_runtime(context)?;
        context.check_execution_stop()?;
        match self.kind() {
            ModuleKind::SourceText(source) => source.evaluate(self, Some(bridge), context),
            ModuleKind::Synthetic(source) => {
                let evaluated = source.evaluate(self, context)?;
                builtins::Promise::perform_promise_then_with_native_target(
                    &evaluated,
                    Some(bridge.fulfilled.clone()),
                    Some(bridge.rejected.clone()),
                    None,
                    &bridge.target,
                    context,
                )
                .into_result()?;
                Ok(evaluated)
            }
        }
    }

    fn check_runtime(&self, context: &Context) -> JsResult<()> {
        if !self.inner.runtime().same_heap(&context.runtime().heap) {
            return Err(JsNativeError::typ()
                .with_message("module belongs to a different runtime")
                .into());
        }
        Ok(())
    }

    pub(crate) fn admit_loader_result(result: JsResult<Self>, context: &Context) -> JsResult<Self> {
        let module = result.map_err(|error| context.runtime().admit_error(error))?;
        module.check_runtime(context).map_err(|_| {
            JsNativeError::typ()
                .with_message("module loader returned a module from a different runtime")
        })?;
        Ok(module)
    }

    /// Abstract operation [`InnerModuleLinking ( module, stack, index )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLinking
    fn inner_evaluate(
        &self,
        stack: &mut lifecycle::EvaluationStack,
        index: usize,
        context: &mut Context,
    ) -> JsResult<source::EvaluationProgress> {
        match self.kind() {
            ModuleKind::SourceText(src) => src.inner_evaluate(self, stack, index, None, context),
            // 1. If module is not a Cyclic Module Record, then
            ModuleKind::Synthetic(synth) => {
                // a. Let promise be ! module.Evaluate().
                let promise: JsPromise = synth.evaluate(self, context)?;
                let state = promise.state()?;
                match state {
                    PromiseState::Pending => {
                        unreachable!("b. Assert: promise.[[PromiseState]] is not pending.")
                    }
                    // d. Return index.
                    PromiseState::Fulfilled(_) => Ok(source::EvaluationProgress::new(index)),
                    // c. If promise.[[PromiseState]] is rejected, then
                    //    i. Return ThrowCompletion(promise.[[PromiseResult]]).
                    PromiseState::Rejected(err) => Err(JsError::from_opaque(err)),
                }
            }
        }
    }

    /// Loads, links and evaluates this module, returning a promise that will resolve after the module
    /// finishes its lifecycle.
    ///
    /// # Examples
    /// ```
    /// # use std::{path::Path, rc::Rc};
    /// # use surgeist_js::{Context, Source, Module, JsValue};
    /// # use surgeist_js::builtins::promise::PromiseState;
    /// # use surgeist_js::module::{ModuleLoader, SimpleModuleLoader};
    /// let loader = Rc::new(SimpleModuleLoader::new(Path::new(".")).unwrap());
    /// let mut context = &mut Context::builder()
    ///     .module_loader(loader.clone())
    ///     .build()
    ///     .unwrap();
    ///
    /// let source = Source::from_bytes("1 + 3");
    ///
    /// let module = Module::parse(source, None, context.access()).unwrap();
    ///
    /// loader.insert(Path::new("main.mjs").to_path_buf(), module.clone());
    ///
    /// let promise = module.load_link_evaluate(context.access()).unwrap();
    ///
    /// context.run_jobs().unwrap();
    ///
    /// assert_eq!(
    ///     promise.state().unwrap(),
    ///     PromiseState::Fulfilled(JsValue::undefined())
    /// );
    /// ```
    /// # Errors
    /// Returns host errors from preparing or retaining loading and continuation work.
    /// Catchable module failures continue to reject the returned promise.
    #[inline]
    pub fn load_link_evaluate(&self, context: ContextAccess<'_>) -> JsResult<JsPromise> {
        let context = context.into_owner();
        let result = (|| {
            self.check_runtime(context)?;
            let capability = builtins::promise::PromiseCapability::new(
                &context.intrinsics().constructors().promise().constructor(),
                context,
            )?;
            let result = capability
                .promise()
                .clone()
                .downcast::<builtins::Promise>()
                .expect("intrinsic result promise");
            builtins::Promise::ensure_native_association(&result, context)?;
            let link = NativeFunction::from_builtin_with_captures(
                |_, _, module, context| {
                    module.link(context.access())?;
                    Ok(JsValue::undefined())
                },
                self.clone(),
            )
            .to_js_function(context.realm())?;
            // Preserve the separate language-job stages and their admissions.
            let linked = self
                .load(context.access())?
                .then(Some(link), None, context.access())?;
            let evaluate = NativeFunction::from_builtin_with_captures(
                |_, _, (module, target), context| {
                    let bridge = NativeEvaluationBridge {
                        target: target.clone(),
                        fulfilled: target.resolve().clone(),
                        rejected: target.reject().clone(),
                    };
                    match module.evaluate_for_native(&bridge, context) {
                        Ok(_) => {}
                        Err(error) => {
                            if error.as_engine().is_some() {
                                return Err(error);
                            }
                            let reason = error.into_opaque(context.access())?;
                            target.reject().call(
                                &JsValue::undefined(),
                                &[reason],
                                context.access(),
                            )?;
                            return Ok(JsValue::undefined());
                        }
                    }
                    Ok(JsValue::undefined())
                },
                (self.clone(), capability.clone()),
            )
            .to_js_function(context.realm())?;
            builtins::Promise::perform_promise_then_with_native_target(
                &linked,
                Some(evaluate),
                Some(capability.reject().clone()),
                None,
                &capability,
                context,
            )
            .into_result()?;
            JsPromise::from_object(capability.promise().clone())
        })();
        match result {
            Ok(promise) => Ok(promise),
            Err(error) if error.is_catchable() => JsPromise::reject(error, context.access()),
            Err(error) => Err(error),
        }
    }

    /// Abstract operation [`GetModuleNamespace ( module )`][spec].
    ///
    /// Gets the [**Module Namespace Object**][ns] that represents this module's exports.
    ///
    /// # Errors
    ///
    /// Returns a `TypeError` if the module belongs to another runtime.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getmodulenamespace
    /// [ns]: https://tc39.es/ecma262/#sec-module-namespace-exotic-objects
    pub fn namespace(&self, context: ContextAccess<'_>) -> JsResult<JsObject> {
        let context = context.into_owner();
        self.check_runtime(context)?;
        // 1. Assert: If module is a Cyclic Module Record, then module.[[Status]] is not new or unlinked.
        // 2. Let namespace be module.[[Namespace]].
        // 3. If namespace is empty, then
        // 4. Return namespace.
        if let Some(namespace) = self.inner.namespace.borrow().clone() {
            return Ok(JsObject::from_inner(namespace));
        }

        // Resolve all unambiguous exports before allocating the namespace.
        let exported_names = self.get_exported_names(&mut Vec::default());
        let unambiguous_names = exported_names
            .into_iter()
            .filter(|name| self.resolve_export(name, &mut HashSet::default()).is_ok())
            .collect();

        let namespace = ModuleNamespace::create(self.clone(), unambiguous_names, context);
        self.inner.namespace.replace(Some(namespace.edge()));
        Ok(namespace)
    }

    /// Get an exported value from the module.
    #[inline]
    pub fn get_value<K>(&self, name: K, context: ContextAccess<'_>) -> JsResult<JsValue>
    where
        K: Into<PropertyKey>,
    {
        let context = context.into_owner();
        let namespace = self.namespace(context.access())?;
        namespace.get(name, context.access())
    }

    /// Get an exported function, typed, from the module.
    #[inline]
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_typed_fn<A, R>(
        &self,
        name: JsString,
        context: ContextAccess<'_>,
    ) -> JsResult<TypedJsFunction<A, R>>
    where
        A: crate::object::TryIntoJsArguments,
        R: TryFromJs,
    {
        let context = context.into_owner();
        let func = self.get_value(name.clone(), context.access())?;
        let func = func.as_function().ok_or_else(|| {
            JsNativeError::typ().with_message(format!("{name:?} is not a function"))
        })?;
        Ok(func.typed())
    }

    /// Returns the path of the module, if it was created from a file or assigned.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.inner.path.as_deref()
    }
}

impl PartialEq for Module {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.handle() == other.inner.handle()
    }
}

impl Eq for Module {}

impl Hash for Module {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.handle().erase().hash(state);
    }
}

/// A trait to convert a type into a JS module.
pub trait IntoJsModule {
    /// Converts the type into a JS module.
    ///
    /// # Errors
    ///
    /// Returns an error if the module cannot be created in the context's runtime.
    fn into_js_module(self, context: ContextAccess<'_>) -> JsResult<Module>;
}

impl<T: IntoIterator<Item = (JsString, NativeFunction)> + Clone> IntoJsModule for T {
    fn into_js_module(self, context: ContextAccess<'_>) -> JsResult<Module> {
        let context = context.into_owner();
        context.check_execution_stop()?;
        context.with_module_metadata(|context| {
            let realm = context.realm().clone();
            let mut control = metadata::Control::new(context);
            // This is the first positive credit before opaque iterator conversion.
            control.admit(metadata::Work::Record)?;
            control.admit(metadata::Work::Checkpoint)?;
            let mut iterator = self.into_iter();
            control.admit(metadata::Work::Checkpoint)?;
            let mut names = metadata::Index::default();
            let mut entries = metadata::Buffer::default();
            let mut functions = metadata::Buffer::default();
            loop {
                control.admit(metadata::Work::Visit)?;
                control.admit(metadata::Work::Checkpoint)?;
                let next = iterator.next();
                control.admit(metadata::Work::Checkpoint)?;
                let Some((name, function)) = next else { break };
                let key = control.copy(&name)?;
                names.insert(key, (), &mut control)?;
                control.admit(metadata::Work::Copy(std::mem::size_of::<
                    crate::native_function::StoredNativeFunction,
                >()))?;
                let stored = function.stored();
                control.push(&mut entries, (name, stored))?;
                control.push(&mut functions, function)?;
            }
            // As with the former eager collection, release the host iterator
            // before constructing/storing the module initializer.
            drop(iterator);
            control.admit(metadata::Work::Checkpoint)?;
            let initializer =
                synthetic::native_exports(entries.into_vec(), functions.into_vec(), &mut control)?;
            Module::publish_synthetic(names, &initializer, None, &realm, &mut control)
        })
    }
}

#[test]
fn native_exports_do_not_root_unreachable_module_cycles() {
    let mut context = Context::default();
    let exported = JsObject::with_null_proto(context.runtime());
    let module = Module::from_value_as_default(exported.clone().into(), context.access()).unwrap();
    let module_edge = module.edge();
    let exported_edge = exported.edge();

    // The module initializer retains the exported object, whose function
    // retains the same module through a native capture.
    let function = NativeFunction::from_fn_with_captures(
        |_, _, module, context| Ok(module.namespace(context)?.into()),
        module.clone(),
    )
    .to_js_function(context.realm())
    .unwrap();
    exported
        .create_data_property_or_throw(js_string!("callback"), function, context.access())
        .unwrap();

    context.runtime().heap.collect().unwrap();
    assert!(module_edge.0.is_live().unwrap());
    assert!(exported_edge.upgrade().unwrap().is_some());

    drop(exported);
    drop(module);
    context.runtime().heap.collect().unwrap();
    assert!(!module_edge.0.is_live().unwrap());
    assert!(exported_edge.upgrade().unwrap().is_none());
}

#[test]
fn module_execution_rejects_a_foreign_context() {
    let mut owner = Context::default();
    let mut foreign = Context::default();
    let module = Module::parse(
        Source::from_bytes("export const value = 1;"),
        None,
        owner.access(),
    )
    .unwrap();

    for promise in [
        module.load(foreign.access()).unwrap(),
        module.load_link_evaluate(foreign.access()).unwrap(),
    ] {
        let PromiseState::Rejected(reason) = promise.state().unwrap() else {
            panic!("module execution accepted a foreign context");
        };
        let error = JsError::from_opaque(reason)
            .try_native(foreign.access())
            .unwrap();
        assert!(error.is_type());
    }
    assert!(
        module
            .link(foreign.access())
            .unwrap_err()
            .as_native()
            .unwrap()
            .is_type()
    );
    assert!(
        module
            .evaluate(foreign.access())
            .unwrap_err()
            .as_native()
            .unwrap()
            .is_type()
    );

    let promise = module.load_link_evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
}

#[test]
fn module_construction_rejects_a_foreign_realm_before_parsing() {
    struct UnreadableSource;

    impl std::io::Read for UnreadableSource {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            panic!("module parsing read source for a foreign realm");
        }
    }

    let mut owner = Context::default();
    let foreign = Context::default();
    let realm = foreign.realm().clone();
    let error = Module::parse(
        Source::from_reader(UnreadableSource, None),
        Some(realm.clone()),
        owner.access(),
    )
    .unwrap_err();
    assert!(error.as_native().unwrap().is_type());

    let error = Module::synthetic(
        &[],
        SyntheticModuleInitializer::from_fn_with_captures(|_, (), _| Ok(()), ()),
        None,
        Some(realm),
        owner.access(),
    )
    .unwrap_err();
    assert!(error.as_native().unwrap().is_type());

    let peer = Context::builder()
        .runtime(owner.runtime().clone())
        .build()
        .unwrap();
    let module = Module::parse(
        Source::from_bytes("export const value = 42;"),
        Some(peer.realm().clone()),
        owner.access(),
    )
    .unwrap();
    assert_eq!(module.realm(), *peer.realm());
    let promise = module.load_link_evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("value"), owner.access())
            .unwrap(),
        JsValue::from(42)
    );
}

#[test]
fn synthetic_module_construction_rejects_foreign_captures() {
    let mut owner = Context::default();
    let foreign = Context::default();
    let object = JsObject::with_null_proto(foreign.runtime());
    let initializer =
        SyntheticModuleInitializer::from_fn_with_captures(|_, _, _| Ok(()), object.clone());
    let error = Module::synthetic(&[], initializer, None, None, owner.access()).unwrap_err();
    assert!(error.as_native().unwrap().is_type());

    let error = Module::from_value_as_default(object.into(), owner.access()).unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    owner.runtime().collect().unwrap();
}

#[test]
fn module_namespace_rejects_foreign_contexts_before_and_after_caching() {
    let mut owner = Context::default();
    let mut peer = Context::builder()
        .runtime(owner.runtime().clone())
        .build()
        .unwrap();
    let mut foreign = Context::default();
    let module = Module::from_value_as_default(42.into(), owner.access()).unwrap();
    let promise = module.load_link_evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let error = module.namespace(foreign.access()).unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    let error = module
        .get_value(js_string!("default"), foreign.access())
        .unwrap_err();
    assert!(error.as_native().unwrap().is_type());

    let namespace = module.namespace(peer.access()).unwrap();
    assert_eq!(namespace, module.namespace(owner.access()).unwrap());
    assert_eq!(
        module
            .get_value(js_string!("default"), peer.access())
            .unwrap(),
        JsValue::from(42)
    );

    let error = module.namespace(foreign.access()).unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    let error = module
        .get_value(js_string!("default"), foreign.access())
        .unwrap_err();
    assert!(error.as_native().unwrap().is_type());
}

#[test]
fn synthetic_module_rejects_foreign_exports_without_overwriting_the_binding() {
    let mut owner = Context::default();
    let foreign = Context::default();
    let object = JsObject::with_null_proto(foreign.runtime());
    let module = Module::from_value_as_default(42.into(), owner.access()).unwrap();
    let promise = module.load_link_evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );

    let ModuleKind::Synthetic(synthetic) = module.kind() else {
        panic!("the default export module must be synthetic");
    };
    let name = js_string!("default");
    let error = synthetic.set_export(&name, object.into()).unwrap_err();
    assert!(error.as_native().unwrap().is_type());
    owner.runtime().collect().unwrap();
    foreign.runtime().collect().unwrap();
    assert_eq!(
        module.get_value(name, owner.access()).unwrap(),
        JsValue::from(42)
    );
}

#[test]
fn static_module_loading_rejects_foreign_dependencies_and_allows_retry() {
    let loader = Rc::new(MapModuleLoader::new());
    let mut owner = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let mut foreign = Context::default();
    let dependency = Module::from_value_as_default(1.into(), foreign.access()).unwrap();
    loader.insert("dependency", dependency);
    let module = Module::parse(
        Source::from_bytes("export { default as value } from 'dependency';"),
        None,
        owner.access(),
    )
    .unwrap();

    let promise = module.load(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    let PromiseState::Rejected(reason) = promise.state().unwrap() else {
        panic!("module loading accepted a foreign dependency");
    };
    assert!(
        JsError::from_opaque(reason)
            .try_native(owner.access())
            .unwrap()
            .is_type()
    );
    owner.runtime().collect().unwrap();

    let dependency = Module::from_value_as_default(42.into(), owner.access()).unwrap();
    loader.insert("dependency", dependency);
    let promise = module.load_link_evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        promise.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    assert_eq!(
        module
            .get_value(js_string!("value"), owner.access())
            .unwrap(),
        JsValue::from(42)
    );
}

#[test]
fn module_loading_rejects_a_foreign_loader_error_in_static_and_dynamic_imports() {
    struct ForeignErrorLoader(JsError);

    impl ModuleLoader for ForeignErrorLoader {
        fn load_imported_module(
            self: Rc<Self>,
            _referrer: Referrer,
            _request: ModuleRequest,
        ) -> crate::job::ContextTask<Module> {
            crate::job::ContextTask::from_future(std::future::ready(Err(self.0.clone())))
        }
    }

    let foreign = Context::default();
    let object = JsObject::with_null_proto(foreign.runtime());
    let loader = Rc::new(ForeignErrorLoader(JsError::from_opaque(object.into())));
    let mut owner = Context::builder().module_loader(loader).build().unwrap();

    let module = Module::parse(
        Source::from_bytes("import 'dependency';"),
        None,
        owner.access(),
    )
    .unwrap();
    let static_import = module.load(owner.access()).unwrap();
    let dynamic_import = owner
        .eval(Source::from_bytes("import('dependency')"))
        .unwrap();
    let dynamic_import = JsPromise::from_object(dynamic_import.as_object().unwrap()).unwrap();
    owner.run_jobs().unwrap();

    for promise in [static_import, dynamic_import] {
        let PromiseState::Rejected(reason) = promise.state().unwrap() else {
            panic!("module loading failed to reject a foreign loader error");
        };
        let error = JsError::from_opaque(reason)
            .try_native(owner.access())
            .unwrap();
        assert!(error.is_type());
    }
    owner.runtime().collect().unwrap();
}

#[test]
fn module_promises_reject_foreign_host_job_callback_data() {
    use crate::{context::HostHooks, job::JobCallback, object::JsFunction};

    struct ForeignCallbackData(JsObject);

    impl HostHooks for ForeignCallbackData {
        fn make_job_callback(&self, callback: JsFunction, _: ContextAccess<'_>) -> JobCallback {
            JobCallback::new(callback, self.0.clone())
        }
    }

    let foreign = Context::default();
    let object = JsObject::with_null_proto(foreign.runtime());
    let loader = Rc::new(MapModuleLoader::new());
    let mut owner = Context::builder()
        .host_hooks(Rc::new(ForeignCallbackData(object)))
        .module_loader(loader.clone())
        .build()
        .unwrap();
    let module = Module::from_value_as_default(42.into(), owner.access()).unwrap();
    loader.insert("dependency", module.clone());
    let load_link_evaluate = module.load_link_evaluate(owner.access()).unwrap();
    let dynamic_import = owner
        .eval(Source::from_bytes("import('dependency')"))
        .unwrap();
    let dynamic_import = JsPromise::from_object(dynamic_import.as_object().unwrap()).unwrap();
    let async_module = Module::parse(Source::from_bytes("await 0;"), None, owner.access()).unwrap();
    let load = async_module.load(owner.access()).unwrap();
    owner.run_jobs().unwrap();
    assert_eq!(
        load.state().unwrap(),
        PromiseState::Fulfilled(JsValue::undefined())
    );
    async_module.link(owner.access()).unwrap();
    let async_evaluation = async_module.evaluate(owner.access()).unwrap();
    owner.run_jobs().unwrap();

    for promise in [load_link_evaluate, dynamic_import, async_evaluation] {
        let PromiseState::Rejected(reason) = promise.state().unwrap() else {
            panic!("module promise accepted foreign host job callback data");
        };
        let error = JsError::from_opaque(reason)
            .try_native(owner.access())
            .unwrap();
        assert!(error.is_type());
    }
    owner.runtime().collect().unwrap();
}

#[test]
fn synthetic_module_foreign_failures_reject_dynamic_import() {
    use std::rc::Rc;

    for nested in [false, true] {
        let loader = Rc::new(MapModuleLoader::new());
        let mut context = Context::builder()
            .module_loader(loader.clone())
            .build()
            .unwrap();
        let initializer = SyntheticModuleInitializer::from_fn_with_captures(
            |_, nested, _| {
                let runtime = crate::Runtime::default();
                let foreign = JsObject::with_null_proto(&runtime);
                let error = JsError::from_opaque(foreign.into());
                Err(if *nested {
                    JsNativeError::error().with_cause(error).into()
                } else {
                    error
                })
            },
            nested,
        );
        let module = Module::synthetic(&[], initializer, None, None, context.access()).unwrap();
        loader.insert("foreign-failure", module);

        let import = context
            .eval(Source::from_bytes("import('foreign-failure')"))
            .unwrap();
        let import = JsPromise::from_object(import.as_object().unwrap()).unwrap();
        context.run_jobs().unwrap();

        let PromiseState::Rejected(reason) = import.state().unwrap() else {
            panic!("foreign initializer failure did not reject dynamic import");
        };
        let error = JsError::from_opaque(reason)
            .try_native(context.access())
            .unwrap();
        assert!(error.is_type());
        context.runtime().collect().unwrap();
    }
}

#[test]
#[allow(clippy::missing_panics_doc)]
fn into_js_module() {
    use crate::native_function::NativeCapture;
    use std::cell::Cell;
    use std::rc::Rc;
    use surgeist_js::interop::{ContextData, JsRest};
    use surgeist_js::{
        Context, IntoJsFunction, IntoJsFunctionWithCaptures, Module, Source, js_string,
    };

    #[derive(Clone, HeapTrace, crate::JsData)]
    struct ResultType(#[heap_trace(skip)] Rc<Cell<u32>>);

    #[derive(Clone, HeapTrace)]
    struct Counter(#[heap_trace(skip)] Rc<Cell<i32>>);

    impl NativeCapture for Counter {
        type Stored = Self;

        fn store(&self) -> Self::Stored {
            self.clone()
        }

        fn restore(stored: &Self::Stored) -> Result<Self, crate::error::RootingError> {
            Ok(stored.clone())
        }
    }

    let loader = Rc::new(MapModuleLoader::default());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let foo_count = Counter(Rc::new(Cell::new(0)));
    let bar_count = Counter(Rc::new(Cell::new(0)));
    let dad_count = Counter(Rc::new(Cell::new(0)));

    context.insert_data(ResultType(Rc::new(Cell::new(0))));

    let module = vec![
        (
            js_string!("foo"),
            ((|counter: &Counter| {
                let count = counter.0.get() + 1;
                counter.0.set(count);
                count
            }) as fn(&Counter) -> i32)
                .into_js_function_with_captures(foo_count.clone()),
        ),
        (
            js_string!("bar"),
            ((|counter: &Counter, i: i32| counter.0.set(counter.0.get() + i)) as fn(&Counter, i32))
                .into_js_function_with_captures(bar_count.clone()),
        ),
        (
            js_string!("dad"),
            ((|counter: &Counter, args: JsRest<'_>, mut context: crate::ContextAccess<'_>| {
                let sum = args
                    .into_iter()
                    .map(|i| i.try_js_into::<i32>(context.reborrow()).unwrap())
                    .sum::<i32>();
                counter.0.set(counter.0.get() + sum);
            }) as fn(&Counter, JsRest<'_>, crate::ContextAccess<'_>))
                .into_js_function_with_captures(dad_count.clone()),
        ),
        (
            js_string!("send"),
            ((|value: u32, ContextData(result): ContextData<ResultType>| {
                result.0.set(value);
            }) as fn(u32, ContextData<ResultType>))
                .into_js_function(),
        ),
    ]
    .into_js_module(context.access())
    .unwrap();

    loader.insert("test", module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            let result = test.foo();
            test.foo();
            for (let i = 1; i <= 5; i++) {
                test.bar(i);
            }
            for (let i = 1; i < 5; i++) {
                test.dad(1, 2, 3);
            }

            test.send(result);
        ",
    );
    let root_module = Module::parse(source, None, context.access()).unwrap();

    let promise_result = root_module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert!(
        promise_result.state().unwrap().as_fulfilled().is_some(),
        "module didn't execute successfully! Promise: {:?}",
        promise_result.state().unwrap()
    );

    let result = context.get_data::<ResultType>().unwrap().0.get();

    assert_eq!(foo_count.0.get(), 2);
    assert_eq!(bar_count.0.get(), 15);
    assert_eq!(dad_count.0.get(), 24);
    assert_eq!(result, 1);
}

#[test]
fn can_throw_exception() {
    use std::rc::Rc;
    use surgeist_js::{
        Context, IntoJsFunction, JsError, JsResult, JsValue, Module, Source, js_string,
    };

    let loader = Rc::new(MapModuleLoader::default());
    let mut context = Context::builder()
        .module_loader(loader.clone())
        .build()
        .unwrap();

    let module = vec![(
        js_string!("doTheThrow"),
        ((|message| Err(JsError::from_opaque(message))) as fn(JsValue) -> JsResult<()>)
            .into_js_function(),
    )]
    .into_js_module(context.access())
    .unwrap();

    loader.insert("test", module);

    let source = Source::from_bytes(
        r"
            import * as test from 'test';
            try {
                test.doTheThrow('javascript');
            } catch(e) {
                throw 'from ' + e;
            }
        ",
    );
    let root_module = Module::parse(source, None, context.access()).unwrap();

    let promise_result = root_module.load_link_evaluate(context.access()).unwrap();
    context.run_jobs().unwrap();

    // Checking if the final promise didn't return an error.
    assert_eq!(
        promise_result.state().unwrap().as_rejected(),
        Some(&js_string!("from javascript").into())
    );
}

#[test]
fn test_module_request_attribute_sorting() {
    let request1 = ModuleRequest::new(
        js_string!("specifier"),
        Box::new([
            ImportAttribute::new(js_string!("key2"), js_string!("val2")),
            ImportAttribute::new(js_string!("key1"), js_string!("val1")),
        ]),
    );

    let request2 = ModuleRequest::new(
        js_string!("specifier"),
        Box::new([
            ImportAttribute::new(js_string!("key1"), js_string!("val1")),
            ImportAttribute::new(js_string!("key2"), js_string!("val2")),
        ]),
    );

    assert_eq!(request1, request2);
    assert_eq!(request1.attributes()[0].key(), &js_string!("key1"));
    assert_eq!(request1.attributes()[1].key(), &js_string!("key2"));
}

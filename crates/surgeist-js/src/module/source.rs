use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::Rc,
};

use rustc_hash::{FxHashMap, FxHashSet};
use surgeist_js_ast::{
    ModuleEntryRef, ModuleEntryWork, ModuleMetadataSink,
    declaration::{ImportName, ReExportImportName},
    operations::{ContainsSymbol, NameWork, try_contains},
};
use surgeist_js_gc::heap::Trace;
use surgeist_js_interner::Interner;
use surgeist_js_macros::js_str;

use crate::{
    Context, JsArgs, JsError, JsExpect, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction,
    builtins::{
        Promise,
        promise::{
            PromiseCapability, ReactionLifecycle, RegistrationOutcome, StoredPromiseCapability,
        },
    },
    bytecompiler::ToJsString,
    environments::{DeclarativeEnvironmentEdge, DeclarativeEnvironmentRef, StoredEnvironmentStack},
    error::StoredJsError,
    job::{ContextTask, NativeAsyncJob, UnwindCleanup},
    js_string,
    module::ModuleKind,
    object::{FunctionObjectBuilder, JsPromise, ObjectEdge},
    realm::{Realm, RealmEdge},
    vm::{ActivatedCode, ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock, CompletionRecord},
};

use super::{
    BindingName, GraphLoadingState, Module, ModuleEdge, ModuleRequest, Referrer,
    ResolveExportError, ResolvedBinding, SourceText,
};

use super::lifecycle::{EvaluationRef, EvaluationStack, PreparedEvaluationRecord};
use super::metadata::{Buffer, Control, Index, Work};
use crate::builtins::promise::PreparedPendingReactions;
use crate::builtins::promise::native_settlement::Association;
use surgeist_js_gc::heap::Rooted;

/// All fallible preparation precedes publication into the evaluation lifecycle.
pub(super) struct PreparedEvaluation {
    capability: PromiseCapability,
    association: Rooted<Association>,
    target: Option<Rooted<Association>>,
    bridge: Option<PreparedPendingReactions>,
}
impl PreparedEvaluation {
    fn publish(&self, evaluation: &EvaluationRef) {
        self.association.attach_module(evaluation.clone());
        evaluation.issue(self.association.signal());
        if let Some(target) = &self.target {
            target.bind_module(evaluation.clone());
        }
    }
    fn notify(self) {
        if let Some(bridge) = self.bridge {
            bridge.notify();
        }
    }
}
mod preparation;
pub(crate) use preparation::ModuleActivation;

pub(super) fn link(module: &Module, context: &mut Context) -> JsResult<()> {
    preparation::link(module, context)
}

enum ModuleExecution {
    Finished,
    Transferred,
    // An actual reported failure with unresolved continuation authority. This
    // grants neither completion nor transfer; only finite metadata may close.
    Stopped,
}

pub(super) struct EvaluationProgress {
    pub(super) index: usize,
    pub(super) stopped: bool,
}
impl EvaluationProgress {
    pub(super) fn new(index: usize) -> Self {
        Self {
            index,
            stopped: false,
        }
    }
}

/// Current status of a [`SourceTextModule`].
///
/// Roughly corresponds to the `[[Status]]` field of [**Cyclic Module Records**][cyclic],
/// but with a state machine-like design for better correctness.
///
/// [cyclic]: https://tc39.es/ecma262/#table-cyclic-module-fields
#[derive(Debug, Trace)]
enum InlineCompletion {
    Fulfilled,
    Rejected(StoredJsError),
}

#[derive(Debug, Trace)]
enum ModuleStatus {
    Unlinked {
        #[heap_trace(skip)]
        source: ModuleSource,
    },
    Linking {
        #[heap_trace(skip)]
        source: ModuleSource,
        ancestor_index: usize,
        #[heap_trace(skip)]
        attempt: Rc<()>,
    },
    Pending {
        #[heap_trace(skip)]
        source: ModuleSource,
        ancestor_index: usize,
        #[heap_trace(skip)]
        attempt: Rc<()>,
    },
    Preparing {
        #[heap_trace(skip)]
        attempt: Rc<()>,
    },
    Transitioning,
    Linked {
        environment: DeclarativeEnvironmentEdge,
        context: SourceTextContext,
        ancestor_index: usize,
    },
    Evaluating {
        environment: DeclarativeEnvironmentEdge,
        context: SourceTextContext,
        top_level_capability: Option<StoredPromiseCapability>,
        cycle_root: ModuleEdge,
        ancestor_index: usize,
        async_evaluation_order: Option<usize>,
        pending_async_dependencies: usize,
        inline_completion: Option<InlineCompletion>,
    },
    EvaluatingAsync {
        environment: DeclarativeEnvironmentEdge,
        context: SourceTextContext,
        top_level_capability: Option<StoredPromiseCapability>,
        cycle_root: ModuleEdge,
        async_evaluation_order: usize,
        pending_async_dependencies: usize,
        inline_completion: Option<InlineCompletion>,
    },
    Evaluated {
        environment: DeclarativeEnvironmentEdge,
        top_level_capability: Option<StoredPromiseCapability>,
        cycle_root: ModuleEdge,
        error: Option<StoredJsError>,
    },
}

impl ModuleStatus {
    /// Transition from one state to another, taking the current state by value to move data
    /// between states.
    fn transition<F>(&mut self, f: F)
    where
        F: FnOnce(Self) -> Self,
    {
        *self = f(std::mem::replace(self, ModuleStatus::Transitioning));
    }

    /// Gets the ancestor index of the current module within the dependency graph, or `None` if the
    /// module is not in a state executing the dfs algorithm.
    const fn ancestor_index(&self) -> Option<usize> {
        match *self {
            Self::Unlinked { .. }
            | Self::Pending { .. }
            | Self::Preparing { .. }
            | Self::Transitioning
            | Self::EvaluatingAsync { .. }
            | Self::Evaluated { .. } => None,
            Self::Linking { ancestor_index, .. }
            | Self::Linked { ancestor_index, .. }
            | Self::Evaluating { ancestor_index, .. } => Some(ancestor_index),
        }
    }

    /// Gets a mutable reference to the current ancestor index of the module
    /// within the dependency graph, or `None` if the module is not in a state
    /// executing the dfs algorithm.
    fn ancestor_index_mut(&mut self) -> Option<&mut usize> {
        match self {
            Self::Unlinked { .. }
            | Self::Pending { .. }
            | Self::Preparing { .. }
            | Self::Transitioning
            | Self::EvaluatingAsync { .. }
            | Self::Evaluated { .. } => None,
            Self::Linking {
                ancestor_index: info,
                ..
            }
            | Self::Linked {
                ancestor_index: info,
                ..
            }
            | Self::Evaluating {
                ancestor_index: info,
                ..
            } => Some(info),
        }
    }

    /// If this module is the top module being evaluated and is in the evaluating state, gets its top
    /// level capability.
    fn top_level_capability(&self) -> Option<PromiseCapability> {
        match &self {
            Self::Unlinked { .. }
            | Self::Linking { .. }
            | Self::Pending { .. }
            | Self::Preparing { .. }
            | Self::Transitioning
            | Self::Linked { .. } => None,
            Self::Evaluating {
                top_level_capability,
                ..
            }
            | Self::EvaluatingAsync {
                top_level_capability,
                ..
            }
            | Self::Evaluated {
                top_level_capability,
                ..
            } => top_level_capability
                .as_ref()
                .map(StoredPromiseCapability::to_capability),
        }
    }

    /// If this module is in the evaluated state, gets its `error` field.
    const fn evaluation_error(&self) -> Option<&StoredJsError> {
        match &self {
            Self::Evaluated { error, .. } => error.as_ref(),
            _ => None,
        }
    }

    /// If this module is in the evaluating state, gets its cycle root.
    fn cycle_root(&self) -> Option<Module> {
        match &self {
            Self::Evaluating { cycle_root, .. }
            | Self::EvaluatingAsync { cycle_root, .. }
            | Self::Evaluated { cycle_root, .. } => Some(Module::from_edge(cycle_root.clone())),
            _ => None,
        }
    }

    /// Gets the declarative environment from the module status.
    fn environment(&self) -> Option<DeclarativeEnvironmentRef> {
        match self {
            ModuleStatus::Unlinked { .. }
            | ModuleStatus::Linking { .. }
            | ModuleStatus::Pending { .. }
            | ModuleStatus::Preparing { .. }
            | ModuleStatus::Transitioning => None,
            ModuleStatus::Linked { environment, .. }
            | ModuleStatus::Evaluating { environment, .. }
            | ModuleStatus::EvaluatingAsync { environment, .. }
            | ModuleStatus::Evaluated { environment, .. } => Some(
                environment
                    .upgrade()
                    .expect("module environment heap is available")
                    .expect("linked module retains its environment"),
            ),
        }
    }
}

/// The execution context of a [`SourceTextModule`].
///
/// Stores the required context data that needs to be in place before executing the
/// inner code of the module.
#[derive(Clone, Trace)]
struct SourceTextContext {
    #[heap_trace(skip)]
    codeblock: Rc<CodeBlock>,
    environments: StoredEnvironmentStack,
    realm: RealmEdge,
}

impl std::fmt::Debug for SourceTextContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextContext")
            .field("codeblock", &self.codeblock)
            .field("environments", &self.environments)
            .field("realm", &self.realm)
            .finish()
    }
}

/// ECMAScript's [**Source Text Module Records**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-source-text-module-records
#[derive(Trace)]
pub(crate) struct SourceTextModule {
    status: RefCell<ModuleStatus>,
    loaded_modules: RefCell<FxHashMap<ModuleRequest, ModuleEdge>>,
    pub(crate) evaluation: EvaluationRef,
    import_meta: RefCell<Option<ObjectEdge>>,
    #[heap_trace(skip)]
    code: ModuleCode,
}

impl std::fmt::Debug for SourceTextModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceTextModule")
            .field("status", &self.status)
            .field("loaded_modules", &self.loaded_modules)
            .field("evaluation", &self.evaluation)
            .field("import_meta", &self.import_meta)
            .finish_non_exhaustive()
    }
}

/// Retains syntax and symbol authority until the complete link attempt commits.
#[derive(Debug)]
struct ModuleSource {
    ast: surgeist_js_ast::Module,
    text: SourceText,
    interner: Interner,
}

#[derive(Clone, Debug)]
enum ImportedName {
    Namespace,
    Name(JsString),
}

#[derive(Debug)]
struct ImportEntry {
    module_request: ModuleRequest,
    import_name: ImportedName,
    local_name: JsString,
}

#[derive(Debug)]
struct LocalExportEntry {
    local_name: JsString,
    export_name: JsString,
}

#[derive(Debug)]
struct IndirectExportEntry {
    module_request: ModuleRequest,
    import_name: ImportedName,
    export_name: JsString,
}

#[derive(Debug)]
struct ModuleCode {
    has_tla: bool,
    requested_modules: Index<ModuleRequest>,
    path: Option<PathBuf>,
    import_entries: Vec<ImportEntry>,
    local_export_entries: Vec<LocalExportEntry>,
    indirect_export_entries: Vec<IndirectExportEntry>,
    star_export_entries: Vec<ModuleRequest>,
}

struct MetadataBuilder<'a, 'control> {
    control: &'a mut Control<'control>,
    interner: &'a Interner,
    requested: Index<ModuleRequest>,
    imports: Buffer<ImportEntry>,
    local_names: Index<JsString, usize>,
    local: Buffer<LocalExportEntry>,
    indirect: Buffer<IndirectExportEntry>,
    stars: Buffer<ModuleRequest>,
}
impl<'ast> ModuleMetadataSink<'ast> for MetadataBuilder<'_, '_> {
    type Error = JsError;
    fn admit(&mut self, work: ModuleEntryWork) -> JsResult<()> {
        self.control.admit(match work {
            ModuleEntryWork::Visit => Work::Visit,
            ModuleEntryWork::Name(work) => Work::Name(work),
        })
    }
    fn emit(&mut self, entry: ModuleEntryRef<'ast>) -> JsResult<()> {
        let control = &mut *self.control;
        match entry {
            ModuleEntryRef::Request {
                specifier,
                attributes,
            } => {
                let request = control.request(specifier, attributes, self.interner)?;
                self.requested.insert(request, (), control)?;
            }
            ModuleEntryRef::Import {
                specifier,
                attributes,
                import_name,
                local_name,
            } => {
                let module_request = control.request(specifier, attributes, self.interner)?;
                let import_name = match import_name {
                    ImportName::Namespace => ImportedName::Namespace,
                    ImportName::Name(name) => {
                        ImportedName::Name(control.string(name, self.interner)?)
                    }
                };
                let local_name = control.string(local_name.sym(), self.interner)?;
                let key = control.copy(&local_name)?;
                self.local_names.insert(key, self.imports.len(), control)?;
                control.push(
                    &mut self.imports,
                    ImportEntry {
                        module_request,
                        import_name,
                        local_name,
                    },
                )?;
            }
            ModuleEntryRef::LocalExport {
                local_name,
                export_name,
            } => {
                let local_name = control.string(local_name, self.interner)?;
                let export_name = control.string(export_name, self.interner)?;
                if let Some(index) = self.local_names.get(&local_name, control)?.copied() {
                    control.admit(Work::Probe)?;
                    let import = &self.imports[index];
                    let module_request = control.copy(&import.module_request)?;
                    let import_name = control.copy(&import.import_name)?;
                    control.push(
                        &mut self.indirect,
                        IndirectExportEntry {
                            module_request,
                            import_name,
                            export_name,
                        },
                    )?;
                } else {
                    control.push(
                        &mut self.local,
                        LocalExportEntry {
                            local_name,
                            export_name,
                        },
                    )?;
                }
            }
            ModuleEntryRef::StarExport {
                specifier,
                attributes,
            } => {
                let request = control.request(specifier, attributes, self.interner)?;
                control.push(&mut self.stars, request)?;
            }
            ModuleEntryRef::IndirectExport {
                specifier,
                attributes,
                import_name,
                export_name,
            } => {
                let module_request = control.request(specifier, attributes, self.interner)?;
                let import_name = match import_name {
                    ReExportImportName::Star => ImportedName::Namespace,
                    ReExportImportName::Name(name) => {
                        ImportedName::Name(control.string(name, self.interner)?)
                    }
                };
                let export_name = control.string(export_name, self.interner)?;
                control.push(
                    &mut self.indirect,
                    IndirectExportEntry {
                        module_request,
                        import_name,
                        export_name,
                    },
                )?;
            }
        }
        Ok(())
    }
}

impl SourceTextModule {
    /// Creates a new `SourceTextModule` from a parsed `ModuleSource`.
    ///
    /// Contains part of the abstract operation [`ParseModule`][parse].
    ///
    /// [parse]: https://tc39.es/ecma262/#sec-parsemodule
    pub(super) fn new(
        source: surgeist_js_ast::Module,
        interner: Interner,
        source_text: SourceText,
        path: Option<PathBuf>,
        control: &mut Control<'_>,
    ) -> JsResult<(Self, PreparedEvaluationRecord)> {
        control.header::<Self>()?;
        let mut builder = MetadataBuilder {
            control,
            interner: &interner,
            requested: Index::default(),
            imports: Buffer::default(),
            local_names: Index::default(),
            local: Buffer::default(),
            indirect: Buffer::default(),
            stars: Buffer::default(),
        };
        source.items().try_emit_requests(&mut builder)?;
        source.items().try_emit_import_entries(&mut builder)?;
        source.items().try_emit_export_entries(&mut builder)?;
        let MetadataBuilder {
            control,
            requested,
            imports,
            local,
            indirect,
            stars,
            ..
        } = builder;
        let has_tla = try_contains(&source, ContainsSymbol::AwaitExpression, || {
            control.admit(Work::Name(NameWork::Visit))
        })?;
        let prepared = EvaluationRef::prepare(requested.len(), control)?;
        Ok((
            Self {
                status: RefCell::new(ModuleStatus::Unlinked {
                    source: ModuleSource {
                        ast: source,
                        text: source_text,
                        interner,
                    },
                }),
                loaded_modules: RefCell::default(),
                evaluation: prepared.evaluation.clone(),
                import_meta: RefCell::default(),
                code: ModuleCode {
                    has_tla,
                    requested_modules: requested,
                    path,
                    import_entries: imports.into_vec(),
                    local_export_entries: local.into_vec(),
                    indirect_export_entries: indirect.into_vec(),
                    star_export_entries: stars.into_vec(),
                },
            },
            prepared,
        ))
    }

    /// Abstract operation [`InnerModuleLoading`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-InnerModuleLoading
    pub(super) fn inner_load(
        &self,
        module_self: &Module,
        state: &Rc<GraphLoadingState>,
        context: &mut Context,
    ) -> JsResult<()> {
        /// Loads the module of an import.
        ///
        /// This combines the operations:
        /// - [`HostLoadImportedModule(module, required, state.[[HostDefined]], state)`][load]
        /// - [`FinishLoadingImportedModule ( referrer, specifier, payload, result )`][finish]
        /// - [`ContinueModuleLoading ( state, moduleCompletion )`][continue]
        ///
        /// [load]: https://tc39.es/ecma262/#sec-HostLoadImportedModule
        /// [finish]: https://tc39.es/ecma262/#sec-FinishLoadingImportedModule
        /// [continue]: https://tc39.es/ecma262/#sec-ContinueModuleLoading
        fn finish_loading_imported_module(
            request: super::ModuleRequest,
            src: Module,
            state: Rc<GraphLoadingState>,
            mut task: ContextTask<Module>,
        ) -> ContextTask<JsValue> {
            ContextTask::from_poll(move |cx, host| task.poll(cx, host).map(Ok)).then(
                move |result, context| {
                    let context = context.into_owner();
                    context.check_execution_stop()?;
                    let completion = Module::admit_loader_result(result, context);

                    // FinishLoadingImportedModule ( referrer, specifier, payload, result )
                    // https://tc39.es/ecma262/#sec-FinishLoadingImportedModule

                    // 1. If result is a normal completion, then
                    if let Ok(loaded) = &completion {
                        let ModuleKind::SourceText(src) = src.kind() else {
                            unreachable!("captured src must be a source text module");
                        };
                        // a. If referrer.[[LoadedModules]] contains a Record whose [[Specifier]] is specifier, then
                        // b. Else,
                        //    i. Append the Record { [[Specifier]]: specifier, [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                        src.insert_loaded_module(request, loaded);
                    }

                    // 2. If payload is a GraphLoadingState Record, then
                    //    a. Perform ContinueModuleLoading(payload, result).

                    // Abstract operation `ContinueModuleLoading ( state, moduleCompletion )`.
                    //
                    // https://tc39.es/ecma262/#sec-ContinueModuleLoading

                    // 1. If state.[[IsLoading]] is false, return unused.
                    if !state.loading.get() {
                        return Ok(JsValue::undefined());
                    }

                    // 2. If moduleCompletion is a normal completion, then
                    match completion {
                        Ok(m) => {
                            // a. Perform InnerModuleLoading(state, moduleCompletion.[[Value]]).
                            m.inner_load(&state, context)?;
                        }
                        // 3. Else,
                        Err(err) => {
                            // a. Set state.[[IsLoading]] to false.
                            state.loading.set(false);

                            let err = err.into_opaque(context.access())?;

                            // b. Perform ! Call(state.[[PromiseCapability]].[[Reject]], undefined, « moduleCompletion.[[Value]] »).
                            state.capability.reject().call(
                                &JsValue::undefined(),
                                &[err],
                                context.access(),
                            )?;
                        }
                    }

                    // 4. Return unused.
                    Ok(JsValue::undefined())
                },
            )
        }

        // 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain
        //    module, then
        // a. Append module to state.[[Visited]].
        if matches!(&*self.status.borrow(), ModuleStatus::Unlinked { .. })
            && state.visited.borrow_mut().insert(module_self.clone())
        {
            // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
            let requested = &self.code.requested_modules;
            // c. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] + requestedModulesCount.
            state
                .pending_modules
                .set(state.pending_modules.get() + requested.len());
            // d. For each String required of module.[[RequestedModules]], do
            for required in requested {
                // i. If module.[[LoadedModules]] contains a Record whose [[Specifier]] is required, then
                let loaded = self.loaded_module(required);
                if let Some(loaded) = loaded {
                    // 1. Let record be that Record.
                    // 2. Perform InnerModuleLoading(state, record.[[Module]]).
                    loaded.inner_load(state, context)?;
                } else {
                    //    ii. Else,
                    //       1. Perform HostLoadImportedModule(module, required, state.[[HostDefined]], state).
                    //       2. NOTE: HostLoadImportedModule will call FinishLoadingImportedModule, which re-enters
                    //          the graph loading process through ContinueModuleLoading.
                    let request = required.clone();
                    let src = module_self.clone();
                    let state = state.clone();
                    let capability = state.capability.clone();
                    let async_job = NativeAsyncJob::with_realm(
                        move |context| {
                            let loader = context.into_owner().module_loader();
                            let task = loader.load_imported_module(
                                Referrer::Module(src.clone()),
                                request.clone(),
                            );
                            Ok(finish_loading_imported_module(request, src, state, task))
                        },
                        context.realm().clone(),
                    );
                    capability.associate_required_job(&async_job.observer(), context)?;
                    crate::job::finish_publication(context.enqueue_job(async_job.into()))?;
                }
                // iii. If state.[[IsLoading]] is false, return unused.
                if !state.loading.get() {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Concrete method [`GetExportedNames ( [ exportStarSet ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-getexportednames
    pub(super) fn get_exported_names(
        &self,
        module_self: &Module,
        export_star_set: &mut Vec<Module>,
    ) -> FxHashSet<JsString> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If exportStarSet is not present, set exportStarSet to a new empty List.

        // 3. If exportStarSet contains module, then
        if export_star_set.contains(module_self) {
            // a. Assert: We've reached the starting point of an export * circularity.
            // b. Return a new empty List.
            return FxHashSet::default();
        }

        // 4. Append module to exportStarSet.
        export_star_set.push(module_self.clone());

        // 5. Let exportedNames be a new empty List.
        let mut exported_names = FxHashSet::default();

        // 6. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &self.code.local_export_entries {
            // a. Assert: module provides the direct binding for this export.
            // b. Append e.[[ExportName]] to exportedNames.
            exported_names.insert(e.export_name.clone());
        }

        // 7. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &self.code.indirect_export_entries {
            // a. Assert: module imports a specific binding for this export.
            // b. Append e.[[ExportName]] to exportedNames.
            exported_names.insert(e.export_name.clone());
        }

        // 8. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &self.code.star_export_entries {
            // a. Let requestedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let requested_module = self
                .loaded_module(e)
                .expect("module loading populated the requested module");

            // b. Let starNames be requestedModule.GetExportedNames(exportStarSet).
            // c. For each element n of starNames, do
            for n in requested_module.get_exported_names(export_star_set) {
                // i. If SameValue(n, "default") is false, then
                if n != js_str!("default") {
                    // 1. If exportedNames does not contain n, then
                    //    a. Append n to exportedNames.
                    exported_names.insert(n);
                }
            }
        }

        // 9. Return exportedNames.
        exported_names
    }

    /// Concrete method [`ResolveExport ( exportName [ , resolveSet ] )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-resolveexport
    #[allow(clippy::mutable_key_type)]
    pub(super) fn resolve_export(
        &self,
        module_self: &Module,
        export_name: &JsString,
        resolve_set: &mut FxHashSet<(Module, JsString)>,
    ) -> Result<ResolvedBinding, ResolveExportError> {
        // 1. Assert: module.[[Status]] is not new.
        // 2. If resolveSet is not present, set resolveSet to a new empty List.
        // 3. For each Record { [[Module]], [[ExportName]] } r of resolveSet, do
        //    a. If module and r.[[Module]] are the same Module Record and SameValue(exportName, r.[[ExportName]]) is true, then
        let value = (module_self.clone(), export_name.clone());
        if resolve_set.contains(&value) {
            //   i. Assert: This is a circular import request.
            //   ii. Return null.
            return Err(ResolveExportError::NotFound);
        }

        // 4. Append the Record { [[Module]]: module, [[ExportName]]: exportName } to resolveSet.
        resolve_set.insert(value);

        // 5. For each ExportEntry Record e of module.[[LocalExportEntries]], do
        for e in &self.code.local_export_entries {
            // a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == &e.export_name {
                // i. Assert: module provides the direct binding for this export.
                // ii. Return ResolvedBinding Record { [[Module]]: module, [[BindingName]]: e.[[LocalName]] }.
                return Ok(ResolvedBinding {
                    module: module_self.clone(),
                    binding_name: BindingName::Name(e.local_name.clone()),
                });
            }
        }

        // 6. For each ExportEntry Record e of module.[[IndirectExportEntries]], do
        for e in &self.code.indirect_export_entries {
            // a. If SameValue(exportName, e.[[ExportName]]) is true, then
            if export_name == &e.export_name {
                // i. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
                let imported_module = self
                    .loaded_module(&e.module_request)
                    .expect("module loading populated the requested module");
                return match &e.import_name {
                    // ii. If e.[[ImportName]] is all, then
                    //    1. Assert: module does not provide the direct binding for this export.
                    //    2. Return ResolvedBinding Record { [[Module]]: importedModule, [[BindingName]]: namespace }.
                    ImportedName::Namespace => Ok(ResolvedBinding {
                        module: imported_module,
                        binding_name: BindingName::Namespace,
                    }),
                    // iii. Else,
                    //    1. Assert: module imports a specific binding for this export.
                    //    2. Return importedModule.ResolveExport(e.[[ImportName]], resolveSet).
                    ImportedName::Name(name) => imported_module.resolve_export(name, resolve_set),
                };
            }
        }

        // 7. If SameValue(exportName, "default") is true, then
        if export_name == &js_str!("default") {
            // a. Assert: A default export was not explicitly defined by this module.
            // b. Return null.
            // c. NOTE: A default export cannot be provided by an export * from "mod" declaration.
            return Err(ResolveExportError::NotFound);
        }

        // 8. Let starResolution be null.
        let mut star_resolution: Option<ResolvedBinding> = None;

        // 9. For each ExportEntry Record e of module.[[StarExportEntries]], do
        for e in &self.code.star_export_entries {
            // a. Let importedModule be GetImportedModule(module, e.[[ModuleRequest]]).
            let imported_module = self
                .loaded_module(e)
                .expect("module loading populated the requested module");
            // b. Let resolution be importedModule.ResolveExport(exportName, resolveSet).
            let resolution = match imported_module.resolve_export(export_name, resolve_set) {
                // d. If resolution is not null, then
                Ok(resolution) => resolution,
                // c. If resolution is ambiguous, return ambiguous.
                Err(e @ ResolveExportError::Ambiguous) => return Err(e),
                Err(ResolveExportError::NotFound) => continue,
            };

            // i. Assert: resolution is a ResolvedBinding Record.
            if let Some(star_resolution) = &star_resolution {
                // iii. Else,
                //    1. Assert: There is more than one * import that includes the requested name.
                //    2. If resolution.[[Module]] and starResolution.[[Module]] are not the same Module Record,
                //       return ambiguous.
                if resolution.module() != star_resolution.module() {
                    return Err(ResolveExportError::Ambiguous);
                }
                // 3. If resolution.[[BindingName]] is not starResolution.[[BindingName]] and either
                //    resolution.[[BindingName]] or starResolution.[[BindingName]] is namespace,
                //    return ambiguous.
                // 4. If resolution.[[BindingName]] is a String, starResolution.[[BindingName]] is a
                //    String, and SameValue(resolution.[[BindingName]], starResolution.[[BindingName]])
                //    is false, return ambiguous.
                match (resolution.binding_name(), star_resolution.binding_name()) {
                    (BindingName::Namespace, BindingName::Name(_))
                    | (BindingName::Name(_), BindingName::Namespace) => {
                        // See step 3 above.
                        return Err(ResolveExportError::Ambiguous);
                    }
                    (BindingName::Name(res), BindingName::Name(star)) if res != star => {
                        // See step 4 above.
                        return Err(ResolveExportError::Ambiguous);
                    }
                    _ => {}
                }
            } else {
                // ii. If starResolution is null, then
                //    1. Set starResolution to resolution.
                star_resolution = Some(resolution);
            }
        }

        // 10. Return starResolution.
        star_resolution.ok_or(ResolveExportError::NotFound)
    }

    /// Concrete method [`Evaluate ( )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-moduleevaluation
    fn target_owner(
        target: Option<&PromiseCapability>,
        context: &Context,
    ) -> JsResult<Option<Rooted<Association>>> {
        target
            .map(|target| {
                let promise = target
                    .promise()
                    .clone()
                    .downcast::<Promise>()
                    .expect("intrinsic native target");
                Promise::ensure_native_association(&promise, context)
            })
            .transpose()
    }
    fn notify_terminal_target(
        &self,
        target: Option<&PromiseCapability>,
        context: &Context,
    ) -> JsResult<()> {
        context.check_execution_stop()?;
        let owner = Self::target_owner(target, context)?;
        context.check_execution_stop()?;
        if let Some(owner) = owner {
            owner.observe_module(self.evaluation.clone());
        }
        Ok(())
    }
    fn connect_target(
        &self,
        capability: &PromiseCapability,
        bridge: Option<&super::NativeEvaluationBridge>,
        context: &mut Context,
    ) -> JsResult<()> {
        if let Some(bridge) = bridge {
            let promise = capability
                .promise()
                .clone()
                .downcast::<Promise>()
                .expect("intrinsic evaluation promise");
            let owner =
                Self::target_owner(Some(&bridge.target), context)?.expect("native bridge target");
            match Promise::perform_promise_then_with_native_target(
                &promise,
                Some(bridge.fulfilled.clone()),
                Some(bridge.rejected.clone()),
                None,
                &bridge.target,
                context,
            )
            .into_outcome()
            {
                RegistrationOutcome::Uncommitted(error) => return Err(error),
                RegistrationOutcome::Committed(committed) => {
                    owner.bind_module(self.evaluation.clone());
                    committed.into_result()?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn evaluate(
        &self,
        module_self: &Module,
        bridge: Option<&super::NativeEvaluationBridge>,
        context: &mut Context,
    ) -> JsResult<JsPromise> {
        context.with_module_run(|context| self.evaluate_run(module_self, bridge, context))
    }
    fn evaluate_run(
        &self,
        module_self: &Module,
        bridge: Option<&super::NativeEvaluationBridge>,
        context: &mut Context,
    ) -> JsResult<JsPromise> {
        let target = bridge.map(|bridge| &bridge.target);
        context.check_execution_stop()?;
        if let Some(error) = self.evaluation.error(module_self) {
            self.notify_terminal_target(target, context)?;
            return Err(error);
        }
        let module = self
            .status
            .borrow()
            .cycle_root()
            .unwrap_or_else(|| module_self.clone());
        let source = module.kind().as_source_text().expect("source cycle root");
        if let Some(error) = source.evaluation.error(&module) {
            source.notify_terminal_target(target, context)?;
            return Err(error);
        }
        let cached = source.status.borrow().top_level_capability();
        if let Some(capability) = cached {
            source.connect_target(&capability, bridge, context)?;
            if let Some(error) = source.evaluation.error(&module) {
                return Err(error);
            }
            return JsPromise::from_object(capability.promise().clone());
        }
        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )?;
        let promise = capability
            .promise()
            .clone()
            .downcast::<Promise>()
            .expect("intrinsic promise");
        let association = Promise::ensure_native_association(&promise, context)?;
        let target_owner = Self::target_owner(target, context)?;
        let reactions = bridge
            .map(|bridge| {
                Promise::prepare_pending_then(
                    &promise,
                    bridge.fulfilled.clone(),
                    bridge.rejected.clone(),
                    &bridge.target,
                    context,
                )
            })
            .transpose()?;
        // No public origin, canonical capability or issued route has been installed.
        context.check_execution_stop()?;
        if let Some(error) = source.evaluation.error(&module) {
            source.notify_terminal_target(target, context)?;
            return Err(error);
        }
        let cached = source.status.borrow().top_level_capability();
        if let Some(existing) = cached {
            source.connect_target(&existing, bridge, context)?;
            if let Some(error) = source.evaluation.error(&module) {
                return Err(error);
            }
            return JsPromise::from_object(existing.promise().clone());
        }
        let prepared = PreparedEvaluation {
            capability: capability.clone(),
            association,
            target: target_owner,
            bridge: reactions,
        };
        // Late native entry can issue a capability for an indirect dependency.
        let existing = {
            let mut status = source.status.borrow_mut();
            match &mut *status {
                ModuleStatus::EvaluatingAsync {
                    top_level_capability,
                    ..
                } => {
                    *top_level_capability = Some(capability.stored());
                    Some(None)
                }
                ModuleStatus::Evaluated {
                    top_level_capability,
                    error,
                    ..
                } => {
                    *top_level_capability = Some(capability.stored());
                    Some(Some(error.clone()))
                }
                ModuleStatus::Linked { .. } => None,
                _ => unreachable!("native evaluation requires linked or completed source"),
            }
        };
        if let Some(existing) = existing {
            prepared.publish(&source.evaluation);
            prepared.notify();
            if let Some(error) = existing {
                if let Some(error) = error {
                    let value = error.to_error().into_opaque(context.access())?;
                    capability
                        .reject()
                        .call(&JsValue::undefined(), &[value], context.access())?;
                } else {
                    capability
                        .resolve()
                        .call(&JsValue::undefined(), &[], context.access())?;
                }
            }
            return JsPromise::from_object(capability.promise().clone());
        }
        let mut stack = EvaluationStack::default();
        let result = source.inner_evaluate(&module, &mut stack, 0, Some(prepared), context);
        let result: JsResult<()> = (|| {
            match result {
                Ok(progress) => {
                    if !progress.stopped
                        && matches!(
                            &*source.status.borrow(),
                            ModuleStatus::Evaluated { error: None, .. }
                        )
                        && matches!(
                            promise.borrow().data().state(),
                            crate::builtins::promise::PromiseState::Pending
                        )
                    {
                        capability
                            .resolve()
                            .call(&JsValue::undefined(), &[], context.access())?;
                    }
                    assert!(stack.is_empty());
                }
                Err(error) if error.as_engine().is_some() => {
                    // Open DFS callers are required owners even before parent registration.
                    stack.interrupt(&error);
                    while stack.pop().is_some() {}
                    return Err(error);
                }
                Err(error) => {
                    while let Some(member) = stack.pop() {
                        let record = member.kind().as_source_text().expect("source stack");
                        record
                            .status
                            .borrow_mut()
                            .transition(|status| match status {
                                ModuleStatus::Evaluating {
                                    environment,
                                    top_level_capability,
                                    cycle_root,
                                    ..
                                }
                                | ModuleStatus::EvaluatingAsync {
                                    environment,
                                    top_level_capability,
                                    cycle_root,
                                    ..
                                } => ModuleStatus::Evaluated {
                                    environment,
                                    top_level_capability,
                                    cycle_root,
                                    error: Some(error.clone().into_stored()),
                                },
                                _ => unreachable!("DFS owns evaluating records"),
                            });
                        if record.status.borrow().top_level_capability().is_none() {
                            record.evaluation.complete();
                        }
                    }
                    let value = error.into_opaque(context.access())?;
                    capability
                        .reject()
                        .call(&JsValue::undefined(), &[value], context.access())?;
                }
            }
            Ok(())
        })();
        if let Err(error) = &result
            && error.as_engine().is_some()
        {
            // Canonical settlement is part of the entered evaluation until the
            // Promise owner publishes its state and completes the origin.
            source.evaluation.interrupt(error);
        }
        result?;
        JsPromise::from_object(capability.promise().clone())
    }

    /// Abstract operation [`InnerModuleEvaluation ( module, stack, index )`][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-innermoduleevaluation
    pub(super) fn inner_evaluate(
        &self,
        module_self: &Module,
        stack: &mut EvaluationStack,
        mut index: usize,
        prepared: Option<PreparedEvaluation>,
        context: &mut Context,
    ) -> JsResult<EvaluationProgress> {
        context.check_execution_stop()?;
        let mut stopped = false;
        if let Some(error) = self.evaluation.error(module_self) {
            return Err(error);
        }
        // TODO: this count should be part of the `context` instead, and we should
        // reset it after the last module has executed.
        /// [`IncrementModuleAsyncEvaluationCount ( )`][spec]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-IncrementModuleAsyncEvaluationCount
        fn increment_module_async_evaluation_count() -> JsResult<usize> {
            thread_local! {
                static ASYNC_EVAL_QUEUE_INDEX: Cell<usize> = const { Cell::new(0) };
            }

            // 1. Let AR be the Agent Record of the surrounding agent.
            // 2. Let count be AR.[[ModuleAsyncEvaluationCount]].
            // 3. Set AR.[[ModuleAsyncEvaluationCount]] to count + 1.
            // 4. Return count.
            ASYNC_EVAL_QUEUE_INDEX
                .with(|idx| {
                    let next = idx.get().checked_add(1)?;
                    Some(idx.replace(next))
                })
                .ok_or_else(|| {
                    JsNativeError::range()
                        .with_message("exceeded the maximum number of async modules")
                        .into()
                })
        }

        // 2. If module.[[Status]] is either evaluating-async or evaluated, then
        match &*self.status.borrow() {
            // 3. If module.[[Status]] is evaluating, return index.
            ModuleStatus::Evaluating { .. } | ModuleStatus::EvaluatingAsync { .. } => {
                return Ok(EvaluationProgress::new(index));
            }
            //     a. If module.[[EvaluationError]] is empty, return index.
            //     b. Otherwise, return ? module.[[EvaluationError]].
            ModuleStatus::Evaluated { error, .. } => {
                return error
                    .as_ref()
                    .map_or(Ok(EvaluationProgress::new(index)), |error| {
                        Err(error.to_error())
                    });
            }
            ModuleStatus::Linked { .. } => {
                // 4. Assert: module.[[Status]] is linked.
                // evaluate a linked module
            }
            _ => unreachable!(
                "2. Assert: module.[[Status]] is one of linked, evaluating-async, or evaluated."
            ),
        }

        let capability = prepared.as_ref().map(|prepared| &prepared.capability);
        // 5. Set module.[[Status]] to evaluating.
        // 6. Set moduleIndex to index.
        // 7. Set module.[[DFSAncestorIndex]] to index.
        // 8. Set module.[[PendingAsyncDependencies]] to 0.
        self.status.borrow_mut().transition(|status| match status {
            ModuleStatus::Linked {
                environment,
                context,
                ..
            } => ModuleStatus::Evaluating {
                environment,
                context,
                top_level_capability: capability.map(PromiseCapability::stored),
                cycle_root: module_self.edge(),
                ancestor_index: index,
                async_evaluation_order: None,
                pending_async_dependencies: 0,
                inline_completion: None,
            },
            _ => unreachable!("already asserted that this state is `Linked`. "),
        });

        if let Some(prepared) = &prepared {
            prepared.publish(&self.evaluation);
        }
        self.evaluation.enter();
        if let Some(prepared) = prepared {
            prepared.notify();
            context.check_execution_stop()?;
        }
        let module_index = index;

        // 9. Set index to index + 1.
        index += 1;

        // 10. Append module to stack.
        stack.push(module_self.clone());

        // 11. 11. For each ModuleRequest Record request of module.[[RequestedModules]], do
        for (request_index, required) in self.code.requested_modules.iter().enumerate() {
            // a. Let requiredModule be GetImportedModule(module, required).
            let required_module = self
                .loaded_module(required)
                .expect("module loading populated the requested module");
            // b. Set index to ? InnerModuleEvaluation(requiredModule, stack, index).
            let required_progress = required_module.inner_evaluate(stack, index, context)?;
            index = required_progress.index;
            stopped |= required_progress.stopped;

            // c. If requiredModule is a Cyclic Module Record, then
            if let ModuleKind::SourceText(required_module_src) = required_module.kind() {
                // i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated.
                // ii. Assert: requiredModule.[[Status]] is evaluating if and only if stack contains requiredModule.
                debug_assert!(match &*required_module_src.status.borrow() {
                    ModuleStatus::EvaluatingAsync { .. } | ModuleStatus::Evaluated { .. } => true,
                    ModuleStatus::Evaluating { .. } if stack.contains(&required_module) => true,
                    _ => false,
                });

                let (required_module, async_eval, req_ancestor) = match &*required_module_src
                    .status
                    .borrow()
                {
                    // iii. If requiredModule.[[Status]] is evaluating, then
                    ModuleStatus::Evaluating {
                        ancestor_index,
                        async_evaluation_order,
                        ..
                    } => {
                        // Continues after the borrow to avoid a borrow panic
                        (
                            required_module.clone(),
                            async_evaluation_order.is_some(),
                            Some(*ancestor_index),
                        )
                    }
                    // iv. Else,
                    ModuleStatus::EvaluatingAsync { cycle_root, .. }
                    | ModuleStatus::Evaluated { cycle_root, .. } => {
                        // 1. Set requiredModule to requiredModule.[[CycleRoot]].
                        let cycle_root = Module::from_edge(cycle_root.clone());
                        let ModuleKind::SourceText(cycle_root_src) = cycle_root.kind() else {
                            unreachable!("cycle_root must be a source text module");
                        };

                        // 2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated.
                        match &*cycle_root_src.status.borrow() {
                            ModuleStatus::EvaluatingAsync { .. } => {
                                (cycle_root.clone(), true, None)
                            }
                            // 3. If requiredModule.[[EvaluationError]] is not empty, return ? requiredModule.[[EvaluationError]].
                            ModuleStatus::Evaluated {
                                error: Some(error), ..
                            } => return Err(error.to_error()),
                            ModuleStatus::Evaluated { .. } => (cycle_root.clone(), false, None),
                            _ => unreachable!(
                                "2. Assert: requiredModule.[[Status]] is either evaluating-async or evaluated."
                            ),
                        }
                    }
                    _ => unreachable!(
                        "i. Assert: requiredModule.[[Status]] is one of evaluating, evaluating-async, or evaluated."
                    ),
                };

                let ModuleKind::SourceText(required_module) = required_module.kind() else {
                    unreachable!("required_module must be a source text module");
                };

                if let Some(req_ancestor) = req_ancestor {
                    let mut status = self.status.borrow_mut();
                    let ancestor_index = status
                        .ancestor_index_mut()
                        .js_expect("self should still be in the evaluating state")?;

                    // 1. Set module.[[DFSAncestorIndex]] to min(module.[[DFSAncestorIndex]], requiredModule.[[DFSAncestorIndex]]).
                    *ancestor_index = usize::min(*ancestor_index, req_ancestor);
                }

                // v. If requiredModule.[[AsyncEvaluationOrder]] is an integer, then
                if async_eval {
                    // 1. Set module.[[PendingAsyncDependencies]] to module.[[PendingAsyncDependencies]] + 1.
                    if let ModuleStatus::Evaluating {
                        pending_async_dependencies,
                        ..
                    } = &mut *self.status.borrow_mut()
                    {
                        *pending_async_dependencies += 1;
                    } else {
                        unreachable!("dependency discovery owns Evaluating");
                    }
                    // 2. Append module to requiredModule.[[AsyncParentModules]].
                    required_module
                        .evaluation
                        .register_parent(module_self, request_index);
                }
            }
        }

        let pending_async_dependencies = match &*self.status.borrow() {
            ModuleStatus::Evaluating {
                pending_async_dependencies,
                ..
            } => *pending_async_dependencies,
            _ => unreachable!("dependency discovery owns Evaluating"),
        };
        // 12. If module.[[PendingAsyncDependencies]] > 0 or module.[[HasTLA]] is true, then
        if pending_async_dependencies > 0 || self.code.has_tla {
            {
                let ModuleStatus::Evaluating {
                    async_evaluation_order,
                    ..
                } = &mut *self.status.borrow_mut()
                else {
                    unreachable!("self should still be in the evaluating state")
                };

                // a. Assert: module.[[AsyncEvaluationOrder]] is unset.
                debug_assert!(async_evaluation_order.is_none());

                // b. Set module.[[AsyncEvaluationOrder]] to IncrementModuleAsyncEvaluationCount().
                *async_evaluation_order = Some(increment_module_async_evaluation_count()?);
            }

            //     d. If module.[[PendingAsyncDependencies]] = 0, perform ExecuteAsyncModule(module).
            if pending_async_dependencies == 0 {
                stopped |= matches!(
                    self.execute_async(module_self, context)?,
                    ModuleExecution::Stopped
                );
            }
        } else {
            // 13. Else,
            //    a. Perform ? module.ExecuteModule().
            stopped |= matches!(
                self.execute(module_self, None, context)?,
                ModuleExecution::Stopped
            );
        }

        let ancestor_index = self.status.borrow().ancestor_index().js_expect(
            "haven't transitioned from the `Evaluating` state, so it should have its ancestor index",
        )?;

        // 14. Assert: module occurs exactly once in stack.
        debug_assert!(stack.contains(module_self));
        // 15. Assert: module.[[DFSAncestorIndex]] ≤ moduleIndex.
        assert!(ancestor_index <= module_index);

        // 16. If module.[[DFSAncestorIndex]] = moduleIndex, then
        if ancestor_index == module_index {
            let mut cycle_next = self.evaluation.clone();
            let mut ordinary = OrdinaryQueue::default();
            // a. Let done be false.
            // b. Repeat, while done is false,
            loop {
                // i. Let requiredModule be the last element of stack.
                // ii. Remove the last element of stack.
                let required_module = stack
                    .pop()
                    .js_expect("should at least have `self` in the stack")?;
                let is_self = module_self == &required_module;

                let ModuleKind::SourceText(required_module_src) = required_module.kind() else {
                    unreachable!("iii. Assert: requiredModule is a Cyclic Module Record.");
                };
                required_module_src
                    .evaluation
                    .cycle(&self.evaluation, &cycle_next);
                cycle_next = required_module_src.evaluation.clone();
                required_module_src.status.borrow_mut().transition(|current| match current {
                ModuleStatus::Evaluating {
                            environment,
                            top_level_capability,
                            cycle_root,
                            async_evaluation_order,
                            pending_async_dependencies,
                            inline_completion,
                            context,
                            ..
                        } => if let Some(async_evaluation_order) = async_evaluation_order {
                            // vi. Else, set requiredModule.[[Status]] to evaluating-async.
                            ModuleStatus::EvaluatingAsync {
                                environment,
                                top_level_capability,
                                // viii. Set requiredModule.[[CycleRoot]] to module.
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    module_self.edge()
                                },
                                async_evaluation_order,
                                pending_async_dependencies,
                                inline_completion,
                                context
                            }
                        } else {
                            // v. If requiredModule.[[AsyncEvaluationOrder]] is unset, set requiredModule.[[Status]] to evaluated.
                            ModuleStatus::Evaluated {
                                environment,
                                top_level_capability,
                                cycle_root: if is_self {
                                    cycle_root
                                } else {
                                    module_self.edge()
                                },
                                error: None,
                            }
                        }
                        _ => unreachable!(
                            "should only transition to `Evaluated` or `EvaluatingAsync` from the `Evaluating` state"
                        )
                    }
                );

                if matches!(
                    &*required_module_src.status.borrow(),
                    ModuleStatus::Evaluated { .. }
                ) && !stopped
                    && required_module_src
                        .status
                        .borrow()
                        .top_level_capability()
                        .is_none()
                {
                    required_module_src.evaluation.complete();
                }
                if matches!(
                    &*required_module_src.status.borrow(),
                    ModuleStatus::EvaluatingAsync { .. }
                ) {
                    ordinary.insert(required_module.clone());
                }
                // vii. If requiredModule and module are the same Module Record, set done to true.
                if is_self {
                    break;
                }
            }
            stopped |= ordinary.drain(context, stopped)?;
        }

        // 17. Return index.
        Ok(EvaluationProgress { index, stopped })
    }

    /// Abstract operation [`ExecuteAsyncModule ( module )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-execute-async-module
    fn execute_async(
        &self,
        module_self: &Module,
        context: &mut Context,
    ) -> JsResult<ModuleExecution> {
        // This is a new required body, not owned graph-completion bookkeeping.
        context.check_execution_stop()?;
        // 1. Assert: module.[[Status]] is either evaluating or evaluating-async.
        debug_assert!(matches!(
            &*self.status.borrow(),
            ModuleStatus::Evaluating { .. } | ModuleStatus::EvaluatingAsync { .. }
        ));
        // 2. Assert: module.[[HasTLA]] is true.
        debug_assert!(self.code.has_tla);

        // 3. Let capability be ! NewPromiseCapability(%Promise%).
        let capability = PromiseCapability::new(
            &context.intrinsics().constructors().promise().constructor(),
            context,
        )
        .expect("cannot fail for the %Promise% intrinsic");
        let promise = capability
            .promise
            .clone()
            .downcast::<Promise>()
            .expect("%Promise% constructor must always return a `Promise` object");

        // 4. Let fulfilledClosure be a new Abstract Closure with no parameters that captures module and performs the following steps when called:
        // 5. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
        let on_fulfilled = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_, _, module, context| {
                    //     a. Perform AsyncModuleExecutionFulfilled(module).
                    async_module_execution_fulfilled(module, context)?;
                    //     b. Return undefined.
                    Ok(JsValue::undefined())
                },
                module_self.clone(),
            ),
        )
        .build()
        .expect("async module reaction retains its current runtime");

        // 6. Let rejectedClosure be a new Abstract Closure with parameters (error) that captures module and performs the following steps when called:
        // 7. Let onRejected be CreateBuiltinFunction(rejectedClosure, 0, "", « »).
        let on_rejected = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_builtin_with_captures(
                |_, args, module, context| {
                    let error = JsError::from_opaque(args.get_or_undefined(0).clone());
                    // a. Perform AsyncModuleExecutionRejected(module, error).
                    async_module_execution_rejected(module, &error, context)?;
                    // b. Return undefined.
                    Ok(JsValue::undefined())
                },
                module_self.clone(),
            ),
        )
        .build()
        .expect("async module reaction retains its current runtime");

        // 8. Perform PerformPromiseThen(capability.[[Promise]], onFulfilled, onRejected).
        let lifecycle = ReactionLifecycle::for_completion(
            super::lifecycle::ModuleAction::complete(module_self)?,
        )?;
        Promise::perform_promise_then_with_lifecycle(
            &promise,
            Some(on_fulfilled),
            Some(on_rejected),
            None,
            lifecycle,
            context,
        )
        .into_result()?;

        // 9. Perform ! module.ExecuteModule(capability).
        // 10. Return unused.
        self.execute(module_self, Some(capability), context)
    }

    /// Abstract operation [`GatherAvailableAncestors ( module, execList )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-gather-available-ancestors
    #[allow(clippy::mutable_key_type)]
    fn gather_available_ancestors(&self, exec_list: &mut FxHashSet<Module>) {
        // 1. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
        for parent in self.evaluation.parents() {
            let m = parent.module();
            if m.kind()
                .as_source_text()
                .is_some_and(|source| source.evaluation.cause().is_some())
            {
                continue;
            }
            let ModuleKind::SourceText(m_src) = m.kind() else {
                continue;
            };
            if matches!(&*m_src.status.borrow(), ModuleStatus::Evaluating { .. }) {
                parent.defer_ready();
                continue;
            }
            if !matches!(
                &*m_src.status.borrow(),
                ModuleStatus::EvaluatingAsync { .. }
            ) || !parent.consume()
            {
                continue;
            }

            if exec_list.contains(&m) {
                continue;
            }

            {
                let m_status = m_src.status.borrow();
                let Some(cycle_root) = m_status.cycle_root() else {
                    continue;
                };
                let Some(cycle_root_src) = cycle_root.kind().as_source_text() else {
                    continue;
                };

                if cycle_root_src.status.borrow().evaluation_error().is_some() {
                    continue;
                }
            }

            // a. If execList does not contain m and m.[[CycleRoot]].[[EvaluationError]] is empty, then

            let (deps, has_tla) = {
                // i. Assert: m.[[Status]] is evaluating-async.
                // ii. Assert: m.[[EvaluationError]] is empty.
                // iii. Assert: m.[[AsyncEvaluation]] is true.
                let ModuleStatus::EvaluatingAsync {
                    pending_async_dependencies,
                    ..
                } = &mut *m_src.status.borrow_mut()
                else {
                    unreachable!("i. Assert: m.[[Status]] is evaluating-async.");
                };
                // iv. Assert: m.[[PendingAsyncDependencies]] > 0.
                assert!(*pending_async_dependencies > 0);

                // v. Set m.[[PendingAsyncDependencies]] to m.[[PendingAsyncDependencies]] - 1.
                *pending_async_dependencies -= 1;
                (*pending_async_dependencies, m_src.code.has_tla)
            };

            // vi. If m.[[PendingAsyncDependencies]] = 0, then
            if deps == 0 {
                // 1. Append m to execList.
                exec_list.insert(m.clone());
                // 2. If m.[[HasTLA]] is false, perform GatherAvailableAncestors(m, execList).
                if !has_tla {
                    m_src.gather_available_ancestors(exec_list);
                }
            }
        }
        // 2. Return unused.
    }

    /// Abstract operation [`ExecuteModule ( [ capability ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-source-text-module-record-execute-module
    fn execute(
        &self,
        module_self: &Module,
        capability: Option<PromiseCapability>,
        context: &mut Context,
    ) -> JsResult<ModuleExecution> {
        context.check_execution_stop()?;
        if let Some(error) = self.evaluation.error(module_self) {
            return Err(error);
        }
        // 1. Let moduleContext be a new ECMAScript code execution context.
        let SourceTextContext {
            codeblock,
            environments,
            realm,
        } = match &*self.status.borrow() {
            ModuleStatus::Evaluating { context, .. }
            | ModuleStatus::EvaluatingAsync { context, .. } => context.clone(),
            _ => unreachable!("`execute` should only be called for evaluating modules."),
        };

        // 2. Set the Function of moduleContext to null.
        // 3. Set the Realm of moduleContext to module.[[Realm]].
        // 4. Set the ScriptOrModule of moduleContext to module.
        // 5. Assert: module has been linked and declarations in its module environment have been instantiated.
        // 6. Set the VariableEnvironment of moduleContext to module.[[Environment]].
        // 7. Set the LexicalEnvironment of moduleContext to module.[[Environment]].
        let environments = environments.to_stack();
        let env_fp = environments.len() as u32;
        let permit = context.reserve_guest_entry(codeblock.clone(), 0)?;
        let active = ActivatedCode::from_installed(codeblock)?;
        let mut callframe = CallFrame::new(
            active,
            permit,
            Some(ActiveRunnable::Module(module_self.clone())),
            environments,
            Realm::from_edge(realm),
        )?
        .with_env_fp(env_fp)
        .with_flags(CallFrameFlags::EXIT_EARLY);

        callframe.set_module_evaluation(self.evaluation.clone());
        // 8. Suspend the running execution context.
        context
            .vm
            .push_frame_with_stack(callframe, JsValue::undefined(), JsValue::null());

        if let Some(capability) = capability {
            context.vm.set_promise_capability(capability)?;
        }

        // 9. If module.[[HasTLA]] is false, then
        //    a. Assert: capability is not present.
        //    b. Push moduleContext onto the execution context stack; moduleContext is now the running execution context.
        //    c. Let result be Completion(Evaluation of module.[[ECMAScriptCode]]).
        //    d. Suspend moduleContext and remove it from the execution context stack.
        //    e. Resume the context that is now on the top of the execution context stack as the running execution context.
        // 10. Else,
        //    a. Assert: capability is a PromiseCapability Record.
        //    b. Perform AsyncBlockStart(capability, module.[[ECMAScriptCode]], moduleContext).
        let result = context.run();
        let mut completed = self.evaluation.is_complete();
        let normal_terminal = matches!(&completed, Ok(true));
        let mut finish = if normal_terminal {
            None
        } else {
            Some(context.vm.frame().continuation_finish())
        };
        if let Err(error) = &mut completed {
            context.capture_error_backtrace(error);
        }
        if let Some(Err(error)) = &mut finish {
            context.capture_error_backtrace(error);
        }
        let transferred = finish
            .as_ref()
            .is_some_and(|finish| finish.as_ref().is_ok_and(|finish| finish.is_transferred()));
        let access_failed = completed.is_err() || finish.as_ref().is_some_and(Result::is_err);
        let (mut cause, value) = match result {
            CompletionRecord::Throw(error) => (Some(error), None),
            CompletionRecord::Normal(value) | CompletionRecord::Return(value) => {
                (None, Some(value))
            }
        };
        let mut owner_error = completed.err();
        let mut finish_error = finish.and_then(Result::err);
        if cause.is_none() {
            cause = owner_error.take().or_else(|| finish_error.take());
        }
        let retained = if normal_terminal || transferred || access_failed {
            cause
                .as_mut()
                .is_some_and(|cause| context.retain_committed_reaction_failure(cause))
        } else {
            false
        };
        context.vm.pop_frame();
        // The actual Throw precedes later inspection failures. Dispose each
        // superseded owner independently before returning/resuming its first cause.
        let mut cleanup = UnwindCleanup::default();
        cleanup.run(move || drop(value));
        cleanup.run(move || drop(owner_error));
        cleanup.run(move || drop(finish_error));
        let cause = cleanup.finish_with(cause);
        if let Some(error) = cause {
            if retained {
                let mut cleanup = UnwindCleanup::default();
                cleanup.run(move || drop(error));
                cleanup.finish();
                return Ok(if normal_terminal {
                    ModuleExecution::Finished
                } else if access_failed {
                    ModuleExecution::Stopped
                } else {
                    ModuleExecution::Transferred
                });
            }
            if !normal_terminal && !transferred && !access_failed && error.as_engine().is_some() {
                self.evaluation.interrupt(&error);
            }
            return Err(error);
        }
        Ok(if transferred {
            ModuleExecution::Transferred
        } else {
            ModuleExecution::Finished
        })
    }

    /// Gets a module already loaded for the given request.
    pub(crate) fn loaded_module(&self, request: &ModuleRequest) -> Option<Module> {
        self.loaded_modules
            .borrow()
            .get(request)
            .cloned()
            .map(Module::from_edge)
    }

    /// Records the result of loading a request, preserving its identity on repeat loads.
    pub(crate) fn insert_loaded_module(&self, request: ModuleRequest, module: &Module) {
        let mut loaded_modules = self.loaded_modules.borrow_mut();
        let edge = module.edge();
        let entry = loaded_modules
            .entry(request)
            .or_insert_with(|| edge.clone());
        assert_eq!(entry, &edge);
    }

    /// Gets the cached import meta object.
    pub(crate) fn import_meta(&self) -> Option<JsObject> {
        self.import_meta.borrow().clone().map(JsObject::from_inner)
    }

    /// Stores the initialized import meta object.
    pub(crate) fn set_import_meta(&self, object: &JsObject) {
        self.import_meta.replace(Some(object.edge()));
    }

    /// Gets the declarative environment of this module.
    pub(crate) fn environment(&self) -> Option<DeclarativeEnvironmentRef> {
        self.status.borrow().environment()
    }
}

/// Abstract operation [`AsyncModuleExecutionFulfilled ( module )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-fulfilled
#[allow(clippy::mutable_key_type)]
fn async_module_execution_fulfilled(module: &Module, context: &mut Context) -> JsResult<()> {
    let ModuleKind::SourceText(module_src) = module.kind() else {
        unreachable!("async executed module must be a source text module");
    };

    if let Some(error) = module_src.evaluation.error(module) {
        return Err(error);
    }
    {
        let mut status = module_src.status.borrow_mut();
        if let ModuleStatus::Evaluating {
            inline_completion, ..
        } = &mut *status
        {
            if inline_completion.is_none() {
                *inline_completion = Some(InlineCompletion::Fulfilled);
            }
            return Ok(());
        }
    }
    // 1. If module.[[Status]] is evaluated, then
    if let ModuleStatus::Evaluated { error, .. } = &*module_src.status.borrow() {
        //     a. Assert: module.[[EvaluationError]] is not empty.
        debug_assert!(error.is_some());
        //     b. Return unused.
        return Ok(());
    }

    // 2. Assert: module.[[Status]] is evaluating-async.
    // 3. Assert: module.[[AsyncEvaluationOrder]] is an integer.
    // 4. Assert: module.[[EvaluationError]] is empty.
    // 5. Set module.[[AsyncEvaluationOrder]] to done.
    // 6. Set module.[[Status]] to evaluated.
    module_src
        .status
        .borrow_mut()
        .transition(|status| match status {
            ModuleStatus::EvaluatingAsync {
                environment,
                top_level_capability,
                cycle_root,
                ..
            } => ModuleStatus::Evaluated {
                environment,
                top_level_capability,
                cycle_root,
                error: None,
            },
            _ => unreachable!(),
        });

    if module_src.status.borrow().top_level_capability().is_none() {
        module_src.evaluation.complete();
    }
    // 7. If module.[[TopLevelCapability]] is not empty, then
    if let Some(cap) = module_src.status.borrow().top_level_capability() {
        // a. Assert: module.[[CycleRoot]] and module are the same Module Record.
        debug_assert_eq!(
            module_src.status.borrow().cycle_root(),
            Some(module.clone())
        );

        // b. Perform ! Call(module.[[TopLevelCapability]].[[Resolve]], undefined, « undefined »).
        cap.resolve()
            .call(&JsValue::undefined(), &[], context.access())
            .js_expect("default `resolve` function cannot fail")?;
    }

    // 8. Let execList be a new empty List.
    let mut ancestors = FxHashSet::default();

    // 9. Perform GatherAvailableAncestors(module, execList).
    module_src.gather_available_ancestors(&mut ancestors);

    // 10. 10. Assert: All elements of execList have their
    // [[AsyncEvaluationOrder]] field set to an integer,
    // [[PendingAsyncDependencies]] field set to 0, and [[EvaluationError]]
    // field set to empty.
    //
    // 11. Let sortedExecList be a List whose elements are the elements of
    // execList, sorted by their [[AsyncEvaluationOrder]] field in ascending
    // order.
    let mut sorted_exec_list = ancestors.into_iter().collect::<Vec<_>>();
    sorted_exec_list.sort_by_cached_key(|m| {
        let ModuleKind::SourceText(m_src) = m.kind() else {
            unreachable!("ancestors must only be source text modules");
        };

        let ModuleStatus::EvaluatingAsync {
            async_evaluation_order,
            ..
        } = &*m_src.status.borrow()
        else {
            unreachable!("GatherAvailableAncestors: i. Assert: m.[[Status]] is evaluating-async.");
        };

        *async_evaluation_order
    });

    for module in sorted_exec_list {
        if matches!(
            execute_available_module(&module, context)?,
            ModuleExecution::Stopped
        ) {
            break;
        }
    }
    // 13. Return unused.
    Ok(())
}

/// Runs a body whose dependency discovery and cyclic status publication are complete.
fn execute_available_module(module: &Module, context: &mut Context) -> JsResult<ModuleExecution> {
    let source = module.kind().as_source_text().expect("source ancestor");
    if source.evaluation.cause().is_some()
        || matches!(&*source.status.borrow(), ModuleStatus::Evaluated { .. })
    {
        return Ok(ModuleExecution::Finished);
    }
    if source.code.has_tla {
        match source.execute_async(module, context) {
            Ok(execution) => return Ok(execution),
            Err(error) if error.as_engine().is_some() => {
                source.evaluation.interrupt(&error);
                return Err(error);
            }
            Err(error) => async_module_execution_rejected(module, &error, context)?,
        }
    } else {
        match source.execute(module, None, context) {
            Ok(ModuleExecution::Finished) => async_module_execution_fulfilled(module, context)?,
            Ok(execution) => return Ok(execution),
            Err(error) if error.as_engine().is_some() => {
                source.evaluation.interrupt(&error);
                return Err(error);
            }
            Err(error) => {
                let value = error.into_opaque(context.access())?;
                async_module_execution_rejected(module, &JsError::from_opaque(value), context)?;
            }
        }
    }
    Ok(ModuleExecution::Finished)
}

/// Stack-owned SCC work order; links were reserved in each evaluation record.
#[derive(Default)]
struct OrdinaryQueue {
    head: Option<Module>,
}
impl OrdinaryQueue {
    fn order(module: &Module) -> usize {
        match &*module
            .kind()
            .as_source_text()
            .expect("source work")
            .status
            .borrow()
        {
            ModuleStatus::EvaluatingAsync {
                async_evaluation_order,
                ..
            } => *async_evaluation_order,
            _ => usize::MAX,
        }
    }
    fn insert(&mut self, module: Module) {
        let source = module.kind().as_source_text().expect("source work");
        if self
            .head
            .as_ref()
            .is_none_or(|head| Self::order(&module) < Self::order(head))
        {
            source.evaluation.set_ordinary_next(self.head.as_ref());
            self.head = Some(module);
            return;
        }
        let mut current = self.head.clone().expect("nonempty work");
        loop {
            let next = current
                .kind()
                .as_source_text()
                .expect("source work")
                .evaluation
                .ordinary_next();
            if next
                .as_ref()
                .is_none_or(|next| Self::order(&module) < Self::order(next))
            {
                source.evaluation.set_ordinary_next(next.as_ref());
                current
                    .kind()
                    .as_source_text()
                    .expect("source work")
                    .evaluation
                    .set_ordinary_next(Some(&module));
                break;
            }
            current = next.expect("successor exists");
        }
    }
    fn drain(mut self, context: &mut Context, mut stopped: bool) -> JsResult<bool> {
        while let Some(module) = self.head.take() {
            let source = module.kind().as_source_text().expect("source work");
            self.head = source.evaluation.take_ordinary_next();
            if source.evaluation.cause().is_some() {
                continue;
            }
            // Restore the owned rejection while the status still traces its
            // complete stored graph. Release that shared borrow before taking
            // or disposing the old stored value and publishing transitions.
            let completion: Option<JsResult<()>> = {
                let status = source.status.borrow();
                let ModuleStatus::EvaluatingAsync {
                    inline_completion, ..
                } = &*status
                else {
                    continue;
                };
                if stopped {
                    None
                } else {
                    inline_completion
                        .as_ref()
                        .map(|completion| match completion {
                            InlineCompletion::Fulfilled => Ok(()),
                            InlineCompletion::Rejected(error) => Err(error.to_error()),
                        })
                }
            };
            let deferred = source.evaluation.take_deferred_ready();
            let stored = {
                let mut status = source.status.borrow_mut();
                let ModuleStatus::EvaluatingAsync {
                    pending_async_dependencies,
                    inline_completion,
                    ..
                } = &mut *status
                else {
                    unreachable!("ordinary restoration does not change module state");
                };
                *pending_async_dependencies = pending_async_dependencies
                    .checked_sub(deferred)
                    .expect("each deferred request consumes one dependency");
                if stopped {
                    None
                } else {
                    inline_completion.take()
                }
            };
            drop(stored);
            if stopped {
                continue;
            }
            match completion {
                Some(Ok(())) => async_module_execution_fulfilled(&module, context)?,
                Some(Err(error)) => async_module_execution_rejected(&module, &error, context)?,
                None => {
                    let ready = matches!(
                        &*source.status.borrow(),
                        ModuleStatus::EvaluatingAsync {
                            pending_async_dependencies: 0,
                            ..
                        }
                    );
                    if ready && !source.evaluation.has_invocation() {
                        stopped = matches!(
                            execute_available_module(&module, context)?,
                            ModuleExecution::Stopped
                        );
                    }
                }
            }
        }
        Ok(stopped)
    }
}

/// Abstract operation [`AsyncModuleExecutionRejected ( module, error )`][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-async-module-execution-rejected
fn async_module_execution_rejected(
    module: &Module,
    error: &JsError,
    context: &mut Context,
) -> JsResult<()> {
    let ModuleKind::SourceText(module_src) = module.kind() else {
        unreachable!("async executed module must be a source text module");
    };
    if let Some(error) = module_src.evaluation.error(module) {
        return Err(error);
    }
    {
        let mut status = module_src.status.borrow_mut();
        if let ModuleStatus::Evaluating {
            inline_completion, ..
        } = &mut *status
        {
            if !matches!(inline_completion, Some(InlineCompletion::Rejected(_))) {
                *inline_completion = Some(InlineCompletion::Rejected(error.clone().into_stored()));
            }
            return Ok(());
        }
    }
    // 1. If module.[[Status]] is evaluated, then
    if let ModuleStatus::Evaluated { error, .. } = &*module_src.status.borrow() {
        //     a. Assert: module.[[EvaluationError]] is not empty.
        debug_assert!(error.is_some());
        //     b. Return unused.
        return Ok(());
    }

    // 2. Assert: module.[[Status]] is evaluating-async.
    // 3. Assert: module.[[AsyncEvaluationOrder]] is an integer..
    // 4. Assert: module.[[EvaluationError]] is empty.
    // 5. Set module.[[EvaluationError]] to ThrowCompletion(error).
    // 6. Set module.[[Status]] to evaluated.
    // 7. Set module.[[AsyncEvaluationOrder]] to done.
    module_src
        .status
        .borrow_mut()
        .transition(|status| match status {
            ModuleStatus::EvaluatingAsync {
                environment,
                top_level_capability,
                cycle_root,
                ..
            } => ModuleStatus::Evaluated {
                environment,
                top_level_capability,
                cycle_root,
                error: Some(error.clone().into_stored()),
            },
            _ => unreachable!(),
        });

    if module_src.status.borrow().top_level_capability().is_none() {
        module_src.evaluation.complete();
    }
    // 8. NOTE: module.[[AsyncEvaluationOrder]] is set to done for symmetry with
    //    AsyncModuleExecutionFulfilled. In InnerModuleEvaluation, the value of a
    //    module's [[AsyncEvaluationOrder]] internal slot is unused when its
    //    [[EvaluationError]] internal slot is not empty.

    {
        let status = module_src.status.borrow();

        // 9. If module.[[TopLevelCapability]] is not empty, then
        if let Some(cap) = status.top_level_capability() {
            // a. Assert: module.[[CycleRoot]] is module.
            debug_assert_eq!(status.cycle_root(), Some(module.clone()));

            // b. Perform ! Call(module.[[TopLevelCapability]].[[Reject]], undefined, « error »).
            cap.reject()
                .call(
                    &JsValue::undefined(),
                    &[error.clone().into_opaque(context.access())?],
                    context.access(),
                )
                .js_expect("default `reject` function cannot fail")?;
        }
    }

    // 10. For each Cyclic Module Record m of module.[[AsyncParentModules]], do
    for parent in module_src.evaluation.parents() {
        let m = parent.module();
        // A valid sibling's ordinary rejection does not drive a terminal
        // ancestor. Direct stale action entry still reports its interruption.
        if m.kind()
            .as_source_text()
            .is_some_and(|source| source.evaluation.cause().is_some())
        {
            continue;
        }
        // a. Perform AsyncModuleExecutionRejected(m, error).
        async_module_execution_rejected(&m, error, context)?;
    }
    // 11. Return unused.
    Ok(())
}

#[cfg(test)]
mod metadata_tests;

#[cfg(test)]
mod name_origin_tests;

#[cfg(test)]
mod constructor_cost_tests;

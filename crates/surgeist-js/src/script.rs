//! Boa's implementation of ECMAScript's Scripts.
//!
//! This module contains the [`Script`] type, which represents a [**Script Record**][script].
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!
//! [spec]: https://tc39.es/ecma262/#sec-scripts
//! [script]: https://tc39.es/ecma262/#sec-script-records

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use rustc_hash::FxHashMap;

use surgeist_js_gc::heap::{RecordRef, Rooted, Trace as HeapTrace};
use surgeist_js_interner::Interner;
use surgeist_js_parser::{Source, source::ReadChar};

use crate::{
    Context, ContextAccess, HostDefined, JsResult, JsString, JsValue, Module, SpannedSourceText,
    bytecompiler::{
        ByteCompiler,
        control::{CompilerWork, payload_bytes},
    },
    environments::EnvironmentStack,
    js_string,
    module::ModuleEdge,
    optimizer::Optimizer,
    realm::{Realm, RealmEdge},
    spanned_source_text::SourceText,
    vm::{
        ActivatedCode, ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock,
        prepare_function_object_fast, source_info::SourcePath,
    },
};

/// ECMAScript's [**Script Record**][spec].
///
/// [spec]: https://tc39.es/ecma262/#sec-script-records
#[derive(Clone)]
pub struct Script {
    inner: Rooted<ScriptRecord>,
}

impl std::fmt::Debug for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Script")
            .field("realm", &self.realm().identity())
            .field("phase", &self.inner.phase.borrow())
            .field("loaded_modules", &self.inner.loaded_modules)
            .finish()
    }
}

#[derive(Debug)]
enum ScriptPhase {
    Parsed(ParsedScript),
    Compiling,
    Compiled(CompiledScript),
    Starting,
    Installed(InstalledScript),
}

#[derive(Debug)]
struct ParsedScript {
    prepared: surgeist_js_ast::PreparedScript,
    interner: Interner,
}

#[derive(Debug)]
struct CompiledScript {
    parsed: ParsedScript,
    code: Rc<CodeBlock>,
}

#[derive(Debug)]
struct InstalledScript {
    code: Rc<CodeBlock>,
    declarations: surgeist_js_ast::ScriptDeclarations,
    scopes: surgeist_js_ast::scope::InstalledScopes,
}

struct Installation<'a> {
    permit: Option<surgeist_js_ast::scope::PublicationPermit<'a>>,
    environment: Option<crate::realm::PreparedGlobalEnvironment>,
    installed: Option<surgeist_js_ast::scope::InstalledScopes>,
}

impl Installation<'_> {
    /// No policy or program callback occurs between scope publication and slot installation.
    fn activate(&mut self, context: &mut Context) -> JsResult<()> {
        context.check_execution_stop()?;
        if self.installed.is_some() {
            return Ok(());
        }
        let installed = match self
            .permit
            .take()
            .expect("pending start owns its publication permit")
            .commit()
        {
            Ok(installed) => installed,
            Err(failure) => {
                let mut error = crate::JsError::from(failure);
                context.retain_preparation_error(&mut error);
                return Err(error);
            }
        };
        if let Some(environment) = self.environment.take() {
            if environment.creates_bindings() {
                context.note_execution_effect();
            }
            environment.install();
        }
        self.installed = Some(installed);
        Ok(())
    }
}

#[derive(Debug, Clone, HeapTrace)]
pub(crate) struct ScriptEdge(RecordRef<ScriptRecord>);

#[derive(HeapTrace)]
struct ScriptRecord {
    realm: RealmEdge,
    #[heap_trace(skip)]
    phase: RefCell<ScriptPhase>,
    #[heap_trace(skip)]
    source_text: SourceText,
    loaded_modules: RefCell<FxHashMap<JsString, ModuleEdge>>,
    host_defined: HostDefined,
    #[heap_trace(skip)]
    path: Option<PathBuf>,
}

impl Script {
    /// Gets the realm of this script.
    #[must_use]
    pub fn realm(&self) -> Realm {
        Realm::from_edge(self.inner.realm.clone())
    }

    pub(crate) fn edge(&self) -> ScriptEdge {
        ScriptEdge(self.inner.record_ref())
    }

    pub(crate) fn from_edge(edge: ScriptEdge) -> Result<Self, crate::error::RootingError> {
        let inner = edge
            .0
            .upgrade()
            .map_err(crate::error::RootingError::Access)?
            .ok_or(crate::error::RootingError::Expired)?;
        Ok(Self { inner })
    }

    /// Returns the [`ECMAScript specification`][spec] defined [`\[\[HostDefined\]\]`][`HostDefined`] field of the [`Module`].
    ///
    /// [spec]: https://tc39.es/ecma262/#script-record
    #[must_use]
    pub fn host_defined(&self) -> &HostDefined {
        &self.inner.host_defined
    }

    pub(crate) fn insert_loaded_module(&self, specifier: JsString, module: &Module) {
        let mut loaded_modules = self.inner.loaded_modules.borrow_mut();
        let edge = loaded_modules
            .entry(specifier)
            .or_insert_with(|| module.edge());
        debug_assert_eq!(edge, &module.edge());
    }

    /// Abstract operation [`ParseScript ( sourceText, realm, hostDefined )`][spec].
    ///
    /// Parses the provided `src` as an ECMAScript script, returning an error if parsing fails.
    ///
    /// Scope analysis remains private to this script. Parsing does not reserve
    /// realm lexical names, publish scope identities or instantiate global declarations.
    /// Realm declaration conflicts are checked when the script is evaluated.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-parse-script
    pub fn parse<R: ReadChar>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: ContextAccess<'_>,
    ) -> JsResult<Self> {
        let context = context.into_owner();
        let path = src.path().map(Path::to_path_buf);
        let realm = realm.unwrap_or_else(|| context.realm().clone());
        if !context.runtime().same_heap(&realm.runtime()) {
            return Err(crate::JsNativeError::typ()
                .with_message("script realm belongs to a different runtime")
                .into());
        }
        let identifier = realm.next_parser_identifier()?;
        let strict = context.is_strict();
        let scope = realm.scope().clone();
        let mut interner = Interner::new();
        let (code, source) = context.parse_source(src, |mut parser| {
            parser.set_identifier(identifier);
            if strict {
                parser.set_strict();
            }
            Ok(parser.parse_script_syntax_with_source(&mut interner)?)
        })?;
        let mut prepared = context.prepare_script_scope(code, &scope, &interner)?;
        if !context.optimizer_options().is_empty() {
            {
                let mut syntax = prepared.edit_tree()?;
                let options = context.optimizer_options();
                context.with_optimizer_work(|admission| {
                    Optimizer::new(options, &mut interner, admission).apply(syntax.statements_mut())
                })?;
            }
            context
                .with_scope_admission(|admission| prepared.refresh(&scope, &interner, admission))?;
        }

        let source_text = SourceText::new(source);

        Ok(Self {
            inner: context
                .runtime()
                .heap
                .allocate(ScriptRecord {
                    realm: realm.edge(),
                    phase: RefCell::new(ScriptPhase::Parsed(ParsedScript { prepared, interner })),
                    source_text,
                    loaded_modules: RefCell::default(),
                    host_defined: HostDefined::default(),
                    path,
                })
                .expect("script has valid heap edges"),
        })
    }

    /// Compiles the codeblock of this script.
    ///
    /// The compiled code is cached. A context from the script's runtime can
    /// compile it using the original parsed symbols and the script's realm.
    /// Pending code is inspectable without instantiating its declarations. If the
    /// realm changes before execution, preparation is refreshed and the pending
    /// cache may be replaced. Installed code retains its identity across unrelated
    /// realm changes. Returns an error if preparation is already in progress.
    pub fn codeblock(&self, context: ContextAccess<'_>) -> JsResult<Rc<CodeBlock>> {
        let context = context.into_owner();
        self.check_context(context)?;
        let mut phase = self.take_phase(ScriptPhase::Compiling)?;
        let mut previous_realm = self.realm();
        context.swap_realm(&mut previous_realm);
        let result = self.ensure_code(
            &mut phase,
            &mut context.guard(move |context| context.swap_realm(&mut previous_realm)),
        );
        *self.inner.phase.borrow_mut() = phase;
        result
    }

    fn check_context(&self, context: &Context) -> JsResult<()> {
        if !self.inner.runtime().same_heap(&context.runtime().heap) {
            return Err(crate::JsNativeError::typ()
                .with_message("script belongs to a different runtime")
                .into());
        }
        Ok(())
    }

    fn take_phase(&self, sentinel: ScriptPhase) -> JsResult<ScriptPhase> {
        let mut phase = self.inner.phase.borrow_mut();
        if matches!(*phase, ScriptPhase::Compiling | ScriptPhase::Starting) {
            return Err(crate::JsNativeError::typ()
                .with_message("script preparation is already in progress")
                .into());
        }
        Ok(std::mem::replace(&mut *phase, sentinel))
    }

    fn ensure_code(
        &self,
        phase: &mut ScriptPhase,
        context: &mut Context,
    ) -> JsResult<Rc<CodeBlock>> {
        if let ScriptPhase::Installed(installed) = phase {
            return context.with_compilation_work(|mut control| {
                control.admit(CompilerWork::MetadataVisit)?;
                installed
                    .code
                    .scope_association
                    .resolve_live(&installed.scopes)?;
                Ok(installed.code.clone())
            });
        }
        let parsed = match phase {
            ScriptPhase::Parsed(parsed) => parsed,
            ScriptPhase::Compiled(compiled) => &mut compiled.parsed,
            _ => unreachable!("owned preparation is outside the reentrancy sentinel"),
        };
        let stale = if parsed.prepared.tree().is_err() {
            true
        } else {
            let parts = parsed.prepared.parts()?;
            context.with_compilation_work(|mut control| {
                control.admit(CompilerWork::MetadataVisit)?;
                control.with_scope_admission(|admission| {
                    parts
                        .scopes
                        .validate_basis(admission)
                        .map_err(crate::JsError::from)
                })
            })? == surgeist_js_ast::scope::BasisStatus::Stale
        };
        if stale {
            let root = self.realm().scope().clone();
            context.with_scope_admission(|admission| {
                parsed.prepared.refresh(&root, &parsed.interner, admission)
            })?;
        }
        if !stale && let ScriptPhase::Compiled(compiled) = phase {
            return Ok(compiled.code.clone());
        }
        let owned = std::mem::replace(phase, ScriptPhase::Compiling);
        let mut parsed = match owned {
            ScriptPhase::Parsed(parsed) => parsed,
            ScriptPhase::Compiled(compiled) => compiled.parsed,
            _ => unreachable!("preparation has analyzed syntax"),
        };
        match self.compile(&mut parsed, context) {
            Ok(code) => {
                *phase = ScriptPhase::Compiled(CompiledScript {
                    parsed,
                    code: code.clone(),
                });
                Ok(code)
            }
            Err(error) => {
                *phase = ScriptPhase::Parsed(parsed);
                Err(error)
            }
        }
    }

    fn compile(&self, parsed: &mut ParsedScript, context: &mut Context) -> JsResult<Rc<CodeBlock>> {
        let parts = parsed.prepared.parts()?;
        let scope_owner = parts.scopes;
        let (code, extension) = context.with_compilation_work(|mut control| {
            let _depth = control.enter()?;
            control.admit(CompilerWork::MetadataVisit)?;
            let root = scope_owner.root_handle();
            let mut extension = control.with_scope_admission(|admission| {
                let owner = scope_owner;
                owner
                    .begin_extension(admission)
                    .map_err(crate::JsError::from)
            })?;
            let path = if let Some(path) = self.path() {
                let bytes = payload_bytes::<u8>(path.as_os_str().len())?;
                control.admit(CompilerWork::Read { bytes })?;
                control.admit(CompilerWork::Capacity { bytes })?;
                control.admit(CompilerWork::Copy { bytes })?;
                SourcePath::Path(Rc::from(path))
            } else {
                SourcePath::None
            };
            let mut compiler = ByteCompiler::new(
                js_string!("<main>"),
                parts.tree.strict(),
                false,
                root.clone(),
                root,
                false,
                false,
                &mut parsed.interner,
                false,
                SpannedSourceText::new_source_only(self.get_source()),
                path,
                &mut extension,
                control.reborrow(),
            )?;
            compiler.global_declaration_instantiation(parts.tree, parts.declarations)?;
            compiler.compile_statement_list(parts.tree.statements(), true, false)?;
            let code = compiler.finish(None)?;
            let bytes = payload_bytes::<CodeBlock>(1)?;
            control.admit(CompilerWork::Capacity { bytes })?;
            control.admit(CompilerWork::Copy { bytes })?;
            control.admit(CompilerWork::MetadataWrite)?;
            Ok((Rc::new(code), extension))
        })?;
        extension.commit_private();
        Ok(code)
    }

    /// Evaluates this script and returns its result.
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobExecutor::run_jobs`] on the provided queue to run them.
    ///
    /// [`JobExecutor::run_jobs`]: crate::job::JobExecutor::run_jobs
    pub fn evaluate(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        let context = context.into_owner();
        self.prepare_run(context)?;
        let record = context.run();

        context.vm.pop_frame();

        record.consume()
    }

    /// Evaluates this script and returns its result, periodically yielding to the executor
    /// in order to avoid blocking the current thread.
    ///
    /// This uses an implementation defined amount of "clock cycles" that need to pass before
    /// execution is suspended. See [`Script::evaluate_async_with_budget`] if you want to also
    /// customize this parameter.
    #[allow(clippy::future_not_send)]
    pub async fn evaluate_async(&self, context: ContextAccess<'_>) -> JsResult<JsValue> {
        self.evaluate_async_with_budget(context, 256).await
    }

    /// Evaluates this script and returns its result, yielding to the executor each time `budget`
    /// number of "clock cycles" pass.
    ///
    /// Note that "clock cycle" is in quotation marks because we can't determine exactly how many
    /// CPU clock cycles a VM instruction will take, but all instructions have a "cost" associated
    /// with them that depends on their individual complexity. We'd recommend benchmarking with
    /// different budget sizes in order to find the ideal yielding time for your application.
    ///
    /// Dropping the pending future releases its invocation's frames and operand storage.
    /// Effects already performed by the script remain; dropping does not run guest cleanup
    /// or cancel jobs already transferred to the executor. An unpolled future runs no code.
    #[allow(clippy::future_not_send)]
    pub async fn evaluate_async_with_budget(
        &self,
        context: ContextAccess<'_>,
        budget: u32,
    ) -> JsResult<JsValue> {
        let context = context.into_owner();
        let checkpoint = context.vm.call_checkpoint();
        let mut active = context.guard(move |context| context.vm.restore_call(checkpoint));
        self.prepare_run(&mut active)?;
        active.run_async_with_budget(budget).await.consume()
    }

    fn prepare_run(&self, context: &mut Context) -> JsResult<()> {
        self.check_context(context)?;
        let mut phase = self.take_phase(ScriptPhase::Starting)?;
        let prepared_code = {
            let mut previous_realm = self.realm();
            context.swap_realm(&mut previous_realm);
            self.ensure_code(
                &mut phase,
                &mut context.guard(move |context| context.swap_realm(&mut previous_realm)),
            )
        };
        let code = match prepared_code {
            Ok(code) => code,
            Err(error) => {
                *self.inner.phase.borrow_mut() = phase;
                return Err(error);
            }
        };
        let (installed, result) = match &mut phase {
            ScriptPhase::Compiled(compiled) => match compiled.parsed.prepared.parts() {
                Ok(parts) => self.start_code(
                    code.clone(),
                    parts.declarations,
                    Some(parts.scopes),
                    None,
                    context,
                ),
                Err(error) => (None, Err(error.into())),
            },
            ScriptPhase::Installed(installed) => self.start_code(
                code.clone(),
                &installed.declarations,
                None,
                Some(installed.scopes.clone()),
                context,
            ),
            _ => unreachable!("successful preparation owns compiled code"),
        };
        if let Some(scopes) = installed {
            phase = match phase {
                ScriptPhase::Compiled(compiled) => ScriptPhase::Installed(InstalledScript {
                    code: compiled.code,
                    declarations: compiled.parsed.prepared.into_declarations(),
                    scopes,
                }),
                installed @ ScriptPhase::Installed(_) => installed,
                _ => unreachable!("only compiled or installed code can activate"),
            };
        }
        *self.inner.phase.borrow_mut() = phase;
        result
    }

    fn start_code(
        &self,
        code: Rc<CodeBlock>,
        declarations: &surgeist_js_ast::ScriptDeclarations,
        scopes: Option<&mut surgeist_js_ast::scope::PreparedScopes>,
        installed: Option<surgeist_js_ast::scope::InstalledScopes>,
        context: &mut Context,
    ) -> (
        Option<surgeist_js_ast::scope::InstalledScopes>,
        JsResult<()>,
    ) {
        let mut installation = Installation {
            permit: None,
            environment: None,
            installed,
        };
        let result = (|| {
            let realm = self.realm();
            let root = realm.scope().clone();
            context.with_installation_work(|admission| {
                declarations
                    .check_against_live(&root, admission)
                    .map_err(crate::JsError::from)
            })?;
            if let Some(scopes) = scopes {
                let handle = scopes.root_handle();
                let count = context.with_installation_work(|admission| {
                    scopes.with_access(admission, |access| {
                        Ok::<_, crate::JsError>(access.num_bindings(&handle)?)
                    })
                })?;
                installation.environment =
                    Some(context.with_installation_work(|mut admission| {
                        if let Some(admit) = admission.as_mut() {
                            admit(surgeist_js_ast::scope::ScopeWork::Capacity {
                                elements: count as usize,
                            })
                            .map_err(crate::error::RuntimeLimitError::from)?;
                        }
                        realm.prepare_global_env(count)
                    })?);
                installation.permit = Some(context.with_installation_work(|admission| {
                    scopes
                        .prepare_publication(admission)
                        .map_err(crate::JsError::from)
                })?);
            }
            context.prepare_guest_call_storage(0, code.register_count)?;
            let environments = EnvironmentStack::new();
            let runnable = Some(ActiveRunnable::Script(self.clone()));
            let recipes = {
                let mut previous_realm = realm.clone();
                context.swap_realm(&mut previous_realm);
                let mut context =
                    context.guard(move |context| context.swap_realm(&mut previous_realm));
                let mut recipes = Vec::new();
                recipes
                    .try_reserve(code.global_fns.len())
                    .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
                for function in &code.global_fns {
                    recipes.push(prepare_function_object_fast(
                        code.constant_function(function.function_index as usize),
                        &environments,
                        runnable.clone(),
                        realm.clone(),
                        &mut context,
                    )?);
                }
                recipes
            };
            context.vm.push_host_start(CallFrame::host_start(
                code.clone(),
                runnable.clone(),
                environments.clone(),
                realm.clone(),
            ))?;
            let start: JsResult<CallFrame> = (|| {
                let eligible = self.annex_b_preflights(&code, declarations, context)?;
                context.preflight_global_declarations(&code, declarations)?;
                if installation
                    .environment
                    .as_ref()
                    .is_some_and(|environment| environment.creates_bindings())
                {
                    installation.activate(context)?;
                }
                for (function, recipe) in code.global_fns.iter().zip(recipes) {
                    let function_code = code.constant_function(function.function_index as usize);
                    let name = code.constant_string(function.name_index as usize);
                    context.create_global_function_binding_prepared(name, false, |context| {
                        installation.activate(context)?;
                        recipe.activate(ActivatedCode::from_installed(function_code)?)
                    })?;
                }
                for name in declarations.var_names() {
                    if !code
                        .global_fns
                        .iter()
                        .any(|function| code.constant_string(function.name_index as usize) == *name)
                    {
                        context.create_global_var_binding(name.clone(), false)?;
                    }
                }
                let entry = context.reserve_guest_entry(code.clone(), 0)?;
                installation.activate(context)?;
                let frame = CallFrame::new(
                    ActivatedCode::from_installed(code.clone())?,
                    entry,
                    runnable,
                    environments,
                    realm,
                )?
                .with_env_fp(0)
                .with_flags(CallFrameFlags::EXIT_EARLY)
                .with_annex_b_eligibility(eligible);
                Ok(frame)
            })();
            context
                .vm
                .pop_frame()
                .expect("script start frame remains owned");
            let frame = start?;
            context
                .vm
                .push_frame_with_stack(frame, JsValue::undefined(), JsValue::null());
            Ok(())
        })();
        (installation.installed, result)
    }

    fn annex_b_preflights(
        &self,
        code: &CodeBlock,
        declarations: &surgeist_js_ast::ScriptDeclarations,
        context: &mut Context,
    ) -> JsResult<Vec<JsString>> {
        let mut eligible = Vec::new();
        let mut declared = declarations.var_names().to_vec();
        for name in &code.annex_b_candidates {
            if !self.realm().scope().has_lex_binding(name)
                && context.can_declare_global_function(name)?
            {
                if !declared.contains(name) {
                    context.create_global_var_binding(name.clone(), false)?;
                    declared.push(name.clone());
                }
                eligible.push(name.clone());
            }
        }
        Ok(eligible)
    }

    pub(super) fn path(&self) -> Option<&Path> {
        self.inner.path.as_deref()
    }

    pub(super) fn get_source(&self) -> SourceText {
        self.inner.source_text.clone()
    }
}

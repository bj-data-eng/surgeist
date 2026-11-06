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
use surgeist_js_parser::{Parser, Source, source::ReadChar};

use crate::{
    Context, HostDefined, JsResult, JsString, JsValue, Module, SpannedSourceText,
    bytecompiler::{ByteCompiler, global_declaration_instantiation_context},
    environments::EnvironmentStack,
    js_string,
    module::ModuleEdge,
    optimizer::Optimizer,
    realm::{Realm, RealmEdge},
    spanned_source_text::SourceText,
    vm::{ActiveRunnable, CallFrame, CallFrameFlags, CodeBlock},
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
    Ast {
        source: surgeist_js_ast::Script,
        interner: Interner,
    },
    Compiling,
    Codeblock(Rc<CodeBlock>),
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

    pub(crate) fn from_edge(edge: ScriptEdge) -> Self {
        let inner = edge
            .0
            .upgrade()
            .expect("script heap is available")
            .expect("live owner retains its script");
        Self { inner }
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
    /// [spec]: https://tc39.es/ecma262/#sec-parse-script
    pub fn parse<R: ReadChar>(
        src: Source<'_, R>,
        realm: Option<Realm>,
        context: &mut Context,
    ) -> JsResult<Self> {
        let path = src.path().map(Path::to_path_buf);
        let realm = realm.unwrap_or_else(|| context.realm().clone());
        if !context.runtime().same_heap(&realm.runtime()) {
            return Err(crate::JsNativeError::typ()
                .with_message("script realm belongs to a different runtime")
                .into());
        }
        let mut parser = Parser::new(src);
        parser.set_identifier(realm.next_parser_identifier()?);
        if context.is_strict() {
            parser.set_strict();
        }
        let scope = realm.scope().clone();
        let mut interner = Interner::new();
        let (mut code, source) = parser.parse_script_with_source(&scope, &mut interner)?;
        if !context.optimizer_options().is_empty() {
            Optimizer::new(context, &mut interner).apply(code.statements_mut());
        }

        let source_text = SourceText::new(source);

        Ok(Self {
            inner: context
                .runtime()
                .heap
                .allocate(ScriptRecord {
                    realm: realm.edge(),
                    phase: RefCell::new(ScriptPhase::Ast {
                        source: code,
                        interner,
                    }),
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
    /// Returns an error if compilation is already in progress.
    pub fn codeblock(&self, context: &mut Context) -> JsResult<Rc<CodeBlock>> {
        if !self.inner.runtime().same_heap(&context.runtime().heap) {
            return Err(crate::JsNativeError::typ()
                .with_message("script belongs to a different runtime")
                .into());
        }
        let (source, mut interner) = {
            let mut phase = self.inner.phase.borrow_mut();
            match &*phase {
                ScriptPhase::Codeblock(codeblock) => {
                    return Ok(codeblock.clone());
                }
                ScriptPhase::Compiling => {
                    return Err(crate::JsNativeError::typ()
                        .with_message("script compilation is already in progress")
                        .into());
                }
                ScriptPhase::Ast { .. } => {}
            }
            let ScriptPhase::Ast { source, interner } =
                std::mem::replace(&mut *phase, ScriptPhase::Compiling)
            else {
                unreachable!("the script phase is still parsed");
            };
            (source, interner)
        };

        let mut previous_realm = self.realm();
        context.swap_realm(&mut previous_realm);
        let result = self.compile(
            &source,
            &mut interner,
            &mut context.guard(move |context| context.swap_realm(&mut previous_realm)),
        );

        *self.inner.phase.borrow_mut() = match &result {
            Ok(codeblock) => ScriptPhase::Codeblock(codeblock.clone()),
            Err(_) => ScriptPhase::Ast { source, interner },
        };
        result
    }

    fn compile(
        &self,
        source: &surgeist_js_ast::Script,
        interner: &mut Interner,
        context: &mut Context,
    ) -> JsResult<Rc<CodeBlock>> {
        let scope = context.realm().scope().clone();
        let mut annex_b_function_names = Vec::new();
        global_declaration_instantiation_context(
            &mut annex_b_function_names,
            source,
            interner,
            &scope,
            context,
        )?;

        let mut compiler = ByteCompiler::new(
            js_string!("<main>"),
            source.strict(),
            false,
            scope.clone(),
            scope,
            false,
            false,
            interner,
            false,
            SpannedSourceText::new_source_only(self.get_source()),
            self.path().map(Path::to_owned).into(),
        );

        #[cfg(feature = "annex-b")]
        {
            compiler.annex_b_function_names = annex_b_function_names;
        }
        compiler.global_declaration_instantiation(source);
        compiler.compile_statement_list(source.statements(), true, false);

        Ok(Rc::new(compiler.finish()))
    }

    /// Evaluates this script and returns its result.
    ///
    /// Note that this won't run any scheduled promise jobs; you need to call [`Context::run_jobs`]
    /// on the context or [`JobExecutor::run_jobs`] on the provided queue to run them.
    ///
    /// [`JobExecutor::run_jobs`]: crate::job::JobExecutor::run_jobs
    pub fn evaluate(&self, context: &mut Context) -> JsResult<JsValue> {
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
    pub async fn evaluate_async(&self, context: &mut Context) -> JsResult<JsValue> {
        self.evaluate_async_with_budget(context, 256).await
    }

    /// Evaluates this script and returns its result, yielding to the executor each time `budget`
    /// number of "clock cycles" pass.
    ///
    /// Note that "clock cycle" is in quotation marks because we can't determine exactly how many
    /// CPU clock cycles a VM instruction will take, but all instructions have a "cost" associated
    /// with them that depends on their individual complexity. We'd recommend benchmarking with
    /// different budget sizes in order to find the ideal yielding time for your application.
    #[allow(clippy::future_not_send)]
    pub async fn evaluate_async_with_budget(
        &self,
        context: &mut Context,
        budget: u32,
    ) -> JsResult<JsValue> {
        self.prepare_run(context)?;

        let record = context.run_async_with_budget(budget).await;

        context.vm.pop_frame();

        record.consume()
    }

    fn prepare_run(&self, context: &mut Context) -> JsResult<()> {
        let codeblock = self.codeblock(context)?;

        let global_env = EnvironmentStack::new();
        context.vm.push_frame_with_stack(
            CallFrame::new(
                codeblock.clone(),
                Some(ActiveRunnable::Script(self.clone())),
                global_env,
                self.realm(),
            )
            .with_env_fp(0)
            .with_flags(CallFrameFlags::EXIT_EARLY),
            JsValue::undefined(),
            JsValue::null(),
        );

        self.realm().resize_global_env();

        context
            .global_declaration_instantiation(&codeblock)
            .inspect_err(|_| {
                context.vm.pop_frame();
            })?;

        Ok(())
    }

    pub(super) fn path(&self) -> Option<&Path> {
        self.inner.path.as_deref()
    }

    pub(super) fn get_source(&self) -> SourceText {
        self.inner.source_text.clone()
    }
}

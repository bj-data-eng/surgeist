use crate::JsResult;
use std::rc::Rc;

use crate::{
    JsString, SpannedSourceText,
    builtins::function::ThisMode,
    bytecompiler::{
        ByteCompiler,
        control::{CompilerControl, CompilerWork, payload_bytes},
    },
    js_string,
    vm::{CodeBlock, CodeBlockFlags, source_info::SourcePath},
};
use surgeist_js_ast::{
    function::{FormalParameterList, FunctionBody},
    scope::{FunctionScopes, ScopeExtension, ScopeHandle},
};
use surgeist_js_interner::Interner;

/// `FunctionCompiler` is used to compile AST functions to bytecode.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct FunctionCompiler {
    name: JsString,
    generator: bool,
    r#async: bool,
    strict: bool,
    arrow: bool,
    method: bool,
    in_with: bool,
    force_function_scope: bool,
    name_scope: Option<ScopeHandle>,
    spanned_source_text: SpannedSourceText,
    source_path: SourcePath,
}

impl FunctionCompiler {
    /// Create a new `FunctionCompiler`.
    pub(crate) fn new(spanned_source_text: SpannedSourceText) -> Self {
        Self {
            name: js_string!(),
            generator: false,
            r#async: false,
            strict: false,
            arrow: false,
            method: false,
            in_with: false,
            force_function_scope: false,
            name_scope: None,
            spanned_source_text,
            source_path: SourcePath::None,
        }
    }

    /// Set the name of the function.
    pub(crate) fn name<N>(mut self, name: N) -> Self
    where
        N: Into<Option<JsString>>,
    {
        let name = name.into();
        if let Some(name) = name {
            self.name = name;
        }
        self
    }

    /// Indicate if the function is an arrow function.
    pub(crate) const fn arrow(mut self, arrow: bool) -> Self {
        self.arrow = arrow;
        self
    }
    /// Indicate if the function is a method function.
    pub(crate) const fn method(mut self, method: bool) -> Self {
        self.method = method;
        self
    }
    /// Indicate if the function is a generator function.
    pub(crate) const fn generator(mut self, generator: bool) -> Self {
        self.generator = generator;
        self
    }

    /// Indicate if the function is an async function.
    pub(crate) const fn r#async(mut self, r#async: bool) -> Self {
        self.r#async = r#async;
        self
    }

    /// Indicate if the function is in a strict context.
    pub(crate) const fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Provide the name scope of the function.
    pub(crate) fn name_scope(mut self, name_scope: Option<ScopeHandle>) -> Self {
        self.name_scope = name_scope;
        self
    }

    /// Indicate if the function is in a `with` statement.
    pub(crate) const fn in_with(mut self, in_with: bool) -> Self {
        self.in_with = in_with;
        self
    }

    /// Indicate if the function is in a `with` statement.
    pub(crate) const fn force_function_scope(mut self, force_function_scope: bool) -> Self {
        self.force_function_scope = force_function_scope;
        self
    }

    /// Set source map file path.
    pub(crate) fn source_path(mut self, source_path: SourcePath) -> Self {
        self.source_path = source_path;
        self
    }

    /// Compile a function statement list and it's parameters into bytecode.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compile<'compile, 'store>(
        mut self,
        parameters: &FormalParameterList,
        body: &FunctionBody,
        variable_environment: ScopeHandle,
        lexical_environment: ScopeHandle,
        scopes: &FunctionScopes,
        contains_direct_eval: bool,
        interner: &'compile mut Interner,
        scopes_access: &'compile mut ScopeExtension<'store>,
        mut control: CompilerControl<'compile>,
    ) -> JsResult<Rc<CodeBlock>> {
        let _depth = control.enter()?;
        control.admit(CompilerWork::Visit)?;
        control.admit(CompilerWork::MetadataVisit)?;
        control.admit(CompilerWork::MetadataWrite)?;
        self.strict = self.strict || body.strict();

        let length = parameters.length();

        let mut compiler = ByteCompiler::new(
            self.name,
            self.strict,
            false,
            variable_environment,
            lexical_environment,
            self.r#async,
            self.generator,
            interner,
            self.in_with,
            self.spanned_source_text,
            self.source_path,
            scopes_access,
            control,
        )?;

        compiler.control.admit(CompilerWork::MetadataWrite)?;
        compiler.length = length;
        compiler.code_block_flags.set(
            CodeBlockFlags::HAS_PROTOTYPE_PROPERTY,
            !self.arrow && !self.method && !self.r#async && !self.generator,
        );

        if self.arrow {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.this_mode = ThisMode::Lexical;
        }

        if let Some(scope) = self.name_scope
            && !compiler.scope_all_bindings_local(&scope)?
        {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags |= CodeBlockFlags::HAS_BINDING_IDENTIFIER;
            let _ = compiler.push_scope(&scope)?;
        }

        if contains_direct_eval || !compiler.scope_all_bindings_local(scopes.function_scope())? {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags |= CodeBlockFlags::HAS_FUNCTION_SCOPE;
        } else if !self.arrow {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.code_block_flags.set(
                CodeBlockFlags::HAS_FUNCTION_SCOPE,
                self.force_function_scope || scopes.requires_function_scope(),
            );
        }

        compiler.control.admit(CompilerWork::MetadataVisit)?;
        if compiler.code_block_flags.has_function_scope() {
            let _ = compiler.push_scope(scopes.function_scope())?;
        } else {
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.variable_scope = scopes.function_scope().clone();
            compiler.lexical_scope = scopes.function_scope().clone();
        }

        // Taken from:
        //  - 15.9.3 Runtime Semantics: EvaluateAsyncConciseBody: <https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncconcisebody>
        //  - 15.8.4 Runtime Semantics: EvaluateAsyncFunctionBody: <https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncfunctionbody>
        //
        // Note: In `EvaluateAsyncGeneratorBody` unlike the async non-generator functions we don't handle exceptions thrown by
        // `FunctionDeclarationInstantiation` (so they are propagated).
        //
        // See: 15.6.2 Runtime Semantics: EvaluateAsyncGeneratorBody: https://tc39.es/ecma262/#sec-runtime-semantics-evaluateasyncgeneratorbody
        compiler.control.admit(CompilerWork::MetadataVisit)?;
        if compiler.is_async() && !compiler.is_generator() {
            // 1. Let promiseCapability be ! NewPromiseCapability(%Promise%).
            compiler
                .bytecode
                .emit_create_promise_capability(&mut compiler.control)?;

            // 2. Let declResult be Completion(FunctionDeclarationInstantiation(functionObject, argumentsList)).
            //
            // Note: We push an exception handler so we catch exceptions that are thrown by the
            // `FunctionDeclarationInstantiation` abstract function.
            //
            // Patched in `ByteCompiler::finish()`.
            compiler.async_handler = Some(compiler.push_handler()?);
        }

        compiler.function_declaration_instantiation(
            body,
            parameters,
            self.arrow,
            self.strict,
            self.generator,
            scopes,
        )?;

        // Taken from:
        // - 27.6.3.2 AsyncGeneratorStart ( generator, generatorBody ): <https://tc39.es/ecma262/#sec-asyncgeneratorstart>
        //
        // Note: We do handle exceptions thrown by generator body in `AsyncGeneratorStart`.
        compiler.control.admit(CompilerWork::MetadataVisit)?;
        if compiler.is_generator() {
            assert!(compiler.async_handler.is_none());

            if compiler.is_async() {
                // Patched in `ByteCompiler::finish()`.
                compiler.async_handler = Some(compiler.push_handler()?);
            }
        }

        {
            let mut compiler = compiler.position_guard(body)?;
            compiler.compile_statement_list(body.statement_list(), false, false)?;
            compiler.finish()?;
        }

        compiler.control.admit(CompilerWork::MetadataWrite)?;
        compiler.parameter_scope = scopes.parameter_scope();

        prepare_code_storage(&mut compiler.control)?;
        let code = compiler.finish(Some(parameters))?;

        Ok(Rc::new(code))
    }
}

/// Qualifies the fixed compiled-record payload before transferring it to an Rc.
/// Native Rc descriptor allocation remains outside the logical payload quote.
pub(super) fn prepare_code_storage(control: &mut CompilerControl<'_>) -> JsResult<()> {
    let bytes = payload_bytes::<CodeBlock>(1)?;
    control.admit(CompilerWork::Capacity { bytes })?;
    control.admit(CompilerWork::Copy { bytes })?;
    control.admit(CompilerWork::MetadataWrite)
}

#[cfg(test)]
mod tests {
    use crate::{
        Context, Source,
        error::{EngineError, RuntimeLimitError},
        execution::{ExecutionOptions, ProfileMode},
    };

    const SOURCE: &str = "
        function outer(a, b) {
            function inner(c = 1) { return c + a; }
            arguments[0] = 40;
            return inner(b);
        }
        outer.length * 100 + outer(0, 2);
    ";

    #[test]
    fn nested_function_work_exact_and_one_short_preserve_formals_and_fresh_retry() {
        let mut baseline = Context::builder().build().unwrap();
        let complete = baseline
            .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(512))
            .unwrap();
        assert_eq!(complete.result.unwrap(), 242.into());
        let work = complete.report.compiler_work().unwrap();
        assert!(work > 0);
        for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
            let mut exact = Context::builder().build().unwrap();
            let result = exact
                .eval_with_budget(
                    Source::from_bytes(SOURCE),
                    ExecutionOptions::new(512)
                        .with_profile(profile)
                        .with_compiler_work_limit(work),
                )
                .unwrap();
            assert_eq!(result.result.unwrap(), 242.into());
            assert_eq!(result.report.compiler_work().unwrap(), work);

            let limit = work - 1;
            let mut retry = Context::builder().build().unwrap();
            let stopped = retry
                .eval_with_budget(
                    Source::from_bytes(SOURCE),
                    ExecutionOptions::new(512)
                        .with_profile(profile)
                        .with_compiler_work_limit(limit),
                )
                .unwrap();
            assert_eq!(
                stopped.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::CompilerWork { limit },
                ))
            );
            assert!(!stopped.report.effects_started());
            assert_eq!(retry.eval(Source::from_bytes(SOURCE)).unwrap(), 242.into());
        }
    }

    #[test]
    fn nested_functions_share_the_structural_depth_limit() {
        let mut baseline = Context::builder().build().unwrap();
        let complete = baseline
            .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(512))
            .unwrap();
        assert_eq!(complete.result.unwrap(), 242.into());
        let peak = complete.report.compiler_depth_peak();
        assert!(peak > 1);
        let mut exact = Context::builder().build().unwrap();
        let exact = exact
            .eval_with_budget(
                Source::from_bytes(SOURCE),
                ExecutionOptions::new(512).with_compiler_depth_limit(peak),
            )
            .unwrap();
        assert_eq!(exact.result.unwrap(), 242.into());
        assert_eq!(exact.report.compiler_depth_peak(), peak);

        let limit = peak - 1;
        let mut retry = Context::builder().build().unwrap();
        let stopped = retry
            .eval_with_budget(
                Source::from_bytes(SOURCE),
                ExecutionOptions::new(512).with_compiler_depth_limit(limit),
            )
            .unwrap();
        assert_eq!(
            stopped.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::CompilerDepth { limit },
            ))
        );
        assert!(stopped.report.compiler_depth_peak() <= limit);
        assert!(!stopped.report.effects_started());
        assert_eq!(retry.eval(Source::from_bytes(SOURCE)).unwrap(), 242.into());
    }
}

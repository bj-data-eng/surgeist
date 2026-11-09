use super::{
    BindingAccessOpcode, ToJsString,
    control::{CompilerControl, CompilerWork, payload_bytes, reserve, size_overflow},
    index::CompilerIndex,
};
use crate::{
    JsResult, SpannedSourceText,
    bytecompiler::{ByteCompiler, FunctionCompiler, FunctionSpec, NodeKind},
    vm::{CallFrame, GlobalFunctionBinding, opcode::BindingOpcode},
};
use surgeist_js_ast::{
    Script, ScriptDeclarations,
    declaration::Binding,
    function::{FormalParameterList, FunctionBody},
    operations::{
        LexicallyScopedDeclaration, VarScopedDeclaration, try_bound_names,
        try_lexically_declared_names, try_lexically_scoped_declarations, try_var_declared_names,
        try_var_scoped_declarations,
    },
    scope::{FunctionScopes, ScopeHandle},
    scope_analyzer::EvalDeclarationBindings,
    visitor::NodeRef,
};
use surgeist_js_interner::Sym;

#[cfg(feature = "annex-b")]
use surgeist_js_ast::operations::try_annex_b_function_declarations_names;

fn contains_name(names: &[Sym], name: Sym, control: &mut CompilerControl<'_>) -> JsResult<bool> {
    for candidate in names {
        control.admit(CompilerWork::Search)?;
        if *candidate == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn retain_declaration<T>(
    values: &mut Vec<T>,
    value: T,
    control: &mut CompilerControl<'_>,
) -> JsResult<()> {
    reserve(values, 1, control)?;
    control.admit(CompilerWork::MetadataWrite)?;
    control.admit(CompilerWork::Copy {
        bytes: std::mem::size_of::<T>(),
    })?;
    values.push(value);
    Ok(())
}

fn reverse_declarations<T>(values: &mut [T], control: &mut CompilerControl<'_>) -> JsResult<()> {
    for left in 0..values.len() / 2 {
        control.admit(CompilerWork::MetadataVisit)?;
        control.admit(CompilerWork::MetadataVisit)?;
        control.admit(CompilerWork::Copy {
            bytes: payload_bytes::<T>(2)?,
        })?;
        control.admit(CompilerWork::MetadataWrite)?;
        control.admit(CompilerWork::MetadataWrite)?;
        values.swap(left, values.len() - 1 - left);
    }
    Ok(())
}

fn copy_names(names: &[Sym], control: &mut CompilerControl<'_>) -> JsResult<Vec<Sym>> {
    let mut copy = Vec::new();
    reserve(&mut copy, names.len(), control)?;
    for name in names {
        control.admit(CompilerWork::MetadataVisit)?;
        control.admit(CompilerWork::MetadataWrite)?;
        control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<Sym>(),
        })?;
        copy.push(*name);
    }
    Ok(copy)
}

impl ByteCompiler<'_, '_> {
    /// `GlobalDeclarationInstantiation ( script, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-globaldeclarationinstantiation
    pub(crate) fn global_declaration_instantiation(
        &mut self,
        script: &Script,
        _declarations: &ScriptDeclarations,
    ) -> JsResult<()> {
        // Live lexical/var declaration checks use authored metadata at execution.
        #[cfg(feature = "annex-b")]
        {
            self.control.admit(CompilerWork::MetadataWrite)?;
            self.script_global_annex = true;
            // The AST owner already excludes authored top-level lexical names
            // and preserves original candidate order, including duplicates.
            for candidate in _declarations.annex_b_candidates() {
                self.control.admit(CompilerWork::MetadataVisit)?;
                self.control.admit(CompilerWork::Copy {
                    bytes: std::mem::size_of::<crate::JsString>(),
                })?;
                retain_declaration(
                    &mut self.annex_b_candidates,
                    candidate.clone(),
                    &mut self.control,
                )?;
            }
        }

        // 5. Let varDeclarations be the VarScopedDeclarations of script.
        // Note: VarScopedDeclarations for a Script node is TopLevelVarScopedDeclarations.
        let var_declarations = self
            .control
            .with_name_admission(|admission| try_var_scoped_declarations(script, admission))?;

        // 6. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 7. Let declaredFunctionNames be a new empty List.
        let mut declared_function_names = Vec::new();

        // 8. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
            // a.iii. Let fn be the sole element of the BoundNames of d.
            let name = match *declaration {
                VarScopedDeclaration::FunctionDeclaration(f) => f.name(),
                VarScopedDeclaration::GeneratorDeclaration(f) => f.name(),
                VarScopedDeclaration::AsyncFunctionDeclaration(f) => f.name(),
                VarScopedDeclaration::AsyncGeneratorDeclaration(f) => f.name(),
                VarScopedDeclaration::VariableDeclaration(_) => continue,
            };

            // a.iv. If declaredFunctionNames does not contain fn, then
            if !contains_name(&declared_function_names, name.sym(), &mut self.control)? {
                // 1. Let fnDefinable be ? env.CanDeclareGlobalFunction(fn).
                // 2. If fnDefinable is false, throw a TypeError exception.
                // Done in `Context::global_declaration_instantiation`.
                // The names checked here are the same names from the functions
                // in `functions_to_initialize`, but in reverse order, so we can
                // reuse `global_fns` for this check.

                // 3. Append fn to declaredFunctionNames.
                retain_declaration(&mut declared_function_names, name.sym(), &mut self.control)?;

                // 4. Insert d as the first element of functionsToInitialize.
                retain_declaration(
                    &mut functions_to_initialize,
                    *declaration,
                    &mut self.control,
                )?;
            }
        }

        reverse_declarations(&mut functions_to_initialize, &mut self.control)?;

        // 9. Let declaredVarNames be a new empty List.
        let mut declared_var_names = Vec::new();

        // 10. For each element d of varDeclarations, do
        //     a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
        for declaration in var_declarations {
            self.control.admit(CompilerWork::MetadataVisit)?;
            let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
                continue;
            };

            // i. For each String vn of the BoundNames of d, do
            for name in self
                .control
                .with_name_admission(|admission| try_bound_names(declaration, admission))?
            {
                self.control.admit(CompilerWork::MetadataVisit)?;
                // 1. If declaredFunctionNames does not contain vn, then
                if !contains_name(&declared_function_names, name, &mut self.control)? {
                    // a. Let vnDefinable be ? env.CanDeclareGlobalVar(vn).
                    // b. If vnDefinable is false, throw a TypeError exception.
                    // Done in `Context::global_declaration_instantiation`
                    // The names checked here are the same names from the functions
                    // in `declared_var_names`, so we can reuse `global_vars`
                    // for this check.

                    // c. If declaredVarNames does not contain vn, then
                    if !contains_name(&declared_var_names, name, &mut self.control)? {
                        // i. Append vn to declaredVarNames.
                        retain_declaration(&mut declared_var_names, name, &mut self.control)?;
                    }
                }
            }
        }

        // 11. NOTE: No abnormal terminations occur after this algorithm step if the
        //     global object is an ordinary object. However, if the global object is
        //     a Proxy exotic object it may exhibit behaviours that cause abnormal
        //     terminations in some of the following steps.

        // Steps 13-15 are covered by the scope analyzer.

        // 16. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. Let fn be the sole element of the BoundNames of f.
            let (name, generator, r#async, parameters, body, scopes, contains_direct_eval) =
                match function {
                    VarScopedDeclaration::FunctionDeclaration(f) => (
                        f.name(),
                        false,
                        false,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::GeneratorDeclaration(f) => (
                        f.name(),
                        true,
                        false,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::AsyncFunctionDeclaration(f) => (
                        f.name(),
                        false,
                        true,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::AsyncGeneratorDeclaration(f) => (
                        f.name(),
                        true,
                        true,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::VariableDeclaration(_) => continue,
                };

            let func_span = function.linear_span();
            self.control.admit(CompilerWork::Copy {
                bytes: std::mem::size_of::<crate::spanned_source_text::SourceText>(),
            })?;
            self.control.admit(CompilerWork::MetadataWrite)?;
            let spanned_source_text = SpannedSourceText::new(self.source_text(), func_span);
            let function_name = name
                .sym()
                .try_to_js_string(&*self.interner, &mut self.control)?;
            self.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<ScopeHandle>(2)?,
            })?;
            let variable_scope = self.variable_scope.clone();
            let lexical_scope = self.lexical_scope.clone();

            self.control.admit(CompilerWork::Copy {
                bytes: std::mem::size_of::<crate::vm::SourcePath>(),
            })?;
            let source_path = self.source_path.clone();
            let code = FunctionCompiler::new(spanned_source_text)
                .name(function_name)
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .in_with(self.in_with)
                .source_path(source_path)
                .compile(
                    parameters,
                    body,
                    variable_scope,
                    lexical_scope,
                    scopes,
                    contains_direct_eval,
                    self.interner,
                    &mut *self.scopes,
                    self.control.reborrow(),
                )?;

            // Ensures global functions are printed when generating the global flowgraph.
            let name_index = self.get_or_insert_name(name.sym())?;
            let function_index = self.push_function_to_constants(code)?;

            // b. Let fo be InstantiateFunctionObject of f with arguments env and privateEnv.
            // c. Perform ? env.CreateGlobalFunctionBinding(fn, fo, false).
            // Done in `Context::global_declaration_instantiation`
            retain_declaration(
                &mut self.global_fns,
                GlobalFunctionBinding {
                    name_index,
                    function_index,
                },
                &mut self.control,
            )?;
        }

        // 17 is done in `Context::global_declaration_instantiation

        // 17. For each String vn of declaredVarNames, do
        for var in declared_var_names {
            self.control.admit(CompilerWork::MetadataVisit)?;
            let index = self.get_or_insert_name(var)?;
            retain_declaration(&mut self.global_vars, index, &mut self.control)?;

            // a. Perform ? env.CreateGlobalVarBinding(vn, false).
            // Done in `Context::global_declaration_instantiation`
        }

        // 18. Return unused.
        Ok(())
    }

    /// `BlockDeclarationInstantiation ( code, env )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-blockdeclarationinstantiation
    pub(crate) fn block_declaration_instantiation<'a, N>(&mut self, block: &'a N) -> JsResult<()>
    where
        &'a N: Into<NodeRef<'a>>,
    {
        // 1. Let declarations be the LexicallyScopedDeclarations of code.
        let declarations = self
            .control
            .with_name_admission(|admission| try_lexically_scoped_declarations(block, admission))?;

        // Note: Not sure if the spec is wrong here or if our implementation just differs too much,
        //       but we need 3.a to be finished for all declarations before 3.b can be done.

        // b. If d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration, then
        //     i. Let fn be the sole element of the BoundNames of d.
        //     ii. Let fo be InstantiateFunctionObject of d with arguments env and privateEnv.
        //     iii. Perform ! env.InitializeBinding(fn, fo). NOTE: This step is replaced in section B.3.2.6.
        // TODO: Support B.3.2.6.
        for d in declarations {
            self.control.admit(CompilerWork::MetadataVisit)?;
            match d {
                LexicallyScopedDeclaration::FunctionDeclaration(function) => {
                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.function_with_binding(function.into(), NodeKind::Declaration, &dst)?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                LexicallyScopedDeclaration::GeneratorDeclaration(function) => {
                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.function_with_binding(function.into(), NodeKind::Declaration, &dst)?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                LexicallyScopedDeclaration::AsyncFunctionDeclaration(function) => {
                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.function_with_binding(function.into(), NodeKind::Declaration, &dst)?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                LexicallyScopedDeclaration::AsyncGeneratorDeclaration(function) => {
                    let dst = self.register_allocator.alloc(&mut self.control)?;
                    self.function_with_binding(function.into(), NodeKind::Declaration, &dst)?;
                    self.register_allocator.dealloc(dst, &mut self.control)?;
                }
                _ => {}
            }
        }

        // 4. Return unused.
        Ok(())
    }

    /// `EvalDeclarationInstantiation ( body, varEnv, lexEnv, privateEnv, strict )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-evaldeclarationinstantiation
    pub(crate) fn eval_declaration_instantiation(
        &mut self,
        body: &Script,
        #[allow(
            unused_variables,
            reason = "only used when the `annex-b` feature is enabled"
        )]
        strict: bool,
        var_env: &ScopeHandle,
        bindings: &EvalDeclarationBindings,
    ) -> JsResult<()> {
        // 2. Let varDeclarations be the VarScopedDeclarations of body.
        let var_declarations = self
            .control
            .with_name_admission(|admission| try_var_scoped_declarations(body, admission))?;

        // SKIP: 3. If strict is false, then
        // covered by the scope analyzer.

        // NOTE: These steps depend on the current environment state are done before bytecode compilation,
        //       in `eval_declaration_instantiation_context`.
        //
        // SKIP: 4. Let privateIdentifiers be a new empty List.
        // SKIP: 5. Let pointer be privateEnv.
        // SKIP: 6. Repeat, while pointer is not null,
        //           a. For each Private Name binding of pointer.[[Names]], do
        //               i. If privateIdentifiers does not contain binding.[[Description]],
        //                  append binding.[[Description]] to privateIdentifiers.
        //           b. Set pointer to pointer.[[OuterPrivateEnvironment]].
        // SKIP: 7. If AllPrivateIdentifiersValid of body with argument privateIdentifiers is false, throw a SyntaxError exception.

        // 8. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 9. Let declaredFunctionNames be a new empty List.
        let mut declared_function_names = Vec::new();

        // 10. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. If d is not either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            // a.ii. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
            // a.iii. Let fn be the sole element of the BoundNames of d.
            let name = match *declaration {
                VarScopedDeclaration::FunctionDeclaration(f) => f.name(),
                VarScopedDeclaration::GeneratorDeclaration(f) => f.name(),
                VarScopedDeclaration::AsyncFunctionDeclaration(f) => f.name(),
                VarScopedDeclaration::AsyncGeneratorDeclaration(f) => f.name(),
                VarScopedDeclaration::VariableDeclaration(_) => continue,
            };
            // a.iv. If declaredFunctionNames does not contain fn, then
            if !contains_name(&declared_function_names, name.sym(), &mut self.control)? {
                // 1. If varEnv is a Global Environment Record, then
                //    a. Let fnDefinable be ? varEnv.CanDeclareGlobalFunction(fn).
                //    b. If fnDefinable is false, throw a TypeError exception.
                //    Done in `Context::eval_declaration_instantiation`
                //    The names checked here are the same names from the functions
                //    in `functions_to_initialize`, but in reverse order, so we can
                //    reuse `global_fns` for this check.

                // 2. Append fn to declaredFunctionNames.
                retain_declaration(&mut declared_function_names, name.sym(), &mut self.control)?;

                // 3. Insert d as the first element of functionsToInitialize.
                retain_declaration(
                    &mut functions_to_initialize,
                    *declaration,
                    &mut self.control,
                )?;
            }
        }

        reverse_declarations(&mut functions_to_initialize, &mut self.control)?;

        // 11. NOTE: Annex B.3.2.3 adds additional steps at this point.
        // 11. If strict is false, then
        #[cfg(feature = "annex-b")]
        if !strict {
            // NOTE: This diviates from the specification, we split the first part of defining the annex-b names
            //       in `eval_declaration_instantiation_context`, because it depends on the context.
            if !self.with_scopes(|access| Ok(access.is_global(var_env)?))? {
                for binding in &bindings.new_annex_b_function_names {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    // i. Let bindingExists be ! varEnv.HasBinding(F).
                    // ii. If bindingExists is false, then
                    // i. Perform ! varEnv.CreateMutableBinding(F, true).
                    // ii. Perform ! varEnv.InitializeBinding(F, undefined).

                    use crate::vm::CallFrame;
                    self.control.admit(CompilerWork::Copy {
                        bytes: std::mem::size_of::<surgeist_js_ast::scope::IdentifierReference>(),
                    })?;
                    let index = self.insert_binding(binding.clone())?;
                    self.emit_binding_access(
                        BindingAccessOpcode::DefInitVar,
                        &index,
                        &CallFrame::undefined_register(),
                    )?;
                }
            }
        }

        // 12. Let declaredVarNames be a new empty List.
        let mut declared_var_names = Vec::new();

        // 13. For each element d of varDeclarations, do
        for declaration in var_declarations {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. If d is either a VariableDeclaration, a ForBinding, or a BindingIdentifier, then
            let VarScopedDeclaration::VariableDeclaration(declaration) = declaration else {
                continue;
            };

            // a.i. For each String vn of the BoundNames of d, do
            for name in self
                .control
                .with_name_admission(|admission| try_bound_names(declaration, admission))?
            {
                self.control.admit(CompilerWork::MetadataVisit)?;
                // 1. If declaredFunctionNames does not contain vn, then
                if !contains_name(&declared_function_names, name, &mut self.control)? {
                    // a. If varEnv is a Global Environment Record, then
                    //    i. Let vnDefinable be ? varEnv.CanDeclareGlobalVar(vn).
                    //    ii. If vnDefinable is false, throw a TypeError exception.
                    //    Done in `Context::eval_declaration_instantiation`
                    //    The names checked here are the same names from the functions
                    //    in `declared_var_names`, so we can reuse `global_vars`
                    //    for this check.

                    // b. If declaredVarNames does not contain vn, then
                    if !contains_name(&declared_var_names, name, &mut self.control)? {
                        // i. Append vn to declaredVarNames.
                        retain_declaration(&mut declared_var_names, name, &mut self.control)?;
                    }
                }
            }
        }

        // 14. NOTE: No abnormal terminations occur after this algorithm step unless varEnv is a
        //           Global Environment Record and the global object is a Proxy exotic object.

        // 15. Let lexDeclarations be the LexicallyScopedDeclarations of body.
        // 16. For each element d of lexDeclarations, do

        let mut function_bindings = CompilerIndex::default();
        for (name, binding) in &bindings.new_function_names {
            self.control.admit(CompilerWork::MetadataVisit)?;
            let _ = function_bindings.insert(*name, binding, &mut self.control)?;
        }

        // 17. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. Let fn be the sole element of the BoundNames of f.
            let (name, generator, r#async, parameters, body, scopes, contains_direct_eval) =
                match function {
                    VarScopedDeclaration::FunctionDeclaration(f) => (
                        f.name(),
                        false,
                        false,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::GeneratorDeclaration(f) => (
                        f.name(),
                        true,
                        false,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::AsyncFunctionDeclaration(f) => (
                        f.name(),
                        false,
                        true,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::AsyncGeneratorDeclaration(f) => (
                        f.name(),
                        true,
                        true,
                        f.parameters(),
                        f.body(),
                        f.scopes(),
                        f.contains_direct_eval(),
                    ),
                    VarScopedDeclaration::VariableDeclaration(_) => {
                        continue;
                    }
                };

            let func_span = function.linear_span();
            self.control.admit(CompilerWork::Copy {
                bytes: std::mem::size_of::<crate::spanned_source_text::SourceText>(),
            })?;
            self.control.admit(CompilerWork::MetadataWrite)?;
            let spanned_source_text = SpannedSourceText::new(self.source_text(), func_span);
            let function_name = name
                .sym()
                .try_to_js_string(&*self.interner, &mut self.control)?;
            self.control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<ScopeHandle>(2)?,
            })?;
            let variable_scope = self.variable_scope.clone();
            let lexical_scope = self.lexical_scope.clone();

            let code = FunctionCompiler::new(spanned_source_text)
                .name(function_name)
                .generator(generator)
                .r#async(r#async)
                .strict(self.strict())
                .in_with(self.in_with)
                .name_scope(None)
                .compile(
                    parameters,
                    body,
                    variable_scope,
                    lexical_scope,
                    scopes,
                    contains_direct_eval,
                    self.interner,
                    &mut *self.scopes,
                    self.control.reborrow(),
                )?;

            // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
            let index = self.push_function_to_constants(code)?;

            // c. If varEnv is a Global Environment Record, then
            if self.with_scopes(|access| Ok(access.is_global(var_env)?))? {
                // i. Perform ? varEnv.CreateGlobalFunctionBinding(fn, fo, true).
                // Done in `Context::eval_declaration_instantiation`
                let name_index = self.get_or_insert_name(name.sym())?;
                retain_declaration(
                    &mut self.global_fns,
                    GlobalFunctionBinding {
                        name_index,
                        function_index: index,
                    },
                    &mut self.control,
                )?;
            }
            // d. Else,
            else {
                let dst = self.register_allocator.alloc(&mut self.control)?;
                self.emit_get_function(&dst, index)?;

                // i. Let bindingExists be ! varEnv.HasBinding(fn).
                let binding = function_bindings
                    .get(&name, &mut self.control)?
                    .expect("binding must exist");
                self.control.admit(CompilerWork::Copy {
                    bytes: std::mem::size_of::<&surgeist_js_ast::scope::IdentifierReference>(),
                })?;
                let binding = *binding;

                // ii. If bindingExists is false, then
                // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
                // 2. Perform ! varEnv.CreateMutableBinding(fn, true).
                // 3. Perform ! varEnv.InitializeBinding(fn, fo).
                // iii. Else,
                // 1. Perform ! varEnv.SetMutableBinding(fn, fo, false).
                self.control.admit(CompilerWork::Copy {
                    bytes: std::mem::size_of::<surgeist_js_ast::scope::IdentifierReference>(),
                })?;
                let index = self.insert_binding(binding.clone())?;
                self.emit_binding_access(BindingAccessOpcode::DefInitVar, &index, &dst)?;
                self.register_allocator.dealloc(dst, &mut self.control)?;
            }
        }

        // 18. For each String vn of declaredVarNames, do
        for name in declared_var_names {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. If varEnv is a Global Environment Record, then
            if self.with_scopes(|access| Ok(access.is_global(var_env)?))? {
                let index = self.get_or_insert_name(name)?;

                // i. Perform ? varEnv.CreateGlobalVarBinding(vn, true).
                // Done in `Context::eval_declaration_instantiation`
                retain_declaration(&mut self.global_vars, index, &mut self.control)?;
            }
        }
        // 18.b
        for binding in &bindings.declared_var_names {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // i. Let bindingExists be ! varEnv.HasBinding(vn).
            // ii. If bindingExists is false, then
            // 1. NOTE: The following invocation cannot return an abrupt completion because of the validation preceding step 14.
            // 2. Perform ! varEnv.CreateMutableBinding(vn, true).
            // 3. Perform ! varEnv.InitializeBinding(vn, undefined).
            self.control.admit(CompilerWork::Copy {
                bytes: std::mem::size_of::<surgeist_js_ast::scope::IdentifierReference>(),
            })?;
            let index = self.insert_binding(binding.clone())?;
            self.emit_binding_access(
                BindingAccessOpcode::DefEvalVar,
                &index,
                &CallFrame::undefined_register(),
            )?;
        }

        // 19. Return unused.
        Ok(())
    }

    /// `FunctionDeclarationInstantiation ( func, argumentsList )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-functiondeclarationinstantiation
    pub(crate) fn function_declaration_instantiation(
        &mut self,
        body: &FunctionBody,
        formals: &FormalParameterList,
        arrow: bool,
        strict: bool,
        generator: bool,
        scopes: &FunctionScopes,
    ) -> JsResult<()> {
        // 1. Let calleeContext be the running execution context.
        // 2. Let code be func.[[ECMAScriptCode]].
        // 3. Let strict be func.[[Strict]].
        // 4. Let formals be func.[[FormalParameters]].

        // 5. Let parameterNames be the BoundNames of formals.
        let mut parameter_names = self
            .control
            .with_name_admission(|admission| try_bound_names(formals, admission))?;

        // 6. If parameterNames has any duplicate entries, let hasDuplicates be true. Otherwise, let hasDuplicates be false.
        // let has_duplicates = formals.has_duplicates();

        // 7. Let simpleParameterList be IsSimpleParameterList of formals.
        // let simple_parameter_list = formals.is_simple();

        // 8. Let hasParameterExpressions be ContainsExpression of formals.
        let has_parameter_expressions = formals.has_expressions();

        // 9. Let varNames be the VarDeclaredNames of code.
        // Preserve the AST-owned set's native order; the selected environment
        // branch requests its opaque iteration extent before yielding candidates.
        let var_names = self
            .control
            .with_name_admission(|admission| try_var_declared_names(body, admission))?;

        // 10. Let varDeclarations be the VarScopedDeclarations of code.
        let var_declarations = self
            .control
            .with_name_admission(|admission| try_var_scoped_declarations(body, admission))?;

        // 11. Let lexicalNames be the LexicallyDeclaredNames of code.
        let lexical_names = self
            .control
            .with_name_admission(|admission| try_lexically_declared_names(body, admission))?;

        // 12. Let functionNames be a new empty List.
        let mut function_names = Vec::new();

        // 13. Let functionsToInitialize be a new empty List.
        let mut functions_to_initialize = Vec::new();

        // 14. For each element d of varDeclarations, in reverse List order, do
        for declaration in var_declarations.iter().rev() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. If d is neither a VariableDeclaration nor a ForBinding nor a BindingIdentifier, then
            // a.i. Assert: d is either a FunctionDeclaration, a GeneratorDeclaration, an AsyncFunctionDeclaration, or an AsyncGeneratorDeclaration.
            // a.ii. Let fn be the sole element of the BoundNames of d.
            let (name, function) = match *declaration {
                VarScopedDeclaration::FunctionDeclaration(f) => (f.name(), FunctionSpec::from(f)),
                VarScopedDeclaration::GeneratorDeclaration(f) => (f.name(), FunctionSpec::from(f)),
                VarScopedDeclaration::AsyncFunctionDeclaration(f) => {
                    (f.name(), FunctionSpec::from(f))
                }
                VarScopedDeclaration::AsyncGeneratorDeclaration(f) => {
                    (f.name(), FunctionSpec::from(f))
                }
                VarScopedDeclaration::VariableDeclaration(_) => continue,
            };

            // a.iii. If functionNames does not contain fn, then
            if !contains_name(&function_names, name.sym(), &mut self.control)? {
                // 1. Insert fn as the first element of functionNames.
                retain_declaration(&mut function_names, name.sym(), &mut self.control)?;

                // 2. NOTE: If there are multiple function declarations for the same name, the last declaration is used.
                // 3. Insert d as the first element of functionsToInitialize.
                retain_declaration(&mut functions_to_initialize, function, &mut self.control)?;
            }
        }

        reverse_declarations(&mut function_names, &mut self.control)?;
        reverse_declarations(&mut functions_to_initialize, &mut self.control)?;

        // 15. Let argumentsObjectNeeded be true.
        let mut arguments_object_needed = true;

        let arguments = Sym::ARGUMENTS;

        // 16. If func.[[ThisMode]] is lexical, then
        // 17. Else if parameterNames contains "arguments", then
        if arrow || contains_name(&parameter_names, arguments, &mut self.control)? {
            // 16.a. NOTE: Arrow functions never have an arguments object.
            // 16.b. Set argumentsObjectNeeded to false.
            // 17.a. Set argumentsObjectNeeded to false.
            arguments_object_needed = false;
        }
        // 18. Else if hasParameterExpressions is false, then
        else if !has_parameter_expressions {
            //a. If functionNames contains "arguments" or lexicalNames contains "arguments", then
            if contains_name(&function_names, arguments, &mut self.control)?
                || contains_name(&lexical_names, arguments, &mut self.control)?
            {
                // i. Set argumentsObjectNeeded to false.
                arguments_object_needed = false;
            }
        }

        if arguments_object_needed {
            arguments_object_needed = self.scope_arguments_object_accessed(scopes)?;
        }

        // 19-20
        drop(self.push_declarative_scope(scopes.parameters_eval_scope())?);

        self.control.admit(CompilerWork::Copy {
            bytes: std::mem::size_of::<ScopeHandle>(),
        })?;
        let scope = self.lexical_scope.clone();

        // 22. If argumentsObjectNeeded is true, then
        //
        // NOTE(HalidOdat): Has been moved up, so "arguments" gets registered as
        //     the first binding in the environment with index 0.
        if arguments_object_needed {
            let arguments = arguments.try_to_js_string(&*self.interner, &mut self.control)?;

            // a. If strict is true or simpleParameterList is false, then
            let value = self.register_allocator.alloc(&mut self.control)?;
            if strict || !formals.is_simple() {
                // i. Let ao be CreateUnmappedArgumentsObject(argumentsList).
                self.bytecode
                    .emit_create_unmapped_arguments_object(value.variable(), &mut self.control)?;
            }
            // b. Else,
            else {
                // i. NOTE: A mapped argument object is only provided for non-strict functions
                //          that don't have a rest parameter, any parameter
                //          default value initializers, or any destructured parameters.
                // ii. Let ao be CreateMappedArgumentsObject(func, formals, argumentsList, env).
                self.bytecode
                    .emit_create_mapped_arguments_object(value.variable(), &mut self.control)?;
                self.control.admit(CompilerWork::MetadataWrite)?;
                self.emitted_mapped_arguments_object_opcode = true;
            }

            // e. Perform ! env.InitializeBinding("arguments", ao).
            self.emit_binding(BindingOpcode::InitLexical, arguments, &value)?;
            self.register_allocator.dealloc(value, &mut self.control)?;
        }

        // 22. If argumentsObjectNeeded is true, then
        if arguments_object_needed {
            // MOVED: a-e.
            //
            // NOTE(HalidOdat): Has been moved up, see comment above.

            // f. Let parameterBindings be the list-concatenation of parameterNames and « "arguments" ».
            retain_declaration(&mut parameter_names, arguments, &mut self.control)?;
        }

        // 23. Else,
        //     a. Let parameterBindings be parameterNames.
        let parameter_bindings = copy_names(&parameter_names, &mut self.control)?;

        // 24. Let iteratorRecord be CreateListIteratorRecord(argumentsList).
        // 25. If hasDuplicates is true, then
        //    a. Perform ? IteratorBindingInitialization of formals with arguments iteratorRecord and undefined.
        // 26. Else,
        //    a. Perform ? IteratorBindingInitialization of formals with arguments iteratorRecord and env.
        for (i, parameter) in formals.as_ref().iter().enumerate() {
            self.control.admit(CompilerWork::MetadataVisit)?;
            let value = self.register_allocator.alloc(&mut self.control)?;
            if parameter.is_rest_param() {
                self.bytecode
                    .emit_rest_parameter_init(value.variable(), &mut self.control)?;
            } else {
                let argument_index = u32::try_from(i).map_err(|_| size_overflow())?;
                self.bytecode.emit_get_argument(
                    argument_index.into(),
                    value.variable(),
                    &mut self.control,
                )?;
            }

            match parameter.variable().binding() {
                Binding::Identifier(ident) => {
                    let ident = ident.try_to_js_string(&*self.interner, &mut self.control)?;
                    self.control.admit(CompilerWork::Visit)?;
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.jump_if_not_undefined(&value)?;
                        self.compile_expr(init, &value)?;
                        self.patch_jump(skip)?;
                    }
                    self.emit_binding(BindingOpcode::InitLexical, ident, &value)?;
                }
                Binding::Pattern(pattern) => {
                    self.control.admit(CompilerWork::Visit)?;
                    if let Some(init) = parameter.variable().init() {
                        let skip = self.jump_if_not_undefined(&value)?;
                        self.compile_expr(init, &value)?;
                        self.patch_jump(skip)?;
                    }
                    self.compile_declaration_pattern(pattern, BindingOpcode::InitLexical, &value)?;
                }
            }
            self.register_allocator.dealloc(value, &mut self.control)?;
        }

        if generator {
            if self.is_async() {
                self.bytecode.emit_async_generator(&mut self.control)?;
            } else {
                self.bytecode.emit_generator(&mut self.control)?;
            }
            self.bytecode.emit_pop(&mut self.control)?;
        }

        // 27. If hasParameterExpressions is false, then
        // 28. Else,
        #[allow(unused_variables, unused_mut)]
        let (mut instantiated_var_names, mut variable_scope) =
            if let Some(scope) = scopes.parameters_scope() {
                // a. NOTE: A separate Environment Record is needed to ensure that closures created by
                //          expressions in the formal parameter list do not have
                //          visibility of declarations in the function body.
                // b. Let varEnv be NewDeclarativeEnvironment(env).
                // c. Set the VariableEnvironment of calleeContext to varEnv.
                drop(self.push_declarative_scope(Some(scope))?);

                self.control.admit(CompilerWork::Copy {
                    bytes: std::mem::size_of::<ScopeHandle>(),
                })?;
                let mut variable_scope = self.lexical_scope.clone();

                // d. Let instantiatedVarNames be a new empty List.
                let mut instantiated_var_names = Vec::new();

                // e. For each element n of varNames, do
                self.control.admit(CompilerWork::NameSetIteration {
                    capacity: var_names.capacity(),
                })?;
                for n in var_names {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    // i. If instantiatedVarNames does not contain n, then
                    if !contains_name(&instantiated_var_names, n, &mut self.control)? {
                        // 1. Append n to instantiatedVarNames.
                        retain_declaration(&mut instantiated_var_names, n, &mut self.control)?;

                        let n_string = n.try_to_js_string(&*self.interner, &mut self.control)?;

                        // 2. Perform ! varEnv.CreateMutableBinding(n, false).
                        let binding = self
                            .scope_binding_reference(&variable_scope, &n_string)?
                            .expect("must have binding");

                        let value = self.register_allocator.alloc(&mut self.control)?;
                        // 3. If parameterBindings does not contain n, or if functionNames contains n, then
                        if !contains_name(&parameter_bindings, n, &mut self.control)?
                            || contains_name(&function_names, n, &mut self.control)?
                        {
                            // a. Let initialValue be undefined.
                            self.bytecode
                                .emit_store_undefined(value.variable(), &mut self.control)?;
                        }
                        // 4. Else,
                        else {
                            // a. Let initialValue be ! env.GetBindingValue(n, false).
                            let binding = self
                                .scope_binding_reference(scope, &n_string)?
                                .expect("must have binding");
                            let index = self.get_binding(&binding)?;
                            self.emit_binding_access(BindingAccessOpcode::GetName, &index, &value)?;
                        }

                        // 5. Perform ! varEnv.InitializeBinding(n, initialValue).
                        let index = self.insert_binding(binding)?;

                        // TODO: What?
                        self.bytecode
                            .emit_store_undefined(value.variable(), &mut self.control)?;
                        self.emit_binding_access(BindingAccessOpcode::DefInitVar, &index, &value)?;
                        self.register_allocator.dealloc(value, &mut self.control)?;

                        // 6. NOTE: A var with the same name as a formal parameter initially has
                        //          the same value as the corresponding initialized parameter.
                    }
                }

                (instantiated_var_names, variable_scope)
            } else {
                // a. NOTE: Only a single Environment Record is needed for the parameters and top-level vars.
                // b. Let instantiatedVarNames be a copy of the List parameterBindings.
                let mut instantiated_var_names = parameter_bindings;

                // c. For each element n of varNames, do
                self.control.admit(CompilerWork::NameSetIteration {
                    capacity: var_names.capacity(),
                })?;
                for n in var_names {
                    self.control.admit(CompilerWork::MetadataVisit)?;
                    // i. If instantiatedVarNames does not contain n, then
                    if !contains_name(&instantiated_var_names, n, &mut self.control)? {
                        // 1. Append n to instantiatedVarNames.
                        retain_declaration(&mut instantiated_var_names, n, &mut self.control)?;

                        let n = n.try_to_js_string(&*self.interner, &mut self.control)?;

                        // 2. Perform ! env.CreateMutableBinding(n, false).
                        // 3. Perform ! env.InitializeBinding(n, undefined).
                        let binding = self
                            .scope_binding_reference(&scope, &n)?
                            .expect("binding must exist");
                        let index = self.insert_binding(binding)?;
                        self.emit_binding_access(
                            BindingAccessOpcode::DefInitVar,
                            &index,
                            &CallFrame::undefined_register(),
                        )?;
                    }
                }

                // d. Let varEnv be env.
                (instantiated_var_names, scope)
            };

        // 29. NOTE: Annex B.3.2.1 adds additional steps at this point.
        // 29. If strict is false, then
        #[cfg(feature = "annex-b")]
        if !strict {
            // a. For each FunctionDeclaration f that is directly contained in the StatementList
            //    of a Block, CaseClause, or DefaultClause, do
            for f in self.control.with_name_admission(|admission| {
                try_annex_b_function_declarations_names(body, admission)
            })? {
                self.control.admit(CompilerWork::MetadataVisit)?;
                // i. Let F be StringValue of the BindingIdentifier of f.
                // ii. If replacing the FunctionDeclaration f with a VariableStatement that has F
                //     as a BindingIdentifier would not produce any Early Errors
                //     for func and parameterNames does not contain F, then
                if !contains_name(&lexical_names, f, &mut self.control)?
                    && !contains_name(&parameter_names, f, &mut self.control)?
                {
                    // 1. NOTE: A var binding for F is only instantiated here if it is neither a
                    //    VarDeclaredName, the name of a formal parameter, or another FunctionDeclaration.

                    // 2. If initializedBindings does not contain F and F is not "arguments", then
                    if !contains_name(&instantiated_var_names, f, &mut self.control)? && {
                        self.control.admit(CompilerWork::Search)?;
                        f != arguments
                    } {
                        let f_string = f.try_to_js_string(&*self.interner, &mut self.control)?;

                        // a. Perform ! varEnv.CreateMutableBinding(F, false).
                        // b. Perform ! varEnv.InitializeBinding(F, undefined).
                        let binding = self
                            .scope_binding_reference(&variable_scope, &f_string)?
                            .expect("binding must exist");
                        let index = self.insert_binding(binding)?;
                        self.emit_binding_access(
                            BindingAccessOpcode::DefInitVar,
                            &index,
                            &CallFrame::undefined_register(),
                        )?;

                        // c. Append F to instantiatedVarNames.
                        retain_declaration(&mut instantiated_var_names, f, &mut self.control)?;
                    }

                    // 3. When the FunctionDeclaration f is evaluated, perform the following steps
                    //    in place of the FunctionDeclaration Evaluation algorithm provided in 15.2.6:
                    //     a. Let fenv be the running execution context's VariableEnvironment.
                    //     b. Let benv be the running execution context's LexicalEnvironment.
                    //     c. Let fobj be ! benv.GetBindingValue(F, false).
                    //     d. Perform ! fenv.SetMutableBinding(F, fobj, false).
                    //     e. Return unused.
                    retain_declaration(&mut self.annex_b_function_names, f, &mut self.control)?;
                }
            }
        }

        // 30-31
        drop(self.push_declarative_scope(scopes.lexical_scope())?);

        // 35. Let privateEnv be the PrivateEnvironment of calleeContext.
        // 36. For each Parse Node f of functionsToInitialize, do
        for function in functions_to_initialize {
            self.control.admit(CompilerWork::MetadataVisit)?;
            // a. Let fn be the sole element of the BoundNames of f.
            // b. Let fo be InstantiateFunctionObject of f with arguments lexEnv and privateEnv.
            // c. Perform ! varEnv.SetMutableBinding(fn, fo, false).
            let dst = self.register_allocator.alloc(&mut self.control)?;
            self.function_with_binding(function, NodeKind::Declaration, &dst)?;
            self.register_allocator.dealloc(dst, &mut self.control)?;
        }

        // 37. Return unused.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsError, JsValue, bytecompiler::control::CompilerDepth};

    #[test]
    fn declaration_name_search_admits_each_candidate_through_the_match_or_miss() {
        let names = [Sym::EVAL, Sym::ARGUMENTS, Sym::LET];
        for (name, expected, visits) in [
            (Sym::EVAL, true, 1),
            (Sym::ARGUMENTS, true, 2),
            (Sym::NULL, false, 3),
        ] {
            let mut inspected = 0;
            let mut admission = |work| {
                assert!(matches!(work, CompilerWork::Search));
                inspected += 1;
                Ok(())
            };
            let depth = CompilerDepth::new();
            assert_eq!(
                contains_name(
                    &names,
                    name,
                    &mut CompilerControl::new(Some(&mut admission), &depth)
                )
                .expect("search"),
                expected
            );
            assert_eq!(inspected, visits);
        }
    }

    #[test]
    fn declaration_name_search_refuses_before_the_next_candidate_without_wrapping_error() {
        let names = [Sym::EVAL, Sym::ARGUMENTS, Sym::LET];
        let marker = JsError::from_opaque(JsValue::new(315)).unwrap();
        let mut attempted = 0;
        let mut stopped = false;
        let mut admission = |_| {
            assert!(!stopped, "search continued after refusal");
            attempted += 1;
            if attempted == 2 {
                stopped = true;
                Err(marker.clone())
            } else {
                Ok(())
            }
        };
        let depth = CompilerDepth::new();
        assert_eq!(
            contains_name(
                &names,
                Sym::NULL,
                &mut CompilerControl::new(Some(&mut admission), &depth)
            )
            .expect_err("second candidate refused"),
            marker
        );
        assert_eq!(attempted, 2);
    }

    #[test]
    fn declaration_reversal_admits_swaps_and_preserves_only_prior_private_progress_on_stop() {
        let depth = CompilerDepth::new();
        let mut exact = [1_u32, 2, 3, 4, 5];
        let mut requests = 0;
        let mut count = |_| {
            requests += 1;
            Ok(())
        };
        reverse_declarations(
            &mut exact,
            &mut CompilerControl::new(Some(&mut count), &depth),
        )
        .expect("all swaps");
        assert_eq!(exact, [5, 4, 3, 2, 1]);
        assert_eq!(requests, 10); // Two visits, one payload transfer and two writes per swap.

        let marker = JsError::from_opaque(JsValue::new(316)).unwrap();
        let mut partial = [1_u32, 2, 3, 4, 5];
        let mut accepted = 0;
        let mut stopped = false;
        let mut one_short = |_| {
            assert!(!stopped);
            if accepted == requests - 1 {
                stopped = true;
                Err(marker.clone())
            } else {
                accepted += 1;
                Ok(())
            }
        };
        assert_eq!(
            reverse_declarations(
                &mut partial,
                &mut CompilerControl::new(Some(&mut one_short), &depth)
            )
            .expect_err("last write refused"),
            marker
        );
        assert_eq!(partial, [5, 2, 3, 4, 1]);
        assert_eq!(accepted, requests - 1);
    }

    #[test]
    fn copied_parameter_names_preserve_duplicates_and_stop_at_the_exact_request() {
        let names = [Sym::EVAL, Sym::ARGUMENTS, Sym::EVAL];
        let depth = CompilerDepth::new();
        let mut requests = 0;
        let mut count = |_| {
            requests += 1;
            Ok(())
        };
        assert_eq!(
            copy_names(&names, &mut CompilerControl::new(Some(&mut count), &depth)).expect("copy"),
            names
        );
        for limit in 0..=requests {
            let marker = JsError::from_opaque(JsValue::new(317)).unwrap();
            let mut accepted = 0;
            let mut stopped = false;
            let mut admission = |_| {
                assert!(!stopped);
                if accepted == limit {
                    stopped = true;
                    Err(marker.clone())
                } else {
                    accepted += 1;
                    Ok(())
                }
            };
            let result = copy_names(
                &names,
                &mut CompilerControl::new(Some(&mut admission), &depth),
            );
            if limit == requests {
                assert_eq!(result.expect("exact"), names);
            } else {
                assert_eq!(result.expect_err("before next copy"), marker);
            }
            assert_eq!(accepted, limit);
        }
    }
}

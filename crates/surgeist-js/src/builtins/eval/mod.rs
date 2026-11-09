//! Boa's implementation of ECMAScript's global `eval` function.
//!
//! The `eval()` function evaluates ECMAScript code represented as a string.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-eval-x
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval

use std::rc::Rc;

use crate::{
    Context, JsArgs, JsResult, JsString, JsValue, SpannedSourceText,
    builtins::{BuiltInObject, function::OrdinaryFunction},
    bytecompiler::{
        ByteCompiler,
        control::{CompilerWork, payload_bytes, size_overflow},
    },
    context::intrinsics::Intrinsics,
    environments::{EnvironmentStack, EvalBindingCounts, PreparedEvalEnvironment},
    error::{JsNativeError, RuntimeLimitError},
    js_string,
    object::JsObject,
    realm::Realm,
    spanned_source_text::SourceText,
    string::{JsStrVariant, StaticJsStrings},
    vm::{ActivatedCode, CallFrame, CallFrameFlags, source_info::SourcePath},
};
#[cfg(feature = "annex-b")]
use declarations::prepare_eval_declaration_instantiation;
use surgeist_js_ast::{operations::ContainsSymbol, scope::Scope};
use surgeist_js_interner::{Interner, Sym};
use surgeist_js_parser::{Source, frontend::FrontendWork, source::ReadChar};

use super::{BuiltInBuilder, IntrinsicObject};

#[cfg(feature = "annex-b")]
mod declarations;

#[cfg(test)]
mod tests;

fn project_private_names<E>(
    environments: &EnvironmentStack,
    interner: &Interner,
    mut admit: impl FnMut(FrontendWork) -> Result<(), E>,
) -> Result<Vec<Sym>, E> {
    let mut names = Vec::new();
    for environment in environments.private_environments() {
        admit(FrontendWork::NameVisit)?;
        'description: for description in environment.descriptions() {
            admit(FrontendWork::NameVisit)?;
            let lookup = |work| admit(FrontendWork::Interner(work));
            let symbol = match description.as_str().variant() {
                JsStrVariant::Latin1(bytes) => interner.try_get_latin1(bytes, lookup)?,
                JsStrVariant::Utf16(units) => interner.try_get(units, lookup)?,
            };
            // Names absent from this parse cannot occur in its AST.
            let Some(symbol) = symbol else {
                continue;
            };
            for &existing in &names {
                admit(FrontendWork::NameComparison)?;
                if existing == symbol {
                    continue 'description;
                }
            }
            admit(FrontendWork::NameRecord)?;
            names.push(symbol);
        }
    }
    Ok(names)
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Eval;

impl IntrinsicObject for Eval {
    fn init(realm: &Realm) -> crate::JsResult<()> {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, Self::eval)
            .name(Self::NAME)
            .length(1)
            .build()?;

        Ok(())
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.objects().eval().into()
    }
}

impl BuiltInObject for Eval {
    const NAME: JsString = StaticJsStrings::EVAL;
}

struct EvalInstallation<'a> {
    publication: Option<surgeist_js_ast::scope::PublicationPermit<'a>>,
    environment: Option<PreparedEvalEnvironment>,
    active: Option<ActivatedCode>,
    environments: Option<EnvironmentStack>,
}

impl EvalInstallation<'_> {
    fn activate(&mut self, code: &Rc<crate::vm::CodeBlock>, context: &mut Context) -> JsResult<()> {
        context.check_execution_stop()?;
        if self.active.is_some() {
            return Ok(());
        }
        self.environment
            .as_ref()
            .expect("pending eval storage")
            .validate()
            .map_err(|mut error| {
                context.retain_preparation_error(&mut error);
                error
            })?;
        let result = ActivatedCode::publish(
            code.clone(),
            self.publication.take().expect("pending eval publication"),
        );
        let (active, _) = result.map_err(|mut error| {
            context.retain_preparation_error(&mut error);
            error
        })?;
        let environment = self.environment.take().expect("pending eval storage");
        if environment.changes_caller() {
            context.note_execution_effect();
        }
        self.environments = Some(environment.install());
        self.active = Some(active);
        Ok(())
    }
}

impl Eval {
    fn parse<R: ReadChar>(
        source: Source<'_, R>,
        direct: bool,
        strict: bool,
        interner: &mut Interner,
        context: &mut Context,
    ) -> JsResult<(surgeist_js_ast::Script, surgeist_js_ast::SourceText)> {
        let identifier = context.realm().next_parser_identifier()?;
        context.parse_source(source, |mut parser| {
            parser.set_identifier(identifier);
            if strict {
                parser.set_strict();
            }
            Ok(parser.parse_eval(direct, interner)?)
        })
    }

    /// `19.2.1 eval ( x )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-eval-x
    fn eval(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Return ? PerformEval(x, false, false).
        Self::perform_eval(args.get_or_undefined(0), false, None, false, context)
    }

    /// `19.2.1.1 PerformEval ( x, strictCaller, direct )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-performeval
    pub(crate) fn perform_eval(
        x: &JsValue,
        direct: bool,
        lexical_scope: Option<Scope>,
        mut strict: bool,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        bitflags::bitflags! {
            /// Flags used to throw early errors on invalid `eval` calls.
            #[derive(Default)]
            struct Flags: u8 {
                const IN_FUNCTION = 0b0001;
                const IN_METHOD = 0b0010;
                const IN_DERIVED_CONSTRUCTOR = 0b0100;
                const IN_CLASS_FIELD_INITIALIZER = 0b1000;
            }
        }

        // 1. Assert: If direct is false, then strictCaller is also false.
        debug_assert!(direct || !strict);

        // 2. If Type(x) is not String, return x.
        let Some(x) = x.as_string() else {
            return Ok(x.clone());
        };

        // Because of implementation details the following code differs from the spec.

        // 3. Let evalRealm be the current Realm Record.
        // 4. NOTE: In the case of a direct eval, evalRealm is the realm of both the caller of eval
        // and of the eval function itself.
        let eval_realm = context.realm().clone();

        // 5. Perform ? HostEnsureCanCompileStrings(evalRealm, « », x, direct).
        context
            .host_hooks()
            .ensure_can_compile_strings(eval_realm, &[], &x, direct, context.access())
            .map_err(|error| context.runtime().admit_error(error))?;

        // 11. Perform the following substeps in an implementation-defined order, possibly interleaving parsing and error detection:
        //     a. Let script be ParseText(StringToCodePoints(x), Script).
        //     b. If script is a List of errors, throw a SyntaxError exception.
        //     c. If script Contains ScriptBody is false, return undefined.
        //     d. Let body be the ScriptBody of script.
        let mut interner = Interner::new();
        let (body, source) = match x.as_str().variant() {
            JsStrVariant::Latin1(bytes) => Self::parse(
                Source::from_latin1(bytes),
                direct,
                strict,
                &mut interner,
                context,
            ),
            JsStrVariant::Utf16(units) => Self::parse(
                Source::from_utf16(units),
                direct,
                strict,
                &mut interner,
                context,
            ),
        }?;

        // 6. Let inFunction be false.
        // 7. Let inMethod be false.
        // 8. Let inDerivedConstructor be false.
        // 9. Let inClassFieldInitializer be false.
        // a. Let thisEnvRec be GetThisEnvironment().
        let flags = match {
            let frame = context.vm.frame();
            frame
                .environments
                .get_this_environment(frame.realm.environment())
        }
        .kind()
        .as_function()
        {
            // 10. If direct is true, then
            //     b. If thisEnvRec is a Function Environment Record, then
            Some(function_env) if direct => {
                // i. Let F be thisEnvRec.[[FunctionObject]].
                let function_object = function_env.slots().function_object();
                let function_object = function_object
                    .downcast_ref::<OrdinaryFunction>()?
                    .ok_or(crate::error::ObjectAccessError::WrongNativeBrand)?;

                // ii. Set inFunction to true.
                let mut flags = Flags::IN_FUNCTION;

                // iii. Set inMethod to thisEnvRec.HasSuperBinding().
                if function_env.has_super_binding()? {
                    flags |= Flags::IN_METHOD;
                }

                // iv. If F.[[ConstructorKind]] is derived, set inDerivedConstructor to true.
                if function_object.is_derived_constructor() {
                    flags |= Flags::IN_DERIVED_CONSTRUCTOR;
                }

                // v. Let classFieldInitializerName be F.[[ClassFieldInitializerName]].
                // vi. If classFieldInitializerName is not empty, set inClassFieldInitializer to true.
                if function_object.in_class_field_initializer() {
                    flags |= Flags::IN_CLASS_FIELD_INITIALIZER;
                }

                flags
            }
            _ => Flags::default(),
        };

        if !flags.contains(Flags::IN_FUNCTION)
            && context.early_error_contains(&body, ContainsSymbol::NewTarget)?
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid `new.target` expression inside eval")
                .into());
        }
        if !flags.contains(Flags::IN_METHOD)
            && context.early_error_contains(&body, ContainsSymbol::SuperProperty)?
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid `super` reference inside eval")
                .into());
        }
        if !flags.contains(Flags::IN_DERIVED_CONSTRUCTOR)
            && context.early_error_contains(&body, ContainsSymbol::SuperCall)?
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid `super` call inside eval")
                .into());
        }
        if flags.contains(Flags::IN_CLASS_FIELD_INITIALIZER)
            && context.early_error_contains_arguments(&body)?
        {
            return Err(JsNativeError::syntax()
                .with_message("invalid `arguments` reference inside eval")
                .into());
        }

        // Finish private name validation before changing the caller's environment.
        // Indirect eval already validated names without an ambient environment.
        if direct {
            let private_identifiers = context.with_name_checks(|control, vm| {
                let environments = &vm.frame().environments;
                match control {
                    Some(control) => project_private_names(environments, &interner, control),
                    None => project_private_names(environments, &interner, |_| Ok(())),
                }
                .map_err(|limit| RuntimeLimitError::from(limit).into())
            })?;

            if !context.all_private_identifiers_valid(&body, private_identifiers)? {
                return Err(JsNativeError::syntax()
                    .with_message("invalid private identifier")
                    .into());
            }
        }

        strict |= body.strict();

        // Keep caller environments and shared scopes unchanged during preparation.
        let realm = context.realm().clone();
        let (var_environment, variable_scope) = if direct {
            context
                .vm
                .frame()
                .environments
                .outer_function_environment()
                .unwrap_or_else(|| (realm.environment(), realm.scope().clone()))
        } else {
            (realm.environment(), realm.scope().clone())
        };
        let lexical_parent = if direct {
            lexical_scope.unwrap_or_else(|| realm.scope().clone())
        } else {
            realm.scope().clone()
        };
        #[cfg(feature = "annex-b")]
        let (annex_b_function_names, annex_b_globals) = {
            let mut names = Vec::new();
            let globals = prepare_eval_declaration_instantiation(
                &mut names,
                &body,
                &interner,
                strict,
                &variable_scope,
                &lexical_parent,
                context,
            )?;
            (names, globals)
        };
        #[cfg(not(feature = "annex-b"))]
        let (annex_b_function_names, annex_b_globals) = (Vec::new(), Vec::<JsString>::new());
        let in_with = direct && context.compiler_has_object_environment()?;
        let mut prepared = context.with_scope_admission(|admission| {
            body.try_prepare_eval(
                strict,
                &variable_scope,
                &lexical_parent,
                &annex_b_function_names,
                &interner,
                admission,
            )
        })?;
        let parts = prepared.parts()?;
        let scope_owner = parts.scopes;
        let (code_block, extension) = context.with_compilation_work(|mut control| {
            let _depth = control.enter()?;
            control.admit(CompilerWork::MetadataVisit)?;
            let mut extension = control.with_scope_admission(|admission| {
                let owner = scope_owner;
                owner
                    .begin_extension(admission)
                    .map_err(crate::JsError::from)
            })?;
            let mut compiler = ByteCompiler::new(
                js_string!("<eval>"),
                parts.tree.strict(),
                false,
                parts.variable_scope.clone(),
                parts.lexical_scope.clone(),
                false,
                false,
                &mut interner,
                in_with,
                SpannedSourceText::new_source_only(SourceText::new(source)),
                SourcePath::Eval,
                &mut extension,
                control.reborrow(),
            )?;
            compiler.control.admit(CompilerWork::MetadataVisit)?;
            let count = compiler
                .current_open_environments_count
                .checked_add(1)
                .ok_or_else(size_overflow)?;
            compiler.control.admit(CompilerWork::MetadataWrite)?;
            compiler.current_open_environments_count = count;
            #[cfg(feature = "annex-b")]
            {
                compiler.control.admit(CompilerWork::MetadataWrite)?;
                compiler.annex_b_function_names = annex_b_function_names;
            }
            compiler.eval_declaration_instantiation(
                parts.tree,
                strict,
                parts.variable_scope,
                parts.bindings,
            )?;
            compiler.compile_statement_list(parts.tree.statements(), true, false)?;
            let code = compiler.finish(None)?;
            let bytes = payload_bytes::<crate::vm::CodeBlock>(1)?;
            control.admit(CompilerWork::Capacity { bytes })?;
            control.admit(CompilerWork::Copy { bytes })?;
            control.admit(CompilerWork::MetadataWrite)?;
            Ok((Rc::new(code), extension))
        })?;
        extension.commit_private();
        let parts = prepared.parts()?;
        let selected = if direct {
            context.vm.frame().environments.clone()
        } else {
            EnvironmentStack::new()
        };
        let counts = context.with_installation_work(|admission| {
            parts.scopes.with_access(admission, |access| {
                let global = access.associate(&realm.scope().handle())?;
                Ok::<_, crate::JsError>(EvalBindingCounts {
                    lexical: access.num_bindings(parts.lexical_scope)?,
                    variable: access.num_bindings(parts.variable_scope)?,
                    global: access.num_bindings(&global)?,
                })
            })
        })?;
        let environment = context.with_installation_work(|mut admission| {
            if let Some(admit) = admission.as_mut() {
                for count in [counts.lexical, counts.variable, counts.global] {
                    admit(surgeist_js_ast::scope::ScopeWork::Capacity {
                        elements: count as usize,
                    })
                    .map_err(RuntimeLimitError::from)?;
                }
            }
            PreparedEvalEnvironment::new(selected, var_environment, counts, &realm, strict, direct)
        })?;
        context.prepare_guest_call_storage(0, code_block.register_count)?;
        let entry = context.reserve_guest_entry(code_block.clone(), 0)?;
        let runnable = context.get_active_script_or_module();
        let recipes = {
            let mut recipes = Vec::new();
            recipes
                .try_reserve(code_block.global_fns.len())
                .map_err(|_| surgeist_js_ast::scope::ScopeStateError::Allocation)?;
            for function in &code_block.global_fns {
                recipes.push(crate::vm::prepare_function_object_fast(
                    code_block.constant_function(function.function_index as usize),
                    environment.environments(),
                    runnable.clone(),
                    realm.clone(),
                    context,
                )?);
            }
            recipes
        };
        let publication = context.with_installation_work(|admission| {
            parts
                .scopes
                .prepare_publication(admission)
                .map_err(crate::JsError::from)
        })?;
        let mut installation = EvalInstallation {
            publication: Some(publication),
            environment: Some(environment),
            active: None,
            environments: None,
        };
        // Declaration callbacks observe source metadata and the correct environment
        // without exposing pending bytecode as executable authority.
        context.vm.push_host_start(CallFrame::host_start(
            code_block.clone(),
            None,
            installation
                .environment
                .as_ref()
                .expect("private eval storage")
                .environments()
                .clone(),
            realm.clone(),
        ))?;
        let started = (|| {
            context.preflight_eval_declarations(&code_block)?;
            installation
                .environment
                .as_ref()
                .expect("private eval storage")
                .validate()
                .map_err(|mut error| {
                    context.retain_preparation_error(&mut error);
                    error
                })?;
            for name in annex_b_globals {
                context.create_global_var_binding(name, true)?;
            }
            for (function, recipe) in code_block.global_fns.iter().zip(recipes) {
                let function_code = code_block.constant_function(function.function_index as usize);
                let name = code_block.constant_string(function.name_index as usize);
                context.create_global_function_binding_prepared(name, true, |context| {
                    installation.activate(&code_block, context)?;
                    recipe.activate(ActivatedCode::from_installed(function_code)?)
                })?;
            }
            for index in &code_block.global_vars {
                context
                    .create_global_var_binding(code_block.constant_string(*index as usize), true)?;
            }
            installation.activate(&code_block, context)?;
            let environments = installation
                .environments
                .take()
                .expect("installed eval storage");
            let env_fp = (environments.len() - 1) as u32;
            CallFrame::new(
                installation.active.take().expect("installed eval code"),
                entry,
                None,
                environments,
                realm,
            )
            .map(|frame| {
                frame
                    .with_env_fp(env_fp)
                    .with_flags(CallFrameFlags::EXIT_EARLY)
            })
        })();
        context.vm.pop_frame();
        let frame = started?;
        context
            .vm
            .push_frame_with_stack(frame, JsValue::undefined(), JsValue::null());

        let record = context.run();
        context.vm.pop_frame();

        record.consume()
    }
}

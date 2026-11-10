//! Eval-owned Annex B eligibility before private scope preparation.

use crate::{
    Context, JsError, JsResult, JsString,
    bytecompiler::{
        ToJsString,
        control::{CompilerControl, CompilerWork, reserve},
    },
};
use surgeist_js_ast::{
    Script,
    operations::{
        VarScopedDeclaration, try_annex_b_function_declarations_names,
        try_lexically_declared_names, try_var_scoped_declarations,
    },
    scope::{PreparedScopes, Scope, ScopeAccess, ScopeHandle},
};
use surgeist_js_interner::{Interner, Sym};

fn contains_name(names: &[Sym], name: Sym, control: &mut CompilerControl<'_>) -> JsResult<bool> {
    for candidate in names {
        control.admit(CompilerWork::Search)?;
        if *candidate == name {
            return Ok(true);
        }
    }
    Ok(false)
}

fn retain<T>(values: &mut Vec<T>, value: T, control: &mut CompilerControl<'_>) -> JsResult<()> {
    reserve(values, 1, control)?;
    control.admit(CompilerWork::MetadataWrite)?;
    control.admit(CompilerWork::Copy {
        bytes: std::mem::size_of::<T>(),
    })?;
    values.push(value);
    Ok(())
}

fn declared_functions(body: &Script, control: &mut CompilerControl<'_>) -> JsResult<Vec<Sym>> {
    let declarations =
        control.with_name_admission(|admission| try_var_scoped_declarations(body, admission))?;
    let mut names = Vec::new();
    for declaration in declarations.iter().rev() {
        control.admit(CompilerWork::MetadataVisit)?;
        let name = match *declaration {
            VarScopedDeclaration::FunctionDeclaration(function) => function.name(),
            VarScopedDeclaration::GeneratorDeclaration(function) => function.name(),
            VarScopedDeclaration::AsyncFunctionDeclaration(function) => function.name(),
            VarScopedDeclaration::AsyncGeneratorDeclaration(function) => function.name(),
            VarScopedDeclaration::VariableDeclaration(_) => continue,
        };
        if !contains_name(&names, name.sym(), control)? {
            retain(&mut names, name.sym(), control)?;
        }
    }
    Ok(names)
}

/// A read-only canonical view rejects callback-owned scope changes without
/// publishing or allocating shared identities. No scope reference escapes it.
fn with_scope<T>(
    scope: &Scope,
    control: &mut CompilerControl<'_>,
    run: impl FnOnce(&mut ScopeAccess<'_, JsError>, &ScopeHandle) -> JsResult<T>,
) -> JsResult<T> {
    let mut scopes = PreparedScopes::new(scope);
    control.with_scope_admission(|admission| {
        scopes.with_access(admission, |access| {
            let root = access.root_handle()?;
            run(access, &root)
        })
    })
}

fn ancestor_binding_exists(
    lexical_scope: &Scope,
    name: Sym,
    interner: &Interner,
    control: &mut CompilerControl<'_>,
) -> JsResult<bool> {
    let mut scopes = PreparedScopes::new(lexical_scope);
    let lexical = control.with_scope_admission(|admission| {
        scopes.with_access(admission, |access| Ok::<_, JsError>(access.root_handle()?))
    })?;
    control.admit(CompilerWork::Copy {
        bytes: std::mem::size_of::<ScopeHandle>(),
    })?;
    let mut current = lexical.clone();
    // The eligibility traversal compares against its lexical starting scope.
    while control.with_scope_admission(|admission| {
        scopes.with_access(admission, |access| {
            Ok::<_, JsError>(access.scope_index(&current)? != access.scope_index(&lexical)?)
        })
    })? {
        control.admit(CompilerWork::Search)?;
        let string = name.try_to_js_string(interner, control)?;
        if control.with_scope_admission(|admission| {
            scopes.with_access(admission, |access| {
                Ok::<_, JsError>(access.has_binding(&current, &string)?)
            })
        })? {
            return Ok(true);
        }
        let outer = control.with_scope_admission(|admission| {
            scopes.with_access(admission, |access| {
                Ok::<_, JsError>(access.outer(&current)?)
            })
        })?;
        let Some(outer) = outer else { break };
        current = outer;
    }
    Ok(false)
}

#[derive(Debug)]
enum Eligibility {
    Excluded,
    Eligible,
    GlobalCheck(JsString),
}

fn before_global_check(
    name: Sym,
    lexical_names: &[Sym],
    var_env: &Scope,
    lex_env: &Scope,
    interner: &Interner,
    control: &mut CompilerControl<'_>,
) -> JsResult<Eligibility> {
    control.admit(CompilerWork::MetadataVisit)?;
    if contains_name(lexical_names, name, control)?
        || ancestor_binding_exists(lex_env, name, interner, control)?
    {
        return Ok(Eligibility::Excluded);
    }
    if !with_scope(var_env, control, |access, root| Ok(access.is_global(root)?))? {
        return Ok(Eligibility::Eligible);
    }
    let string = name.try_to_js_string(interner, control)?;
    if with_scope(var_env, control, |access, root| {
        Ok(access.has_lex_binding(root, &string)?)
    })? {
        Ok(Eligibility::Excluded)
    } else {
        Ok(Eligibility::GlobalCheck(string))
    }
}

/// `EvalDeclarationInstantiation ( body, varEnv, lexEnv, privateEnv, strict )`
///
/// Pure eligibility and metadata work share the current compilation operation;
/// real global-object checks run after each borrowed control has been released.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-evaldeclarationinstantiation
#[cfg(feature = "annex-b")]
pub(crate) fn prepare_eval_declaration_instantiation(
    annex_b_function_names: &mut Vec<Sym>,
    body: &Script,
    interner: &Interner,
    strict: bool,
    var_env: &Scope,
    lex_env: &Scope,
    context: &mut Context,
) -> JsResult<Vec<JsString>> {
    // Private identifiers are validated before this preparation. Lexical/var
    // bindings and callable recipes remain with the subsequent private stages.
    let (declared, lexical, candidates) = context.with_compilation_work(|mut control| {
        let declared = declared_functions(body, &mut control)?;
        if strict {
            return Ok((declared, Vec::new(), Vec::new()));
        }
        let lexical = control
            .with_name_admission(|admission| try_lexically_declared_names(body, admission))?;
        let candidates = control.with_name_admission(|admission| {
            try_annex_b_function_declarations_names(body, admission)
        })?;
        Ok((declared, lexical, candidates))
    })?;
    let mut global_names = Vec::new();
    for name in candidates {
        let eligibility = context.with_compilation_work(|mut control| {
            before_global_check(name, &lexical, var_env, lex_env, interner, &mut control)
        })?;
        let definable = match eligibility {
            Eligibility::Excluded => false,
            Eligibility::Eligible => true,
            // This can invoke host or guest code. No compiler control or scope
            // view is borrowed here, and its effects are never rolled back.
            Eligibility::GlobalCheck(string) => context.can_declare_global_var(&string)?,
        };
        if !definable {
            continue;
        }
        context.with_compilation_work(|mut control| {
            if !contains_name(&declared, name, &mut control)?
                && with_scope(var_env, &mut control, |access, root| {
                    Ok(access.is_global(root)?)
                })?
            {
                let string = name.try_to_js_string(interner, &mut control)?;
                retain(&mut global_names, string, &mut control)?;
            }
            // Repeated candidates remain repeated, as do their global checks.
            retain(annex_b_function_names, name, &mut control)
        })?;
    }
    Ok(global_names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EngineError, JsValue,
        bytecompiler::control::CompilerDepth,
        error::RuntimeLimitError,
        execution::{Execution, ExecutionOptions, ExecutionReport},
    };
    use surgeist_js_ast::{
        Declaration, LinearPosition, LinearSpan, Span, Statement, StatementList, StatementListItem,
        expression::Identifier,
        function::{FormalParameterList, FunctionBody, FunctionDeclaration},
        scope::{ScopeStateError, ScopeWork},
        statement::Block,
    };

    fn function(name: Sym) -> StatementListItem {
        Declaration::FunctionDeclaration(FunctionDeclaration::new(
            Identifier::new(name, Span::new((1, 1), (1, 2))),
            FormalParameterList::default(),
            FunctionBody::new(StatementList::default(), Span::new((1, 1), (1, 2))),
            LinearSpan::new(LinearPosition::new(0), LinearPosition::new(1)),
        ))
        .into()
    }

    fn source(blocks: usize) -> (Script, Interner, Sym) {
        let mut interner = Interner::default();
        let name = interner.get_or_intern("annex_candidate");
        let statements = (0..blocks)
            .map(|_| {
                Statement::Block(Block::from(StatementList::new(
                    vec![function(name)],
                    LinearPosition::new(1),
                    false,
                )))
                .into()
            })
            .collect::<Vec<StatementListItem>>();
        (
            Script::new(StatementList::new(
                statements,
                LinearPosition::new(1),
                false,
            )),
            interner,
            name,
        )
    }

    #[test]
    fn declared_function_collection_forwards_ast_work_once_and_searches_only_actual_candidates() {
        let mut interner = Interner::default();
        let a = interner.get_or_intern("a");
        let b = interner.get_or_intern("b");
        let body = Script::new(StatementList::new(
            vec![function(a), function(b), function(a)],
            LinearPosition::new(1),
            false,
        ));
        let mut expected_names = Vec::new();
        try_var_scoped_declarations(
            &body,
            Some(&mut |work| {
                expected_names.push(work);
                Ok::<_, JsError>(())
            }),
        )
        .expect("AST declaration producer");
        let mut names = Vec::new();
        let mut candidates = 0;
        let mut admission = |work| {
            match work {
                CompilerWork::Name(work) => names.push(work),
                CompilerWork::Search => candidates += 1,
                _ => {}
            }
            Ok(())
        };
        let depth = CompilerDepth::new();
        assert_eq!(
            declared_functions(
                &body,
                &mut CompilerControl::new(Some(&mut admission), &depth)
            )
            .expect("declared functions"),
            [a, b]
        );
        assert_eq!(names, expected_names);
        assert_eq!(candidates, 2); // Empty list; one miss for b; first-slot match for a.
    }

    #[test]
    fn lexical_starting_scope_comparison_keeps_the_existing_ancestor_eligibility_behavior() {
        let mut interner = Interner::default();
        let name = interner.get_or_intern("candidate");
        let global = Scope::new_global();
        let lexical = Scope::new(global.clone(), false);
        drop(lexical.create_mutable_binding(JsString::from("candidate"), false));
        let depth = CompilerDepth::new();
        let eligibility = before_global_check(
            name,
            &[],
            &global,
            &lexical,
            &interner,
            &mut CompilerControl::new(None, &depth),
        )
        .expect("eligibility");
        assert!(
            matches!(eligibility, Eligibility::GlobalCheck(string) if string.as_str() == crate::JsStr::latin1(b"candidate"))
        );
        assert!(lexical.has_binding(&JsString::from("candidate")));
    }

    #[test]
    fn scope_callback_edits_survive_conflict_and_refusal_keeps_its_original_reason() {
        let mut interner = Interner::default();
        let name = interner.get_or_intern("candidate");
        for refuse in [false, true] {
            let root = Scope::new_global();
            let marker = JsError::from_opaque(JsValue::new(318)).unwrap();
            let mut changed = false;
            let mut admission = |work| {
                assert!(
                    !changed,
                    "eligibility continued after a scope conflict or refusal"
                );
                if matches!(work, CompilerWork::Scope(ScopeWork::ScopeProbe)) {
                    drop(root.create_mutable_binding(JsString::from("callback"), false));
                    changed = true;
                    if refuse {
                        return Err(marker.clone());
                    }
                }
                Ok(())
            };
            let depth = CompilerDepth::new();
            let error = before_global_check(
                name,
                &[],
                &root,
                &root,
                &interner,
                &mut CompilerControl::new(Some(&mut admission), &depth),
            )
            .expect_err("scope changed");
            if refuse {
                assert_eq!(error, marker);
            } else {
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
                );
            }
            assert!(root.has_binding(&JsString::from("callback")));
        }
    }

    fn run_with_limit(
        body: &Script,
        interner: &Interner,
        limit: u64,
        strict: bool,
    ) -> (JsResult<Vec<JsString>>, Vec<Sym>, ExecutionReport) {
        let mut context = Context::builder().build().unwrap();
        let root = context.realm().scope().clone();
        let before = Scope::new(root.clone(), false).unique_id();
        let bindings = root.num_bindings();
        context.execution = Some(
            Execution::new(ExecutionOptions::new(32).with_compiler_work_limit(limit))
                .expect("operation budget"),
        );
        let mut names = Vec::new();
        let result = prepare_eval_declaration_instantiation(
            &mut names,
            body,
            interner,
            strict,
            &root,
            &root,
            &mut context,
        );
        let completion = result
            .as_ref()
            .map(|_| JsValue::undefined())
            .map_err(Clone::clone);
        let outcome = context
            .execution
            .take()
            .expect("same operation")
            .finish(completion);
        assert_eq!(root.num_bindings(), bindings);
        assert_eq!(Scope::new(root, false).unique_id(), before + 1);
        (result, names, outcome.report)
    }

    #[test]
    fn repeated_eligible_candidates_remain_repeated_and_strict_eval_retains_none() {
        let (body, interner, name) = source(2);
        let (result, names, _) = run_with_limit(&body, &interner, u64::MAX, false);
        assert_eq!(
            result.expect("eligible globals"),
            [
                JsString::from("annex_candidate"),
                JsString::from("annex_candidate")
            ]
        );
        assert_eq!(names, [name, name]);
        let (result, names, _) = run_with_limit(&body, &interner, u64::MAX, true);
        assert!(result.expect("strict globals").is_empty());
        assert!(names.is_empty());
    }

    #[test]
    fn pure_blocks_share_one_operation_limit_and_preserve_accepted_prefix_on_late_refusal() {
        let (body, interner, name) = source(2);
        let (result, _, report) = run_with_limit(&body, &interner, u64::MAX, false);
        result.expect("calibrate eligible globals");
        let exact = report.compiler_work().unwrap();
        assert!(exact > 0);
        let (result, names, report) = run_with_limit(&body, &interner, exact, false);
        assert_eq!(result.expect("exact work").len(), 2);
        assert_eq!(names, [name, name]);
        assert_eq!(report.compiler_work().unwrap(), exact);
        let limit = exact - 1;
        let (result, names, report) = run_with_limit(&body, &interner, limit, false);
        assert_eq!(
            result.expect_err("last metadata copy refused").as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::CompilerWork { limit }
            ))
        );
        assert_eq!(names, [name]);
        assert!(report.compiler_work().unwrap() <= limit);
    }
}

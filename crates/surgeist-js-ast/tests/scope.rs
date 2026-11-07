//! Tests for the scope analysis of the AST.

#![allow(unused_crate_dependencies)]

use surgeist_js_ast::{
    Declaration, Expression, LinearPosition, LinearSpan, Script, Span, Statement, StatementList,
    StatementListItem,
    declaration::{LexicalDeclaration, Variable},
    expression::Identifier,
    function::{FormalParameter, FormalParameterList, FunctionBody, FunctionDeclaration},
    scope::Scope,
    statement::Block,
};
use surgeist_js_interner::Interner;
use surgeist_js_string::JsString;

#[test]
fn empty_script_is_ok() {
    let scope = &Scope::new_global();
    let mut script = Script::default();
    let ok = script.analyze_scope(scope, &Interner::new()).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
}

#[test]
fn script_global_let() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(a, Span::new((1, 1), (1, 1))),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        LinearPosition::default(),
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
}

#[test]
fn script_global_const() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::Lexical(LexicalDeclaration::Const(
            vec![Variable::from_identifier(
                Identifier::new(a, Span::new((1, 1), (1, 1))),
                None,
            )]
            .try_into()
            .unwrap(),
        ))
        .into()],
        LinearPosition::default(),
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
}

#[test]
fn script_block_let() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Statement::Block(Block::from((
            vec![
                Declaration::Lexical(LexicalDeclaration::Let(
                    vec![Variable::from_identifier(
                        Identifier::new(a, Span::new((1, 1), (1, 1))),
                        None,
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into(),
            ],
            LinearPosition::default(),
        )))
        .into()],
        LinearPosition::default(),
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
    let StatementListItem::Statement(statement) = script.statements().first().unwrap() else {
        panic!("Expected a block statement");
    };
    let Statement::Block(block) = statement.as_ref() else {
        panic!("Expected a block statement");
    };
    let handle = block.scope().unwrap();
    let scope = handle
        .resolve_live(&handle.installed_scopes().unwrap())
        .unwrap();
    assert_eq!(scope.num_bindings(), 1);
    let a = scope.get_binding_reference(&JsString::from("a")).unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}

#[test]
fn script_function_mapped_arguments_not_accessed() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let f = interner.get_or_intern("f");
    let a = interner.get_or_intern("a");
    let mut script = Script::new(StatementList::new(
        [Declaration::FunctionDeclaration(FunctionDeclaration::new(
            Identifier::new(f, Span::new((1, 1), (1, 1))),
            FormalParameterList::from_parameters(vec![FormalParameter::new(
                Variable::from_identifier(Identifier::new(a, Span::new((1, 1), (1, 1))), None),
                false,
            )]),
            FunctionBody::new(
                StatementList::new(
                    [Declaration::Lexical(LexicalDeclaration::Let(
                        vec![Variable::from_identifier(
                            Identifier::new(a, Span::new((1, 1), (1, 1))),
                            None,
                        )]
                        .try_into()
                        .unwrap(),
                    ))
                    .into()],
                    LinearPosition::default(),
                    false,
                ),
                Span::new((1, 1), (1, 1)),
            ),
            LinearSpan::default(),
        ))
        .into()],
        LinearPosition::default(),
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);

    let StatementListItem::Declaration(declaration) = script.statements().first().unwrap() else {
        panic!("Expected a block statement");
    };
    let Declaration::FunctionDeclaration(f) = declaration.as_ref() else {
        panic!("Expected a block statement");
    };

    assert_eq!(
        f.scopes()
            .function_scope()
            .resolve_live(&scope.installed_scopes())
            .unwrap()
            .num_bindings(),
        2
    );
    assert_eq!(f.scopes().parameters_eval_scope(), None);
    assert_eq!(f.scopes().parameters_scope(), None);
    assert_eq!(
        f.scopes()
            .lexical_scope()
            .unwrap()
            .resolve_live(&scope.installed_scopes())
            .unwrap()
            .num_bindings(),
        1
    );
    let arguments = f
        .scopes()
        .function_scope()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("arguments"))
        .unwrap();
    assert!(
        !f.scopes()
            .arguments_object_accessed(&scope.installed_scopes())
            .unwrap()
    );
    assert!(!arguments.is_global_object());
    assert!(arguments.is_lexical());
    assert!(arguments.local());
    let a = f
        .scopes()
        .function_scope()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
    let a = f
        .scopes()
        .lexical_scope()
        .unwrap()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}

#[test]
fn script_function_mapped_arguments_accessed() {
    let scope = Scope::new_global();
    let mut interner = Interner::new();
    let f = interner.get_or_intern("f");
    let a = interner.get_or_intern("a");
    let arguments = interner.get_or_intern("arguments");
    let mut script = Script::new(StatementList::new(
        [Declaration::FunctionDeclaration(FunctionDeclaration::new(
            Identifier::new(f, Span::new((1, 1), (1, 1))),
            FormalParameterList::from_parameters(vec![FormalParameter::new(
                Variable::from_identifier(Identifier::new(a, Span::new((1, 1), (1, 1))), None),
                false,
            )]),
            FunctionBody::new(
                StatementList::new(
                    [
                        Declaration::Lexical(LexicalDeclaration::Let(
                            vec![Variable::from_identifier(
                                Identifier::new(a, Span::new((1, 1), (1, 1))),
                                None,
                            )]
                            .try_into()
                            .unwrap(),
                        ))
                        .into(),
                        Statement::Expression(Expression::Identifier(Identifier::new(
                            arguments,
                            Span::new((1, 1), (1, 1)),
                        )))
                        .into(),
                    ],
                    LinearPosition::default(),
                    false,
                ),
                Span::new((1, 1), (1, 1)),
            ),
            LinearSpan::default(),
        ))
        .into()],
        LinearPosition::default(),
        false,
    ));
    let ok = script.analyze_scope(&scope, &interner).is_ok();
    assert!(ok);
    assert_eq!(scope.num_bindings(), 0);
    let StatementListItem::Declaration(declaration) = script.statements().first().unwrap() else {
        panic!("Expected a block statement");
    };
    let Declaration::FunctionDeclaration(f) = declaration.as_ref() else {
        panic!("Expected a block statement");
    };
    assert!(
        f.scopes()
            .arguments_object_accessed(&scope.installed_scopes())
            .unwrap()
    );
    assert_eq!(
        f.scopes()
            .function_scope()
            .resolve_live(&scope.installed_scopes())
            .unwrap()
            .num_bindings(),
        2
    );
    assert_eq!(f.scopes().parameters_eval_scope(), None);
    assert_eq!(f.scopes().parameters_scope(), None);
    assert_eq!(
        f.scopes()
            .lexical_scope()
            .unwrap()
            .resolve_live(&scope.installed_scopes())
            .unwrap()
            .num_bindings(),
        1
    );
    let arguments = f
        .scopes()
        .function_scope()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("arguments"))
        .unwrap();
    assert!(!arguments.is_global_object());
    assert!(arguments.is_lexical());
    assert!(arguments.local());
    let a = f
        .scopes()
        .function_scope()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(!a.local());
    let a = f
        .scopes()
        .lexical_scope()
        .unwrap()
        .resolve_live(&scope.installed_scopes())
        .unwrap()
        .get_binding_reference(&JsString::from("a"))
        .unwrap();
    assert!(!a.is_global_object());
    assert!(a.is_lexical());
    assert!(a.local());
}

#[test]
fn multiple_scopes_with_same_parent_have_distinct_ids() {
    let global = Scope::new_global();
    let child1 = Scope::new(global.clone(), false);
    let child2 = Scope::new(global.clone(), false);
    let child3 = Scope::new(child1.clone(), false);
    let child4 = Scope::new(child1.clone(), false);

    // Check uniqueness by inserting in a set and asserting that there are
    // 5 elements numbered from 0 to 4.
    let mut ids = [global, child1, child2, child3, child4]
        .into_iter()
        .map(|scope| scope.unique_id())
        .collect::<Vec<u32>>();
    ids.sort_unstable();

    assert_eq!(ids, vec![0, 1, 2, 3, 4]);
}

#[test]
fn can_correlate_binding_to_scope() {
    // GIVEN
    let global = Scope::new_global();
    let child1 = Scope::new(global.clone(), false);
    let child2 = Scope::new(child1.clone(), false);
    let _unused = child1.create_mutable_binding("x".into(), false);

    // WHEN
    let binding = child2.get_identifier_reference("x".into());

    // THEN
    assert_eq!(binding.locator().unique_scope_id(), child1.unique_id());
}

mod prepared_scope {
    use surgeist_js_ast::{
        Declaration, LinearPosition, Script, Span, StatementList,
        declaration::{LexicalDeclaration, Variable},
        expression::Identifier,
        scope::{
            BasisStatus, BindingLocatorError, Scope, ScopeAccess, ScopeAnalysisError,
            ScopeStateError, ScopeWork,
        },
    };
    use surgeist_js_interner::Interner;
    use surgeist_js_string::JsString;

    fn lexical_script(interner: &mut Interner, constant: bool) -> Script {
        let name = interner.get_or_intern("name");
        let variables = vec![Variable::from_identifier(
            Identifier::new(name, Span::new((1, 1), (1, 5))),
            None,
        )]
        .try_into()
        .unwrap();
        let declaration = if constant {
            LexicalDeclaration::Const(variables)
        } else {
            LexicalDeclaration::Let(variables)
        };
        Script::new(StatementList::new(
            [Declaration::Lexical(declaration).into()],
            LinearPosition::default(),
            false,
        ))
    }

    #[test]
    fn private_projection_uses_fresh_slots_and_source_mutability() {
        for proposed_const in [false, true] {
            let root = Scope::new_global();
            let name = JsString::from("name");
            let mut interner = Interner::new();
            lexical_script(&mut interner, !proposed_const)
                .analyze_scope(&root, &interner)
                .unwrap();
            let mut prepared = lexical_script(&mut interner, proposed_const)
                .try_prepare::<()>(&root, &interner, None)
                .unwrap();
            assert_eq!(root.num_bindings(), 1);
            let pending = prepared.scopes().root_handle();
            assert_eq!(
                pending.installed_scopes().unwrap_err(),
                ScopeStateError::Unprepared
            );
            let parts = prepared.parts().unwrap();
            parts
                .scopes
                .with_access(
                    None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                    |access| {
                        assert_eq!(access.num_bindings(&pending)?, 2);
                        let projected = access.get_binding_reference(&pending, &name)?.unwrap();
                        assert_eq!(projected.locator().binding_index(), 1);
                        let assignment = access.set_mutable_binding(&pending, name.clone())?;
                        if proposed_const {
                            assert!(matches!(
                                assignment,
                                Err(BindingLocatorError::MutateImmutable)
                            ));
                        } else {
                            assert_eq!(assignment.unwrap().locator().binding_index(), 1);
                        }
                        Ok::<_, ScopeAnalysisError<()>>(())
                    },
                )
                .unwrap();
            assert_eq!(prepared.declarations().lexical_names(), &[name]);
            assert_eq!(
                prepared
                    .declarations()
                    .check_against_live::<()>(&root, None),
                Err(ScopeAnalysisError::Syntax("duplicate lexical declaration"))
            );
            assert!(prepared.tree().is_ok());
            assert_eq!(root.num_bindings(), 1);
        }
    }

    #[test]
    fn empty_script_has_pending_association_and_installs_canonically() {
        let root = Scope::new_global();
        let interner = Interner::new();
        let mut prepared = Script::default()
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let handle = prepared.scopes().root_handle();
        assert_eq!(
            handle.installed_scopes().unwrap_err(),
            ScopeStateError::Unprepared
        );
        assert!(handle.resolve_live(&root.installed_scopes()).is_err());
        let installed = prepared
            .parts()
            .unwrap()
            .scopes
            .prepare_publication::<()>(None)
            .unwrap()
            .commit()
            .unwrap();
        let canonical = handle.resolve_live(&installed).unwrap();
        drop(canonical.create_mutable_binding(JsString::from("published"), false));
        assert!(root.has_binding(&JsString::from("published")));
        // Installed handles do not inherit pending freshness restrictions.
        assert!(handle.resolve_live(&installed).is_ok());
    }

    #[test]
    fn stale_pending_refresh_remains_inspectable_and_conflicts_only_on_evaluation() {
        let root = Scope::new_global();
        let mut interner = Interner::new();
        let mut first = lexical_script(&mut interner, false)
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let mut second = lexical_script(&mut interner, false)
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let old_handle = second.scopes().root_handle();
        first
            .parts()
            .unwrap()
            .scopes
            .prepare_publication::<()>(None)
            .unwrap()
            .commit()
            .unwrap();
        assert_eq!(
            second.scopes().validate_basis::<()>(None).unwrap(),
            BasisStatus::Stale
        );
        second.refresh::<()>(&root, &interner, None).unwrap();
        assert!(second.tree().is_ok());
        assert_eq!(
            old_handle.installed_scopes().unwrap_err(),
            ScopeStateError::AssociationMismatch
        );
        assert_eq!(
            second.declarations().check_against_live::<()>(&root, None),
            Err(ScopeAnalysisError::Syntax("duplicate lexical declaration"))
        );
        assert_eq!(root.num_bindings(), 1);
    }

    #[test]
    fn refused_reset_retains_syntax_and_original_names_for_retry() {
        let root = Scope::new_global();
        let mut interner = Interner::new();
        let mut prepared = lexical_script(&mut interner, false)
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let old_handle = prepared.scopes().root_handle();
        {
            let _syntax = prepared.edit_tree().unwrap();
        }
        let mut visits = 0;
        let mut refuse = |work| {
            if work == ScopeWork::Node {
                visits += 1;
                Err("reset")
            } else {
                Ok(())
            }
        };
        assert_eq!(
            prepared.refresh(&root, &interner, Some(&mut refuse)),
            Err(ScopeAnalysisError::Refused("reset"))
        );
        assert_eq!(visits, 1);
        assert_eq!(prepared.tree().unwrap_err(), ScopeStateError::Unprepared);
        assert_eq!(
            prepared.declarations().lexical_names(),
            &[JsString::from("name")]
        );
        assert!(old_handle.installed_scopes().is_err());
        prepared.refresh::<()>(&root, &interner, None).unwrap();
        assert_eq!(prepared.tree().unwrap().statements().len(), 1);
        assert_eq!(root.num_bindings(), 0);
    }

    #[test]
    fn compiler_savepoint_abort_invalidates_aliases_and_restores_private_ids() {
        let root = Scope::new_global();
        let interner = Interner::new();
        let mut prepared = Script::default()
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let pending_root = prepared.scopes().root_handle();
        let scopes = prepared.parts().unwrap().scopes;
        let child;
        let alias;
        {
            let mut extension = scopes.begin_extension::<()>(None).unwrap();
            (child, alias) = extension
                .with_access(
                    None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                    |access| {
                        let child = access.new_scope(pending_root.clone(), false)?;
                        let alias = access.associate(&child)?;
                        Ok::<_, ScopeAnalysisError<()>>((child, alias))
                    },
                )
                .unwrap();
        }
        scopes
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| {
                    assert!(access.scope_index(&child).is_err());
                    assert!(access.scope_index(&alias).is_err());
                    Ok::<_, ScopeAnalysisError<()>>(())
                },
            )
            .unwrap();
        let mut extension = scopes.begin_extension::<()>(None).unwrap();
        let retained = extension
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| access.new_scope(pending_root.clone(), false),
            )
            .unwrap();
        extension.commit_private();
        let installed = scopes
            .prepare_publication::<()>(None)
            .unwrap()
            .commit()
            .unwrap();
        assert_eq!(retained.resolve_live(&installed).unwrap().unique_id(), 1);
        assert_eq!(Scope::new(root, false).unique_id(), 2);
        assert!(child.resolve_live(&installed).is_err());
    }

    #[test]
    fn refusal_keeps_callback_effect_and_does_not_publish_private_changes() {
        let root = Scope::new_global();
        let mut interner = Interner::new();
        let mut prepared = lexical_script(&mut interner, false)
            .try_prepare::<()>(&root, &interner, None)
            .unwrap();
        let mut admission = |work| {
            if work == ScopeWork::Publication {
                drop(root.create_mutable_binding(JsString::from("callback"), false));
                return Err("caller stop");
            }
            Ok(())
        };
        let result = prepared
            .parts()
            .unwrap()
            .scopes
            .prepare_publication(Some(&mut admission));
        assert!(matches!(
            result,
            Err(ScopeAnalysisError::Refused("caller stop"))
        ));
        assert_eq!(root.num_bindings(), 1);
        assert!(!root.has_binding(&JsString::from("name")));
        assert!(root.has_binding(&JsString::from("callback")));
        assert!(
            ScopeAccess::with_live(
                &root,
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| access.root_handle()
            )
            .is_ok()
        );
    }
    #[cfg(feature = "annex-b")]
    #[test]
    fn original_annex_candidates_keep_duplicates_and_ignore_live_conflicts() {
        use surgeist_js_ast::{
            LinearSpan, Statement, StatementListItem,
            function::{FormalParameterList, FunctionBody, FunctionDeclaration},
            statement::Block,
        };
        use surgeist_js_interner::Sym;
        fn block(name: Sym) -> StatementListItem {
            Statement::Block(Block::from(StatementList::new(
                [Declaration::FunctionDeclaration(FunctionDeclaration::new(
                    Identifier::new(name, Span::new((1, 1), (1, 2))),
                    FormalParameterList::default(),
                    FunctionBody::new(StatementList::default(), Span::new((1, 1), (1, 2))),
                    LinearSpan::default(),
                ))
                .into()],
                LinearPosition::default(),
                false,
            )))
            .into()
        }
        let root = Scope::new_global();
        let mut interner = Interner::new();
        let one = interner.get_or_intern("one");
        let two = interner.get_or_intern("two");
        let excluded = interner.get_or_intern("name");
        let mut seed = Script::new(StatementList::new(
            [Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(one, Span::new((1, 1), (1, 2))),
                    None,
                )]
                .try_into()
                .unwrap(),
            ))
            .into()],
            LinearPosition::default(),
            false,
        ));
        seed.analyze_scope(&root, &interner).unwrap();
        let excluded_declaration = Declaration::Lexical(LexicalDeclaration::Let(
            vec![Variable::from_identifier(
                Identifier::new(excluded, Span::new((1, 1), (1, 2))),
                None,
            )]
            .try_into()
            .unwrap(),
        ));
        let script = Script::new(StatementList::new(
            vec![
                excluded_declaration.into(),
                block(one),
                block(one),
                block(excluded),
                block(two),
            ],
            LinearPosition::default(),
            false,
        ));
        let mut prepared = script.try_prepare::<()>(&root, &interner, None).unwrap();
        assert_eq!(
            prepared.declarations().annex_b_candidates(),
            &[
                JsString::from("one"),
                JsString::from("one"),
                JsString::from("two")
            ]
        );
        {
            let _syntax = prepared.edit_tree().unwrap();
        }
        prepared.refresh::<()>(&root, &interner, None).unwrap();
        assert_eq!(
            prepared.declarations().annex_b_candidates(),
            &[
                JsString::from("one"),
                JsString::from("one"),
                JsString::from("two")
            ]
        );
        assert_eq!(root.num_bindings(), 1);
    }

    #[test]
    fn refresh_preserves_class_binding_syntax_and_template_site_identity() {
        use surgeist_js_ast::{
            Expression, Statement, StatementListItem, expression::TaggedTemplate,
            function::ClassExpression,
        };
        let root = Scope::new_global();
        let mut interner = Interner::new();
        let name = interner.get_or_intern("Named");
        let span = Span::new((1, 1), (1, 10));
        let class = ClassExpression::new(
            Some(Identifier::new(name, span)),
            None,
            None,
            Box::default(),
            true,
            span,
        );
        let text = interner.get_or_intern("text");
        let tag = TaggedTemplate::new(
            Expression::ClassExpression(class.into()),
            Box::new([text]),
            Box::new([Some(text)]),
            Box::default(),
            73,
            span,
        );
        let script = Script::new(StatementList::new(
            [Statement::Expression(tag.into()).into()],
            LinearPosition::default(),
            false,
        ));
        let mut prepared = script.try_prepare::<()>(&root, &interner, None).unwrap();
        let class_handle = {
            let StatementListItem::Statement(statement) = &prepared.tree().unwrap().statements()[0]
            else {
                panic!("statement");
            };
            let Statement::Expression(Expression::TaggedTemplate(tag)) = statement.as_ref() else {
                panic!("tag");
            };
            let Expression::ClassExpression(class) = tag.tag() else {
                panic!("class");
            };
            assert!(class.has_binding_identifier());
            class.name_scope().unwrap().clone()
        };
        {
            let _syntax = prepared.edit_tree().unwrap();
        }
        prepared.refresh::<()>(&root, &interner, None).unwrap();
        let StatementListItem::Statement(statement) = &prepared.tree().unwrap().statements()[0]
        else {
            panic!("statement");
        };
        let Statement::Expression(Expression::TaggedTemplate(tag)) = statement.as_ref() else {
            panic!("tag");
        };
        assert_eq!(tag.identifier(), 73);
        let Expression::ClassExpression(class) = tag.tag() else {
            panic!("class");
        };
        assert!(class.has_binding_identifier());
        assert!(class.name_scope().is_some());
        assert_eq!(
            class_handle.installed_scopes().unwrap_err(),
            ScopeStateError::AssociationMismatch
        );
        assert_eq!(root.num_bindings(), 0);
    }
    #[test]
    fn installed_independent_analysis_values_compare_structurally_including_flags() {
        use surgeist_js_ast::{PreparedScript, Statement, StatementListItem, statement::Block};
        fn block_script(interner: &mut Interner, constant: bool) -> Script {
            let mut inner = lexical_script(interner, constant);
            let block = Block::from(std::mem::take(inner.statements_mut()));
            Script::new(StatementList::new(
                [Statement::Block(block).into()],
                LinearPosition::default(),
                false,
            ))
        }
        fn block_handle(prepared: &PreparedScript) -> surgeist_js_ast::scope::ScopeHandle {
            let StatementListItem::Statement(statement) = &prepared.tree().unwrap().statements()[0]
            else {
                panic!("block statement");
            };
            let Statement::Block(block) = statement.as_ref() else {
                panic!("block");
            };
            block.scope().unwrap().clone()
        }
        let mut interner = Interner::new();
        let first_root = Scope::new_global();
        let second_root = Scope::new_global();
        let const_root = Scope::new_global();
        let mut first = block_script(&mut interner, false)
            .try_prepare::<()>(&first_root, &interner, None)
            .unwrap();
        let mut second = block_script(&mut interner, false)
            .try_prepare::<()>(&second_root, &interner, None)
            .unwrap();
        let mut constant = block_script(&mut interner, true)
            .try_prepare::<()>(&const_root, &interner, None)
            .unwrap();
        let first_handle = block_handle(&first);
        let second_handle = block_handle(&second);
        let const_handle = block_handle(&constant);
        assert_ne!(first_handle, second_handle);
        for prepared in [&mut first, &mut second, &mut constant] {
            prepared
                .parts()
                .unwrap()
                .scopes
                .prepare_publication::<()>(None)
                .unwrap()
                .commit()
                .unwrap();
        }
        assert_eq!(first_root, second_root);
        assert_eq!(first_root, const_root);
        assert_eq!(first_handle, second_handle);
        assert!(matches!(
            first_handle.resolve_live(&second_handle.installed_scopes().unwrap()),
            Err(ScopeStateError::AssociationMismatch)
        ));
        assert_eq!(first.tree().unwrap(), second.tree().unwrap());
        // Both bindings have the same name, index, lexical status, enclosing
        // values, and unique scope ID. Their const/mutable flags differ.
        assert_ne!(first_handle, const_handle);
    }

    #[test]
    fn pending_projections_of_the_same_canonical_root_are_owner_isolated() {
        use surgeist_js_ast::scope::PreparedScopes;
        let root = Scope::new_global();
        let first = PreparedScopes::new(&root);
        let second = PreparedScopes::new(&root);
        let first_handle = first.root_handle();
        assert_eq!(first_handle, first.root_handle());
        assert_ne!(first_handle, second.root_handle());
        assert_ne!(first_handle, root.handle());
        assert!(first_handle.resolve_live(&root.installed_scopes()).is_err());
    }
    #[test]
    fn neutral_annotations_are_values_without_live_scope_authority() {
        use surgeist_js_ast::scope::ScopeHandle;
        let first = ScopeHandle::default();
        let second = ScopeHandle::default();
        assert_eq!(first, second);
        assert_eq!(
            first.installed_scopes().unwrap_err(),
            ScopeStateError::Unprepared
        );
        let live = Scope::new_global();
        assert_ne!(first, live.handle());
        assert_eq!(
            first.resolve_live(&live.installed_scopes()).unwrap_err(),
            ScopeStateError::Unprepared
        );
    }
    #[test]
    fn original_var_names_keep_first_declaration_order_and_function_membership() {
        use surgeist_js_ast::{
            LinearSpan, Statement, StatementListItem,
            declaration::VarDeclaration,
            function::{FormalParameterList, FunctionBody, FunctionDeclaration},
        };
        use surgeist_js_interner::Sym;
        fn vars(names: &[Sym]) -> StatementListItem {
            let variables: Vec<_> = names
                .iter()
                .map(|name| {
                    Variable::from_identifier(
                        Identifier::new(*name, Span::new((1, 1), (1, 2))),
                        None,
                    )
                })
                .collect();
            Statement::Var(VarDeclaration(variables.try_into().unwrap())).into()
        }
        fn function(name: Sym) -> StatementListItem {
            Declaration::FunctionDeclaration(FunctionDeclaration::new(
                Identifier::new(name, Span::new((1, 1), (1, 2))),
                FormalParameterList::default(),
                FunctionBody::new(StatementList::default(), Span::new((1, 1), (1, 2))),
                LinearSpan::default(),
            ))
            .into()
        }
        let mut interner = Interner::new();
        let first = interner.get_or_intern("first");
        let second = interner.get_or_intern("second");
        let declared = interner.get_or_intern("declared");
        let third = interner.get_or_intern("third");
        let script = Script::new(StatementList::new(
            vec![
                vars(&[first, second]),
                function(declared),
                vars(&[second, third]),
                vars(&[first]),
                function(declared),
            ],
            LinearPosition::default(),
            false,
        ));
        let root = Scope::new_global();
        let prepared = script.try_prepare::<()>(&root, &interner, None).unwrap();
        assert_eq!(
            prepared.declarations().var_names(),
            &[
                JsString::from("first"),
                JsString::from("second"),
                JsString::from("declared"),
                JsString::from("third")
            ]
        );
        assert!(prepared.declarations().lexical_names().is_empty());
        assert_eq!(root.num_bindings(), 0);
    }
}

//! Public composition of syntax parsing and atomic, consuming scope analysis.
use std::convert::Infallible;

use surgeist_js_ast::{
    Declaration, Script, StatementListItem,
    scope::{FunctionScopes, Scope, ScopeAnalysisError, ScopeStateError, ScopeWork},
};
use surgeist_js_interner::Interner;
use surgeist_js_parser::{Parser, Source};

fn syntax(source: &str, interner: &mut Interner) -> Script {
    Parser::new(Source::from_bytes(source))
        .parse_script_syntax_with_source(interner)
        .expect("valid script syntax")
        .0
}

fn declared_function<'a>(
    script: &'a Script,
    name: &str,
    interner: &Interner,
) -> &'a FunctionScopes {
    let symbol = interner.get(name).expect("function name was parsed");
    script
        .statements()
        .statements()
        .iter()
        .find_map(|item| match item {
            StatementListItem::Declaration(declaration) => match declaration.as_ref() {
                Declaration::FunctionDeclaration(function) if function.name().sym() == symbol => {
                    Some(function.scopes())
                }
                _ => None,
            },
            StatementListItem::Statement(_) => None,
        })
        .expect("top-level function declaration")
}

#[test]
fn controlled_scope_passes_preserve_nested_scope_semantics() {
    let source = r#"
        let globalValue = 1;
        function mapped(parameter) { return arguments[0]; }
        function lexicalThis() { return () => this; }
        function nested(parameter = globalValue) {
            let local = parameter;
            function capture() { return local; }
            for (let index = 0; index < 2; index++) { let copy = index; capture(); }
            for (let key in {x: 1}) { key; }
            for (const value of [1, 2]) { value; }
            switch (parameter) { case 1: let selected = local; selected; break; default: break; }
            try { throw parameter; } catch ({message}) { let caught = message; caught; }
            with ({external: 1}) { local; external; }
            return capture;
        }
        class Example {
            field = () => this;
            static value = globalValue;
            static { let hidden = globalValue; hidden; }
            method(parameter) { return () => parameter; }
        }
        function dynamic(parameter) { eval('parameter'); return parameter; }
        nested();
    "#;
    let mut interner = Interner::default();
    let mut ordinary = syntax(source, &mut interner);
    let ordinary_root = Scope::new_global();
    ordinary
        .analyze_scope(&ordinary_root, &interner)
        .expect("ordinary analysis");

    let controlled_root = Scope::new_global();
    let mut work = Vec::new();
    let controlled = syntax(source, &mut interner)
        .try_analyze_scope(
            &controlled_root,
            &interner,
            Some(&mut |item| {
                work.push(item);
                Ok::<_, Infallible>(())
            }),
        )
        .expect("controlled analysis");

    // Structural equality includes nested Scope bindings, indices, identities,
    // escaped-this flags and FunctionScopes metadata, on independent scope trees.
    assert_eq!(controlled, ordinary);
    assert_eq!(controlled_root, ordinary_root);
    assert!(work.contains(&ScopeWork::Publication));

    // Access to a mapped arguments object makes its parameter nonlocal, even
    // when that access exists only in the staged state during escape analysis.
    let mapped = declared_function(&controlled, "mapped", &interner);
    let installed = mapped
        .function_scope()
        .installed_scopes()
        .expect("controlled analysis published the scope association");
    assert!(
        mapped
            .arguments_object_accessed(&installed)
            .expect("mapped arguments have installed scopes")
    );
    assert!(mapped.requires_function_scope());
    let parameter_scope = mapped
        .parameter_scope()
        .resolve_live(&installed)
        .expect("parameter scope is published");
    let parameter = parameter_scope
        .get_binding_reference(&"parameter".into())
        .expect("parameter binding");
    assert!(!parameter.local());
    assert_eq!(parameter_scope.num_bindings_non_local(), 1);
    assert_eq!(parameter.locator().binding_index(), 0);

    let lexical_this = declared_function(&controlled, "lexicalThis", &interner);
    assert!(
        lexical_this
            .function_scope()
            .resolve_live(&installed)
            .expect("enclosing function scope is published")
            .escaped_this()
    );
    assert!(lexical_this.requires_function_scope());
}

#[test]
fn every_refused_work_item_preserves_published_scope_and_allows_retry() {
    let source = "let globalValue = 1; { let blockValue = globalValue; }";
    let mut interner = Interner::default();
    let mut accepted = Vec::new();
    let accepted_root = Scope::new_global();
    let _ = accepted_root.create_mutable_binding("kept".into(), false);
    syntax(source, &mut interner)
        .try_analyze_scope(
            &accepted_root,
            &interner,
            Some(&mut |work| {
                accepted.push(work);
                Ok::<_, usize>(())
            }),
        )
        .expect("complete work trace");
    assert!(!accepted.is_empty());

    for refused_at in 0..accepted.len() {
        let root = Scope::new_global();
        let kept = root.create_mutable_binding("kept".into(), false);
        let before = format!("{root:?}");
        let mut visited = 0;
        let result = syntax(source, &mut interner).try_analyze_scope(
            &root,
            &interner,
            Some(&mut |work| {
                assert_eq!(work, accepted[visited]);
                let current = visited;
                visited += 1;
                if current == refused_at {
                    Err(refused_at)
                } else {
                    Ok(())
                }
            }),
        );
        assert_eq!(visited, refused_at + 1);
        assert!(matches!(result, Err(ScopeAnalysisError::Refused(index)) if index == refused_at));
        assert_eq!(format!("{root:?}"), before);
        assert_eq!(root.get_binding(&"kept".into()), Some(kept));
        assert!(!root.has_binding(&"globalValue".into()));
        assert_eq!(root.num_bindings(), 1);
        assert_eq!(Scope::new(root.clone(), false).unique_id(), 1);

        syntax(source, &mut interner)
            .try_analyze_scope::<Infallible>(&root, &interner, None)
            .expect("same-source retry after refusal");
        assert!(root.has_binding(&"globalValue".into()));
        // One probe above and one successfully analyzed block have spent IDs.
        assert_eq!(Scope::new(root, false).unique_id(), 3);
    }
}

#[test]
fn publication_callback_allocation_conflicts_without_undoing_caller_effects() {
    for refuse in [false, true] {
        let root = Scope::new_global();
        let mut child = None;
        let mut interner = Interner::default();
        let result = syntax("let visible = 1; { let nested = visible; }", &mut interner)
            .try_analyze_scope(
                &root,
                &interner,
                Some(&mut |work| {
                    if work == ScopeWork::Publication {
                        child = Some(Scope::new(root.clone(), false));
                        if refuse {
                            return Err("publication refused");
                        }
                    }
                    Ok(())
                }),
            );
        if refuse {
            assert!(matches!(
                result,
                Err(ScopeAnalysisError::Refused("publication refused"))
            ));
        } else {
            assert!(matches!(
                result,
                Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
            ));
        }
        assert_eq!(child.expect("publication callback ran").unique_id(), 1);
        assert_eq!(root.num_bindings(), 0);
        assert!(!root.has_binding(&"visible".into()));
        assert_eq!(Scope::new(root, false).unique_id(), 2);
    }
}

#[test]
fn discarded_empty_block_spends_identity_only_after_successful_publication() {
    let mut interner = Interner::default();
    let root = Scope::new_global();
    syntax("{}", &mut interner)
        .try_analyze_scope::<Infallible>(&root, &interner, None)
        .expect("empty block succeeds");
    assert_eq!(Scope::new(root, false).unique_id(), 2);

    let root = Scope::new_global();
    let result = syntax("{}", &mut interner).try_analyze_scope(
        &root,
        &interner,
        Some(&mut |work| {
            if work == ScopeWork::Publication {
                Err("stop before publication")
            } else {
                Ok(())
            }
        }),
    );
    assert!(matches!(
        result,
        Err(ScopeAnalysisError::Refused("stop before publication"))
    ));
    assert_eq!(Scope::new(root, false).unique_id(), 1);
}

#[test]
fn controlled_preflight_preserves_duplicate_diagnostics_and_first_refusal() {
    use surgeist_js_parser::{
        Error,
        frontend::{FrontendLimit, FrontendWork},
    };

    const SOURCE: &str = "let staged; let existing;";
    for refuse in [false, true] {
        let root = Scope::new_global();
        let existing = root.create_mutable_binding("existing".into(), false);
        let mut interner = Interner::default();
        let mut control = |work| {
            if refuse && matches!(work, FrontendWork::Scope(ScopeWork::BindingProbe)) {
                Err(FrontendLimit::Scope(0))
            } else {
                Ok(())
            }
        };
        let error = Parser::with_control(Source::from_bytes(SOURCE), Some(&mut control))
            .parse_script(&root, &mut interner)
            .expect_err("duplicate declaration or earlier admission refusal");
        if refuse {
            assert_eq!(error.frontend_limit(), Some(FrontendLimit::Scope(0)));
        } else {
            assert!(matches!(
                error,
                Error::ScopeAnalysis { ref err } if err.as_ref() == "duplicate lexical declaration"
            ));
            assert_eq!(
                error.to_string(),
                "invalid scope analysis: duplicate lexical declaration"
            );
            let ordinary = Parser::new(Source::from_bytes(SOURCE))
                .parse_script(&root, &mut interner)
                .unwrap_err();
            assert_eq!(ordinary.to_string(), error.to_string());
        }
        assert_eq!(root.num_bindings(), 1);
        assert_eq!(root.get_binding(&"existing".into()), Some(existing));
        assert!(!root.has_binding(&"staged".into()));
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }
}

#[test]
fn parser_preserves_a_publication_conflict_as_scope_state() {
    use std::cell::RefCell;
    use surgeist_js_parser::{Error, frontend::FrontendWork};

    // Frontend callbacks retain their Send + Sync contract; a host can still
    // reenter the current thread's live scope without sharing it across threads.
    thread_local! {
        static LIVE_SCOPE: RefCell<Option<Scope>> = const { RefCell::new(None) };
    }
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            LIVE_SCOPE.set(None);
        }
    }

    let root = Scope::new_global();
    LIVE_SCOPE.set(Some(root.clone()));
    let _reset = Reset;
    let mut interner = Interner::default();
    let mut control = |work| {
        if matches!(work, FrontendWork::Scope(ScopeWork::Publication)) {
            LIVE_SCOPE.with_borrow(|scope| {
                drop(Scope::new(scope.as_ref().unwrap().clone(), false));
            });
        }
        Ok(())
    };
    let error = Parser::with_control(Source::from_bytes("let staged;"), Some(&mut control))
        .parse_script(&root, &mut interner)
        .unwrap_err();
    assert!(matches!(
        error,
        Error::ScopeState(ScopeStateError::Conflict)
    ));
    assert_eq!(root.num_bindings(), 0);
    assert_eq!(Scope::new(root, false).unique_id(), 2);
}

#[test]
fn late_refusal_preserves_existing_caller_bindings_and_lexical_this() {
    const SOURCE: &str =
        "kept; const arrow = () => this; function capture() { return kept + other; }";
    fn caller_chain() -> (Scope, Scope) {
        let caller = Scope::new(Scope::new_global(), true);
        let _ = caller.create_mutable_binding("kept".into(), false);
        let _ = caller.create_mutable_binding("other".into(), false);
        let supplied = Scope::new(caller.clone(), false);
        (caller, supplied)
    }
    let mut interner = Interner::default();
    let (caller, supplied) = caller_chain();
    let before_caller = format!("{caller:?}");
    let before_supplied = format!("{supplied:?}");
    let before_kept = caller.get_binding_reference(&"kept".into()).unwrap();
    let before_other = caller.get_binding_reference(&"other".into()).unwrap();
    assert!(before_kept.local());
    assert!(before_other.local());
    assert!(!caller.escaped_this());

    let refused = syntax(SOURCE, &mut interner).try_analyze_scope(
        &supplied,
        &interner,
        Some(&mut |work| {
            if work == ScopeWork::Publication {
                Err("late refusal")
            } else {
                Ok(())
            }
        }),
    );
    assert!(matches!(
        refused,
        Err(ScopeAnalysisError::Refused("late refusal"))
    ));
    assert_eq!(format!("{caller:?}"), before_caller);
    assert_eq!(format!("{supplied:?}"), before_supplied);
    assert_eq!(
        caller.get_binding_reference(&"kept".into()),
        Some(before_kept)
    );
    assert_eq!(
        caller.get_binding_reference(&"other".into()),
        Some(before_other)
    );
    assert!(!caller.escaped_this());

    let retried = syntax(SOURCE, &mut interner)
        .try_analyze_scope::<Infallible>(&supplied, &interner, None)
        .expect("retry publishes captured bindings and lexical this");
    assert!(
        !caller
            .get_binding_reference(&"kept".into())
            .unwrap()
            .local()
    );
    assert!(
        !caller
            .get_binding_reference(&"other".into())
            .unwrap()
            .local()
    );
    assert!(caller.escaped_this());

    let (ordinary_caller, ordinary_supplied) = caller_chain();
    let mut ordinary = syntax(SOURCE, &mut interner);
    ordinary
        .analyze_scope(&ordinary_supplied, &interner)
        .unwrap();
    assert_eq!(retried, ordinary);
    assert_eq!(caller, ordinary_caller);
    assert_eq!(supplied, ordinary_supplied);
    assert_eq!(
        Scope::new(supplied, false).unique_id(),
        Scope::new(ordinary_supplied, false).unique_id()
    );
}

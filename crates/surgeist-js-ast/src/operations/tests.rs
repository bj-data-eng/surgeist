use surgeist_js_interner::{Interner, Sym};

use crate::{
    Expression, LinearPosition, Position, Script, Span, Statement, StatementList,
    declaration::{Declaration, LexicalDeclaration, VarDeclaration, Variable, VariableList},
    expression::{
        Call, Identifier, NewTarget, Parenthesized, SuperCall, TaggedTemplate, This,
        literal::{ArrayLiteral, ObjectLiteral, PropertyDefinition},
    },
    function::{FormalParameterList, FunctionBody, FunctionExpression},
    operations::{
        CheckLabelsError, CheckLabelsFailure, ContainsSymbol, LabelCheckWork, NameWork,
        VarScopedDeclaration, check_labels, contains, contains_invalid_object_literal,
        try_bound_names, try_contains, try_contains_arguments, try_has_direct_super,
        try_lexically_declared_names, try_lexically_scoped_declarations,
        try_var_scoped_declarations, var_scoped_declarations,
    },
    statement::{
        Block, Case, Switch, With,
        iteration::{Break, ForInLoop, ForOfLoop, IterableLoopInitializer},
    },
};

fn empty_span() -> Span {
    Span::new(Position::new(1, 1), Position::new(1, 1))
}

#[test]
fn var_scoped_declarations_borrow_the_original_variable_nodes() {
    fn same_declaration(candidate: &Variable, original: &Variable) -> bool {
        std::ptr::eq(candidate, original)
    }

    let declaration = VarDeclaration(
        VariableList::new(Box::new([
            Variable::from_identifier(Identifier::new(Sym::EVAL, empty_span()), None),
            Variable::from_identifier(Identifier::new(Sym::ARGUMENTS, empty_span()), None),
        ]))
        .unwrap(),
    );
    let declarations = var_scoped_declarations(&declaration);
    let [
        VarScopedDeclaration::VariableDeclaration(first),
        VarScopedDeclaration::VariableDeclaration(second),
    ] = declarations.as_slice()
    else {
        panic!("expected two variable declarations");
    };
    let originals = declaration.0.as_ref();
    assert!(same_declaration(first, &originals[0]));
    assert!(same_declaration(second, &originals[1]));
}

#[test]
fn scoped_declaration_collectors_refuse_before_recording_a_borrowed_node() {
    let list = || {
        VariableList::new(Box::new([Variable::from_identifier(
            Identifier::new(Sym::EVAL, empty_span()),
            None,
        )]))
        .unwrap()
    };
    let variable = VarDeclaration(list());
    let script = Script::new(StatementList::from((
        vec![
            Declaration::from(LexicalDeclaration::Let(list())).into(),
            Statement::Empty.into(),
        ],
        LinearPosition::new(0),
    )));

    let mut attempted = Vec::new();
    let mut refuse_record = |work| {
        attempted.push(work);
        if work == NameWork::Record {
            Err("record refused")
        } else {
            Ok(())
        }
    };
    assert!(matches!(
        try_var_scoped_declarations(&variable, Some(&mut refuse_record)),
        Err("record refused")
    ));
    assert_eq!(
        attempted,
        [NameWork::Visit, NameWork::Visit, NameWork::Record]
    );

    attempted.clear();
    let mut admit = |work| {
        attempted.push(work);
        Ok::<_, ()>(())
    };
    let lexical = try_lexically_scoped_declarations(&script, Some(&mut admit)).unwrap();
    assert_eq!(lexical[0].bound_names(), [Sym::EVAL]);
    assert_eq!(
        attempted,
        [
            NameWork::Visit,
            NameWork::Visit,
            NameWork::Visit,
            NameWork::Record,
            NameWork::Visit
        ]
    );
}

#[test]
fn scoped_declarations_admit_each_case_in_the_default_search() {
    let switch = Switch::new(
        This::new(empty_span()).into(),
        Box::new([
            Case::new(This::new(empty_span()).into(), StatementList::default()),
            Case::default(StatementList::default()),
        ]),
    );
    let mut accepted = 0;
    let result = try_var_scoped_declarations(
        &switch,
        Some(&mut |work| {
            assert_eq!(work, NameWork::Visit);
            // The switch, both cases and their bodies, then both entries in the default search.
            if accepted == 7 {
                return Err("default body refused");
            }
            accepted += 1;
            Ok(())
        }),
    );
    assert_eq!(accepted, 7);
    assert!(matches!(result, Err("default body refused")));
}

#[test]
fn scoped_declarations_admit_for_in_and_for_of_initializers_before_recording() {
    let initializer = || {
        IterableLoopInitializer::Var(Variable::from_identifier(
            Identifier::new(Sym::EVAL, empty_span()),
            None,
        ))
    };
    let target = || This::new(empty_span()).into();
    let for_in = ForInLoop::new(initializer(), target(), Statement::Empty);
    let for_of = ForOfLoop::new(initializer(), target(), Statement::Empty, false);
    for node in [Statement::ForInLoop(for_in), Statement::ForOfLoop(for_of)] {
        let mut attempts = Vec::new();
        let result = try_var_scoped_declarations(
            &node,
            Some(&mut |work| {
                attempts.push(work);
                if work == NameWork::Record {
                    Err("record refused")
                } else {
                    Ok(())
                }
            }),
        );
        assert!(matches!(result, Err("record refused")));
        assert_eq!(
            attempts,
            [
                NameWork::Visit,
                NameWork::Visit,
                NameWork::Visit,
                NameWork::Record
            ]
        );
    }
}

#[test]
fn bound_name_collection_admits_inspection_before_recording() {
    let identifier = Identifier::new(Sym::EVAL, empty_span());
    let mut attempted = Vec::new();
    let refused = try_bound_names(
        &identifier,
        Some(&mut |work| {
            attempted.push(work);
            if work == NameWork::Record {
                Err("record refused")
            } else {
                Ok(())
            }
        }),
    );
    assert_eq!(refused, Err("record refused"));
    assert_eq!(attempted, [NameWork::Visit, NameWork::Record]);
    assert_eq!(
        try_bound_names::<_, ()>(&identifier, None),
        Ok(vec![Sym::EVAL])
    );
}

#[test]
fn lexical_name_collection_admits_empty_statement_slots() {
    let script = Script::new(StatementList::from((
        vec![Statement::Empty.into(), Statement::Empty.into()],
        LinearPosition::new(0),
    )));
    let mut attempted = Vec::new();
    let result = try_lexically_declared_names(
        &script,
        Some(&mut |work| {
            attempted.push(work);
            Ok::<_, ()>(())
        }),
    );
    assert_eq!(result, Ok(Vec::new()));
    assert_eq!(attempted, [NameWork::Visit; 3]);
}

#[test]
fn callable_metadata_refuses_before_construction_and_admits_the_match_exactly() {
    for limit in [6, 7] {
        let call = Call::new(
            Identifier::new(Sym::EVAL, empty_span()).into(),
            Box::default(),
            empty_span(),
        );
        let body = FunctionBody::new(
            StatementList::from((
                vec![Statement::Expression(call.into()).into()],
                LinearPosition::new(0),
            )),
            empty_span(),
        );
        let mut visits = 0;
        let mut admit = || {
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        };
        let node = FunctionExpression::try_new(
            None,
            FormalParameterList::default(),
            body,
            None,
            false,
            empty_span(),
            Some(&mut admit),
        );
        assert_eq!(visits, limit);
        if limit == 6 {
            assert_eq!(node.unwrap_err(), "refused");
        } else {
            assert!(node.unwrap().contains_direct_eval);
        }
    }
}

#[test]
fn switch_metadata_continues_to_the_default_case_after_a_match() {
    for limit in [4, 6] {
        let call = Call::new(
            Identifier::new(Sym::EVAL, empty_span()).into(),
            Box::default(),
            empty_span(),
        );
        let cases = Box::new([
            Case::new(call.into(), StatementList::default()),
            Case::default(StatementList::default()),
        ]);
        let mut visits = 0;
        let mut admit = || {
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        };
        let node = Switch::try_new(This::new(empty_span()).into(), cases, Some(&mut admit));
        assert_eq!(visits, limit);
        if limit == 4 {
            assert_eq!(node.unwrap_err(), "refused");
        } else {
            assert!(node.unwrap().contains_direct_eval);
        }
    }
}

#[test]
fn empty_block_metadata_admits_the_statement_list() {
    let mut visits = 0;
    let mut admit = || {
        visits += 1;
        Err("refused")
    };
    let result = Block::try_from_statement_list(StatementList::default(), Some(&mut admit));
    assert_eq!(result.unwrap_err(), "refused");
    assert_eq!(visits, 1);
}

#[test]
fn contains_distinguishes_nested_function_pruning_from_a_concrete_root() {
    let function = FunctionExpression::new(
        None,
        FormalParameterList::default(),
        FunctionBody::new(
            StatementList::from((
                vec![Statement::Expression(This::new(empty_span()).into()).into()],
                LinearPosition::new(0),
            )),
            empty_span(),
        ),
        None,
        false,
        empty_span(),
    );
    let nested: Expression = function.clone().into();
    let mut visits = 0;
    assert_eq!(
        try_contains(&nested, ContainsSymbol::This, || {
            visits += 1;
            Ok::<(), std::convert::Infallible>(())
        }),
        Ok(false)
    );
    assert_eq!(visits, 2);
    let mut visits = 0;
    assert_eq!(
        try_contains(&function, ContainsSymbol::This, || {
            visits += 1;
            Ok::<(), std::convert::Infallible>(())
        }),
        Ok(true)
    );
    assert_eq!(visits, 7);
}

#[test]
fn contains_direct_eval_admits_every_callee_candidate_before_matching() {
    let eval = Identifier::new(Sym::EVAL, empty_span());
    let callee = Parenthesized::new(
        Parenthesized::new(eval.into(), empty_span()).into(),
        empty_span(),
    );
    let node: Expression = Call::new(callee.into(), Box::default(), empty_span()).into();
    for limit in [4, 5] {
        let mut visits = 0;
        let result = try_contains(&node, ContainsSymbol::DirectEval, || {
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        });
        assert_eq!(visits, limit);
        assert_eq!(result, if limit == 5 { Ok(true) } else { Err("refused") });
    }
}

#[test]
fn contains_arguments_stops_before_later_siblings() {
    let mut interner = Interner::new();
    let callee = Identifier::new(interner.get_or_intern("f"), empty_span());
    let arg = Identifier::new(Sym::ARGUMENTS, empty_span());
    let call = Call::new(
        callee.into(),
        Box::new([arg.into(), arg.into()]),
        empty_span(),
    );
    for limit in [4, 5] {
        let mut visits = 0;
        let result = try_contains_arguments(&call, || {
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        });
        assert_eq!(visits, limit);
        assert_eq!(result, if limit == 5 { Ok(true) } else { Err("refused") });
    }
}

#[test]
fn has_direct_super_shares_admission_between_parameters_and_body() {
    let call = SuperCall::new(Box::<[Expression]>::default(), empty_span());
    let body = FunctionBody::new(
        StatementList::from((
            vec![Statement::Expression(call.into()).into()],
            LinearPosition::new(0),
        )),
        empty_span(),
    );
    for limit in [5, 6] {
        let mut visits = 0;
        let result = try_has_direct_super(&FormalParameterList::default(), &body, || {
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        });
        assert_eq!(visits, limit);
        assert_eq!(result, if limit == 6 { Ok(true) } else { Err("refused") });
    }
}

#[test]
fn label_lookup_refusal_precedes_the_undefined_target_diagnostic() {
    let target = Interner::new().get_or_intern("missing");
    let node = Statement::Break(Break::new(Some(target)));
    for refused in [false, true] {
        let mut events = Vec::new();
        let result = check_labels(&node, |work| {
            events.push(work);
            if refused && work == LabelCheckWork::Lookup {
                Err("refused")
            } else {
                Ok(())
            }
        });
        assert_eq!(
            events,
            [
                LabelCheckWork::Visit,
                LabelCheckWork::Visit,
                LabelCheckWork::Lookup
            ]
        );
        if refused {
            assert!(matches!(
                result,
                Err(CheckLabelsFailure::Refused("refused"))
            ));
        } else {
            assert!(
                matches!(result, Err(CheckLabelsFailure::Syntax(CheckLabelsError::UndefinedBreakTarget(label))) if label == target)
            );
        }
    }
}

#[test]
fn label_walk_charges_each_switch_case_and_body_once() {
    let node = Statement::Switch(Switch::new(
        This::new(empty_span()).into(),
        Box::new([
            Case::new(This::new(empty_span()).into(), StatementList::default()),
            Case::default(StatementList::default()),
        ]),
    ));
    for limit in [4, 6] {
        let mut visits = 0;
        let result = check_labels(&node, |work| {
            assert_eq!(work, LabelCheckWork::Visit);
            if visits == limit {
                return Err("refused");
            }
            visits += 1;
            Ok(())
        });
        assert_eq!(visits, limit);
        if limit == 4 {
            assert!(matches!(
                result,
                Err(CheckLabelsFailure::Refused("refused"))
            ));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn invalid_object_walk_admits_before_the_match_and_stops_there() {
    let name = Interner::new().get_or_intern("a");
    let property = PropertyDefinition::CoverInitializedName(
        Identifier::new(name, empty_span()),
        This::new(empty_span()).into(),
    );
    let node: Expression = ObjectLiteral::new([property.clone(), property], empty_span()).into();
    for limit in [0, 2, 3] {
        let mut admitted = 0;
        let result = contains_invalid_object_literal(&node, || {
            if admitted == limit {
                return Err("refused");
            }
            admitted += 1;
            Ok(())
        });
        assert_eq!(admitted, limit);
        assert_eq!(result, if limit == 3 { Ok(true) } else { Err("refused") });
    }
}

#[test]
fn invalid_object_walk_charges_each_empty_array_slot() {
    let node = ArrayLiteral::new([None, None], false, empty_span());
    let mut admitted = 0;
    let result = contains_invalid_object_literal(&node, || {
        if admitted == 2 {
            return Err("refused");
        }
        admitted += 1;
        Ok(())
    });
    assert_eq!(result, Err("refused"));
    assert_eq!(admitted, 2);
}

#[test]
fn invalid_object_walk_charges_a_missing_cooked_template_segment() {
    let raw = Interner::new().get_or_intern("\\u");
    let node = TaggedTemplate::new(
        This::new(empty_span()).into(),
        Box::new([raw]),
        Box::new([None]),
        Box::default(),
        0,
        empty_span(),
    );
    let mut admitted = 0;
    let result = contains_invalid_object_literal(&node, || {
        if admitted == 4 {
            return Err("refused");
        }
        admitted += 1;
        Ok(())
    });
    assert_eq!(result, Err("refused"));
    assert_eq!(admitted, 4);
}

#[test]
fn check_contains_this_in_with_statement_expression() {
    let node = With::new(This::new(empty_span()).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_with_statement_expression() {
    let node = With::new(NewTarget::new(empty_span()).into(), Statement::Empty);
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_new_target_in_call_function_position() {
    let node = Call::new(
        NewTarget::new(empty_span()).into(),
        Box::default(),
        empty_span(),
    );
    assert!(contains(&node, ContainsSymbol::NewTarget));
}

#[test]
fn check_contains_this_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![This::new(empty_span()).into()].into_boxed_slice(),
        empty_span(),
    );

    assert!(contains(&node, ContainsSymbol::This));
}

#[test]
fn check_contains_new_target_in_call_argument_position() {
    let mut interner = Interner::new();
    let function_name = Identifier::new(interner.get_or_intern("func"), Span::new((1, 1), (1, 5)));
    let node = Call::new(
        function_name.into(),
        vec![NewTarget::new(empty_span()).into()].into_boxed_slice(),
        empty_span(),
    );

    assert!(contains(&node, ContainsSymbol::NewTarget));
}

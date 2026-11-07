use surgeist_js_interner::{Interner, Sym};

use crate::{
    Expression, LinearPosition, LinearSpan, Position, Script, Span, Statement, StatementList,
    StatementListItem,
    declaration::{Declaration, LexicalDeclaration, VarDeclaration, Variable, VariableList},
    expression::{
        Call, Identifier, NewTarget, Parenthesized, SuperCall, TaggedTemplate, This,
        literal::{ArrayLiteral, ObjectLiteral, PropertyDefinition},
    },
    function::{FormalParameterList, FunctionBody, FunctionDeclaration, FunctionExpression},
    operations::{
        CheckLabelsError, CheckLabelsFailure, ContainsSymbol, LabelCheckWork, NameWork,
        VarScopedDeclaration, annex_b_function_declarations_names, check_labels, contains,
        contains_invalid_object_literal, try_annex_b_function_declarations_names, try_bound_names,
        try_contains, try_contains_arguments, try_has_direct_super, try_lexically_declared_names,
        try_lexically_scoped_declarations, try_var_scoped_declarations, var_scoped_declarations,
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

fn annex_function(name: Sym, body: StatementList) -> StatementListItem {
    Declaration::FunctionDeclaration(FunctionDeclaration::new(
        Identifier::new(name, empty_span()),
        FormalParameterList::default(),
        FunctionBody::new(body, empty_span()),
        LinearSpan::new(LinearPosition::new(0), LinearPosition::new(0)),
    ))
    .into()
}

fn annex_list(items: Vec<StatementListItem>) -> StatementList {
    StatementList::from((items, LinearPosition::new(0)))
}

#[test]
fn annex_block_admits_raw_scan_nested_names_comparison_and_writes_before_work() {
    use NameWork::{Comparison as C, Record as R, Visit as V};
    let node = Block::from(annex_list(vec![annex_function(
        Sym::EVAL,
        StatementList::default(),
    )]));
    // Block/list/item, raw declaration scan/record, legacy lexical-name walk,
    // candidate read, symbol comparison, retained write and final truncation.
    let expected = [V, V, V, V, R, V, V, V, V, V, R, V, C, R, R];
    for limit in 0..=expected.len() {
        let mut attempted = Vec::new();
        let result = try_annex_b_function_declarations_names(
            &node,
            Some(&mut |work| {
                attempted.push(work);
                if attempted.len() > limit {
                    Err(limit)
                } else {
                    Ok(())
                }
            }),
        );
        if limit == expected.len() {
            assert_eq!(result, Ok(vec![Sym::EVAL]));
            assert_eq!(attempted, expected);
        } else {
            assert_eq!(result, Err(limit));
            assert_eq!(attempted, expected[..=limit]);
        }
    }
    assert_eq!(annex_b_function_declarations_names(&node), [Sym::EVAL]);
}

#[test]
fn annex_switch_admits_the_second_default_search_and_body() {
    use NameWork::{Record as R, Visit as V};
    let node = Switch::new(
        This::new(empty_span()).into(),
        Box::new([Case::default(StatementList::default())]),
    );
    // Switch, raw case, typed case/body, second raw search, repeated default body,
    // legacy lexical walk (switch/value/case/body), final truncation.
    let expected = [V, V, V, V, V, V, V, V, V, V, R];
    for limit in 0..=expected.len() {
        let mut attempted = Vec::new();
        let result = try_annex_b_function_declarations_names(
            &node,
            Some(&mut |work| {
                attempted.push(work);
                if attempted.len() > limit {
                    Err(limit)
                } else {
                    Ok(())
                }
            }),
        );
        assert_eq!(
            result,
            if limit == expected.len() {
                Ok(vec![])
            } else {
                Err(limit)
            }
        );
        assert_eq!(attempted, expected[..expected.len().min(limit + 1)]);
    }
}

#[test]
fn annex_collection_preserves_default_duplicates_filters_and_function_pruning() {
    let hidden = Block::from(annex_list(vec![annex_function(
        Sym::ARGUMENTS,
        StatementList::default(),
    )]));
    let default = Case::default(annex_list(vec![annex_function(
        Sym::EVAL,
        annex_list(vec![Statement::Block(hidden).into()]),
    )]));
    let node = Switch::new(This::new(empty_span()).into(), Box::new([default]));
    assert_eq!(
        try_annex_b_function_declarations_names::<_, ()>(&node, None),
        Ok(vec![Sym::EVAL, Sym::EVAL])
    );
    assert_eq!(
        annex_b_function_declarations_names(&node),
        [Sym::EVAL, Sym::EVAL]
    );

    // A non-function lexical declaration removes every matching collected name,
    // including names collected from nested blocks before this outer filter.
    let nested = Block::from(annex_list(vec![
        annex_function(Sym::EVAL, StatementList::default()),
        annex_function(Sym::ARGUMENTS, StatementList::default()),
        annex_function(Sym::EVAL, StatementList::default()),
    ]));
    let lexical = Declaration::from(LexicalDeclaration::Let(
        VariableList::new(Box::new([Variable::from_identifier(
            Identifier::new(Sym::EVAL, empty_span()),
            None,
        )]))
        .unwrap(),
    ));
    let filtered = Block::from(annex_list(vec![
        Statement::Block(nested).into(),
        lexical.into(),
    ]));
    assert_eq!(
        try_annex_b_function_declarations_names::<_, ()>(&filtered, None),
        Ok(vec![Sym::ARGUMENTS])
    );
}

#[test]
fn annex_loop_admits_initializer_bound_names_and_filtering_in_order() {
    use NameWork::{Comparison as C, Record as R, Visit as V};
    let body = Block::from(annex_list(vec![annex_function(
        Sym::EVAL,
        StatementList::default(),
    )]));
    let node = ForInLoop::new(
        IterableLoopInitializer::Let(Identifier::new(Sym::EVAL, empty_span()).into()),
        This::new(empty_span()).into(),
        Statement::Block(body),
    );
    // Loop/body statement; full block collection; first initializer inspection,
    // bound binding/identifier/record; candidate/comparison/truncation; second
    // initializer inspection (the existing independent const test).
    let expected = [
        V, V, V, V, V, V, R, V, V, V, V, V, R, V, C, R, R, V, V, V, R, V, C, R, V,
    ];
    for limit in 0..=expected.len() {
        let mut attempted = Vec::new();
        let result = try_annex_b_function_declarations_names(
            &node,
            Some(&mut |work| {
                attempted.push(work);
                if attempted.len() > limit {
                    Err(limit)
                } else {
                    Ok(())
                }
            }),
        );
        assert_eq!(
            result,
            if limit == expected.len() {
                Ok(vec![])
            } else {
                Err(limit)
            }
        );
        assert_eq!(attempted, expected[..expected.len().min(limit + 1)]);
    }
    assert!(annex_b_function_declarations_names(&node).is_empty());
}

#[test]
fn completion_search_admits_exactly_the_inspected_prefix_and_refuses_one_short() {
    use crate::operations::{returns_value, try_returns_value};
    let block = Block::from(annex_list(vec![
        annex_function(Sym::EVAL, StatementList::default()),
        Statement::Empty.into(),
        Statement::Expression(This::new(empty_span()).into()).into(),
        Statement::Block(Block::from(annex_list(vec![Statement::Empty.into()]))).into(),
    ]));
    // Block; raw declaration (pruned); raw empty + typed empty;
    // raw expression statement + typed expression statement (found).
    let expected = [NameWork::Visit; 6];
    let mut seen = Vec::new();
    let mut exact = |work| {
        seen.push(work);
        if seen.len() > expected.len() {
            Err("past first match")
        } else {
            Ok(())
        }
    };
    assert_eq!(try_returns_value(&block, Some(&mut exact)), Ok(true));
    assert_eq!(seen, expected);
    assert!(returns_value(&block));
    assert_eq!(try_returns_value::<_, ()>(&block, None), Ok(true));

    let mut attempted = 0;
    let mut one_short = |work| {
        assert_eq!(work, NameWork::Visit);
        attempted += 1;
        if attempted == expected.len() {
            Err("last inspection refused")
        } else {
            Ok(())
        }
    };
    assert_eq!(
        try_returns_value(&block, Some(&mut one_short)),
        Err("last inspection refused")
    );
    assert_eq!(attempted, expected.len());
}

#[test]
fn completion_search_skips_labelled_functions_and_case_conditions() {
    use crate::{
        operations::{returns_value, try_returns_value},
        statement::{Labelled, LabelledItem},
    };
    let function = FunctionDeclaration::new(
        Identifier::new(Sym::EVAL, empty_span()),
        FormalParameterList::default(),
        FunctionBody::new(
            annex_list(vec![
                Statement::Expression(This::new(empty_span()).into()).into(),
            ]),
            empty_span(),
        ),
        LinearSpan::new(LinearPosition::new(0), LinearPosition::new(0)),
    );
    let labelled = Labelled::new(LabelledItem::FunctionDeclaration(function), Sym::ARGUMENTS);
    let mut visited = 0;
    let mut admit = |work| {
        assert_eq!(work, NameWork::Visit);
        visited += 1;
        Ok::<_, ()>(())
    };
    assert_eq!(try_returns_value(&labelled, Some(&mut admit)), Ok(false));
    assert_eq!(visited, 2); // Typed labelled node, then its raw item only.
    assert!(!returns_value(&labelled));

    let case = Case::new(
        This::new(empty_span()).into(),
        annex_list(vec![
            annex_function(Sym::EVAL, StatementList::default()),
            Statement::Empty.into(),
        ]),
    );
    let mut visited = 0;
    let mut admit = |work| {
        assert_eq!(work, NameWork::Visit);
        visited += 1;
        Ok::<_, ()>(())
    };
    assert_eq!(try_returns_value(&case, Some(&mut admit)), Ok(false));
    assert_eq!(visited, 4); // Case, raw declaration, raw empty, typed empty.
    assert!(!returns_value(&case));
}

#[test]
fn completion_search_traverses_nested_labelled_blocks_without_visiting_var_initializers() {
    use crate::{
        operations::{returns_value, try_returns_value},
        statement::{Labelled, LabelledItem},
    };
    let variable = Statement::Var(VarDeclaration(
        VariableList::new(Box::new([Variable::from_identifier(
            Identifier::new(Sym::EVAL, empty_span()),
            Some(This::new(empty_span()).into()),
        )]))
        .unwrap(),
    ));
    let labelled = Labelled::new(
        LabelledItem::Statement(Statement::Block(Block::from(annex_list(vec![
            variable.into(),
            Statement::Empty.into(),
        ])))),
        Sym::ARGUMENTS,
    );
    let mut visited = 0;
    let mut admit = |work| {
        assert_eq!(work, NameWork::Visit);
        visited += 1;
        Ok::<_, ()>(())
    };
    assert_eq!(try_returns_value(&labelled, Some(&mut admit)), Ok(false));
    assert_eq!(visited, 8); // Labelled + item + statement + block + two (raw, typed) statements.
    assert!(!returns_value(&labelled));
}

#[test]
fn completion_search_preserves_nonclone_refusal_identity_and_stops_immediately() {
    use crate::operations::try_returns_value;
    #[derive(Debug)]
    struct Stop;
    let original = Stop;
    let block = Block::from(annex_list(vec![
        Statement::Empty.into(),
        Statement::Expression(This::new(empty_span()).into()).into(),
    ]));
    let mut attempted = 0;
    let mut refuse = |_| {
        attempted += 1;
        if attempted == 2 {
            Err(&original)
        } else {
            Ok(())
        }
    };
    let reason = try_returns_value(&block, Some(&mut refuse)).expect_err("raw candidate refused");
    assert!(std::ptr::eq(reason, &original));
    assert_eq!(attempted, 2);
}

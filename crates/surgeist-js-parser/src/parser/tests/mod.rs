//! Tests for the parser.

mod format;

use std::convert::TryInto;

use super::name_in_lexically_declared_names;
use crate::{
    Parser, Source,
    frontend::{FrontendLimit, FrontendWork},
};
use indoc::indoc;
use surgeist_js_ast::{
    Expression, LinearPosition, LinearSpan, Module, ModuleItem, ModuleItemList, Position, Script,
    Span, Statement, StatementList, StatementListItem,
    declaration::{Declaration, LexicalDeclaration, VarDeclaration, Variable},
    expression::{
        Call, Identifier, New, Parenthesized,
        access::SimplePropertyAccess,
        literal::{Literal, ObjectLiteral, PropertyDefinition},
        operator::{
            Assign, Binary, Update,
            assign::AssignOp,
            binary::{ArithmeticOp, BinaryOp, LogicalOp, RelationalOp},
            update::{UpdateOp, UpdateTarget},
        },
    },
    function::{
        ArrowFunction, FormalParameter, FormalParameterList, FormalParameterListFlags,
        FunctionBody, FunctionDeclaration,
    },
    scope::Scope,
    statement::{Block, If, Return, With},
};
use surgeist_js_interner::Interner;
use surgeist_js_macros::utf16;

const PSEUDO_LINEAR_POS: LinearPosition = LinearPosition::new(0);
const EMPTY_LINEAR_SPAN: LinearSpan = LinearSpan::new(PSEUDO_LINEAR_POS, PSEUDO_LINEAR_POS);

#[test]
fn parameter_name_conflict_stops_at_the_first_matching_comparison() {
    let mut interner = Interner::default();
    let a = interner.get_or_intern("a");
    let b = interner.get_or_intern("b");
    let x = interner.get_or_intern("x");
    let y = interner.get_or_intern("y");

    // `a` misses three times. `b` misses `x` and then matches.
    for limit in [4, 5] {
        let mut comparisons = 0;
        let mut control = |work| {
            assert_eq!(work, FrontendWork::NameComparison);
            if comparisons == limit {
                return Err(FrontendLimit::NameComparison(limit));
            }
            comparisons += 1;
            Ok(())
        };
        let mut parser = Parser::with_control(Source::from_bytes(""), Some(&mut control));
        let error = name_in_lexically_declared_names(
            &[a, b],
            &[x, b, y],
            Position::new(1, 1),
            &interner,
            &mut parser.cursor,
        )
        .unwrap_err();
        assert_eq!(comparisons, limit);
        if limit == 4 {
            assert_eq!(
                error.frontend_limit(),
                Some(FrontendLimit::NameComparison(4))
            );
        } else {
            assert!(error.to_string().contains("formal parameter `b` declared"));
        }
    }
}

#[test]
fn module_rejects_a_duplicate_export_only_after_admitting_its_insertion() {
    let input = "export { x } from 'm'; export { x } from 'n';";
    for limit in [1, 2] {
        let mut comparisons = 0;
        let mut control = |work| {
            if work == FrontendWork::NameComparison {
                if comparisons == limit {
                    return Err(FrontendLimit::NameComparison(limit));
                }
                comparisons += 1;
            }
            Ok(())
        };
        let error = Parser::with_control(Source::from_bytes(input), Some(&mut control))
            .parse_module(&Scope::new_global(), &mut Interner::default())
            .unwrap_err();
        assert_eq!(comparisons, limit);
        if limit == 1 {
            assert_eq!(
                error.frontend_limit(),
                Some(FrontendLimit::NameComparison(1))
            );
        } else {
            assert!(
                error
                    .to_string()
                    .contains("exported name `x` declared multiple times")
            );
        }
    }
}

#[test]
fn direct_eval_metadata_has_an_exact_construction_budget() {
    for (input, expected) in [
        ("{}", 1),
        ("{ eval('1'); }", 6),
        // Empty parameters cost one visit; the body matches at the callee inspection.
        ("function f(){eval('1');}", 7),
        ("({m(){}});", 2),
        ("class C { m() {} }", 2),
        // The first case matches after four visits, then the default costs two.
        ("switch(0){case eval('1'): default:}", 4 + 2),
        // The loop body matches after four visits, then the condition costs two.
        ("for(;this;)eval('1');", 4 + 2),
        ("for(x in eval('1'));", 3 + 2 + 1),
        ("for(x of eval('1'));", 3 + 2 + 1),
        // Each empty block costs one visit, then Catch walks its block and statement list.
        ("try{}catch{}", 1 + 1 + 2),
        #[cfg(feature = "annex-b")]
        // Each declaration costs two root visits, then its synthetic block visits four nodes.
        (
            "if (true) function f() {} else function g() {}",
            2 * (2 + 4),
        ),
    ] {
        for limit in [expected - 1, expected] {
            let mut visits = 0;
            let mut control = |work| {
                if work == FrontendWork::DirectEvalVisit {
                    if visits == limit {
                        return Err(FrontendLimit::DirectEvalVisit(limit));
                    }
                    visits += 1;
                }
                Ok(())
            };
            let result = Parser::with_control(Source::from_bytes(input), Some(&mut control))
                .parse_script(&Scope::new_global(), &mut Interner::default());
            assert_eq!(visits, limit, "{input}");
            if limit == expected {
                result.unwrap_or_else(|error| panic!("{input}: {error}"));
            } else {
                assert_eq!(
                    result.unwrap_err().frontend_limit(),
                    Some(FrontendLimit::DirectEvalVisit(limit)),
                    "{input}"
                );
            }
        }
    }
}

#[test]
fn cover_form_walk_refusal_precedes_the_syntax_diagnostic() {
    // The label check visits the root, item, and statement. Each preceding
    // Contains or ContainsArguments walk visits eleven nodes of the object fixture.
    for (entry, input, visits_before_cover) in [
        ("script", "({x = 1});", 3 + 2 * 11),
        ("function body", "({x = 1});", 3),
        ("module", "({x = 1});", 3),
        ("script", "class C { static { ({x = 1}); } }", 3 + 3 * 11),
    ] {
        for refused in [false, true] {
            let mut visits = 0;
            let mut control = |work| {
                if work == FrontendWork::EarlyErrorVisit {
                    if refused && visits == visits_before_cover {
                        return Err(FrontendLimit::EarlyErrorVisit(visits_before_cover));
                    }
                    visits += 1;
                }
                Ok(())
            };
            let mut parser = Parser::with_control(Source::from_bytes(input), Some(&mut control));
            let scope = Scope::new_global();
            let mut interner = Interner::default();
            let error = match entry {
                "script" => parser.parse_script(&scope, &mut interner).unwrap_err(),
                "function body" => parser
                    .parse_function_body(&mut interner, false, false)
                    .unwrap_err(),
                "module" => parser.parse_module(&scope, &mut interner).unwrap_err(),
                _ => unreachable!(),
            };
            if refused {
                assert_eq!(
                    error.frontend_limit(),
                    Some(FrontendLimit::EarlyErrorVisit(visits_before_cover)),
                    "{input}"
                );
            } else {
                assert!(
                    error.to_string().contains("invalid object literal"),
                    "{input}: {error}"
                );
            }
        }
    }
}

#[test]
fn predicate_refusal_precedes_the_syntax_diagnostic() {
    for (input, visits_to_match, diagnostic) in [
        // Both Contains searches visit the list, item, statement, expression, and new.target.
        ("new.target;", 2 * 5, "invalid new.target usage"),
        // The three label visits precede five visits by ContainsArguments.
        (
            "class C { static { arguments; } }",
            3 + 5,
            "'arguments' not allowed in class static block",
        ),
        // Labels and cover forms cost three and five visits. HasDirectSuper
        // visits the empty parameter list once, then visits the body through the super call.
        (
            "({ m() { super(); } });",
            3 + 5 + 1 + 5,
            "invalid super call usage",
        ),
    ] {
        for limit in [visits_to_match - 1, visits_to_match] {
            let mut visits = 0;
            let mut control = |work| {
                if work == FrontendWork::EarlyErrorVisit {
                    if visits == limit {
                        return Err(FrontendLimit::EarlyErrorVisit(limit));
                    }
                    visits += 1;
                }
                Ok(())
            };
            let error = Parser::with_control(Source::from_bytes(input), Some(&mut control))
                .parse_script(&Scope::new_global(), &mut Interner::default())
                .unwrap_err();
            assert_eq!(visits, limit, "{input}");
            if limit < visits_to_match {
                assert_eq!(
                    error.frontend_limit(),
                    Some(FrontendLimit::EarlyErrorVisit(limit)),
                    "{input}"
                );
            } else {
                assert!(error.to_string().contains(diagnostic), "{input}: {error}");
            }
        }
    }
}

#[test]
fn label_resolution_preserves_active_chains_and_statement_barriers() {
    for input in [
        "a:b:while(true){while(true){continue a;}}",
        "a:;a:;",
        "a:while(true){b:while(true){continue a;} continue a;}",
    ] {
        Parser::new(Source::from_bytes(input))
            .parse_script(&Scope::new_global(), &mut Interner::default())
            .unwrap_or_else(|error| panic!("{input}: {error}"));
    }
    for (input, diagnostic) in [
        ("a:a:;", "duplicate label: a"),
        ("break missing;", "undefined break target: missing"),
        (
            "a:{while(true) continue a;}",
            "undefined continue target: a",
        ),
        (
            "a:while(true){b:while(true){} continue b;}",
            "undefined continue target: b",
        ),
    ] {
        let error = Parser::new(Source::from_bytes(input))
            .parse_script(&Scope::new_global(), &mut Interner::default())
            .unwrap_err();
        assert!(error.to_string().contains(diagnostic), "{input}: {error}");
    }
}

#[test]
fn empty_input_has_an_exact_grammar_budget_for_each_entry_point() {
    for (entry, expected) in [
        ("script", 3),
        ("eval", 3),
        ("module", 2),
        ("function body", 2),
        ("formal parameters", 1),
    ] {
        for limit in [expected - 1, expected] {
            let mut admitted = 0;
            let mut control = |work| {
                if work == FrontendWork::GrammarDispatch {
                    if admitted == limit {
                        return Err(FrontendLimit::GrammarDispatch(limit));
                    }
                    admitted += 1;
                }
                Ok(())
            };
            let mut parser = Parser::with_control(Source::from_bytes(""), Some(&mut control));
            let scope = Scope::new_global();
            let mut interner = Interner::default();
            let result = match entry {
                "script" => parser.parse_script(&scope, &mut interner).map(drop),
                "eval" => parser.parse_eval(true, &mut interner).map(drop),
                "module" => parser.parse_module(&scope, &mut interner).map(drop),
                "function body" => parser
                    .parse_function_body(&mut interner, false, false)
                    .map(drop),
                "formal parameters" => parser
                    .parse_formal_parameters(&mut interner, false, false)
                    .map(drop),
                _ => unreachable!(),
            };
            assert_eq!(admitted, limit, "{entry}");
            if limit == expected {
                result.unwrap_or_else(|error| panic!("{entry}: {error:?}"));
            } else {
                assert!(
                    matches!(result, Err(crate::Error::FrontendLimit(FrontendLimit::GrammarDispatch(value))) if value == limit),
                    "{entry}"
                );
            }
        }
    }
}

#[test]
fn grammar_refusal_precedes_the_first_source_read() {
    let mut reads = 0;
    let source = Source::from_decoder(
        || {
            reads += 1;
            Ok(None)
        },
        None,
    );
    let mut control = |work| {
        assert_eq!(work, FrontendWork::GrammarDispatch);
        Err(FrontendLimit::GrammarDispatch(0))
    };
    let error = Parser::with_control(source, Some(&mut control))
        .parse_script(&Scope::new_global(), &mut Interner::default())
        .unwrap_err();
    assert_eq!(
        error.frontend_limit(),
        Some(FrontendLimit::GrammarDispatch(0))
    );
    assert_eq!(reads, 0);
}

#[test]
fn repeated_inline_items_and_recursive_unary_work_each_charge_once() {
    let count = |input, module| {
        let mut admitted = 0;
        let mut control = |work| {
            if work == FrontendWork::GrammarDispatch {
                admitted += 1;
            }
            Ok(())
        };
        let mut parser = Parser::with_control(Source::from_bytes(input), Some(&mut control));
        let scope = Scope::new_global();
        let mut interner = Interner::default();
        if module {
            parser.parse_module(&scope, &mut interner).unwrap();
        } else {
            parser.parse_script(&scope, &mut interner).unwrap();
        }
        admitted
    };

    for (single, double, module) in [
        ("!x", "!!x", false),
        ("a.b", "a.b.c", false),
        ("f().b", "f().b.c", false),
        ("a?.b", "a?.b.c", false),
        ("[,]", "[,,]", false),
        ("let [,]=a", "let [,,]=a", false),
        (
            "import 'm' with { a: 'x' }",
            "import 'm' with { a: 'x', b: 'y' }",
            true,
        ),
    ] {
        assert_eq!(count(double, module), count(single, module) + 1, "{double}");
    }

    let limit = count("a?.b", false);
    let mut dispatches = 0;
    let mut advances = 0;
    let mut control = |work| {
        match work {
            FrontendWork::GrammarDispatch => {
                if dispatches == limit {
                    return Err(FrontendLimit::GrammarDispatch(limit));
                }
                dispatches += 1;
            }
            FrontendWork::TokenAdvance => advances += 1,
            _ => {}
        }
        Ok(())
    };
    let refusal = Parser::with_control(Source::from_bytes("a?.b?.c"), Some(&mut control))
        .parse_script(&Scope::new_global(), &mut Interner::default())
        .unwrap_err();
    assert_eq!(
        refusal.frontend_limit(),
        Some(FrontendLimit::GrammarDispatch(limit))
    );
    assert_eq!(advances, 3, "the refused continuation advanced its marker");
}

#[test]
fn ordinary_parser_and_lexer_preserve_the_readers_thread_traits() {
    fn require_send_sync<T: Send + Sync>(_: &T) {}

    require_send_sync(&Parser::new(Source::from_bytes("1")));
    require_send_sync(&crate::Lexer::from(&b"1"[..]));
}

#[test]
fn a_hashbang_does_not_hide_a_source_read_failure() {
    assert_reader_failure_at(b"#!\n1", 2);
}

#[test]
fn a_nonzero_numeric_literal_does_not_hide_a_source_read_failure() {
    assert_reader_failure_at(b"4;", 1);
}

fn assert_reader_failure_at(input: &'static [u8], fail_at: u64) {
    use std::io::{self, Read};

    struct SourceWithFault {
        input: io::Cursor<&'static [u8]>,
        fail_at: u64,
        reported: bool,
    }

    impl Read for SourceWithFault {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !output.is_empty() && !self.reported && self.input.position() == self.fail_at {
                self.reported = true;
                return Err(io::Error::new(io::ErrorKind::InvalidData, "source fault"));
            }
            self.input.read(output)
        }
    }

    let source = Source::from_reader(
        SourceWithFault {
            input: io::Cursor::new(input),
            fail_at,
            reported: false,
        },
        None,
    );
    let error = Parser::new(source)
        .parse_script(&Scope::new_global(), &mut Interner::default())
        .unwrap_err();
    let crate::Error::Lex {
        err: crate::lexer::error::Error::IO(error),
    } = error
    else {
        panic!("expected the original reader failure, got {error:?}");
    };
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "source fault");
}

/// Checks that the given JavaScript string gives the expected expression.
#[track_caller]
pub(super) fn check_script_parser<L>(js: &str, expr: L, interner: &mut Interner)
where
    L: Into<Box<[StatementListItem]>>,
{
    let mut script = Script::new(StatementList::from((expr.into(), PSEUDO_LINEAR_POS)));
    let scope = Scope::new_global();
    script
        .analyze_scope(&scope, interner)
        .expect("failed to analyze script");
    assert_eq!(
        Parser::new(Source::from_bytes(js))
            .parse_script(&Scope::new_global(), interner)
            .expect("failed to parse"),
        script,
    );
}

/// Checks that the given JavaScript string gives the expected expression.
#[track_caller]
pub(super) fn check_module_parser<L>(js: &str, expr: L, interner: &mut Interner)
where
    L: Into<Box<[ModuleItem]>>,
{
    let mut module = Module::new(ModuleItemList::from(expr.into()));
    let scope = Scope::new_global();
    module
        .analyze_scope(&scope, interner)
        .expect("failed to analyze");
    assert_eq!(
        Parser::new(Source::from_bytes(js))
            .parse_module(&Scope::new_global(), interner)
            .expect("failed to parse"),
        module,
    );
}

/// Checks that the given javascript string creates a parse error.
#[track_caller]
pub(super) fn check_invalid_script(js: &str) {
    assert!(
        Parser::new(Source::from_bytes(js))
            .parse_script(&Scope::new_global(), &mut Interner::default())
            .is_err()
    );
}

/// Should be parsed as `new Class().method()` instead of `new (Class().method())`
#[test]
fn check_construct_call_precedence() {
    let interner = &mut Interner::default();
    check_script_parser(
        "new Date().getTime()",
        vec![
            Statement::Expression(
                Call::new(
                    Expression::PropertyAccess(
                        SimplePropertyAccess::new(
                            New::from(Call::new(
                                Identifier::new(
                                    interner.get_or_intern_static("Date", utf16!("Date")),
                                    Span::new((1, 5), (1, 9)),
                                )
                                .into(),
                                Box::default(),
                                Span::new((1, 1), (1, 11)),
                            ))
                            .into(),
                            Identifier::new(
                                interner.get_or_intern_static("getTime", utf16!("getTime")),
                                Span::new((1, 12), (1, 19)),
                            ),
                        )
                        .into(),
                    ),
                    Box::default(),
                    Span::new((1, 19), (1, 21)),
                )
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn assign_operator_precedence() {
    let interner = &mut Interner::default();
    check_script_parser(
        "a = a + 1",
        vec![
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((1, 1), (1, 2)),
                )
                .into(),
                Binary::new(
                    ArithmeticOp::Add.into(),
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    )
                    .into(),
                    Literal::new(1, Span::new((1, 9), (1, 10))).into(),
                )
                .into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn hoisting() {
    let interner = &mut Interner::default();
    let hello = interner.get_or_intern_static("hello", utf16!("hello"));
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        indoc! {"
            var a = hello();
            a++;

            function hello() { return 10 }
        "},
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(a, Span::new((1, 5), (1, 6))),
                    Some(
                        Call::new(
                            Identifier::new(hello, Span::new((1, 9), (1, 14))).into(),
                            Box::default(),
                            Span::new((1, 14), (1, 16)),
                        )
                        .into(),
                    ),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(
                Update::new(
                    UpdateOp::IncrementPost,
                    UpdateTarget::Identifier(Identifier::new(a, Span::new((2, 1), (2, 2)))),
                    Span::new((2, 1), (2, 4)),
                )
                .into(),
            )
            .into(),
            Declaration::FunctionDeclaration(FunctionDeclaration::new(
                Identifier::new(hello, Span::new((4, 10), (4, 15))),
                FormalParameterList::default(),
                FunctionBody::new(
                    StatementList::new(
                        [Statement::Return(Return::new(Some(
                            Literal::new(10, Span::new((4, 27), (4, 29))).into(),
                        )))
                        .into()],
                        PSEUDO_LINEAR_POS,
                        false,
                    ),
                    Span::new((4, 18), (4, 31)),
                ),
                EMPTY_LINEAR_SPAN,
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        indoc! {"
            a = 10;
            a++;

            var a;
        "},
        vec![
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a, Span::new((1, 1), (1, 2))).into(),
                Literal::new(10, Span::new((1, 5), (1, 7))).into(),
            )))
            .into(),
            Statement::Expression(
                Update::new(
                    UpdateOp::IncrementPost,
                    UpdateTarget::Identifier(Identifier::new(a, Span::new((2, 1), (2, 2)))),
                    Span::new((2, 1), (2, 4)),
                )
                .into(),
            )
            .into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(a, Span::new((4, 5), (4, 6))),
                    None,
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn ambiguous_regex_divide_expression() {
    let s = "1 / a === 1 / b";

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Statement::Expression(Expression::from(Binary::new(
                RelationalOp::StrictEqual.into(),
                Binary::new(
                    ArithmeticOp::Div.into(),
                    Literal::new(1, Span::new((1, 1), (1, 2))).into(),
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    )
                    .into(),
                )
                .into(),
                Binary::new(
                    ArithmeticOp::Div.into(),
                    Literal::new(1, Span::new((1, 11), (1, 12))).into(),
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((1, 15), (1, 16)),
                    )
                    .into(),
                )
                .into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn two_divisions_in_expression() {
    let s = "a !== 0 || 1 / a === 1 / b;";

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        s,
        vec![
            Statement::Expression(Expression::from(Binary::new(
                LogicalOp::Or.into(),
                Binary::new(
                    RelationalOp::StrictNotEqual.into(),
                    Identifier::new(a, Span::new((1, 1), (1, 2))).into(),
                    Literal::new(0, Span::new((1, 7), (1, 8))).into(),
                )
                .into(),
                Binary::new(
                    RelationalOp::StrictEqual.into(),
                    Binary::new(
                        ArithmeticOp::Div.into(),
                        Literal::new(1, Span::new((1, 12), (1, 13))).into(),
                        Identifier::new(a, Span::new((1, 16), (1, 17))).into(),
                    )
                    .into(),
                    Binary::new(
                        ArithmeticOp::Div.into(),
                        Literal::new(1, Span::new((1, 22), (1, 23))).into(),
                        Identifier::new(
                            interner.get_or_intern_static("b", utf16!("b")),
                            Span::new((1, 26), (1, 27)),
                        )
                        .into(),
                    )
                    .into(),
                )
                .into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn comment_semi_colon_insertion() {
    let s = indoc! {"
        let a = 10 // Comment
        let b = 20;
    "};

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(10, Span::new((1, 9), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((2, 5), (2, 6)),
                    ),
                    Some(Literal::new(20, Span::new((2, 9), (2, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn multiline_comment_semi_colon_insertion() {
    let s = indoc! {"
        let a = 10 /* Test
        Multiline
        Comment
        */ let b = 20;
    "};

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(10, Span::new((1, 9), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((4, 8), (4, 9)),
                    ),
                    Some(Literal::new(20, Span::new((4, 12), (4, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn multiline_comment_no_lineterminator() {
    let s = indoc! {"
        let a = 10; /* Test comment */ let b = 20;
    "};

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(10, Span::new((1, 9), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((1, 36), (1, 37)),
                    ),
                    Some(Literal::new(20, Span::new((1, 40), (1, 42))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn assignment_line_terminator() {
    let s = indoc! {"
        let a = 3;

        a =
        5;
    "};

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(3, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(
                    interner.get_or_intern_static("a", utf16!("a")),
                    Span::new((3, 1), (3, 2)),
                )
                .into(),
                Literal::new(5, Span::new((4, 1), (4, 2))).into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn assignment_multiline_terminator() {
    let s = indoc! {"
        let a = 3;


        a =


        5;
    "};

    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(a, Span::new((1, 5), (1, 6))),
                    Some(Literal::new(3, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(Expression::from(Assign::new(
                AssignOp::Assign,
                Identifier::new(a, Span::new((4, 1), (4, 2))).into(),
                Literal::new(5, Span::new((7, 1), (7, 2))).into(),
            )))
            .into(),
        ],
        interner,
    );
}

#[test]
fn bracketed_expr() {
    let s = "(b)";

    let interner = &mut Interner::default();
    check_script_parser(
        s,
        vec![
            Statement::Expression(
                Parenthesized::new(
                    Identifier::new(
                        interner.get_or_intern_static("b", utf16!("b")),
                        Span::new((1, 2), (1, 3)),
                    )
                    .into(),
                    Span::new((1, 1), (1, 4)),
                )
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn increment_in_comma_op() {
    let s = "(b++, b)";

    let interner = &mut Interner::default();
    let b = interner.get_or_intern_static("b", utf16!("b"));
    check_script_parser(
        s,
        vec![
            Statement::Expression(
                Parenthesized::new(
                    Binary::new(
                        BinaryOp::Comma,
                        Update::new(
                            UpdateOp::IncrementPost,
                            UpdateTarget::Identifier(Identifier::new(b, Span::new((1, 2), (1, 3)))),
                            Span::new((1, 2), (1, 5)),
                        )
                        .into(),
                        Identifier::new(b, Span::new((1, 7), (1, 8))).into(),
                    )
                    .into(),
                    Span::new((1, 1), (1, 9)),
                )
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn spread_in_object() {
    let s = indoc! {"
        let x = {
            a: 1,
            ...b,
        }
    "};

    let interner = &mut Interner::default();

    let object_properties = vec![
        PropertyDefinition::Property(
            Identifier::new(
                interner.get_or_intern_static("a", utf16!("a")),
                Span::new((2, 5), (2, 6)),
            )
            .into(),
            Literal::new(1, Span::new((2, 8), (2, 9))).into(),
        ),
        PropertyDefinition::SpreadObject(
            Identifier::new(
                interner.get_or_intern_static("b", utf16!("b")),
                Span::new((3, 8), (3, 9)),
            )
            .into(),
        ),
    ];

    check_script_parser(
        s,
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("x", utf16!("x")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(ObjectLiteral::new(object_properties, Span::new((1, 9), (4, 2))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn spread_in_arrow_function() {
    let s = indoc! {r#"
        (...b) => {
            b
        }
    "#};

    let interner = &mut Interner::default();
    let b = interner.get_or_intern_static("b", utf16!("b"));
    let params = FormalParameterList::from(FormalParameter::new(
        Variable::from_identifier(Identifier::new(b, Span::new((1, 5), (1, 6))), None),
        true,
    ));
    assert_eq!(params.flags(), FormalParameterListFlags::HAS_REST_PARAMETER);
    assert_eq!(params.length(), 0);
    check_script_parser(
        s,
        vec![
            Statement::Expression(
                ArrowFunction::new(
                    None,
                    params,
                    FunctionBody::new(
                        StatementList::new(
                            [Statement::Expression(
                                Identifier::new(b, Span::new((2, 5), (2, 6))).into(),
                            )
                            .into()],
                            PSEUDO_LINEAR_POS,
                            false,
                        ),
                        Span::new((1, 11), (3, 2)),
                    ),
                    EMPTY_LINEAR_SPAN,
                    Span::new((1, 1), (3, 2)),
                )
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

#[test]
fn empty_statement() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    check_script_parser(
        indoc! {"
            ;;var a = 10;
            if(a) ;
        "},
        vec![
            Statement::Empty.into(),
            Statement::Empty.into(),
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(a, Span::new((1, 7), (1, 8))),
                    Some(Literal::new(10, Span::new((1, 11), (1, 13))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::If(If::new(
                Identifier::new(a, Span::new((2, 4), (2, 5))).into(),
                Statement::Empty,
                None,
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn empty_statement_ends_directive_prologues() {
    let interner = &mut Interner::default();
    let a = interner.get_or_intern_static("a", utf16!("a"));
    let use_strict = interner.get_or_intern_static("use strict", utf16!("use strict"));
    let public = interner.get_or_intern_static("public", utf16!("public"));
    check_script_parser(
        indoc! {r#"
            "a";
            ;
            "use strict";
            let public = 5;
        "#},
        vec![
            Statement::Expression(Literal::new(a, Span::new((1, 1), (1, 4))).into()).into(),
            Statement::Empty.into(),
            Statement::Expression(Literal::new(use_strict, Span::new((3, 1), (3, 13))).into())
                .into(),
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(public, Span::new((4, 5), (4, 11))),
                    Some(Literal::new(5, Span::new((4, 14), (4, 15))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

#[test]
fn hashbang_use_strict_no_with() {
    check_script_parser(
        r#"#!\"use strict"
        "#,
        vec![],
        &mut Interner::default(),
    );
}

#[test]
fn hashbang_use_strict_with_with_statement() {
    check_script_parser(
        indoc! {r#"
            #!\"use strict"
            with({}) {}
        "#},
        vec![
            Statement::With(With::new(
                ObjectLiteral::new([], Span::new((2, 6), (2, 8))).into(),
                Block::from(StatementList::new([], LinearPosition::new(27), false)).into(),
            ))
            .into(),
        ],
        &mut Interner::default(),
    );
}

#[test]
fn hashbang_comment() {
    check_script_parser(r"#!Comment Here", vec![], &mut Interner::default());
}

#[test]
fn deny_unicode_escape_in_false_expression() {
    check_invalid_script(r"let x = f\u{61}lse;");
}

#[test]
fn deny_unicode_escape_in_true_expression() {
    check_invalid_script(r"let x = tru\u{65};");
}

#[test]
fn deny_unicode_escape_in_null_expression() {
    check_invalid_script(r"let x = n\u{75}ll;");
}

#[test]
fn stress_test_operations() {
    let src = ("1 * 2 + /* comment why not */ 3 / 4 % 5 + ".repeat(1_000)
        + "1; // end of line\n\n")
        .repeat(1_000);

    assert!(
        Parser::new(Source::from_bytes(&src))
            .parse_script(&Scope::new_global(), &mut Interner::default())
            .is_ok()
    );
}

#[test]
fn debugger_statement() {
    check_script_parser(
        "debugger;",
        vec![Statement::Debugger.into()],
        &mut Interner::default(),
    );

    check_script_parser(
        "debugger",
        vec![Statement::Debugger.into()],
        &mut Interner::default(),
    );

    check_invalid_script("!debugger");

    check_invalid_script("let x = debugger;");

    check_invalid_script("debugger + debugger");
}

#[test]
fn invalid_arrow_function() {
    check_invalid_script(r#"(!()=>"#);
    check_invalid_script(r#"!()=>{}"#);
}

#![allow(unused_crate_dependencies)]

use std::{
    cell::{Cell, RefCell},
    io::{self, Read},
    path::Path,
    rc::Rc,
    time::Duration,
};

struct CountingReader<'a> {
    bytes: &'a [u8],
    reads: &'a Cell<u64>,
}

#[test]
fn interner_publication_admits_index_and_retained_units_together() {
    use surgeist_js_parser::frontend::InternerLimitKind;

    let input = r#""\uD800""#;
    let mut refused_index = 0;
    for limit in [3, 4] {
        let mut context = Context::default();
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(64).with_interner_utf16_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.interner_utf16_units(), limit);
        assert_eq!(evaluation.report.interner_bytes(), 0);
        if limit == 3 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
                    kind: InternerLimitKind::Utf16,
                    limit,
                }))
            );
            assert_eq!(evaluation.report.instructions(), 0);
            refused_index = evaluation.report.interner_index_work();
        } else {
            assert_eq!(evaluation.report.interner_index_work(), refused_index + 1);
            assert_eq!(
                evaluation.result.unwrap().as_string().unwrap().to_vec(),
                [0xD800]
            );
        }
    }
}

#[test]
fn interner_byte_refusal_precedes_the_first_lookup_probe() {
    use surgeist_js_parser::frontend::InternerLimitKind;

    let mut context = Context::default();
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("frontendBudgetName;"),
            ExecutionOptions::new(64).with_interner_byte_limit(0),
        )
        .unwrap();
    assert_eq!(
        evaluation.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
            kind: InternerLimitKind::Bytes,
            limit: 0,
        }))
    );
    assert_eq!(evaluation.report.interner_bytes(), 0);
    assert_eq!(evaluation.report.interner_index_work(), 0);
    assert_eq!(evaluation.report.instructions(), 0);
}

#[test]
fn nested_eval_interner_refusal_bypasses_guest_catch() {
    use surgeist_js_parser::frontend::InternerLimitKind;

    // The outer frontend hashes and compares the four bytes of the common
    // eval name and publishes 17 UTF-8 bytes for the string literal. Its eval
    // then tries to hash the identifier in a fresh interner.
    let limit = 4 + 4 + 17;
    let mut context = Context::default();
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("try { eval('hiddenRuntimeName') } catch { 99 }"),
            ExecutionOptions::new(128).with_interner_byte_limit(limit),
        )
        .unwrap();
    assert_eq!(evaluation.report.interner_bytes(), limit);
    assert!(evaluation.report.instructions() > 0);
    assert_eq!(
        evaluation.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
            kind: InternerLimitKind::Bytes,
            limit,
        }))
    );
}

#[test]
fn label_lookup_budget_refuses_before_resolution_and_preserves_exact_work() {
    let mut context = Context::default();
    for (source, limit) in [("break missing;", 0), ("a:while(false){continue a;}", 1)] {
        let refused = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(64).with_label_lookup_limit(limit),
            )
            .unwrap();
        assert_eq!(refused.report.label_lookups(), limit);
        assert_eq!(refused.report.instructions(), 0);
        assert_eq!(
            refused.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::LabelLookup {
                limit
            }))
        );
    }
    let exact = context
        .eval_with_budget(
            Source::from_bytes("a:while(false){continue a;}"),
            ExecutionOptions::new(64).with_label_lookup_limit(2),
        )
        .unwrap();
    assert_eq!(exact.report.label_lookups(), 2);
    assert_eq!(exact.result.unwrap(), JsValue::undefined());

    let nested = context
        .eval_with_budget(
            Source::from_bytes(
                "globalThis.started = true;
                 try { eval('break missing;'); } catch { globalThis.caught = true; }",
            ),
            ExecutionOptions::new(64).with_label_lookup_limit(0),
        )
        .unwrap();
    assert_eq!(nested.report.label_lookups(), 0);
    assert_eq!(
        nested.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::LabelLookup {
            limit: 0
        }))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "started && typeof caught === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn early_error_visit_budget_is_exact_and_independent_of_profiling() {
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut context = Context::default();
        // Four early-error walks each admit the list, item and empty statement.
        for limit in [0, 11, 12] {
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes(";"),
                    ExecutionOptions::new(64)
                        .with_early_error_visit_limit(limit)
                        .with_profile(profile),
                )
                .unwrap();
            assert_eq!(outcome.report.early_error_visits(), limit);
            assert_eq!(outcome.report.grammar_dispatches(), 5);
            if limit == 12 {
                assert_eq!(outcome.result.unwrap(), JsValue::undefined());
            } else {
                assert_eq!(outcome.report.instructions(), 0);
                assert_eq!(
                    outcome.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::EarlyErrorVisit { limit }
                    ))
                );
            }
        }
    }
}

#[test]
fn a_nested_early_error_walk_keeps_the_outer_operation_and_first_stop() {
    let mut context = Context::default();
    let body = ";".repeat(512);
    let script = format!(
        "globalThis.entered = true; try {{ eval('{body}'); }}
         catch {{ globalThis.caught = true; }}"
    );
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(&script),
            ExecutionOptions::new(128)
                .with_early_error_visit_limit(512)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(outcome.report.early_error_visits(), 512);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == "frontend")
        .unwrap();
    assert_eq!(frontend.phases, 2);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::EarlyErrorVisit { limit: 512 }
        ))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "entered && typeof caught === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn direct_eval_charges_the_contextual_new_target_check() {
    use surgeist_js_interner::Interner;
    use surgeist_js_parser::{Parser, frontend::FrontendWork};

    const SCRIPT: &str = "globalThis.entered = true;\n\
                         try { eval('new.target;'); }\n\
                         catch (error) { globalThis.caught = error instanceof SyntaxError; }";
    let inner: Vec<u16> = "new.target;".encode_utf16().collect();
    let mut parse_visits = 0;
    {
        let mut count_visits = |work| {
            if work == FrontendWork::EarlyErrorVisit {
                parse_visits += 1;
            }
            Ok(())
        };
        Parser::with_control(Source::from_bytes(SCRIPT), Some(&mut count_visits))
            .parse_script(
                &surgeist_js_ast::scope::Scope::new_global(),
                &mut Interner::default(),
            )
            .unwrap();
        Parser::with_control(Source::from_utf16(&inner), Some(&mut count_visits))
            .parse_eval(true, &mut Interner::default())
            .unwrap();
    }

    // The contextual check visits the statement list, its item, the statement,
    // the expression and finally the new.target node that proves the syntax error.
    let exact = parse_visits + 5;
    for limit in [exact - 1, exact] {
        let mut context = Context::default();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(SCRIPT),
                ExecutionOptions::new(128).with_early_error_visit_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.early_error_visits(), limit);
        if limit == exact {
            outcome.result.unwrap();
            assert_eq!(
                context
                    .eval(Source::from_bytes("entered && caught"))
                    .unwrap(),
                JsValue::new(true)
            );
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::EarlyErrorVisit { limit }
                ))
            );
            assert_eq!(
                context
                    .eval(Source::from_bytes(
                        "entered && typeof caught === 'undefined'"
                    ))
                    .unwrap(),
                JsValue::new(true)
            );
        }
    }
}

#[test]
fn parser_diagnostics_preserve_source_spellings_and_error_positions() {
    let mut context = Context::default();
    let err = context
        .eval(Source::from_bytes(r#"let x = 0 'A😀\uD800é';"#))
        .unwrap_err();
    assert_eq!(
        err.as_native().unwrap().message(),
        "expected one of ';' or 'line terminator', got 'A😀\\uD800é' in lexical declaration binding list at line 1, col 11"
    );

    let Err(err) =
        surgeist_js::module::Module::parse(Source::from_bytes("let π; let π;"), None, &mut context)
    else {
        panic!("duplicate lexical name must be rejected");
    };
    assert_eq!(
        err.as_native().unwrap().message(),
        "lexical name `π` declared multiple times at line 1, col 1"
    );
}

#[test]
fn diagnostic_limits_preserve_the_admitted_prefix_and_bypass_guest_catch() {
    const SCRIPT: &str = r#"try { eval("let x = 0 'A😀\\uD800é';"); }
        catch { globalThis.caught = true; }"#;
    let cases = [
        (
            2,
            13,
            2,
            1,
            Some(RuntimeLimitError::DiagnosticInput { limit: 2 }),
        ),
        (
            4,
            13,
            4,
            5,
            Some(RuntimeLimitError::DiagnosticInput { limit: 4 }),
        ),
        (
            5,
            4,
            3,
            1,
            Some(RuntimeLimitError::DiagnosticOutput { limit: 4 }),
        ),
        (
            5,
            10,
            5,
            5,
            Some(RuntimeLimitError::DiagnosticOutput { limit: 10 }),
        ),
        (
            5,
            12,
            5,
            11,
            Some(RuntimeLimitError::DiagnosticOutput { limit: 12 }),
        ),
        (5, 13, 5, 13, None),
    ];
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        for (input_limit, output_limit, units, bytes, expected) in cases {
            let mut context = Context::default();
            let evaluation = context
                .eval_with_budget(
                    Source::from_bytes(SCRIPT),
                    ExecutionOptions::new(128)
                        .with_diagnostic_input_limit(input_limit)
                        .with_diagnostic_output_limit(output_limit)
                        .with_profile(profile),
                )
                .unwrap();
            assert_eq!(evaluation.report.diagnostic_input(), units);
            assert_eq!(evaluation.report.diagnostic_output(), bytes);
            if let Some(error) = expected {
                assert_eq!(
                    evaluation.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(error))
                );
                assert!(
                    context
                        .eval(Source::from_bytes("globalThis.caught"))
                        .unwrap()
                        .is_undefined()
                );
            } else {
                evaluation.result.unwrap();
                assert_eq!(
                    context
                        .eval(Source::from_bytes("globalThis.caught"))
                        .unwrap(),
                    JsValue::new(true)
                );
            }
        }
    }
}

#[test]
fn label_diagnostics_charge_the_message_prefix_and_source_name() {
    let mut context = Context::default();
    let prefix = u64::try_from("undefined break target: ".len()).unwrap();
    let limit = prefix + 1;
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("break π;"),
            ExecutionOptions::new(32).with_diagnostic_output_limit(limit),
        )
        .unwrap();
    assert_eq!(evaluation.report.diagnostic_input(), 1);
    assert_eq!(evaluation.report.diagnostic_output(), prefix);
    assert_eq!(evaluation.report.instructions(), 0);
    assert_eq!(
        evaluation.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::DiagnosticOutput { limit }
        ))
    );
}

#[test]
fn dynamic_function_source_limits_include_separators_and_wrapped_body_eof() {
    const SCRIPT: &str = "globalThis.built = C('π', '', 'return π');";
    // UTF-8 script and EOF, then UTF-16 `π,` and `\nreturn π\n`, each with its own EOF.
    let outer_reads = u64::try_from(SCRIPT.len()).unwrap() + 1;
    // Annex B probes for an HTML close comment after the trailing line break.
    // It peeks one point, then three points, before the ordinary EOF attempt.
    let html_close_probes = if cfg!(feature = "annex-b") { 4 } else { 0 };
    let inner_reads = 3 + 11 + html_close_probes;

    for setup in [
        "globalThis.C = Function;",
        "globalThis.C = (function* () {}).constructor;",
        "globalThis.C = (async function () {}).constructor;",
        "globalThis.C = (async function* () {}).constructor;",
    ] {
        for allowed in [inner_reads - 1, inner_reads] {
            let mut context = Context::default();
            context.eval(Source::from_bytes(setup)).unwrap();
            let limit = outer_reads + allowed;
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes(SCRIPT),
                    ExecutionOptions::new(128)
                        .with_source_read_limit(limit)
                        .with_profile(ProfileMode::Summary { timing: false }),
                )
                .unwrap();
            assert_eq!(outcome.report.source_reads(), limit, "{setup}");
            let utf16 = outcome
                .report
                .profile()
                .unwrap()
                .scopes
                .iter()
                .find(|scope| scope.name == "utf16-source")
                .unwrap();
            assert_eq!(utf16.accepted_items, allowed, "{setup}");

            if allowed == inner_reads {
                outcome.result.unwrap();
                assert_eq!(utf16.refused_batches, 0);
                assert_eq!(
                    context.eval(Source::from_bytes("built.length")).unwrap(),
                    JsValue::new(1)
                );
            } else {
                assert_eq!(utf16.refused_batches, 1);
                assert_eq!(
                    outcome.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
                        limit
                    }))
                );
                assert!(
                    context
                        .eval(Source::from_bytes("globalThis.built"))
                        .unwrap()
                        .is_undefined()
                );
            }
        }
    }
}

#[test]
fn dynamic_function_only_parses_an_empty_parameter_list_when_an_argument_supplies_it() {
    for (script, nested_reads) in [
        ("Function()", 0),
        ("Function('')", 0),
        ("Function('', '')", 1),
    ] {
        let mut context = Context::default();
        let outcome = context
            .eval_with_budget(Source::from_bytes(script), ExecutionOptions::new(64))
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(
            outcome.report.source_reads(),
            u64::try_from(script.len()).unwrap() + 1 + nested_reads
        );
    }
}

#[test]
fn dynamic_function_body_preserves_utf16_literals_and_wrapped_error_lines() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"Function("return '\uD83D\uDE00\uD800'")()"#,
        ))
        .unwrap();
    assert_eq!(
        result.as_string().unwrap().to_vec(),
        [0xD83D, 0xDE00, 0xD800]
    );

    let error = context
        .eval(Source::from_bytes("Function('return @')"))
        .unwrap_err();
    let message = error.as_native().unwrap().message();
    assert!(message.starts_with("failed to parse function body: "));
    assert!(message.contains("at line 2,"), "{message}");
}

#[test]
fn dynamic_function_charges_direct_eval_metadata_before_publication() {
    const SCRIPT: &str = "globalThis.constructed = Function(\"eval('1');\");";

    // The empty formal parameter list takes one visit. The body search then
    // visits the list, item, statement, expression, call and direct callee.
    for (limit, published) in [(6, false), (7, true)] {
        let mut context = Context::default();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(SCRIPT),
                ExecutionOptions::new(128).with_direct_eval_visit_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.direct_eval_visits(), limit);
        assert!(outcome.report.instructions() > 0);
        if published {
            outcome.result.unwrap();
            assert_eq!(
                context
                    .eval(Source::from_bytes("typeof constructed === 'function'"))
                    .unwrap(),
                JsValue::new(true)
            );
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::DirectEvalVisit { limit }
                ))
            );
            assert_eq!(
                context
                    .eval(Source::from_bytes("typeof constructed === 'undefined'"))
                    .unwrap(),
                JsValue::new(true)
            );
        }
    }
}

#[test]
fn dynamic_function_charges_private_name_validation_before_publication() {
    let input = "typeof Function('class C { #a; #b; m() { return this.#b; } }')";
    let mut context = Context::default();
    for limit in [3, 4] {
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(128).with_name_comparison_limit(limit),
            )
            .unwrap();
        if limit == 3 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::NameComparison { limit }
                ))
            );
        } else {
            assert_eq!(
                evaluation.result.unwrap(),
                JsValue::from(js_string!("function"))
            );
        }
        assert_eq!(evaluation.report.name_comparisons(), limit);
    }
}

#[test]
fn dynamic_function_private_parameters_charge_resolution_before_publication() {
    let input = "typeof Function('x = class { #a; #b; m() { return this.#b; } }', '')";
    let mut context = Context::default();
    // Two declaration operations, three parameter metadata checks, and two
    // comparisons to resolve the access to the second private name.
    for limit in [6, 7] {
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(128).with_name_comparison_limit(limit),
            )
            .unwrap();
        if limit == 6 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::NameComparison { limit }
                ))
            );
        } else {
            assert_eq!(
                evaluation.result.unwrap(),
                JsValue::from(js_string!("function"))
            );
        }
        assert_eq!(evaluation.report.name_comparisons(), limit);
    }
}

#[test]
fn direct_eval_private_name_refusal_reaches_the_host() {
    let input = "new (class {
        #x = 1;
        m() {
            globalThis.privateMeterMarker = 5;
            try { return eval('this.#x'); } catch { return 7; }
        }
    })().m();";
    for limit in [1, 2] {
        let mut context = Context::default();
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(256).with_name_comparison_limit(limit),
            )
            .unwrap();
        if limit == 1 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::NameComparison { limit }
                ))
            );
        } else {
            assert_eq!(evaluation.result.unwrap(), JsValue::new(1));
        }
        assert_eq!(evaluation.report.name_comparisons(), limit);
        assert_eq!(
            context
                .eval(Source::from_bytes("privateMeterMarker"))
                .unwrap(),
            JsValue::new(5)
        );
    }
}

#[test]
fn dynamic_function_charges_each_strict_binding_and_redeclaration_comparison() {
    for (source, records, earlier_comparisons, diagnostic) in [
        (
            "Function('eval', '\"use strict\";');",
            2,
            3,
            "Unexpected 'eval' or 'arguments' in strict mode",
        ),
        (
            "Function('a', 'let b; let a;');",
            6,
            7,
            "Redeclaration of formal parameter `a`",
        ),
    ] {
        let mut context = Context::default();
        let options = ExecutionOptions::new(128);
        let record_limit = records - 1;
        let denied = context
            .eval_with_budget(
                Source::from_bytes(source),
                options.with_name_record_limit(record_limit),
            )
            .unwrap();
        assert!(denied.report.instructions() > 0);
        assert_eq!(denied.report.name_records(), record_limit);
        assert_eq!(denied.report.name_comparisons(), earlier_comparisons);
        assert_eq!(
            denied.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::NameRecord {
                limit: record_limit,
            }))
        );

        // The formal parser has already checked `arguments`, then looked up
        // and inserted the parameter. Each body declaration also checks `let`
        // and inserts its name. The final check compares `eval` against
        // `arguments` and `eval`, or parameter `a` against `b` and `a`.
        for final_comparisons in [1, 2] {
            let limit = earlier_comparisons + final_comparisons;
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes(source),
                    options.with_name_comparison_limit(limit),
                )
                .unwrap();
            assert_eq!(outcome.report.name_records(), records);
            assert_eq!(outcome.report.name_comparisons(), limit);
            let error = outcome.result.unwrap_err();
            if final_comparisons == 1 {
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::NameComparison { limit }
                    ))
                );
            } else {
                let error = error.as_native().unwrap();
                assert!(error.is_syntax());
                assert_eq!(error.message(), diagnostic);
            }
        }
    }
}

#[test]
fn grammar_dispatch_budget_is_exact_and_refuses_before_source_work() {
    let mut context = Context::default();
    for limit in [0, 4, 5] {
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(";"),
                ExecutionOptions::new(64).with_grammar_dispatch_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.grammar_dispatches(), limit);
        if limit == 0 {
            assert_eq!(outcome.report.source_reads(), 0);
            assert_eq!(outcome.report.token_probes(), 0);
        }
        if limit == 5 {
            assert_eq!(outcome.result.unwrap(), JsValue::undefined());
        } else {
            assert_eq!(outcome.report.instructions(), 0);
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::GrammarDispatch { limit }
                ))
            );
        }
    }
}

#[test]
fn nested_eval_grammar_refusal_bypasses_guest_catch() {
    let mut context = Context::default();
    let body = ";".repeat(512);
    let script = format!(
        "globalThis.entered = true; try {{ eval('{body}'); }}
         catch {{ globalThis.caught = true; }}"
    );
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(&script),
            ExecutionOptions::new(128)
                .with_grammar_dispatch_limit(512)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(outcome.report.grammar_dispatches(), 512);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == "frontend")
        .unwrap();
    assert_eq!(frontend.phases, 2);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::GrammarDispatch { limit: 512 }
        ))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "entered && typeof caught === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn buffered_token_limits_refuse_before_guest_execution() {
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut context = Context::default();
        let options = ExecutionOptions::new(64).with_profile(profile);
        let probe = context
            .eval_with_budget(Source::from_bytes("1;"), options.with_token_probe_limit(0))
            .unwrap();
        assert_eq!(probe.report.token_probes(), 0);
        assert_eq!(probe.report.source_reads(), 0);
        assert_eq!(probe.report.instructions(), 0);
        assert_eq!(
            probe.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::TokenProbe {
                limit: 0
            }))
        );

        let advance = context
            .eval_with_budget(
                Source::from_bytes("1;"),
                options.with_token_advance_limit(0),
            )
            .unwrap();
        assert!(advance.report.token_probes() > 0);
        assert_eq!(advance.report.token_advances(), 0);
        assert_eq!(advance.report.instructions(), 0);
        assert_eq!(
            advance.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::TokenAdvance { limit: 0 }
            ))
        );

        let admitted = context
            .eval_with_budget(Source::from_bytes("1;"), options)
            .unwrap();
        assert_eq!(admitted.result.unwrap(), JsValue::new(1));
        assert!(admitted.report.token_probes() > 0);
        assert_eq!(admitted.report.token_advances(), 2);
    }
}

#[test]
fn lexical_dispatch_limit_counts_discarded_comments_and_refuses_before_tokenization() {
    let mut context = Context::default();
    let input = "/*x*/1";
    let refused = context
        .eval_with_budget(
            Source::from_bytes(input),
            ExecutionOptions::new(64).with_lexer_dispatch_limit(1),
        )
        .unwrap();
    assert_eq!(refused.report.lexer_dispatches(), 1);
    assert_eq!(refused.report.source_code_points(), 6);
    assert_eq!(refused.report.instructions(), 0);
    assert_eq!(
        refused.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::LexerDispatch { limit: 1 }
        ))
    );

    let admitted = context
        .eval_with_budget(
            Source::from_bytes(input),
            ExecutionOptions::new(64).with_lexer_dispatch_limit(2),
        )
        .unwrap();
    assert_eq!(admitted.report.lexer_dispatches(), 2);
    assert_eq!(admitted.result.unwrap(), JsValue::new(1));
}

#[test]
fn lookahead_refusal_preserves_the_partial_source_and_profile_equivalence() {
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut context = Context::default();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("/*x*/1"),
                ExecutionOptions::new(64)
                    .with_lexer_lookahead_limit(0)
                    .with_profile(profile),
            )
            .unwrap();
        assert_eq!(outcome.report.source_reads(), 1);
        assert_eq!(outcome.report.source_code_points(), 1);
        assert_eq!(outcome.report.retained_source_units(), 1);
        assert_eq!(outcome.report.lexer_lookahead_probes(), 0);
        assert_eq!(outcome.report.instructions(), 0);
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::LexerLookahead { limit: 0 }
            ))
        );
    }
}

#[test]
fn a_host_callback_cannot_swallow_the_first_lookahead_stop() {
    let mut context = Context::default();
    context
        .register_global_builtin_callable(
            js_string!("swallowLookaheadStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let input = "/*x*/".repeat(128);
                let error = context.eval(Source::from_bytes(&input)).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::LexerLookahead { limit: 64 }
                    ))
                );
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("swallowLookaheadStop(); globalThis.afterLookaheadStop = true;"),
            ExecutionOptions::new(64).with_lexer_lookahead_limit(64),
        )
        .unwrap();
    assert_eq!(outcome.report.lexer_lookahead_probes(), 64);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::LexerLookahead { limit: 64 }
        ))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "typeof afterLookaheadStop === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn character_and_retention_caps_include_crlf_trivia_and_astral_width() {
    let input = "'😀';\r\n/*x*/";
    for (limit, expected) in [(11, 11), (10, 10)] {
        let mut context = Context::default();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(64)
                    .with_source_consumption_limit(limit)
                    .with_source_retention_limit(12),
            )
            .unwrap();
        assert_eq!(outcome.report.source_code_points(), expected);
        assert_eq!(outcome.report.retained_source_units(), expected + 1);
        if limit == 11 {
            assert_eq!(
                outcome
                    .result
                    .unwrap()
                    .as_string()
                    .unwrap()
                    .to_std_string()
                    .unwrap(),
                "😀"
            );
        } else {
            assert_eq!(outcome.report.instructions(), 0);
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::SourceConsumption { limit }
                ))
            );
        }
    }
}

#[test]
fn astral_retention_refusal_leaves_both_consumption_dimensions_unchanged() {
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("'😀'"),
            ExecutionOptions::new(64).with_source_retention_limit(2),
        )
        .unwrap();
    assert_eq!(outcome.report.source_reads(), 5);
    assert_eq!(outcome.report.source_code_points(), 1);
    assert_eq!(outcome.report.retained_source_units(), 1);
    assert_eq!(outcome.report.instructions(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::SourceRetention { limit: 2 }
        ))
    );
}

#[test]
fn consumption_is_checked_first_when_both_frontend_caps_are_exhausted() {
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("1"),
            ExecutionOptions::new(64)
                .with_source_consumption_limit(0)
                .with_source_retention_limit(0),
        )
        .unwrap();
    assert_eq!(outcome.report.source_reads(), 1);
    assert_eq!(outcome.report.source_code_points(), 0);
    assert_eq!(outcome.report.retained_source_units(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::SourceConsumption { limit: 0 }
        ))
    );
}

#[test]
fn nested_eval_shares_the_retained_source_budget_and_preserves_the_host_stop() {
    let mut context = Context::default();
    let padding = " ".repeat(128);
    let script = format!(
        "globalThis.entered = true; try {{ eval('{padding}1'); }}
         catch {{ globalThis.caught = true; }}"
    );
    let limit = u64::try_from(script.len()).unwrap() + 8;
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(&script),
            ExecutionOptions::new(128)
                .with_source_retention_limit(limit)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(outcome.report.retained_source_units(), limit);
    assert_eq!(outcome.report.source_code_points(), limit);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == "frontend")
        .unwrap();
    assert_eq!(frontend.phases, 2);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::SourceRetention { limit }
        ))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "entered && typeof caught === 'undefined'"
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

impl Read for CountingReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        self.bytes.read(output)
    }
}

#[test]
fn a_source_limit_stops_before_the_next_utf8_byte_and_before_dispatch() {
    let reads = Cell::new(0);
    let source = Source::from_reader(
        CountingReader {
            bytes: "'é'".as_bytes(),
            reads: &reads,
        },
        None,
    );
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64).with_source_read_limit(2))
        .unwrap();
    assert_eq!(reads.get(), 2);
    assert_eq!(outcome.report.source_reads(), 2);
    assert_eq!(outcome.report.instructions(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
            limit: 2,
        }))
    );
}

#[test]
fn lookahead_and_eof_report_only_the_reader_calls_that_occurred() {
    let bytes = "/*é*/ 1 === 1".as_bytes();
    let reads = Cell::new(0);
    let source = Source::from_reader(
        CountingReader {
            bytes,
            reads: &reads,
        },
        None,
    );
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(true));
    assert!(reads.get() > u64::try_from(bytes.len()).unwrap());
    assert_eq!(outcome.report.source_reads(), reads.get());
}

#[test]
fn utf16_surrogate_inspection_has_its_own_source_read_admission() {
    let mut context = Context::default();
    // Quote, high surrogate, low surrogate, quote.
    let source = Source::from_utf16(&[0x27, 0xD83D, 0xDE00, 0x27]);
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(64)
                .with_source_read_limit(2)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(outcome.report.source_reads(), 2);
    assert_eq!(outcome.report.instructions(), 0);
    let profile = outcome.report.profile().unwrap();
    let utf16 = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "utf16-source")
        .unwrap();
    assert_eq!(utf16.accepted_items, 2);
    assert_eq!(utf16.refused_batches, 1);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
            limit: 2,
        }))
    );
}

#[test]
fn nested_eval_preserves_source_units_and_attributes_decoder_attempts() {
    let sources = [
        (JsString::from("'é'"), "latin1-source", &[0xE9][..], 4),
        (
            JsString::from(&[0x27, 0xD83D, 0xDE00, 0x27][..]),
            "utf16-source",
            &[0xD83D, 0xDE00][..],
            5,
        ),
        (
            JsString::from(&[0x27, 0xD800, 0x27][..]),
            "utf16-source",
            &[0xD800][..],
            // The surrogate reader probes the following quote before reading it.
            5,
        ),
    ];
    for script in ["eval(src)", "(0, eval)(src)"] {
        for (source, scope_name, expected, attempts) in &sources {
            let mut context = Context::default();
            context
                .register_global_property(js_string!("src"), source.clone(), Attribute::default())
                .unwrap();
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes(script),
                    ExecutionOptions::new(128).with_profile(ProfileMode::Summary { timing: false }),
                )
                .unwrap();
            assert_eq!(
                outcome.result.unwrap().as_string().unwrap().to_vec(),
                *expected
            );
            let retained = u64::try_from(script.len() + source.len()).unwrap();
            assert_eq!(outcome.report.retained_source_units(), retained);
            let profile = outcome.report.profile().unwrap();
            let decoder = profile
                .scopes
                .iter()
                .find(|scope| scope.name == *scope_name)
                .unwrap_or_else(|| panic!("{script}: missing {scope_name} profile"));
            assert_eq!(decoder.accepted_items, *attempts);
            assert_eq!(decoder.refused_batches, 0);
        }
    }
}

#[test]
fn nested_eval_read_refusal_uses_the_outer_budget_and_bypasses_guest_catch() {
    let sources = [
        (JsString::from("'é'"), "latin1-source"),
        (
            JsString::from(&[0x27, 0xD83D, 0xDE00, 0x27][..]),
            "utf16-source",
        ),
    ];
    for script in [
        "try { eval(src); } catch { globalThis.caught = true; }",
        "try { (0, eval)(src); } catch { globalThis.caught = true; }",
    ] {
        let mut context = Context::default();
        context
            .register_global_property(js_string!("src"), 1, Attribute::default())
            .unwrap();
        let outer = context
            .eval_with_budget(Source::from_bytes(script), ExecutionOptions::new(128))
            .unwrap();
        outer.result.unwrap();
        for (source, scope_name) in &sources {
            // Refuse the closing quote or low surrogate, then separately refuse only EOF.
            for units in [2, source.len()] {
                let admitted_units = u64::try_from(units).unwrap();
                let limit = outer.report.source_reads() + admitted_units;
                let mut context = Context::default();
                context
                    .register_global_property(
                        js_string!("src"),
                        source.clone(),
                        Attribute::default(),
                    )
                    .unwrap();
                let outcome = context
                    .eval_with_budget(
                        Source::from_bytes(script),
                        ExecutionOptions::new(128)
                            .with_source_read_limit(limit)
                            .with_profile(ProfileMode::Summary { timing: false }),
                    )
                    .unwrap();
                assert_eq!(outcome.report.source_reads(), limit);
                assert!(outcome.report.instructions() > 0);
                assert_eq!(
                    outcome.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
                        limit
                    }))
                );
                assert!(
                    context
                        .eval(Source::from_bytes("globalThis.caught"))
                        .unwrap()
                        .is_undefined()
                );
                let profile = outcome.report.profile().unwrap();
                let decoder = profile
                    .scopes
                    .iter()
                    .find(|scope| scope.name == *scope_name)
                    .unwrap_or_else(|| panic!("{script}: missing {scope_name} profile"));
                assert_eq!(decoder.accepted_items, admitted_units);
                assert_eq!(decoder.refused_batches, 1);
            }
        }
    }
}

#[test]
fn interrupted_utf8_reads_cannot_retry_past_the_source_budget() {
    struct InterruptedReader<'a>(&'a Cell<u64>);

    impl Read for InterruptedReader<'_> {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            self.0.set(self.0.get() + 1);
            Err(io::ErrorKind::Interrupted.into())
        }
    }

    let reads = Cell::new(0);
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(
            Source::from_reader(InterruptedReader(&reads), None),
            ExecutionOptions::new(64).with_source_read_limit(3),
        )
        .unwrap();
    assert_eq!(reads.get(), 3);
    assert_eq!(outcome.report.source_reads(), 3);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
            limit: 3,
        }))
    );
}

#[test]
fn opaque_decoder_runs_only_for_admitted_calls() {
    let reads = Cell::new(0);
    let mut characters = ['4', '2'].into_iter().map(u32::from);
    let source = Source::from_decoder(
        || {
            reads.set(reads.get() + 1);
            Ok(characters.next())
        },
        None,
    );
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(64)
                .with_source_read_limit(1)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(reads.get(), 1);
    assert_eq!(outcome.report.source_reads(), 1);
    assert_eq!(outcome.report.instructions(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
            limit: 1,
        }))
    );
    let decoder = outcome
        .report
        .profile()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == "host-source")
        .unwrap();
    assert_eq!(decoder.accepted_items, 1);
    assert_eq!(decoder.refused_batches, 1);
}

#[test]
fn opaque_decoder_rejects_code_points_outside_unicode() {
    let source = Source::from_decoder(|| Ok(Some(u32::MAX)), None);
    let mut context = Context::default();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64).with_source_read_limit(1))
        .unwrap();
    assert_eq!(outcome.report.source_reads(), 1);
    assert_eq!(outcome.report.instructions(), 0);
    let error = outcome.result.unwrap_err();
    let native = error.as_native().unwrap();
    assert_eq!(native.kind(), &surgeist_js::JsNativeErrorKind::Syntax);
    assert_eq!(
        native.message(),
        "host decoder returned a code point outside Unicode"
    );
}

#[test]
fn opaque_decoder_preserves_surrogate_source_units() {
    let mut input = [0x27, 0xD800, 0x27].into_iter();
    let mut context = Context::default();
    let result = context
        .eval(Source::from_decoder(|| Ok(input.next()), None))
        .unwrap();
    assert_eq!(
        result.as_string().unwrap().iter().collect::<Vec<_>>(),
        [0xD800]
    );
}

#[test]
fn guest_parser_entry_points_share_the_outer_source_cap() {
    for builtin in ["eval", "Function", "JSON.parse", "parseModule"] {
        let mut context = Context::default();
        context
            .register_global_builtin_callable(
                js_string!("parseModule"),
                0,
                NativeFunction::from_fn_ptr(|_, _, context| {
                    let source = " ".repeat(128) + "export {}";
                    surgeist_js::module::Module::parse(Source::from_bytes(&source), None, context)?;
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        let padding = " ".repeat(128);
        let script = format!(
            "globalThis.entered = true; try {{ {builtin}('{padding}1'); }}
             catch {{ globalThis.caught = true; }}"
        );
        let limit = u64::try_from(script.len()).unwrap() + 20;
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(&script),
                ExecutionOptions::new(128)
                    .with_source_read_limit(limit)
                    .with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        assert_eq!(outcome.report.source_reads(), limit, "{builtin}");
        assert!(
            outcome.report.instructions() > 0,
            "{builtin} must be reached during execution"
        );
        let frontend = outcome
            .report
            .profile()
            .unwrap()
            .scopes
            .iter()
            .find(|scope| scope.name == "frontend")
            .unwrap();
        assert_eq!(frontend.phases, 2, "{builtin}");
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
                limit,
            })),
            "{builtin}"
        );
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "entered && typeof caught === 'undefined'"
                ))
                .unwrap(),
            JsValue::new(true),
            "{builtin}"
        );
    }
}

#[test]
fn a_host_callback_cannot_swallow_the_first_source_stop() {
    let mut context = Context::default();
    context
        .register_global_builtin_callable(
            js_string!("swallowSourceStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let error = context.eval(Source::from_utf16(&[0x20; 128])).unwrap_err();
                assert!(matches!(
                    error.as_engine(),
                    Some(EngineError::RuntimeLimit(
                        RuntimeLimitError::SourceRead { .. }
                    ))
                ));
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();
    let source = "swallowSourceStop(); globalThis.afterStop = true;";
    let limit = u64::try_from(source.len()).unwrap() + 8;
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(source),
            ExecutionOptions::new(64).with_source_read_limit(limit),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
            limit,
        }))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes("typeof afterStop === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn the_file_loader_propagates_source_stops_from_dynamic_imports() {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets");
    for import in [
        "import('file1.js'); drainJobs();",
        "import('data.json', { with: { type: 'json' } }); drainJobs();",
    ] {
        let loader = Rc::new(surgeist_js::module::SimpleModuleLoader::new(&assets).unwrap());
        let mut context = Context::builder().module_loader(loader).build().unwrap();
        context
            .register_global_builtin_callable(
                js_string!("drainJobs"),
                0,
                NativeFunction::from_fn_ptr(|_, _, context| {
                    context.run_jobs()?;
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        let limit = u64::try_from(import.len()).unwrap() + 8;
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(import),
                ExecutionOptions::new(128)
                    .with_source_read_limit(limit)
                    .with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        let frontend = outcome
            .report
            .profile()
            .unwrap()
            .scopes
            .iter()
            .find(|scope| scope.name == "frontend")
            .unwrap();
        assert_eq!(frontend.phases, 2, "{import}");
        assert_eq!(outcome.report.source_reads(), limit, "{import}");
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::SourceRead {
                limit,
            })),
            "{import}"
        );
    }
}

use surgeist_js::{
    Context, JsError, JsResult, JsString, JsValue, NativeFunction, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ExecutionSetupError, ProfileMode},
    job::{Job, JobExecutor},
    js_string,
    property::Attribute,
};

#[test]
fn an_empty_script_admits_its_return_check_and_return() {
    let mut context = Context::default();

    // The empty script emits only CheckReturn and Return.
    let denied = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(1))
        .unwrap();
    assert_eq!(denied.report.instructions(), 1);
    assert_eq!(
        denied.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 1,
        }))
    );

    let exact = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(2))
        .unwrap();
    assert_eq!(exact.report.instructions(), 2);
    assert_eq!(exact.result.unwrap(), JsValue::undefined());
}

#[test]
fn a_host_stop_preserves_prior_effects_and_bypasses_guest_handlers() {
    let mut context = Context::default();
    let source = "globalThis.started = true;
        try { while (true) {} }
        catch { globalThis.caught = true; }
        finally { globalThis.finalized = true; }";
    let outcome = context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(outcome.report.instructions(), 64);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 64,
        }))
    );
    assert_eq!(
        context
            .eval(Source::from_bytes(
                "started === true && typeof caught === 'undefined' && typeof finalized === 'undefined'",
            ))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn reentrant_evaluation_cannot_hide_the_first_instruction_stop() {
    let mut context = Context::default();
    context
        .register_global_builtin_callable(
            js_string!("reenter"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                assert_eq!(
                    context
                        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(200))
                        .unwrap_err(),
                    ExecutionSetupError::AlreadyActive
                );
                let nested = context.eval(
                    Source::from_bytes("while (true) {}").with_path(Path::new("ledger-inner.js")),
                );
                assert_eq!(
                    nested.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                        limit: 32,
                    }))
                );
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("reenter(); globalThis.after = true;")
                .with_path(Path::new("ledger-outer.js")),
            ExecutionOptions::new(32),
        )
        .unwrap();
    assert_eq!(outcome.report.instructions(), 32);
    let error = outcome.result.unwrap_err();
    assert_eq!(
        error.as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 32,
        }))
    );
    assert!(error.to_string().contains("ledger-inner.js"), "{error}");
    assert_eq!(
        context
            .eval(Source::from_bytes("typeof after === 'undefined'"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn a_nested_legacy_stop_preserves_the_callers_pending_throw() {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(0);
    context
        .register_global_builtin_callable(
            js_string!("swallowStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, context| {
                let stop = context
                    .eval(Source::from_bytes("while (true) {}"))
                    .unwrap_err();
                assert_eq!(
                    stop.as_engine(),
                    Some(&EngineError::RuntimeLimit(RuntimeLimitError::LoopIteration))
                );
                Ok(JsValue::undefined())
            }),
        )
        .unwrap();

    let error = context
        .eval(Source::from_bytes(
            "try { throw 1; } catch { throw 2; } finally { swallowStop(); }",
        ))
        .unwrap_err();
    assert_eq!(error.as_opaque(), Some(&JsValue::new(2)));
}

#[derive(Default)]
struct InlinePromiseExecutor {
    error: RefCell<Option<JsError>>,
}

impl JobExecutor for InlinePromiseExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut Context) {
        let Job::PromiseJob(job) = job else {
            panic!("unexpected job kind");
        };
        if let Err(error) = job.call(context) {
            self.error.replace(Some(error));
        }
    }

    fn run_jobs(self: Rc<Self>, _: &mut Context) -> JsResult<()> {
        Ok(())
    }
}

#[test]
fn an_inline_await_stop_reaches_the_executor_and_releases_the_generator() {
    let executor = Rc::new(InlinePromiseExecutor::default());
    let mut context = Context::builder()
        .job_executor(executor.clone())
        .build()
        .unwrap();
    context
        .eval(Source::from_bytes(
            "const generator = (async function* () { await 0; while (true) {} })();",
        ))
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("generator.next()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    let expected = EngineError::RuntimeLimit(RuntimeLimitError::Instruction { limit: 256 });
    assert_eq!(outcome.result.unwrap_err().as_engine(), Some(&expected));
    assert_eq!(
        executor
            .error
            .borrow()
            .as_ref()
            .and_then(JsError::as_engine),
        Some(&expected)
    );
    context
        .eval(Source::from_bytes(
            "generator.next().then(result => { globalThis.finished = result.done; });",
        ))
        .unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("globalThis.finished"))
            .unwrap(),
        JsValue::new(true)
    );
}

#[test]
fn bounded_profiling_preserves_admission_and_reports_trace_loss() {
    let mut context = Context::default();
    let source = Source::from_bytes("");
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(2).with_profile(ProfileMode::Detailed {
                timing: false,
                event_limit: 0,
                filter: EventFilter::ScopeName("evaluation"),
            }),
        )
        .unwrap();

    assert_eq!(outcome.result.unwrap(), JsValue::undefined());
    assert_eq!(outcome.report.instructions(), 2);
    let profile = outcome.report.profile().unwrap();
    assert_eq!(profile.dropped_events, 2);
    assert!(profile.events.is_empty());
    assert_eq!(profile.elapsed, None);
}

#[test]
fn the_configured_clock_runs_only_for_timed_profiling() {
    thread_local! {
        static TICKS: Cell<u64> = const { Cell::new(0) };
    }
    fn clock() -> Duration {
        TICKS.with(|ticks| {
            let elapsed = Duration::from_millis(ticks.get());
            ticks.set(ticks.get() + 1);
            elapsed
        })
    }

    let mut context = Context::default();
    for timing in [false, true] {
        TICKS.set(0);
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(""),
                ExecutionOptions::new(2)
                    .with_profile(ProfileMode::Summary { timing })
                    .with_profile_clock(clock),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::undefined());
        assert_eq!(outcome.report.instructions(), 2);
        assert_eq!(
            outcome.report.profile().unwrap().elapsed,
            timing.then_some(Duration::from_millis(3))
        );
        let frontend = outcome
            .report
            .profile()
            .unwrap()
            .scopes
            .iter()
            .find(|scope| scope.name == "frontend")
            .unwrap();
        assert_eq!(frontend.phases, 1);
        assert_eq!(
            frontend.phase_elapsed,
            timing.then_some(Duration::from_millis(1))
        );
        assert_eq!(TICKS.get(), if timing { 4 } else { 0 });
    }
}

#[test]
fn an_instruction_stop_preserves_an_async_generator_return_already_given_to_a_job() {
    // Sweep the checkpoints of this small call so the fixture exercises the
    // stop between closing the generator and returning from its VM frame.
    for limit in 0..128 {
        let mut context = Context::default();
        context
            .eval(Source::from_bytes(
                "globalThis.generator = (async function* () {
                    generator.return(7).then(result => {
                        globalThis.drained = result.done && result.value === 7;
                    });
                })();",
            ))
            .unwrap();

        let outcome = context
            .eval_with_budget(
                Source::from_bytes("generator.next()"),
                ExecutionOptions::new(limit),
            )
            .unwrap();
        if outcome.result.is_ok() {
            break;
        }
        context.run_jobs().unwrap();
        let drained = context
            .eval(Source::from_bytes("globalThis.drained === true"))
            .unwrap();
        if drained == JsValue::new(true) {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit,
                }))
            );
            return;
        }
    }
    panic!("no stopped evaluation preserved an already scheduled generator return");
}

#[test]
fn an_instruction_stop_before_async_return_registration_discards_the_unowned_queue() {
    let mut context = Context::default();
    context
        .eval(Source::from_bytes(
            "const value = Promise.resolve(7);
             Object.defineProperty(value, 'constructor', { get() { while (true) {} } });
             const generator = (async function* () { generator.return(value); })();",
        ))
        .unwrap();

    let outcome = context
        .eval_with_budget(
            Source::from_bytes("generator.next()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 256,
        }))
    );

    context
        .eval(Source::from_bytes(
            "generator.next().then(result => { globalThis.finished = result.done; });",
        ))
        .unwrap();
    context.run_jobs().unwrap();
    assert_eq!(
        context
            .eval(Source::from_bytes("globalThis.finished"))
            .unwrap(),
        JsValue::new(true)
    );
}

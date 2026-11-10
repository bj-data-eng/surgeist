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
fn folded_literal_interning_cannot_bypass_the_operation_budget() {
    use surgeist_js::optimizer::OptimizerOptions;
    use surgeist_js_parser::frontend::InternerLimitKind;

    let source = r#""\u0100optimizerLeft" + "\u0101optimizerRight""#;
    let expected = JsValue::from(js_string!("\u{100}optimizerLeft\u{101}optimizerRight"));
    let mut unoptimized = Context::builder().build().unwrap();
    unoptimized.set_optimizer_options(OptimizerOptions::empty());
    let parsed = unoptimized
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(128))
        .unwrap();
    assert_eq!(parsed.result.unwrap(), expected);
    let source_units = parsed.report.interner_utf16_units().unwrap();
    assert!(source_units > 0);

    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: true }] {
        let options = ExecutionOptions::new(128)
            .with_interner_utf16_limit(source_units)
            .with_profile(profile);
        let mut control = Context::builder().build().unwrap();
        control.set_optimizer_options(OptimizerOptions::empty());
        let admitted_source = control
            .eval_with_budget(Source::from_bytes(source), options)
            .unwrap();
        assert_eq!(admitted_source.result.unwrap(), expected);
        assert_eq!(
            admitted_source.report.interner_utf16_units().unwrap(),
            source_units
        );

        let mut optimized = Context::builder().build().unwrap();
        optimized.set_optimizer_options(OptimizerOptions::CONSTANT_FOLDING);
        let refused_output = optimized
            .eval_with_budget(Source::from_bytes(source), options)
            .unwrap();
        assert_eq!(
            refused_output.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
                kind: InternerLimitKind::Utf16,
                limit: source_units,
            }))
        );
        assert_eq!(refused_output.report.instructions().unwrap(), 0);
        assert!(!refused_output.report.effects_started());
        assert_eq!(
            optimized.eval(Source::from_bytes(source)).unwrap(),
            expected
        );
    }
}

#[test]
fn interner_publication_admits_index_and_retained_units_together() {
    use surgeist_js_parser::frontend::InternerLimitKind;

    let input = r#""\uD800""#;
    let mut refused_index = 0;
    for limit in [3, 4] {
        let mut context = Context::builder().build().unwrap();
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(64).with_interner_utf16_limit(limit),
            )
            .unwrap();
        assert_eq!(evaluation.report.interner_utf16_units().unwrap(), limit);
        assert_eq!(evaluation.report.interner_bytes().unwrap(), 0);
        if limit == 3 {
            assert_eq!(
                evaluation.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Interner {
                    kind: InternerLimitKind::Utf16,
                    limit,
                }))
            );
            assert_eq!(evaluation.report.instructions().unwrap(), 0);
            refused_index = evaluation.report.interner_index_work().unwrap();
        } else {
            assert_eq!(
                evaluation.report.interner_index_work().unwrap(),
                refused_index + 1
            );
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

    let mut context = Context::builder().build().unwrap();
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
    assert_eq!(evaluation.report.interner_bytes().unwrap(), 0);
    assert_eq!(evaluation.report.interner_index_work().unwrap(), 0);
    assert_eq!(evaluation.report.instructions().unwrap(), 0);
}

#[test]
fn nested_eval_interner_refusal_bypasses_guest_catch() {
    use surgeist_js_parser::frontend::InternerLimitKind;

    // The outer frontend hashes and compares the four bytes of the common
    // eval name and publishes 17 UTF-8 bytes for the string literal. Its eval
    // then tries to hash the identifier in a fresh interner.
    let limit = 4 + 4 + 17;
    let mut context = Context::builder().build().unwrap();
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("try { eval('hiddenRuntimeName') } catch { 99 }"),
            ExecutionOptions::new(128).with_interner_byte_limit(limit),
        )
        .unwrap();
    assert_eq!(evaluation.report.interner_bytes().unwrap(), limit);
    assert!(evaluation.report.instructions().unwrap() > 0);
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
    let mut context = Context::builder().build().unwrap();
    for (source, limit) in [("break missing;", 0), ("a:while(false){continue a;}", 1)] {
        let refused = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(64).with_label_lookup_limit(limit),
            )
            .unwrap();
        assert_eq!(refused.report.label_lookups().unwrap(), limit);
        assert_eq!(refused.report.instructions().unwrap(), 0);
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
    assert_eq!(exact.report.label_lookups().unwrap(), 2);
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
    assert_eq!(nested.report.label_lookups().unwrap(), 0);
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
        let mut context = Context::builder().build().unwrap();
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
            assert_eq!(outcome.report.early_error_visits().unwrap(), limit);
            assert_eq!(outcome.report.grammar_dispatches().unwrap(), 5);
            if limit == 12 {
                assert_eq!(outcome.result.unwrap(), JsValue::undefined());
            } else {
                assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
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
    assert_eq!(outcome.report.early_error_visits().unwrap(), 512);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
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
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(SCRIPT),
                ExecutionOptions::new(128).with_early_error_visit_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.early_error_visits().unwrap(), limit);
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
    let mut context = Context::builder().build().unwrap();
    let err = context
        .eval(Source::from_bytes(r#"let x = 0 'A😀\uD800é';"#))
        .unwrap_err();
    assert_eq!(
        err.as_native().unwrap().message(),
        "expected one of ';' or 'line terminator', got 'A😀\\uD800é' in lexical declaration binding list at line 1, col 11"
    );

    let Err(err) = surgeist_js::module::Module::parse(
        Source::from_bytes("let π; let π;"),
        None,
        context.access(),
    ) else {
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
            let mut context = Context::builder().build().unwrap();
            let evaluation = context
                .eval_with_budget(
                    Source::from_bytes(SCRIPT),
                    ExecutionOptions::new(128)
                        .with_diagnostic_input_limit(input_limit)
                        .with_diagnostic_output_limit(output_limit)
                        .with_profile(profile),
                )
                .unwrap();
            assert_eq!(evaluation.report.diagnostic_input().unwrap(), units);
            assert_eq!(evaluation.report.diagnostic_output().unwrap(), bytes);
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
    let mut context = Context::builder().build().unwrap();
    let prefix = u64::try_from("undefined break target: ".len()).unwrap();
    let limit = prefix + 1;
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("break π;"),
            ExecutionOptions::new(32).with_diagnostic_output_limit(limit),
        )
        .unwrap();
    assert_eq!(evaluation.report.diagnostic_input().unwrap(), 1);
    assert_eq!(evaluation.report.diagnostic_output().unwrap(), prefix);
    assert_eq!(evaluation.report.instructions().unwrap(), 0);
    assert_eq!(
        evaluation.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::DiagnosticOutput { limit }
        ))
    );
}

#[test]
fn template_limits_share_head_and_tail_and_admit_astral_output_together() {
    const SCRIPT: &str = r#"try { (s => s[1])`a${0}\u{1F600}`; }
        catch { globalThis.caught = true; }"#;
    // The head has one unit in each pass. The tail inspects all nine escape
    // units in each pass and appends two cooked units together.
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        for (input, output, expected) in [
            (19, 13, Some(RuntimeLimitError::TemplateInput { limit: 19 })),
            (
                20,
                12,
                Some(RuntimeLimitError::TemplateOutput { limit: 12 }),
            ),
            (20, 13, None),
        ] {
            let mut context = Context::builder().build().unwrap();
            let evaluation = context
                .eval_with_budget(
                    Source::from_bytes(SCRIPT),
                    ExecutionOptions::new(128)
                        .with_template_input_limit(input)
                        .with_template_output_limit(output)
                        .with_profile(profile),
                )
                .unwrap();
            assert_eq!(evaluation.report.template_input().unwrap(), input);
            if let Some(error) = expected {
                assert_eq!(evaluation.report.template_output().unwrap(), 11);
                assert_eq!(
                    evaluation.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(error))
                );
                assert_eq!(evaluation.report.instructions().unwrap(), 0);
            } else {
                assert_eq!(evaluation.report.template_output().unwrap(), 13);
                assert_eq!(
                    evaluation.result.unwrap().as_string().unwrap().to_vec(),
                    [0xD83D, 0xDE00]
                );
            }
        }
    }
}

#[test]
fn template_budget_refuses_before_the_invalid_braced_escape_is_diagnosed() {
    for (script, tagged) in [(r"`\u{z}`", false), (r"(s => s[0])`\u{z}`", true)] {
        for input in [9, 10] {
            let mut context = Context::builder().build().unwrap();
            let evaluation = context
                .eval_with_budget(
                    Source::from_bytes(script),
                    ExecutionOptions::new(128).with_template_input_limit(input),
                )
                .unwrap();
            assert_eq!(evaluation.report.template_input().unwrap(), input);
            assert_eq!(evaluation.report.template_output().unwrap(), 5);
            if input == 9 {
                assert_eq!(
                    evaluation.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::TemplateInput { limit: 9 }
                    ))
                );
            } else if tagged {
                assert!(evaluation.result.unwrap().is_undefined());
            } else {
                assert!(
                    evaluation
                        .result
                        .unwrap_err()
                        .as_native()
                        .unwrap()
                        .is_syntax()
                );
            }
        }
    }
}

#[test]
fn template_normalization_and_cooking_charge_crlf_and_cached_lookahead() {
    for (script, input, output, cooked) in [
        ("`a\r\nb\\\r\nc`", 12, 10, &[0x61, 0x0A, 0x62, 0x63][..]),
        (r"`\0x`", 6, 5, &[0, 0x78][..]),
    ] {
        let mut context = Context::builder().build().unwrap();
        let evaluation = context
            .eval_with_budget(
                Source::from_bytes(script),
                ExecutionOptions::new(128)
                    .with_template_input_limit(input)
                    .with_template_output_limit(output),
            )
            .unwrap();
        assert_eq!(evaluation.report.template_input().unwrap(), input);
        assert_eq!(evaluation.report.template_output().unwrap(), output);
        assert_eq!(
            evaluation.result.unwrap().as_string().unwrap().to_vec(),
            cooked
        );
    }
}

#[test]
fn scope_refusal_leaves_realm_bindings_and_identities_ready_for_retry() {
    const SOURCE: &str = "let pending = 40; { let extra = 2; pending + extra; }";
    let mut completed = Context::builder().build().unwrap();
    let full = completed
        .eval_with_budget(Source::from_bytes(SOURCE), ExecutionOptions::new(128))
        .unwrap();
    assert_eq!(full.result.unwrap(), JsValue::new(42));
    let work = full.report.scope_work().unwrap();
    assert!(work > 1);

    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut context = Context::builder().build().unwrap();
        let limit = work - 1;
        let refused = context
            .eval_with_budget(
                Source::from_bytes(SOURCE),
                ExecutionOptions::new(128)
                    .with_scope_work_limit(limit)
                    .with_profile(profile),
            )
            .unwrap();
        assert_eq!(refused.report.scope_work().unwrap(), limit);
        assert_eq!(refused.report.instructions().unwrap(), 0);
        assert_eq!(
            refused.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
                limit
            }))
        );
        assert!(
            context
                .realm()
                .scope()
                .get_binding(&js_string!("pending"))
                .is_none()
        );
        let retry = context.eval(Source::from_bytes(SOURCE)).unwrap();
        assert_eq!(retry, JsValue::new(42));
        let retried_id =
            surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false).unique_id();
        let mut uninterrupted = Context::builder().build().unwrap();
        uninterrupted.eval(Source::from_bytes(SOURCE)).unwrap();
        let uninterrupted_id =
            surgeist_js_ast::scope::Scope::new(uninterrupted.realm().scope().clone(), false)
                .unique_id();
        assert_eq!(retried_id, uninterrupted_id);
    }
}

#[test]
fn scope_preparation_has_separate_analysis_compilation_and_installation_phases() {
    let mut context = Context::builder().build().unwrap();
    let evaluation = context
        .eval_with_budget(
            Source::from_bytes("{}"),
            ExecutionOptions::new(32).with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    evaluation.result.unwrap();
    let profile = evaluation.report.profile().unwrap().unwrap();
    let phase = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "scope-analysis")
        .unwrap();
    // Default optimization edits the syntax, then scope analysis runs again.
    assert_eq!(phase.phases, 2);
    assert!(evaluation.report.scope_work().unwrap() > 0);
    assert!(
        profile
            .scopes
            .iter()
            .find(|scope| scope.name == "scope-node")
            .unwrap()
            .accepted_items
            > 0
    );
    let compilation = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "bytecode-compilation")
        .unwrap();
    let installation = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "script-installation")
        .unwrap();
    assert!(compilation.phases > 0);
    assert!(installation.phases > 0);
    let publication = profile
        .scopes
        .iter()
        .find(|scope| scope.name == "scope-publication" && scope.parent == Some(installation.scope))
        .unwrap();
    assert!(publication.accepted_items > 0);
    assert_eq!(
        profile
            .scopes
            .iter()
            .find(|scope| scope.name == "scope-publication" && scope.parent == Some(phase.scope))
            .unwrap()
            .accepted_items,
        0,
    );
    assert!(phase.phase_elapsed.is_none());
    let next = surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false);
    assert_eq!(next.unique_id(), 2);
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
            let mut context = Context::builder().build().unwrap();
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
            assert_eq!(outcome.report.source_reads().unwrap(), limit, "{setup}");
            let utf16 = outcome
                .report
                .profile()
                .unwrap()
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
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(Source::from_bytes(script), ExecutionOptions::new(64))
            .unwrap();
        outcome.result.unwrap();
        assert_eq!(
            outcome.report.source_reads().unwrap(),
            u64::try_from(script.len()).unwrap() + 1 + nested_reads
        );
    }
}

#[test]
fn dynamic_function_body_preserves_utf16_literals_and_wrapped_error_lines() {
    let mut context = Context::builder().build().unwrap();
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
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(SCRIPT),
                ExecutionOptions::new(128).with_direct_eval_visit_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.direct_eval_visits().unwrap(), limit);
        assert!(outcome.report.instructions().unwrap() > 0);
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
    let mut context = Context::builder().build().unwrap();
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
        assert_eq!(evaluation.report.name_comparisons().unwrap(), limit);
    }
}

#[test]
fn dynamic_function_private_parameters_charge_resolution_before_publication() {
    let input = "typeof Function('x = class { #a; #b; m() { return this.#b; } }', '')";
    let mut context = Context::builder().build().unwrap();
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
        assert_eq!(evaluation.report.name_comparisons().unwrap(), limit);
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
        let mut context = Context::builder().build().unwrap();
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
        assert_eq!(evaluation.report.name_comparisons().unwrap(), limit);
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
        let mut context = Context::builder().build().unwrap();
        let options = ExecutionOptions::new(128);
        let record_limit = records - 1;
        let denied = context
            .eval_with_budget(
                Source::from_bytes(source),
                options.with_name_record_limit(record_limit),
            )
            .unwrap();
        assert!(denied.report.instructions().unwrap() > 0);
        assert_eq!(denied.report.name_records().unwrap(), record_limit);
        assert_eq!(
            denied.report.name_comparisons().unwrap(),
            earlier_comparisons
        );
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
            assert_eq!(outcome.report.name_records().unwrap(), records);
            assert_eq!(outcome.report.name_comparisons().unwrap(), limit);
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
    let mut context = Context::builder().build().unwrap();
    for limit in [0, 4, 5] {
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(";"),
                ExecutionOptions::new(64).with_grammar_dispatch_limit(limit),
            )
            .unwrap();
        assert_eq!(outcome.report.grammar_dispatches().unwrap(), limit);
        if limit == 0 {
            assert_eq!(outcome.report.source_reads().unwrap(), 0);
            assert_eq!(outcome.report.token_probes().unwrap(), 0);
        }
        if limit == 5 {
            assert_eq!(outcome.result.unwrap(), JsValue::undefined());
        } else {
            assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
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
    assert_eq!(outcome.report.grammar_dispatches().unwrap(), 512);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
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
        let mut context = Context::builder().build().unwrap();
        let options = ExecutionOptions::new(64).with_profile(profile);
        let probe = context
            .eval_with_budget(Source::from_bytes("1;"), options.with_token_probe_limit(0))
            .unwrap();
        assert_eq!(probe.report.token_probes().unwrap(), 0);
        assert_eq!(probe.report.source_reads().unwrap(), 0);
        assert_eq!(probe.report.instructions().unwrap(), 0);
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
        assert!(advance.report.token_probes().unwrap() > 0);
        assert_eq!(advance.report.token_advances().unwrap(), 0);
        assert_eq!(advance.report.instructions().unwrap(), 0);
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
        assert!(admitted.report.token_probes().unwrap() > 0);
        assert_eq!(admitted.report.token_advances().unwrap(), 2);
    }
}

#[test]
fn lexical_dispatch_limit_counts_discarded_comments_and_refuses_before_tokenization() {
    let mut context = Context::builder().build().unwrap();
    let input = "/*x*/1";
    let refused = context
        .eval_with_budget(
            Source::from_bytes(input),
            ExecutionOptions::new(64).with_lexer_dispatch_limit(1),
        )
        .unwrap();
    assert_eq!(refused.report.lexer_dispatches().unwrap(), 1);
    assert_eq!(refused.report.source_code_points().unwrap(), 6);
    assert_eq!(refused.report.instructions().unwrap(), 0);
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
    assert_eq!(admitted.report.lexer_dispatches().unwrap(), 2);
    assert_eq!(admitted.result.unwrap(), JsValue::new(1));
}

#[test]
fn lookahead_refusal_preserves_the_partial_source_and_profile_equivalence() {
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("/*x*/1"),
                ExecutionOptions::new(64)
                    .with_lexer_lookahead_limit(0)
                    .with_profile(profile),
            )
            .unwrap();
        assert_eq!(outcome.report.source_reads().unwrap(), 1);
        assert_eq!(outcome.report.source_code_points().unwrap(), 1);
        assert_eq!(outcome.report.retained_source_units().unwrap(), 1);
        assert_eq!(outcome.report.lexer_lookahead_probes().unwrap(), 0);
        assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("swallowLookaheadStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, mut context| {
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
    assert_eq!(outcome.report.lexer_lookahead_probes().unwrap(), 64);
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
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(input),
                ExecutionOptions::new(64)
                    .with_source_consumption_limit(limit)
                    .with_source_retention_limit(12),
            )
            .unwrap();
        assert_eq!(outcome.report.source_code_points().unwrap(), expected);
        assert_eq!(
            outcome.report.retained_source_units().unwrap(),
            expected + 1
        );
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
            assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("'😀'"),
            ExecutionOptions::new(64).with_source_retention_limit(2),
        )
        .unwrap();
    assert_eq!(outcome.report.source_reads().unwrap(), 5);
    assert_eq!(outcome.report.source_code_points().unwrap(), 1);
    assert_eq!(outcome.report.retained_source_units().unwrap(), 1);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::SourceRetention { limit: 2 }
        ))
    );
}

#[test]
fn consumption_is_checked_first_when_both_frontend_caps_are_exhausted() {
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("1"),
            ExecutionOptions::new(64)
                .with_source_consumption_limit(0)
                .with_source_retention_limit(0),
        )
        .unwrap();
    assert_eq!(outcome.report.source_reads().unwrap(), 1);
    assert_eq!(outcome.report.source_code_points().unwrap(), 0);
    assert_eq!(outcome.report.retained_source_units().unwrap(), 0);
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::SourceConsumption { limit: 0 }
        ))
    );
}

#[test]
fn nested_eval_shares_the_retained_source_budget_and_preserves_the_host_stop() {
    let mut context = Context::builder().build().unwrap();
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
    assert_eq!(outcome.report.retained_source_units().unwrap(), limit);
    assert_eq!(outcome.report.source_code_points().unwrap(), limit);
    let frontend = outcome
        .report
        .profile()
        .unwrap()
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64).with_source_read_limit(2))
        .unwrap();
    assert_eq!(reads.get(), 2);
    assert_eq!(outcome.report.source_reads().unwrap(), 2);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(true));
    assert!(reads.get() > u64::try_from(bytes.len()).unwrap());
    assert_eq!(outcome.report.source_reads().unwrap(), reads.get());
}

#[test]
fn utf16_surrogate_inspection_has_its_own_source_read_admission() {
    let mut context = Context::builder().build().unwrap();
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
    assert_eq!(outcome.report.source_reads().unwrap(), 2);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    let profile = outcome.report.profile().unwrap().unwrap();
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
            let mut context = Context::builder().build().unwrap();
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
            assert_eq!(outcome.report.retained_source_units().unwrap(), retained);
            let profile = outcome.report.profile().unwrap().unwrap();
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
        let mut context = Context::builder().build().unwrap();
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
                let limit = outer.report.source_reads().unwrap() + admitted_units;
                let mut context = Context::builder().build().unwrap();
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
                assert_eq!(outcome.report.source_reads().unwrap(), limit);
                assert!(outcome.report.instructions().unwrap() > 0);
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
                let profile = outcome.report.profile().unwrap().unwrap();
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_reader(InterruptedReader(&reads), None),
            ExecutionOptions::new(64).with_source_read_limit(3),
        )
        .unwrap();
    assert_eq!(reads.get(), 3);
    assert_eq!(outcome.report.source_reads().unwrap(), 3);
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(64)
                .with_source_read_limit(1)
                .with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(reads.get(), 1);
    assert_eq!(outcome.report.source_reads().unwrap(), 1);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(source, ExecutionOptions::new(64).with_source_read_limit(1))
        .unwrap();
    assert_eq!(outcome.report.source_reads().unwrap(), 1);
    assert_eq!(outcome.report.instructions().unwrap(), 0);
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
    let mut context = Context::builder().build().unwrap();
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
        let mut context = Context::builder().build().unwrap();
        context
            .register_global_builtin_callable(
                js_string!("parseModule"),
                0,
                NativeFunction::from_fn_ptr(|_, _, mut context| {
                    let source = " ".repeat(128) + "export {}";
                    surgeist_js::module::Module::parse(
                        Source::from_bytes(&source),
                        None,
                        context.reborrow(),
                    )?;
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
        assert_eq!(outcome.report.source_reads().unwrap(), limit, "{builtin}");
        assert!(
            outcome.report.instructions().unwrap() > 0,
            "{builtin} must be reached during execution"
        );
        let frontend = outcome
            .report
            .profile()
            .unwrap()
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
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("swallowSourceStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, mut context| {
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
                NativeFunction::from_fn_ptr(|_, _, mut context| {
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
            .unwrap()
            .scopes
            .iter()
            .find(|scope| scope.name == "frontend")
            .unwrap();
        assert_eq!(frontend.phases, 2, "{import}");
        assert_eq!(outcome.report.source_reads().unwrap(), limit, "{import}");
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
    Context, ContextAccess, JsError, JsResult, JsString, JsValue, NativeFunction, Source,
    error::{EngineError, RuntimeLimitError},
    execution::{EventFilter, ExecutionOptions, ExecutionSetupError, ProfileMode},
    job::{
        Job, JobAcceptance, JobCall, JobExecutor, JobInvocation, JobPublicationCause,
        JobPublicationError,
    },
    js_string,
    property::Attribute,
};

#[test]
fn an_empty_script_admits_its_return_check_and_return() {
    let mut context = Context::builder().build().unwrap();

    // The empty script emits only CheckReturn and Return.
    let denied = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(1))
        .unwrap();
    assert_eq!(denied.report.instructions().unwrap(), 1);
    assert_eq!(
        denied.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
            limit: 1,
        }))
    );

    let exact = context
        .eval_with_budget(Source::from_bytes(""), ExecutionOptions::new(2))
        .unwrap();
    assert_eq!(exact.report.instructions().unwrap(), 2);
    assert_eq!(exact.result.unwrap(), JsValue::undefined());
}

#[test]
fn a_host_stop_preserves_prior_effects_and_bypasses_guest_handlers() {
    let mut context = Context::builder().build().unwrap();
    let source = "globalThis.started = true;
        try { while (true) {} }
        catch { globalThis.caught = true; }
        finally { globalThis.finalized = true; }";
    let outcome = context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(64))
        .unwrap();
    assert_eq!(outcome.report.instructions().unwrap(), 64);
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
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_builtin_callable(
            js_string!("reenter"),
            0,
            NativeFunction::from_fn_ptr(|_, _, mut context| {
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
    assert_eq!(outcome.report.instructions().unwrap(), 32);
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
    let mut context = Context::builder().build().unwrap();
    context.runtime_limits_mut().set_loop_iteration_limit(0);
    context
        .register_global_builtin_callable(
            js_string!("swallowStop"),
            0,
            NativeFunction::from_fn_ptr(|_, _, mut context| {
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
    deferred: RefCell<Vec<Job>>,
}

impl JobExecutor for InlinePromiseExecutor {
    fn enqueue_job(
        self: Rc<Self>,
        job: Job,
        mut context: ContextAccess<'_>,
    ) -> Result<JobAcceptance, JobPublicationError> {
        if let Err(cause) = job.check_publication(context.reborrow()) {
            return Err(JobPublicationError::new(job, cause));
        }
        let Job::PromiseJob(job) = job else {
            panic!("unexpected job kind");
        };
        match job.call(context.reborrow()) {
            JobCall::Complete(Ok(_)) => Ok(JobAcceptance::Accepted),
            JobCall::Complete(Err(error)) => {
                let cause = JobPublicationCause::try_new(error.clone(), context.reborrow());
                self.error.replace(Some(error));
                match cause {
                    Ok(cause) => Ok(JobAcceptance::AcceptedWithError(cause)),
                    // Preserve the fixture's recorded catchable callback-error policy.
                    Err(_catchable) => Ok(JobAcceptance::Accepted),
                }
            }
            JobCall::Deferred { job, error } => {
                self.deferred.borrow_mut().push(job.into());
                self.error.replace(Some(error));
                Ok(JobAcceptance::Accepted)
            }
        }
    }

    fn run_jobs(self: Rc<Self>, _: ContextAccess<'_>) -> JsResult<()> {
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
    let mut context = Context::builder().build().unwrap();
    let source = Source::from_bytes("");
    let outcome = context
        .eval_with_budget(
            source,
            ExecutionOptions::new(2).with_profile(ProfileMode::Detailed {
                timing: false,
                event_limit: 0,
                filter: EventFilter::ScopeName("execution"),
            }),
        )
        .unwrap();

    assert_eq!(outcome.result.unwrap(), JsValue::undefined());
    assert_eq!(outcome.report.instructions().unwrap(), 2);
    let profile = outcome.report.profile().unwrap().unwrap();
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

    let mut context = Context::builder().build().unwrap();
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
        assert_eq!(outcome.report.instructions().unwrap(), 2);
        let report = outcome.report.profile().unwrap().unwrap();
        for (name, expected_phases) in [
            ("frontend", Some(1)),
            ("scope-analysis", Some(2)),
            ("bytecode-compilation", None),
            ("script-installation", None),
        ] {
            let phase = report
                .scopes
                .iter()
                .find(|scope| scope.name == name)
                .unwrap();
            if let Some(expected) = expected_phases {
                assert_eq!(phase.phases, expected);
            }
            assert!(phase.phases > 0);
            assert_eq!(
                phase.phase_elapsed,
                timing.then_some(Duration::from_millis(phase.phases))
            );
        }
        // Every phase contributes its own start/end sample; operation timing
        // contributes one sample at each end. The synthetic clock advances 1ms
        // per observation, independently of any charge count.
        let phases: u64 = report.scopes.iter().map(|scope| scope.phases).sum();
        assert_eq!(
            report.elapsed,
            timing.then_some(Duration::from_millis(2 * phases + 1))
        );
        assert_eq!(TICKS.get(), if timing { 2 * phases + 2 } else { 0 });
    }
}

#[test]
fn an_instruction_stop_preserves_an_async_generator_return_already_given_to_a_job() {
    // Sweep the checkpoints of this small call so the fixture exercises the
    // stop between closing the generator and returning from its VM frame.
    for limit in 0..128 {
        let mut context = Context::builder().build().unwrap();
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
    let mut context = Context::builder().build().unwrap();
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

#[test]
fn preparation_and_first_instruction_refusals_report_no_execution_effect() {
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        for (source, options) in [
            ("42", ExecutionOptions::new(32).with_source_read_limit(0)),
            ("42", ExecutionOptions::new(32).with_scope_work_limit(0)),
            ("let =", ExecutionOptions::new(32)),
            ("42", ExecutionOptions::new(0)),
        ] {
            let mut context = Context::builder().build().unwrap();
            let outcome = context
                .eval_with_budget(Source::from_bytes(source), options.with_profile(profile))
                .unwrap();
            assert!(outcome.result.is_err());
            assert!(!outcome.report.effects_started());
            assert_eq!(outcome.report.instructions().unwrap(), 0);
        }
    }
}

#[test]
fn an_instruction_stop_retains_execution_effect_status() {
    let mut context = Context::builder().build().unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("globalThis.retainedEffect = 41; while (true) {}"),
            ExecutionOptions::new(32),
        )
        .unwrap();
    assert!(outcome.result.is_err());
    assert!(outcome.report.effects_started());
    assert_eq!(outcome.report.instructions().unwrap(), 32);
    assert_eq!(
        context.eval(Source::from_bytes("retainedEffect")).unwrap(),
        JsValue::new(41)
    );
}

#[test]
fn nested_preparation_refusal_cannot_clear_execution_effect_status() {
    let mut context = Context::builder().build().unwrap();
    context
        .register_global_callable(
            js_string!("nestedRefusal"),
            0,
            NativeFunction::from_fn_ptr(|_, _, mut context| {
                context.eval(Source::from_bytes("let ="))
            }),
        )
        .unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("nestedRefusal()"),
            ExecutionOptions::new(32),
        )
        .unwrap();
    assert!(outcome.result.is_err());
    assert!(outcome.report.effects_started());
    assert!(outcome.report.instructions().unwrap() > 0);
}

#[test]
fn late_refusals_in_each_preparation_phase_preserve_fresh_source_retry() {
    const SOURCE: &str = "let pending = seed + 2; (class {}); pending";
    fn seeded() -> Context {
        let mut context = Context::builder().build().unwrap();
        context.eval(Source::from_bytes("let seed = 40;")).unwrap();
        context
    }
    let mut baseline = seeded();
    let complete = baseline
        .eval_with_budget(
            Source::from_bytes(SOURCE),
            ExecutionOptions::new(256).with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(complete.result.unwrap(), JsValue::new(42));
    let work = surgeist_js::execution::ExecutionWork::ScopeWork;
    let report = complete.report.profile().unwrap().unwrap();
    let phase_work = |name| {
        report
            .scopes
            .iter()
            .find(|scope| scope.name == name)
            .unwrap()
            .usage
            .work(work)
    };
    let analysis = phase_work("scope-analysis");
    let compilation = phase_work("bytecode-compilation");
    let installation = phase_work("script-installation");
    assert!(analysis > 0 && compilation > 0 && installation > 0);
    assert_eq!(
        analysis + compilation + installation,
        complete.report.scope_work().unwrap()
    );
    let next_baseline =
        surgeist_js_ast::scope::Scope::new(baseline.realm().scope().clone(), false).unique_id();
    for limit in [
        analysis - 1,
        analysis + compilation - 1,
        analysis + compilation + installation - 1,
    ] {
        for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
            let mut context = seeded();
            let stopped = context
                .eval_with_budget(
                    Source::from_bytes(SOURCE),
                    ExecutionOptions::new(256)
                        .with_scope_work_limit(limit)
                        .with_profile(profile),
                )
                .unwrap();
            assert_eq!(
                stopped.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
                    limit
                }))
            );
            assert_eq!(stopped.report.instructions().unwrap(), 0);
            assert!(!stopped.report.effects_started());
            assert!(!context.realm().scope().has_binding(&js_string!("pending")));
            assert!(context.realm().scope().has_binding(&js_string!("seed")));
            assert_eq!(
                context.eval(Source::from_bytes(SOURCE)).unwrap(),
                JsValue::new(42)
            );
            let next = surgeist_js_ast::scope::Scope::new(context.realm().scope().clone(), false)
                .unique_id();
            assert_eq!(next, next_baseline);
        }
    }
}

#[test]
fn optimizer_work_boundaries_preserve_results_and_first_refusal() {
    use surgeist_js::optimizer::OptimizerOptions;
    // The constant-folding walk of `1 + 2` visits 6 typed nodes, then 2
    // in its stable check. The driver visits 4; there are 2 iterations,
    // 4 attempts and 1 rewrite. Retirement quotes 6 typed nodes, with one
    // inspection and one reservation each: 12 + 2 + 4 + 1 + 12 = 31.
    const EXACT: u64 = 31;
    let modes = [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Detailed {
            timing: false,
            event_limit: 8,
            filter: EventFilter::All,
        },
    ];
    for profile in modes {
        for limit in [EXACT - 1, EXACT] {
            let mut context = Context::builder().build().unwrap();
            context.set_optimizer_options(OptimizerOptions::CONSTANT_FOLDING);
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes("1 + 2"),
                    ExecutionOptions::new(64)
                        .with_optimizer_work_limit(limit)
                        .with_profile(profile),
                )
                .unwrap();
            if limit == EXACT {
                assert_eq!(outcome.result.unwrap(), JsValue::new(3));
                assert_eq!(outcome.report.optimizer_work().unwrap(), EXACT);
            } else {
                assert_eq!(
                    outcome.result.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::OptimizerWork { limit }
                    ))
                );
                assert_eq!(outcome.report.optimizer_work().unwrap(), limit);
                assert_eq!(outcome.report.instructions().unwrap(), 0);
                assert!(!outcome.report.effects_started());
                assert_eq!(
                    context.eval(Source::from_bytes("1 + 2")).unwrap(),
                    JsValue::new(3)
                );
            }
        }
    }
}

#[test]
fn disabled_optimizer_ignores_a_zero_optimizer_budget() {
    use surgeist_js::optimizer::OptimizerOptions;
    let mut context = Context::builder().build().unwrap();
    context.set_optimizer_options(OptimizerOptions::empty());
    let result = context
        .eval_with_budget(
            Source::from_bytes("1 + 2"),
            ExecutionOptions::new(64).with_optimizer_work_limit(0),
        )
        .unwrap();
    assert_eq!(result.result.unwrap(), JsValue::new(3));
    assert_eq!(result.report.optimizer_work().unwrap(), 0);
}

#[test]
fn optimizer_refusal_keeps_lexical_names_and_script_effects_private() {
    use surgeist_js::optimizer::OptimizerOptions;
    let mut context = Context::builder().build().unwrap();
    context.set_optimizer_options(OptimizerOptions::OPTIMIZE_ALL);
    let source = "let pendingOptimizer = 1 + 2; pendingOptimizer";
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(source),
            ExecutionOptions::new(128).with_optimizer_work_limit(1),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::OptimizerWork { limit: 1 }
        ))
    );
    assert!(!outcome.report.effects_started());
    assert_eq!(outcome.report.instructions().unwrap(), 0);
    assert!(
        context
            .realm()
            .scope()
            .get_binding(&js_string!("pendingOptimizer"))
            .is_none()
    );
    assert_eq!(
        context.eval(Source::from_bytes(source)).unwrap(),
        JsValue::new(3)
    );
}

#[test]
fn caught_optimizer_refusal_remains_the_operation_first_stop() {
    use surgeist_js::optimizer::OptimizerOptions;
    fn prepare_inner(
        _: &JsValue,
        _: &[JsValue],
        mut context: surgeist_js::ContextAccess<'_>,
    ) -> surgeist_js::JsResult<JsValue> {
        // Rust may catch a preparation error, but it cannot restore the parent
        // operation's authority to publish a successful result.
        let _ = context.eval(Source::from_bytes("1+2+3+4+5+6+7+8+9+10+11+12+13+14+15+16"));
        Ok(JsValue::new(42))
    }
    let mut context = Context::builder().build().unwrap();
    context.set_optimizer_options(OptimizerOptions::CONSTANT_FOLDING);
    context
        .register_global_builtin_callable(
            js_string!("prepareInner"),
            0,
            NativeFunction::from_fn_ptr(prepare_inner),
        )
        .unwrap();
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("prepareInner()"),
            ExecutionOptions::new(128).with_optimizer_work_limit(64),
        )
        .unwrap();
    assert!(outcome.report.effects_started());
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(
            RuntimeLimitError::OptimizerWork { limit: 64 }
        ))
    );
    assert_eq!(
        context.eval(Source::from_bytes("prepareInner()")).unwrap(),
        JsValue::new(42)
    );
}

#[test]
fn optimizer_large_literals_and_wide_deep_inputs_preserve_profile_decisions() {
    use surgeist_js::optimizer::OptimizerOptions;
    let long = "x".repeat(1024);
    let wide = vec!["1+2"; 64].join(";");
    let nested = format!("{}1{}", "!".repeat(48), "");
    let cases = [
        (
            format!("'{long}' + '\\u0100'"),
            JsValue::from(js_string!(format!("{long}Ā"))),
        ),
        (
            "(18446744073709551617n * 4294967297n).toString()".to_owned(),
            JsValue::from(js_string!("79228162532711081671548469249")),
        ),
        (wide, JsValue::new(3)),
        (nested, JsValue::new(true)),
        (
            "if (false) { var retained; } typeof retained".to_owned(),
            JsValue::from(js_string!("undefined")),
        ),
    ];
    for (source, expected) in cases {
        let mut usage = None;
        for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
            let mut context = Context::builder().build().unwrap();
            context.set_optimizer_options(OptimizerOptions::OPTIMIZE_ALL);
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes(&source),
                    ExecutionOptions::new(4096).with_profile(profile),
                )
                .unwrap();
            assert_eq!(outcome.result.unwrap(), expected, "{source}");
            let admitted = outcome.report.optimizer_work().unwrap();
            if let Some(usage) = usage {
                assert_eq!(admitted, usage);
            }
            usage = Some(admitted);
            let mut refused_context = Context::builder().build().unwrap();
            refused_context.set_optimizer_options(OptimizerOptions::OPTIMIZE_ALL);
            let limit = admitted - 1;
            let refused = refused_context
                .eval_with_budget(
                    Source::from_bytes(&source),
                    ExecutionOptions::new(4096)
                        .with_profile(profile)
                        .with_optimizer_work_limit(limit),
                )
                .unwrap();
            assert_eq!(
                refused.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::OptimizerWork { limit }
                ))
            );
            assert_eq!(refused.report.instructions().unwrap(), 0);
            assert!(!refused.report.effects_started());
        }
    }
}

#[test]
fn unlimited_optimizer_preserves_bigint_power_guards_and_zero_results() {
    use surgeist_js::optimizer::OptimizerOptions;
    let mut context = Context::builder().build().unwrap();
    context.set_optimizer_options(OptimizerOptions::CONSTANT_FOLDING);
    let source = "let ranged = false; try { 1n ** 18446744073709551616n; } catch (error) { ranged = error instanceof RangeError; } ranged && 0n ** 18446744073709551616n === 0n && 0n ** 0n === 1n";
    let result = context
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(512))
        .unwrap();
    assert_eq!(result.result.unwrap(), JsValue::new(true));
}

#[test]
fn optimizer_phase_timing_is_selected_by_the_operation_profile() {
    fn clock() -> Duration {
        thread_local! { static TICKS: Cell<u64> = const { Cell::new(0) }; }
        TICKS.with(|ticks| {
            let next = ticks.get() + 1;
            ticks.set(next);
            Duration::from_nanos(next)
        })
    }
    for timing in [false, true] {
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("1 + 2"),
                ExecutionOptions::new(64)
                    .with_profile(ProfileMode::Summary { timing })
                    .with_profile_clock(clock),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(3));
        let profile = outcome.report.profile().unwrap().unwrap();
        let optimizer = profile
            .scopes
            .iter()
            .find(|scope| scope.name == "optimizer")
            .unwrap();
        assert_eq!(optimizer.phases, 1);
        assert_eq!(optimizer.phase_elapsed.is_some(), timing);
    }
}

fn no_nested_preparation(
    _: &JsValue,
    _: &[JsValue],
    _: surgeist_js::ContextAccess<'_>,
) -> JsResult<JsValue> {
    Ok(JsValue::new(42))
}

fn construct_selected_function(
    _: &JsValue,
    _: &[JsValue],
    mut context: surgeist_js::ContextAccess<'_>,
) -> JsResult<JsValue> {
    let constructor = context
        .global_object()
        .unwrap()
        .get(js_string!("SelectedConstructor"), context.reborrow())?
        .as_object()
        .unwrap();
    constructor.construct(
        &[JsValue::from(js_string!(
            "let privateParameter = 1; return privateParameter"
        ))],
        None,
        context.reborrow(),
    )?;
    Ok(JsValue::new(42))
}

fn selected_constructor_context(expression: &str, callback: NativeFunction) -> Context {
    let mut context = Context::builder().build().unwrap();
    let constructor = context.eval(Source::from_bytes(expression)).unwrap();
    context
        .register_global_property(
            js_string!("SelectedConstructor"),
            constructor,
            Attribute::all(),
        )
        .unwrap();
    context
        .register_global_builtin_callable(js_string!("prepareNested"), 0, callback)
        .unwrap();
    context
}

fn assert_constructor_uses_the_shared_scope_budget(expression: &str) {
    let mut control = selected_constructor_context(
        expression,
        NativeFunction::from_fn_ptr(no_nested_preparation),
    );
    let control = control
        .eval_with_budget(
            Source::from_bytes("prepareNested()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let limit = control.report.scope_work().unwrap();
    for profile in [ProfileMode::Off, ProfileMode::Summary { timing: false }] {
        let mut unlimited = selected_constructor_context(
            expression,
            NativeFunction::from_fn_ptr(construct_selected_function),
        );
        assert_eq!(
            unlimited
                .eval_with_budget(
                    Source::from_bytes("prepareNested()"),
                    ExecutionOptions::new(256)
                )
                .unwrap()
                .result
                .unwrap(),
            JsValue::new(42)
        );
        let mut context = selected_constructor_context(
            expression,
            NativeFunction::from_fn_ptr(construct_selected_function),
        );
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("prepareNested()"),
                ExecutionOptions::new(256)
                    .with_scope_work_limit(limit)
                    .with_profile(profile),
            )
            .unwrap();
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
                limit
            })),
            "{expression}"
        );
        assert_eq!(outcome.report.scope_work().unwrap(), limit);
        assert!(outcome.report.effects_started());
    }
}

#[test]
fn non_script_function_preparation_uses_the_operation_scope_budget() {
    assert_constructor_uses_the_shared_scope_budget("Function");
}

#[test]
fn non_script_async_function_preparation_uses_the_operation_scope_budget() {
    assert_constructor_uses_the_shared_scope_budget("(async function () {}).constructor");
}

#[test]
fn non_script_generator_preparation_uses_the_operation_scope_budget() {
    assert_constructor_uses_the_shared_scope_budget("(function* () {}).constructor");
}

#[test]
fn non_script_async_generator_preparation_uses_the_operation_scope_budget() {
    assert_constructor_uses_the_shared_scope_budget("(async function* () {}).constructor");
}

#[test]
fn non_script_eval_preparation_uses_the_operation_scope_budget() {
    for source in [
        "eval('let privateEval = 1; privateEval')",
        "(0, eval)('let privateEval = 1; privateEval')",
    ] {
        let mut control = Context::builder().build().unwrap();
        control
            .register_global_builtin_callable(
                js_string!("eval"),
                1,
                NativeFunction::from_fn_ptr(no_nested_preparation),
            )
            .unwrap();
        let control = control
            .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(256))
            .unwrap();
        assert_eq!(control.result.unwrap(), JsValue::new(42));
        let limit = control.report.scope_work().unwrap();
        let mut context = Context::builder().build().unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(256).with_scope_work_limit(limit),
            )
            .unwrap();
        assert_eq!(
            outcome.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
                limit
            })),
            "{source}"
        );
        assert_eq!(outcome.report.scope_work().unwrap(), limit);
        assert!(outcome.report.effects_started());
    }
}

#[test]
fn non_script_module_preparation_uses_the_operation_scope_budget() {
    fn prepare_module(
        _: &JsValue,
        _: &[JsValue],
        mut context: surgeist_js::ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let module = surgeist_js::Module::parse(
            Source::from_bytes("export const privateModule = 1;"),
            None,
            context.reborrow(),
        )?;
        let loaded = module.load(context.reborrow())?;
        assert!(matches!(
            loaded.state().unwrap(),
            surgeist_js::builtins::promise::PromiseState::Fulfilled(_)
        ));
        module.link(context.reborrow())?;
        Ok(JsValue::new(42))
    }
    fn context(callback: NativeFunction) -> Context {
        let mut context = Context::builder().build().unwrap();
        context
            .register_global_builtin_callable(js_string!("prepareNested"), 0, callback)
            .unwrap();
        context
    }
    let mut control = context(NativeFunction::from_fn_ptr(no_nested_preparation));
    let control = control
        .eval_with_budget(
            Source::from_bytes("prepareNested()"),
            ExecutionOptions::new(256),
        )
        .unwrap();
    let limit = control.report.scope_work().unwrap();
    assert_eq!(control.result.unwrap(), JsValue::new(42));
    let mut unlimited = context(NativeFunction::from_fn_ptr(prepare_module));
    assert_eq!(
        unlimited
            .eval_with_budget(
                Source::from_bytes("prepareNested()"),
                ExecutionOptions::new(256)
            )
            .unwrap()
            .result
            .unwrap(),
        JsValue::new(42)
    );
    let mut context = context(NativeFunction::from_fn_ptr(prepare_module));
    let outcome = context
        .eval_with_budget(
            Source::from_bytes("prepareNested()"),
            ExecutionOptions::new(256).with_scope_work_limit(limit),
        )
        .unwrap();
    assert_eq!(
        outcome.result.unwrap_err().as_engine(),
        Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
            limit
        }))
    );
    assert_eq!(outcome.report.scope_work().unwrap(), limit);
    assert!(outcome.report.effects_started());
}

fn assert_private_frontdoor_boundary(
    make_context: impl Fn() -> Context,
    source: &str,
    unpublished_name: Option<&str>,
) {
    let mut baseline = make_context();
    let complete = baseline
        .eval_with_budget(
            Source::from_bytes(source),
            ExecutionOptions::new(512).with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(complete.result.unwrap(), JsValue::new(42), "{source}");
    let total = complete.report.scope_work().unwrap();
    let work = surgeist_js::execution::ExecutionWork::ScopeWork;
    let profile = complete.report.profile().unwrap().unwrap();
    let mut phase_total = 0;
    for name in [
        "scope-analysis",
        "bytecode-compilation",
        "script-installation",
    ] {
        let phase = profile
            .scopes
            .iter()
            .find(|phase| phase.name == name)
            .unwrap();
        let charged = phase.usage.work(work);
        assert!(charged > 0, "{source}: {name}");
        phase_total += charged;
    }
    assert_eq!(phase_total, total, "{source}");

    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::All,
        },
    ] {
        let mut exact = make_context();
        let accepted = exact
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(512)
                    .with_scope_work_limit(total)
                    .with_profile(mode),
            )
            .unwrap();
        assert_eq!(accepted.result.unwrap(), JsValue::new(42), "{source}");
        assert_eq!(accepted.report.scope_work().unwrap(), total);

        let mut context = make_context();
        let root = context.realm().scope().clone();
        let before = surgeist_js_ast::scope::Scope::new(root.clone(), false).unique_id();
        let limit = total - 1;
        let refused = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(512)
                    .with_scope_work_limit(limit)
                    .with_profile(mode),
            )
            .unwrap();
        assert_eq!(
            refused.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::ScopeWork {
                limit
            })),
            "{source}"
        );
        assert!(refused.report.scope_work().unwrap() <= limit);
        let after = surgeist_js_ast::scope::Scope::new(root, false).unique_id();
        assert_eq!(after, before + 1, "{source}: refused private scope IDs");
        if let Some(name) = unpublished_name {
            let name = JsString::from(name);
            assert!(
                !context.realm().scope().has_binding(&name),
                "{source}: refused declaration was published"
            );
            assert!(
                !context
                    .global_object()
                    .unwrap()
                    .has_own_property(name, context.access())
                    .unwrap(),
                "{source}: refused global variable was published"
            );
        }
        assert_eq!(
            context.eval(Source::from_bytes(source)).unwrap(),
            JsValue::new(42),
            "{source}: fresh source retry"
        );
    }
}

#[test]
fn non_script_constructor_exact_boundaries_preserve_profiles_and_fresh_retry() {
    for constructor in [
        "Function",
        "(async function () {}).constructor",
        "(function* () {}).constructor",
        "(async function* () {}).constructor",
    ] {
        assert_private_frontdoor_boundary(
            || {
                selected_constructor_context(
                    constructor,
                    NativeFunction::from_fn_ptr(construct_selected_function),
                )
            },
            "prepareNested()",
            None,
        );
    }
}

#[test]
fn non_script_eval_exact_boundaries_preserve_profiles_and_fresh_retry() {
    for source in [
        "eval('var pendingEvalName = 1; 42')",
        "(0, eval)('var pendingEvalName = 1; 42')",
        "eval('\"use strict\"; var pendingEvalName = 1; 42')",
        "(0, eval)('\"use strict\"; var pendingEvalName = 1; 42')",
    ] {
        assert_private_frontdoor_boundary(
            || Context::builder().build().unwrap(),
            source,
            Some("pendingEvalName"),
        );
    }
}

fn compilation_profile_work(source: &str) -> u64 {
    fn total(
        usage: &surgeist_ledger::Usage<
            surgeist_js::execution::ExecutionWork,
            surgeist_ledger::NoDimension,
        >,
    ) -> u64 {
        <surgeist_js::execution::ExecutionWork as surgeist_ledger::Dimension>::ALL
            .iter()
            .map(|&dimension| usage.work(dimension))
            .sum()
    }
    let mut context = Context::builder().build().unwrap();
    context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
    let outcome = context
        .eval_with_budget(
            Source::from_bytes(source),
            ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false }),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap(), JsValue::new(0));
    let report = outcome.report.profile().unwrap().unwrap();
    let compilation = report
        .scopes
        .iter()
        .find(|scope| scope.name == "bytecode-compilation")
        .unwrap();
    total(&compilation.usage)
}

#[test]
fn compiler_statement_lowering_records_additional_literal_work() {
    let narrow = compilation_profile_work("0;");
    let wide = compilation_profile_work(&format!("{}0;", "0;".repeat(128)));
    assert!(
        wide > narrow,
        "lowering 129 literals charged {wide}, one charged {narrow}"
    );
}

#[test]
fn compiler_dynamic_function_lowering_records_additional_literal_work() {
    let narrow = compilation_profile_work("Function('return 0;')()");
    let wide = compilation_profile_work(&format!("Function('{}return 0;')()", "0;".repeat(128)));
    assert!(
        wide > narrow,
        "nested function lowering charged {wide}, narrow charged {narrow}"
    );
}

#[test]
fn compiler_eval_lowering_records_additional_literal_work() {
    let narrow = compilation_profile_work("eval('0;')");
    let wide = compilation_profile_work(&format!("eval('{}0;')", "0;".repeat(128)));
    assert!(
        wide > narrow,
        "nested eval lowering charged {wide}, narrow charged {narrow}"
    );
}

#[test]
fn compiler_json_lowering_records_additional_literal_work() {
    fn parse_selected(
        _: &JsValue,
        _: &[JsValue],
        mut context: surgeist_js::ContextAccess<'_>,
    ) -> JsResult<JsValue> {
        let input = context
            .global_object()
            .unwrap()
            .get(js_string!("selectedJsonInput"), context.reborrow())?;
        let json = context
            .global_object()
            .unwrap()
            .get(js_string!("JSON"), context.reborrow())?
            .as_object()
            .unwrap();
        let parse = json
            .get(js_string!("parse"), context.reborrow())?
            .as_object()
            .unwrap();
        parse.call(&JsValue::undefined(), &[input], context.reborrow())?;
        Ok(JsValue::new(0))
    }
    fn run(input: &str) -> u64 {
        fn total(
            usage: &surgeist_ledger::Usage<
                surgeist_js::execution::ExecutionWork,
                surgeist_ledger::NoDimension,
            >,
        ) -> u64 {
            <surgeist_js::execution::ExecutionWork as surgeist_ledger::Dimension>::ALL
                .iter()
                .map(|&dimension| usage.work(dimension))
                .sum()
        }
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
        context
            .register_global_property(
                js_string!("selectedJsonInput"),
                JsString::from(input),
                Attribute::all(),
            )
            .unwrap();
        context
            .register_global_builtin_callable(
                js_string!("parseSelectedJson"),
                0,
                NativeFunction::from_fn_ptr(parse_selected),
            )
            .unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("parseSelectedJson()"),
                ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(0));
        let profile = outcome.report.profile().unwrap().unwrap();
        let compilation = profile
            .scopes
            .iter()
            .find(|scope| scope.name == "bytecode-compilation")
            .unwrap();
        total(&compilation.usage)
    }
    let narrow = run("[0]");
    let wide = run(&format!("[{}0]", "0,".repeat(128)));
    assert!(
        wide > narrow,
        "JSON lowering 129 literals charged {wide}, one charged {narrow}"
    );
}

fn assert_compiler_boundary(source: &str, effects_before_compilation_stop: bool) {
    fn context() -> Context {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
        context
    }
    let mut baseline = context();
    let complete = baseline
        .eval_with_budget(Source::from_bytes(source), ExecutionOptions::new(4096))
        .unwrap();
    assert_eq!(complete.result.unwrap(), JsValue::new(42), "{source}");
    let work = complete.report.compiler_work().unwrap();
    let depth = complete.report.compiler_depth_peak();
    let scope_work = complete.report.scope_work().unwrap();
    let name_visits = complete.report.name_visits().unwrap();
    assert!(work > 0 && depth > 0, "{source}");

    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::All,
        },
    ] {
        let mut exact = context();
        let accepted = exact
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(4096)
                    .with_compiler_work_limit(work)
                    .with_compiler_depth_limit(depth)
                    .with_profile(mode),
            )
            .unwrap();
        assert_eq!(accepted.result.unwrap(), JsValue::new(42), "{source}");
        assert_eq!(accepted.report.compiler_work().unwrap(), work, "{source}");
        assert_eq!(accepted.report.compiler_depth_peak(), depth, "{source}");
        assert_eq!(
            accepted.report.scope_work().unwrap(),
            scope_work,
            "{source}"
        );
        assert_eq!(
            accepted.report.name_visits().unwrap(),
            name_visits,
            "{source}"
        );
        if let Some(profile) = accepted.report.profile().unwrap() {
            let compilation = profile
                .scopes
                .iter()
                .find(|scope| scope.name == "bytecode-compilation")
                .unwrap();
            let dimension = surgeist_js::execution::ExecutionWork::CompilerWork;
            assert_eq!(compilation.usage.work(dimension), work, "{source}");
            let origins: u64 = profile
                .scopes
                .iter()
                .filter(|scope| scope.parent == Some(compilation.scope))
                .map(|scope| scope.usage.work(dimension))
                .sum();
            assert_eq!(
                origins, work,
                "{source}: delegated compiler work charged once"
            );
            if matches!(mode, ProfileMode::Detailed { .. }) {
                assert!(profile.events.len() <= 8);
                assert!(profile.dropped_events > 0);
            }
        } else {
            assert_eq!(mode, ProfileMode::Off);
        }

        let mut retry = context();
        let limit = work - 1;
        let refused = retry
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(4096)
                    .with_compiler_work_limit(limit)
                    .with_profile(mode),
            )
            .unwrap();
        assert_eq!(
            refused.result.unwrap_err().as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::CompilerWork { limit }
            )),
            "{source}"
        );
        assert!(refused.report.compiler_work().unwrap() <= limit, "{source}");
        assert_eq!(
            refused.report.effects_started(),
            effects_before_compilation_stop,
            "{source}"
        );
        assert_eq!(
            retry.eval(Source::from_bytes(source)).unwrap(),
            JsValue::new(42),
            "{source}: fresh retry"
        );
    }
}

#[test]
fn compiler_exact_boundaries_preserve_lowering_profiles_and_private_retry() {
    assert_compiler_boundary(
        "function* values(a, ...rest) { try { yield a; } finally { rest.length; } }\n\
         let {a = 1, ...tail} = {a: 2, b: 3};\n\
         class Box { #value = 40; read(delta = 2) { return this.#value + delta; } }\n\
         let iterator = values(a, ...[tail.b]); iterator.next(); iterator.return();\n\
         new Box()?.read?.()",
        false,
    );
}

#[test]
fn compiler_eval_and_constructor_exact_boundaries_preserve_host_stops() {
    for source in [
        "eval('let compiledEvalValue = 21; compiledEvalValue * 2')",
        "(0, eval)('\"use strict\"; let compiledEvalValue = 21; compiledEvalValue * 2')",
        "Function('value', 'return value * 2;')(21)",
        "(async function(){}).constructor('return 42;'); 42",
        "(function*(){}).constructor('yield 42;'); 42",
        "(async function*(){}).constructor('yield 42;'); 42",
    ] {
        assert_compiler_boundary(source, true);
    }
}

#[test]
fn compiler_json_exact_boundaries_preserve_reviver_sources_and_proto_data() {
    assert_compiler_boundary(
        r#"(function () { let parsed = JSON.parse('{"__proto__":{"polluted":true},"a":20,"b":22}',
            function(key, value, context) {
                if (key === 'a' && context.source !== '20') throw Error('source');
                return value;
            });
            return Object.hasOwn(parsed, '__proto__') && !({}).polluted ? parsed.a + parsed.b : 0;
            })()"#,
        true,
    );
    assert_compiler_boundary(
        "try { JSON.parse('[20,22]'); } catch (error) { throw Error('caught host refusal'); } 42",
        true,
    );
}

#[test]
fn compiler_class_anonymous_queries_charge_parenthesized_searches() {
    fn searches(source: &str) -> u64 {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(source),
                ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        let profile = outcome.report.profile().unwrap().unwrap();
        let phase = profile
            .scopes
            .iter()
            .find(|scope| scope.name == "bytecode-compilation")
            .unwrap();
        profile
            .scopes
            .iter()
            .find(|scope| scope.parent == Some(phase.scope) && scope.name == "compiler-search")
            .unwrap()
            .usage
            .work(surgeist_js::execution::ExecutionWork::CompilerWork)
    }
    let mut observations = Vec::new();
    for (field, result, expected_name) in [
        ("f", "new A().f", "f"),
        ("static f", "A.f", "f"),
        ("static #f", "A.read()", "#f"),
    ] {
        let source = |wrappers: usize| {
            format!(
                "class A {{ {field} = {}function() {{}}{}; static read() {{ return this.#f; }} }} {result}.name === '{expected_name}' ? 42 : 0",
                "(".repeat(wrappers),
                ")".repeat(wrappers),
            )
        };
        // Only the private-field case can refer to #f in a method.
        let source = |wrappers| {
            let source = source(wrappers);
            if field == "static #f" {
                source
            } else {
                source.replace("static read() { return this.#f; }", "")
            }
        };
        observations.push((field, searches(&source(1)), searches(&source(9))));
    }
    assert!(
        observations
            .iter()
            .all(|(_, narrow, wide)| *wide >= *narrow + 8),
        "anonymous classification must admit every extra wrapper: {observations:?}"
    );
}

#[test]
fn compiler_default_export_anonymous_query_charges_parenthesized_searches() {
    fn searches(wrappers: usize) -> u64 {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
        let source = format!(
            "export default {}function() {{}}{};",
            "(".repeat(wrappers),
            ")".repeat(wrappers)
        );
        let module =
            surgeist_js::Module::parse(Source::from_bytes(&source), None, context.access())
                .unwrap();
        let loaded = module.load(context.access()).unwrap();
        context.run_jobs().unwrap();
        assert!(matches!(
            loaded.state().unwrap(),
            surgeist_js::builtins::promise::PromiseState::Fulfilled(_)
        ));
        context
            .register_global_builtin_callable(
                js_string!("linkSelected"),
                0,
                NativeFunction::from_fn_with_captures(
                    |_, _, module: &surgeist_js::Module, mut context| {
                        module.link(context.reborrow())?;
                        let evaluated = module.evaluate(context.reborrow())?;
                        context.run_jobs()?;
                        assert!(matches!(
                            evaluated.state().unwrap(),
                            surgeist_js::builtins::promise::PromiseState::Fulfilled(_)
                        ));
                        let function = module
                            .get_value(js_string!("default"), context.reborrow())?
                            .as_object()
                            .unwrap();
                        assert_eq!(
                            function.get(js_string!("name"), context.reborrow())?,
                            JsValue::from(js_string!("default"))
                        );
                        Ok(JsValue::new(42))
                    },
                    module,
                ),
            )
            .unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("linkSelected()"),
                ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        let profile = outcome.report.profile().unwrap().unwrap();
        let phase = profile
            .scopes
            .iter()
            .find(|scope| scope.name == "bytecode-compilation")
            .unwrap();
        profile
            .scopes
            .iter()
            .find(|scope| scope.parent == Some(phase.scope) && scope.name == "compiler-search")
            .unwrap()
            .usage
            .work(surgeist_js::execution::ExecutionWork::CompilerWork)
    }
    let narrow = searches(1);
    let wide = searches(9);
    assert!(
        wide >= narrow + 8,
        "default export anonymous classification charged {wide}, narrow charged {narrow}"
    );
}

#[test]
fn compiler_source_position_queries_charge_the_actual_start_path() {
    fn visits(expression: &str) -> u64 {
        let mut context = Context::builder().build().unwrap();
        context.set_optimizer_options(surgeist_js::optimizer::OptimizerOptions::empty());
        let source = format!("try {{ throw {expression}; }} catch {{}} 42;");
        let outcome = context
            .eval_with_budget(
                Source::from_bytes(&source),
                ExecutionOptions::new(4096).with_profile(ProfileMode::Summary { timing: false }),
            )
            .unwrap();
        assert_eq!(outcome.result.unwrap(), JsValue::new(42));
        let profile = outcome.report.profile().unwrap().unwrap();
        let phase = profile
            .scopes
            .iter()
            .find(|scope| scope.name == "bytecode-compilation")
            .unwrap();
        profile
            .scopes
            .iter()
            .find(|scope| scope.parent == Some(phase.scope) && scope.name == "name-visit")
            .unwrap()
            .usage
            .work(surgeist_js::execution::ExecutionWork::NameVisit)
    }
    // Both trees have four literals and three Binary nodes. Only addition's
    // left-associative start path crosses all three operators; exponentiation
    // is right-associative, so its first operand is already the start leaf.
    let left_spine = visits("1+2+3+4");
    let right_spine = visits("1**2**3**4");
    assert!(
        left_spine > right_spine,
        "source-position queries charged left spine {left_spine}, right spine {right_spine}"
    );
}

mod async_operation_regressions {
    fn native_completed<J>(
        call: surgeist_js::job::JobCall<J>,
    ) -> surgeist_js::JsResult<surgeist_js::JsValue> {
        match call {
            surgeist_js::job::JobCall::Complete(result) => result,
            surgeist_js::job::JobCall::Deferred { .. } => panic!("expected an admitted native job"),
        }
    }
    fn reusable_completed(
        call: surgeist_js::job::JobInvocation,
    ) -> surgeist_js::JsResult<surgeist_js::JsValue> {
        match call {
            surgeist_js::job::JobInvocation::Complete(result) => result,
            surgeist_js::job::JobInvocation::Deferred(_) => {
                panic!("expected an admitted reusable job")
            }
        }
    }

    use super::*;
    use surgeist_js::{
        Script,
        job::{NativeJob, NativeJobFn},
    };

    #[test]
    fn successful_inline_reactions_keep_the_latest_async_generator_continuation() {
        let executor = Rc::new(InlinePromiseExecutor::default());
        let mut context = Context::builder()
            .job_executor(executor.clone())
            .build()
            .unwrap();
        context
            .eval(Source::from_bytes(
                "globalThis.results = [];
                 globalThis.generator = (async function* () {
                     await 0;
                     yield 1;
                     await 0;
                     yield 2;
                     return 3;
                 })();",
            ))
            .unwrap();

        for expected in [
            "[[1,false]]",
            "[[1,false],[2,false]]",
            "[[1,false],[2,false],[3,true]]",
            "[[1,false],[2,false],[3,true],[null,true]]",
        ] {
            context
                .eval(Source::from_bytes(
                    "generator.next().then(result => results.push([result.value, result.done]));",
                ))
                .unwrap();
            assert!(
                executor.error.borrow().is_none(),
                "inline reaction failed: {:?}",
                executor.error.borrow()
            );
            assert_eq!(
                context
                    .eval(Source::from_bytes("JSON.stringify(results)"))
                    .unwrap(),
                JsValue::from(JsString::from(expected))
            );
        }
    }

    #[cfg(feature = "experimental")]
    #[test]
    fn inline_native_array_await_preserves_the_generators_later_continuation() {
        let executor = Rc::new(InlinePromiseExecutor::default());
        let mut context = Context::builder()
            .job_executor(executor.clone())
            .build()
            .unwrap();
        context
            .eval(Source::from_bytes(
                "globalThis.results = [];
                 globalThis.nativeInput = new Promise(resolve => {
                     globalThis.resolveNativeInput = resolve;
                 });
                 globalThis.generator = (async function* () {
                     yield Array.fromAsync({ 0: nativeInput, length: 1 });
                     await 0;
                     yield 2;
                     return 3;
                 })();
                 generator.next().then(result => results.push([result.value, result.done]));",
            ))
            .unwrap();
        assert!(executor.error.borrow().is_none());
        assert_eq!(
            context
                .eval(Source::from_bytes("JSON.stringify(results)"))
                .unwrap(),
            JsValue::from(js_string!("[]"))
        );
        // Array.fromAsync's array-like coroutine awaits the still-pending
        // element. Resolve it after the first next() has suspended. Async
        // generator yield awaits the resulting array promise before yielding.
        context
            .eval(Source::from_bytes("resolveNativeInput(1)"))
            .unwrap();
        assert!(
            executor.error.borrow().is_none(),
            "native await reaction failed: {:?}",
            executor.error.borrow()
        );
        assert_eq!(
            context
                .eval(Source::from_bytes("JSON.stringify(results)"))
                .unwrap(),
            JsValue::from(js_string!("[[[1],false]]"))
        );
        for expected in [
            "[[[1],false],[2,false]]",
            "[[[1],false],[2,false],[3,true]]",
            "[[[1],false],[2,false],[3,true],[null,true]]",
        ] {
            context
                .eval(Source::from_bytes(
                    "generator.next().then(result => results.push([result.value, result.done]));",
                ))
                .unwrap();
            assert!(
                executor.error.borrow().is_none(),
                "later inline reaction failed: {:?}",
                executor.error.borrow()
            );
            assert_eq!(
                context
                    .eval(Source::from_bytes("JSON.stringify(results)"))
                    .unwrap(),
                JsValue::from(JsString::from(expected))
            );
        }
    }

    #[derive(Clone)]
    struct NativeJobStopProbe {
        reusable: bool,
        with_realm: bool,
        calls: Rc<Cell<u32>>,
        first_stop: Rc<RefCell<Option<JsError>>>,
        results: Rc<RefCell<Vec<JsResult<JsValue>>>>,
    }

    fn assert_native_job_stops_before_its_closure(reusable: bool) {
        const LIMIT: u64 = 64;
        for with_realm in [false, true] {
            let probe = NativeJobStopProbe {
                reusable,
                with_realm,
                calls: Rc::default(),
                first_stop: Rc::default(),
                results: Rc::default(),
            };
            let mut context = Context::builder().build().unwrap();
            let _ = context.insert_data(probe.clone());
            context
                .register_global_builtin_callable(
                    js_string!("probeStoppedJob"),
                    0,
                    NativeFunction::from_fn_ptr(|_, _, mut context| {
                        let first_stop = context
                            .eval(Source::from_bytes("while (true) {}"))
                            .unwrap_err();
                        let probe = context.get_data::<NativeJobStopProbe>().unwrap().clone();
                        probe.first_stop.replace(Some(first_stop));
                        let calls = probe.calls.clone();
                        let callback = move |_: ContextAccess<'_>| {
                            calls.set(calls.get() + 1);
                            Ok(JsValue::undefined())
                        };
                        if probe.reusable {
                            let job = if probe.with_realm {
                                NativeJobFn::with_realm(callback, context.realm().clone())
                            } else {
                                NativeJobFn::new(callback)
                            };
                            for _ in 0..2 {
                                let JobInvocation::Deferred(error) = job.call(context.reborrow())
                                else {
                                    panic!("the retained stop must defer the reusable callback")
                                };
                                probe.results.borrow_mut().push(Err(error));
                            }
                        } else {
                            let job = if probe.with_realm {
                                NativeJob::with_realm(callback, context.realm().clone())
                            } else {
                                NativeJob::new(callback)
                            };
                            let JobCall::Deferred {
                                job: _uncalled,
                                error,
                            } = job.call(context.reborrow())
                            else {
                                panic!("the retained stop must return its uncalled owned job")
                            };
                            probe.results.borrow_mut().push(Err(error));
                        }
                        // Deliberately swallow the Rust errors, as a host caller can.
                        Ok(JsValue::undefined())
                    }),
                )
                .unwrap();
            let outcome = context
                .eval_with_budget(
                    Source::from_bytes("probeStoppedJob()"),
                    ExecutionOptions::new(LIMIT),
                )
                .unwrap();
            let first_stop = probe.first_stop.borrow().clone().unwrap();
            assert_eq!(
                first_stop.as_engine(),
                Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                    limit: LIMIT
                }))
            );
            assert_eq!(outcome.result.unwrap_err(), first_stop);
            assert_eq!(outcome.report.instructions().unwrap(), LIMIT);
            assert_eq!(
                probe.calls.get(),
                0,
                "a stopped job invoked its host closure"
            );
            let results = probe.results.borrow();
            assert_eq!(results.len(), if reusable { 2 } else { 1 });
            for result in results.iter() {
                assert_eq!(result.as_ref().unwrap_err(), &first_stop);
            }
        }
    }

    #[test]
    fn native_job_after_a_swallowed_typed_stop_never_invokes_its_host_closure() {
        assert_native_job_stops_before_its_closure(false);
    }

    #[test]
    fn native_job_fn_after_a_swallowed_typed_stop_never_invokes_its_host_closure() {
        assert_native_job_stops_before_its_closure(true);
    }

    #[derive(Clone)]
    struct PostInvocationStopProbe {
        reusable: bool,
        returns_guest_error: bool,
        target_realm: surgeist_js::realm::Realm,
        calls: Rc<Cell<u32>>,
        entered_target_realm: Rc<Cell<bool>>,
        restored_caller_realm: Rc<Cell<bool>>,
        first_stop: Rc<RefCell<Option<JsError>>>,
        job_result: Rc<RefCell<Option<JsResult<JsValue>>>>,
    }

    fn assert_job_preserves_its_closures_first_stop(reusable: bool, returns_guest_error: bool) {
        const LIMIT: u64 = 64;
        let mut context = Context::builder().build().unwrap();
        let caller_realm = context.realm().clone();
        let target_realm = context.create_realm().unwrap();
        assert_ne!(target_realm, caller_realm);
        let probe = PostInvocationStopProbe {
            reusable,
            returns_guest_error,
            target_realm,
            calls: Rc::default(),
            entered_target_realm: Rc::default(),
            restored_caller_realm: Rc::default(),
            first_stop: Rc::default(),
            job_result: Rc::default(),
        };
        let _ = context.insert_data(probe.clone());
        context
            .register_global_builtin_callable(
                js_string!("probeJobCallbackStop"),
                0,
                NativeFunction::from_fn_ptr(|_, _, mut context| {
                    let probe = context
                        .get_data::<PostInvocationStopProbe>()
                        .unwrap()
                        .clone();
                    let caller_realm = context.realm().clone();
                    let callback = |mut context: ContextAccess<'_>| {
                        let probe = context
                            .get_data::<PostInvocationStopProbe>()
                            .unwrap()
                            .clone();
                        probe.calls.set(probe.calls.get() + 1);
                        probe
                            .entered_target_realm
                            .set(context.realm() == &probe.target_realm);
                        let stop = context
                            .eval(Source::from_bytes("while (true) {}"))
                            .unwrap_err();
                        probe.first_stop.replace(Some(stop));
                        if probe.returns_guest_error {
                            Err(surgeist_js::JsNativeError::typ()
                                .with_message("guest callback result after the host stop")
                                .into())
                        } else {
                            Ok(JsValue::new(99))
                        }
                    };
                    let result = if probe.reusable {
                        reusable_completed(
                            NativeJobFn::with_realm(callback, probe.target_realm.clone())
                                .call(context.reborrow()),
                        )
                    } else {
                        native_completed(
                            NativeJob::with_realm(callback, probe.target_realm.clone())
                                .call(context.reborrow()),
                        )
                    };
                    probe
                        .restored_caller_realm
                        .set(context.realm() == &caller_realm);
                    probe.job_result.replace(Some(result));
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        let outcome = context
            .eval_with_budget(
                Source::from_bytes("probeJobCallbackStop()"),
                ExecutionOptions::new(LIMIT),
            )
            .unwrap();
        let first_stop = probe.first_stop.borrow().clone().unwrap();
        assert_eq!(
            first_stop.as_engine(),
            Some(&EngineError::RuntimeLimit(RuntimeLimitError::Instruction {
                limit: LIMIT
            }))
        );
        assert_eq!(outcome.result.unwrap_err(), first_stop);
        assert_eq!(outcome.report.instructions().unwrap(), LIMIT);
        assert_eq!(probe.calls.get(), 1);
        assert!(probe.entered_target_realm.get());
        assert!(probe.restored_caller_realm.get());
        assert_eq!(context.realm(), &caller_realm);
        assert_eq!(
            probe
                .job_result
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .unwrap_err(),
            &first_stop
        );
    }

    #[test]
    fn native_job_preserves_a_swallowed_stop_when_its_closure_returns_success() {
        assert_job_preserves_its_closures_first_stop(false, false);
    }

    #[test]
    fn native_job_preserves_a_swallowed_stop_when_its_closure_returns_a_guest_error() {
        assert_job_preserves_its_closures_first_stop(false, true);
    }

    #[test]
    fn native_job_fn_preserves_a_swallowed_stop_when_its_closure_returns_success() {
        assert_job_preserves_its_closures_first_stop(true, false);
    }

    #[test]
    fn native_job_fn_preserves_a_swallowed_stop_when_its_closure_returns_a_guest_error() {
        assert_job_preserves_its_closures_first_stop(true, true);
    }

    fn assert_panicking_native_job_restores_its_caller_realm(reusable: bool) {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let mut context = Context::builder().build().unwrap();
        let caller_realm = context.realm().clone();
        let target_realm = context.create_realm().unwrap();
        assert_ne!(target_realm, caller_realm);
        let entered_realm = target_realm.clone();
        let callback = move |context: ContextAccess<'_>| -> JsResult<JsValue> {
            assert_eq!(context.realm(), &entered_realm);
            panic!("native job callback panic");
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            if reusable {
                reusable_completed(
                    NativeJobFn::with_realm(callback, target_realm).call(context.access()),
                )
            } else {
                native_completed(
                    NativeJob::with_realm(callback, target_realm).call(context.access()),
                )
            }
        }));
        let panic = result.expect_err("the callback panic must propagate to its host caller");
        assert_eq!(
            panic.downcast_ref::<&str>(),
            Some(&"native job callback panic")
        );
        assert_eq!(context.realm(), &caller_realm);
    }

    #[test]
    fn a_panicking_native_job_restores_its_caller_realm() {
        assert_panicking_native_job_restores_its_caller_realm(false);
    }

    #[test]
    fn a_panicking_native_job_fn_restores_its_caller_realm() {
        assert_panicking_native_job_restores_its_caller_realm(true);
    }

    fn assert_nested_native_job_unwind_releases_its_invocation(reusable: bool) {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use surgeist_js::object::builtins::JsFunction;

        for host_function_call in [true, false] {
            let mut context = Context::builder().build().unwrap();
            let caller_realm = context.realm().clone();
            let fresh_function = JsFunction::from_object(
                context
                    .eval(Source::from_bytes("(function () { return 42; })"))
                    .unwrap()
                    .as_object()
                    .unwrap(),
            )
            .unwrap();
            let recursion_limit = context.runtime_limits().recursion_limit();
            // Establish the same public call's sufficient limit before the unwind.
            context.runtime_limits_mut().set_recursion_limit(1);
            assert_eq!(
                fresh_function
                    .call(&JsValue::undefined(), &[], context.access())
                    .unwrap(),
                JsValue::new(42)
            );
            context
                .runtime_limits_mut()
                .set_recursion_limit(recursion_limit);
            let target_realm = context.create_realm().unwrap();
            assert_ne!(target_realm, caller_realm);
            context.enter_realm(target_realm.clone()).unwrap();
            context
                .register_global_builtin_callable(
                    js_string!("panicHost"),
                    0,
                    NativeFunction::from_fn_ptr(|_, _, _| {
                        panic!("nested native job callback panic")
                    }),
                )
                .unwrap();
            context
                .eval(
                    Source::from_bytes(
                        "globalThis.beforePanic = 0; globalThis.finallyRan = false;
                 globalThis.panicWrapper = function panicWrapper() {
                     globalThis.beforePanic = 7;
                     try { panicHost(); } finally { globalThis.finallyRan = true; }
                 };",
                    )
                    .with_path(Path::new("owned-native-job-nested-unwind.js")),
                )
                .unwrap();
            context.enter_realm(caller_realm.clone()).unwrap();
            let baseline_frames = context.stack_trace().count();
            let callback = move |mut context: ContextAccess<'_>| {
                if host_function_call {
                    let function = context
                        .global_object()
                        .unwrap()
                        .get(js_string!("panicWrapper"), context.reborrow())?;
                    JsFunction::from_object(function.as_object().unwrap())
                        .unwrap()
                        .call(&JsValue::undefined(), &[], context.reborrow())
                } else {
                    context.eval(
                        Source::from_bytes(
                            "globalThis.beforePanic = 7;
                     try { panicHost(); } finally { globalThis.finallyRan = true; }",
                        )
                        .with_path(Path::new("owned-native-job-nested-unwind.js")),
                    )
                }
            };
            let result = catch_unwind(AssertUnwindSafe(|| {
                if reusable {
                    reusable_completed(
                        NativeJobFn::with_realm(callback, target_realm.clone())
                            .call(context.access()),
                    )
                } else {
                    native_completed(
                        NativeJob::with_realm(callback, target_realm.clone())
                            .call(context.access()),
                    )
                }
            }));
            let panic = result.expect_err("the nested callback panic must reach its host caller");
            let remaining_frames = context.stack_trace().count();
            let restored_realm = context.realm().clone();
            let fresh_stack = context
                .eval(
                    Source::from_bytes("new Error('after nested unwind').stack")
                        .with_path(Path::new("after-native-job-nested-unwind.js")),
                )
                .unwrap()
                .as_string()
                .unwrap()
                .to_std_string_escaped();
            context.enter_realm(target_realm).unwrap();
            let effects = context
                .eval(Source::from_bytes(
                    "beforePanic === 7 && finallyRan === false",
                ))
                .unwrap();
            context.enter_realm(caller_realm.clone()).unwrap();
            context.runtime_limits_mut().set_recursion_limit(1);
            let tight_call = fresh_function.call(&JsValue::undefined(), &[], context.access());
            context
                .runtime_limits_mut()
                .set_recursion_limit(recursion_limit);
            let subsequent = context
                .eval(Source::from_bytes(
                    "(() => { let value = 40; return value + 2; })()",
                ))
                .unwrap();

            assert_eq!(
                panic.downcast_ref::<&str>(),
                Some(&"nested native job callback panic")
            );
            assert_eq!(remaining_frames, baseline_frames);
            assert_eq!(restored_realm, caller_realm);
            assert!(!fresh_stack.contains("panicHost"), "{fresh_stack}");
            assert!(
                !fresh_stack.contains("owned-native-job-nested-unwind.js"),
                "{fresh_stack}"
            );
            assert_eq!(
                effects,
                JsValue::new(true),
                "cleanup rolled back an effect or ran guest finally"
            );
            assert_eq!(subsequent, JsValue::new(42));
            assert_eq!(
                tight_call.unwrap(),
                JsValue::new(42),
                "host_function_call={host_function_call}"
            );
            assert_eq!(context.stack_trace().count(), baseline_frames);
            assert_eq!(context.realm(), &caller_realm);
        }
    }

    #[test]
    fn nested_guest_unwind_in_a_native_job_releases_owned_frames_and_trace() {
        assert_nested_native_job_unwind_releases_its_invocation(false);
    }

    #[test]
    fn nested_guest_unwind_in_a_native_job_fn_releases_owned_frames_and_trace() {
        assert_nested_native_job_unwind_releases_its_invocation(true);
    }

    fn assert_native_job_without_a_realm_keeps_its_callbacks_realm_change(reusable: bool) {
        let mut context = Context::builder().build().unwrap();
        let caller_realm = context.realm().clone();
        let selected_realm = context.create_realm().unwrap();
        assert_ne!(selected_realm, caller_realm);
        let callback_realm = selected_realm.clone();
        let callback = move |mut context: ContextAccess<'_>| {
            context.enter_realm(callback_realm.clone())?;
            Ok(JsValue::new(42))
        };
        let result = if reusable {
            reusable_completed(NativeJobFn::new(callback).call(context.access()))
        } else {
            native_completed(NativeJob::new(callback).call(context.access()))
        };
        assert_eq!(result.unwrap(), JsValue::new(42));
        assert_eq!(context.realm(), &selected_realm);
    }

    #[test]
    fn native_job_without_a_realm_preserves_its_normal_callbacks_realm_change() {
        assert_native_job_without_a_realm_keeps_its_callbacks_realm_change(false);
    }

    #[test]
    fn native_job_fn_without_a_realm_preserves_its_normal_callbacks_realm_change() {
        assert_native_job_without_a_realm_keeps_its_callbacks_realm_change(true);
    }

    #[derive(Clone)]
    struct MovedStackProbe {
        reusable: bool,
        callback_source: &'static str,
        caught_expected_panic: Rc<Cell<bool>>,
    }

    fn assert_caught_moved_stack_panic_preserves_the_guest_caller(
        reusable: bool,
        native_await: bool,
    ) {
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut context = if native_await {
            Context::builder()
                .job_executor(Rc::new(InlinePromiseExecutor::default()))
                .build()
                .unwrap()
        } else {
            Context::builder().build().unwrap()
        };
        let caller_realm = context.realm().clone();
        let caught_expected_panic = Rc::new(Cell::new(false));
        let _ = context.insert_data(MovedStackProbe {
            reusable,
            callback_source: if native_await {
                "resolveMovedInput(1)"
            } else {
                "movedGenerator.next()"
            },
            caught_expected_panic: caught_expected_panic.clone(),
        });
        context
            .register_global_builtin_callable(
                js_string!("panicHost"),
                0,
                NativeFunction::from_fn_ptr(|_, _, _| panic!("moved stack host panic")),
            )
            .unwrap();
        context
            .register_global_builtin_callable(
                js_string!("reenterMovedStackAndCatch"),
                0,
                NativeFunction::from_fn_ptr(|_, _, mut context| {
                    let probe = context.get_data::<MovedStackProbe>().unwrap().clone();
                    let source = probe.callback_source;
                    let callback = move |mut context: ContextAccess<'_>| {
                        context.eval(
                            Source::from_bytes(source)
                                .with_path(Path::new("owned-moved-stack-callback.js")),
                        )
                    };
                    let result = catch_unwind(AssertUnwindSafe(|| {
                        if probe.reusable {
                            reusable_completed(NativeJobFn::new(callback).call(context.reborrow()))
                        } else {
                            native_completed(NativeJob::new(callback).call(context.reborrow()))
                        }
                    }));
                    let panic = result.expect_err("the nested fixture must reach panicHost");
                    assert_eq!(
                        panic.downcast_ref::<&str>(),
                        Some(&"moved stack host panic")
                    );
                    probe.caught_expected_panic.set(true);
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();

        // These locals stay live across the generator suspension or the native
        // coroutine's ambient snapshot. Later use makes them guest state rather
        // than an incidental operand-length assertion.
        let declarations = (0..128)
            .map(|index| format!("let p{index} = {index};"))
            .collect::<String>();
        let values = (0..128)
            .map(|index| format!("p{index}"))
            .collect::<Vec<_>>()
            .join(",");
        let setup = if native_await {
            format!(
                "globalThis.movedBeforePanic = 0;
                 globalThis.movedFinallyRan = false;
                 globalThis.movedInput = new Promise(resolve => {{
                     globalThis.resolveMovedInput = resolve;
                 }});
                 globalThis.nativePreparationValues = (function prepareNativeAwait() {{
                     {declarations}
                     globalThis.movedArray = Array.fromAsync(
                         {{ 0: movedInput, length: 1 }},
                         function mapMovedValue(value) {{
                             globalThis.movedBeforePanic = 7;
                             try {{ panicHost(); }} finally {{ globalThis.movedFinallyRan = true; }}
                             return value;
                         }}
                     );
                     return [{values}];
                 }})();"
            )
        } else {
            format!(
                "globalThis.movedBeforePanic = 0;
                 globalThis.movedFinallyRan = false;
                 globalThis.movedGenerator = (function* () {{
                     {declarations}
                     globalThis.movedBeforePanic = 7;
                     try {{ panicHost(); }} finally {{ globalThis.movedFinallyRan = true; }}
                     yield [{values}];
                 }})();"
            )
        };
        context
            .eval(Source::from_bytes(&setup).with_path(Path::new("owned-moved-stack-setup.js")))
            .unwrap();
        let baseline_frames = context.stack_trace().count();
        let sentinel = context
            .eval(Source::from_bytes(
                "(function outer() {
                     const sentinel = { value: 42 };
                     reenterMovedStackAndCatch();
                     return sentinel.value;
                 })()",
            ))
            .unwrap();
        let remaining_frames = context.stack_trace().count();
        let restored_realm = context.realm().clone();
        let effects = context
            .eval(Source::from_bytes(
                "movedBeforePanic === 7 && movedFinallyRan === false",
            ))
            .unwrap();
        let subsequent = context
            .eval(Source::from_bytes(
                "(() => { let value = 40; return value + 2; })()",
            ))
            .unwrap();
        let fresh_stack = context
            .eval(Source::from_bytes(
                "new Error('after moved-stack unwind').stack",
            ))
            .unwrap()
            .as_string()
            .unwrap()
            .to_std_string_escaped();

        assert!(caught_expected_panic.get());
        assert_eq!(
            sentinel,
            JsValue::new(42),
            "unwind changed the live guest caller's value"
        );
        assert_eq!(
            effects,
            JsValue::new(true),
            "cleanup rolled back an effect or ran guest finally"
        );
        assert_eq!(remaining_frames, baseline_frames);
        assert_eq!(restored_realm, caller_realm);
        assert_eq!(subsequent, JsValue::new(42));
        assert!(!fresh_stack.contains("panicHost"), "{fresh_stack}");
        assert!(!fresh_stack.contains("owned-moved-stack"), "{fresh_stack}");
        assert_eq!(context.stack_trace().count(), baseline_frames);
        assert_eq!(context.realm(), &caller_realm);
    }

    #[test]
    fn native_job_unwind_after_generator_resume_preserves_the_live_guest_caller() {
        assert_caught_moved_stack_panic_preserves_the_guest_caller(false, false);
    }

    #[test]
    fn native_job_fn_unwind_after_generator_resume_preserves_the_live_guest_caller() {
        assert_caught_moved_stack_panic_preserves_the_guest_caller(true, false);
    }

    #[cfg(feature = "experimental")]
    #[test]
    fn native_job_unwind_after_native_await_preserves_the_live_guest_caller() {
        assert_caught_moved_stack_panic_preserves_the_guest_caller(false, true);
    }

    #[cfg(feature = "experimental")]
    #[test]
    fn native_job_fn_unwind_after_native_await_preserves_the_live_guest_caller() {
        assert_caught_moved_stack_panic_preserves_the_guest_caller(true, true);
    }

    #[test]
    fn dropping_an_unpolled_script_future_creates_no_frames_or_effects() {
        let mut context = Context::builder().build().unwrap();
        let script = Script::parse(
            Source::from_bytes("globalThis.unpolledEffect = 1;"),
            None,
            context.access(),
        )
        .unwrap();
        let baseline_frames = context.stack_trace().count();
        let evaluation = script.evaluate_async_with_budget(context.access(), 1);
        drop(evaluation);
        assert_eq!(context.stack_trace().count(), baseline_frames);
        assert_eq!(
            context
                .eval(Source::from_bytes("typeof unpolledEffect === 'undefined'"))
                .unwrap(),
            JsValue::new(true)
        );
        assert_eq!(context.stack_trace().count(), baseline_frames);
    }

    #[test]
    fn dropping_a_polled_script_future_restores_frames_and_keeps_committed_effects() {
        use std::{
            future::Future,
            task::{Context as TaskContext, Poll, Waker},
        };

        let mut context = Context::builder().build().unwrap();
        let committed = Rc::new(Cell::new(0_u32));
        let _ = context.insert_data(committed.clone());
        context
            .register_global_builtin_callable(
                js_string!("recordCommittedEffect"),
                0,
                NativeFunction::from_fn_ptr(|_, _, context| {
                    let committed = context.get_data::<Rc<Cell<u32>>>().unwrap();
                    committed.set(committed.get() + 1);
                    Ok(JsValue::undefined())
                }),
            )
            .unwrap();
        let script = Script::parse(
            Source::from_bytes(
                "globalThis.committedValue = 7; recordCommittedEffect(); while (true) {}",
            ),
            None,
            context.access(),
        )
        .unwrap();
        let baseline_frames = context.stack_trace().count();
        {
            let mut evaluation = Box::pin(script.evaluate_async_with_budget(context.access(), 1));
            let mut task_context = TaskContext::from_waker(Waker::noop());
            // Each poll reaches a real cooperative suspension. Stop only after
            // both the guest property write and host callback have committed.
            for _ in 0..128 {
                assert!(matches!(
                    evaluation.as_mut().poll(&mut task_context),
                    Poll::Pending
                ));
                if committed.get() == 1 {
                    break;
                }
            }
            assert_eq!(committed.get(), 1, "the script never reached its effect");
        }
        assert_eq!(context.stack_trace().count(), baseline_frames);
        assert_eq!(
            committed.get(),
            1,
            "dropping the future replayed its callback"
        );
        assert_eq!(
            context.eval(Source::from_bytes("committedValue")).unwrap(),
            JsValue::new(7)
        );
        assert_eq!(
            context
                .eval(Source::from_bytes(
                    "(() => { let value = 40; return value + 2; })()"
                ))
                .unwrap(),
            JsValue::new(42)
        );
        assert_eq!(context.stack_trace().count(), baseline_frames);
    }
}

// Intended as a child of the source-module owner.
use super::*;
use crate::execution::{ExecutionOptions, ExecutionReport, ProfileMode};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn origin(report: &ExecutionReport, name: &str, dimension: usize) -> u64 {
    report
        .profile()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap()
        .usage
        .work(surgeist_ledger::Work::new(dimension).unwrap())
}

#[test]
fn actual_has_tla_metadata_origins_keep_pruned_bodies_and_early_exit_visit_counts() {
    for (text, expected, visits) in [
        ("", false, 1),
        ("async function hidden() { await 0; }", false, 5),
        (
            "async function hidden() { await 0; await 1; await 2; }",
            false,
            5,
        ),
        (
            "await (unknown + other); async function later() { await 1; }",
            true,
            6,
        ),
    ] {
        let mut context = context();
        let outcome = context
            .with_execution(
                ExecutionOptions::new(100_000).with_profile(ProfileMode::Summary { timing: false }),
                |access| Module::parse(crate::Source::from_bytes(text), None, access),
            )
            .unwrap();
        let module = outcome.result.unwrap();
        assert_eq!(
            module.kind().as_source_text().unwrap().code.has_tla,
            expected
        );
        // Empty: root query. Pruned function: root/item/list-item/declaration/function.
        // First await: root/item/list-item/statement/expression/await, without its
        // target or any suffix. Parser work has different origins and is not subtracted.
        assert_eq!(
            origin(&outcome.report, "metadata-name-visit", 12),
            visits,
            "{text}"
        );
        assert_eq!(origin(&outcome.report, "metadata-name-record", 13), 0);
        assert_eq!(origin(&outcome.report, "metadata-name-comparison", 14), 0);
        for name in [
            "metadata-name-visit",
            "metadata-name-record",
            "metadata-name-comparison",
        ] {
            assert_eq!(origin(&outcome.report, name, 31), 0);
        }
        assert!(!outcome.report.effects_started());
    }
}

#[test]
fn exported_destructuring_records_its_two_names_under_metadata_name_origins_only() {
    let mut context = context();
    let outcome = context
        .with_execution(
            ExecutionOptions::new(100_000).with_profile(ProfileMode::Summary { timing: false }),
            |access| {
                Module::parse(
                    crate::Source::from_bytes(
                        "export const { a: first, b: [second] } = sourceObject;",
                    ),
                    None,
                    access,
                )
            },
        )
        .unwrap();
    let module = outcome.result.unwrap();
    assert_eq!(
        module
            .kind()
            .as_source_text()
            .unwrap()
            .code
            .local_export_entries
            .len(),
        2
    );
    assert_eq!(origin(&outcome.report, "metadata-name-record", 13), 2);
    assert_eq!(origin(&outcome.report, "metadata-name-record", 31), 0);
    assert_eq!(origin(&outcome.report, "metadata-name-comparison", 14), 0);
    assert!(!outcome.report.effects_started());
}

#[test]
fn has_tla_one_short_name_refusal_preserves_the_ast_and_spends_no_metadata_credits() {
    let mut context = context();
    let module = Module::parse(
        crate::Source::from_bytes("await (unknown + other);"),
        None,
        context.access(),
    )
    .unwrap();
    let source_module = module.kind().as_source_text().unwrap();
    let status = source_module.status.borrow();
    let ModuleStatus::Unlinked { source } = &*status else {
        unreachable!()
    };
    for limit in [0, 5, 6] {
        let outcome = context
            .with_execution(
                ExecutionOptions::new(100_000)
                    .with_module_metadata_work_limit(0)
                    .with_name_visit_limit(limit)
                    .with_profile(ProfileMode::Summary { timing: false }),
                |access| {
                    access.into_owner().with_module_metadata(|owner| {
                        let mut control = Control::new(owner);
                        try_contains(&source.ast, ContainsSymbol::AwaitExpression, || {
                            control.admit(Work::Name(NameWork::Visit))
                        })
                    })
                },
            )
            .unwrap();
        assert_eq!(outcome.report.name_visits(), limit);
        assert_eq!(origin(&outcome.report, "metadata-name-visit", 12), limit);
        assert_eq!(outcome.report.module_metadata_work(), 0);
        assert_eq!(origin(&outcome.report, "metadata-name-visit", 31), 0);
        if limit == 6 {
            assert!(outcome.result.unwrap());
        } else {
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&crate::error::EngineError::RuntimeLimit(
                    crate::error::RuntimeLimitError::NameVisit { limit }
                ))
            );
        }
        assert!(!outcome.report.effects_started());
    }
}

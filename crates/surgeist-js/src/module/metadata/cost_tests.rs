// Intended as a child of the constructor metadata owner.
use super::*;
use crate::execution::{ExecutionOptions, ExecutionReport, ExecutionWork, ProfileMode};
use surgeist_js_ast::operations::NameWork;

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn options(limit: u64) -> ExecutionOptions {
    ExecutionOptions::new(100_000)
        .with_module_metadata_work_limit(limit)
        .with_profile(ProfileMode::Summary { timing: false })
}
fn origin(report: &ExecutionReport, name: &str, dimension: ExecutionWork) -> u64 {
    report
        .profile()
        .unwrap()
        .unwrap()
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap()
        .usage
        .work(dimension)
}

#[test]
fn path_payload_has_capacity_then_copy_byte_batches_and_none_has_no_payload_work() {
    for path in [Path::new("a"), Path::new("nested/長🙂.js")] {
        let bytes = u64::try_from(path.as_os_str().len()).unwrap();
        for (limit, accepted) in [(bytes - 1, 0), (bytes, bytes), (2 * bytes - 1, bytes)] {
            let mut context = context();
            let outcome = context
                .with_execution(options(limit), |access| {
                    access
                        .into_owner()
                        .with_module_metadata(|owner| Control::new(owner).path(Some(path)))
                })
                .unwrap();
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&crate::error::EngineError::RuntimeLimit(
                    crate::error::RuntimeLimitError::ModuleMetadataWork { limit }
                ))
            );
            assert_eq!(outcome.report.module_metadata_work().unwrap(), accepted);
            assert_eq!(
                origin(
                    &outcome.report,
                    "metadata-capacity",
                    ExecutionWork::ModuleMetadataWork
                ),
                accepted
            );
            assert_eq!(
                origin(
                    &outcome.report,
                    "metadata-copy",
                    ExecutionWork::ModuleMetadataWork
                ),
                0
            );
            assert!(!outcome.report.effects_started());
        }
        let mut context = context();
        let complete = context
            .with_execution(options(2 * bytes), |access| {
                access
                    .into_owner()
                    .with_module_metadata(|owner| Control::new(owner).path(Some(path)))
            })
            .unwrap();
        assert_eq!(complete.result.unwrap().as_deref(), Some(path));
        assert_eq!(complete.report.module_metadata_work().unwrap(), 2 * bytes);
        assert_eq!(
            origin(
                &complete.report,
                "metadata-capacity",
                ExecutionWork::ModuleMetadataWork
            ),
            bytes
        );
        assert_eq!(
            origin(
                &complete.report,
                "metadata-copy",
                ExecutionWork::ModuleMetadataWork
            ),
            bytes
        );
    }
    let mut context = context();
    let none = context
        .with_execution(options(0), |access| {
            access
                .into_owner()
                .with_module_metadata(|owner| Control::new(owner).path(None))
        })
        .unwrap();
    assert_eq!(none.result.unwrap(), None);
    assert_eq!(none.report.module_metadata_work().unwrap(), 0);
    assert!(!none.report.effects_started());
}

#[test]
fn shallow_record_copy_prices_its_declared_descriptor_without_recopying_attribute_payloads() {
    let request = ModuleRequest::new(
        JsString::from("x".repeat(512)),
        vec![ImportAttribute::new(
            JsString::from("a"),
            JsString::from("長🙂".repeat(128)),
        )]
        .into_boxed_slice(),
    );
    let bytes = u64::try_from(std::mem::size_of::<ModuleRequest>()).unwrap();
    let mut context = context();
    let refused = context
        .with_execution(options(bytes - 1), |access| {
            access
                .into_owner()
                .with_module_metadata(|owner| Control::new(owner).copy(&request))
        })
        .unwrap();
    assert_eq!(
        refused.result.unwrap_err().as_engine(),
        Some(&crate::error::EngineError::RuntimeLimit(
            crate::error::RuntimeLimitError::ModuleMetadataWork { limit: bytes - 1 }
        ))
    );
    assert_eq!(refused.report.module_metadata_work().unwrap(), 0);
    let accepted = context
        .with_execution(options(bytes), |access| {
            access
                .into_owner()
                .with_module_metadata(|owner| Control::new(owner).copy(&request))
        })
        .unwrap();
    assert_eq!(accepted.result.unwrap(), request);
    assert_eq!(accepted.report.module_metadata_work().unwrap(), bytes);
    assert_eq!(
        origin(
            &accepted.report,
            "metadata-copy",
            ExecutionWork::ModuleMetadataWork
        ),
        bytes
    );
    for name in [
        "metadata-string-scan",
        "metadata-string-copy",
        "metadata-string-allocate",
    ] {
        assert_eq!(
            origin(&accepted.report, name, ExecutionWork::ModuleMetadataWork),
            0
        );
    }
    assert!(!accepted.report.effects_started());
}

#[test]
fn delegated_string_prefixes_are_charged_once_to_metadata_without_compiler_or_interner_work() {
    for (units, scan, allocation, copy) in [
        (vec![0x61u16, 0x62, 0x63], 3u64, 3u64, 3u64),
        (vec![0xD800u16, 0x78], 2u64, 4u64, 2u64),
    ] {
        let mut interner = Interner::default();
        let symbol = interner.get_or_intern(units.as_slice());
        let prefix = scan + allocation + copy;
        for (limit, accepted) in [
            (scan - 1, 0),
            (scan, scan),
            (scan + allocation - 1, scan),
            (scan + allocation, scan + allocation),
            (prefix, prefix),
        ] {
            let mut context = context();
            let outcome = context
                .with_execution(options(limit), |access| {
                    access
                        .into_owner()
                        .with_module_metadata(|owner| Control::new(owner).string(symbol, &interner))
                })
                .unwrap();
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&crate::error::EngineError::RuntimeLimit(
                    crate::error::RuntimeLimitError::ModuleMetadataWork { limit }
                ))
            );
            assert_eq!(outcome.report.module_metadata_work().unwrap(), accepted);
            assert_eq!(
                origin(
                    &outcome.report,
                    "metadata-string-scan",
                    ExecutionWork::ModuleMetadataWork
                ),
                if accepted >= scan { scan } else { 0 }
            );
            assert_eq!(
                origin(
                    &outcome.report,
                    "metadata-string-allocate",
                    ExecutionWork::ModuleMetadataWork
                ),
                if accepted >= scan + allocation {
                    allocation
                } else {
                    0
                }
            );
            assert_eq!(
                origin(
                    &outcome.report,
                    "metadata-string-copy",
                    ExecutionWork::ModuleMetadataWork
                ),
                if accepted == prefix { copy } else { 0 }
            );
            assert_eq!(outcome.report.compiler_work().unwrap(), 0);
            assert_eq!(outcome.report.interner_bytes().unwrap(), 0);
            assert_eq!(outcome.report.interner_utf16_units().unwrap(), 0);
            assert_eq!(outcome.report.interner_index_work().unwrap(), 0);
            assert_eq!(outcome.report.name_visits().unwrap(), 0);
            assert!(!outcome.report.effects_started());
        }
    }
}

#[test]
fn delegated_name_dimensions_and_origins_remain_separate_from_a_zero_metadata_cap() {
    let mut context = context();
    let accepted = context
        .with_execution(
            options(0)
                .with_name_visit_limit(1)
                .with_name_record_limit(1)
                .with_name_comparison_limit(1),
            |access| {
                access.into_owner().with_module_metadata(|owner| {
                    let mut control = Control::new(owner);
                    control.admit(Work::Name(NameWork::Visit))?;
                    control.admit(Work::Name(NameWork::Record))?;
                    control.admit(Work::Name(NameWork::Comparison))
                })
            },
        )
        .unwrap();
    accepted.result.unwrap();
    assert_eq!(accepted.report.module_metadata_work().unwrap(), 0);
    assert_eq!(
        (
            accepted.report.name_visits().unwrap(),
            accepted.report.name_records().unwrap(),
            accepted.report.name_comparisons().unwrap()
        ),
        (1, 1, 1)
    );
    for (name, dimension) in [
        ("metadata-name-visit", ExecutionWork::NameVisit),
        ("metadata-name-record", ExecutionWork::NameRecord),
        ("metadata-name-comparison", ExecutionWork::NameComparison),
    ] {
        assert_eq!(origin(&accepted.report, name, dimension), 1);
        assert_eq!(
            origin(&accepted.report, name, ExecutionWork::ModuleMetadataWork),
            0
        );
    }
    assert!(!accepted.report.effects_started());
    let mut first = None;
    let stopped = context
        .with_execution(options(0).with_name_visit_limit(1), |access| {
            access.into_owner().with_module_metadata(|owner| {
                let mut control = Control::new(owner);
                control.admit(Work::Name(NameWork::Visit))?;
                let error = control.admit(Work::Record).unwrap_err();
                assert_eq!(
                    error.as_engine(),
                    Some(&crate::error::EngineError::RuntimeLimit(
                        crate::error::RuntimeLimitError::ModuleMetadataWork { limit: 0 }
                    ))
                );
                assert_eq!(
                    control.admit(Work::Name(NameWork::Comparison)).unwrap_err(),
                    error
                );
                first = Some(error.clone());
                Err::<(), _>(error)
            })
        })
        .unwrap();
    assert_eq!(stopped.result.unwrap_err(), first.unwrap());
    assert_eq!(stopped.report.name_visits().unwrap(), 1);
    assert_eq!(stopped.report.name_comparisons().unwrap(), 0);
    assert_eq!(stopped.report.module_metadata_work().unwrap(), 0);
}

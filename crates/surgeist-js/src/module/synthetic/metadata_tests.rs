// Intended as a child module of the synthetic-module owner.
use super::*;
use crate::{Module, execution::ExecutionOptions};

fn prepared(names: &[JsString]) -> (Context, Module, crate::execution::ExecutionReport) {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    let mut context = builder.build().unwrap();
    let steps = SyntheticModuleInitializer::from_fn_with_captures(|_, _, _| Ok(()), ());
    let outcome = context
        .with_execution(ExecutionOptions::new(100_000), |access| {
            Module::synthetic(names, steps, None, None, access)
        })
        .unwrap();
    assert!(!outcome.report.effects_started());
    (context, outcome.result.unwrap(), outcome.report)
}

#[test]
fn synthetic_uniqueness_preserves_exact_long_utf16_names_and_unpaired_surrogates() {
    let prefix = "x".repeat(512);
    let first = JsString::from(format!("{prefix}🙂a"));
    let second = JsString::from(format!("{prefix}🙂b"));
    let unpaired = JsString::from(&[0xD800u16, 0x0078][..]);
    let latin = js_string!("é");
    let utf16 = JsString::from(&[0x00E9u16][..]);
    let (_context, module, _) = prepared(&[
        first.clone(),
        latin.clone(),
        utf16,
        unpaired.clone(),
        second.clone(),
        first.clone(),
    ]);
    let crate::module::ModuleKind::Synthetic(source) = module.kind() else {
        unreachable!()
    };
    assert_eq!(source.export_names.len(), 4);
    for expected in [&first, &second, &unpaired, &latin] {
        assert!(source.export_names.contains(expected));
    }
}

#[test]
fn duplicate_synthetic_name_attempts_have_positive_work_without_additional_unique_exports() {
    let (_single_context, single, single_report) = prepared(&[js_string!("answer")]);
    let (_duplicate_context, duplicate, duplicate_report) =
        prepared(&[js_string!("answer"), js_string!("answer")]);
    let crate::module::ModuleKind::Synthetic(single) = single.kind() else {
        unreachable!()
    };
    let crate::module::ModuleKind::Synthetic(duplicate) = duplicate.kind() else {
        unreachable!()
    };
    assert_eq!(single.export_names.len(), 1);
    assert_eq!(duplicate.export_names.len(), 1);
    assert!(single.export_names.contains(&js_string!("answer")));
    assert!(duplicate.export_names.contains(&js_string!("answer")));
    assert!(duplicate_report.module_metadata_work() > single_report.module_metadata_work());
}

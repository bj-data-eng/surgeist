// Intended as a child of the synthetic module owner.
use super::*;
use crate::{execution::ExecutionOptions, module::ModuleRepr};
use std::{cell::Cell, mem::size_of};

#[derive(Clone)]
struct Capture(Rc<Cell<usize>>);
#[derive(Trace)]
struct StoredCapture(#[heap_trace(skip)] Rc<Cell<usize>>);
impl NativeCapture for Capture {
    type Stored = StoredCapture;
    fn store(&self) -> StoredCapture {
        self.0.set(self.0.get() + 1);
        StoredCapture(self.0.clone())
    }
    fn restore(stored: &StoredCapture) -> Result<Self, crate::error::RootingError> {
        Ok(Self(stored.0.clone()))
    }
}
fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}
fn initializer(stores: Rc<Cell<usize>>) -> SyntheticModuleInitializer {
    SyntheticModuleInitializer::from_fn_with_captures(|_, _, _| Ok(()), Capture(stores))
}

#[test]
fn empty_synthetic_headers_store_and_publication_follow_independently_summed_prefixes() {
    let synthetic = u64::try_from(size_of::<SyntheticModule>()).unwrap();
    let callback = u64::try_from(size_of::<StoredCallback<Capture>>()).unwrap();
    let module = u64::try_from(size_of::<ModuleRepr>()).unwrap();
    let stored_prefix = 2 + synthetic + callback;
    let total = 4 + synthetic + callback + module;
    for (limit, accepted, stores) in [
        (0, 0, 0),
        (1, 1, 0),
        (1 + synthetic, 1 + synthetic, 0),
        (2 + synthetic, 2 + synthetic, 0),
        (stored_prefix, stored_prefix, 1),
        (stored_prefix + 1, stored_prefix + 1, 1),
        (total - 1, total - 1, 1),
    ] {
        let mut context = context();
        let observed = Rc::new(Cell::new(0));
        let steps = initializer(observed.clone());
        let mut direct = None;
        let outcome = context
            .with_execution(
                ExecutionOptions::new(100_000).with_module_metadata_work_limit(limit),
                |access| {
                    let result = Module::synthetic(&[], steps, None, None, access);
                    direct = result.as_ref().err().cloned();
                    result
                },
            )
            .unwrap();
        let error = direct.expect("the next header or final publication debit must refuse");
        assert_eq!(
            error.as_engine(),
            Some(&crate::error::EngineError::RuntimeLimit(
                crate::error::RuntimeLimitError::ModuleMetadataWork { limit }
            ))
        );
        assert_eq!(outcome.result.unwrap_err(), error);
        assert_eq!(outcome.report.module_metadata_work().unwrap(), accepted);
        assert_eq!(observed.get(), stores);
        assert!(!outcome.report.effects_started());
    }
    let mut context = context();
    let stores = Rc::new(Cell::new(0));
    let steps = initializer(stores.clone());
    let outcome = context
        .with_execution(
            ExecutionOptions::new(100_000).with_module_metadata_work_limit(total),
            |access| Module::synthetic(&[], steps, None, None, access),
        )
        .unwrap();
    outcome.result.unwrap();
    assert_eq!(outcome.report.module_metadata_work().unwrap(), total);
    assert_eq!(stores.get(), 1);
    assert!(!outcome.report.effects_started());
}

#[test]
fn moving_an_owned_synthetic_path_adds_no_fictitious_payload_copy() {
    let mut context = context();
    let total = u64::try_from(
        4 + size_of::<SyntheticModule>()
            + size_of::<StoredCallback<Capture>>()
            + size_of::<ModuleRepr>(),
    )
    .unwrap();
    let path = std::path::PathBuf::from(format!("{}/module.js", "長🙂".repeat(128)));
    let expected = path.clone();
    let steps = initializer(Rc::new(Cell::new(0)));
    let outcome = context
        .with_execution(
            ExecutionOptions::new(100_000).with_module_metadata_work_limit(total),
            |access| Module::synthetic(&[], steps, Some(path), None, access),
        )
        .unwrap();
    assert_eq!(outcome.result.unwrap().path(), Some(expected.as_path()));
    assert_eq!(outcome.report.module_metadata_work().unwrap(), total);
    assert!(!outcome.report.effects_started());
}

use std::{cell::Cell, rc::Rc};

use super::{EngineError, JsError, JsNativeError, PanicError};
use crate::{Context, JsObject};

const PANIC_MESSAGE: &str = "diagnostic custody failure";
const FOREIGN_REALM_MESSAGE: &str = "error realm belongs to a different runtime";

#[derive(crate::JsData, surgeist_js_gc::heap::Trace)]
struct DisposalProbe(#[heap_trace(skip)] Rc<Cell<usize>>);

impl Drop for DisposalProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn panic_with_foreign_parent_and_local_cause(
    foreign: &Context,
    receiving: &Context,
    drops: &Rc<Cell<usize>>,
) -> JsError {
    let cause = JsObject::new_unique(receiving.runtime(), None, DisposalProbe(drops.clone()))
        .unwrap()
        .upcast();
    let diagnostic = JsNativeError::range()
        .with_message("foreign diagnostic parent")
        .with_realm(foreign.realm().clone())
        .with_cause(JsError::from_opaque(cause.into()));
    PanicError::new(PANIC_MESSAGE)
        .with_source(diagnostic)
        .into()
}

fn assert_foreign_parent_was_replaced(error: &JsError) {
    let expected = JsError::from(
        PanicError::new(PANIC_MESSAGE)
            .with_source(JsNativeError::typ().with_message(FOREIGN_REALM_MESSAGE)),
    );
    assert_eq!(error, &expected);
    let Some(EngineError::Panic(panic)) = error.as_engine() else {
        panic!("diagnostic admission must preserve the outer panic")
    };
    assert_eq!(panic.message(), PANIC_MESSAGE);
}

#[test]
fn admission_replaces_a_foreign_diagnostic_parent_and_releases_its_local_cause() {
    let foreign = Context::default();
    let receiving = Context::default();
    let drops = Rc::new(Cell::new(0));
    let source = panic_with_foreign_parent_and_local_cause(&foreign, &receiving, &drops);

    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the unadmitted diagnostic still owns its local opaque cause"
    );

    let admitted = source.admit_runtime(receiving.runtime());
    assert_foreign_parent_was_replaced(&admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "replacing the whole foreign parent releases its hidden local cause"
    );
    // Keep the admitted panic alive across collection and inspect its unchanged
    // semantic value afterward; it must not retain the removed child as storage.
    assert_foreign_parent_was_replaced(&admitted);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

#[test]
fn an_unadmitted_alias_keeps_a_removed_diagnostic_cause_until_the_alias_is_dropped() {
    let foreign = Context::default();
    let receiving = Context::default();
    let drops = Rc::new(Cell::new(0));
    let source = panic_with_foreign_parent_and_local_cause(&foreign, &receiving, &drops);
    let unadmitted_alias = source.clone();
    let admitted = source.admit_runtime(receiving.runtime());

    assert_foreign_parent_was_replaced(&admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        0,
        "the independent unadmitted alias legitimately keeps the child alive"
    );

    drop(unadmitted_alias);
    receiving.runtime().collect().unwrap();
    assert_eq!(
        drops.get(),
        1,
        "the admitted panic must not keep the child after its last real owner is dropped"
    );
    assert_foreign_parent_was_replaced(&admitted);
    drop(admitted);
    receiving.runtime().collect().unwrap();
    assert_eq!(drops.get(), 1);
}

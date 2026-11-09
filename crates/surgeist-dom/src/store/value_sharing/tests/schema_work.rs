//! The single pool-work count the adapter admits, compared with the HTML
//! accounting schema's credits for the same work.

use super::*;
use surgeist_html::{
    accounting::{HtmlLimits, HtmlWork, charging_tariffs, value_pool_work},
    bounded::values::HtmlStringPool,
};
use surgeist_ledger::Operation;

#[test]
fn pool_work_converts_to_the_single_count_the_adapter_admits() {
    for work in [
        HtmlStringPool::root_work(),
        HtmlStringPool::mark_work(),
        HtmlStringPool::seal_work(),
        HtmlStringPool::disposal_work(),
    ] {
        let mut operation = Operation::new("count", HtmlLimits::unlimited(), 2).unwrap();
        let count = operation.root();
        let charging = operation
            .child(count, "pool", HtmlLimits::unlimited(), charging_tariffs())
            .unwrap();
        operation
            .apply_batch(charging, &value_pool_work(work))
            .unwrap();
        let credits = operation.usage(count).unwrap().work(HtmlWork::Credits);
        assert_eq!(Ok(credits), html_work(work).map(|steps| steps as u64));
    }
}

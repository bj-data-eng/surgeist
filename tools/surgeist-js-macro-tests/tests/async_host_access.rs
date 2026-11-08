#![allow(unused_crate_dependencies)]

#[test]
fn async_host_callbacks_cannot_expose_the_owner_cell_or_suspend_a_short_lease() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/async_host_access/owned_tasks.rs");
    cases.compile_fail("tests/async_host_access/native_holder_cell.rs");
    cases.compile_fail("tests/async_host_access/job_lease_future_escape.rs");
}

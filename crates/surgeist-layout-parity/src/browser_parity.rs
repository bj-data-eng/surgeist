//! Browser-parity support: golden XML parsing, the adapter into `surgeist-layout`,
//! exact comparison and the registered qualification editions.

#[cfg(test)]
mod browser_controls_tests;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the library admits the registered editions; the crate's tests use the rest"
    )
)]
pub mod portable_qualification;
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the generator and qualification driver use part of the comparison support; the crate's tests use the rest"
    )
)]
pub(crate) mod support;
#[cfg(test)]
mod tests;

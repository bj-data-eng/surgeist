//! Integration tests grouped by the CSS domain they exercise.

#[macro_use]
mod support;

mod characterization;
mod color;
mod conformance;
mod container;
mod corpora;
mod cssom;
mod media;
mod numeric;
mod properties;
mod rules;
mod selectors;
mod serialization;
mod supports;
mod syntax;
mod values;

mod isolated_stack_lifecycle;

//! The authentic scope frontier remains stronger than either Pretty phase.
use super::*;
#[test]
fn pretty_native_scope_frontier_precedes_its_attractive_tail_f32() {
    avoid_short_last_line::with_style::<f32>(l::TextWrapStyle::Pretty);
}
#[test]
fn pretty_native_scope_frontier_precedes_its_attractive_tail_f64() {
    avoid_short_last_line::with_style::<f64>(l::TextWrapStyle::Pretty);
}

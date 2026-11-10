//! Structural equality of authored values, and the provenance comparison.
//!
//! `==` compares what a value says and ignores where it was read from. `same_provenance`
//! also compares origins, spans, positions and the spelling a value retains.

mod layout_equality;
mod layout_rules_sweep;
mod layout_sweep;
mod retained_equality;
mod spelling_equality;
mod structural_equality;
mod text_equality;

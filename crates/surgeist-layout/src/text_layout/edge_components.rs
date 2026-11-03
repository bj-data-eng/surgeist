//! Private consuming bridge for the two genuine sealed Layout scalar lanes.
use crate::{InlineBoundaryEdgeOf, LayoutScalar};
use surgeist_text::InlineBoxEdgeComponents;

pub(crate) fn from_native<S: LayoutScalar>(
    edge: InlineBoundaryEdgeOf<S>,
) -> surgeist_text::Result<InlineBoxEdgeComponents> {
    use std::any::TypeId;
    if TypeId::of::<S>() == TypeId::of::<f32>() {
        // A genuine f32 widens to f64 and recovers exactly, including subnormals.
        InlineBoxEdgeComponents::from_f32(
            edge.margin().to_f64() as f32,
            edge.border().to_f64() as f32,
            edge.padding().to_f64() as f32,
        )
    } else if TypeId::of::<S>() == TypeId::of::<f64>() {
        InlineBoxEdgeComponents::from_f64(
            edge.margin().to_f64(),
            edge.border().to_f64(),
            edge.padding().to_f64(),
        )
    } else {
        unreachable!("LayoutScalar is sealed to f32 and f64")
    }
}

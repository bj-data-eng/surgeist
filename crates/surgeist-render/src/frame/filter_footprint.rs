//! Bounded native-filter operation membership, independent of source alpha.
use super::GraphLoweringSpatialDescriptor;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FilterOutputFootprint {
    source_spatial: GraphLoweringSpatialDescriptor,
    output_clip: [f64; 4],
}
impl FilterOutputFootprint {
    pub(super) const fn from_qualified_grid(
        source_spatial: GraphLoweringSpatialDescriptor,
        output_clip: [f64; 4],
    ) -> Self {
        Self {
            source_spatial,
            output_clip,
        }
    }
    pub(crate) const fn source_spatial(self) -> GraphLoweringSpatialDescriptor {
        self.source_spatial
    }
    pub(crate) const fn output_clip(self) -> [f64; 4] {
        self.output_clip
    }
}

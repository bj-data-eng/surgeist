//! Runtime serialization of the checked offset texel mapping.
use crate::{
    Result,
    accounting::Account,
    frame::{GraphLoweringOffset, OffsetStage},
    shader::{OffsetParameterBytes, OffsetParameterFacts},
};

pub(super) fn parameters(
    operation: &GraphLoweringOffset,
    account: &Account,
) -> Result<OffsetParameterBytes> {
    let map = operation.mapping();
    OffsetParameterBytes::try_new(
        OffsetParameterFacts {
            translation: map.translation(),
            ratio: map.ratio(),
            source_clip: map.source_clip(),
            result_clip: map.result_clip(),
            source_extent: map.source_extent(),
            result_extent: map.result_extent(),
            sampling: map.sampling().shader_value(),
            rebase: operation.stage() == OffsetStage::RebaseSource,
        },
        account,
    )
}

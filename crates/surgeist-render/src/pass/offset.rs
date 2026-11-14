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
    let facts = OffsetParameterFacts {
        translation: map.translation(),
        ratio: map.ratio(),
        source_clip: map.source_clip(),
        result_clip: map.result_clip(),
        source_extent: map.source_extent(),
        result_extent: map.result_extent(),
        sampling: map.sampling().shader_value(),
        rebase: operation.stage() == OffsetStage::RebaseSource,
    };
    match operation.purpose() {
        crate::frame::OffsetPurpose::Standalone => OffsetParameterBytes::try_new(facts, account),
        crate::frame::OffsetPurpose::CoveredAlphaDeferred => {
            OffsetParameterBytes::try_new_covered_alpha(facts, account)
        }
    }
}

//! Serialization of the checked native morphology route without kernel resources.
use crate::{
    FilterColorSpace, MorphologyOperator, Result,
    accounting::{Account, RenderWork},
    frame::{MorphologyPassFacts, MorphologyStage},
    shader::{
        MorphologyParameterBytes, MorphologyParameterFacts, ShaderMorphologyStageKey,
        ShaderProgramKey,
    },
};

pub(super) const fn stage_key(stage: MorphologyStage) -> ShaderMorphologyStageKey {
    match stage {
        MorphologyStage::RebaseSource => ShaderMorphologyStageKey::RebaseSource,
        MorphologyStage::Copy => ShaderMorphologyStageKey::Copy,
        MorphologyStage::Prepare => ShaderMorphologyStageKey::Prepare,
        MorphologyStage::Table { .. } => ShaderMorphologyStageKey::Table,
        MorphologyStage::FirstQuery { .. } => ShaderMorphologyStageKey::FirstQuery,
        MorphologyStage::RefineQuery { .. } => ShaderMorphologyStageKey::RefineQuery,
        MorphologyStage::Query { .. } => ShaderMorphologyStageKey::Query,
        MorphologyStage::Resolve => ShaderMorphologyStageKey::Resolve,
        MorphologyStage::Constant => ShaderMorphologyStageKey::Constant,
    }
}

pub(super) fn shader_program(work: &MorphologyPassFacts) -> ShaderProgramKey {
    ShaderProgramKey::Morphology {
        stage: stage_key(work.stage()),
    }
}

pub(super) fn parameters(
    work: &MorphologyPassFacts,
    account: &Account,
) -> Result<MorphologyParameterBytes> {
    let map = work.mapping();
    let stage = work.stage();
    let query = map.query();
    let endpoint = |lower: bool| -> Result<(u32, i32)> {
        let Some(query) = query else {
            return Ok((0, 0));
        };
        let endpoint = if lower { query.lower() } else { query.upper() };
        let kind = u32::try_from(endpoint.kind())
            .map_err(|_| super::close::preparation_error("morphology endpoint kind is invalid"))?;
        Ok((kind, endpoint.delta()))
    };
    let (lower_kind, lower_delta) = endpoint(true)?;
    let (upper_kind, upper_delta) = endpoint(false)?;
    let color = if stage == MorphologyStage::Constant {
        account.work(RenderWork::CoordinateVisits, 4)?;
        let color = map.color();
        [
            color.r() * color.a(),
            color.g() * color.a(),
            color.b() * color.a(),
            color.a(),
        ]
    } else {
        [0.0; 4]
    };
    MorphologyParameterBytes::try_new(
        MorphologyParameterFacts {
            stage: stage_key(stage).parameter_code(),
            source_size: map.source_extent(),
            target_size: map.target_extent(),
            source_base: map.source_base(),
            source_clip: map.source_clip(),
            output_clip: map.output_clip(),
            periodic_base: map.periodic_base(),
            periodic_step: map.periodic_step(),
            periodic: map.periodic(),
            converted: map.converted(),
            linear: work.operation().color_space() == FilterColorSpace::LinearSrgb,
            operator: match work.operation().operator() {
                MorphologyOperator::Erode => 0,
                MorphologyOperator::Dilate => 1,
            },
            axis: stage.axis().map_or(0, |axis| axis.shader_value()),
            level: stage.level(),
            span: stage.span(),
            input_length: query.map_or(0, |query| query.input_length()),
            lower_kind,
            lower_delta,
            upper_kind,
            upper_delta,
            valid_output: query.map_or([0; 2], |query| query.valid_output()),
            constant_length: query.and_then(|query| query.constant_length()),
            color,
        },
        account,
    )
}

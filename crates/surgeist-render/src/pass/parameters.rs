use super::PassMap;
use crate::accounting::{Account, RenderWork};

use super::{
    close::preparation_error,
    model::{
        RuntimeBlur, RuntimeComposite, RuntimeCompositeKind, RuntimePass, RuntimePassKind,
        RuntimeReadBinding, RuntimeReadRole, RuntimeResourceId, RuntimeResourceRequest,
        RuntimeResultBinding, RuntimeSamplingEdge, RuntimeSpatialDescriptor,
    },
};
use crate::{
    Result,
    shader::{
        BlurEdgeParameterBytes, ColorFilterOperationBufferLimits, ColorFilterOperationBytes,
        CompositeParameterBytes, DropShadowParameterBytes, PassSpatialUniformBytes,
    },
};

pub(super) fn prepare_blur_edge_parameters(
    pass: &RuntimePass,
    account: &Account,
) -> Result<Option<BlurEdgeParameterBytes>> {
    let blur = match &pass.kind {
        RuntimePassKind::BlurHorizontal(Some(blur)) | RuntimePassKind::BlurVertical(Some(blur)) => {
            blur
        }
        _ => return Ok(None),
    };
    match blur.edge {
        RuntimeSamplingEdge::SemanticBorderMirror(bounds) => {
            BlurEdgeParameterBytes::try_from_semantic_bounds(bounds, Some(account)).map(Some)
        }
        RuntimeSamplingEdge::TransparentBlack => Ok(None),
        RuntimeSamplingEdge::ClampToExtent => Err(preparation_error(
            "a Gaussian blur cannot use clamp-to-extent edge semantics",
        )),
    }
}

pub(super) fn prepare_drop_shadow_parameters(
    pass: &RuntimePass,
    account: &Account,
) -> Result<Option<DropShadowParameterBytes>> {
    let RuntimePassKind::DropShadowColorize(Some(shadow)) = &pass.kind else {
        return Ok(None);
    };
    let bytes = DropShadowParameterBytes::try_new(shadow.offset, shadow.color, Some(account))?;
    if bytes.as_bytes().len() != 32 {
        return Err(preparation_error(
            "drop-shadow parameter serialization changed its exact WGSL byte length",
        ));
    }
    Ok(Some(bytes))
}

pub(super) fn prepare_color_filter_operations(
    pass: &RuntimePass,
    limits: ColorFilterOperationBufferLimits,
    account: &Account,
) -> Result<Option<ColorFilterOperationBytes>> {
    let RuntimePassKind::ColorFilter(Some(filter)) = &pass.kind else {
        return Ok(None);
    };
    let bytes = ColorFilterOperationBytes::try_from_runtime_operations_with_limits(
        account,
        filter.operations(),
        limits,
    )?;
    if bytes.as_bytes().is_empty() {
        return Err(preparation_error(
            "prepared color-filter operation bytes are empty",
        ));
    }
    Ok(Some(bytes))
}

pub(super) fn prepared_pass_spatial_uniform(
    pass: &RuntimePass,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    root_working_image: RuntimeResourceId,
    account: &Account,
) -> Result<Option<PassSpatialUniformBytes>> {
    if pass.cache_keys.is_none() {
        return Ok(None);
    }
    let result_spatial = || -> Result<RuntimeSpatialDescriptor> {
        let RuntimeResultBinding::Resource(resource) = pass.result else {
            return Err(preparation_error(
                "custom pass has no concrete runtime result resource",
            ));
        };
        resources
            .get(&resource, account)?
            .map(|resource| resource.spatial)
            .ok_or_else(|| preparation_error("custom pass result spatial binding is missing"))
    };
    let read_spatial = |role| -> Result<RuntimeSpatialDescriptor> {
        let mut resource = None;
        for read in &pass.reads {
            account.work(RenderWork::ResourceProbes, 1)?;
            if read.role == role {
                resource = Some(read.resource);
                break;
            }
        }
        let resource = resource
            .ok_or_else(|| preparation_error("custom pass source spatial binding is missing"))?;
        resources
            .get(&resource, account)?
            .map(|resource| resource.spatial)
            .ok_or_else(|| preparation_error("custom pass source resource is missing"))
    };

    let (source, destination) = match &pass.kind {
        RuntimePassKind::CanonicalizeCapture => (
            read_spatial(RuntimeReadRole::CaptureSource)?,
            result_spatial()?,
        ),
        RuntimePassKind::CopyBackdrop => (
            read_spatial(RuntimeReadRole::CompletedParent)?,
            result_spatial()?,
        ),
        RuntimePassKind::ColorFilter(Some(filter)) => {
            (filter.spatial.source, filter.spatial.result)
        }
        RuntimePassKind::BlurHorizontal(Some(blur)) | RuntimePassKind::BlurVertical(Some(blur)) => {
            (blur.spatial.source, blur.spatial.result)
        }
        RuntimePassKind::DropShadowColorize(Some(shadow)) => {
            (shadow.spatial.source, shadow.spatial.result)
        }
        RuntimePassKind::Composite(Some(_)) => (
            read_spatial(RuntimeReadRole::CompositeSource)?,
            result_spatial()?,
        ),
        RuntimePassKind::Present => {
            let source = read_spatial(RuntimeReadRole::FinalWorkingImage)?;
            let destination = resources
                .get(&root_working_image, account)?
                .map(|resource| resource.spatial)
                .ok_or_else(|| preparation_error("present destination spatial is missing"))?;
            (source, destination)
        }
        RuntimePassKind::ClearRoot { .. }
        | RuntimePassKind::VelloCapture(_)
        | RuntimePassKind::ColorFilter(None)
        | RuntimePassKind::BlurHorizontal(None)
        | RuntimePassKind::BlurVertical(None)
        | RuntimePassKind::DropShadowColorize(None)
        | RuntimePassKind::Composite(None) => {
            return Err(preparation_error(
                "non-executable pass unexpectedly requested spatial serialization",
            ));
        }
    };
    PassSpatialUniformBytes::try_from_runtime_spatial_descriptors(
        source,
        destination,
        Some(account),
    )
    .map(Some)
}

pub(super) fn prepared_pass_composite_parameters(
    pass: &RuntimePass,
    resources: &PassMap<RuntimeResourceId, &RuntimeResourceRequest>,
    account: &Account,
) -> Result<Option<CompositeParameterBytes>> {
    match &pass.kind {
        RuntimePassKind::Composite(Some(RuntimeComposite {
            kind: RuntimeCompositeKind::Layer { parameters, .. },
            ..
        })) => {
            let RuntimeResultBinding::Resource(result) = pass.result else {
                return Err(preparation_error(
                    "composite pass has no result spatial binding",
                ));
            };
            let destination = resources
                .get(&result, account)?
                .ok_or_else(|| preparation_error("composite result spatial binding is missing"))?
                .spatial;
            let bytes = CompositeParameterBytes::try_from_runtime_layer(
                parameters,
                destination,
                Some(account),
            )?;
            if bytes.as_bytes().len() != 112 {
                return Err(preparation_error(
                    "composite parameter serialization changed its exact WGSL byte length",
                ));
            }
            Ok(Some(bytes))
        }
        _ => Ok(None),
    }
}

pub(super) fn backdrop_blur_edge_uniform_bytes(
    blur: &RuntimeBlur,
    source: &RuntimeReadBinding,
    prepared_parameters: Option<&BlurEdgeParameterBytes>,
    account: &Account,
) -> Result<Option<[u8; 16]>> {
    let RuntimeSamplingEdge::SemanticBorderMirror(bounds) = blur.edge else {
        if blur.edge != RuntimeSamplingEdge::TransparentBlack
            || source.sampling_edge() != RuntimeSamplingEdge::TransparentBlack
            || prepared_parameters.is_some()
        {
            return Err(preparation_error(
                "the spatial-filter transparent blur changed its checked edge contract",
            ));
        }
        return Ok(None);
    };
    let expected = BlurEdgeParameterBytes::try_from_semantic_bounds(bounds, Some(account))?;
    if source.sampling_edge() != blur.edge || prepared_parameters != Some(&expected) {
        return Err(preparation_error(
            "the backdrop mirrored blur changed its checked semantic edge",
        ));
    }
    let values = [
        bounds.x() as f32,
        bounds.y() as f32,
        (bounds.x() + bounds.width()) as f32,
        (bounds.y() + bounds.height()) as f32,
    ];
    account.work(RenderWork::SerializedBytes, 16)?;
    let mut bytes = [0_u8; 16];
    for (index, value) in values.into_iter().enumerate() {
        let offset = index * 4;
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(Some(bytes))
}

#[cfg(test)]
mod role_lookup_accounting_tests {
    use super::*;
    use crate::accounting::SharedValue;
    use crate::pass::test_support::{lower_capture_proof_fixture, measure_proof_work};

    #[test]
    fn spatial_role_lookup_counts_the_candidate_before_resource_probes() {
        let capabilities = crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096);
        let lowered = lower_capture_proof_fixture(1, false, &capabilities);
        let pass = lowered
            .passes
            .iter()
            .find(|pass| matches!(pass.kind, RuntimePassKind::CanonicalizeCapture))
            .unwrap();
        let source = pass.reads[0].resource;
        let RuntimeResultBinding::Resource(target) = pass.result else {
            unreachable!()
        };
        assert_eq!(pass.reads.len(), 1);
        assert!(source < target);
        let setup = Account::for_test();
        let mut resources = PassMap::temporary(&setup, 2).unwrap();
        for id in [source, target] {
            resources
                .insert_with(
                    id,
                    lowered
                        .resources
                        .iter()
                        .find(|resource| resource.id == id)
                        .unwrap(),
                    &setup,
                )
                .unwrap();
        }
        for limit in [4, 3] {
            let (result, report) =
                measure_proof_work(RenderWork::ResourceProbes, limit, |account| {
                    prepared_pass_spatial_uniform(
                        pass,
                        &resources,
                        lowered.root_working_image,
                        account,
                    )
                });
            let attempt = report.attempt().unwrap();
            if limit == 4 {
                assert!(result.unwrap().is_some());
                // One role candidate, two binary probes for the first of two
                // sorted keys, then one probe for the second key.
                assert_eq!(attempt.usage().work(RenderWork::ResourceProbes), 4);
            } else {
                let error = result.unwrap_err();
                assert!(matches!(
                    error.accounting_diagnostic(),
                    Some(crate::RenderAccountingError::Refused(_))
                ));
                let refusal = attempt.refusal().unwrap();
                assert_eq!(
                    refusal.quantity,
                    crate::RenderQuantity::Work(RenderWork::ResourceProbes)
                );
                assert_eq!((refusal.current, refusal.amount), (3, 1));
                assert_eq!(attempt.usage().work(RenderWork::SerializedBytes), 0);
            }
        }
    }

    #[test]
    fn spatial_missing_role_refuses_before_the_semantic_missing_binding_error() {
        let capabilities = crate::backend::DeviceCapabilities::from_test_facts(true, true, 4096);
        let lowered = lower_capture_proof_fixture(1, false, &capabilities);
        let mut pass = lowered
            .passes
            .iter()
            .find(|pass| matches!(pass.kind, RuntimePassKind::CanonicalizeCapture))
            .unwrap()
            .clone();
        let setup = Account::for_test();
        let mut read = pass.reads[0];
        read.role = RuntimeReadRole::CompletedParent;
        let mut reads = setup.retained(1).unwrap();
        reads.append_slot().unwrap().write(read);
        pass.reads = SharedValue::from_accounted(reads, &setup.storage()).unwrap();
        let resources = PassMap::temporary(&setup, 0).unwrap();
        let control =
            prepared_pass_spatial_uniform(&pass, &resources, lowered.root_working_image, &setup)
                .unwrap_err();
        assert_eq!(control.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(control.accounting_diagnostic(), None);
        let (result, report) = measure_proof_work(RenderWork::ResourceProbes, 0, |account| {
            prepared_pass_spatial_uniform(&pass, &resources, lowered.root_working_image, account)
        });
        let error = result.unwrap_err();
        let Some(crate::RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
        else {
            panic!("role inspection must refuse before the missing-binding diagnostic");
        };
        assert_eq!(
            refusal.quantity(),
            crate::RenderQuantity::Work(RenderWork::ResourceProbes)
        );
        assert_eq!((refusal.current(), refusal.amount()), (0, 1));
        assert_eq!(
            report
                .attempt()
                .unwrap()
                .usage()
                .work(RenderWork::SerializedBytes),
            0
        );
    }
}

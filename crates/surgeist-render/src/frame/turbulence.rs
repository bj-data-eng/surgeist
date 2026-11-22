//! Closed parameters and exact result footprint for the source-free filter.

use super::{
    FilterOutputFootprint, FrameContext, GraphLoweringSpatialDescriptor,
    bounds::NonEmptyFrameSpatialPlan,
    convolution::intersection,
    filter_grid::{self, FilterGridMapping, FilterGridOwner},
};
use crate::{
    Result,
    accounting::{Account, RenderWork},
    geometry::{Rect, Transform},
    turbulence::{Turbulence, TurbulenceParameters, TurbulenceRegion},
};

pub(super) fn output_domain(region: TurbulenceRegion) -> Rect {
    intersection(region.primitive(), region.filter())
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TurbulencePassFacts {
    mapping: FilterGridMapping,
    parameters: TurbulenceParameters,
}

impl TurbulencePassFacts {
    pub(super) fn try_new(
        context: FrameContext,
        operation: Turbulence,
        region: TurbulenceRegion,
        transform: Transform,
        account: &Account,
    ) -> Result<Self> {
        account.work(RenderWork::FilterOperationVisits, 1)?;
        let domain = output_domain(region);
        let owner = FilterGridOwner::Turbulence;
        let spatial = filter_grid::require_spatial(context, domain, transform, owner)?;
        let mapping = FilterGridMapping::new(None, spatial, owner)
            .with_output_clip(domain, spatial, account)?;
        let origin = spatial.texel_center_mapping.origin;
        let parameters = operation.parameters(
            region,
            mapping.output_clip(),
            [origin.x(), origin.y()],
            1.0 / spatial.raster_scale.get(),
            filter_grid::extent(spatial),
            account,
        )?;
        Ok(Self {
            mapping,
            parameters,
        })
    }

    pub(super) fn spatial(&self) -> NonEmptyFrameSpatialPlan {
        self.mapping.target_plan()
    }

    pub(crate) fn target_plan(&self) -> GraphLoweringSpatialDescriptor {
        self.mapping.target_spatial()
    }

    pub(crate) fn output_footprint(&self) -> FilterOutputFootprint {
        self.mapping.output_footprint()
    }

    pub(crate) fn output_clip(&self) -> [f64; 4] {
        self.mapping.output_clip()
    }

    pub(crate) fn uniform_bytes(&self) -> &[u8; TurbulenceParameters::SIZE] {
        self.parameters.as_bytes()
    }

    pub(crate) fn octaves(&self) -> u32 {
        self.parameters.octaves()
    }
}

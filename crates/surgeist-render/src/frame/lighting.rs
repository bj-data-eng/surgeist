//! Qualified per-stage data for finite native lighting.

use super::{
    FilterOutputFootprint, GraphLoweringSpatialDescriptor,
    bounds::NonEmptyFrameSpatialPlan,
    filter_grid::{extent, spatial_descriptor},
    lighting_grid::LightingGrid,
};
use crate::{
    Result,
    accounting::{Account, RenderWork, SharedValue},
    filter_model::FilterColorSpace,
    lighting::{LightingNumericPhase as Phase, PreparedLight, PreparedLighting, numeric_error},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LightingStage {
    ResampleBump,
    Shade,
    Resolve,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LightingPassFacts(SharedValue<LightingPassData>);

#[derive(Debug, PartialEq)]
pub(super) struct LightingOperationPlan {
    pub(super) grid: LightingGrid,
    pub(super) operation: PreparedLighting,
}

#[derive(Clone, Debug, PartialEq)]
struct LightingPassData {
    plan: SharedValue<LightingOperationPlan>,
    stage: LightingStage,
    source: Option<GraphLoweringSpatialDescriptor>,
    target: GraphLoweringSpatialDescriptor,
    scissor: [u32; 4],
    uniform: [u8; 256],
}

impl LightingPassFacts {
    pub(super) fn try_new(
        plan: SharedValue<LightingOperationPlan>,
        stage: LightingStage,
        source: Option<NonEmptyFrameSpatialPlan>,
        account: &Account,
    ) -> Result<Self> {
        use LightingStage::{ResampleBump, Resolve, Shade};
        let grid = &plan.grid;
        let operation = plan.operation;
        let target = match stage {
            ResampleBump => grid.bump.spatial,
            Shade => grid.shade.spatial,
            Resolve => grid.output,
        };
        let expected_source = match stage {
            ResampleBump => grid.capture.map(|capture| capture.spatial),
            Shade => source.map(|_| grid.bump.spatial),
            Resolve => Some(grid.shade.spatial),
        };
        if source != expected_source || (stage == ResampleBump && source.is_none()) {
            return Err(contradictory_lighting_source());
        }
        account.work(RenderWork::CoordinateVisits, 1)?;
        let [width, height] = extent(target);
        let [source_width, source_height] = source.map(extent).unwrap_or([0, 0]);
        let mut uniform = LightingUniform {
            sizes: [source_width, source_height, width, height],
            ..LightingUniform::default()
        };
        match stage {
            ResampleBump => {
                let capture = grid.capture.ok_or_else(contradictory_lighting_source)?;
                uniform.sample_map = capture.sample_map;
                uniform.clip = capture.clip;
            }
            Shade => {
                if source.is_some() {
                    uniform.edge_flags = grid.bump.edge_flags;
                    uniform.source_offset = grid.shade_source_offset;
                }
                uniform.qualify_shade(grid, operation, source.is_some(), account)?;
            }
            Resolve => {
                uniform.edge_flags = grid.shade.edge_flags;
                uniform.linear = operation.color_space() == FilterColorSpace::LinearSrgb;
                uniform.sample_map = grid.resolve_map;
                uniform.clip = grid.output_clip;
            }
        }
        account.work(RenderWork::SerializedBytes, 256)?;
        Ok(Self(SharedValue::new(
            LightingPassData {
                plan,
                stage,
                source: source.map(spatial_descriptor),
                target: spatial_descriptor(target),
                scissor: [0, 0, width, height],
                uniform: uniform.bytes(),
            },
            &account.storage(),
        )?))
    }

    pub(crate) fn stage(&self) -> LightingStage {
        self.0.stage
    }

    pub(crate) fn source_plan(&self) -> Option<GraphLoweringSpatialDescriptor> {
        self.0.source
    }

    pub(crate) fn reads_source(&self) -> bool {
        self.source_plan().is_some()
    }

    pub(crate) fn target_plan(&self) -> GraphLoweringSpatialDescriptor {
        self.0.target
    }

    pub(crate) fn scissor(&self) -> [u32; 4] {
        self.0.scissor
    }

    pub(crate) fn uniform_bytes(&self) -> &[u8; 256] {
        &self.0.uniform
    }

    pub(crate) fn output_clip(&self) -> [f64; 4] {
        self.0.plan.grid.output_clip.map(f64::from)
    }

    pub(super) fn output_footprint(&self) -> FilterOutputFootprint {
        FilterOutputFootprint::from_qualified_grid(self.target_plan(), self.output_clip())
    }

    pub(crate) fn same_operation(&self, other: &Self) -> bool {
        self.0.plan.same_allocation(&other.0.plan)
    }

    pub(crate) fn primitive_operation(&self) -> crate::diagnostic::PrimitiveOperation {
        self.0.plan.operation.primitive_operation()
    }
}

fn contradictory_lighting_source() -> crate::Error {
    crate::Error::new(
        crate::error::BackendErrorCode::RenderFailed,
        "lighting source does not match its qualified stage",
    )
}

#[derive(Default)]
struct LightingUniform {
    sizes: [u32; 4],
    edge_flags: u32,
    linear: bool,
    light_kind: u32,
    cone_kind: u32,
    sample_map: [f32; 4],
    clip: [f32; 4],
    node_base_step: [f32; 4],
    surface_material: [f32; 4],
    incident: [f32; 3],
    spot_axis: [f32; 4],
    spot_cone: [f32; 2],
    light_rgb: [f32; 3],
    source_offset: [i32; 2],
    light_anchor: [i32; 2],
    height_anchor: f32,
    material_exponent: f32,
}

impl LightingUniform {
    fn qualify_shade(
        &mut self,
        grid: &LightingGrid,
        operation: PreparedLighting,
        textured: bool,
        account: &Account,
    ) -> Result<()> {
        let color = operation.lighting_color();
        self.light_rgb = [color.r(), color.g(), color.b()];
        self.linear = operation.color_space() == FilterColorSpace::LinearSrgb;
        let scale = operation.surface_scale();
        if textured {
            for (axis, ratio) in self.surface_material.iter_mut().take(2).enumerate() {
                if grid.counts[axis] > 1 {
                    account.work(RenderWork::CoordinateVisits, 1)?;
                    let value = scale / grid.spacing[axis];
                    *ratio = uploaded(value, Phase::HeightGradient)?;
                    finite(*ratio * 2.0, Phase::HeightGradient)?;
                }
            }
        }
        self.surface_material[3] = uploaded(operation.constant(), Phase::ColorProduct)?;
        if let Some(exponent) = operation.material_exponent() {
            self.material_exponent = uploaded(exponent, Phase::ColorProduct)?;
        }

        account.work(RenderWork::CoordinateVisits, 3)?;
        match operation.light() {
            PreparedLight::Distant { direction } => {
                let [x, y, z] = direction;
                self.incident = [
                    uploaded(x, Phase::LightVector)?,
                    uploaded(y, Phase::LightVector)?,
                    uploaded(z, Phase::LightVector)?,
                ];
            }
            PreparedLight::Point { position } => {
                self.light_kind = 1;
                self.qualify_position(grid, position, scale, textured, account)?;
            }
            PreparedLight::Spot {
                position,
                direction: Some([x, y, z]),
                exponent,
                cone_angle,
            } => {
                self.light_kind = 2;
                self.qualify_position(grid, position, scale, textured, account)?;
                self.spot_axis = [
                    uploaded(x, Phase::LightVector)?,
                    uploaded(y, Phase::LightVector)?,
                    uploaded(z, Phase::LightVector)?,
                    uploaded(exponent, Phase::LightVector)?,
                ];
                match cone_angle {
                    None => {}
                    Some(0.0) => self.cone_kind = 1,
                    Some(angle) => {
                        self.cone_kind = 2;
                        let cut = angle.to_radians().cos();
                        let full = (cut + 0.016).min(1.0);
                        let cut = uploaded(cut, Phase::LightVector)?;
                        let full = uploaded(full, Phase::LightVector)?;
                        let reciprocal = finite((full - cut).recip(), Phase::LightVector)?;
                        finite((1.0 - cut) * reciprocal, Phase::LightVector)?;
                        self.spot_cone = [cut, reciprocal];
                    }
                }
            }
            PreparedLight::Spot {
                direction: None, ..
            } => self.light_kind = 3,
        }
        Ok(())
    }

    fn qualify_position(
        &mut self,
        grid: &LightingGrid,
        position: [f64; 3],
        scale: f64,
        textured: bool,
        account: &Account,
    ) -> Result<()> {
        let windows = extent(grid.shade.spatial);
        for axis in 0..2 {
            account.work(RenderWork::CoordinateVisits, 1)?;
            let spacing = grid.spacing[axis];
            let origin = grid.shade.index_origin[axis];
            let first = grid.relative_node(axis, origin, position[axis])?;
            // Pack the residual at the node nearest the light, instead of
            // cancelling two large f32 values near the light's projection.
            let last = windows[axis] - 1;
            let anchor = (-first / spacing).round().clamp(0.0, f64::from(last)) as u32;
            let anchor_index = origin
                .checked_add(i64::from(anchor))
                .ok_or_else(|| numeric_error(Phase::GridIndex, origin as f64))?;
            let base = uploaded(
                grid.relative_node(axis, anchor_index, position[axis])?,
                Phase::LightVector,
            )?;
            let step = if last == 0 {
                0.0
            } else {
                uploaded(spacing, Phase::LightVector)?
            };
            for delta in [-(anchor as f32), (last - anchor) as f32] {
                qualify_sum_of_product(delta, step, base)?;
            }
            self.light_anchor[axis] = i32::try_from(anchor)
                .map_err(|_| numeric_error(Phase::GridIndex, f64::from(anchor)))?;
            self.node_base_step[axis] = base;
            self.node_base_step[axis + 2] = step;
        }
        let z = position[2];
        if textured && scale != 0.0 {
            self.surface_material[2] = uploaded(scale, Phase::HeightGradient)?;
            // Choose the reached height fraction nearest the light. The
            // uploaded residual preserves near-coincidence at that fraction.
            self.height_anchor = (z / scale).clamp(0.0, 1.0) as f32;
            self.incident[2] = uploaded(
                (-scale).mul_add(f64::from(self.height_anchor), z),
                Phase::LightVector,
            )?;
            for alpha in [0.0, 1.0] {
                qualify_sum_of_product(
                    self.height_anchor - alpha,
                    self.surface_material[2],
                    self.incident[2],
                )?;
            }
        } else {
            self.incident[2] = uploaded(z, Phase::LightVector)?;
        }
        Ok(())
    }

    fn bytes(self) -> [u8; 256] {
        // One fixed 256-byte WGSL uniform. The unused lanes and padding are
        // zero, including every stage-specific field absent from this pass.
        let [x, y, z] = self.incident;
        let [red, green, blue] = self.light_rgb;
        let [cut, reciprocal] = self.spot_cone;
        let [offset_x, offset_y] = self.source_offset;
        let [anchor_x, anchor_y] = self.light_anchor;
        let lanes = [
            self.sizes,
            [
                self.edge_flags,
                u32::from(self.linear),
                self.light_kind,
                self.cone_kind,
            ],
            self.sample_map.map(f32::to_bits),
            self.clip.map(f32::to_bits),
            self.node_base_step.map(f32::to_bits),
            self.surface_material.map(f32::to_bits),
            [x, y, z, 0.0].map(f32::to_bits),
            self.spot_axis.map(f32::to_bits),
            [cut, reciprocal, 0.0, 0.0].map(f32::to_bits),
            [red, green, blue, 0.0].map(f32::to_bits),
            [offset_x as u32, offset_y as u32, 0, 0],
            [
                anchor_x as u32,
                anchor_y as u32,
                self.height_anchor.to_bits(),
                self.material_exponent.to_bits(),
            ],
        ];
        let mut bytes = [0; 256];
        for (dst, word) in bytes
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(lanes.into_iter().flatten())
        {
            *dst = word.to_le_bytes();
        }
        bytes
    }
}

fn uploaded(value: f64, phase: Phase) -> Result<f32> {
    let upload = value as f32;
    if upload.is_finite() {
        Ok(if upload.is_subnormal() { 0.0 } else { upload })
    } else {
        Err(numeric_error(phase, value))
    }
}

fn finite(value: f32, phase: Phase) -> Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(numeric_error(phase, f64::from(value)))
    }
}

fn qualify_sum_of_product(delta: f32, step: f32, base: f32) -> Result<()> {
    let phase = Phase::LightVector;
    let product = finite(delta * step, phase)?;
    finite(base + product, phase)?;
    finite(delta.mul_add(step, base), phase)?;
    Ok(())
}

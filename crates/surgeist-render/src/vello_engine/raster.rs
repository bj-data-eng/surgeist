// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

use std::mem::size_of;

use peniko::Color;
use vello_encoding::{Encoding, RenderConfig, Resolver, make_mask_lut, make_mask_lut_16};

use super::bounded;
use crate::accounting::{
    Account, AccountedVec, LocalQuota, RenderPhase, RenderResource, RenderWork,
};
use crate::{Antialiasing, Error, PhysicalSize, Result};

#[cfg(test)]
use super::PreparedVelloPassObservation;
use super::recording::{
    BinningBindings, BufferHandle, BufferRole, ClipLeafBindings, CoarseDispatch,
    CoarseRasterBindings, DrawLeafBindings, FineDispatch, FineDispatchBindings, FineRasterVariant,
    ImageHandle, ImageRole, RasterImageFormat, Recording, RecordingBuilder, ResourceIntent,
};

const VELLO_TILE_EXTENT: u32 = 16;

/// Validated algorithm inputs for one private Vello recording pass.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RasterParameters {
    target_extent: PhysicalSize,
    base_color: Color,
    antialiasing: Antialiasing,
}

impl RasterParameters {
    pub(crate) fn try_new(
        target_extent: PhysicalSize,
        base_color: Color,
        antialiasing: Antialiasing,
    ) -> Result<Self> {
        validate_target_dimension("raster target width", target_extent.width())?;
        validate_target_dimension("raster target height", target_extent.height())?;
        Ok(Self {
            target_extent,
            base_color,
            antialiasing,
        })
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Antialiasing selection remains symbolic until production scene lowering reaches the engine."
        )
    )]
    pub(crate) const fn with_antialiasing(mut self, antialiasing: Antialiasing) -> Self {
        self.antialiasing = antialiasing;
        self
    }
}

/// The required external output contract for one prepared Vello pass.
struct RasterTargetIntent {
    extent: PhysicalSize,
    format: RasterImageFormat,
    access: RasterTargetAccess,
    base_color: Color,
    antialiasing: Antialiasing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RasterTargetAccess {
    StorageWrite,
}

/// An opaque prepared pass that has no runtime resource or submission authority.
pub(crate) struct PreparedVelloPass {
    recording: Recording,
    target_intent: RasterTargetIntent,
    resource_intents: AccountedVec<ResourceIntent>,
}

/// Unforgeable proof that one prepared pass carries one logical direct-Vello pass.
#[must_use]
pub(crate) struct DirectVelloLogicalPass {
    _prepared_pass: (),
}

impl DirectVelloLogicalPass {
    #[cfg(test)]
    pub(crate) const fn cardinality_for_test(&self) -> usize {
        1
    }
}

#[must_use]
pub(crate) struct EncodedVelloPass {
    resources: super::recording::VelloResourceLease,
    logical_pass: DirectVelloLogicalPass,
}

/// Proof produced only after one prepared capture has encoded successfully.
#[must_use]
pub(crate) struct EncodedVelloCaptureProof {
    target_extent: PhysicalSize,
    target_format: wgpu::TextureFormat,
    target_usage: wgpu::TextureUsages,
    antialiasing: Antialiasing,
    transparent_base: bool,
    #[cfg(test)]
    target_view_identity: usize,
}

#[must_use]
pub(crate) struct EncodedVelloCapture {
    resources: super::recording::VelloResourceLease,
    proof: EncodedVelloCaptureProof,
}

impl EncodedVelloPass {
    pub(crate) fn into_resources_and_logical_pass(
        self,
    ) -> (super::recording::VelloResourceLease, DirectVelloLogicalPass) {
        (self.resources, self.logical_pass)
    }
}

impl EncodedVelloCapture {
    pub(crate) fn into_resources_and_proof(
        self,
    ) -> (
        super::recording::VelloResourceLease,
        EncodedVelloCaptureProof,
    ) {
        (self.resources, self.proof)
    }
}

impl EncodedVelloCaptureProof {
    pub(crate) fn proves_capture_contract(
        &self,
        target_extent: PhysicalSize,
        target_format: wgpu::TextureFormat,
        target_usage: wgpu::TextureUsages,
        antialiasing: Antialiasing,
    ) -> bool {
        self.target_extent == target_extent
            && self.target_format == target_format
            && self.target_usage == target_usage
            && self.antialiasing == antialiasing
            && self.transparent_base
    }

    #[cfg(test)]
    pub(crate) const fn transparent_base_for_test(&self) -> bool {
        self.transparent_base
    }

    #[cfg(test)]
    pub(crate) const fn antialiasing_for_test(&self) -> Antialiasing {
        self.antialiasing
    }

    #[cfg(test)]
    pub(crate) const fn target_extent_for_test(&self) -> PhysicalSize {
        self.target_extent
    }

    #[cfg(test)]
    pub(crate) const fn target_format_for_test(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    #[cfg(test)]
    pub(crate) const fn target_usage_for_test(&self) -> wgpu::TextureUsages {
        self.target_usage
    }

    #[cfg(test)]
    pub(crate) const fn target_view_identity_for_test(&self) -> usize {
        self.target_view_identity
    }
}

/// Admits the ordinary Resolver's selected image inputs without changing its
/// packing or residency policy. The locked Resolver queues these Image patches
/// and keys its fresh image cache by Blob identity alone, not content/dimensions.
fn admit_ordinary_atlas_inputs(encoding: &Encoding, account: &Account) -> Result<LocalQuota> {
    let account = account.at(RenderPhase::Atlas);
    account.phase(RenderPhase::Atlas, || {
        let max_texels = super::image_atlas::MAX_TEXELS;
        let quota = account.local_quota(RenderPhase::Atlas, [max_texels * 4, max_texels])?;
        let mut identities = bounded::RasterRegistry::temporary();
        for patch in &encoding.resources.patches {
            account.work(RenderWork::ResourceProbes, 1)?;
            let vello_encoding::Patch::Image { image, .. } = patch else {
                continue;
            };
            if image.width == 0 || image.height == 0 {
                continue;
            }
            let identity = image.data.id();
            if identities.get(&identity, &account)?.is_some() {
                continue;
            }
            let bytes = u64::try_from(image.data.data().len()).map_err(|_| {
                account
                    .storage()
                    .capacity_error(RenderResource::RetainedCpuBytes)
            })?;
            // Both factors originate as u32, so their product fits u64.
            let texels = u64::from(image.width) * u64::from(image.height);
            quota.acquire(RenderResource::LocalAtlasSourceByteQuota, bytes)?;
            quota.acquire(RenderResource::LocalAtlasTexelQuota, texels)?;
            account.work_u64(RenderWork::ImageSourceBytes, bytes)?;
            account.work_u64(RenderWork::AtlasTexels, texels)?;
            identities.insert(identity, (), &account)?;
        }
        Ok(quota)
    })
}

pub(super) fn prepare(
    encoding: &Encoding,
    expansions: &super::image_atlas::ImageExpansions,
    glyph_sizes: &vello_encoding::StreamOffsets,
    endpoints: &[super::gradient_endpoints::CapturedEndpoint],
    parameters: RasterParameters,
    account: &Account,
) -> Result<PreparedVelloPass> {
    // Only pixelated draws need a prepared integer-expanded image grid.
    // Other passes retain the ordinary Resolver image-atlas policy.
    let atlas = if expansions.is_empty() {
        None
    } else {
        Some(super::image_atlas::ImageAtlasPlan::prepare(
            encoding, expansions, account,
        )?)
    };
    let resolved_encoding = atlas.as_ref().map_or(encoding, |atlas| &atlas.encoding);
    // Keep the existing per-build quota live through ordinary resolution. The
    // custom expansion plan has already admitted its own selected inputs.
    let _ordinary_atlas_quota = if atlas.is_none() {
        Some(admit_ordinary_atlas_inputs(resolved_encoding, account)?)
    } else {
        None
    };
    let mut resolver = Resolver::default();
    let producer = bounded::PackedProducer::new(resolved_encoding, glyph_sizes, account)?;
    // Ordinary image packing is inside this indivisible dependency call. Its
    // Atlas observation includes glyph/ramp resolution and serialization.
    let resolution = if atlas.is_none() {
        account.phase(RenderPhase::Atlas, || producer.resolve(&mut resolver))?
    } else {
        producer.resolve(&mut resolver)?
    };
    let (mut packed, layout, ramps, images) = resolution.into_parts();
    let endpoint_texture = super::gradient_endpoints::prepare(
        &layout,
        &mut packed,
        ramps,
        glyph_sizes,
        endpoints,
        account,
    )?;
    let config = RenderConfig::new(
        &layout,
        parameters.target_extent.width(),
        parameters.target_extent.height(),
        &parameters.base_color,
    );
    let mut recording = RecordingBuilder::new(account)?;

    let gradient_image = if let Some(texture) = endpoint_texture {
        recording.upload_gradient_ramps(
            PhysicalSize::new(super::gradient_endpoints::RAMP_WIDTH as u32, texture.height),
            texture.texels,
        )?
    } else if ramps.height == 0 {
        recording.new_transient_image(ImageRole::GradientRamp, PhysicalSize::new(1, 1))?
    } else {
        let mut data = account.retained(ramps.data.len())?;
        for value in ramps.data {
            account.work(RenderWork::SerializedBytes, size_of::<u32>())?;
            data.push(*value)?;
        }
        recording.upload_gradient_ramps(PhysicalSize::new(ramps.width, ramps.height), data)?
    };
    let image_atlas = account.phase(RenderPhase::Atlas, || {
        if let Some(atlas) = atlas {
            let image_atlas = recording.request_image_atlas(atlas.extent)?;
            for entry in atlas.entries {
                if entry.factors() == [1, 1] {
                    recording.write_image(image_atlas, entry.origin, entry.image)?;
                } else {
                    recording.expand_image(image_atlas, entry)?;
                }
            }
            Ok(image_atlas)
        } else {
            let image_atlas = recording.request_image_atlas(PhysicalSize::new(
                images.width.max(1),
                images.height.max(1),
            ))?;
            for (image, x, y) in images.images {
                recording.write_image(image_atlas, [*x, *y], image.clone())?;
            }
            Ok(image_atlas)
        }
    })?;

    if packed.is_empty() {
        account.work(RenderWork::SerializedBytes, size_of::<u32>())?;
        packed.resize(size_of::<u32>(), u8::MAX)?;
    }
    let scene = recording.upload_scene(packed)?;
    let config_buffer = recording.upload_config(config.gpu)?;
    let fine = record_coarse(
        &mut recording,
        CoarseInputs {
            workgroups: config.workgroup_counts,
            sizes: config.buffer_sizes,
            scene,
            config: config_buffer,
            gradient_image,
            image_atlas,
            antialiasing: parameters.antialiasing,
        },
    )?;
    record_fine(&mut recording, fine)?;

    let (recording, resource_intents) = recording.finish();
    Ok(PreparedVelloPass {
        recording,
        target_intent: RasterTargetIntent {
            extent: parameters.target_extent,
            format: RasterImageFormat::Rgba8Unorm,
            access: RasterTargetAccess::StorageWrite,
            base_color: parameters.base_color,
            antialiasing: parameters.antialiasing,
        },
        resource_intents,
    })
}

struct FineSchedule {
    variant: FineRasterVariant,
    workgroups: vello_encoding::WorkgroupSize,
    config: BufferHandle,
    tile: BufferHandle,
    segments: BufferHandle,
    per_tile_command_list: BufferHandle,
    gradient_image: ImageHandle,
    info_bin_data: BufferHandle,
    image_atlas: ImageHandle,
    blend_spill: BufferHandle,
}

struct CoarseInputs {
    workgroups: vello_encoding::WorkgroupCounts,
    sizes: vello_encoding::BufferSizes,
    scene: BufferHandle,
    config: BufferHandle,
    gradient_image: ImageHandle,
    image_atlas: ImageHandle,
    antialiasing: Antialiasing,
}

fn record_coarse(recording: &mut RecordingBuilder, inputs: CoarseInputs) -> Result<FineSchedule> {
    let CoarseInputs {
        workgroups,
        sizes,
        scene,
        config,
        gradient_image,
        image_atlas,
        antialiasing,
    } = inputs;
    let (info_bin_data, tile, segments, per_tile_command_list) =
        allocate_coarse_outputs(recording, &sizes)?;
    let path_monoids = record_path_scan(recording, &workgroups, &sizes, scene, config)?;

    let (path_bboxes, bump, lines, draw_monoids, clip_bboxes) = record_draw_and_clip(
        recording,
        &workgroups,
        &sizes,
        scene,
        config,
        path_monoids,
        info_bin_data,
    )?;

    let draw_bboxes = buffer(
        recording,
        BufferRole::DrawBboxes,
        sizes.draw_bboxes.size_in_bytes(),
    )?;
    let bin_headers = buffer(
        recording,
        BufferRole::BinHeaders,
        sizes.bin_headers.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::binning(
        workgroups.binning,
        BinningBindings {
            config,
            draw_monoids,
            path_bboxes,
            clip_bboxes,
            draw_bboxes,
            bump,
            info_bin_data,
            bin_headers,
        },
    ))?;
    recording.release(path_bboxes)?;
    recording.release(clip_bboxes)?;

    let paths = buffer(recording, BufferRole::Paths, sizes.paths.size_in_bytes())?;
    recording.record_coarse(CoarseDispatch::tile_alloc(
        workgroups.tile_alloc,
        config,
        scene,
        draw_bboxes,
        bump,
        paths,
        tile,
    ))?;
    recording.release(draw_bboxes)?;
    recording.release(path_monoids)?;

    record_path_tiling(
        recording,
        &workgroups,
        &sizes,
        PathTilingInputs {
            config,
            scene,
            draw_monoids,
            bin_headers,
            info_bin_data,
            paths,
            tile,
            segments,
            lines,
            bump,
            per_tile_command_list,
        },
    )?;

    Ok(FineSchedule {
        variant: fine_variant(antialiasing),
        workgroups: workgroups.fine,
        config,
        tile,
        segments,
        per_tile_command_list,
        gradient_image,
        info_bin_data,
        image_atlas,
        blend_spill: buffer(
            recording,
            BufferRole::BlendSpill,
            sizes.blend_spill.size_in_bytes(),
        )?,
    })
}

fn allocate_coarse_outputs(
    recording: &mut RecordingBuilder,
    sizes: &vello_encoding::BufferSizes,
) -> Result<(BufferHandle, BufferHandle, BufferHandle, BufferHandle)> {
    Ok((
        buffer(
            recording,
            BufferRole::InfoBinData,
            sizes.bin_data.size_in_bytes(),
        )?,
        buffer(recording, BufferRole::Tile, sizes.tiles.size_in_bytes())?,
        buffer(
            recording,
            BufferRole::Segments,
            sizes.segments.size_in_bytes(),
        )?,
        buffer(
            recording,
            BufferRole::PerTileCommandList,
            sizes.ptcl.size_in_bytes(),
        )?,
    ))
}

fn record_path_scan(
    recording: &mut RecordingBuilder,
    workgroups: &vello_encoding::WorkgroupCounts,
    sizes: &vello_encoding::BufferSizes,
    scene: BufferHandle,
    config: BufferHandle,
) -> Result<BufferHandle> {
    let path_reduced = buffer(
        recording,
        BufferRole::PathReduced,
        sizes.path_reduced.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::path_tag_reduce(
        workgroups.path_reduce,
        config,
        scene,
        path_reduced,
    ))?;
    let mut path_tag_parent = path_reduced;
    let mut large_path_scan = None;
    if workgroups.use_large_path_scan {
        let path_reduced2 = buffer(
            recording,
            BufferRole::PathReduced2,
            sizes.path_reduced2.size_in_bytes(),
        )?;
        recording.record_coarse(CoarseDispatch::path_tag_reduce2(
            workgroups.path_reduce2,
            path_reduced,
            path_reduced2,
        ))?;
        let path_reduced_scan = buffer(
            recording,
            BufferRole::PathReducedScan,
            sizes.path_reduced_scan.size_in_bytes(),
        )?;
        recording.record_coarse(CoarseDispatch::path_tag_scan1(
            workgroups.path_scan1,
            path_reduced,
            path_reduced2,
            path_reduced_scan,
        ))?;
        path_tag_parent = path_reduced_scan;
        large_path_scan = Some((path_reduced2, path_reduced_scan));
    }
    let path_monoids = buffer(
        recording,
        BufferRole::PathMonoids,
        sizes.path_monoids.size_in_bytes(),
    )?;
    recording.record_coarse(if workgroups.use_large_path_scan {
        CoarseDispatch::path_tag_scan_large(
            workgroups.path_scan,
            config,
            scene,
            path_tag_parent,
            path_monoids,
        )
    } else {
        CoarseDispatch::path_tag_scan(
            workgroups.path_scan,
            config,
            scene,
            path_tag_parent,
            path_monoids,
        )
    })?;
    recording.release(path_reduced)?;
    if let Some((path_reduced2, path_reduced_scan)) = large_path_scan {
        recording.release(path_reduced2)?;
        recording.release(path_reduced_scan)?;
    }
    Ok(path_monoids)
}

type DrawAndClipBuffers = (
    BufferHandle,
    BufferHandle,
    BufferHandle,
    BufferHandle,
    BufferHandle,
);

struct PathTilingInputs {
    config: BufferHandle,
    scene: BufferHandle,
    draw_monoids: BufferHandle,
    bin_headers: BufferHandle,
    info_bin_data: BufferHandle,
    paths: BufferHandle,
    tile: BufferHandle,
    segments: BufferHandle,
    lines: BufferHandle,
    bump: BufferHandle,
    per_tile_command_list: BufferHandle,
}

fn record_path_tiling(
    recording: &mut RecordingBuilder,
    workgroups: &vello_encoding::WorkgroupCounts,
    sizes: &vello_encoding::BufferSizes,
    inputs: PathTilingInputs,
) -> Result<()> {
    let indirect_count = buffer(
        recording,
        BufferRole::IndirectCount,
        sizes.indirect_count.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::path_count_setup(
        workgroups.path_count_setup,
        inputs.bump,
        indirect_count,
    ))?;
    let segment_counts = buffer(
        recording,
        BufferRole::SegmentCounts,
        sizes.seg_counts.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::path_count(
        indirect_count,
        inputs.config,
        inputs.bump,
        inputs.lines,
        inputs.paths,
        inputs.tile,
        segment_counts,
    ))?;
    recording.record_coarse(CoarseDispatch::backdrop(
        workgroups.backdrop,
        inputs.config,
        inputs.bump,
        inputs.paths,
        inputs.tile,
    ))?;
    recording.record_coarse(CoarseDispatch::coarse(
        workgroups.coarse,
        CoarseRasterBindings {
            config: inputs.config,
            scene: inputs.scene,
            draw_monoids: inputs.draw_monoids,
            bin_headers: inputs.bin_headers,
            info_bin_data: inputs.info_bin_data,
            paths: inputs.paths,
            tile: inputs.tile,
            bump: inputs.bump,
            per_tile_command_list: inputs.per_tile_command_list,
        },
    ))?;
    recording.release(inputs.draw_monoids)?;
    recording.release(inputs.bin_headers)?;
    recording.release(inputs.scene)?;
    recording.record_coarse(CoarseDispatch::path_tiling_setup(
        workgroups.path_tiling_setup,
        inputs.bump,
        indirect_count,
        inputs.per_tile_command_list,
    ))?;
    recording.record_coarse(CoarseDispatch::path_tiling(
        indirect_count,
        inputs.bump,
        segment_counts,
        inputs.lines,
        inputs.paths,
        inputs.tile,
        inputs.segments,
    ))?;
    recording.release(indirect_count)?;
    recording.release(segment_counts)?;
    recording.release(inputs.lines)?;
    recording.release(inputs.paths)?;
    recording.release(inputs.bump)?;
    Ok(())
}

fn record_draw_and_clip(
    recording: &mut RecordingBuilder,
    workgroups: &vello_encoding::WorkgroupCounts,
    sizes: &vello_encoding::BufferSizes,
    scene: BufferHandle,
    config: BufferHandle,
    path_monoids: BufferHandle,
    info_bin_data: BufferHandle,
) -> Result<DrawAndClipBuffers> {
    let path_bboxes = buffer(
        recording,
        BufferRole::PathBboxes,
        sizes.path_bboxes.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::bbox_clear(
        workgroups.bbox_clear,
        config,
        path_bboxes,
    ))?;
    let bump = buffer(
        recording,
        BufferRole::Bump,
        sizes.bump_alloc.size_in_bytes(),
    )?;
    recording.clear_buffer(bump)?;
    let lines = buffer(recording, BufferRole::Lines, sizes.lines.size_in_bytes())?;
    recording.record_coarse(CoarseDispatch::flatten(
        workgroups.flatten,
        config,
        scene,
        path_monoids,
        path_bboxes,
        bump,
        lines,
    ))?;
    let draw_reduced = buffer(
        recording,
        BufferRole::DrawReduced,
        sizes.draw_reduced.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::draw_reduce(
        workgroups.draw_reduce,
        config,
        scene,
        draw_reduced,
    ))?;
    let draw_monoids = buffer(
        recording,
        BufferRole::DrawMonoids,
        sizes.draw_monoids.size_in_bytes(),
    )?;
    let clip_inputs = buffer(
        recording,
        BufferRole::ClipInputs,
        sizes.clip_inps.size_in_bytes(),
    )?;
    recording.record_coarse(CoarseDispatch::draw_leaf(
        workgroups.draw_leaf,
        DrawLeafBindings {
            config,
            scene,
            draw_reduced,
            path_bboxes,
            draw_monoids,
            info_bin_data,
            clip_inputs,
        },
    ))?;
    recording.release(draw_reduced)?;
    let clip_bboxes = record_clip_stage(
        recording,
        workgroups,
        sizes,
        config,
        path_bboxes,
        draw_monoids,
        clip_inputs,
    )?;
    Ok((path_bboxes, bump, lines, draw_monoids, clip_bboxes))
}

fn record_clip_stage(
    recording: &mut RecordingBuilder,
    workgroups: &vello_encoding::WorkgroupCounts,
    sizes: &vello_encoding::BufferSizes,
    config: BufferHandle,
    path_bboxes: BufferHandle,
    draw_monoids: BufferHandle,
    clip_inputs: BufferHandle,
) -> Result<BufferHandle> {
    let clip_elements = buffer(
        recording,
        BufferRole::ClipElements,
        sizes.clip_els.size_in_bytes(),
    )?;
    let clip_bics = buffer(
        recording,
        BufferRole::ClipBics,
        sizes.clip_bics.size_in_bytes(),
    )?;
    if workgroups.clip_reduce.0 > 0 {
        recording.record_coarse(CoarseDispatch::clip_reduce(
            workgroups.clip_reduce,
            clip_inputs,
            path_bboxes,
            clip_bics,
            clip_elements,
        ))?;
    }
    let clip_bboxes = buffer(
        recording,
        BufferRole::ClipBboxes,
        sizes.clip_bboxes.size_in_bytes(),
    )?;
    if workgroups.clip_leaf.0 > 0 {
        recording.record_coarse(CoarseDispatch::clip_leaf(
            workgroups.clip_leaf,
            ClipLeafBindings {
                config,
                clip_inputs,
                path_bboxes,
                clip_bics,
                clip_elements,
                draw_monoids,
                clip_bboxes,
            },
        ))?;
    }
    recording.release(clip_inputs)?;
    recording.release(clip_bics)?;
    recording.release(clip_elements)?;
    Ok(clip_bboxes)
}

fn record_fine(recording: &mut RecordingBuilder, fine: FineSchedule) -> Result<()> {
    let bindings = FineDispatchBindings {
        workgroups: fine.workgroups,
        config: fine.config,
        segments: fine.segments,
        per_tile_command_list: fine.per_tile_command_list,
        info_bin_data: fine.info_bin_data,
        blend_spill: fine.blend_spill,
        gradient_image: fine.gradient_image,
        image_atlas: fine.image_atlas,
    };
    match fine.variant {
        FineRasterVariant::Area => recording.record_fine(FineDispatch::area(bindings))?,
        FineRasterVariant::Msaa8 => {
            let mask_lut = recording.upload_mask_lut(
                FineRasterVariant::Msaa8,
                mask_lut(FineRasterVariant::Msaa8, recording.account())?,
            )?;
            recording.record_fine(FineDispatch::msaa8(bindings, mask_lut))?;
            recording.release(mask_lut)?;
        }
        FineRasterVariant::Msaa16 => {
            let mask_lut = recording.upload_mask_lut(
                FineRasterVariant::Msaa16,
                mask_lut(FineRasterVariant::Msaa16, recording.account())?,
            )?;
            recording.record_fine(FineDispatch::msaa16(bindings, mask_lut))?;
            recording.release(mask_lut)?;
        }
    }
    release_fine_resources(recording, fine)?;
    Ok(())
}

fn release_fine_resources(recording: &mut RecordingBuilder, fine: FineSchedule) -> Result<()> {
    recording.release(fine.config)?;
    recording.release(fine.tile)?;
    recording.release(fine.segments)?;
    recording.release(fine.per_tile_command_list)?;
    recording.release(fine.gradient_image)?;
    recording.release(fine.info_bin_data)?;
    recording.release(fine.blend_spill)?;
    Ok(())
}

fn buffer(
    recording: &mut RecordingBuilder,
    role: BufferRole,
    byte_len: u32,
) -> Result<BufferHandle> {
    recording.new_buffer(role, u64::from(byte_len))
}

const fn fine_variant(antialiasing: Antialiasing) -> FineRasterVariant {
    match antialiasing {
        Antialiasing::Area => FineRasterVariant::Area,
        Antialiasing::Msaa8 => FineRasterVariant::Msaa8,
        Antialiasing::Msaa16 => FineRasterVariant::Msaa16,
    }
}

fn validate_target_dimension(field: &'static str, value: u32) -> Result<()> {
    if value == 0 || value > u32::MAX - (VELLO_TILE_EXTENT - 1) {
        return Err(Error::invalid_value(
            field,
            value,
            "must be nonzero and leave room for Vello tile padding",
        ));
    }
    Ok(())
}

#[cfg(test)]
impl PreparedVelloPass {
    pub(crate) fn observation_for_test(&self) -> PreparedVelloPassObservation {
        PreparedVelloPassObservation {
            target_extent: self.target_intent.extent,
            transparent_base: self.target_intent.base_color == Color::TRANSPARENT,
            antialiasing: self.target_intent.antialiasing,
            is_rgba8_storage: self.target_intent.is_rgba8_storage_for_test(),
            final_dispatch_targets_output: self.recording.final_dispatch_targets_output_for_test(),
            is_self_consistent: self
                .recording
                .is_self_consistent_for_test(&self.resource_intents),
            has_persistent_image_atlas: self
                .resource_intents
                .iter()
                .any(ResourceIntent::is_persistent_image_atlas_for_test),
            has_transient_buffer: self
                .resource_intents
                .iter()
                .any(ResourceIntent::is_transient_buffer_for_test),
        }
    }
}

#[cfg(test)]
impl RasterTargetIntent {
    const fn is_rgba8_storage_for_test(&self) -> bool {
        matches!(self.format, RasterImageFormat::Rgba8Unorm)
            && matches!(self.access, RasterTargetAccess::StorageWrite)
    }
}

/// The two locked LUT factories retain construction custody until return. Their
/// public API has no fallible or caller-buffer alternative. Preview the literal
/// output before invoking them, then admit observed capacity at handoff; a refusal
/// drops the unpublished result. The 16-sample factory's private intermediate and
/// inherited infallible allocation are part of this narrow dependency boundary.
fn mask_lut(variant: FineRasterVariant, account: &Account) -> Result<AccountedVec<u8>> {
    let (count, sample_visits) = match variant {
        FineRasterVariant::Msaa8 => (1024, 1024 * 8),
        FineRasterVariant::Msaa16 => (8192, 4096 * 16),
        FineRasterVariant::Area => {
            return Err(Error::new(
                crate::BackendErrorCode::RenderFailed,
                "area rasterization has no mask LUT",
            ));
        }
    };
    let account = account.at(RenderPhase::Kernel);
    account.preview(RenderResource::RetainedCpuBytes, count)?;
    let values = account.phase(RenderPhase::Kernel, || {
        account.work(RenderWork::KernelSampleVisits, sample_visits)?;
        Ok(match variant {
            FineRasterVariant::Msaa8 => make_mask_lut(),
            FineRasterVariant::Msaa16 => make_mask_lut_16(),
            FineRasterVariant::Area => unreachable!(),
        })
    })?;
    if values.len() != count {
        return Err(Error::new(
            crate::BackendErrorCode::RenderFailed,
            "locked Vello mask LUT format changed",
        ));
    }
    let hold = account.acquire(RenderResource::RetainedCpuBytes, values.capacity())?;
    AccountedVec::from_parts(values, hold)
}

#[cfg(test)]
mod lut_tests {
    use super::*;
    use crate::accounting::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderClock,
        RenderPhaseFilter, RenderProfile,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn locked_lut_handoff_has_literal_lengths_and_extreme_sample_masks() {
        let mut context = RenderAccountingContext::default();
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                crate::GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
                #[cfg(feature = "render-text")]
                None,
                |account| {
                    let lut = mask_lut(FineRasterVariant::Msaa8, account)?;
                    assert_eq!(lut.len(), 1024);
                    assert_eq!(lut.capacity(), 1024);
                    assert_eq!((lut[0], lut[31]), (0, 255));
                    let lut16 = mask_lut(FineRasterVariant::Msaa16, account)?;
                    assert_eq!(lut16.len(), 8192);
                    assert_eq!(lut16.capacity(), 8192);
                    assert_eq!(&lut16[..2], &[0, 0]);
                    assert_eq!(&lut16[126..128], &[255, 255]);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            context
                .last_attempt()
                .unwrap().attempt().unwrap()
                .usage()
                .peak(RenderResource::RetainedCpuBytes),
            9216
        );
    }

    #[test]
    fn insufficient_lut_preview_does_no_factory_work_or_upload_recording() {
        let mut context = RenderAccountingContext::default();
        context
            .prepare(
                RenderAttemptKind::GlyphRun,
                crate::GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, 1023),
                #[cfg(feature = "render-text")]
                None,
                |account| {
                    assert!(mask_lut(FineRasterVariant::Msaa8, account).is_err());
                    Ok(())
                },
            )
            .unwrap();
        let usage = context.last_attempt().unwrap().attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::KernelSampleVisits), 0);
        assert_eq!(usage.work(RenderWork::UploadBytes), 0);
        assert_eq!(usage.peak(RenderResource::RetainedCpuBytes), 0);
    }

    static TICKS: AtomicU64 = AtomicU64::new(0);
    fn ticking_clock() -> std::time::Duration {
        std::time::Duration::from_nanos(TICKS.fetch_add(1, Ordering::SeqCst))
    }
    fn forbidden_clock() -> std::time::Duration {
        panic!("untimed LUT must not read clock")
    }

    #[test]
    fn lut_factory_phase_uses_selected_clock_only_for_timed_profiles() {
        for timing in [false, true] {
            TICKS.store(0, Ordering::SeqCst);
            let options = RenderAccountingOptions::new()
                .with_clock(RenderClock::Host(if timing {
                    ticking_clock
                } else {
                    forbidden_clock
                }))
                .with_profile(RenderProfile::Detailed {
                    timing,
                    event_limit: 64,
                    filter: RenderPhaseFilter::Phase(RenderPhase::Kernel),
                });
            let mut context = RenderAccountingContext::new(options);
            context
                .prepare(
                    RenderAttemptKind::GlyphRun,
                    crate::GlyphRunLimits::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX),
                    #[cfg(feature = "render-text")]
                    None,
                    |account| {
                        drop(mask_lut(FineRasterVariant::Msaa8, account)?);
                        Ok(())
                    },
                )
                .unwrap();
            assert_eq!(TICKS.load(Ordering::SeqCst) > 0, timing);
            assert_eq!(
                context
                    .last_attempt()
                    .unwrap().attempt().unwrap()
                    .usage()
                    .work(RenderWork::KernelSampleVisits),
                8192
            );
        }
    }
}

#[cfg(test)]
mod image_accounting_tests {
    use super::super::scene::VelloScene;
    use super::*;
    use crate::accounting::{
        FrameAttempt, FrameReportSlot, RenderAccountingError, RenderAccountingOptions,
        RenderAttemptStatus, RenderFrameReport, RenderLimits, RenderQuantity, RenderRefusalReason,
        RendererCpu,
    };
    use crate::{Image, Size};

    fn image() -> Image {
        Image::from_rgba(Size::new(1.0, 1.0), vec![7, 11, 13, 255]).unwrap()
    }

    fn parameters() -> RasterParameters {
        RasterParameters::try_new(
            PhysicalSize::new(2, 2),
            Color::TRANSPARENT,
            Antialiasing::Area,
        )
        .unwrap()
    }

    fn scene(images: &[&Image], pixelated: bool, account: &Account) -> Result<VelloScene> {
        let mut scene = VelloScene::default();
        for image in images {
            let brush = crate::encode::image_brush(image);
            if pixelated {
                scene.with_pixelated_image(
                    image,
                    kurbo::Affine::IDENTITY,
                    |scene, compensation| scene.draw_image(&brush, compensation, account),
                    account,
                )?;
            } else {
                scene.import_image(image, account)?;
                scene.draw_image(&brush, kurbo::Affine::IDENTITY, account)?;
            }
        }
        Ok(scene)
    }

    fn attempt(
        limits: RenderLimits,
        run: impl FnOnce(&Account) -> Result<()>,
    ) -> (Result<()>, RenderFrameReport) {
        let options = RenderAccountingOptions::new().with_limits(limits);
        let cpu = RendererCpu::new(options).unwrap();
        let attempt = FrameAttempt::begin(FrameReportSlot::default(), &cpu, options).unwrap();
        let result = run(attempt.account());
        let status = match &result {
            Ok(()) => RenderAttemptStatus::Succeeded,
            Err(error) => attempt.status_for(error),
        };
        let diagnostic = result
            .as_ref()
            .err()
            .and_then(|error| error.accounting_diagnostic());
        let (report, fault) = attempt.finish(status, diagnostic);
        assert_eq!(fault, None);
        (result, report)
    }

    fn prepare(
        images: &[&Image],
        pixelated: bool,
        limits: RenderLimits,
    ) -> (Result<()>, RenderFrameReport) {
        attempt(limits, |account| {
            let scene = scene(images, pixelated, account)?;
            scene.prepare_raster(parameters(), account).map(drop)
        })
    }

    #[test]
    fn ordinary_image_preparation_counts_blob_identity_once_per_atlas_build() {
        let first = image();
        let equal = image();
        // Both are literal 1x1 RGBA8 images. The ordinary Resolver keys by Blob
        // identity: repeated draws share one region; independently owned equal
        // content still introduces a second four-byte, one-texel region.
        for (images, bytes, texels) in [
            (vec![&first], 4, 1),
            (vec![&first, &first], 4, 1),
            (vec![&first, &equal], 8, 2),
        ] {
            let (result, report) = prepare(&images, false, RenderLimits::new());
            result.unwrap();
            let usage = report.attempt().unwrap().usage();
            assert_eq!(usage.work(RenderWork::ImageSourceBytes), bytes);
            assert_eq!(usage.work(RenderWork::AtlasTexels), texels);
            assert_eq!(usage.peak(RenderResource::LocalAtlasSourceByteQuota), bytes);
            assert_eq!(usage.peak(RenderResource::LocalAtlasTexelQuota), texels);
            assert_eq!(usage.live(RenderResource::LocalAtlasSourceByteQuota), 0);
            assert_eq!(usage.live(RenderResource::LocalAtlasTexelQuota), 0);
        }
    }

    #[test]
    fn pixelated_image_preparation_counts_the_same_unexpanded_literal_input() {
        let image = image();
        let (result, report) = prepare(&[&image], true, RenderLimits::new());
        result.unwrap();
        let usage = report.attempt().unwrap().usage();
        // Identity transform requires no integer expansion: the working custom
        // path must observe the same four source bytes and one selected texel.
        assert_eq!(usage.work(RenderWork::ImageSourceBytes), 4);
        assert_eq!(usage.work(RenderWork::AtlasTexels), 1);
        assert_eq!(usage.peak(RenderResource::LocalAtlasSourceByteQuota), 4);
        assert_eq!(usage.peak(RenderResource::LocalAtlasTexelQuota), 1);
    }

    #[test]
    fn empty_ordinary_image_is_transparent_without_admitting_an_atlas_region() {
        let image = Image::from_rgba(Size::new(0.0, 1.0), Vec::<u8>::new()).unwrap();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ImageSourceBytes, 0)
            .unwrap()
            .with_work_limit(RenderWork::AtlasTexels, 0)
            .unwrap();
        let (result, report) = prepare(&[&image], false, limits);
        result.unwrap();
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::ImageSourceBytes), 0);
        assert_eq!(usage.work(RenderWork::AtlasTexels), 0);
        assert_eq!(usage.peak(RenderResource::LocalAtlasSourceByteQuota), 0);
        assert_eq!(usage.peak(RenderResource::LocalAtlasTexelQuota), 0);
    }

    fn work_limit_refuses_one_under_and_accepts_exact(work: RenderWork, exact: u64) {
        let image = image();
        let limit = exact - 1;
        let limits = RenderLimits::new().with_work_limit(work, limit).unwrap();
        let (result, report) = prepare(&[&image], false, limits);
        let error = result.unwrap_err();
        let Some(RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic() else {
            panic!("ordinary image preparation must retain its typed work refusal: {error}");
        };
        assert_eq!(refusal.quantity(), RenderQuantity::Work(work));
        assert_eq!(refusal.current(), 0);
        assert_eq!(refusal.amount(), exact);
        assert_eq!(refusal.reason(), RenderRefusalReason::Limit { limit });
        assert_eq!(report.status(), RenderAttemptStatus::Refused);
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(work), 0);
        assert_eq!(usage.work(RenderWork::UploadBytes), 0);
        assert_eq!(usage.work(RenderWork::ResourceAllocationRequests), 0);
        assert_eq!(usage.work(RenderWork::PreparedTransferredBytes), 0);
        assert_eq!(usage.live(RenderResource::LocalAtlasSourceByteQuota), 0);
        assert_eq!(usage.live(RenderResource::LocalAtlasTexelQuota), 0);
        let limits = RenderLimits::new().with_work_limit(work, exact).unwrap();
        let (result, report) = prepare(&[&image], false, limits);
        result.unwrap();
        assert_eq!(report.status(), RenderAttemptStatus::Succeeded);
        assert_eq!(report.attempt().unwrap().usage().work(work), exact);
    }

    #[test]
    fn ordinary_image_source_byte_limit_refuses_three_and_accepts_four() {
        work_limit_refuses_one_under_and_accepts_exact(RenderWork::ImageSourceBytes, 4);
    }

    #[test]
    fn ordinary_image_texel_limit_refuses_zero_and_accepts_one() {
        work_limit_refuses_one_under_and_accepts_exact(RenderWork::AtlasTexels, 1);
    }

    #[test]
    fn repeated_ordinary_preparation_accumulates_work_but_renews_local_atlas_quota() {
        let image = image();
        let limits = RenderLimits::new()
            .with_work_limit(RenderWork::ImageSourceBytes, 8)
            .unwrap()
            .with_work_limit(RenderWork::AtlasTexels, 2)
            .unwrap();
        let (result, report) = attempt(limits, |account| {
            let scene = scene(&[&image], false, account)?;
            // prepare_raster creates a fresh Resolver. Reusing the same input
            // therefore prepares the same four bytes/one texel on each call.
            drop(scene.prepare_raster(parameters(), account)?);
            drop(scene.prepare_raster(parameters(), account)?);
            Ok(())
        });
        result.unwrap();
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::ImageSourceBytes), 8);
        assert_eq!(usage.work(RenderWork::AtlasTexels), 2);
        assert_eq!(usage.peak(RenderResource::LocalAtlasSourceByteQuota), 4);
        assert_eq!(usage.peak(RenderResource::LocalAtlasTexelQuota), 1);
        assert_eq!(usage.live(RenderResource::LocalAtlasSourceByteQuota), 0);
        assert_eq!(usage.live(RenderResource::LocalAtlasTexelQuota), 0);
    }
}

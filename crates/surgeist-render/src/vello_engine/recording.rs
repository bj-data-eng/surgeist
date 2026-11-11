// Copyright 2022 the Vello Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Modified for surgeist-render; see NOTICE-VELLO.md for provenance and adaptations.

#[cfg(test)]
use std::mem::size_of;

use peniko::ImageData;
use vello_encoding::ConfigUniform;

use super::bounded::PayloadCustody;
use crate::accounting::{Account, AccountedVec, RenderWork};
use crate::{BackendErrorCode, Error, PhysicalSize, Result};

/// A symbolic resource identity within one prepared Vello pass.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ResourceId(u64);

/// A symbolic buffer reference used by the compute-dispatch recording.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct BufferHandle(ResourceId);

/// A symbolic image reference used by the compute-dispatch recording.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct ImageHandle(ResourceId);

/// The only image format required by the Vello raster path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum RasterImageFormat {
    Rgba8Unorm,
}

/// The fixed algorithm phase associated with a recorded dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RasterPhase {
    ImagePreparation,
    Coarse,
    Fine,
}

/// The antialiasing-specific final raster program selected for one pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FineRasterVariant {
    Area,
    Msaa8,
    Msaa16,
}

/// The closed set of compute programs used by the raster schedule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RasterKernel {
    ExpandImage,
    PathTagReduce,
    PathTagReduce2,
    PathTagScan1,
    PathTagScan,
    PathTagScanLarge,
    BboxClear,
    Flatten,
    DrawReduce,
    DrawLeaf,
    ClipReduce,
    ClipLeaf,
    Binning,
    TileAlloc,
    PathCountSetup,
    PathCount,
    Backdrop,
    Coarse,
    PathTilingSetup,
    PathTiling,
    FineArea,
    FineMsaa8,
    FineMsaa16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResourceBinding {
    Buffer(BufferHandle),
    Image(ImageHandle),
    TargetOutput,
}

/// Built-in binding arrays remain inline in each command's backing.
/// The word-sized tag preserves the footprint of the former length word.
#[repr(usize)]
enum Bindings {
    Two([ResourceBinding; 2]),
    Three([ResourceBinding; 3]),
    Four([ResourceBinding; 4]),
    Six([ResourceBinding; 6]),
    Seven([ResourceBinding; 7]),
    Eight([ResourceBinding; 8]),
    Nine([ResourceBinding; 9]),
}
impl std::ops::Deref for Bindings {
    type Target = [ResourceBinding];
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Two(values) => values,
            Self::Three(values) => values,
            Self::Four(values) => values,
            Self::Six(values) => values,
            Self::Seven(values) => values,
            Self::Eight(values) => values,
            Self::Nine(values) => values,
        }
    }
}
impl<'a> IntoIterator for &'a Bindings {
    type Item = &'a ResourceBinding;
    type IntoIter = std::slice::Iter<'a, ResourceBinding>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// The bindings accepted by the coarse draw-leaf operation.
pub(super) struct DrawLeafBindings {
    pub(super) config: BufferHandle,
    pub(super) scene: BufferHandle,
    pub(super) draw_reduced: BufferHandle,
    pub(super) path_bboxes: BufferHandle,
    pub(super) draw_monoids: BufferHandle,
    pub(super) info_bin_data: BufferHandle,
    pub(super) clip_inputs: BufferHandle,
}

/// The bindings accepted by the coarse clip-leaf operation.
pub(super) struct ClipLeafBindings {
    pub(super) config: BufferHandle,
    pub(super) clip_inputs: BufferHandle,
    pub(super) path_bboxes: BufferHandle,
    pub(super) clip_bics: BufferHandle,
    pub(super) clip_elements: BufferHandle,
    pub(super) draw_monoids: BufferHandle,
    pub(super) clip_bboxes: BufferHandle,
}

/// The bindings accepted by the coarse binning operation.
pub(super) struct BinningBindings {
    pub(super) config: BufferHandle,
    pub(super) draw_monoids: BufferHandle,
    pub(super) path_bboxes: BufferHandle,
    pub(super) clip_bboxes: BufferHandle,
    pub(super) draw_bboxes: BufferHandle,
    pub(super) bump: BufferHandle,
    pub(super) info_bin_data: BufferHandle,
    pub(super) bin_headers: BufferHandle,
}

/// The bindings accepted by the final coarse-raster operation.
pub(super) struct CoarseRasterBindings {
    pub(super) config: BufferHandle,
    pub(super) scene: BufferHandle,
    pub(super) draw_monoids: BufferHandle,
    pub(super) bin_headers: BufferHandle,
    pub(super) info_bin_data: BufferHandle,
    pub(super) paths: BufferHandle,
    pub(super) tile: BufferHandle,
    pub(super) bump: BufferHandle,
    pub(super) per_tile_command_list: BufferHandle,
}

/// A coarse-phase operation whose constructor fixes its kernel and binding layout.
pub(super) struct CoarseDispatch(DispatchIntent);

impl CoarseDispatch {
    pub(super) fn path_tag_reduce(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        path_reduced: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTagReduce,
            workgroups,
            Bindings::Three([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(path_reduced),
            ]),
        ))
    }

    pub(super) fn path_tag_reduce2(
        workgroups: vello_encoding::WorkgroupSize,
        path_reduced: BufferHandle,
        path_reduced2: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTagReduce2,
            workgroups,
            Bindings::Two([
                ResourceBinding::Buffer(path_reduced),
                ResourceBinding::Buffer(path_reduced2),
            ]),
        ))
    }

    pub(super) fn path_tag_scan1(
        workgroups: vello_encoding::WorkgroupSize,
        path_reduced: BufferHandle,
        path_reduced2: BufferHandle,
        path_reduced_scan: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTagScan1,
            workgroups,
            Bindings::Three([
                ResourceBinding::Buffer(path_reduced),
                ResourceBinding::Buffer(path_reduced2),
                ResourceBinding::Buffer(path_reduced_scan),
            ]),
        ))
    }

    pub(super) fn path_tag_scan(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        path_tag_parent: BufferHandle,
        path_monoids: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTagScan,
            workgroups,
            Bindings::Four([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(path_tag_parent),
                ResourceBinding::Buffer(path_monoids),
            ]),
        ))
    }

    pub(super) fn path_tag_scan_large(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        path_tag_parent: BufferHandle,
        path_monoids: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTagScanLarge,
            workgroups,
            Bindings::Four([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(path_tag_parent),
                ResourceBinding::Buffer(path_monoids),
            ]),
        ))
    }

    pub(super) fn bbox_clear(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        path_bboxes: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::BboxClear,
            workgroups,
            Bindings::Two([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(path_bboxes),
            ]),
        ))
    }

    pub(super) fn flatten(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        path_monoids: BufferHandle,
        path_bboxes: BufferHandle,
        bump: BufferHandle,
        lines: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::Flatten,
            workgroups,
            Bindings::Six([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(path_monoids),
                ResourceBinding::Buffer(path_bboxes),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(lines),
            ]),
        ))
    }

    pub(super) fn draw_reduce(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        draw_reduced: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::DrawReduce,
            workgroups,
            Bindings::Three([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(draw_reduced),
            ]),
        ))
    }

    pub(super) fn draw_leaf(
        workgroups: vello_encoding::WorkgroupSize,
        bindings: DrawLeafBindings,
    ) -> Self {
        let DrawLeafBindings {
            config,
            scene,
            draw_reduced,
            path_bboxes,
            draw_monoids,
            info_bin_data,
            clip_inputs,
        } = bindings;
        Self(DispatchIntent::coarse_direct(
            RasterKernel::DrawLeaf,
            workgroups,
            Bindings::Seven([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(draw_reduced),
                ResourceBinding::Buffer(path_bboxes),
                ResourceBinding::Buffer(draw_monoids),
                ResourceBinding::Buffer(info_bin_data),
                ResourceBinding::Buffer(clip_inputs),
            ]),
        ))
    }

    pub(super) fn clip_reduce(
        workgroups: vello_encoding::WorkgroupSize,
        clip_inputs: BufferHandle,
        path_bboxes: BufferHandle,
        clip_bics: BufferHandle,
        clip_elements: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::ClipReduce,
            workgroups,
            Bindings::Four([
                ResourceBinding::Buffer(clip_inputs),
                ResourceBinding::Buffer(path_bboxes),
                ResourceBinding::Buffer(clip_bics),
                ResourceBinding::Buffer(clip_elements),
            ]),
        ))
    }

    pub(super) fn clip_leaf(
        workgroups: vello_encoding::WorkgroupSize,
        bindings: ClipLeafBindings,
    ) -> Self {
        let ClipLeafBindings {
            config,
            clip_inputs,
            path_bboxes,
            clip_bics,
            clip_elements,
            draw_monoids,
            clip_bboxes,
        } = bindings;
        Self(DispatchIntent::coarse_direct(
            RasterKernel::ClipLeaf,
            workgroups,
            Bindings::Seven([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(clip_inputs),
                ResourceBinding::Buffer(path_bboxes),
                ResourceBinding::Buffer(clip_bics),
                ResourceBinding::Buffer(clip_elements),
                ResourceBinding::Buffer(draw_monoids),
                ResourceBinding::Buffer(clip_bboxes),
            ]),
        ))
    }

    pub(super) fn binning(
        workgroups: vello_encoding::WorkgroupSize,
        bindings: BinningBindings,
    ) -> Self {
        let BinningBindings {
            config,
            draw_monoids,
            path_bboxes,
            clip_bboxes,
            draw_bboxes,
            bump,
            info_bin_data,
            bin_headers,
        } = bindings;
        Self(DispatchIntent::coarse_direct(
            RasterKernel::Binning,
            workgroups,
            Bindings::Eight([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(draw_monoids),
                ResourceBinding::Buffer(path_bboxes),
                ResourceBinding::Buffer(clip_bboxes),
                ResourceBinding::Buffer(draw_bboxes),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(info_bin_data),
                ResourceBinding::Buffer(bin_headers),
            ]),
        ))
    }

    pub(super) fn tile_alloc(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        scene: BufferHandle,
        draw_bboxes: BufferHandle,
        bump: BufferHandle,
        paths: BufferHandle,
        tile: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::TileAlloc,
            workgroups,
            Bindings::Six([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(draw_bboxes),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(paths),
                ResourceBinding::Buffer(tile),
            ]),
        ))
    }

    pub(super) fn path_count_setup(
        workgroups: vello_encoding::WorkgroupSize,
        bump: BufferHandle,
        indirect_count: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathCountSetup,
            workgroups,
            Bindings::Two([
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(indirect_count),
            ]),
        ))
    }

    pub(super) fn path_count(
        indirect_count: BufferHandle,
        config: BufferHandle,
        bump: BufferHandle,
        lines: BufferHandle,
        paths: BufferHandle,
        tile: BufferHandle,
        segment_counts: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_indirect(
            RasterKernel::PathCount,
            indirect_count,
            Bindings::Six([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(lines),
                ResourceBinding::Buffer(paths),
                ResourceBinding::Buffer(tile),
                ResourceBinding::Buffer(segment_counts),
            ]),
        ))
    }

    pub(super) fn backdrop(
        workgroups: vello_encoding::WorkgroupSize,
        config: BufferHandle,
        bump: BufferHandle,
        paths: BufferHandle,
        tile: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::Backdrop,
            workgroups,
            Bindings::Four([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(paths),
                ResourceBinding::Buffer(tile),
            ]),
        ))
    }

    pub(super) fn coarse(
        workgroups: vello_encoding::WorkgroupSize,
        bindings: CoarseRasterBindings,
    ) -> Self {
        let CoarseRasterBindings {
            config,
            scene,
            draw_monoids,
            bin_headers,
            info_bin_data,
            paths,
            tile,
            bump,
            per_tile_command_list,
        } = bindings;
        Self(DispatchIntent::coarse_direct(
            RasterKernel::Coarse,
            workgroups,
            Bindings::Nine([
                ResourceBinding::Buffer(config),
                ResourceBinding::Buffer(scene),
                ResourceBinding::Buffer(draw_monoids),
                ResourceBinding::Buffer(bin_headers),
                ResourceBinding::Buffer(info_bin_data),
                ResourceBinding::Buffer(paths),
                ResourceBinding::Buffer(tile),
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(per_tile_command_list),
            ]),
        ))
    }

    pub(super) fn path_tiling_setup(
        workgroups: vello_encoding::WorkgroupSize,
        bump: BufferHandle,
        indirect_count: BufferHandle,
        per_tile_command_list: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_direct(
            RasterKernel::PathTilingSetup,
            workgroups,
            Bindings::Three([
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(indirect_count),
                ResourceBinding::Buffer(per_tile_command_list),
            ]),
        ))
    }

    pub(super) fn path_tiling(
        indirect_count: BufferHandle,
        bump: BufferHandle,
        segment_counts: BufferHandle,
        lines: BufferHandle,
        paths: BufferHandle,
        tile: BufferHandle,
        segments: BufferHandle,
    ) -> Self {
        Self(DispatchIntent::coarse_indirect(
            RasterKernel::PathTiling,
            indirect_count,
            Bindings::Six([
                ResourceBinding::Buffer(bump),
                ResourceBinding::Buffer(segment_counts),
                ResourceBinding::Buffer(lines),
                ResourceBinding::Buffer(paths),
                ResourceBinding::Buffer(tile),
                ResourceBinding::Buffer(segments),
            ]),
        ))
    }

    fn into_dispatch_intent(self) -> DispatchIntent {
        self.0
    }
}

/// The shared resources bound by every fine-phase operation.
pub(super) struct FineDispatchBindings {
    pub(super) workgroups: vello_encoding::WorkgroupSize,
    pub(super) config: BufferHandle,
    pub(super) segments: BufferHandle,
    pub(super) per_tile_command_list: BufferHandle,
    pub(super) info_bin_data: BufferHandle,
    pub(super) blend_spill: BufferHandle,
    pub(super) gradient_image: ImageHandle,
    pub(super) image_atlas: ImageHandle,
}

impl FineDispatchBindings {
    fn into_parts(self) -> (vello_encoding::WorkgroupSize, [ResourceBinding; 8]) {
        (
            self.workgroups,
            [
                ResourceBinding::Buffer(self.config),
                ResourceBinding::Buffer(self.segments),
                ResourceBinding::Buffer(self.per_tile_command_list),
                ResourceBinding::Buffer(self.info_bin_data),
                ResourceBinding::Buffer(self.blend_spill),
                ResourceBinding::TargetOutput,
                ResourceBinding::Image(self.gradient_image),
                ResourceBinding::Image(self.image_atlas),
            ],
        )
    }

    fn into_parts_with_mask_lut(
        self,
        mask_lut: BufferHandle,
    ) -> (vello_encoding::WorkgroupSize, [ResourceBinding; 9]) {
        let (workgroups, bindings) = self.into_parts();
        let [
            config,
            segments,
            per_tile_command_list,
            info_bin_data,
            blend_spill,
            target,
            gradient,
            atlas,
        ] = bindings;
        (
            workgroups,
            [
                config,
                segments,
                per_tile_command_list,
                info_bin_data,
                blend_spill,
                target,
                gradient,
                atlas,
                ResourceBinding::Buffer(mask_lut),
            ],
        )
    }
}

/// A fine-phase operation with a fixed final-raster kernel and binding layout.
pub(super) struct FineDispatch(DispatchIntent);

impl FineDispatch {
    pub(super) fn area(bindings: FineDispatchBindings) -> Self {
        let (workgroups, bindings) = bindings.into_parts();
        Self(DispatchIntent::fine_direct(
            RasterKernel::FineArea,
            workgroups,
            Bindings::Eight(bindings),
        ))
    }

    pub(super) fn msaa8(bindings: FineDispatchBindings, mask_lut: BufferHandle) -> Self {
        let (workgroups, bindings) = bindings.into_parts_with_mask_lut(mask_lut);
        Self(DispatchIntent::fine_direct(
            RasterKernel::FineMsaa8,
            workgroups,
            Bindings::Nine(bindings),
        ))
    }

    pub(super) fn msaa16(bindings: FineDispatchBindings, mask_lut: BufferHandle) -> Self {
        let (workgroups, bindings) = bindings.into_parts_with_mask_lut(mask_lut);
        Self(DispatchIntent::fine_direct(
            RasterKernel::FineMsaa16,
            workgroups,
            Bindings::Nine(bindings),
        ))
    }

    fn into_dispatch_intent(self) -> DispatchIntent {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResourceReference {
    Buffer(BufferHandle),
    Image(ImageHandle),
}

impl From<BufferHandle> for ResourceReference {
    fn from(value: BufferHandle) -> Self {
        Self::Buffer(value)
    }
}

impl From<ImageHandle> for ResourceReference {
    fn from(value: ImageHandle) -> Self {
        Self::Image(value)
    }
}

/// A recorded compute dispatch with only symbolic resource bindings.
pub(super) struct DispatchIntent {
    phase: RasterPhase,
    kernel: RasterKernel,
    workgroups: vello_encoding::WorkgroupSize,
    bindings: Bindings,
    indirect: Option<IndirectDispatch>,
}

pub(super) struct IndirectDispatch {
    buffer: BufferHandle,
    offset: u64,
}

/// A typed resource request, not a live allocation.
pub(super) enum ResourceIntent {
    Buffer(BufferIntent),
    Image(ImageIntent),
}

pub(super) struct BufferIntent {
    resource: BufferHandle,
    role: BufferRole,
    byte_len: u64,
}

pub(super) struct ImageIntent {
    resource: ImageHandle,
    role: ImageRole,
    extent: PhysicalSize,
    format: RasterImageFormat,
    retention: ImageRetention,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum ImageRetention {
    Transient,
    PersistentImageAtlas,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum BufferRole {
    ImageUpload,
    Scene,
    Config,
    InfoBinData,
    Tile,
    Segments,
    PerTileCommandList,
    PathReduced,
    PathReduced2,
    PathReducedScan,
    PathMonoids,
    PathBboxes,
    Bump,
    Lines,
    DrawReduced,
    DrawMonoids,
    ClipInputs,
    ClipElements,
    ClipBics,
    ClipBboxes,
    DrawBboxes,
    BinHeaders,
    Paths,
    IndirectCount,
    SegmentCounts,
    BlendSpill,
    MaskLut,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum ImageRole {
    GradientRamp,
    ImageAtlas,
}

/// A runtime-resource-free sequence of uploads, compute dispatches, and symbolic releases.
pub(super) struct Recording {
    commands: AccountedVec<RasterCommand>,
    _payloads: PayloadCustody,
}

pub(super) enum RasterCommand {
    UploadScene {
        buffer: BufferHandle,
        packed: AccountedVec<u8>,
    },
    UploadConfig {
        buffer: BufferHandle,
        config: ConfigUniform,
    },
    UploadGradientRamps {
        image: ImageHandle,
        ramps: AccountedVec<u32>,
    },
    UploadMaskLut {
        buffer: BufferHandle,
        variant: FineRasterVariant,
        samples: AccountedVec<u8>,
    },
    WriteImage {
        image: ImageHandle,
        origin: [u32; 2],
        image_data: ImageData,
    },
    UploadImageSource {
        buffer: BufferHandle,
        header: [u32; 8],
        image_data: ImageData,
    },
    ClearBuffer(BufferHandle),
    Dispatch(DispatchIntent),
    Release(ResourceReference),
}

/// Builder-only state that turns symbolic requests into a completed recording.
pub(super) struct RecordingBuilder {
    next_resource_id: u64,
    commands: AccountedVec<RasterCommand>,
    resource_intents: AccountedVec<ResourceIntent>,
    account: Account,
    payloads: PayloadCustody,
}

impl RecordingBuilder {
    pub(super) fn new(account: &Account) -> Result<Self> {
        Ok(Self {
            next_resource_id: 1,
            commands: account.retained(0)?,
            resource_intents: account.retained(0)?,
            account: account.clone(),
            payloads: PayloadCustody::default(),
        })
    }
    fn push(&mut self, command: RasterCommand) -> Result<()> {
        if self.commands.len() == self.commands.capacity() {
            self.commands.grow(self.commands.capacity().max(1))?;
        }
        self.account.work(RenderWork::RasterCommandsEmitted, 1)?;
        self.commands.push(command)
    }
    fn intent(&mut self, intent: ResourceIntent) -> Result<()> {
        if self.resource_intents.len() == self.resource_intents.capacity() {
            self.resource_intents
                .grow(self.resource_intents.capacity().max(1))?;
        }
        self.resource_intents.push(intent)
    }
    pub(super) fn account(&self) -> &Account {
        &self.account
    }

    pub(super) fn upload_scene(&mut self, packed: AccountedVec<u8>) -> Result<BufferHandle> {
        let buffer = self.allocate_buffer(BufferRole::Scene, packed.len() as u64)?;
        self.push(RasterCommand::UploadScene { buffer, packed })?;
        Ok(buffer)
    }

    pub(super) fn upload_config(&mut self, config: ConfigUniform) -> Result<BufferHandle> {
        let buffer = self.allocate_buffer(
            BufferRole::Config,
            std::mem::size_of::<ConfigUniform>() as u64,
        )?;
        self.push(RasterCommand::UploadConfig { buffer, config })?;
        Ok(buffer)
    }

    pub(super) fn upload_gradient_ramps(
        &mut self,
        extent: PhysicalSize,
        ramps: AccountedVec<u32>,
    ) -> Result<ImageHandle> {
        let image = self.allocate_image(
            ImageRole::GradientRamp,
            extent,
            RasterImageFormat::Rgba8Unorm,
            ImageRetention::Transient,
        )?;
        self.push(RasterCommand::UploadGradientRamps { image, ramps })?;
        Ok(image)
    }

    pub(super) fn new_transient_image(
        &mut self,
        role: ImageRole,
        extent: PhysicalSize,
    ) -> Result<ImageHandle> {
        self.allocate_image(
            role,
            extent,
            RasterImageFormat::Rgba8Unorm,
            ImageRetention::Transient,
        )
    }

    pub(super) fn request_image_atlas(&mut self, extent: PhysicalSize) -> Result<ImageHandle> {
        self.allocate_image(
            ImageRole::ImageAtlas,
            extent,
            RasterImageFormat::Rgba8Unorm,
            ImageRetention::PersistentImageAtlas,
        )
    }

    pub(super) fn write_image(
        &mut self,
        image: ImageHandle,
        origin: [u32; 2],
        image_data: ImageData,
    ) -> Result<()> {
        self.payloads
            .keep(self.account.import_image_data(&image_data)?, &self.account)?;
        self.push(RasterCommand::WriteImage {
            image,
            origin,
            image_data,
        })?;
        Ok(())
    }

    pub(super) fn expand_image(
        &mut self,
        atlas: ImageHandle,
        entry: super::image_atlas::AtlasEntry,
    ) -> Result<()> {
        self.payloads
            .keep(self.account.import_image_data(&entry.image)?, &self.account)?;
        let extent = entry.extent();
        let factors = entry.factors();
        let (source_size, first_pixel) = entry.window().map_or_else(
            || (PhysicalSize::new(entry.image.width, entry.image.height), 0),
            |window| (window.size(), window.first_pixel()),
        );
        let byte_len = u64::try_from(entry.image.data.data().len())
            .ok()
            .and_then(|length| length.checked_add(32))
            .ok_or_else(|| {
                Error::new(
                    BackendErrorCode::RenderFailed,
                    "image expansion upload size overflows",
                )
            })?;
        let buffer = self.allocate_buffer(BufferRole::ImageUpload, byte_len)?;
        self.push(RasterCommand::UploadImageSource {
            buffer,
            header: [
                source_size.width(),
                source_size.height(),
                entry.origin[0],
                entry.origin[1],
                factors[0],
                factors[1],
                entry.image.width,
                first_pixel,
            ],
            image_data: entry.image,
        })?;
        self.push(RasterCommand::Dispatch(DispatchIntent {
            phase: RasterPhase::ImagePreparation,
            kernel: RasterKernel::ExpandImage,
            workgroups: (extent.width().div_ceil(8), extent.height().div_ceil(8), 1),
            bindings: Bindings::Two([
                ResourceBinding::Buffer(buffer),
                ResourceBinding::Image(atlas),
            ]),
            indirect: None,
        }))?;
        self.release(buffer)?;
        Ok(())
    }

    pub(super) fn new_buffer(&mut self, role: BufferRole, byte_len: u64) -> Result<BufferHandle> {
        self.allocate_buffer(role, byte_len)
    }

    pub(super) fn clear_buffer(&mut self, buffer: BufferHandle) -> Result<()> {
        self.push(RasterCommand::ClearBuffer(buffer))?;
        Ok(())
    }

    pub(super) fn upload_mask_lut(
        &mut self,
        variant: FineRasterVariant,
        samples: AccountedVec<u8>,
    ) -> Result<BufferHandle> {
        let buffer = self.allocate_buffer(BufferRole::MaskLut, samples.len() as u64)?;
        self.push(RasterCommand::UploadMaskLut {
            buffer,
            variant,
            samples,
        })?;
        Ok(buffer)
    }

    pub(super) fn record_coarse(&mut self, dispatch: CoarseDispatch) -> Result<()> {
        self.push(RasterCommand::Dispatch(dispatch.into_dispatch_intent()))?;
        Ok(())
    }

    pub(super) fn record_fine(&mut self, dispatch: FineDispatch) -> Result<()> {
        self.push(RasterCommand::Dispatch(dispatch.into_dispatch_intent()))?;
        Ok(())
    }

    pub(super) fn release(&mut self, resource: impl Into<ResourceReference>) -> Result<()> {
        self.push(RasterCommand::Release(resource.into()))?;
        Ok(())
    }

    pub(super) fn finish(self) -> (Recording, AccountedVec<ResourceIntent>) {
        (
            Recording {
                commands: self.commands,
                _payloads: self.payloads,
            },
            self.resource_intents,
        )
    }

    fn allocate_buffer(&mut self, role: BufferRole, byte_len: u64) -> Result<BufferHandle> {
        if byte_len == 0 {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "Vello recording cannot request an empty buffer",
            ));
        }
        let buffer = BufferHandle(self.next_resource_id()?);
        self.intent(ResourceIntent::Buffer(BufferIntent {
            resource: buffer,
            role,
            byte_len,
        }))?;
        Ok(buffer)
    }

    fn allocate_image(
        &mut self,
        role: ImageRole,
        extent: PhysicalSize,
        format: RasterImageFormat,
        retention: ImageRetention,
    ) -> Result<ImageHandle> {
        if extent.width() == 0 || extent.height() == 0 {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "Vello recording cannot request an empty image",
            ));
        }
        let image = ImageHandle(self.next_resource_id()?);
        self.intent(ResourceIntent::Image(ImageIntent {
            resource: image,
            role,
            extent,
            format,
            retention,
        }))?;
        Ok(image)
    }

    fn next_resource_id(&mut self) -> Result<ResourceId> {
        let Some(next) = self.next_resource_id.checked_add(1) else {
            return Err(Error::new(
                BackendErrorCode::RenderFailed,
                "Vello recording exhausted its symbolic resource identities",
            ));
        };
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id = next;
        Ok(id)
    }
}

impl DispatchIntent {
    fn coarse_direct(
        kernel: RasterKernel,
        workgroups: vello_encoding::WorkgroupSize,
        bindings: Bindings,
    ) -> Self {
        Self {
            phase: RasterPhase::Coarse,
            kernel,
            workgroups,
            bindings,
            indirect: None,
        }
    }

    fn coarse_indirect(kernel: RasterKernel, buffer: BufferHandle, bindings: Bindings) -> Self {
        Self {
            phase: RasterPhase::Coarse,
            kernel,
            workgroups: (0, 0, 0),
            bindings,
            indirect: Some(IndirectDispatch { buffer, offset: 0 }),
        }
    }

    fn fine_direct(
        kernel: RasterKernel,
        workgroups: vello_encoding::WorkgroupSize,
        bindings: Bindings,
    ) -> Self {
        Self {
            phase: RasterPhase::Fine,
            kernel,
            workgroups,
            bindings,
            indirect: None,
        }
    }
}

#[cfg(test)]
impl Recording {
    pub(super) fn is_self_consistent_for_test(&self, intents: &[ResourceIntent]) -> bool {
        let Some((known, persistent)) = validated_resource_ids_for_test(intents) else {
            return false;
        };
        commands_are_self_consistent_for_test(&self.commands, &known, &persistent)
    }

    pub(super) fn final_dispatch_targets_output_for_test(&self) -> bool {
        self.commands
            .iter()
            .rev()
            .find_map(|command| match command {
                RasterCommand::Dispatch(dispatch) => Some(
                    dispatch
                        .bindings
                        .iter()
                        .any(|binding| matches!(binding, ResourceBinding::TargetOutput)),
                ),
                _ => None,
            })
            == Some(true)
    }
}

#[cfg(test)]
fn validated_resource_ids_for_test(
    intents: &[ResourceIntent],
) -> Option<(Vec<ResourceId>, Vec<ResourceId>)> {
    let mut known = Vec::with_capacity(intents.len());
    let mut persistent = Vec::new();
    for intent in intents {
        let (resource, is_persistent) = match intent {
            ResourceIntent::Buffer(BufferIntent {
                resource,
                role,
                byte_len,
                ..
            }) => {
                (*byte_len != 0).then_some(())?;
                let _ = role;
                (buffer_id(*resource), false)
            }
            ResourceIntent::Image(ImageIntent {
                resource,
                role,
                extent,
                format,
                retention,
            }) => {
                (extent.width() != 0
                    && extent.height() != 0
                    && matches!(format, RasterImageFormat::Rgba8Unorm))
                .then_some(())?;
                let _ = role;
                (
                    image_id(*resource),
                    matches!(retention, ImageRetention::PersistentImageAtlas),
                )
            }
        };
        (!known.contains(&resource)).then_some(())?;
        known.push(resource);
        if is_persistent {
            persistent.push(resource);
        }
    }
    Some((known, persistent))
}

#[cfg(test)]
fn commands_are_self_consistent_for_test(
    commands: &[RasterCommand],
    known: &[ResourceId],
    persistent: &[ResourceId],
) -> bool {
    let mut released = Vec::new();
    for command in commands {
        let valid = match command {
            RasterCommand::UploadScene { buffer, packed } => {
                !packed.is_empty() && is_active(buffer_id(*buffer), known, &released)
            }
            RasterCommand::UploadConfig { buffer, config } => {
                config.target_width != 0
                    && config.target_height != 0
                    && is_active(buffer_id(*buffer), known, &released)
            }
            RasterCommand::UploadGradientRamps { image, ramps } => {
                !ramps.is_empty() && is_active(image_id(*image), known, &released)
            }
            RasterCommand::UploadMaskLut {
                buffer,
                variant,
                samples,
            } => {
                !matches!(variant, FineRasterVariant::Area)
                    && !samples.is_empty()
                    && is_active(buffer_id(*buffer), known, &released)
            }
            RasterCommand::WriteImage {
                image,
                origin,
                image_data,
            } => {
                let _ = (origin, image_data);
                is_active(image_id(*image), known, &released)
            }
            RasterCommand::UploadImageSource { buffer, .. } => {
                is_active(buffer_id(*buffer), known, &released)
            }
            RasterCommand::ClearBuffer(buffer) => is_active(buffer_id(*buffer), known, &released),
            RasterCommand::Dispatch(dispatch) => {
                dispatch_is_well_formed(dispatch, known, &released)
            }
            RasterCommand::Release(resource) => {
                let id = resource_reference_id(*resource);
                if is_active(id, known, &released) && !persistent.contains(&id) {
                    released.push(id);
                    true
                } else {
                    false
                }
            }
        };
        if !valid {
            return false;
        }
    }
    released.len() + persistent.len() == known.len()
}

#[cfg(test)]
impl ResourceIntent {
    pub(super) const fn is_persistent_image_atlas_for_test(&self) -> bool {
        matches!(
            self,
            Self::Image(ImageIntent {
                role: ImageRole::ImageAtlas,
                retention: ImageRetention::PersistentImageAtlas,
                ..
            })
        )
    }

    pub(super) const fn is_transient_buffer_for_test(&self) -> bool {
        matches!(self, Self::Buffer(_))
    }
}

#[cfg(test)]
fn dispatch_is_well_formed(
    dispatch: &DispatchIntent,
    known: &[ResourceId],
    released: &[ResourceId],
) -> bool {
    let is_fine_kernel = matches!(
        dispatch.kernel,
        RasterKernel::FineArea | RasterKernel::FineMsaa8 | RasterKernel::FineMsaa16
    );
    if (is_fine_kernel && dispatch.phase != RasterPhase::Fine)
        || (dispatch.kernel == RasterKernel::ExpandImage
            && dispatch.phase != RasterPhase::ImagePreparation)
        || (!is_fine_kernel
            && dispatch.kernel != RasterKernel::ExpandImage
            && dispatch.phase != RasterPhase::Coarse)
        || dispatch.bindings.is_empty()
    {
        return false;
    }
    if let Some(IndirectDispatch { buffer, offset }) = dispatch.indirect {
        if dispatch.workgroups != (0, 0, 0)
            || !is_active(buffer_id(buffer), known, released)
            || offset % size_of::<u32>() as u64 != 0
        {
            return false;
        }
    } else if dispatch.workgroups == (0, 0, 0) {
        return false;
    }
    dispatch.bindings.iter().all(|binding| match binding {
        ResourceBinding::Buffer(buffer) => is_active(buffer_id(*buffer), known, released),
        ResourceBinding::Image(image) => is_active(image_id(*image), known, released),
        ResourceBinding::TargetOutput => true,
    })
}

#[cfg(test)]
fn is_active(resource: ResourceId, known: &[ResourceId], released: &[ResourceId]) -> bool {
    known.contains(&resource) && !released.contains(&resource)
}

#[cfg(test)]
const fn resource_reference_id(reference: ResourceReference) -> ResourceId {
    match reference {
        ResourceReference::Buffer(buffer) => buffer_id(buffer),
        ResourceReference::Image(image) => image_id(image),
    }
}

#[cfg(test)]
const fn buffer_id(buffer: BufferHandle) -> ResourceId {
    buffer.0
}

#[cfg(test)]
const fn image_id(image: ImageHandle) -> ResourceId {
    image.0
}

#[cfg(test)]
mod binding_arity_tests {
    use super::*;
    use ResourceBinding::{Buffer, Image, TargetOutput};
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    use std::mem::align_of;

    const fn buffer(id: u64) -> BufferHandle {
        BufferHandle(ResourceId(id))
    }

    #[test]
    fn fine_dispatches_preserve_literal_bindings() {
        let inputs = || FineDispatchBindings {
            workgroups: (3, 5, 1),
            config: buffer(11),
            segments: buffer(23),
            per_tile_command_list: buffer(37),
            info_bin_data: buffer(41),
            blend_spill: buffer(53),
            gradient_image: ImageHandle(ResourceId(61)),
            image_atlas: ImageHandle(ResourceId(79)),
        };
        let area = FineDispatch::area(inputs()).into_dispatch_intent();
        let msaa8 = FineDispatch::msaa8(inputs(), buffer(83)).into_dispatch_intent();
        let msaa16 = FineDispatch::msaa16(inputs(), buffer(97)).into_dispatch_intent();
        for (dispatch, kernel, expected) in [
            (
                area,
                RasterKernel::FineArea,
                [
                    Buffer(buffer(11)),
                    Buffer(buffer(23)),
                    Buffer(buffer(37)),
                    Buffer(buffer(41)),
                    Buffer(buffer(53)),
                    TargetOutput,
                    Image(ImageHandle(ResourceId(61))),
                    Image(ImageHandle(ResourceId(79))),
                ]
                .as_slice(),
            ),
            (
                msaa8,
                RasterKernel::FineMsaa8,
                [
                    Buffer(buffer(11)),
                    Buffer(buffer(23)),
                    Buffer(buffer(37)),
                    Buffer(buffer(41)),
                    Buffer(buffer(53)),
                    TargetOutput,
                    Image(ImageHandle(ResourceId(61))),
                    Image(ImageHandle(ResourceId(79))),
                    Buffer(buffer(83)),
                ]
                .as_slice(),
            ),
            (
                msaa16,
                RasterKernel::FineMsaa16,
                [
                    Buffer(buffer(11)),
                    Buffer(buffer(23)),
                    Buffer(buffer(37)),
                    Buffer(buffer(41)),
                    Buffer(buffer(53)),
                    TargetOutput,
                    Image(ImageHandle(ResourceId(61))),
                    Image(ImageHandle(ResourceId(79))),
                    Buffer(buffer(97)),
                ]
                .as_slice(),
            ),
        ] {
            assert_eq!(dispatch.kernel, kernel);
            assert_eq!(dispatch.phase, RasterPhase::Fine);
            assert_eq!(dispatch.workgroups, (3, 5, 1));
            assert!(dispatch.indirect.is_none());
            assert_eq!(&*dispatch.bindings, expected);
        }
    }

    #[test]
    fn coarse_dispatches_preserve_literal_bindings_and_indirect_count() {
        let coarse = CoarseDispatch::coarse(
            (2, 4, 1),
            CoarseRasterBindings {
                config: buffer(11),
                scene: buffer(23),
                draw_monoids: buffer(37),
                bin_headers: buffer(41),
                info_bin_data: buffer(53),
                paths: buffer(61),
                tile: buffer(79),
                bump: buffer(83),
                per_tile_command_list: buffer(97),
            },
        )
        .into_dispatch_intent();
        let reduce = CoarseDispatch::path_tag_reduce2((7, 1, 1), buffer(101), buffer(113))
            .into_dispatch_intent();
        let tiling = CoarseDispatch::path_tiling(
            buffer(127),
            buffer(83),
            buffer(131),
            buffer(149),
            buffer(61),
            buffer(79),
            buffer(157),
        )
        .into_dispatch_intent();
        for (dispatch, kernel, workgroups, indirect, expected) in [
            (
                coarse,
                RasterKernel::Coarse,
                (2, 4, 1),
                None,
                [11, 23, 37, 41, 53, 61, 79, 83, 97].as_slice(),
            ),
            (
                reduce,
                RasterKernel::PathTagReduce2,
                (7, 1, 1),
                None,
                [101, 113].as_slice(),
            ),
            (
                tiling,
                RasterKernel::PathTiling,
                (0, 0, 0),
                Some((buffer(127), 0)),
                [83, 131, 149, 61, 79, 157].as_slice(),
            ),
        ] {
            assert_eq!(dispatch.kernel, kernel);
            assert_eq!(dispatch.phase, RasterPhase::Coarse);
            assert_eq!(dispatch.workgroups, workgroups);
            assert_eq!(
                dispatch.indirect.map(|count| (count.buffer, count.offset)),
                indirect,
            );
            assert!(
                dispatch
                    .bindings
                    .iter()
                    .copied()
                    .eq(expected.iter().map(|&id| Buffer(buffer(id))))
            );
        }
    }

    #[test]
    fn recording_keeps_inline_dispatch_footprint_and_charged_capacity() {
        use crate::{RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderResource};

        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        {
            assert_eq!((size_of::<Bindings>(), align_of::<Bindings>()), (152, 8));
            assert_eq!(
                (size_of::<DispatchIntent>(), align_of::<DispatchIntent>()),
                (192, 8)
            );
            assert_eq!(
                (size_of::<RasterCommand>(), align_of::<RasterCommand>()),
                (192, 8)
            );
        }

        let mut context = RenderAccountingContext::default();
        context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                let mut recording = RecordingBuilder::new(account)?;
                let input = recording.new_buffer(BufferRole::PathReduced, 16)?;
                let output = recording.new_buffer(BufferRole::PathReduced2, 16)?;
                recording.record_coarse(CoarseDispatch::path_tag_reduce2(
                    (7, 1, 1),
                    input,
                    output,
                ))?;
                recording.release(input)?;
                recording.release(output)?;
                let (recording, intents) = recording.finish();
                assert!(recording.is_self_consistent_for_test(&intents));
                assert_eq!(recording.commands.len(), 3);
                assert_eq!(recording.commands.capacity(), 4);
                assert_eq!(intents.capacity(), 2);
                Ok(())
            })
            .unwrap();
        let report = context.last_attempt().unwrap();
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::RasterCommandsEmitted), 3);
        assert_eq!(usage.work(RenderWork::ShaderBindingVisits), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(
            usage.peak(RenderResource::RetainedCpuBytes),
            (6 * size_of::<RasterCommand>() + 2 * size_of::<ResourceIntent>()) as u64,
        );
        assert_eq!(report.cleanup_fault(), None);
    }
}

//! Valid image sources and per-draw integer expansion are resolved before Vello packing.

use super::bounded::{self, EncodingBacking, OwnedEncoding, RasterRegistry};
use crate::accounting::{Account, AccountedVec, RenderPhase, RenderResource, RenderWork};
use crate::image::ImageWindow;
use peniko::ImageData;
use vello_encoding::{DrawImage, Encoding, Patch};

use crate::{BackendErrorCode, Error, PhysicalSize, Result};

const PACKED_MAX: u32 = u16::MAX as u32;
pub(super) const MAX_TEXELS: u64 = PACKED_MAX as u64 * PACKED_MAX as u64;
/// A validated nearest-neighbor expansion of the image's original pixel grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ImageExpansion {
    factors: [u32; 2],
    extent: PhysicalSize,
    window: Option<ImageWindow>,
}

impl ImageExpansion {
    pub(super) fn try_new(
        image: &ImageData,
        source_to_device: kurbo::Affine,
        account: &Account,
    ) -> Result<Self> {
        Self::from_factors(
            image,
            crate::image::pixelated_multipliers(source_to_device, account)?,
        )
    }

    fn from_factors(image: &ImageData, factors: [u32; 2]) -> Result<Self> {
        Self::from_source(PhysicalSize::new(image.width, image.height), factors, None)
    }

    pub(super) fn for_region(
        window: ImageWindow,
        pixelated: bool,
        source_to_device: kurbo::Affine,
        account: &Account,
    ) -> Result<Self> {
        let factors = if pixelated {
            crate::image::pixelated_multipliers(source_to_device, account)?
        } else {
            [1, 1]
        };
        Self::from_source(window.size(), factors, Some(window))
    }

    fn from_source(
        source: PhysicalSize,
        factors: [u32; 2],
        window: Option<ImageWindow>,
    ) -> Result<Self> {
        let width = source.width().checked_mul(factors[0]);
        let height = source.height().checked_mul(factors[1]);
        match (width, height) {
            (Some(width), Some(height))
                if width != 0
                    && height != 0
                    && width <= PACKED_MAX
                    && height <= PACKED_MAX
                    && factors[0] != 0
                    && factors[1] != 0 =>
            {
                Ok(Self {
                    factors,
                    extent: PhysicalSize::new(width, height),
                    window,
                })
            }
            _ => Err(failed(
                "image extent cannot be represented in the Vello image atlas",
            )),
        }
    }

    pub(super) fn compensation(self) -> kurbo::Affine {
        kurbo::Affine::scale_non_uniform(
            1.0 / f64::from(self.factors[0]),
            1.0 / f64::from(self.factors[1]),
        )
    }
}

pub(super) struct AtlasEntry {
    pub(super) image: ImageData,
    pub(super) origin: [u32; 2],
    pub(super) expansion: ImageExpansion,
}

impl AtlasEntry {
    pub(super) fn factors(&self) -> [u32; 2] {
        self.expansion.factors
    }
    pub(super) fn extent(&self) -> PhysicalSize {
        self.expansion.extent
    }

    pub(super) fn window(&self) -> Option<ImageWindow> {
        self.expansion.window
    }
}

pub(super) type ImageExpansions = RasterRegistry<usize, ImageExpansion>;

pub(super) struct ImageAtlasPlan {
    pub(super) encoding: OwnedEncoding,
    pub(super) extent: PhysicalSize,
    pub(super) entries: AccountedVec<AtlasEntry>,
}

impl ImageAtlasPlan {
    pub(super) fn prepare(
        encoding: &Encoding,
        expansions: &ImageExpansions,
        account: &Account,
    ) -> Result<Self> {
        let account = account.at(RenderPhase::Atlas);
        account.phase(RenderPhase::Atlas, || {
            let mut owned = EncodingBacking::copy(encoding, &account)?;
            let (encoding, _backing) = owned.parts();
            let mut entries = account.retained(0)?;
            let mut identities = RasterRegistry::temporary();
            let mut draws = account.temporary(0)?;
            let quota = account.local_quota(RenderPhase::Atlas, [MAX_TEXELS * 4, MAX_TEXELS])?;
            for patch in &encoding.resources.patches {
                account.work(RenderWork::ResourceProbes, 1)?;
                let Patch::Image {
                    image,
                    draw_data_offset,
                } = patch
                else {
                    continue;
                };
                // Empty image brushes are transparent. Keep their draw records, but
                // allocate no region and never issue a zero-sized texture upload.
                if image.width == 0 || image.height == 0 {
                    bounded::push_record(&mut draws, (*draw_data_offset, None), &account)?;
                    continue;
                }
                let expansion = match expansions.get(draw_data_offset, &account)? {
                    Some(expansion) => *expansion,
                    None => ImageExpansion::from_factors(image, [1, 1])?,
                };
                let bytes = (u64::from(image.width) * u64::from(image.height))
                    .checked_mul(4)
                    .ok_or_else(|| failed("image atlas requires complete RGBA8 source data"))?;
                if image.format != peniko::ImageFormat::Rgba8
                    || u64::try_from(image.data.data().len()).ok() != Some(bytes)
                {
                    return Err(failed("image atlas requires complete RGBA8 source data"));
                }
                if let Some(window) = expansion.window
                    && window.source_size() != PhysicalSize::new(image.width, image.height)
                {
                    return Err(failed(
                        "image atlas region must match its original source axes",
                    ));
                }
                // Blob identity identifies bytes, while dimensions identify their
                // two-dimensional interpretation. Sampling/alpha flags stay per draw.
                let identity = (
                    image.data.id(),
                    image.width,
                    image.height,
                    expansion.factors,
                    expansion.window.map(|window| {
                        (
                            window.first_pixel(),
                            window.size().width(),
                            window.size().height(),
                        )
                    }),
                );
                let index = if let Some(index) = identities.get(&identity, &account)? {
                    *index
                } else {
                    let texels =
                        u64::from(expansion.extent.width()) * u64::from(expansion.extent.height());
                    quota.acquire(RenderResource::LocalAtlasSourceByteQuota, bytes)?;
                    quota.acquire(RenderResource::LocalAtlasTexelQuota, texels)?;
                    account.work_u64(RenderWork::ImageSourceBytes, bytes)?;
                    account.work_u64(RenderWork::AtlasTexels, texels)?;
                    let index = entries.len();
                    bounded::reserve_record(&mut entries)?;
                    let slot = entries.append_slot()?;
                    account.work(RenderWork::RasterResourceCopies, 1)?;
                    slot.write(AtlasEntry {
                        image: image.clone(),
                        origin: [0, 0],
                        expansion,
                    });
                    identities.insert(identity, index, &account)?;
                    index
                };
                bounded::push_record(&mut draws, (*draw_data_offset, Some(index)), &account)?;
            }
            let area = quota.live(RenderResource::LocalAtlasTexelQuota)?;
            let extent = pack_entries(&mut entries, area, &account)?;
            for &(offset, index) in &draws {
                account.work(RenderWork::ResourceProbes, 1)?;
                let words = encoding
                    .draw_data
                    .get_mut(
                        offset
                            ..offset
                                .checked_add(3)
                                .ok_or_else(|| failed("image draw offset overflow"))?,
                    )
                    .ok_or_else(|| {
                        failed("image patch does not identify a complete draw record")
                    })?;
                let mut draw: DrawImage = bytemuck::pod_read_unaligned(bytemuck::cast_slice(words));
                if let Some(index) = index {
                    let entry = &entries[index];
                    draw.xy = (entry.origin[0] << 16) | entry.origin[1];
                    draw.width_height = (entry.extent().width() << 16) | entry.extent().height();
                } else {
                    draw.xy = 0;
                    draw.width_height = 0;
                }
                words.copy_from_slice(bytemuck::cast_slice(bytemuck::bytes_of(&draw)));
            }
            // Qualify the exact nonallocating retain move before invoking it.
            // The count is a pinned operation-size proof, not a usage counter.
            let (_, moves) = encoding.resources.patches.iter().try_fold(
                (false, 0usize),
                |(removed, moves), patch| -> Result<(bool, usize)> {
                    account.work(RenderWork::ResourceProbes, 1)?;
                    if matches!(patch, Patch::Image { .. }) {
                        Ok((true, moves))
                    } else if removed {
                        Ok((true, bounded::add(moves, 1, &account)?))
                    } else {
                        Ok((false, moves))
                    }
                },
            )?;
            account.preview_work([(
                RenderWork::EncodingStreamCopies,
                crate::accounting::as_u64(moves)?,
            )])?;
            let mut removed = false;
            let mut refusal = None;
            encoding.resources.patches.retain(|patch| {
                // Stop inspecting the private copy after refusal. Discarding
                // its suffix avoids retain moving any unadmitted survivors.
                if refusal.is_some() {
                    return false;
                }
                if let Err(error) = account.work(RenderWork::ResourceProbes, 1) {
                    refusal = Some(error);
                    return false;
                }
                if matches!(patch, Patch::Image { .. }) {
                    removed = true;
                    return false;
                }
                if removed && let Err(error) = account.work(RenderWork::EncodingStreamCopies, 1) {
                    refusal = Some(error);
                    return false;
                }
                true
            });
            if let Some(error) = refusal {
                return Err(error);
            }
            Ok(Self {
                encoding: owned,
                extent,
                entries,
            })
        })
    }
}

fn pack_entries(entries: &mut [AtlasEntry], area: u64, account: &Account) -> Result<PhysicalSize> {
    let mut order = account.temporary(entries.len())?;
    for index in 0..entries.len() {
        account.work(RenderWork::ResourceProbes, 1)?;
        bounded::push_record(&mut order, index, account)?;
    }
    sort_order(&mut order, entries, account)?;
    let mut largest_dimension = 1;
    for entry in entries.iter() {
        account.work(RenderWork::ResourceProbes, 1)?;
        largest_dimension =
            largest_dimension.max(entry.extent().width().max(entry.extent().height()));
    }
    // Match the power-of-two square allocation classes used by the Vello
    // atlas. A narrow successful shelf must not become an unnecessarily tall
    // texture that exceeds a device limit despite fitting the next square.
    let mut width = largest_dimension
        .max((area as f64).sqrt().ceil() as u32)
        .max(1)
        .next_power_of_two()
        .min(PACKED_MAX);
    // Every extent is at most PACKED_MAX and an overflowing shelf stops
    // immediately. These coordinate sums are therefore at most 2*PACKED_MAX,
    // well within u32; aggregate texel arithmetic belongs to the ledger above.
    loop {
        let (mut x, mut y, mut row_height, mut used_width) = (0, 0, 0, 1);
        let mut fits = true;
        for &index in &order {
            account.work(RenderWork::ResourceProbes, 1)?;
            let entry = &mut entries[index];
            let extent = entry.extent();
            if x + extent.width() > width {
                x = 0;
                y += row_height;
                row_height = 0;
            }
            if y + extent.height() > width {
                fits = false;
                break;
            }
            entry.origin = [x, y];
            x += extent.width();
            used_width = used_width.max(x);
            row_height = row_height.max(extent.height());
        }
        if fits {
            return Ok(PhysicalSize::new(used_width, (y + row_height).max(1)));
        }
        if width == PACKED_MAX {
            return Err(failed(
                "images do not fit the representable Vello atlas extent",
            ));
        }
        width = width.saturating_mul(2).min(PACKED_MAX);
    }
}

/// Fallible heap sort keeps all comparisons at the owner loop and needs no
/// hidden stable-sort scratch. The index tie break preserves prior stable order.
fn sort_order(order: &mut [usize], entries: &[AtlasEntry], account: &Account) -> Result<()> {
    fn compare(
        a: usize,
        b: usize,
        entries: &[AtlasEntry],
        account: &Account,
    ) -> Result<std::cmp::Ordering> {
        account.work(RenderWork::ResourceProbes, 1)?;
        let ah = entries[a].extent().height();
        account.work(RenderWork::ResourceProbes, 1)?;
        let bh = entries[b].extent().height();
        Ok(bh.cmp(&ah).then_with(|| a.cmp(&b)))
    }
    fn sift(
        order: &mut [usize],
        mut root: usize,
        end: usize,
        entries: &[AtlasEntry],
        account: &Account,
    ) -> Result<()> {
        while let Some(child) = root
            .checked_mul(2)
            .and_then(|v| v.checked_add(1))
            .filter(|child| *child < end)
        {
            let mut next = child;
            if child + 1 < end && compare(order[child], order[child + 1], entries, account)?.is_lt()
            {
                next = child + 1;
            }
            if !compare(order[root], order[next], entries, account)?.is_lt() {
                break;
            }
            account.work(RenderWork::RasterResourceCopies, 2)?;
            order.swap(root, next);
            root = next;
        }
        Ok(())
    }
    for root in (0..order.len() / 2).rev() {
        sift(order, root, order.len(), entries, account)?;
    }
    for end in (1..order.len()).rev() {
        account.work(RenderWork::RasterResourceCopies, 2)?;
        order.swap(0, end);
        sift(order, 0, end, entries, account)?;
    }
    Ok(())
}

fn failed(message: impl Into<String>) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{
        FrameFixture, RenderAccountingOptions, RenderAttemptStatus, RenderCpuScope,
        RenderFrameReport, RenderLimits, RendererCpu,
    };

    fn measure_atlas(
        encoding: &Encoding,
        expansions: &ImageExpansions,
        limits: RenderLimits,
    ) -> (Result<ImageAtlasPlan>, RenderFrameReport) {
        let options = RenderAccountingOptions::new().with_limits(limits);
        let cpu = RendererCpu::new(options).unwrap();
        let (result, _, report, fault) = FrameFixture::new(&cpu, options)
            .run_sync(|attempt| ImageAtlasPlan::prepare(encoding, expansions, attempt.account()))
            .unwrap()
            .into_parts();
        assert_eq!(fault, None);
        (result, report)
    }

    fn empty_image_encoding() -> Encoding {
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 3.0), Vec::<u8>::new()).unwrap();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image.data, 1.0);
        encoding
    }

    fn assert_work_refusal(
        report: &RenderFrameReport,
        work: RenderWork,
        current: u64,
        amount: u64,
    ) {
        let attempt = report.attempt().unwrap();
        assert_eq!(attempt.status(), RenderAttemptStatus::Refused);
        let refusal = attempt.refusal().unwrap();
        assert_eq!(refusal.quantity, crate::RenderQuantity::Work(work));
        assert_eq!((refusal.current, refusal.amount), (current, amount));
        let cpu = report.renderer_cpu().unwrap();
        for resource in [
            RenderResource::RetainedCpuBytes,
            RenderResource::ScratchCpuBytes,
        ] {
            assert_eq!(
                cpu.scope_usage(RenderCpuScope::FrameBacking).live(resource),
                0
            );
        }
        assert_eq!(attempt.usage().work(RenderWork::UploadBytes), 0);
        assert_eq!(
            attempt.usage().work(RenderWork::ResourceAllocationRequests),
            0
        );
    }

    #[test]
    fn atlas_empty_image_admits_its_final_retain_classification() {
        let encoding = empty_image_encoding();
        assert_eq!(
            (
                encoding.draw_tags.len(),
                encoding.draw_data.len(),
                encoding.resources.patches.len()
            ),
            (1, 3, 1)
        );
        let original = encoding.clone();
        for limit in [4, 3] {
            let (result, report) = measure_atlas(
                &encoding,
                &ImageExpansions::default(),
                RenderLimits::new()
                    .with_work_limit(RenderWork::ResourceProbes, limit)
                    .unwrap(),
            );
            let usage = report.attempt().unwrap().usage();
            if limit == 4 {
                let plan = result.unwrap();
                assert!(plan.entries.is_empty());
                assert!(plan.encoding.resources.patches.is_empty());
                assert_eq!(usage.work(RenderWork::ResourceProbes), 4);
            } else {
                assert!(
                    result.is_err(),
                    "the retain predicate must refuse after its three predecessors"
                );
                assert_work_refusal(&report, RenderWork::ResourceProbes, 3, 1);
            }
            // One draw tag, three words and one patch were actually copied.
            assert_eq!(usage.work(RenderWork::EncodingStreamCopies), 5);
            assert_eq!(encoding.draw_data, original.draw_data);
            assert!(encoding.draw_tags == original.draw_tags);
            assert_eq!(encoding.resources.patches.len(), 1);
        }
    }

    #[test]
    fn atlas_nonempty_and_expanded_images_count_the_independent_retain_visit() {
        let source = image();
        let mut encoding = Encoding::new();
        encoding.encode_image(&source, 1.0);
        for explicit in [false, true] {
            let expansions = if explicit {
                ImageExpansions::from_records([(
                    0,
                    ImageExpansion::from_factors(&source, [2, 3]).unwrap(),
                )])
            } else {
                ImageExpansions::default()
            };
            let exact = if explicit { 8 } else { 7 };
            for limit in [exact, exact - 1] {
                let (result, report) = measure_atlas(
                    &encoding,
                    &expansions,
                    RenderLimits::new()
                        .with_work_limit(RenderWork::ResourceProbes, limit)
                        .unwrap(),
                );
                if limit == exact {
                    let plan = result.unwrap();
                    assert_eq!(plan.entries.len(), 1);
                    assert!(plan.encoding.resources.patches.is_empty());
                    assert_eq!(
                        report
                            .attempt()
                            .unwrap()
                            .usage()
                            .work(RenderWork::ResourceProbes),
                        exact
                    );
                } else {
                    assert!(result.is_err());
                    assert_work_refusal(&report, RenderWork::ResourceProbes, exact - 1, 1);
                }
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::EncodingStreamCopies),
                    5
                );
                assert_eq!(encoding.resources.patches.len(), 1);
            }
        }
    }

    fn mixed_patch_encoding() -> Encoding {
        let mut encoding = Encoding::new();
        for index in 0..3 {
            encoding.encode_linear_gradient(
                vello_encoding::DrawLinearGradient {
                    index: 0,
                    p0: [index as f32, 0.0],
                    p1: [index as f32 + 1.0, 0.0],
                },
                [
                    peniko::ColorStop::from((0.0, peniko::color::palette::css::BLACK)),
                    peniko::ColorStop::from((1.0, peniko::color::palette::css::WHITE)),
                ]
                .into_iter(),
                1.0,
                peniko::Extend::Pad,
            );
            if index == 0 {
                let image =
                    crate::Image::from_rgba(crate::Size::new(0.0, 3.0), Vec::<u8>::new()).unwrap();
                encoding.encode_image(&image.data, 1.0);
            }
        }
        encoding
    }

    fn ramp_offsets(encoding: &Encoding) -> Vec<usize> {
        encoding
            .resources
            .patches
            .iter()
            .filter_map(|patch| match patch {
                Patch::Ramp {
                    draw_data_offset, ..
                } => Some(*draw_data_offset),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn atlas_compaction_preserves_survivors_and_counts_each_actual_shift() {
        let encoding = mixed_patch_encoding();
        let original = encoding.clone();
        assert_eq!(ramp_offsets(&encoding), [0, 8, 13]);
        let (result, report) =
            measure_atlas(&encoding, &ImageExpansions::default(), RenderLimits::new());
        let plan = result.unwrap();
        assert_eq!(ramp_offsets(&plan.encoding), [0, 8, 13]);
        assert_eq!(plan.encoding.resources.patches.len(), 3);
        assert_eq!(
            plan.encoding.resources.color_stops,
            encoding.resources.color_stops
        );
        // Three real gradients contribute9elements each; the image contributes5.
        // Main scan4 + draw rewrite1 + preflight4 + retain4 =13 probes;
        // only the two gradient patches after the removed image move.
        let usage = report.attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::ResourceProbes), 13);
        assert_eq!(usage.work(RenderWork::EncodingStreamCopies), 34);
        assert_eq!(encoding.draw_data, original.draw_data);
        assert!(encoding.draw_tags == original.draw_tags);
        assert_eq!(
            encoding.resources.color_stops,
            original.resources.color_stops
        );
        assert_eq!(encoding.resources.patches.len(), 4);
    }

    #[test]
    fn atlas_predicate_refusal_never_counts_an_unexecuted_survivor_shift() {
        let encoding = mixed_patch_encoding();
        let original = encoding.clone();
        for (probes, copies) in [(11, 32), (12, 33)] {
            let (result, report) = measure_atlas(
                &encoding,
                &ImageExpansions::default(),
                RenderLimits::new()
                    .with_work_limit(RenderWork::ResourceProbes, probes)
                    .unwrap(),
            );
            assert!(
                result.is_err(),
                "the next retained-source classification must refuse"
            );
            assert_work_refusal(&report, RenderWork::ResourceProbes, probes, 1);
            assert_eq!(
                report
                    .attempt()
                    .unwrap()
                    .usage()
                    .work(RenderWork::EncodingStreamCopies),
                copies
            );
            assert_eq!(encoding.draw_data, original.draw_data);
            assert_eq!(ramp_offsets(&encoding), [0, 8, 13]);
            assert_eq!(encoding.resources.patches.len(), 4);
        }
    }

    #[test]
    fn atlas_copy_preview_refuses_without_committing_survivor_work() {
        let encoding = mixed_patch_encoding();
        for limit in [33, 34] {
            let (result, report) = measure_atlas(
                &encoding,
                &ImageExpansions::default(),
                RenderLimits::new()
                    .with_work_limit(RenderWork::EncodingStreamCopies, limit)
                    .unwrap(),
            );
            if limit == 33 {
                assert!(result.is_err());
                assert_work_refusal(&report, RenderWork::EncodingStreamCopies, 32, 2);
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::EncodingStreamCopies),
                    32
                );
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::ResourceProbes),
                    9
                );
            } else {
                let plan = result.unwrap();
                assert_eq!(ramp_offsets(&plan.encoding), [0, 8, 13]);
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::EncodingStreamCopies),
                    34
                );
                assert_eq!(
                    report
                        .attempt()
                        .unwrap()
                        .usage()
                        .work(RenderWork::ResourceProbes),
                    13
                );
            }
        }
    }

    fn image() -> ImageData {
        crate::Image::from_rgba(
            crate::Size::new(2.0, 1.0),
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap()
        .data
    }

    #[test]
    fn nearest_positive_expansion_is_independent_per_transformed_source_axis() {
        let account = Account::for_test();
        let expansion = ImageExpansion::try_new(
            &image(),
            kurbo::Affine::new([0.0, 2.4, -3.6, 0.0, 9.0, 7.0]),
            &account,
        )
        .unwrap();
        assert_eq!(expansion.factors, [2, 4]);
        assert_eq!(expansion.extent, PhysicalSize::new(4, 4));
        let downscale = ImageExpansion::try_new(
            &image(),
            kurbo::Affine::scale_non_uniform(0.1, 0.4),
            &account,
        )
        .unwrap();
        assert_eq!(downscale.factors, [1, 1]);
    }

    #[test]
    fn oversized_expansion_is_rejected_without_truncation() {
        let account = Account::for_test();
        for scale in [32768.0, f64::INFINITY] {
            assert!(
                ImageExpansion::try_new(&image(), kurbo::Affine::scale(scale), &account).is_err()
            );
        }
    }

    #[test]
    fn distinct_valid_views_of_one_blob_keep_their_source_dimensions() {
        let original = image();
        let mut vertical = original.clone();
        vertical.width = 1;
        vertical.height = 2;
        let mut encoding = Encoding::new();
        encoding.encode_image(&original, 1.0);
        encoding.encode_image(&vertical, 1.0);
        let plan =
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.encoding.draw_data[1], (2 << 16) | 1);
        assert_eq!(plan.encoding.draw_data[4], (1 << 16) | 2);
    }

    #[test]
    fn atlas_refuses_an_expansion_from_different_source_axes() {
        let horizontal = image();
        let mut vertical = horizontal.clone();
        vertical.width = 1;
        vertical.height = 2;
        let mut encoding = Encoding::new();
        encoding.encode_image(&horizontal, 1.0);
        encoding.encode_image(&vertical, 1.0);
        let original_words = encoding.draw_data.clone();
        let mismatched = ImageExpansions::from_records([(
            3,
            ImageExpansion::from_factors(&horizontal, [2, 3]).unwrap(),
        )]);

        let (result, report) = measure_atlas(&encoding, &mismatched, RenderLimits::new());
        let error = match result {
            Ok(plan) => panic!(
                "atlas accepted 1×2 source with a 2×1 expansion: {:?}",
                plan.entries[1].extent()
            ),
            Err(error) => error,
        };
        assert_eq!(error.code(), crate::ErrorCode::RenderFailed);
        assert_eq!(
            error.message(),
            "image atlas expansion does not match source dimensions"
        );
        let attempt = report.attempt().unwrap();
        assert_eq!(
            attempt.status(),
            RenderAttemptStatus::Failed(crate::ErrorCode::RenderFailed)
        );
        let usage = attempt.phase_usage(RenderPhase::Atlas);
        assert_eq!(usage.work(RenderWork::ResourceProbes), 4);
        assert_eq!(usage.work(RenderWork::EncodingStreamCopies), 10);
        assert_eq!(usage.work(RenderWork::ImageSourceBytes), 8);
        assert_eq!(usage.work(RenderWork::AtlasTexels), 2);
        assert_eq!(usage.work(RenderWork::RasterResourceCopies), 3);
        assert_eq!(usage.work(RenderWork::UploadBytes), 0);
        let cpu = report.renderer_cpu().unwrap();
        for resource in [
            RenderResource::RetainedCpuBytes,
            RenderResource::ScratchCpuBytes,
        ] {
            assert_eq!(
                cpu.scope_usage(RenderCpuScope::FrameBacking).live(resource),
                0
            );
        }
        assert_eq!(encoding.draw_data, original_words);
        assert_eq!(encoding.resources.patches.len(), 2);

        let (result, report) = measure_atlas(
            &encoding,
            &mismatched,
            RenderLimits::new()
                .with_work_limit(RenderWork::ResourceProbes, 3)
                .unwrap(),
        );
        assert!(result.is_err());
        assert_work_refusal(&report, RenderWork::ResourceProbes, 3, 1);

        let matching = ImageExpansions::from_records([(
            3,
            ImageExpansion::from_factors(&vertical, [2, 3]).unwrap(),
        )]);
        let (retry, _) = measure_atlas(&encoding, &matching, RenderLimits::new());
        let plan = retry.unwrap();
        assert_eq!(plan.entries[0].extent(), PhysicalSize::new(2, 1));
        assert_eq!(plan.entries[1].extent(), PhysicalSize::new(2, 6));
        assert_eq!(plan.encoding.draw_data[1], (2 << 16) | 1);
        assert_eq!(plan.encoding.draw_data[4], (2 << 16) | 6);
    }

    #[test]
    fn malformed_or_unsupported_source_is_refused_before_recording_uploads() {
        let mut source = image();
        source.format = peniko::ImageFormat::Bgra8;
        let mut encoding = Encoding::new();
        encoding.encode_image(&source, 1.0);
        assert!(
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .is_err()
        );
        let mut source = image();
        source.width = 3;
        let mut encoding = Encoding::new();
        encoding.encode_image(&source, 1.0);
        assert!(
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .is_err()
        );
    }

    #[test]
    fn aggregate_expansion_admission_rejects_unrepresentable_atlas_work() {
        let image = image();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image, 1.0);
        encoding.encode_image(&image, 1.0);
        let first = ImageExpansion::from_factors(&image, [32767, 65535]).unwrap();
        let second = ImageExpansion::from_factors(&image, [32766, 65535]).unwrap();
        let error = ImageAtlasPlan::prepare(
            &encoding,
            &ImageExpansions::from_records([(0, first), (3, second)]),
            &Account::for_test(),
        )
        .err()
        .expect("the two expanded grids exceed the representable atlas area");
        let Some(crate::RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
        else {
            panic!("atlas area must preserve its typed local-quota refusal");
        };
        assert_eq!(refusal.phase(), RenderPhase::AtlasQuota);
        assert_eq!(
            refusal.quantity(),
            crate::RenderQuantity::Resource(RenderResource::LocalAtlasTexelQuota)
        );
        assert_eq!(encoding.resources.patches.len(), 2);
    }

    #[test]
    fn empty_image_brush_has_no_upload_and_resolves_to_transparent_dimensions() {
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 3.0), Vec::<u8>::new()).unwrap();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image.data, 1.0);
        let plan =
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .unwrap();
        assert!(plan.entries.is_empty());
        assert_eq!(plan.extent, PhysicalSize::new(1, 1));
        let mut resolver = vello_encoding::Resolver::default();
        let mut packed = Vec::new();
        let (layout, _, _) = resolver.resolve(&plan.encoding, &mut packed);
        assert_eq!(layout.draw_data(&packed)[1], 0);
    }

    #[test]
    fn four_large_rectangles_fit_the_supported_square_using_small_original_sources() {
        let source = image();
        let expansion = ImageExpansion::from_factors(&source, [2000, 2500]).unwrap();
        let mut entries = (0..4)
            .map(|_| AtlasEntry {
                image: source.clone(),
                origin: [0, 0],
                expansion,
            })
            .collect::<Vec<_>>();
        let extent = pack_entries(&mut entries, 4 * 4000 * 2500, &Account::for_test()).unwrap();
        assert_eq!(extent, PhysicalSize::new(8000, 5000));
        assert!(
            entries
                .iter()
                .all(|entry| entry.image.data.data().len() == 8)
        );
    }

    #[test]
    fn four_rectangles_fit_one_square_without_an_overlong_texture_axis() {
        let mut encoding = Encoding::new();
        for _ in 0..4 {
            let image = crate::Image::from_rgba(crate::Size::new(4.0, 3.0), vec![255; 48]).unwrap();
            encoding.encode_image(&image.data, 1.0);
        }
        let plan =
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .unwrap();
        assert_eq!(plan.extent, PhysicalSize::new(8, 6));
        assert_eq!(plan.entries.len(), 4);
    }

    #[test]
    fn atlas_regions_are_deterministic_nonoverlapping_and_contain_every_source() {
        let mut encoding = Encoding::new();
        for (width, height) in [(4_u32, 2_u32), (2, 4), (1, 1), (3, 2)] {
            let image = crate::Image::from_rgba(
                crate::Size::new(f64::from(width), f64::from(height)),
                vec![255; (width * height * 4) as usize],
            )
            .unwrap();
            encoding.encode_image(&image.data, 1.0);
        }
        let first =
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .unwrap();
        let second =
            ImageAtlasPlan::prepare(&encoding, &ImageExpansions::default(), &Account::for_test())
                .unwrap();
        assert_eq!(first.entries.len(), 4);
        assert_eq!(first.extent, second.extent);
        for (index, entry) in first.entries.iter().enumerate() {
            assert_eq!(entry.origin, second.entries[index].origin);
            let [x, y] = entry.origin;
            let extent = entry.extent();
            assert!(x + extent.width() <= first.extent.width());
            assert!(y + extent.height() <= first.extent.height());
            for other in &first.entries[index + 1..] {
                let [other_x, other_y] = other.origin;
                assert!(
                    x + extent.width() <= other_x
                        || other_x + other.extent().width() <= x
                        || y + extent.height() <= other_y
                        || other_y + other.extent().height() <= y
                );
            }
        }
    }

    #[test]
    fn separate_expansions_of_one_valid_source_have_separate_atlas_regions() {
        let image = image();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image, 1.0);
        encoding.encode_image(&image, 1.0);
        let expansion = ImageExpansion::from_factors(&image, [2, 3]).unwrap();
        let plan = ImageAtlasPlan::prepare(
            &encoding,
            &ImageExpansions::from_records([(3, expansion)]),
            &Account::for_test(),
        )
        .unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(
            plan.entries[0].image.data.data(),
            plan.entries[1].image.data.data()
        );
        assert_eq!(plan.entries[0].extent(), PhysicalSize::new(2, 1));
        assert_eq!(plan.entries[1].extent(), PhysicalSize::new(4, 3));
        assert!(plan.encoding.resources.patches.is_empty());
        assert_ne!(plan.encoding.draw_data[0], plan.encoding.draw_data[3]);
    }

    #[test]
    fn packing_order_preserves_source_order_for_equal_height_regions() {
        let source = image();
        let entries = [2, 5, 5, 1].map(|height| AtlasEntry {
            image: source.clone(),
            origin: [0, 0],
            expansion: ImageExpansion::from_factors(&source, [1, height]).unwrap(),
        });
        let mut order = [0, 1, 2, 3];
        sort_order(&mut order, &entries, &Account::for_test()).unwrap();
        assert_eq!(order, [1, 2, 0, 3]);
    }
}

//! Valid image sources and per-draw integer expansion are resolved before Vello packing.

use std::collections::HashMap;

use peniko::ImageData;
use surgeist_ledger::{Change, Limits, Operation, Work};
use vello_encoding::{DrawImage, Encoding, Patch};

use crate::{BackendErrorCode, Error, PhysicalSize, Result};

const PACKED_MAX: u32 = u16::MAX as u32;
const MAX_TEXELS: u64 = PACKED_MAX as u64 * PACKED_MAX as u64;
const SOURCE_BYTES: Work<2> = match Work::new(0) {
    Some(value) => value,
    None => unreachable!(),
};
const EXPANDED_TEXELS: Work<2> = match Work::new(1) {
    Some(value) => value,
    None => unreachable!(),
};

/// A validated nearest-neighbor expansion of the image's original pixel grid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ImageExpansion {
    factors: [u32; 2],
    extent: PhysicalSize,
}

impl ImageExpansion {
    pub(super) fn try_new(image: &ImageData, source_to_device: kurbo::Affine) -> Result<Self> {
        Self::from_factors(
            image,
            crate::image::pixelated_multipliers(source_to_device)?,
        )
    }

    fn from_factors(image: &ImageData, factors: [u32; 2]) -> Result<Self> {
        let width = image.width.checked_mul(factors[0]);
        let height = image.height.checked_mul(factors[1]);
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
}

pub(super) struct ImageAtlasPlan {
    pub(super) encoding: Encoding,
    pub(super) extent: PhysicalSize,
    pub(super) entries: Vec<AtlasEntry>,
}

impl ImageAtlasPlan {
    pub(super) fn prepare(
        encoding: &Encoding,
        expansions: &HashMap<usize, ImageExpansion>,
    ) -> Result<Self> {
        let mut encoding = encoding.clone();
        let mut entries: Vec<AtlasEntry> = Vec::new();
        let mut identities = HashMap::new();
        let mut draws = Vec::new();
        // This ledger owns aggregate preparation/upload work. Geometry coordinates
        // below are packing state, not another aggregate accounting authority.
        let mut ledger = Operation::<2, 0>::new(
            "render image atlas",
            Limits::new([MAX_TEXELS * 4, MAX_TEXELS], []),
            1,
        )
        .map_err(|error| failed(format!("image atlas admission failed: {error}")))?;
        let scope = ledger.root();
        for patch in &encoding.resources.patches {
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
                draws.push((*draw_data_offset, None));
                continue;
            }
            let expansion = match expansions.get(draw_data_offset) {
                Some(expansion) => *expansion,
                None => ImageExpansion::from_factors(image, [1, 1])?,
            };
            let bytes = u64::from(image.width) * u64::from(image.height) * 4;
            if image.format != peniko::ImageFormat::Rgba8
                || u64::try_from(image.data.data().len()).ok() != Some(bytes)
            {
                return Err(failed("image atlas requires complete RGBA8 source data"));
            }
            // Blob identity identifies bytes, while dimensions identify their
            // two-dimensional interpretation. Sampling/alpha flags stay per draw.
            let identity = (
                image.data.id(),
                image.width,
                image.height,
                expansion.factors,
            );
            let index = if let Some(index) = identities.get(&identity) {
                *index
            } else {
                let texels =
                    u64::from(expansion.extent.width()) * u64::from(expansion.extent.height());
                ledger
                    .apply_batch(
                        scope,
                        &[
                            Change::Work(SOURCE_BYTES, bytes),
                            Change::Work(EXPANDED_TEXELS, texels),
                        ],
                    )
                    .map_err(|error| failed(format!("image atlas admission failed: {error}")))?;
                let index = entries.len();
                entries.push(AtlasEntry {
                    image: image.clone(),
                    origin: [0, 0],
                    expansion,
                });
                identities.insert(identity, index);
                index
            };
            draws.push((*draw_data_offset, Some(index)));
        }
        let area = ledger
            .usage(scope)
            .map_err(|error| failed(format!("image atlas admission failed: {error}")))?
            .work(EXPANDED_TEXELS);
        let extent = pack_entries(&mut entries, area)?;
        for (offset, index) in draws {
            let words = encoding
                .draw_data
                .get_mut(
                    offset
                        ..offset
                            .checked_add(3)
                            .ok_or_else(|| failed("image draw offset overflow"))?,
                )
                .ok_or_else(|| failed("image patch does not identify a complete draw record"))?;
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
        encoding
            .resources
            .patches
            .retain(|patch| !matches!(patch, Patch::Image { .. }));
        Ok(Self {
            encoding,
            extent,
            entries,
        })
    }
}

fn pack_entries(entries: &mut [AtlasEntry], area: u64) -> Result<PhysicalSize> {
    let mut order = (0..entries.len()).collect::<Vec<_>>();
    order.sort_by_key(|index| std::cmp::Reverse(entries[*index].extent().height()));
    let largest_dimension = entries
        .iter()
        .map(|entry| entry.extent().width().max(entry.extent().height()))
        .max()
        .unwrap_or(1);
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

fn failed(message: impl Into<String>) -> Error {
    Error::new(BackendErrorCode::RenderFailed, message)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let expansion = ImageExpansion::try_new(
            &image(),
            kurbo::Affine::new([0.0, 2.4, -3.6, 0.0, 9.0, 7.0]),
        )
        .unwrap();
        assert_eq!(expansion.factors, [2, 4]);
        assert_eq!(expansion.extent, PhysicalSize::new(4, 4));
        let downscale =
            ImageExpansion::try_new(&image(), kurbo::Affine::scale_non_uniform(0.1, 0.4)).unwrap();
        assert_eq!(downscale.factors, [1, 1]);
    }

    #[test]
    fn oversized_expansion_is_rejected_without_truncation() {
        assert!(ImageExpansion::try_new(&image(), kurbo::Affine::scale(32768.0)).is_err());
        assert!(ImageExpansion::try_new(&image(), kurbo::Affine::scale(f64::INFINITY)).is_err());
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
        let plan = ImageAtlasPlan::prepare(&encoding, &HashMap::new()).unwrap();
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.encoding.draw_data[1], (2 << 16) | 1);
        assert_eq!(plan.encoding.draw_data[4], (1 << 16) | 2);
    }

    #[test]
    fn malformed_or_unsupported_source_is_refused_before_recording_uploads() {
        let mut source = image();
        source.format = peniko::ImageFormat::Bgra8;
        let mut encoding = Encoding::new();
        encoding.encode_image(&source, 1.0);
        assert!(ImageAtlasPlan::prepare(&encoding, &HashMap::new()).is_err());
        let mut source = image();
        source.width = 3;
        let mut encoding = Encoding::new();
        encoding.encode_image(&source, 1.0);
        assert!(ImageAtlasPlan::prepare(&encoding, &HashMap::new()).is_err());
    }

    #[test]
    fn aggregate_expansion_admission_rejects_unrepresentable_atlas_work() {
        let image = image();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image, 1.0);
        encoding.encode_image(&image, 1.0);
        let first = ImageExpansion::from_factors(&image, [32767, 65535]).unwrap();
        let second = ImageExpansion::from_factors(&image, [32766, 65535]).unwrap();
        let error = ImageAtlasPlan::prepare(&encoding, &HashMap::from([(0, first), (3, second)]))
            .err()
            .expect("the two expanded grids exceed the representable atlas area");
        assert!(error.to_string().contains("admission"));
        assert_eq!(encoding.resources.patches.len(), 2);
    }

    #[test]
    fn empty_image_brush_has_no_upload_and_resolves_to_transparent_dimensions() {
        let image = crate::Image::from_rgba(crate::Size::new(0.0, 3.0), Vec::<u8>::new()).unwrap();
        let mut encoding = Encoding::new();
        encoding.encode_image(&image.data, 1.0);
        let plan = ImageAtlasPlan::prepare(&encoding, &HashMap::new()).unwrap();
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
        let extent = pack_entries(&mut entries, 4 * 4000 * 2500).unwrap();
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
        let plan = ImageAtlasPlan::prepare(&encoding, &HashMap::new()).unwrap();
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
        let first = ImageAtlasPlan::prepare(&encoding, &HashMap::new()).unwrap();
        let second = ImageAtlasPlan::prepare(&encoding, &HashMap::new()).unwrap();
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
        let plan = ImageAtlasPlan::prepare(&encoding, &HashMap::from([(3, expansion)])).unwrap();
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
}

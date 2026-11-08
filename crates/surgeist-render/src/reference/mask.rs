use super::{
    PremultipliedRgba8, ReferencePremultipliedRgba8Buffer,
    premultiplied_rgba8_reference_to_straight_rgba8_image_buffer, round_byte,
    straight_rgba8_image_buffer_to_premultiplied_rgba8_reference,
};
use crate::{
    Error, Extend, Image, ImageBuffer, ImageQuality, ImageRendering, Rect, ResolvedLayerAlphaMask,
    Result, image::validate_image_buffer_rgba_len, layer::BlendMode,
};

impl PremultipliedRgba8 {
    pub(crate) fn apply_opacity(self, opacity: f32) -> Result<Self> {
        if !opacity.is_finite() {
            return Err(Error::invalid_value(
                "reference opacity",
                opacity,
                "must be finite",
            ));
        }
        Ok(self.apply_opacity_amount(f64::from(opacity)))
    }

    pub(crate) const fn source_over(self, destination: Self) -> Self {
        if self.alpha == 0 {
            return destination;
        }
        if self.alpha == u8::MAX {
            return self;
        }
        if destination.alpha == 0 {
            return self;
        }
        let inverse_source_alpha = u8::MAX - self.alpha;
        Self {
            red: self.red + scale_channel_by_alpha(destination.red, inverse_source_alpha),
            green: self.green + scale_channel_by_alpha(destination.green, inverse_source_alpha),
            blue: self.blue + scale_channel_by_alpha(destination.blue, inverse_source_alpha),
            alpha: self.alpha + scale_channel_by_alpha(destination.alpha, inverse_source_alpha),
        }
    }

    pub(crate) fn blend_over(self, destination: Self, mode: BlendMode) -> Self {
        match mode {
            BlendMode::Normal => self.source_over(destination),
            BlendMode::Plus => self.plus_lighter(destination),
            BlendMode::Multiply
            | BlendMode::Screen
            | BlendMode::Overlay
            | BlendMode::Darken
            | BlendMode::Lighten
            | BlendMode::ColorDodge
            | BlendMode::ColorBurn
            | BlendMode::HardLight
            | BlendMode::SoftLight
            | BlendMode::Difference
            | BlendMode::Exclusion => self.mix_blend_over(destination, mode),
        }
    }

    const fn plus_lighter(self, destination: Self) -> Self {
        Self {
            red: self.red.saturating_add(destination.red),
            green: self.green.saturating_add(destination.green),
            blue: self.blue.saturating_add(destination.blue),
            alpha: self.alpha.saturating_add(destination.alpha),
        }
    }

    fn mix_blend_over(self, destination: Self, mode: BlendMode) -> Self {
        if self.alpha == 0 {
            return destination;
        }
        if destination.alpha == 0 {
            return self;
        }

        let output_alpha = self.source_over(destination).alpha;
        let context = MixBlendContext {
            source_alpha: f64::from(self.alpha) / 255.0,
            destination_alpha: f64::from(destination.alpha) / 255.0,
            mode,
            output_alpha,
        };
        Self {
            red: mix_blend_channel(
                self.red,
                self.alpha,
                destination.red,
                destination.alpha,
                context,
            ),
            green: mix_blend_channel(
                self.green,
                self.alpha,
                destination.green,
                destination.alpha,
                context,
            ),
            blue: mix_blend_channel(
                self.blue,
                self.alpha,
                destination.blue,
                destination.alpha,
                context,
            ),
            alpha: output_alpha,
        }
    }

    pub(crate) const fn source_in_alpha_of(self, destination: Self) -> Self {
        self.scale_by_alpha(destination.alpha)
    }

    pub(crate) const fn destination_in_alpha_of(self, source: Self) -> Self {
        self.scale_by_alpha(source.alpha)
    }

    const fn scale_by_alpha(self, alpha: u8) -> Self {
        if alpha == 0 || self.alpha == 0 {
            return Self::TRANSPARENT;
        }
        if alpha == u8::MAX {
            return self;
        }
        Self {
            red: scale_channel_by_alpha(self.red, alpha),
            green: scale_channel_by_alpha(self.green, alpha),
            blue: scale_channel_by_alpha(self.blue, alpha),
            alpha: scale_channel_by_alpha(self.alpha, alpha),
        }
    }

    pub(crate) fn apply_opacity_amount(self, opacity: f64) -> Self {
        let opacity = opacity.clamp(0.0, 1.0);
        Self {
            red: scale_channel_by_opacity(self.red, opacity),
            green: scale_channel_by_opacity(self.green, opacity),
            blue: scale_channel_by_opacity(self.blue, opacity),
            alpha: scale_channel_by_opacity(self.alpha, opacity),
        }
    }
}

impl ReferencePremultipliedRgba8Buffer {
    pub(crate) fn apply_opacity(&self, opacity: f32) -> Result<Self> {
        let pixels = self
            .pixels
            .iter()
            .copied()
            .map(|pixel| pixel.apply_opacity(opacity))
            .collect::<Result<Vec<_>>>()?;
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn apply_alpha_mask(&self, mask: &Self) -> Result<Self> {
        if self.physical_size != mask.physical_size {
            return Err(Error::invalid_value(
                "reference alpha mask size",
                format!(
                    "{}x{}",
                    mask.physical_size.width(),
                    mask.physical_size.height()
                ),
                "must match source size",
            ));
        }
        let pixels = self
            .pixels
            .iter()
            .copied()
            .zip(mask.pixels.iter().copied())
            .map(|(source, mask)| source.destination_in_alpha_of(mask))
            .collect();
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn apply_resolved_alpha_mask(
        &self,
        source_bounds: Rect,
        mask: &Image,
        mask_bounds: Rect,
    ) -> Result<Self> {
        let width = self.physical_size.width();
        let height = self.physical_size.height();
        let source_to_device = kurbo::Affine::scale_non_uniform(
            mask_bounds.width() / f64::from(mask.data.width) * f64::from(width)
                / source_bounds.width(),
            mask_bounds.height() / f64::from(mask.data.height) * f64::from(height)
                / source_bounds.height(),
        );
        let factors = if mask.rendering_hint() == Some(ImageRendering::Pixelated) {
            crate::image::pixelated_multipliers(source_to_device)?
        } else {
            [1, 1]
        };
        let mut pixels = Vec::with_capacity(self.pixels.len());
        for y in 0..height {
            let local_y = source_bounds.y()
                + (f64::from(y) + 0.5) * source_bounds.height() / f64::from(height);
            for x in 0..width {
                let local_x = source_bounds.x()
                    + (f64::from(x) + 0.5) * source_bounds.width() / f64::from(width);
                let mask_alpha =
                    sample_resolved_mask_alpha(mask, mask_bounds, local_x, local_y, factors);
                pixels.push(self.pixel(x, y)?.apply_opacity_amount(mask_alpha));
            }
        }
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn source_over(&self, destination: &Self) -> Result<Self> {
        if self.physical_size != destination.physical_size {
            return Err(Error::invalid_value(
                "reference source-over destination size",
                format!(
                    "{}x{}",
                    destination.physical_size.width(),
                    destination.physical_size.height()
                ),
                "must match source size",
            ));
        }
        let pixels = self
            .pixels
            .iter()
            .copied()
            .zip(destination.pixels.iter().copied())
            .map(|(source, destination)| source.source_over(destination))
            .collect();
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn blend_over(&self, destination: &Self, mode: BlendMode) -> Result<Self> {
        if self.physical_size != destination.physical_size {
            return Err(Error::invalid_value(
                "reference blend destination size",
                format!(
                    "{}x{}",
                    destination.physical_size.width(),
                    destination.physical_size.height()
                ),
                "must match source size",
            ));
        }
        let pixels = self
            .pixels
            .iter()
            .copied()
            .zip(destination.pixels.iter().copied())
            .map(|(source, destination)| source.blend_over(destination, mode))
            .collect();
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn source_in_alpha_of(&self, destination: &Self) -> Result<Self> {
        if self.physical_size != destination.physical_size {
            return Err(Error::invalid_value(
                "reference source-in destination size",
                format!(
                    "{}x{}",
                    destination.physical_size.width(),
                    destination.physical_size.height()
                ),
                "must match source size",
            ));
        }
        let pixels = self
            .pixels
            .iter()
            .copied()
            .zip(destination.pixels.iter().copied())
            .map(|(source, destination)| source.source_in_alpha_of(destination))
            .collect();
        Self::from_pixels(self.physical_size, pixels)
    }

    pub(crate) fn destination_in_alpha_of(&self, source: &Self) -> Result<Self> {
        if self.physical_size != source.physical_size {
            return Err(Error::invalid_value(
                "reference destination-in source size",
                format!(
                    "{}x{}",
                    source.physical_size.width(),
                    source.physical_size.height()
                ),
                "must match destination size",
            ));
        }
        let pixels = self
            .pixels
            .iter()
            .copied()
            .zip(source.pixels.iter().copied())
            .map(|(destination, source)| destination.destination_in_alpha_of(source))
            .collect();
        Self::from_pixels(self.physical_size, pixels)
    }
}

fn sample_resolved_mask_alpha(
    mask: &Image,
    bounds: Rect,
    x: f64,
    y: f64,
    factors: [u32; 2],
) -> f64 {
    let max = bounds.max();
    if x < bounds.x() || x > max.x() || y < bounds.y() || y > max.y() {
        return 0.0;
    }

    let width = mask.data.width;
    let height = mask.data.height;
    if width == 0 || height == 0 {
        return 0.0;
    }

    let sample_x = ((x - bounds.x()) / bounds.width()).mul_add(f64::from(width), -0.5);
    let sample_y = ((y - bounds.y()) / bounds.height()).mul_add(f64::from(height), -0.5);
    if mask.rendering_hint() == Some(ImageRendering::Pixelated) {
        let factor_x = f64::from(factors[0]);
        let factor_y = f64::from(factors[1]);
        let enlarged_x = (sample_x + 0.5) * factor_x - 0.5;
        let enlarged_y = (sample_y + 0.5) * factor_y - 0.5;
        let left = enlarged_x.floor();
        let top = enlarged_y.floor();
        let horizontal = enlarged_x - left;
        let vertical = enlarged_y - top;
        let first_x = (left / factor_x).floor() as i64;
        let first_y = (top / factor_y).floor() as i64;
        let second_x = ((left + 1.0) / factor_x).floor() as i64;
        let second_y = ((top + 1.0) / factor_y).floor() as i64;
        let top_alpha = mask_alpha_tap(mask, first_x, first_y).mul_add(
            1.0 - horizontal,
            mask_alpha_tap(mask, second_x, first_y) * horizontal,
        );
        let bottom_alpha = mask_alpha_tap(mask, first_x, second_y).mul_add(
            1.0 - horizontal,
            mask_alpha_tap(mask, second_x, second_y) * horizontal,
        );
        return top_alpha.mul_add(1.0 - vertical, bottom_alpha * vertical);
    }
    match mask.sampling_quality() {
        ImageQuality::Low => mask_alpha_tap(
            mask,
            (sample_x + 0.5).floor() as i64,
            (sample_y + 0.5).floor() as i64,
        ),
        ImageQuality::Medium => {
            let left = sample_x.floor() as i64;
            let top = sample_y.floor() as i64;
            let horizontal = sample_x - left as f64;
            let vertical = sample_y - top as f64;
            let top_alpha = mask_alpha_tap(mask, left, top).mul_add(
                1.0 - horizontal,
                mask_alpha_tap(mask, left + 1, top) * horizontal,
            );
            let bottom_alpha = mask_alpha_tap(mask, left, top + 1).mul_add(
                1.0 - horizontal,
                mask_alpha_tap(mask, left + 1, top + 1) * horizontal,
            );
            top_alpha.mul_add(1.0 - vertical, bottom_alpha * vertical)
        }
        ImageQuality::High => {
            let base_x = sample_x.floor() as i64;
            let base_y = sample_y.floor() as i64;
            let mut alpha = 0.0;
            for offset_y in -1_i64..=2 {
                let tap_y = base_y + offset_y;
                let weight_y = mitchell_netravali(sample_y - tap_y as f64);
                for offset_x in -1_i64..=2 {
                    let tap_x = base_x + offset_x;
                    let weight_x = mitchell_netravali(sample_x - tap_x as f64);
                    alpha += mask_alpha_tap(mask, tap_x, tap_y) * weight_x * weight_y;
                }
            }
            alpha.clamp(0.0, 1.0)
        }
    }
}

fn mask_alpha_tap(mask: &Image, x: i64, y: i64) -> f64 {
    let Some(x) = extend_mask_index(x, mask.data.width, mask.extend) else {
        return 0.0;
    };
    let Some(y) = extend_mask_index(y, mask.data.height, mask.extend) else {
        return 0.0;
    };
    let alpha_index = u64::from(y)
        .checked_mul(u64::from(mask.data.width))
        .and_then(|row| row.checked_add(u64::from(x)))
        .and_then(|pixel| pixel.checked_mul(4))
        .and_then(|byte| byte.checked_add(3))
        .and_then(|byte| usize::try_from(byte).ok());
    alpha_index
        .and_then(|index| mask.bytes.get(index))
        .map_or(0.0, |alpha| f64::from(*alpha) / 255.0)
}

fn extend_mask_index(index: i64, length: u32, extend: Extend) -> Option<u32> {
    if length == 0 {
        return None;
    }
    let length = i64::from(length);
    let extended = match extend {
        Extend::Pad => index.clamp(0, length - 1),
        Extend::Repeat => index.rem_euclid(length),
        Extend::Reflect => {
            let period = length * 2;
            let reflected = index.rem_euclid(period);
            if reflected < length {
                reflected
            } else {
                period - reflected - 1
            }
        }
    };
    u32::try_from(extended).ok()
}

fn mitchell_netravali(distance: f64) -> f64 {
    const B: f64 = 1.0 / 3.0;
    const C: f64 = 1.0 / 3.0;
    let distance = distance.abs();
    if distance < 1.0 {
        ((12.0 - 9.0 * B - 6.0 * C) * distance.powi(3)
            + (-18.0 + 12.0 * B + 6.0 * C) * distance.powi(2)
            + (6.0 - 2.0 * B))
            / 6.0
    } else if distance < 2.0 {
        ((-B - 6.0 * C) * distance.powi(3)
            + (6.0 * B + 30.0 * C) * distance.powi(2)
            + (-12.0 * B - 48.0 * C) * distance
            + (8.0 * B + 24.0 * C))
            / 6.0
    } else {
        0.0
    }
}

fn scale_channel_by_opacity(channel: u8, opacity: f64) -> u8 {
    round_byte(f64::from(channel) * opacity)
}

#[derive(Clone, Copy)]
struct MixBlendContext {
    source_alpha: f64,
    destination_alpha: f64,
    mode: BlendMode,
    output_alpha: u8,
}

fn mix_blend_channel(
    source_channel: u8,
    source_alpha_byte: u8,
    destination_channel: u8,
    destination_alpha_byte: u8,
    context: MixBlendContext,
) -> u8 {
    let source = f64::from(source_channel) / f64::from(source_alpha_byte);
    let destination = f64::from(destination_channel) / f64::from(destination_alpha_byte);
    let blended = blend_straight_channel(source, destination, context.mode);
    let premultiplied = (1.0 - context.source_alpha) * context.destination_alpha * destination
        + (1.0 - context.destination_alpha) * context.source_alpha * source
        + context.source_alpha * context.destination_alpha * blended;
    round_byte(premultiplied * 255.0).min(context.output_alpha)
}

fn blend_straight_channel(source: f64, destination: f64, mode: BlendMode) -> f64 {
    match mode {
        BlendMode::Multiply => source * destination,
        BlendMode::Screen => source + destination - source * destination,
        BlendMode::Overlay => {
            if destination <= 0.5 {
                2.0 * source * destination
            } else {
                1.0 - 2.0 * (1.0 - source) * (1.0 - destination)
            }
        }
        BlendMode::Darken => source.min(destination),
        BlendMode::Lighten => source.max(destination),
        BlendMode::ColorDodge => {
            if destination == 0.0 {
                0.0
            } else if source == 1.0 {
                1.0
            } else {
                (destination / (1.0 - source)).min(1.0)
            }
        }
        BlendMode::ColorBurn => {
            if destination == 1.0 {
                1.0
            } else if source == 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - destination) / source).min(1.0)
            }
        }
        BlendMode::HardLight => {
            if source <= 0.5 {
                2.0 * source * destination
            } else {
                1.0 - 2.0 * (1.0 - source) * (1.0 - destination)
            }
        }
        BlendMode::SoftLight => {
            if source <= 0.5 {
                destination - (1.0 - 2.0 * source) * destination * (1.0 - destination)
            } else {
                let curve = if destination <= 0.25 {
                    ((16.0 * destination - 12.0) * destination + 4.0) * destination
                } else {
                    destination.sqrt()
                };
                destination + (2.0 * source - 1.0) * (curve - destination)
            }
        }
        BlendMode::Difference => (destination - source).abs(),
        BlendMode::Exclusion => destination + source - 2.0 * destination * source,
        BlendMode::Normal | BlendMode::Plus => {
            unreachable!("normal and plus are handled before mix blend math")
        }
    }
    .clamp(0.0, 1.0)
}

const fn scale_channel_by_alpha(channel: u8, alpha: u8) -> u8 {
    let scaled = (channel as u16) * (alpha as u16) + 127;
    (scaled / 255) as u8
}

#[cfg(test)]
pub(crate) fn execute_transitional_resolved_mask_bridge_for_test(
    source: &ImageBuffer,
    source_bounds: Rect,
    image: Image,
    mask_bounds: Rect,
) -> Result<ImageBuffer> {
    let mask = ResolvedLayerAlphaMask::try_new(image, mask_bounds)?;
    validate_image_buffer_rgba_len(source.size(), source.rgba().len())?;
    crate::validation::validate_point(source_bounds.origin(), "resolved-mask source bounds")?;
    crate::validation::validate_positive_f64(
        source_bounds.width(),
        "resolved-mask source bounds width",
    )?;
    crate::validation::validate_positive_f64(
        source_bounds.height(),
        "resolved-mask source bounds height",
    )?;
    let source = straight_rgba8_image_buffer_to_premultiplied_rgba8_reference(source)?;
    let masked = source.apply_resolved_alpha_mask(source_bounds, mask.image(), mask.bounds())?;
    premultiplied_rgba8_reference_to_straight_rgba8_image_buffer(&masked)
}

#[cfg(test)]
mod separable_tests {
    use super::*;

    #[test]
    fn reference_separable_division_guards_preserve_corner_precedence() {
        for (mode, source, backdrop, expected) in [
            (BlendMode::ColorDodge, 1.0, 0.0, 0.0),
            (BlendMode::ColorDodge, 1.0, 0.25, 1.0),
            (BlendMode::ColorDodge, 0.5, 0.25, 0.5),
            (BlendMode::ColorDodge, 0.5, 0.75, 1.0),
            (BlendMode::ColorBurn, 0.0, 1.0, 1.0),
            (BlendMode::ColorBurn, 0.0, 0.75, 0.0),
            (BlendMode::ColorBurn, 0.5, 0.75, 0.5),
            (BlendMode::ColorBurn, 0.5, 0.25, 0.0),
        ] {
            assert_eq!(blend_straight_channel(source, backdrop, mode), expected);
        }
    }

    #[test]
    fn reference_separable_light_thresholds_match_independent_fraction_anchors() {
        // Independently evaluated Compositing 1 equations. Values on both sides
        // of each threshold distinguish branches that agree at equality.
        for (mode, source, backdrop, expected) in [
            (BlendMode::HardLight, 0.25, 0.25, 0.125),
            (BlendMode::HardLight, 0.5, 0.25, 0.25),
            (BlendMode::HardLight, 0.75, 0.25, 0.625),
            (BlendMode::Overlay, 0.75, 0.25, 0.375),
            (BlendMode::Overlay, 0.25, 0.75, 0.625),
            (BlendMode::SoftLight, 0.75, 0.125, 0.234375),
            (BlendMode::SoftLight, 0.75, 0.25, 0.375),
            (BlendMode::SoftLight, 0.75, 0.5625, 0.65625),
            (BlendMode::SoftLight, 0.25, 0.25, 0.15625),
            (BlendMode::SoftLight, 0.5, 0.25, 0.25),
            (BlendMode::SoftLight, 1.0, 0.0, 0.0),
            (BlendMode::SoftLight, 1.0, 1.0, 1.0),
        ] {
            assert_eq!(blend_straight_channel(source, backdrop, mode), expected);
        }
    }

    #[test]
    fn reference_separable_difference_and_exclusion_preserve_distinct_channels() {
        for (source, backdrop, difference, exclusion) in [
            (0.75, 0.25, 0.5, 0.625),
            (0.25, 0.75, 0.5, 0.625),
            (0.5, 0.5, 0.0, 0.5),
            (0.75, 0.0, 0.75, 0.75),
            (0.75, 1.0, 0.25, 0.25),
        ] {
            assert_eq!(
                blend_straight_channel(source, backdrop, BlendMode::Difference),
                difference
            );
            assert_eq!(
                blend_straight_channel(source, backdrop, BlendMode::Exclusion),
                exclusion
            );
        }
    }

    #[test]
    fn reference_separable_transparent_edges_keep_premultiplied_inputs() {
        let source = PremultipliedRgba8::try_new(96, 32, 64, 128).unwrap();
        let backdrop = PremultipliedRgba8::try_new(32, 96, 64, 128).unwrap();
        for mode in [
            BlendMode::ColorDodge,
            BlendMode::ColorBurn,
            BlendMode::HardLight,
            BlendMode::SoftLight,
            BlendMode::Difference,
            BlendMode::Exclusion,
        ] {
            assert_eq!(
                source.blend_over(PremultipliedRgba8::TRANSPARENT, mode),
                source
            );
            assert_eq!(
                PremultipliedRgba8::TRANSPARENT.blend_over(backdrop, mode),
                backdrop
            );
            assert_eq!(
                PremultipliedRgba8::TRANSPARENT.blend_over(PremultipliedRgba8::TRANSPARENT, mode),
                PremultipliedRgba8::TRANSPARENT
            );
        }
    }
}

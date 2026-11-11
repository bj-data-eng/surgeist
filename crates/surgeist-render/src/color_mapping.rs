use surgeist_color::{Color as ResolvedColor, ColorSpace, Finite, PredefinedColorSpace};

use crate::accounting::Account;
use crate::{Color, RenderAccountingContext, RenderAttemptKind, RenderPhase, RenderWork, Result};

const SRGB: ColorSpace = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
const JUST_NOTICEABLE_DIFFERENCE: f64 = 0.02;
const SEARCH_EPSILON: f64 = 0.0001;

/// Prepares one resolved individual color for an SDR sRGB destination.
///
/// The caller selects the destination before lowering paint. In-gamut sRGB
/// coordinates pass through, and other colors use CSS Color 4 binary search
/// with Local MINDE. Missing components are zero for rendering. Alpha remains
/// straight and is independent of the gamut search.
///
/// The operation reports coordinate work and timing through `accounting`.
/// A work refusal returns a typed diagnostic before yielding a paint color.
pub fn prepare_sdr_srgb_color(
    resolved: ResolvedColor,
    accounting: &mut RenderAccountingContext,
) -> Result<Color> {
    accounting.run(
        RenderAttemptKind::SdrColor,
        RenderPhase::Preparation,
        |account| {
            account.phase(RenderPhase::Preparation, || {
                let coordinates = account.at(RenderPhase::Coordinates);
                coordinates.phase(RenderPhase::Coordinates, || {
                    coordinates.work(RenderWork::PaintVisits, 1)?;
                    let alpha = coordinates.coordinate(|| resolved.alpha().unwrap_or(0.0))?;
                    let [r, g, b] = map_to_srgb(resolved, &coordinates)?;
                    coordinates.work(RenderWork::CoordinateVisits, 4)?;
                    Color::try_rgba(r as f32, g as f32, b as f32, alpha as f32)
                })
            })
        },
    )
}

fn map_to_srgb(resolved: ResolvedColor, account: &Account) -> Result<[f64; 3]> {
    let rgb = channels(convert(resolved, SRGB, account)?, account)?;
    if in_gamut(rgb, account)? {
        return Ok(rgb);
    }

    // Going through Cartesian Oklab canonicalizes any signed polar chroma
    // while preserving the represented color and the independent alpha.
    let original = convert(resolved, ColorSpace::Oklab, account)?;
    let polar = convert(original, ColorSpace::Oklch, account)?;
    account.work(RenderWork::CoordinateVisits, 3)?;
    let [lightness, chroma, hue] = polar.finite_channels();
    let lightness_value = lightness.map_or(0.0, Finite::get);
    if account.coordinate(|| lightness_value >= 1.0)? {
        return Ok([1.0; 3]);
    }
    if account.coordinate(|| lightness_value <= 0.0)? {
        return Ok([0.0; 3]);
    }

    let mut clipped = clip(rgb, account)?;
    let initial_difference = delta_ok(original, clipped, account)?;
    if account.coordinate(|| initial_difference < JUST_NOTICEABLE_DIFFERENCE)? {
        return Ok(clipped);
    }

    let mut low: f64 = 0.0;
    let mut high = chroma.map_or(0.0, Finite::get);
    let mut low_is_in_gamut = true;
    while account.coordinate(|| high - low > SEARCH_EPSILON)? {
        let chroma = account.coordinate(|| low.midpoint(high))?;
        // The overflow-safe midpoint strictly shrinks the finite interval
        // unless its endpoints are adjacent in binary64.
        if account.coordinate(|| chroma == low || chroma == high)? {
            break;
        }
        let candidate = ResolvedColor::new(
            ColorSpace::Oklch,
            [lightness, Finite::new(chroma), hue],
            None,
        );
        let rgb = channels(convert(candidate, SRGB, account)?, account)?;
        if low_is_in_gamut && in_gamut(rgb, account)? {
            low = chroma;
            continue;
        }
        clipped = clip(rgb, account)?;
        let difference = delta_ok(candidate, clipped, account)?;
        if account.coordinate(|| difference < JUST_NOTICEABLE_DIFFERENCE)? {
            if account.coordinate(|| JUST_NOTICEABLE_DIFFERENCE - difference < SEARCH_EPSILON)? {
                return Ok(clipped);
            }
            low_is_in_gamut = false;
            low = chroma;
        } else {
            high = chroma;
        }
    }
    Ok(clipped)
}

fn convert(
    color: ResolvedColor,
    destination: ColorSpace,
    account: &Account,
) -> Result<ResolvedColor> {
    if color.space() == destination {
        return Ok(color);
    }
    // One fixed Color conversion evaluates three destination coordinates.
    account.work(RenderWork::CoordinateVisits, 3)?;
    Ok(color.converted(destination))
}

fn channels(color: ResolvedColor, account: &Account) -> Result<[f64; 3]> {
    account.work(RenderWork::CoordinateVisits, 3)?;
    Ok(color.channels().map(|channel| channel.unwrap_or(0.0)))
}

fn in_gamut(rgb: [f64; 3], account: &Account) -> Result<bool> {
    for channel in rgb {
        if !account.coordinate(|| (0.0..=1.0).contains(&channel))? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn clip(rgb: [f64; 3], account: &Account) -> Result<[f64; 3]> {
    account.work(RenderWork::CoordinateVisits, 3)?;
    Ok(rgb.map(|channel| channel.clamp(0.0, 1.0)))
}

fn delta_ok(candidate: ResolvedColor, clipped: [f64; 3], account: &Account) -> Result<f64> {
    let candidate = channels(convert(candidate, ColorSpace::Oklab, account)?, account)?;
    let clipped = ResolvedColor::new(SRGB, clipped.map(Finite::new), None);
    let clipped = channels(convert(clipped, ColorSpace::Oklab, account)?, account)?;
    let [l, a, b] = candidate;
    let [clipped_l, clipped_a, clipped_b] = clipped;
    account.work(RenderWork::CoordinateVisits, 5)?;
    Ok((l - clipped_l).hypot(a - clipped_a).hypot(b - clipped_b))
}

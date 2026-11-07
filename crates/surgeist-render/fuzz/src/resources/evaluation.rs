use std::error::Error as _;

use surgeist_font::{DecodeError, DecodeLimits, LimitKind};
use surgeist_render::{
    Error, ErrorCode, FontData, GlyphNumericPhase, GlyphPreparationError, GlyphResource,
    GlyphRunLimits, GlyphSynthesis, Image, NormalizedCoordinate, PathElement, Point,
    PreparedGlyphRun, Rect, RenderAccountingContext, RenderAccountingError,
    RenderAccountingOptions, RenderAttemptKind, RenderAttemptStatus, RenderLimits, RenderQuantity,
    RenderRefusalReason, RenderResource, RenderWork, Size, TextGlyph, TextInstance,
    TextRunBoundsKind,
};

use super::*;
use crate::{evaluated, product_observation, rejected, violated};

fn decode_limits(f: &FontCase) -> DecodeLimits {
    let [a, b, c, d, e, f, g] = f.limits.map(|v| v as usize);
    DecodeLimits::new(a, b, c, d, e, f, g)
}
fn glyph_limits(limits: [u32; 4]) -> GlyphRunLimits {
    let [a, b, c, d] = limits.map(|v| v as usize);
    GlyphRunLimits::new(a, b, c, d)
}
fn admit(f: &FontCase) -> surgeist_render::Result<FontData> {
    FontData::try_from_bytes(f.bytes.clone(), f.index, decode_limits(f))
}

pub(super) fn evaluate(case: &ResourceCase) -> Result<CaseOutcome> {
    match &case.operation {
        Operation::Font(f) => match admit(f) {
            Ok(font) => {
                if f.bytes == AHEM && f.index != 0 {
                    return violated(
                        FONT,
                        "font-face-index",
                        "single-face Ahem accepted an unavailable face",
                    );
                }
                if f.bytes.len() > f.limits[0] as usize {
                    return violated(
                        FONT,
                        "font-encoded-cap",
                        "font admission exceeded explicit encoded cap",
                    );
                }
                if !font_preserved(f, &font) {
                    return violated(
                        FONT,
                        "font-provenance",
                        "admitted face changed original bytes or requested index",
                    );
                }
                evaluated(&[FONT])
            }
            Err(e) => font_rejection(f, &e),
        },
        Operation::Image {
            width,
            height,
            rgba,
        } => image(*width, *height, rgba),
        Operation::Glyph(g) => glyph(g),
        Operation::Retry {
            glyph,
            refusal,
            refused_limits,
            refused_path_work,
            raised_path_work,
        } => retry(
            glyph,
            *refusal,
            *refused_limits,
            *refused_path_work,
            *raised_path_work,
        ),
    }
}

fn font_preserved(input: &FontCase, font: &FontData) -> bool {
    font.admitted_face().index() == input.index
        && font.admitted_face().resource().source().bytes() == input.bytes
}

fn font_rejection(f: &FontCase, e: &Error) -> Result<CaseOutcome> {
    if f.bytes == AHEM && f.index == 0 && f.limits == FONT_CAPS {
        return violated(
            FONT,
            "known-font-refused",
            format!("exact licensed Ahem refused: {e}"),
        );
    }
    if e.code() != ErrorCode::InvalidInput
        || !e
            .invalid_value_diagnostic()
            .is_some_and(|d| d.field() == "font_data")
    {
        return violated(
            FONT,
            "font-refusal-type",
            "font refusal lost the public invalid font-data diagnostic",
        );
    }
    let source = e.source().and_then(|e| e.downcast_ref::<DecodeError>());
    if f.bytes.len() > f.limits[0] as usize
        && !matches!(
            source,
            Some(DecodeError::LimitExceeded {
                kind: LimitKind::EncodedBytes,
                ..
            })
        )
    {
        return violated(
            FONT,
            "font-encoded-refusal",
            "encoded cap refusal lost producer limit diagnostic",
        );
    }
    // Other arbitrary inputs may be invalid containers, unsupported encodings,
    // or valid containers with an unavailable selected index. No ink is guessed.
    if e.source().is_some() && source.is_none() {
        return violated(
            FONT,
            "font-producer-source",
            "font refusal replaced the producer's typed source",
        );
    }
    rejected(&[FONT], "typed font container or selected-face refusal")
}

fn image(width: f64, height: f64, rgba: &[u8]) -> Result<CaseOutcome> {
    let geometry_valid = width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0;
    let size = match Size::try_new(width, height) {
        Ok(size) if geometry_valid => size,
        Err(e) if !geometry_valid && e.code() == ErrorCode::InvalidInput => {
            return rejected(&[IMAGE], "typed invalid image geometry");
        }
        _ => {
            return violated(
                IMAGE,
                "image-geometry",
                "finite nonnegative geometry admission disagreed with the input",
            );
        }
    };
    let integral = width.fract() == 0.0
        && height.fract() == 0.0
        && width <= f64::from(u32::MAX)
        && height <= f64::from(u32::MAX);
    let valid = integral && width * height * 4.0 == rgba.len() as f64;
    match Image::from_rgba(size, rgba.to_vec()) {
        Ok(image) if valid => {
            let repeated = match product_observation(
                Image::from_rgba(size, rgba.to_vec()),
                IMAGE,
                "image-repeat-admission",
            )? {
                Ok(image) => image,
                Err(outcome) => return Ok(outcome),
            };
            if image.size() != size || image != repeated || image.id() != repeated.id() {
                return violated(
                    IMAGE,
                    "image-content",
                    "RGBA admission lost dimensions or deterministic exact content equality",
                );
            }
            evaluated(&[IMAGE])
        }
        Err(e) if !valid && e.code() == ErrorCode::ImageUploadFailed => {
            rejected(&[IMAGE], "typed invalid RGBA dimensions or length")
        }
        _ => violated(
            IMAGE,
            "image-admission",
            "RGBA acceptance disagreed with integer dimensions and exact byte length",
        ),
    }
}

// Caller-invalid fields are checked independently before comparing the public
// constructor's actual result. This is not a replacement Render validator.
fn construct(g: &GlyphCase) -> std::result::Result<(TextInstance, Vec<TextGlyph>), CaseOutcome> {
    let reject = |ok: bool, discriminator: &str| {
        if ok {
            rejected(&[FONT, GLYPH], "typed invalid glyph input")
        } else {
            violated(
                GLYPH,
                discriminator,
                "glyph input constructor disagreed with its typed public contract",
            )
        }
        .expect("static valid observation")
    };
    let mut coordinates = Vec::new();
    for &bits in &g.coordinates {
        let result = NormalizedCoordinate::try_from_bits(bits);
        if !(-16384..=16384).contains(&bits) {
            return Err(reject(
                matches!(result, Err(ref e) if matches!(e.glyph_preparation_diagnostic(), Some(GlyphPreparationError::CoordinateValue { .. }))),
                "coordinate-value",
            ));
        }
        coordinates.push(result.map_err(|_| reject(false, "coordinate-value"))?);
    }
    let result = GlyphSynthesis::try_new(g.skew, g.embolden);
    if !g.skew.is_finite() || !g.embolden.is_finite() || g.embolden < 0.0 {
        return Err(reject(
            matches!(result, Err(ref e) if matches!(e.glyph_preparation_diagnostic(), Some(GlyphPreparationError::InvalidSynthesis))),
            "synthesis-value",
        ));
    }
    let synthesis = result.map_err(|_| reject(false, "synthesis-value"))?;
    let result = TextInstance::try_new(g.size, coordinates, synthesis);
    if !g.size.is_finite() || g.size <= 0.0 {
        return Err(reject(
            matches!(result, Err(ref e) if matches!(e.glyph_preparation_diagnostic(), Some(GlyphPreparationError::Numeric { phase: GlyphNumericPhase::Size, .. }))),
            "instance-size",
        ));
    }
    let instance = result.map_err(|_| reject(false, "instance-size"))?;
    let mut glyphs = Vec::new();
    for value in &g.glyphs {
        let result = TextGlyph::try_new(value.id, value.x, value.y, value.advance);
        if !value.x.is_finite() || !value.y.is_finite() || !value.advance.is_finite() {
            return Err(reject(
                matches!(result, Err(ref e) if e.code() == ErrorCode::InvalidInput && e.invalid_value_diagnostic().is_some()),
                "glyph-numeric",
            ));
        }
        glyphs.push(result.map_err(|_| reject(false, "glyph-numeric"))?);
    }
    Ok((instance, glyphs))
}

fn glyph(g: &GlyphCase) -> Result<CaseOutcome> {
    let font = match admit(&g.font) {
        Ok(font) => font,
        Err(e) => return font_rejection(&g.font, &e),
    };
    if !font_preserved(&g.font, &font) {
        return violated(
            FONT,
            "font-provenance",
            "glyph font changed its original bytes or index",
        );
    }
    let (instance, glyphs) = match construct(g) {
        Ok(v) => v,
        Err(outcome) => return Ok(outcome),
    };
    let mut context = RenderAccountingContext::default();
    match font.prepare_glyph_run(&instance, &glyphs, glyph_limits(g.limits), &mut context) {
        Ok(output) => {
            if !g.coordinates.is_empty()
                || g.glyphs.iter().any(|g| g.id >= 281)
                || glyphs.len() > g.limits[0] as usize
            {
                return violated(
                    GLYPH,
                    "invalid-preparation-accepted",
                    "preparation accepted an unavailable axis, glyph, or glyph count",
                );
            }
            if let Some(outcome) = check_output(
                g,
                &font,
                &instance,
                &glyphs,
                &output,
                GLYPH,
                "prepared-content",
            )? {
                return Ok(outcome);
            }
            if !report_clean(&context, RenderAttemptStatus::Succeeded) {
                return violated(
                    GLYPH,
                    "preparation-report",
                    "successful preparation lost its completed report or transient cleanup",
                );
            }
            evaluated(&[FONT, GLYPH])
        }
        Err(e) => {
            let valid = match e.glyph_preparation_diagnostic() {
                Some(GlyphPreparationError::Limit {
                    resource,
                    required,
                    limit,
                }) => {
                    let cap = match resource {
                        GlyphResource::Glyphs => g.limits[0],
                        GlyphResource::PathElements => g.limits[1],
                        GlyphResource::ScratchBytes => g.limits[2],
                        GlyphResource::RetainedBytes => g.limits[3],
                    } as usize;
                    *limit == cap && required > limit && g.limits != GLYPH_CAPS
                }
                Some(GlyphPreparationError::CoordinateCount { supplied, required }) => {
                    *supplied == g.coordinates.len() && *required == 0 && *supplied != 0
                }
                Some(GlyphPreparationError::GlyphRange { glyph, count }) => {
                    *count == 281 && *glyph >= 281 && g.glyphs.iter().any(|g| g.id == *glyph)
                }
                Some(GlyphPreparationError::Numeric { .. }) => !golden_domain(g),
                _ => false,
            };
            if !valid {
                return violated(
                    GLYPH,
                    "preparation-refusal-type",
                    format!("unexpected Ahem preparation refusal: {e}"),
                );
            }
            if context.last_attempt().is_none_or(|r| {
                r.kind() != RenderAttemptKind::GlyphRun
                    || r.usage().work(RenderWork::PreparedTransferredBytes) != 0
                    || r.usage().live(RenderResource::ScratchCpuBytes) != 0
                    || r.usage().live(RenderResource::RetainedCpuBytes) != 0
            }) {
                return violated(
                    GLYPH,
                    "failed-preparation-custody",
                    "failed preparation transferred output or retained transient custody",
                );
            }
            rejected(&[FONT, GLYPH], "typed glyph preparation refusal")
        }
    }
}

fn golden_domain(g: &GlyphCase) -> bool {
    (1.0..=4096.0).contains(&g.size)
        && g.skew.is_finite()
        && g.skew.abs() <= 2.0
        && (g.embolden == 0.0 || (1.0..=64.0).contains(&g.embolden))
        && g.glyphs.iter().all(|v| {
            v.x.is_finite() && v.y.is_finite() && v.x.abs() <= 4096.0 && v.y.abs() <= 4096.0
        })
}

fn check_output(
    g: &GlyphCase,
    font: &FontData,
    instance: &TextInstance,
    glyphs: &[TextGlyph],
    output: &PreparedGlyphRun,
    property: &str,
    discriminator: &str,
) -> Result<Option<CaseOutcome>> {
    if output.font() != font
        || output.instance() != instance
        || output.glyphs() != glyphs
        || output.bounds().kind() == TextRunBoundsKind::Unspecified
        || output.instance().size().to_bits() != g.size.to_bits()
        || output.instance().synthesis().skew().to_bits() != g.skew.to_bits()
        || output.instance().synthesis().embolden().to_bits() != g.embolden.to_bits()
        || output.glyphs().iter().zip(&g.glyphs).any(|(actual, raw)| {
            actual.id() != raw.id
                || actual.x().to_bits() != raw.x.to_bits()
                || actual.y().to_bits() != raw.y.to_bits()
                || actual.advance().to_bits() != raw.advance.to_bits()
        })
    {
        return violated(
            property,
            discriminator,
            "preparation changed immutable font, instance, glyphs or omitted actual bounds",
        )
        .map(Some);
    }
    let clone = output.clone();
    if !clone.is_same_preparation(output) {
        return violated(
            property,
            discriminator,
            "cloning changed the immutable preparation identity",
        )
        .map(Some);
    }
    if golden_domain(g) {
        let (expected, bounds) = match expected_outline(g, property)? {
            Ok(outline) => outline,
            Err(outcome) => return Ok(Some(outcome)),
        };
        if output.path_elements().collect::<Vec<_>>() != expected
            || output.bounds().ink_rect() != bounds
        {
            return violated(property,discriminator,
                "exact fixture rectangle, reflected placement, skew, empty outline or union bounds disagreed with source facts",
            ).map(Some);
        }
    }
    Ok(None)
}

// Ahem's licensed X is the rectangle (0,-200)..(1000,800), UPEM1000;
// space has no outline. Source: Render tests/prepared_outline.rs and fixture
// PROVENANCE.md. Positions and advance are distinct authored inputs.
type ExpectedOutline = (Vec<PathElement>, Option<Rect>);

fn expected_outline(
    g: &GlyphCase,
    property: &str,
) -> Result<std::result::Result<ExpectedOutline, CaseOutcome>> {
    let mut path = Vec::new();
    let mut bounds: Option<(f64, f64, f64, f64)> = None;
    for glyph in g.glyphs.iter().filter(|v| v.id == 58) {
        for shift in [0.0, g.embolden]
            .into_iter()
            .take(if g.embolden > 0.0 { 2 } else { 1 })
        {
            for (i, (x, y)) in [
                (0.0, 800.0),
                (1000.0, 800.0),
                (1000.0, -200.0),
                (0.0, -200.0),
            ]
            .into_iter()
            .enumerate()
            {
                let scale = f64::from(g.size) / 1000.0;
                let x = f64::from(
                    (x * scale + f64::from(g.skew) * y * scale + glyph.x + f64::from(shift)) as f32,
                );
                let y = f64::from((-y * scale + glyph.y) as f32);
                let point =
                    match product_observation(Point::try_new(x, y), property, "golden-point")? {
                        Ok(point) => point,
                        Err(outcome) => return Ok(Err(outcome)),
                    };
                path.push(if i == 0 {
                    PathElement::MoveTo(point)
                } else {
                    PathElement::LineTo(point)
                });
                bounds = Some(bounds.map_or((x, y, x, y), |(a, b, c, d)| {
                    (a.min(x), b.min(y), c.max(x), d.max(y))
                }));
            }
            path.push(PathElement::Close);
        }
    }
    let bounds = match bounds {
        Some((a, b, c, d)) => {
            match product_observation(Rect::try_new(a, b, c - a, d - b), property, "golden-bounds")?
            {
                Ok(bounds) => Some(bounds),
                Err(outcome) => return Ok(Err(outcome)),
            }
        }
        None => None,
    };
    Ok(Ok((path, bounds)))
}

fn report_clean(context: &RenderAccountingContext, status: RenderAttemptStatus) -> bool {
    context.last_attempt().is_some_and(|r| {
        r.kind() == RenderAttemptKind::GlyphRun
            && r.status() == status
            && r.usage().live(RenderResource::ScratchCpuBytes) == 0
            && r.usage().live(RenderResource::RetainedCpuBytes) == 0
    })
}

fn work_options(
    path_work: u64,
) -> Result<std::result::Result<RenderAccountingOptions, CaseOutcome>> {
    Ok(product_observation(
        RenderLimits::new().with_work_limit(RenderWork::PathElementsEmitted, path_work),
        RETRY,
        "retry-accounting-settings",
    )?
    .map(|limits| RenderAccountingOptions::new().with_limits(limits)))
}

fn retry(
    g: &GlyphCase,
    refusal: Refusal,
    low: [u32; 4],
    refused_work: u64,
    raised_work: u64,
) -> Result<CaseOutcome> {
    let font = match admit(&g.font) {
        Ok(v) => v,
        Err(e) => return violated(RETRY, "retry-font", e.to_string()),
    };
    let (instance, glyphs) = match construct(g) {
        Ok(v) => v,
        Err(v) => return Ok(v),
    };
    let raised_options = match work_options(raised_work)? {
        Ok(options) => options,
        Err(outcome) => return Ok(outcome),
    };
    let refused_options = match work_options(refused_work)? {
        Ok(options) => options,
        Err(outcome) => return Ok(outcome),
    };
    let mut context = RenderAccountingContext::new(raised_options);
    let previous =
        match font.prepare_glyph_run(&instance, &glyphs, glyph_limits(g.limits), &mut context) {
            Ok(v) => v,
            Err(e) => return violated(RETRY, "retry-baseline", e.to_string()),
        };
    if let Some(outcome) = check_output(
        g,
        &font,
        &instance,
        &glyphs,
        &previous,
        RETRY,
        "retry-baseline-content",
    )? {
        return Ok(outcome);
    }
    let retained = previous.clone();
    let resource = match refusal {
        Refusal::Glyphs => Some(GlyphResource::Glyphs),
        Refusal::Path => Some(GlyphResource::PathElements),
        Refusal::Scratch => Some(GlyphResource::ScratchBytes),
        Refusal::Retained => Some(GlyphResource::RetainedBytes),
        Refusal::Accounting => None,
    };
    context.set_options(refused_options);
    let error = match font.prepare_glyph_run(&instance, &glyphs, glyph_limits(low), &mut context) {
        Err(e) => e,
        Ok(_) => {
            return violated(
                RETRY,
                "resource-cap-accepted",
                "one-under or zero resource cap unexpectedly admitted a complete run",
            );
        }
    };
    let typed = match resource {
        Some(expected) => {
            matches!(error.glyph_preparation_diagnostic(), Some(GlyphPreparationError::Limit {resource,required,limit})
            if *resource == expected && required > limit && *limit == low[match expected {
                GlyphResource::Glyphs=>0,GlyphResource::PathElements=>1,GlyphResource::ScratchBytes=>2,GlyphResource::RetainedBytes=>3
            }] as usize)
        }
        None => matches!(error.accounting_diagnostic(), Some(RenderAccountingError::Refused(r))
            if r.quantity()==RenderQuantity::Work(RenderWork::PathElementsEmitted)
                && matches!(r.reason(),RenderRefusalReason::Limit {limit} if limit==refused_work)),
    };
    if !typed
        || !report_clean(&context, RenderAttemptStatus::Refused)
        || context.last_attempt().is_none_or(|r| {
            r.usage().work(RenderWork::PreparedTransferredBytes) != 0 || r.refusal().is_none()
        })
    {
        return violated(
            RETRY,
            "resource-refusal-report",
            "resource refusal lost typed cause, accepted-prefix report or transient cleanup",
        );
    }
    if !previous.is_same_preparation(&retained) {
        return violated(
            RETRY,
            "prior-preparation-mutated",
            "refusal changed the caller's retained immutable prior preparation",
        );
    }
    if let Some(outcome) = check_output(
        g,
        &font,
        &instance,
        &glyphs,
        &previous,
        RETRY,
        "prior-preparation-mutated",
    )? {
        return Ok(outcome);
    }
    context.set_options(raised_options);
    let retried =
        match font.prepare_glyph_run(&instance, &glyphs, glyph_limits(g.limits), &mut context) {
            Ok(v) => v,
            Err(e) => return violated(RETRY, "raised-retry-refused", e.to_string()),
        };
    if let Some(outcome) = check_output(
        g,
        &font,
        &instance,
        &glyphs,
        &retried,
        RETRY,
        "raised-retry-content",
    )? {
        return Ok(outcome);
    }
    if !report_clean(&context, RenderAttemptStatus::Succeeded)
        || retried.is_same_preparation(&previous)
        || previous.path_elements().collect::<Vec<_>>()
            != retried.path_elements().collect::<Vec<_>>()
        || previous.bounds() != retried.bounds()
    {
        return violated(
            RETRY,
            "raised-retry-publication",
            "same-input raised retry lost content, identity, report or cleanup semantics",
        );
    }
    evaluated(&[FONT, GLYPH, RETRY])
}

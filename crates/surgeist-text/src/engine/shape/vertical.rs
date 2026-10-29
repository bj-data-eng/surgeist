//! Upright metric and provider adaptation for one actual shaping invocation.

use super::{PreparedRun, ResolvedStyle, RunMetricBasis, RunMetrics, RunRequest, logical_multiply};
use crate::{
    EffectiveFontInstance, Error, ErrorCode, ErrorDetail, FontBaselineScript, LineHeight, Result,
    TextNumericError, TextNumericPhase, UprightContent, UprightMetricReference,
    VerticalOriginReferenceReason, VerticalPositionStage, VerticalShapingContextReason,
};

pub(super) fn line_height(
    style: &ResolvedStyle,
    normal_extent_design: Option<f64>,
    design_scale: f64,
) -> Result<f64> {
    match style.line_height {
        LineHeight::Absolute(value) => Ok(value),
        LineHeight::FontSizeRelative(value) => {
            logical_multiply("line height", value, f64::from(style.font_size))
        }
        LineHeight::MetricsRelative(value) => {
            let normal = normal_extent_design.ok_or_else(|| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "upright line-height demand was not resolved",
                )
            })?;
            let normal = logical_multiply("line height", normal, design_scale)?;
            logical_multiply("line height", normal, value)
        }
    }
}

pub(super) fn resolved_run(
    request: &RunRequest<'_, '_>,
    instance: EffectiveFontInstance<'_>,
    coords: &[i16],
) -> Result<(PreparedRun, f64, f64)> {
    let input = request.input;
    let style = &input.styles[request.style].style;
    let script = baseline_script(request.script)?;
    let metrics = instance.unscaled_upright_metrics(input.font_catalogue, script)?;
    let scale = f64::from(style.font_size) / f64::from(metrics.units_per_em());
    let needs_normal = std::iter::once(style)
        .chain(
            request
                .atoms
                .clone()
                .map(|atom| &input.styles[input.analysis.infos[atom].1].style),
        )
        .any(|style| matches!(style.line_height, LineHeight::MetricsRelative(_)));
    let normal_extent_design = if needs_normal {
        let gap = instance.upright_line_gap(input.font_catalogue)?;
        Some(normal_extent(
            metrics.used_over(),
            metrics.used_under(),
            gap,
        )?)
    } else {
        None
    };
    let ascent = logical_multiply("upright cross before", metrics.used_over(), scale)?;
    let descent = logical_multiply("upright cross after", -metrics.used_under(), scale)?;
    let run = PreparedRun {
        font: request.font.clone(),
        font_size: style.font_size,
        font_attrs: request.attrs,
        synthesis: request.synthesis,
        coords: coords.to_vec(),
        style: request.style,
        level: request.level,
        script: request.script,
        atoms: request.atoms.clone(),
        metrics: RunMetrics {
            ascent,
            descent,
            line_height: line_height(style, normal_extent_design, scale)?,
            basis: RunMetricBasis::Upright {
                normal_extent_design,
                design_scale: scale,
            },
        },
    };
    // Upright half-leading uses the separately resolved normal extent; no
    // horizontal leading is transferred into this coordinate frame.
    Ok((run, scale, 0.0))
}

fn normal_extent(over: f64, under: f64, gap: f64) -> Result<f64> {
    let normal = over - under + gap;
    if !normal.is_finite() || normal < 0.0 {
        return Err(super::super::text_numeric_error(
            "upright normal extent",
            TextNumericPhase::Preparation,
            if normal.is_finite() {
                TextNumericError::OutOfRange
            } else {
                TextNumericError::NonFinite
            },
            normal,
        ));
    }
    Ok(normal)
}

fn baseline_script(script: icu_properties::props::Script) -> Result<FontBaselineScript> {
    FontBaselineScript::from_script(script).ok_or_else(|| {
        Error::new(
            ErrorCode::LayoutFailed,
            "actual shaping script has no canonical baseline label",
        )
    })
}

pub(super) fn shape(
    request: &RunRequest<'_, '_>,
    shaper: &surgeist_shaper::Shaper<'_>,
    plan: &surgeist_shaper::ShapePlan,
    buffer: surgeist_shaper::UnicodeBuffer,
    features: &[surgeist_shaper::Feature],
) -> Result<surgeist_shaper::VerticalGlyphBuffer> {
    let coords: Vec<_> = shaper
        .coords()
        .iter()
        .map(|coord| coord.to_bits())
        .collect();
    let instance = EffectiveFontInstance::qualify_synthesized(
        request.font.face(),
        request.input.styles[request.style].style.font_size,
        &coords,
        request.synthesis,
    )?;
    Operation {
        shaper,
        plan,
        features,
    }
    .shape_with_instance(
        &instance,
        request.input.font_catalogue,
        request.script,
        || request.input.upright_content(request.atoms.clone()),
        buffer,
    )
}

/// A provider operation retains the plan together with its actual feature input.
pub(super) struct Operation<'call, 'font> {
    pub(super) shaper: &'call surgeist_shaper::Shaper<'font>,
    pub(super) plan: &'call surgeist_shaper::ShapePlan,
    pub(super) features: &'call [surgeist_shaper::Feature],
}

impl Operation<'_, '_> {
    pub(super) fn shape_with_instance(
        self,
        instance: &EffectiveFontInstance<'_>,
        catalogue: &crate::FontCatalogueIdentity,
        script: icu_properties::props::Script,
        content: impl FnOnce() -> UprightContent,
        buffer: surgeist_shaper::UnicodeBuffer,
    ) -> Result<surgeist_shaper::VerticalGlyphBuffer> {
        let Self {
            shaper,
            plan,
            features,
        } = self;
        let metrics = instance.unscaled_upright_metrics(catalogue, baseline_script(script)?)?;
        let value = metrics.central_coordinate_or_distance();
        let reference = metrics.reference();
        let encoded_reference = match reference {
            UprightMetricReference::FontDesignX => {
                surgeist_shaper::VerticalOriginReference::try_font_design_x(value)
            }
            UprightMetricReference::NominalVerticalPenX => {
                surgeist_shaper::VerticalOriginReference::try_nominal_vertical_pen_x(value)
            }
        };
        let provider_reference = match encoded_reference {
            Ok(reference) => reference,
            Err(reason) => {
                let (code, reason) = match reason {
                    surgeist_shaper::VerticalOriginReferenceError::NonFinite => (
                        ErrorCode::InvalidTextNumeric,
                        VerticalOriginReferenceReason::NonFinite,
                    ),
                    surgeist_shaper::VerticalOriginReferenceError::OutOfRange => (
                        ErrorCode::InvalidTextNumeric,
                        VerticalOriginReferenceReason::OutOfRange,
                    ),
                    surgeist_shaper::VerticalOriginReferenceError::NotHalfUnit => (
                        ErrorCode::UnsupportedFeature,
                        VerticalOriginReferenceReason::NotHalfUnit,
                    ),
                };
                return Err(Error::new(
                    code,
                    "upright reference cannot be encoded by the shaping provider",
                )
                .with_detail(ErrorDetail::VerticalOriginReference {
                    content: content(),
                    reference,
                    value,
                    reason,
                }));
            }
        };
        #[cfg(test)]
        super::observe_provider_call();
        shaper
            .shape_vertical_with_metrics(plan, buffer, features, provider_reference, |glyph_id| {
                let nominal = instance.unscaled_vertical_glyph_metrics(glyph_id, catalogue)?;
                let advance = exact_integer(
                    "vertical nominal advance",
                    nominal.advance(),
                    0.0,
                    f64::from(u32::MAX),
                )?;
                let origin_x_twice = exact_integer(
                    "vertical nominal origin x twice",
                    nominal.origin_x() * 2.0,
                    f64::from(i32::MIN),
                    f64::from(i32::MAX),
                )?;
                let origin_y = exact_integer(
                    "vertical nominal origin y",
                    nominal.origin_y(),
                    f64::from(i32::MIN),
                    f64::from(i32::MAX),
                )?;
                Ok(surgeist_shaper::VerticalNominalMetrics::new(
                    advance as u32,
                    origin_x_twice as i32,
                    origin_y as i32,
                ))
            })
            .map_err(|error| provider_error(error, content()))
    }
}

fn exact_integer(field: &'static str, value: f64, min: f64, max: f64) -> Result<f64> {
    if !value.is_finite() || value < min || value > max || value.fract() != 0.0 {
        return Err(super::super::text_numeric_error(
            field,
            TextNumericPhase::Shaping,
            if value.is_finite() {
                TextNumericError::OutOfRange
            } else {
                TextNumericError::NonFinite
            },
            value,
        ));
    }
    Ok(value)
}

fn provider_error(
    error: surgeist_shaper::VerticalShapeError<Error>,
    content: UprightContent,
) -> Error {
    use surgeist_shaper::{
        VerticalPositionStage as Stage, VerticalShapeContextError as Context,
        VerticalShapeError as Provider,
    };
    match error {
        Provider::Context(reason) => {
            let reason = match reason {
                Context::Direction => VerticalShapingContextReason::Direction,
                Context::PlanDirection => VerticalShapingContextReason::PlanDirection,
                Context::PlanScript => VerticalShapingContextReason::PlanScript,
                Context::Morx => VerticalShapingContextReason::Morx,
                Context::Kerx => VerticalShapingContextReason::Kerx,
                Context::Kern => VerticalShapingContextReason::Kern,
                Context::Trak => VerticalShapingContextReason::Trak,
            };
            Error::new(
                ErrorCode::ShapingRefused,
                "unsupported actual upright shaping context",
            )
            .with_detail(ErrorDetail::VerticalShapingContext { content, reason })
        }
        Provider::Supplier { glyph_id, error } => {
            let mut outer = Error::new(error.code, "actual upright nominal metric supplier failed")
                .with_detail(ErrorDetail::VerticalMetricSupplier { content, glyph_id });
            outer.source = Some(Box::new(error));
            outer
        }
        Provider::CoordinateRange { stage } => {
            let stage = match stage {
                Stage::Nominal => VerticalPositionStage::Nominal,
                Stage::Gpos => VerticalPositionStage::Gpos,
                Stage::Attachment => VerticalPositionStage::Attachment,
                Stage::Fallback => VerticalPositionStage::Fallback,
            };
            Error::new(
                ErrorCode::ShapingRefused,
                "actual upright positioning exceeded provider range",
            )
            .with_detail(ErrorDetail::VerticalPositionRange { content, stage })
        }
        Provider::Refused(reason) => super::super::shaping_refused(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::normal_extent;
    use crate::{ErrorCode, ErrorDetail, TextNumericError, TextNumericPhase};

    #[test]
    fn signed_vertical_gap_changes_normal_extent_without_moving_em_sides() {
        assert_eq!(normal_extent(500.0, -500.0, 200.0).unwrap(), 1200.0);
        assert_eq!(normal_extent(500.0, -500.0, -200.0).unwrap(), 800.0);
    }

    #[test]
    fn negative_raw_normal_refuses_before_a_factor_could_erase_its_sign() {
        let negative = -f64::from_bits(1);
        assert_eq!(negative * f64::from_bits(1), -0.0);
        let error = normal_extent(negative, 0.0, 0.0).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
        assert!(
            matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
            field: "upright normal extent",
            phase: TextNumericPhase::Preparation,
            category: TextNumericError::OutOfRange,
            value,
        }) if *value == negative)
        );
    }

    #[test]
    fn nonfinite_derived_normal_preserves_its_preparation_failure() {
        let error = normal_extent(f64::MAX, -f64::MAX, 0.0).unwrap_err();
        assert!(
            matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
            field: "upright normal extent",
            phase: TextNumericPhase::Preparation,
            category: TextNumericError::NonFinite,
            value,
        }) if value.is_infinite())
        );
    }
}

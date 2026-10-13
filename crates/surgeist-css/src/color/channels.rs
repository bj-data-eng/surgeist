//! Fixed intrinsic channel resolution after the canonical numeric grammar.
//!
//! The caller admits the three-channel kernel. No token, input, fact array or
//! variadic argument is observed here; those continuations supply typed values.

use super::keywords::ColorFunction as F;
use super::{CssUsedColor, CssUsedColorError};
use crate::{
    CssColorInterpolationSpace as Space, CssColorSyntax as Syntax,
    CssPredefinedColorSpace as Predefined,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Domain {
    Number,
    Percentage,
    Degrees,
    Missing,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Component {
    pub(crate) domain: Domain,
    pub(crate) value: f64,
}
impl Component {
    pub(crate) const MISSING: Self = Self {
        domain: Domain::Missing,
        value: 0.0,
    };
    pub(crate) const fn number(value: f64) -> Self {
        Self {
            domain: Domain::Number,
            value,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Grammar,
    Numerical(CssUsedColorError),
}
pub(crate) fn legacy_rgb_domains(domains: [Domain; 3]) -> bool {
    matches!(domains[0], Domain::Number | Domain::Percentage)
        && domains[1] == domains[0]
        && domains[2] == domains[0]
}
impl From<CssUsedColorError> for Error {
    fn from(value: CssUsedColorError) -> Self {
        Self::Numerical(value)
    }
}

pub(crate) fn scalar(component: Component, percentage: f64) -> Result<Option<f64>, Error> {
    if component.domain == Domain::Missing {
        return Ok(None);
    }
    if component.domain == Domain::Degrees {
        return Err(Error::Grammar);
    }
    // CSS Values calculation NaN is censored to zero at the calculation root.
    let value = if component.value.is_nan() {
        0.0
    } else {
        component.value
    };
    Ok(Some(if component.domain == Domain::Percentage {
        value / 100.0 * percentage
    } else {
        value
    }))
}
pub(crate) fn hue(component: Component) -> Result<Option<f64>, Error> {
    match component.domain {
        Domain::Missing => Ok(None),
        Domain::Percentage => Err(Error::Grammar),
        Domain::Degrees | Domain::Number => Ok(Some(if component.value.is_finite() {
            component.value.rem_euclid(360.0)
        } else {
            0.0
        })),
    }
}
fn clamp(value: Option<f64>, maximum: f64) -> Option<f64> {
    value.map(|value| {
        if value.is_nan() {
            0.0
        } else {
            value.clamp(0.0, maximum)
        }
    })
}
pub(crate) fn alpha(value: Option<Component>, legacy: bool) -> Result<Option<f64>, Error> {
    if legacy && value.is_some_and(|value| value.domain == Domain::Missing) {
        return Err(Error::Grammar);
    }
    Ok(clamp(
        value.map_or(Ok(Some(1.0)), |value| scalar(value, 1.0))?,
        1.0,
    ))
}

/// Function-space coordinates remain available to same-space relative syntax
/// before the final Color4 used-value conversion.
pub(crate) fn absolute_coordinates(
    function: F,
    predefined: Option<Predefined>,
    syntax: Syntax,
    components: [Component; 3],
    alpha_component: Option<Component>,
) -> Result<CssUsedColor, Error> {
    let legacy = syntax == Syntax::Legacy;
    if legacy && !matches!(function, F::Rgb | F::Hsl) {
        return Err(Error::Grammar);
    }
    if legacy
        && components
            .iter()
            .any(|value| value.domain == Domain::Missing)
    {
        return Err(Error::Grammar);
    }
    let alpha = alpha(alpha_component, legacy)?;
    let (space, channels) = match function {
        F::Rgb => {
            if legacy && !legacy_rgb_domains(components.map(|value| value.domain)) {
                return Err(Error::Grammar);
            }
            let channel = |value: Component| -> Result<Option<f64>, Error> {
                if value.domain == Domain::Missing {
                    return Ok(None);
                }
                if value.domain == Domain::Degrees {
                    return Err(Error::Grammar);
                }
                let value_number = if value.value.is_nan() {
                    0.0
                } else {
                    value.value
                };
                Ok(Some(
                    (value_number
                        / if value.domain == Domain::Percentage {
                            100.0
                        } else {
                            255.0
                        })
                    .clamp(0.0, 1.0),
                ))
            };
            (
                Space::Predefined(Predefined::Srgb),
                [
                    channel(components[0])?,
                    channel(components[1])?,
                    channel(components[2])?,
                ],
            )
        }
        F::Hsl | F::Hwb => {
            if legacy
                && (components[1].domain != Domain::Percentage
                    || components[2].domain != Domain::Percentage)
            {
                return Err(Error::Grammar);
            }
            let saturation = scalar(components[1], 100.0)?;
            let third = scalar(components[2], 100.0)?;
            let second = if function == F::Hsl {
                saturation.map(|value| value.max(0.0))
            } else {
                saturation
            };
            (
                if function == F::Hsl {
                    Space::Hsl
                } else {
                    Space::Hwb
                },
                [hue(components[0])?, second, third],
            )
        }
        F::Lab | F::Oklab => {
            let ok = function == F::Oklab;
            (
                if ok { Space::Oklab } else { Space::Lab },
                [
                    clamp(
                        scalar(components[0], if ok { 1.0 } else { 100.0 })?,
                        if ok { 1.0 } else { 100.0 },
                    ),
                    scalar(components[1], if ok { 0.4 } else { 125.0 })?,
                    scalar(components[2], if ok { 0.4 } else { 125.0 })?,
                ],
            )
        }
        F::Lch | F::Oklch => {
            let ok = function == F::Oklch;
            (
                if ok { Space::Oklch } else { Space::Lch },
                [
                    clamp(
                        scalar(components[0], if ok { 1.0 } else { 100.0 })?,
                        if ok { 1.0 } else { 100.0 },
                    ),
                    scalar(components[1], if ok { 0.4 } else { 150.0 })?
                        .map(|value| value.max(0.0)),
                    hue(components[2])?,
                ],
            )
        }
        F::Color => (
            Space::Predefined(predefined.ok_or(Error::Grammar)?),
            [
                scalar(components[0], 1.0)?,
                scalar(components[1], 1.0)?,
                scalar(components[2], 1.0)?,
            ],
        ),
        _ => return Err(Error::Grammar),
    };
    let value = CssUsedColor::try_new(space, channels, alpha)?;
    // Color4 16.2.2 retains the absolute rgb()/rgba() serialization family.
    // Missing channels still select color(srgb ...) in the canonical serializer;
    // relative RGB is a separate Color5 color-function family.
    Ok(if function == F::Rgb {
        value.with_legacy_srgb()
    } else {
        value
    })
}
#[cfg(test)]
pub(crate) fn absolute(
    function: F,
    predefined: Option<Predefined>,
    syntax: Syntax,
    components: [Component; 3],
    alpha_component: Option<Component>,
) -> Result<CssUsedColor, Error> {
    let value = absolute_coordinates(function, predefined, syntax, components, alpha_component)?;
    let alpha = value.alpha();
    let channels = value.channels();
    // Color4§15.1 resolves present HSL/HWB channels to corresponding sRGB.
    // Explicit missing channels instead retain their modern natural form.
    if matches!(function, F::Hsl | F::Hwb) && channels.iter().all(Option::is_some) {
        let converted = value.convert(Space::Predefined(Predefined::Srgb))?;
        CssUsedColor::try_new(
            converted.space(),
            converted.channels().map(|value| clamp(value, 1.0)),
            alpha,
        )
        .map(CssUsedColor::with_legacy_srgb)
        .map_err(Into::into)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn percent(value: f64) -> Component {
        Component {
            domain: Domain::Percentage,
            value,
        }
    }
    fn degrees(value: f64) -> Component {
        Component {
            domain: Domain::Degrees,
            value,
        }
    }
    #[test]
    fn literal_reference_ranges_clamping_and_extended_values_are_distinct() {
        let rgb = absolute(
            F::Rgb,
            None,
            Syntax::Modern,
            [
                percent(100.0),
                Component::number(-1.0),
                Component::number(128.0),
            ],
            Some(percent(50.0)),
        )
        .unwrap();
        assert_eq!(rgb.channels(), [Some(1.0), Some(0.0), Some(128.0 / 255.0)]);
        assert_eq!(rgb.alpha(), Some(0.5));
        assert_eq!(rgb.srgb_bytes(), None);
        let lab = absolute(
            F::Lab,
            None,
            Syntax::Modern,
            [percent(120.0), percent(100.0), percent(-100.0)],
            None,
        )
        .unwrap();
        assert_eq!(lab.channels(), [Some(100.0), Some(125.0), Some(-125.0)]);
        let ok = absolute(
            F::Oklch,
            None,
            Syntax::Modern,
            [percent(50.0), percent(100.0), degrees(450.0)],
            None,
        )
        .unwrap();
        assert_eq!(ok.channels(), [Some(0.5), Some(0.4), Some(90.0)]);
        let wide = absolute(
            F::Color,
            Some(Predefined::DisplayP3),
            Syntax::Modern,
            [Component::number(1.5), percent(-50.0), Component::MISSING],
            Some(Component::MISSING),
        )
        .unwrap();
        assert_eq!(wide.channels(), [Some(1.5), Some(-0.5), None]);
        assert_eq!(wide.alpha(), None);
    }
    #[test]
    fn legacy_domain_failures_are_not_numerical_failures() {
        assert_eq!(
            absolute(
                F::Rgb,
                None,
                Syntax::Legacy,
                [Component::number(1.0), percent(2.0), percent(3.0)],
                None
            ),
            Err(Error::Grammar)
        );
        assert_eq!(
            absolute(
                F::Hsl,
                None,
                Syntax::Legacy,
                [degrees(0.0), Component::number(100.0), percent(50.0)],
                None
            ),
            Err(Error::Grammar)
        );
        assert_eq!(
            absolute(
                F::Lab,
                None,
                Syntax::Modern,
                [
                    Component::number(50.0),
                    Component::number(f64::INFINITY),
                    Component::number(0.0)
                ],
                None
            ),
            Err(Error::Numerical(CssUsedColorError::NonFinite {
                component: 1
            }))
        );
        let rgb = absolute(
            F::Rgb,
            None,
            Syntax::Modern,
            [
                Component::number(f64::INFINITY),
                Component::number(f64::NAN),
                Component::number(f64::NEG_INFINITY),
            ],
            None,
        )
        .unwrap();
        assert_eq!(rgb.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
    }
    #[test]
    fn present_hsl_resolves_srgb_but_missing_channels_preserve_natural_form() {
        let red = absolute(
            F::Hsl,
            None,
            Syntax::Modern,
            [degrees(360.0), percent(100.0), percent(50.0)],
            None,
        )
        .unwrap();
        assert_eq!(red.space(), Space::Predefined(Predefined::Srgb));
        assert_eq!(red.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
        assert!(!red.uses_color_function());
        let missing = absolute(
            F::Hsl,
            None,
            Syntax::Modern,
            [Component::MISSING, percent(0.0), percent(50.0)],
            None,
        )
        .unwrap();
        assert_eq!(missing.space(), Space::Hsl);
        assert_eq!(missing.channels(), [None, Some(0.0), Some(50.0)]);
    }
}

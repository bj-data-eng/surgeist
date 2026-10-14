//! Fixed relative component binding and extended used-value resolution.

use super::Color;
use super::channels::{self, Component, Error};
use super::keywords::ColorFunction as F;
use crate::{
    ColorSpace as Space, CssRelativeColorChannel as Channel,
    CssRelativeColorEnvironment as Environment, PredefinedColorSpace as Predefined,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Binding {
    environment: Environment,
    origin: Color,
}
impl Binding {
    pub(crate) fn identical(self, other: Self) -> bool {
        self.environment == other.environment && self.origin.identical(other.origin)
    }
    pub(crate) const fn environment(self) -> Environment {
        self.environment
    }
    pub(crate) fn new(environment: Environment, origin: Color) -> Result<Self, Error> {
        if !environment.is_consistent() {
            return Err(Error::Grammar);
        }
        let space = match environment {
            Environment::Alpha => origin.space(),
            Environment::Rgb => Space::Predefined(Predefined::Srgb),
            Environment::Hsl => Space::Hsl,
            Environment::Hwb => Space::Hwb,
            Environment::Lab => Space::Lab,
            Environment::Lch => Space::Lch,
            Environment::Oklab => Space::Oklab,
            Environment::Oklch => Space::Oklch,
            Environment::PredefinedRgb(space) | Environment::Xyz(space) => Space::Predefined(space),
        };
        let origin = origin
            .prepared_for_interpolation(space)
            .map_err(Error::Numerical)?;
        Ok(Self {
            environment,
            origin,
        })
    }
    #[cfg(test)]
    pub(crate) const fn origin(self) -> Color {
        self.origin
    }
    pub(crate) fn channel(self, channel: Channel, calculation: bool) -> Result<Component, Error> {
        let value = if channel == Channel::Alpha {
            self.origin.alpha()
        } else {
            let names = match self.environment {
                Environment::Alpha => return Err(Error::Grammar),
                Environment::Rgb | Environment::PredefinedRgb(_) => {
                    [Channel::R, Channel::G, Channel::B]
                }
                Environment::Hsl => [Channel::H, Channel::S, Channel::L],
                Environment::Hwb => [Channel::H, Channel::W, Channel::B],
                Environment::Lab | Environment::Oklab => [Channel::L, Channel::A, Channel::B],
                Environment::Lch | Environment::Oklch => [Channel::L, Channel::C, Channel::H],
                Environment::Xyz(_) => [Channel::X, Channel::Y, Channel::Z],
            };
            let index = names
                .iter()
                .position(|value| *value == channel)
                .ok_or(Error::Grammar)?;
            self.origin.channels()[index].map(|value| {
                if self.environment == Environment::Rgb {
                    value * 255.0
                } else {
                    value
                }
            })
        };
        Ok(if let Some(value) = value {
            Component::number(value)
        } else if calculation {
            Component::number(0.0)
        } else {
            Component::MISSING
        })
    }
}

pub(crate) fn coordinates(
    function: F,
    predefined: Option<Predefined>,
    origin: Color,
    components: [Component; 3],
    alpha: Option<Component>,
) -> Result<Color, Error> {
    let alpha = if let Some(alpha) = alpha {
        channels::alpha(Some(alpha), false)?
    } else {
        origin.alpha()
    };
    let (space, channels) = match function {
        F::Rgb => {
            let channel = |value: Component| -> Result<Option<f64>, Error> {
                Ok(channels::scalar(value, 255.0)?.map(|value| value / 255.0))
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
        F::Hsl | F::Hwb => (
            if function == F::Hsl {
                Space::Hsl
            } else {
                Space::Hwb
            },
            [
                channels::hue(components[0])?,
                channels::scalar(components[1], 100.0)?,
                channels::scalar(components[2], 100.0)?,
            ],
        ),
        F::Lab | F::Oklab => {
            let ok = function == F::Oklab;
            (
                if ok { Space::Oklab } else { Space::Lab },
                [
                    channels::scalar(components[0], if ok { 1.0 } else { 100.0 })?,
                    channels::scalar(components[1], if ok { 0.4 } else { 125.0 })?,
                    channels::scalar(components[2], if ok { 0.4 } else { 125.0 })?,
                ],
            )
        }
        F::Lch | F::Oklch => {
            let ok = function == F::Oklch;
            (
                if ok { Space::Oklch } else { Space::Lch },
                [
                    channels::scalar(components[0], if ok { 1.0 } else { 100.0 })?,
                    channels::scalar(components[1], if ok { 0.4 } else { 150.0 })?,
                    channels::hue(components[2])?,
                ],
            )
        }
        F::Color => (
            Space::Predefined(predefined.ok_or(Error::Grammar)?),
            [
                channels::scalar(components[0], 1.0)?,
                channels::scalar(components[1], 1.0)?,
                channels::scalar(components[2], 1.0)?,
            ],
        ),
        _ => return Err(Error::Grammar),
    };
    // Relative color results use Color5 serialization, including complete
    // HSL/HWB results lowered to color(srgb) by the serialization owner.
    Ok(Color::try_new(space, channels, alpha)?
        .with_serialization_family(surgeist_color::SerializationFamily::ColorFunction)?)
}
#[cfg(test)]
pub(crate) fn resolve(
    function: F,
    predefined: Option<Predefined>,
    origin: Color,
    components: [Component; 3],
    alpha: Option<Component>,
) -> Result<Color, Error> {
    let value = coordinates(function, predefined, origin, components, alpha)?;
    let alpha = value.alpha();
    let channels = value.channels();
    // Relative HSL/HWB with any missing component, including alpha, retain
    // natural form. Present results resolve to extended sRGB without clipping.
    if matches!(function, F::Hsl | F::Hwb)
        && channels.iter().all(Option::is_some)
        && alpha.is_some()
    {
        Ok(value
            .converted(Space::Predefined(Predefined::Srgb))?
            .with_serialization_family(surgeist_color::SerializationFamily::ColorFunction)?)
    } else {
        Ok(value)
    }
}

pub(crate) fn alpha_only(origin: Color, component: Option<Component>) -> Result<Color, Error> {
    let Some(component) = component else {
        return Ok(origin);
    };
    Ok(origin.with_alpha(channels::alpha(Some(component), false)?)?)
}

#[cfg(test)]
mod tests {
    use super::super::channels::Domain;
    use super::*;
    const SRGB: Space = Space::Predefined(Predefined::Srgb);
    fn percent(value: f64) -> Component {
        Component {
            domain: Domain::Percentage,
            value,
        }
    }
    #[test]
    fn binding_numbers_have_environment_units_and_missing_is_zero_only_in_math() {
        let bytes = Color::from_srgba8([255, 128, 0, 128]);
        let rgb = Binding::new(Environment::Rgb, bytes).unwrap();
        assert_eq!(rgb.channel(Channel::G, false), Ok(Component::number(128.0)));
        let predefined = Binding::new(Environment::PredefinedRgb(Predefined::Srgb), bytes).unwrap();
        assert_eq!(
            predefined.channel(Channel::G, false),
            Ok(Component::number(128.0 / 255.0))
        );
        assert_eq!(rgb.channel(Channel::H, false), Err(Error::Grammar));
        let missing = Color::try_new(Space::Oklch, [Some(0.526), Some(0.115), None], None).unwrap();
        let binding = Binding::new(Environment::Oklch, missing).unwrap();
        assert_eq!(binding.channel(Channel::H, false), Ok(Component::MISSING));
        assert_eq!(
            binding.channel(Channel::H, true),
            Ok(Component::number(0.0))
        );
        assert_eq!(
            binding.channel(Channel::Alpha, false),
            Ok(Component::MISSING)
        );
        assert_eq!(
            binding.channel(Channel::Alpha, true),
            Ok(Component::number(0.0))
        );
        assert_eq!(binding.origin(), missing);
        assert!(Binding::new(Environment::Xyz(Predefined::Srgb), bytes).is_err());
    }
    #[test]
    fn relative_channels_are_extended_and_omitted_alpha_comes_from_origin() {
        let origin = Color::try_new(
            Space::Oklab,
            [Some(0.543), Some(-0.09), Some(-0.02)],
            Some(0.3),
        )
        .unwrap();
        let modified = resolve(
            F::Oklab,
            None,
            origin,
            [
                Component::number(1.0 - 0.543),
                Component::number(-0.09 * 0.8),
                Component::number(-0.02),
            ],
            None,
        )
        .unwrap();
        assert!((modified.channels()[0].unwrap() - 0.457).abs() < 1e-12);
        assert_eq!(modified.channels()[1..], [Some(-0.072), Some(-0.02)]);
        assert_eq!(modified.alpha(), Some(0.3));
        let extended = resolve(
            F::Rgb,
            None,
            origin,
            [
                Component::number(510.0),
                Component::number(-255.0),
                percent(150.0),
            ],
            Some(percent(200.0)),
        )
        .unwrap();
        assert_eq!(extended.channels(), [Some(2.0), Some(-1.0), Some(1.5)]);
        assert_eq!(extended.alpha(), Some(1.0));
        assert_eq!(extended.srgb_bytes(), None);
        let light = resolve(
            F::Lab,
            None,
            origin,
            [percent(120.0), percent(200.0), percent(-200.0)],
            None,
        )
        .unwrap();
        assert_eq!(light.channels(), [Some(120.0), Some(250.0), Some(-250.0)]);
    }
    #[test]
    fn signed_polar_and_hsl_values_keep_history_without_changing_converter_types() {
        let origin = Color::opaque_black();
        let signed = resolve(
            F::Lch,
            None,
            origin,
            [
                Component::number(50.0),
                Component::number(-20.0),
                Component::number(0.0),
            ],
            None,
        )
        .unwrap();
        assert_eq!(signed.channels(), [Some(50.0), Some(-20.0), Some(0.0)]);
        let cartesian = signed.converted(Space::Lab).unwrap().channels();
        assert_eq!(cartesian[0], Some(50.0));
        assert_eq!(cartesian[1], Some(-20.0));
        assert!(cartesian[2].unwrap().abs() < 1e-12);
        let cyan = resolve(
            F::Hsl,
            None,
            origin,
            [
                Component::number(0.0),
                Component::number(-100.0),
                Component::number(50.0),
            ],
            None,
        )
        .unwrap();
        assert_eq!(cyan.space(), SRGB);
        assert_eq!(cyan.channels(), [Some(0.0), Some(1.0), Some(1.0)]);
        let missing = resolve(
            F::Hsl,
            None,
            origin,
            [
                Component::MISSING,
                Component::number(-100.0),
                Component::number(50.0),
            ],
            Some(Component::MISSING),
        )
        .unwrap();
        assert_eq!(missing.space(), Space::Hsl);
        assert_eq!(missing.channels(), [None, Some(-100.0), Some(50.0)]);
        assert_eq!(missing.alpha(), None);
    }
    #[test]
    fn alpha_only_retains_origin_channels_and_omission_is_lossless() {
        let origin = Color::from_srgba8([1, 2, 3, 237]);
        assert_eq!(alpha_only(origin, None), Ok(origin));
        let result = alpha_only(origin, Some(percent(80.0))).unwrap();
        assert_eq!(result.srgb_bytes(), Some([1, 2, 3]));
        assert_eq!(result.alpha(), Some(0.8));
        assert_eq!(result.alpha_byte(), None);
        assert_eq!(
            alpha_only(
                origin,
                Some(Component {
                    domain: Domain::Degrees,
                    value: 1.0
                })
            ),
            Err(Error::Grammar)
        );
    }
    #[test]
    fn same_function_origin_retains_actual_hue_and_extended_coordinates_until_used_boundary() {
        let natural = channels::absolute_coordinates(
            F::Hsl,
            None,
            crate::CssColorSyntax::Modern,
            [Component::number(130.0), percent(0.0), percent(50.0)],
            None,
        )
        .unwrap();
        assert_eq!(natural.space(), Space::Hsl);
        assert_eq!(natural.channels(), [Some(130.0), Some(0.0), Some(50.0)]);
        let binding = Binding::new(Environment::Hsl, natural).unwrap();
        assert_eq!(
            binding.channel(Channel::H, false),
            Ok(Component::number(130.0))
        );
        let extended = coordinates(
            F::Hsl,
            None,
            natural,
            [
                Component::number(30.0),
                Component::number(-100.0),
                Component::number(150.0),
            ],
            None,
        )
        .unwrap();
        assert_eq!(extended.channels(), [Some(30.0), Some(-100.0), Some(150.0)]);
        let next = Binding::new(Environment::Hsl, extended).unwrap();
        assert_eq!(
            next.channel(Channel::S, false),
            Ok(Component::number(-100.0))
        );
        assert_eq!(
            next.channel(Channel::L, false),
            Ok(Component::number(150.0))
        );
        let gray = resolve(
            F::Hsl,
            None,
            natural,
            [
                Component::number(130.0),
                Component::number(0.0),
                Component::number(50.0),
            ],
            None,
        )
        .unwrap();
        assert_eq!(gray.space(), SRGB);
        assert_eq!(gray.channels(), [Some(0.5); 3]);
    }
}

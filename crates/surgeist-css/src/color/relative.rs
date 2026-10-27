//! Fixed relative component binding and extended used-value resolution.

use super::channels::{self, Component, Error};
use super::keywords::ColorFunction as F;
use crate::color::CssRelativeColorChannel as Channel;
use crate::color::CssRelativeColorEnvironment as Environment;
use surgeist_color::Color;
use surgeist_color::ColorSpace as Space;
use surgeist_color::PredefinedColorSpace as Predefined;

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
        let origin = origin.prepared_for_interpolation(space);
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
            // An rgb() channel keyword is the origin channel in 0..255 units;
            // a product beyond binary64 is the closest supported value.
            self.origin.channels()[index].map(|value| {
                if self.environment == Environment::Rgb {
                    channels::supported(value * 255.0).get()
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
        origin.unit_alpha()
    };
    let space = channels::function_space(function, predefined)?;
    let channels = channels::relative_channels(function, components)?;
    // Relative color results use Color5 serialization, including complete
    // HSL/HWB results lowered to color(srgb) by the serialization owner.
    Ok(Color::new(space, channels, alpha).with_color_function())
}
pub(crate) fn alpha_only(origin: Color, component: Option<Component>) -> Result<Color, Error> {
    let Some(component) = component else {
        return Ok(origin);
    };
    Ok(origin.with_alpha(channels::alpha(Some(component), false)?))
}

#[cfg(test)]
mod tests {
    use super::super::channels::Domain;
    use super::*;
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
        let missing = Color::new(
            Space::Oklch,
            [
                surgeist_color::Finite::new(0.526),
                surgeist_color::Finite::new(0.115),
                None,
            ],
            None,
        );
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
            crate::color::CssColorSyntax::Modern,
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
    }
}

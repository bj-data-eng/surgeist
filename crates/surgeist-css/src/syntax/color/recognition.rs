//! Decoded color names and relative channel environment recognition.

use super::{
    CssCalculationType, CssPredefinedColorSpace, CssRelativeColorChannel,
    CssRelativeColorEnvironment,
};
use cssparser::match_ignore_ascii_case;

/// Token admission, extensions, origins and failures belong to each caller.
pub(crate) fn predefined_color_space(name: &str) -> Option<CssPredefinedColorSpace> {
    Some(match_ignore_ascii_case! { name,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
        "display-p3-linear" => CssPredefinedColorSpace::DisplayP3Linear,
        "a98-rgb" => CssPredefinedColorSpace::A98Rgb,
        "prophoto-rgb" => CssPredefinedColorSpace::ProphotoRgb,
        "rec2020" => CssPredefinedColorSpace::Rec2020,
        "xyz" | "xyz-d65" => CssPredefinedColorSpace::XyzD65,
        "xyz-d50" => CssPredefinedColorSpace::XyzD50,
        _ => return None,
    })
}

pub(crate) fn numeric_relative_channel(
    environment: CssRelativeColorEnvironment,
    name: &str,
) -> Option<(CssRelativeColorChannel, CssCalculationType)> {
    let channel = relative_color_channel(environment, name)?;
    Some((channel, relative_channel_type(environment, channel)))
}

fn relative_color_channel(
    environment: CssRelativeColorEnvironment,
    ident: &str,
) -> Option<CssRelativeColorChannel> {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    let channel = match environment {
        CssRelativeColorEnvironment::Alpha => {
            if ident.eq_ignore_ascii_case("alpha") {
                Alpha
            } else {
                return None;
            }
        }
        CssRelativeColorEnvironment::Rgb | CssRelativeColorEnvironment::PredefinedRgb(_) => {
            match_ignore_ascii_case! { ident,
                "r" => R,
                "g" => G,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Hsl => match_ignore_ascii_case! { ident,
            "h" => H,
            "s" => S,
            "l" => L,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Hwb => match_ignore_ascii_case! { ident,
            "h" => H,
            "w" => W,
            "b" => B,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Lab | CssRelativeColorEnvironment::Oklab => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "a" => A,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Lch | CssRelativeColorEnvironment::Oklch => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "c" => C,
                "h" => H,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Xyz(_) => match_ignore_ascii_case! { ident,
            "x" => X,
            "y" => Y,
            "z" => Z,
            "alpha" => Alpha,
            _ => return None,
        },
    };
    Some(channel)
}

fn relative_channel_type(
    _environment: CssRelativeColorEnvironment,
    _channel: CssRelativeColorChannel,
) -> CssCalculationType {
    CssCalculationType::Number
}

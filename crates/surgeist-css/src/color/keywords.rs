//! Shared allocation-free decoded keyword recognition.

use crate::color::CssSystemColor;
use surgeist_css_parser::match_ignore_ascii_case;

pub(crate) fn system_color(ident: &str) -> Option<CssSystemColor> {
    let value = match_ignore_ascii_case! { ident,
        "canvas" => CssSystemColor::Canvas,
        "canvastext" => CssSystemColor::CanvasText,
        "linktext" => CssSystemColor::LinkText,
        "visitedtext" => CssSystemColor::VisitedText,
        "activetext" => CssSystemColor::ActiveText,
        "buttonface" => CssSystemColor::ButtonFace,
        "buttontext" => CssSystemColor::ButtonText,
        "buttonborder" => CssSystemColor::ButtonBorder,
        "field" => CssSystemColor::Field,
        "fieldtext" => CssSystemColor::FieldText,
        "highlight" => CssSystemColor::Highlight,
        "highlighttext" => CssSystemColor::HighlightText,
        "mark" => CssSystemColor::Mark,
        "marktext" => CssSystemColor::MarkText,
        "graytext" => CssSystemColor::GrayText,
        "selecteditem" => CssSystemColor::SelectedItem,
        "selecteditemtext" => CssSystemColor::SelectedItemText,
        "accentcolor" => CssSystemColor::AccentColor,
        "accentcolortext" => CssSystemColor::AccentColorText,
        "activeborder" => CssSystemColor::ActiveBorder,
        "activecaption" => CssSystemColor::ActiveCaption,
        "appworkspace" => CssSystemColor::AppWorkspace,
        "background" => CssSystemColor::Background,
        "buttonhighlight" => CssSystemColor::ButtonHighlight,
        "buttonshadow" => CssSystemColor::ButtonShadow,
        "captiontext" => CssSystemColor::CaptionText,
        "inactiveborder" => CssSystemColor::InactiveBorder,
        "inactivecaption" => CssSystemColor::InactiveCaption,
        "inactivecaptiontext" => CssSystemColor::InactiveCaptionText,
        "infobackground" => CssSystemColor::InfoBackground,
        "infotext" => CssSystemColor::InfoText,
        "menu" => CssSystemColor::Menu,
        "menutext" => CssSystemColor::MenuText,
        "scrollbar" => CssSystemColor::Scrollbar,
        "threeddarkshadow" => CssSystemColor::ThreeDDarkShadow,
        "threedface" => CssSystemColor::ThreeDFace,
        "threedhighlight" => CssSystemColor::ThreeDHighlight,
        "threedlightshadow" => CssSystemColor::ThreeDLightShadow,
        "threedshadow" => CssSystemColor::ThreeDShadow,
        "window" => CssSystemColor::Window,
        "windowframe" => CssSystemColor::WindowFrame,
        "windowtext" => CssSystemColor::WindowText,
        _ => return None,
    };
    Some(value)
}

#[derive(Clone, Copy)]
pub(crate) enum RelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color,
}

pub(crate) fn relative_color_function_from_name(name: &str) -> Option<RelativeColorFunction> {
    let function = match_ignore_ascii_case! { name,
        "rgb" | "rgba" => RelativeColorFunction::Rgb,
        "hsl" | "hsla" => RelativeColorFunction::Hsl,
        "hwb" => RelativeColorFunction::Hwb,
        "lab" => RelativeColorFunction::Lab,
        "lch" => RelativeColorFunction::Lch,
        "oklab" => RelativeColorFunction::Oklab,
        "oklch" => RelativeColorFunction::Oklch,
        "color" => RelativeColorFunction::Color,
        _ => return None,
    };
    Some(function)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color,
    Alpha,
    Mix,
    LightDark,
    Contrast,
    DeviceCmyk,
}
impl ColorFunction {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Some(match relative_color_function_from_name(name) {
            Some(RelativeColorFunction::Rgb) => Self::Rgb,
            Some(RelativeColorFunction::Hsl) => Self::Hsl,
            Some(RelativeColorFunction::Hwb) => Self::Hwb,
            Some(RelativeColorFunction::Lab) => Self::Lab,
            Some(RelativeColorFunction::Lch) => Self::Lch,
            Some(RelativeColorFunction::Oklab) => Self::Oklab,
            Some(RelativeColorFunction::Oklch) => Self::Oklch,
            Some(RelativeColorFunction::Color) => Self::Color,
            None => match_ignore_ascii_case! { name,
                "alpha" => Self::Alpha, "color-mix" => Self::Mix,
                "light-dark" => Self::LightDark, "contrast-color" => Self::Contrast,
                "device-cmyk" => Self::DeviceCmyk, _ => return None,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Word {
    Current,
    Transparent,
    System(crate::color::CssSystemColor),
    Named([u8; 3]),
    Function(ColorFunction),
    Math(crate::numeric::CssMathFunction),
    Predefined(surgeist_color::PredefinedColorSpace),
    Interpolation(surgeist_color::ColorSpace),
    Hue(surgeist_color::HueInterpolation),
    From,
    In,
    None,
    HueSuffix,
    Constant(crate::numeric::CssNumericConstant),
    Strategy(crate::numeric::CssRoundingStrategy),
    Unknown,
}

/// The input is already decoded by the shared CSS Syntax name owner. This
/// fixed keyword dispatch is also available to ordinary authored adapters.
pub(crate) fn word(name: &str, kind: crate::tokenization::tokenizer::NameKind) -> Word {
    use crate::tokenization::tokenizer::NameKind;
    if kind == NameKind::Function {
        if let Some(value) = ColorFunction::parse(name) {
            return Word::Function(value);
        }
        return crate::numeric::CssMathFunction::parse(name).map_or(Word::Unknown, Word::Math);
    }
    if kind != NameKind::Ident {
        return Word::Unknown;
    }
    if let Some(value) = crate::color::syntax::recognition::predefined_color_space(name) {
        return Word::Predefined(value);
    }
    use surgeist_color::ColorSpace as Space;
    let interpolation = match ColorFunction::parse(name) {
        Some(ColorFunction::Hsl) => Some(Space::Hsl),
        Some(ColorFunction::Hwb) => Some(Space::Hwb),
        Some(ColorFunction::Lab) => Some(Space::Lab),
        Some(ColorFunction::Lch) => Some(Space::Lch),
        Some(ColorFunction::Oklab) => Some(Space::Oklab),
        Some(ColorFunction::Oklch) => Some(Space::Oklch),
        _ => None,
    };
    if let Some(value) = interpolation {
        return Word::Interpolation(value);
    }
    if let Some(value) = system_color(name) {
        return Word::System(value);
    }
    if let Some(value) = crate::numeric::CssNumericConstant::parse(name) {
        return Word::Constant(value);
    }
    match_ignore_ascii_case! { name,
        "currentcolor" => Word::Current, "transparent" => Word::Transparent,
        "from" => Word::From, "in" => Word::In, "none" => Word::None, "hue" => Word::HueSuffix,
        "shorter" => Word::Hue(surgeist_color::HueInterpolation::Shorter),
        "longer" => Word::Hue(surgeist_color::HueInterpolation::Longer),
        "increasing" => Word::Hue(surgeist_color::HueInterpolation::Increasing),
        "decreasing" => Word::Hue(surgeist_color::HueInterpolation::Decreasing),
        "nearest" => Word::Strategy(crate::numeric::CssRoundingStrategy::Nearest),
        "up" => Word::Strategy(crate::numeric::CssRoundingStrategy::Up),
        "down" => Word::Strategy(crate::numeric::CssRoundingStrategy::Down),
        "to-zero" => Word::Strategy(crate::numeric::CssRoundingStrategy::ToZero),
        _ => cssparser::color::parse_named_color(name).map_or(Word::Unknown, |(r, g, b)| Word::Named([r, g, b])),
    }
}

/// Color4 Appendix A aliases are processing semantics, not crate compatibility.
pub(crate) const fn canonical_system(value: CssSystemColor) -> CssSystemColor {
    use CssSystemColor::*;
    match value {
        ActiveBorder | InactiveBorder | ThreeDDarkShadow | ThreeDHighlight | ThreeDLightShadow
        | ThreeDShadow | WindowFrame => ButtonBorder,
        ActiveCaption | AppWorkspace | Background | InactiveCaption | InfoBackground | Menu
        | Scrollbar | Window => Canvas,
        ButtonHighlight | ButtonShadow | ThreeDFace => ButtonFace,
        CaptionText | InfoText | MenuText | WindowText => CanvasText,
        InactiveCaptionText => GrayText,
        canonical => canonical,
    }
}

#[cfg(test)]
mod namespace_tests {
    use super::*;
    use crate::tokenization::tokenizer::NameKind;
    #[test]
    fn named_identifiers_and_math_functions_keep_distinct_namespaces() {
        assert_eq!(word("tan", NameKind::Ident), Word::Named([210, 180, 140]));
        assert_eq!(
            word("TAN", NameKind::Function),
            Word::Math(crate::numeric::CssMathFunction::Tan)
        );
        assert_eq!(word("rgb", NameKind::Ident), Word::Unknown);
        assert_eq!(
            word("RGB", NameKind::Function),
            Word::Function(ColorFunction::Rgb)
        );
        assert_eq!(word("red", NameKind::Function), Word::Unknown);
        assert_eq!(
            word("hsl", NameKind::Ident),
            Word::Interpolation(surgeist_color::ColorSpace::Hsl)
        );
        assert_eq!(
            word("hsl", NameKind::Function),
            Word::Function(ColorFunction::Hsl)
        );
    }
}

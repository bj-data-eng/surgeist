#![allow(unused_macros)]

/// Test-only ordinary projection through the public ordered descriptor model.
/// A pending last occurrence deliberately shadows an earlier ordinary value.
macro_rules! ordinary_face {
    ($descriptors:expr, $kind:ident) => {{
        $descriptors
            .effective(surgeist_css::rule::font_face::CssFontFaceDescriptorKind::$kind)
            .and_then(|record| match record.value() {
                surgeist_css::rule::font_face::CssAuthoredFontFaceDescriptorValue::Ordinary(
                    surgeist_css::rule::font_face::CssFontFaceDescriptorValue::$kind(value),
                ) => Some(value),
                _ => None,
            })
    }};
}

macro_rules! string_format {
    ($source:expr) => {{
        let Some(surgeist_css::rule::font_face::CssFontFormat::String(value)) = $source.format()
        else {
            panic!("expected an authored string format");
        };
        value.as_str()
    }};
}

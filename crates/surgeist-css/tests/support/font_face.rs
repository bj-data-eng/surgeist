#![allow(unused_macros)]

/// Test-only ordinary projection through the public ordered descriptor model.
/// A pending last occurrence deliberately shadows an earlier ordinary value.
macro_rules! ordinary_face {
    ($descriptors:expr, $kind:ident) => {{
        $descriptors
            .effective(surgeist_css::CssFontFaceDescriptorKind::$kind)
            .and_then(|record| match record.value() {
                surgeist_css::CssAuthoredFontFaceDescriptorValue::Ordinary(
                    surgeist_css::CssFontFaceDescriptorValue::$kind(value),
                ) => Some(value),
                _ => None,
            })
    }};
}

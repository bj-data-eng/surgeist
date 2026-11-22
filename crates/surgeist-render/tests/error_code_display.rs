//! `ErrorCode` prints one lowercase phrase per classification, true at every place the code is raised.
//!
//! The exhaustive `match` has no wildcard: a new code fails to compile here until it gets a phrase.

use surgeist_render::ErrorCode;

fn phrase(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Accounting => "accounting failed",
        ErrorCode::GeometryRealization => "geometry realization refused",
        ErrorCode::GradientHint => "gradient hint refused",
        ErrorCode::GradientRealization => "gradient realization refused",
        #[cfg(feature = "render-paint")]
        ErrorCode::PaintGradient => "paint gradient preparation refused",
        #[cfg(feature = "render-text")]
        ErrorCode::TextLayout => "text layout admission refused",
        #[cfg(feature = "render-text")]
        ErrorCode::LeaderPattern => "leader pattern admission refused",
        ErrorCode::GlyphPreparation => "preparation refused",
        ErrorCode::DeviceCreateFailed => "gpu device creation failed",
        ErrorCode::RendererCreateFailed => "renderer creation failed",
        ErrorCode::SurfaceCreateFailed => "surface setup failed",
        ErrorCode::SurfaceConfigureFailed => "surface configuration unavailable",
        ErrorCode::SurfaceOutOfMemory => "gpu memory exhausted",
        ErrorCode::SurfaceTimeout => "surface operation timed out",
        ErrorCode::SurfaceOutdated => "surface configuration outdated",
        ErrorCode::InvalidInput => "invalid value",
        ErrorCode::UnsupportedPrimitive => "unsupported primitive",
        ErrorCode::UnresolvedResource => "unresolved resource",
        ErrorCode::DegradedQuality => "degraded quality",
        ErrorCode::RuntimeCapabilityUnavailable => "runtime capability unavailable",
        ErrorCode::ImageUploadFailed => "image data refused",
        ErrorCode::RenderFailed => "render pipeline failed",
        ErrorCode::ReadbackFailed => "readback failed",
        ErrorCode::PresentFailed => "presented output failed",
        ErrorCode::UnsupportedBackend => "unsupported backend",
        ErrorCode::IccProfile => "icc profile refused",
    }
}

const CODES: &[ErrorCode] = &[
    ErrorCode::Accounting,
    ErrorCode::GeometryRealization,
    ErrorCode::GradientHint,
    ErrorCode::GradientRealization,
    #[cfg(feature = "render-paint")]
    ErrorCode::PaintGradient,
    #[cfg(feature = "render-text")]
    ErrorCode::TextLayout,
    #[cfg(feature = "render-text")]
    ErrorCode::LeaderPattern,
    ErrorCode::GlyphPreparation,
    ErrorCode::DeviceCreateFailed,
    ErrorCode::RendererCreateFailed,
    ErrorCode::SurfaceCreateFailed,
    ErrorCode::SurfaceConfigureFailed,
    ErrorCode::SurfaceOutOfMemory,
    ErrorCode::SurfaceTimeout,
    ErrorCode::SurfaceOutdated,
    ErrorCode::InvalidInput,
    ErrorCode::UnsupportedPrimitive,
    ErrorCode::UnresolvedResource,
    ErrorCode::DegradedQuality,
    ErrorCode::RuntimeCapabilityUnavailable,
    ErrorCode::ImageUploadFailed,
    ErrorCode::RenderFailed,
    ErrorCode::ReadbackFailed,
    ErrorCode::PresentFailed,
    ErrorCode::UnsupportedBackend,
    ErrorCode::IccProfile,
];

#[test]
fn every_code_prints_its_phrase() {
    let mut seen = std::collections::BTreeSet::new();
    for &code in CODES {
        let text = code.to_string();
        assert_eq!(text, phrase(code), "{code:?}");
        assert_ne!(text, format!("{code:?}"));
        assert!(!text.is_empty() && !text.contains(['{', '}', ':']));
        assert!(text.chars().next().is_some_and(|c| c.is_ascii_lowercase()));
        assert!(seen.insert(text), "{code:?} repeats a phrase");
    }
}

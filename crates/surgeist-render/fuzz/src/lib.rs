//! Render-owned bounded public headless cases, canonical capture and replay.
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

mod codec;
mod resources;
mod scenes;

pub use resources::{ResourceCase, ResourceTarget};
pub use scenes::{SceneCase, SceneTarget};

use surgeist_fuzz::{
    CaseOutcome, FailureClass, FailureKey, FuzzError, FuzzErrorKind, PropertyId, Result,
    TargetDescriptor, registry::FuzzManifest,
};

pub(crate) fn error(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "render headless case", detail)
}

pub(crate) fn descriptor(id: &str) -> TargetDescriptor {
    let manifest: FuzzManifest =
        toml::from_str(include_str!("../fuzz.toml")).expect("checked Render descriptor");
    manifest
        .targets
        .into_iter()
        .find(|target| target.id().as_str() == id)
        .expect("registered Render target")
}

pub(crate) fn evaluated(properties: &[&str]) -> Result<CaseOutcome> {
    CaseOutcome::evaluated(
        properties
            .iter()
            .map(|name| PropertyId::new(*name))
            .collect::<Result<_>>()?,
    )
}

pub(crate) fn rejected(properties: &[&str], detail: impl Into<String>) -> Result<CaseOutcome> {
    CaseOutcome::expected_rejection(
        properties
            .iter()
            .map(|name| PropertyId::new(*name))
            .collect::<Result<_>>()?,
        detail,
    )
}

pub(crate) fn violated(
    property: &str,
    discriminator: &str,
    detail: impl Into<String>,
) -> Result<CaseOutcome> {
    CaseOutcome::failure(
        FailureKey::new(
            PropertyId::new(property)?,
            FailureClass::PropertyViolation,
            Some(discriminator.into()),
        )?,
        detail,
    )
}

/// Preserve a validated positive case's unexpected producer rejection as a
/// semantic finding. Transport, schema and harness allocation errors stay separate.
pub(crate) fn product_observation<T, E: std::fmt::Display>(
    result: std::result::Result<T, E>,
    property: &str,
    discriminator: &str,
) -> Result<std::result::Result<T, CaseOutcome>> {
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(cause) => Ok(Err(violated(property, discriminator, cause.to_string())?)),
    }
}

/// Observe a closed interval only when the operation published its terminal
/// record and completed mandatory cleanup.
fn standalone_attempt<'a>(
    context: &'a surgeist_render::accounting::RenderAccountingContext,
    property: &str,
    missing: &str,
) -> Result<std::result::Result<&'a surgeist_render::accounting::RenderAttemptReport, CaseOutcome>>
{
    let Some(report) = context.last_attempt() else {
        return Ok(Err(violated(
            property,
            missing,
            "operation did not publish a terminal accounting record",
        )?));
    };
    if let Some(fault) = report.cleanup_fault() {
        return Ok(Err(violated(
            property,
            "standalone-report-cleanup",
            format!("standalone accounting cleanup or finalization failed: {fault}"),
        )?));
    }
    product_observation(report.attempt(), property, "standalone-report-extraction")
}

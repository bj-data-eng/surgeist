//! CSS-owned bounded cases over the public ingestion and authored-value front doors.
//!
//! Each target owns a strict canonical codec, bounded generation, independent
//! expectations and one evaluator that ordinary workers, property tests,
//! coverage drivers and canonical replay all share. The shared
//! [`surgeist_fuzz`] framework owns engines, process custody and evidence.
#![forbid(unsafe_code)]

mod bytes;
mod check;
mod codec;
mod cursor;
mod nodes;
mod source;
#[cfg(test)]
mod testing;
mod values;

pub use bytes::{ByteCase, ByteTarget};
pub use check::Violation;
pub use source::{SourceCase, SourceTarget};
pub use values::{ValueCase, ValueTarget};

use surgeist_fuzz::{
    CaseOutcome, FailureClass, FailureKey, FuzzError, FuzzErrorKind, PropertyId, Result,
    TargetDescriptor, registry::FuzzManifest,
};

pub(crate) fn error(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::InvalidCase, "CSS fuzz case", detail)
}

pub(crate) fn descriptor(id: &str) -> TargetDescriptor {
    let manifest: FuzzManifest =
        toml::from_str(include_str!("../fuzz.toml")).expect("checked CSS descriptor");
    manifest
        .targets
        .into_iter()
        .find(|target| target.id().as_str() == id)
        .expect("registered CSS target")
}

fn property_ids(names: &[&str]) -> Result<Vec<PropertyId>> {
    names.iter().map(|name| PropertyId::new(*name)).collect()
}

pub(crate) fn evaluated(properties: &[&str]) -> Result<CaseOutcome> {
    CaseOutcome::evaluated(property_ids(properties)?)
}

pub(crate) fn rejected(properties: &[&str], detail: impl Into<String>) -> Result<CaseOutcome> {
    CaseOutcome::expected_rejection(property_ids(properties)?, detail)
}

/// A property failure whose stable key is the violated property and its discriminator.
pub(crate) fn violation(violation: &Violation) -> Result<CaseOutcome> {
    CaseOutcome::failure(
        FailureKey::new(
            PropertyId::new(violation.property())?,
            FailureClass::PropertyViolation,
            Some(violation.discriminator().to_owned()),
        )?,
        violation.detail().to_owned(),
    )
}

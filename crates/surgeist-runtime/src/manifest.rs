//! The authored application contract: validated names, command and event values, descriptors and the manifest.
//!
//! Command, event and payload type names ([`CommandName`], [`EventName`], [`PayloadTypeName`]) enter through fallible
//! validation and are rejected with a [`NameError`]; [`AppCommand`] and [`AppEvent`] carry validated names.
//! An [`AppManifest`] is an authored, unchecked set of descriptors ([`AppDescriptor`], [`CommandDescriptor`],
//! [`EventDescriptor`], [`ResourceDescriptor`], [`TaskDescriptor`], [`RootDescriptor`], [`WindowDescriptor`] and
//! [`StartupWindow`]). [`AppManifest::validate`] consumes it and returns a [`ValidatedAppManifest`], whose
//! cross-descriptor references have been validated, or a [`ManifestValidationError`] holding every
//! [`ManifestValidationIssue`] it found. [`App`] owns one validated manifest.

pub use crate::internal::command::{
    AppCommand, CommandDescriptor, CommandName, NameError, PayloadTypeName,
};
pub use crate::internal::descriptor::{
    App, AppDescriptor, AppManifest, ManifestValidationError, ManifestValidationErrorCode,
    ManifestValidationIssue, ResourceDescriptor, RootDescriptor, StartupWindow, TaskDescriptor,
    ValidatedAppManifest, WindowDescriptor, WindowDescriptorId,
};
pub use crate::internal::event::{AppEvent, EventDescriptor, EventName};

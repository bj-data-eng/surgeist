//! Declared snapshot bindings and the values collected under them.
//!
//! A [`SnapshotBinding`] declares one binding with its [`SnapshotBindingId`] and [`SnapshotSourceType`]. An
//! [`AppSnapshot`] is a root-bound collection of declared values at one state version; each [`SnapshotEntry`] associates a
//! validated binding with an opaque serialized [`SnapshotValue`]. A rejected input or transition is a
//! [`SnapshotError`] with a [`SnapshotErrorCode`].

pub use crate::internal::snapshot::{
    AppSnapshot, SnapshotBinding, SnapshotBindingId, SnapshotEntry, SnapshotError,
    SnapshotErrorCode, SnapshotSourceType, SnapshotValue,
};

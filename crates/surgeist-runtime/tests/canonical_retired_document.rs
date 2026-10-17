//! Canonical Document facts survive actual native hosted execution retirement.
//! DOM3071 creation/clone/attach steps consume Document facts; the separate
//! hosted Global remains unavailable for current execution after teardown.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

#[path = "canonical_retired_document/controls.rs"]
mod controls;
#[path = "canonical_retired_document/foundation.rs"]
mod foundation;
#[path = "canonical_retired_document/support.rs"]
mod support;

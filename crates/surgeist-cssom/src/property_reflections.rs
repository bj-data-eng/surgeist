//! Ordinary property reflectors share the declaration query and mutation owners.
//!
//! Live reflectors use the same public limits and admission contract:
//! ```no_run
//! use surgeist_cssom::{CssStyleProperties, CssomStore, CssomError,
//!     CssomDeclarationRequestLimits};
//! fn live(style: &CssStyleProperties, store: &mut CssomStore) -> Result<(), CssomError> {
//!     drop(style.set_width(store, "1px", CssomDeclarationRequestLimits::default(),
//!         |_| Err(CssomError::InvalidInput("provider decision required")))?);
//!     Ok(())
//! }
//! ```
//! Captures expose generated reads:
//! ```no_run
//! use surgeist_cssom::{CssStyleProperties, CssomCapture, CssomError,
//!     CssomDeclarationRequestLimits};
//! fn frozen(style: &CssomCapture<CssStyleProperties>) -> Result<String, CssomError> {
//!     style.width(CssomDeclarationRequestLimits::default())
//! }
//! ```
//! Descriptor objects have their own named members:
//! ```compile_fail
//! use surgeist_cssom::{CssPageDescriptors, CssomStore, CssomDeclarationRequestLimits};
//! fn descriptor(page: &CssPageDescriptors, store: &CssomStore) {
//!     let _ = page.width(store, CssomDeclarationRequestLimits::default());
//! }
//! ```
//! The retained keyframe interface returns the common declaration base:
//! ```compile_fail
//! use surgeist_cssom::{CssKeyframeRule, CssomStore, CssomDeclarationRequestLimits};
//! fn keyframe(frame: &CssKeyframeRule, store: &CssomStore) {
//!     let style = frame.style(store).unwrap();
//!     let _ = style.width(store, CssomDeclarationRequestLimits::default());
//! }
//! ```
//! Captures cannot perform generated writes:
//! ```compile_fail
//! use surgeist_cssom::{CssStyleProperties, CssomCapture, CssomStore,
//!     CssomDeclarationRequestLimits};
//! fn frozen(style: &CssomCapture<CssStyleProperties>, store: &mut CssomStore) {
//!     let _ = style.set_width(store, "1px", CssomDeclarationRequestLimits::default(),
//!         |_| unreachable!());
//! }
//! ```
use crate::*;

macro_rules! reflected_properties {
    ($($getter:ident, $setter:ident => $name:literal;)*) => {$(
        impl CssStyleProperties {
            #[doc = concat!("Read the authored `", $name, "` property through get_property_value. The Rust reflection is `", stringify!($getter), "`.")]
            pub fn $getter(
                &self,
                store: &CssomStore,
                limits: CssomDeclarationRequestLimits,
            ) -> Result<String, CssomError> {
                self.get_property_value(store, $name, limits)
            }

            #[doc = concat!("Set the authored `", $name, "` property with empty priority through the shared admission and publication owner. The Rust reflection is `", stringify!($setter), "`.")]
            pub fn $setter(
                &self,
                store: &mut CssomStore,
                value: &str,
                limits: CssomDeclarationRequestLimits,
                resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
            ) -> Result<CssomOperationOutcome<()>, CssomError> {
                self.set_property(store, $name, value, Some(""), limits, resolver)
            }
        }
        impl CssomCapture<CssStyleProperties> {
            #[doc = concat!("Read the captured authored `", $name, "` property without consulting live state. The Rust reflection is `", stringify!($getter), "`.")]
            pub fn $getter(&self, limits: CssomDeclarationRequestLimits) -> Result<String, CssomError> {
                self.get_property_value($name, limits)
            }
        }
    )*};
}

include!("generated_property_reflections.rs");

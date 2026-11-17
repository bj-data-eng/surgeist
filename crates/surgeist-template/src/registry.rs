//! Native-element and component vocabularies for validation.
//!
//! A template names elements that only the caller's environment can define.
//! A [`NativeElementRegistry`] lists the native elements and a
//! [`ComponentRegistry`] lists the components. Each [`NativeElementSpec`] or
//! [`ComponentSpec`] declares its allowed attributes as [`AttributeSpec`]
//! values, and each [`AttributeRule`] names the [`AttributeKind`] value forms
//! (boolean, static, expression or interpolated) that attribute accepts.
//!
//! [`NativeElementRegistry::from_html_metadata`] derives a closed native
//! vocabulary from HTML's known-name inventory with an explicit value-form
//! policy. That profile is a discovery aid, not complete HTML conformance, and
//! it does not recognize open or custom families.
//!
//! Constructors reject invalid and duplicate names with a [`RegistryError`].
//! Registries are consumed by [`validate_template`](crate::validate_template).

#[doc(inline)]
pub use crate::validate::{
    AttributeKind, AttributeRule, AttributeSpec, ComponentRegistry, ComponentSpec,
    NativeElementRegistry, NativeElementSpec, RegistryError,
};

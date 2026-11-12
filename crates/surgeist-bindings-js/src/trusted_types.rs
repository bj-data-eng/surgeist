//! Native brands for the string-valued Trusted Types interfaces.
use std::rc::Rc;

use crate::{DomString, Error, NativeObject, Scope, UsvString, Value};

/// The exact interface selected by a Trusted Types policy or sink.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustedTypeKind {
    Html,
    Script,
    ScriptUrl,
}

/// Converted associated data. Only the ScriptURL callback returns USVString.
#[derive(Debug)]
pub enum TrustedTypeData {
    Html(DomString),
    Script(DomString),
    ScriptUrl(UsvString),
}

impl TrustedTypeData {
    pub fn as_code_units(&self) -> &[u16] {
        match self {
            Self::Html(value) | Self::Script(value) => value.as_code_units(),
            Self::ScriptUrl(value) => value.as_code_units(),
        }
    }

    pub(crate) fn into_brand(self) -> Brand {
        let (kind, units) = match self {
            Self::Html(value) => (TrustedTypeKind::Html, value.into_code_units()),
            Self::Script(value) => (TrustedTypeKind::Script, value.into_code_units()),
            Self::ScriptUrl(value) => (TrustedTypeKind::ScriptUrl, value.into_code_units()),
        };
        Brand { kind, units }
    }
}

// Only the binding-owned projection can mint this exact native type.
pub(crate) struct Brand {
    kind: TrustedTypeKind,
    pub(crate) units: Vec<u16>,
}

/// An authentic Trusted Type observed on the exact original guest object.
/// This holds the associated data and its native backing without a guest root.
pub struct TrustedType(pub(crate) Rc<NativeObject<Brand>>);

impl TrustedType {
    pub fn kind(&self) -> TrustedTypeKind {
        self.0.payload().kind
    }

    pub fn as_code_units(&self) -> &[u16] {
        &self.0.payload().units
    }
}

impl<'scope> Scope<'scope> {
    /// Read the exact original object's Trusted Type brand without guest code.
    pub fn trusted_type(&mut self, object: &Value<'scope>) -> Result<Option<TrustedType>, Error> {
        Ok(self.native_object::<Brand>(object)?.map(TrustedType))
    }
}

/// The selected Web IDL union for Element's named attribute setters.
pub enum TrustedAttributeValue {
    Trusted(TrustedType),
    String(DomString),
}

impl TrustedAttributeValue {
    pub fn as_code_units(&self) -> &[u16] {
        match self {
            Self::Trusted(value) => value.as_code_units(),
            Self::String(value) => value.as_code_units(),
        }
    }
}

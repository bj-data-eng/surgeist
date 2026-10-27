//! Checked authored custom-media definitions and symbolic references.
use crate::component::{CssComponentValue, CssComponentValueRef, CssValueOrigin, CssValueTokenRef};
use crate::query::media::{CssMediaConstructionError, CssMediaQueryList};
use crate::source::CssSourcePosition;

/// A decoded extension name and its original identifier component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomMediaName {
    component: CssComponentValue,
    name: String,
}
impl CssCustomMediaName {
    /// Constructs from a decoded identifier value, escaping it as necessary.
    pub fn try_new(name: impl Into<String>) -> Result<Self, CssMediaConstructionError> {
        Self::try_from_component(CssComponentValue::try_ident(name)?)
    }
    pub fn try_from_component(
        component: CssComponentValue,
    ) -> Result<Self, CssMediaConstructionError> {
        let Some(name) = extension_name(&component) else {
            return Err(CssMediaConstructionError::from(
                crate::query::media::construction::MediaConstructionKind::InvalidName {
                    origin: component.origin().clone(),
                },
            ));
        };
        let name = name.to_owned();
        Ok(Self { component, name })
    }
    pub fn as_str(&self) -> &str {
        &self.name
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.component.origin()
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
}
/// An authored definition body, without definition lookup or evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum CssCustomMediaBody {
    True,
    False,
    Media(CssMediaQueryList),
}
/// A checked symbolic boolean reference. The original parentheses remain available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomMediaReference {
    syntax: Box<CustomMediaReferenceSyntax>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct CustomMediaReferenceSyntax {
    name: CssCustomMediaName,
    component: CssComponentValue,
}
impl CssCustomMediaReference {
    pub(crate) fn new(name: CssCustomMediaName, component: CssComponentValue) -> Self {
        Self {
            syntax: Box::new(CustomMediaReferenceSyntax { name, component }),
        }
    }
    pub const fn name(&self) -> &CssCustomMediaName {
        &self.syntax.name
    }
    pub const fn component(&self) -> &CssComponentValue {
        &self.syntax.component
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        self.syntax.component.origin()
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
}

pub(crate) fn ident(value: &CssComponentValue) -> Option<&str> {
    match value.view() {
        CssComponentValueRef::Token(CssValueTokenRef::Ident(name)) => Some(name),
        _ => None,
    }
}
pub(crate) fn extension_name(value: &CssComponentValue) -> Option<&str> {
    ident(value).filter(|name| name.starts_with("--"))
}

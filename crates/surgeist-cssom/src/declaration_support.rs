//! Explicit composed-owner usability facts; parser recognition is not usable support.
use crate::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use surgeist_css::declaration::CssDeclaration;
use surgeist_css::property::CssPropertyGrammar;
use surgeist_css::rule::font_face::{CssFontFaceDescriptor, CssFontFaceDescriptorKind};
use surgeist_css::rule::page::{CssPageDescriptor, CssPageDescriptorKind};

/// The composed owner either guarantees every checked form or decides the whole declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomUsabilityRequirement {
    AllCheckedValues,
    WholeDeclarationDecision,
}
/// A bounded immutable usable-property membership supplied by the composed owner.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CssomDeclarationSupport {
    properties: Vec<(CssPropertyGrammar, CssomUsabilityRequirement)>,
    page: Vec<(CssPageDescriptorKind, CssomUsabilityRequirement)>,
    font_face: Vec<(CssFontFaceDescriptorKind, CssomUsabilityRequirement)>,
    svg_glyph: Option<CssomUsabilityRequirement>,
}
impl CssomDeclarationSupport {
    pub fn try_new(
        properties: Vec<(CssPropertyGrammar, CssomUsabilityRequirement)>,
    ) -> Result<Self, CssomError> {
        for (index, (grammar, _)) in properties.iter().enumerate() {
            if properties[..index]
                .iter()
                .any(|(other, _)| other == grammar)
            {
                return Err(CssomError::InvalidInput("duplicate usable grammar"));
            }
        }
        Ok(Self {
            properties,
            page: Vec::new(),
            font_face: Vec::new(),
            svg_glyph: None,
        })
    }
    pub fn with_page_descriptors(
        mut self,
        descriptors: Vec<(CssPageDescriptorKind, CssomUsabilityRequirement)>,
    ) -> Result<Self, CssomError> {
        for (index, (kind, _)) in descriptors.iter().enumerate() {
            if descriptors[..index].iter().any(|(other, _)| other == kind) {
                return Err(CssomError::InvalidInput("duplicate usable Page descriptor"));
            }
        }
        self.page = descriptors;
        Ok(self)
    }
    pub fn with_font_face_descriptors(
        mut self,
        descriptors: Vec<(CssFontFaceDescriptorKind, CssomUsabilityRequirement)>,
    ) -> Result<Self, CssomError> {
        for (index, (kind, _)) in descriptors.iter().enumerate() {
            if descriptors[..index].iter().any(|(other, _)| other == kind) {
                return Err(CssomError::InvalidInput(
                    "duplicate usable FontFace descriptor",
                ));
            }
        }
        self.font_face = descriptors;
        Ok(self)
    }
    pub fn page_descriptors(&self) -> &[(CssPageDescriptorKind, CssomUsabilityRequirement)] {
        &self.page
    }
    pub fn font_face_descriptors(
        &self,
    ) -> &[(CssFontFaceDescriptorKind, CssomUsabilityRequirement)] {
        &self.font_face
    }
    /// Supplied support for the existing independent SVG terminal, distinct from its finite alias.
    pub fn with_svg_glyph_orientation_vertical(
        mut self,
        requirement: CssomUsabilityRequirement,
    ) -> Self {
        self.svg_glyph = Some(requirement);
        self
    }
    pub fn svg_glyph_orientation_vertical(&self) -> Option<CssomUsabilityRequirement> {
        self.svg_glyph
    }
    pub fn properties(&self) -> &[(CssPropertyGrammar, CssomUsabilityRequirement)] {
        &self.properties
    }
    pub(crate) fn requirement(
        &self,
        grammar: CssPropertyGrammar,
    ) -> Option<CssomUsabilityRequirement> {
        self.properties
            .iter()
            .find(|(candidate, _)| *candidate == grammar)
            .map(|(_, requirement)| *requirement)
    }
}
/// A whole checked declaration is admitted or discarded, never partially selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssomDeclarationUsability {
    Usable,
    Unusable,
}
/// A complete domain-owned checked candidate; no grammar or source coordinates are manufactured.
#[derive(Clone, Debug)]
pub enum CssomDeclarationValue {
    Property(CssDeclaration),
    PageDescriptor(CssPageDescriptor),
    FontFaceDescriptor(CssFontFaceDescriptor),
}
/// An actual checked candidate exposed for a composed-owner usability decision.
#[derive(Clone, Debug)]
pub struct CssomDeclarationCandidate {
    pub(crate) preparation: Arc<()>,
    pub(crate) ordinal: usize,
    pub(crate) value: CssomDeclarationValue,
}
impl CssomDeclarationCandidate {
    pub fn value(&self) -> &CssomDeclarationValue {
        &self.value
    }
    pub fn declaration(&self) -> Option<&CssDeclaration> {
        match &self.value {
            CssomDeclarationValue::Property(declaration) => Some(declaration),
            _ => None,
        }
    }
    pub fn decide(
        &self,
        version: CssomInputVersion,
        usability: CssomDeclarationUsability,
    ) -> Result<CssomDeclarationDecision, CssomError> {
        if version.role != CssomInputRole::Support {
            return Err(CssomError::InvalidInput("declaration decision role"));
        }
        Ok(CssomDeclarationDecision {
            preparation: self.preparation.clone(),
            ordinal: self.ordinal,
            version,
            usability,
        })
    }
}
#[derive(Clone, Debug)]
pub struct CssomDeclarationDecision {
    pub(crate) preparation: Arc<()>,
    pub(crate) ordinal: usize,
    pub(crate) version: CssomInputVersion,
    pub(crate) usability: CssomDeclarationUsability,
}
pub(crate) struct PendingPreparation {
    pub count: Arc<AtomicUsize>,
    pub active: bool,
}
impl PendingPreparation {
    pub fn new(count: Arc<AtomicUsize>) -> Self {
        count.fetch_add(1, Ordering::Relaxed);
        Self {
            count,
            active: true,
        }
    }
}
impl Drop for PendingPreparation {
    fn drop(&mut self) {
        if self.active {
            self.count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

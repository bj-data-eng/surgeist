//! The live dataset association over one Element's ordered canonical Attrs.

use surgeist_html::{
    microsyntax::dataset::{HtmlDatasetAttributeName, HtmlDatasetPropertyName},
    vocabulary::{ElementKey, ElementNamespace},
};

use crate::{
    DomError, DomSnapshot,
    internal::{
        attr::{DomAttr, DomNamedNodeMap},
        identity::NodeId,
        snapshot::DomNode,
    },
};

/// The dataset association of an HTML, SVG or MathML Element.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomDataset(NodeId);

impl DomDataset {
    /// Authenticate an already-admitted Element of a family implementing
    /// HTMLOrSVGOrMathMLElement.
    pub fn new(node: DomNode<'_>) -> Option<Self> {
        let element = node.as_element()?;
        match ElementKey::from_name(element.name()).namespace() {
            ElementNamespace::Html | ElementNamespace::Svg | ElementNamespace::MathMl => {
                Some(Self(node.id()))
            }
            ElementNamespace::Other => None,
        }
    }

    /// The associated canonical Element identity.
    pub const fn element(self) -> NodeId {
        self.0
    }

    /// The first eligible Attr matching a property in canonical attribute order.
    /// Matching uses the exact local name, independent of namespace and prefix.
    pub fn get_with<'a, E: From<DomError>>(
        self,
        snapshot: &'a DomSnapshot,
        property: &[u16],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomAttr<'a>>, E> {
        let attributes = self.attributes_with(snapshot, &mut step)?;
        if attributes.is_empty() {
            return Ok(None);
        }
        let Some(local) = HtmlDatasetAttributeName::for_set_with(property, &mut step)? else {
            return Ok(None);
        };
        attributes.find_with(step, |attribute, step| {
            local.matches_local_name_with(attribute.name().local_name().code_units(), step)
        })
    }

    /// Visit eligible property/Attr pairs in canonical order, including repeated
    /// spellings from distinct Attr identities. The caller may deduplicate
    /// property keys for an interface that requires unique own keys.
    pub fn visit_with<'a, E: From<DomError>>(
        self,
        snapshot: &'a DomSnapshot,
        mut step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(HtmlDatasetPropertyName<'a>, DomAttr<'a>) -> Result<(), E>,
    ) -> Result<(), E> {
        self.attributes_with(snapshot, &mut step)?
            .visit_accounted_with(step, |attribute, step| {
                if let Some(property) = HtmlDatasetPropertyName::from_attribute_with(
                    attribute.name().local_name().code_units(),
                    step,
                )? {
                    visit(property, attribute)?;
                }
                Ok(())
            })
    }

    fn attributes_with<E: From<DomError>>(
        self,
        snapshot: &DomSnapshot,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<DomNamedNodeMap<'_>, E> {
        Ok(snapshot
            .node_with(self.0, step)?
            .attributes()
            .ok_or(DomError::WrongNodeKind)?)
    }
}

//! DOM namespace queries over ordinary tree and attribute relationships.

use super::{DomNode, DomNodeKind};
use crate::{DocumentString, DomError, names::same_code_units};

const XML: &[u16] = &ascii_units(b"xml");
const XMLNS: &[u16] = &ascii_units(b"xmlns");
const XML_URI: &[u16] = &ascii_units(b"http://www.w3.org/XML/1998/namespace");
const XMLNS_URI: &[u16] = &ascii_units(b"http://www.w3.org/2000/xmlns/");

const fn ascii_units<const N: usize>(bytes: &[u8; N]) -> [u16; N] {
    let mut units = [0; N];
    let mut index = 0;
    while index < N {
        units[index] = bytes[index] as u16;
        index += 1;
    }
    units
}

/// A nonempty namespace URI from a captured DOM query. The result shares a
/// native string or refers to one of the fixed XML namespace constants.
#[derive(Clone, Debug)]
pub struct DomNamespaceUri(NamespaceUri);

#[derive(Clone, Debug)]
enum NamespaceUri {
    Xml,
    Xmlns,
    Stored(DocumentString),
}

impl DomNamespaceUri {
    pub fn code_units(&self) -> &[u16] {
        match &self.0 {
            NamespaceUri::Xml => XML_URI,
            NamespaceUri::Xmlns => XMLNS_URI,
            NamespaceUri::Stored(value) => value.code_units(),
        }
    }
}

impl<'a> DomNode<'a> {
    /// Find the first prefix for a nonempty namespace URI, preserving native
    /// attribute order. Every reached lookup and compared code unit is metered.
    pub fn lookup_prefix_with<E: From<DomError>>(
        self,
        namespace: Option<&[u16]>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DocumentString>, E> {
        let Some(namespace) = nonempty(namespace) else {
            return Ok(None);
        };
        let mut current = self.namespace_element(&mut step)?;
        while let Some(node) = current {
            let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
            let name = element.name();
            if let (Some(uri), Some(prefix)) = (name.namespace_uri(), name.prefix())
                && same_code_units(uri.code_units(), namespace, &mut step)?
            {
                return Ok(Some(prefix.clone()));
            }
            let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
            let declaration = attributes.find_with(&mut step, |attribute, step| {
                Ok(
                    equal_units(units(attribute.name().prefix()), Some(XMLNS), &mut *step)?
                        && same_code_units(attribute.value().code_units(), namespace, step)?,
                )
            })?;
            if let Some(attribute) = declaration {
                return Ok(Some(attribute.name().local_name().clone()));
            }
            current = node.ordinary_parent_element(&mut step)?;
        }
        Ok(None)
    }

    /// Find the namespace URI for a prefix in this ordinary tree. An empty
    /// prefix selects the default declaration. An empty declaration ends lookup
    /// with no URI. Every reached lookup and compared code unit is metered.
    pub fn lookup_namespace_uri_with<E: From<DomError>>(
        self,
        prefix: Option<&[u16]>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomNamespaceUri>, E> {
        let prefix = nonempty(prefix);
        let Some(mut node) = self.namespace_element(&mut step)? else {
            return Ok(None);
        };
        for (reserved, uri) in [(XML, NamespaceUri::Xml), (XMLNS, NamespaceUri::Xmlns)] {
            if equal_units(prefix, Some(reserved), &mut step)? {
                return Ok(Some(DomNamespaceUri(uri)));
            }
        }
        let (local, expected_prefix) = match prefix {
            Some(prefix) => (prefix, Some(XMLNS)),
            None => (XMLNS, None),
        };
        loop {
            let element = node.as_element().ok_or(DomError::WrongNodeKind)?;
            let name = element.name();
            if let Some(uri) = name.namespace_uri()
                && equal_units(units(name.prefix()), prefix, &mut step)?
            {
                return Ok(Some(DomNamespaceUri(NamespaceUri::Stored(uri.clone()))));
            }
            let attributes = node.attributes().ok_or(DomError::WrongNodeKind)?;
            let declaration = attributes.find_with(&mut step, |attribute, step| {
                let name = attribute.name();
                if !equal_units(units(name.namespace_uri()), Some(XMLNS_URI), &mut *step)? {
                    return Ok(false);
                }
                Ok(
                    equal_units(units(name.prefix()), expected_prefix, &mut *step)?
                        && same_code_units(name.local_name().code_units(), local, step)?,
                )
            })?;
            if let Some(attribute) = declaration {
                let value = attribute.value();
                return Ok((!value.is_empty())
                    .then(|| DomNamespaceUri(NamespaceUri::Stored(value.clone()))));
            }
            let Some(parent) = node.ordinary_parent_element(&mut step)? else {
                return Ok(None);
            };
            node = parent;
        }
    }

    /// Compare the normalized namespace with this node's current default URI.
    pub fn is_default_namespace_with<E: From<DomError>>(
        self,
        namespace: Option<&[u16]>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let current = self.lookup_namespace_uri_with(None, &mut step)?;
        equal_units(
            current.as_ref().map(DomNamespaceUri::code_units),
            nonempty(namespace),
            step,
        )
    }

    fn namespace_element<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        match self.kind() {
            DomNodeKind::Element(_) => Ok(Some(self)),
            DomNodeKind::Document(_) => self.first_element_child_with(&mut step),
            DomNodeKind::Attr(_) => self
                .as_attr()
                .and_then(|attribute| attribute.owner_element())
                .map(|id| self.snapshot.node_with(id, step))
                .transpose(),
            DomNodeKind::DocumentType(_)
            | DomNodeKind::DocumentFragment
            | DomNodeKind::ShadowRoot(_) => Ok(None),
            _ => self.ordinary_parent_element(step),
        }
    }
}

fn nonempty(units: Option<&[u16]>) -> Option<&[u16]> {
    units.filter(|units| !units.is_empty())
}

fn units(value: Option<&DocumentString>) -> Option<&[u16]> {
    value.map(DocumentString::code_units)
}

fn equal_units<E>(
    left: Option<&[u16]>,
    right: Option<&[u16]>,
    step: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    match (left, right) {
        (Some(left), Some(right)) => same_code_units(left, right, step),
        (None, None) => Ok(true),
        _ => Ok(false),
    }
}

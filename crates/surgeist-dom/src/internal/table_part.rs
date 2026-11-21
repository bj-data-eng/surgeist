//! Bounded observations of the first direct HTML Table parts.

use super::{identity::NodeId, snapshot::DomNode, store::DomError};
use surgeist_html::vocabulary::KnownHtmlElement;

/// One of the first direct HTML parts of a Table element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHtmlTablePart {
    /// An HTML caption element.
    Caption,
    /// An HTML thead element.
    Head,
    /// An HTML tfoot element.
    Foot,
}

impl DomHtmlTablePart {
    pub(crate) fn element(self) -> KnownHtmlElement {
        match self {
            Self::Caption => KnownHtmlElement::Caption,
            Self::Head => KnownHtmlElement::Thead,
            Self::Foot => KnownHtmlElement::Tfoot,
        }
    }

    pub(crate) fn first_with<Id: Copy, E>(
        self,
        children: impl IntoIterator<Item = Id>,
        mut kind: impl FnMut(Id) -> Result<Option<KnownHtmlElement>, E>,
    ) -> Result<Option<Id>, E> {
        let expected = self.element();
        for child in children {
            if kind(child)? == Some(expected) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }
}

impl DomNode<'_> {
    /// Read the first matching direct HTML child of this Table, if present.
    ///
    /// The result belongs to this snapshot, including a caller-authorized
    /// published Source segment. Ordinary child order also applies when the
    /// Table is detached. The meter admits the reached table classification,
    /// child sequence, each reached child and its exact canonical lookup.
    ///
    /// # Errors
    ///
    /// Preserves the first caller refusal or lookup failure. A non-Table
    /// receiver returns [WrongNodeKind](DomError::WrongNodeKind).
    pub fn html_table_part_with<E: From<DomError>>(
        self,
        part: DomHtmlTablePart,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        step()?;
        if self
            .as_element()
            .and_then(|element| KnownHtmlElement::from_name(element.name()))
            != Some(KnownHtmlElement::Table)
        {
            return Err(DomError::WrongNodeKind.into());
        }
        step()?;
        part.first_with(self.children().iter().copied(), |child| {
            step()?;
            let child = self.snapshot.node_with(child, &mut step)?;
            step()?;
            Ok(child
                .as_element()
                .and_then(|element| KnownHtmlElement::from_name(element.name())))
        })
    }
}

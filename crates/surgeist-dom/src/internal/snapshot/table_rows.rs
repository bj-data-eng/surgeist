//! Live scalar row indexes over checked ordinary tree views.

use super::{DomError, DomNode, DomSnapshot, NodeId};
use std::{ops::ControlFlow, slice};
use surgeist_html::vocabulary::KnownHtmlElement;
use surgeist_html_semantics::table_rows::{
    HtmlTableRowChild, HtmlTableRowIndexKind, HtmlTableRowOrder, HtmlTableRowParentRoute,
    HtmlTableRowReader,
};

impl DomNode<'_> {
    /// Observe this HTML row's zero-based index in its Table or section.
    ///
    /// Returns `None` for an HTML row without the applicable ordinary context.
    /// The result belongs to this snapshot, including an authorized published
    /// Source segment. Only the reached parent and row prefix are read.
    ///
    /// # Errors
    ///
    /// Returns the first caller refusal or failed lookup. A non-HTML-row
    /// subject returns [WrongNodeKind](DomError::WrongNodeKind). A qualifying
    /// row absent from the canonical sequence returns [NotFound](DomError::NotFound).
    pub fn html_table_row_index_with<E: From<DomError>>(
        self,
        kind: HtmlTableRowIndexKind,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<usize>, E> {
        let mut read = RowRead {
            snapshot: self.snapshot,
            step,
        };
        if read.kind(self)? != Some(KnownHtmlElement::Tr) {
            return Err(DomError::WrongNodeKind.into());
        }
        let Some(parent) = read.parent(self)? else {
            return Ok(None);
        };
        let (container, order) = match kind.parent_route(read.kind(parent)?) {
            HtmlTableRowParentRoute::None => return Ok(None),
            HtmlTableRowParentRoute::Direct(order) => (parent, order),
            HtmlTableRowParentRoute::SectionParentTable => {
                let Some(table) = read.parent(parent)? else {
                    return Ok(None);
                };
                if read.kind(table)? != Some(KnownHtmlElement::Table) {
                    return Ok(None);
                }
                (table, HtmlTableRowOrder::Table)
            }
        };
        let target = self.id();
        let mut index = 0_usize;
        match order.visit(&mut read, &container.id(), |read, id| {
            read.step()?;
            if id == target {
                return Ok(ControlFlow::Break(index));
            }
            index = index
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            Ok(ControlFlow::Continue(()))
        })? {
            ControlFlow::Break(index) => Ok(Some(index)),
            ControlFlow::Continue(()) => Err(DomError::NotFound.into()),
        }
    }
}

struct RowRead<'a, Step> {
    snapshot: &'a DomSnapshot,
    step: Step,
}

impl<'a, E: From<DomError>, Step: FnMut() -> Result<(), E>> RowRead<'a, Step> {
    fn kind(&mut self, node: DomNode<'_>) -> Result<Option<KnownHtmlElement>, E> {
        self.step()?;
        // Recognition is bounded by the fixed HTML namespace and longest HTML name.
        Ok(node
            .as_element()
            .and_then(|element| KnownHtmlElement::from_name(element.name())))
    }

    fn parent(&mut self, node: DomNode<'_>) -> Result<Option<DomNode<'a>>, E> {
        self.step()?;
        node.parent()
            .map(|id| self.snapshot.node_with(id, &mut self.step))
            .transpose()
    }
}

impl<'a, E: From<DomError>, Step: FnMut() -> Result<(), E>> HtmlTableRowReader
    for RowRead<'a, Step>
{
    type Id = NodeId;
    type Children = slice::Iter<'a, NodeId>;
    type Error = E;

    fn step(&mut self) -> Result<(), E> {
        (self.step)()
    }

    fn children(&mut self, container: &NodeId) -> Result<Self::Children, E> {
        let node = self.snapshot.node_with(*container, &mut self.step)?;
        self.step()?;
        Ok(node.children().iter())
    }

    fn next_element(
        &mut self,
        children: &mut Self::Children,
    ) -> Result<Option<HtmlTableRowChild<NodeId>>, E> {
        loop {
            self.step()?;
            let Some(&id) = children.next() else {
                return Ok(None);
            };
            let node = self.snapshot.node_with(id, &mut self.step)?;
            self.step()?;
            if let Some(element) = node.as_element() {
                let kind = KnownHtmlElement::from_name(element.name());
                return Ok(Some(HtmlTableRowChild { id, kind }));
            }
        }
    }
}

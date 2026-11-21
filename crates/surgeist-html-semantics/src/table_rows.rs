//! Ordinary HTML table row ordering shared by captured and live tree providers.

use std::ops::ControlFlow;
use surgeist_html::vocabulary::KnownHtmlElement;

/// Which HTML row index is being queried.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableRowIndexKind {
    /// Position in the containing Table's rows order.
    Table,
    /// Position in the immediate Table or section parent's rows order.
    Section,
}

/// The collection route determined by the row's ordinary parent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableRowParentRoute {
    /// The known parent kind supplies neither applicable collection.
    None,
    /// The ordinary parent supplies this row order.
    Direct(HtmlTableRowOrder),
    /// The ordinary parent is a section. Its own ordinary parent must be an HTML Table.
    SectionParentTable,
}

impl HtmlTableRowIndexKind {
    /// Select the route from a qualified parent's expanded HTML name.
    ///
    /// The adapter distinguishes a known unrecognized parent from an unavailable relationship.
    pub fn parent_route(self, parent: Option<KnownHtmlElement>) -> HtmlTableRowParentRoute {
        match (self, parent) {
            (_, Some(KnownHtmlElement::Table)) => {
                HtmlTableRowParentRoute::Direct(HtmlTableRowOrder::Table)
            }
            (
                _,
                Some(KnownHtmlElement::Thead | KnownHtmlElement::Tbody | KnownHtmlElement::Tfoot),
            ) => match self {
                Self::Table => HtmlTableRowParentRoute::SectionParentTable,
                Self::Section => HtmlTableRowParentRoute::Direct(HtmlTableRowOrder::Section),
            },
            _ => HtmlTableRowParentRoute::None,
        }
    }
}

/// The source order of rows in an HTML Table or Table section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableRowOrder {
    /// Direct header-group rows first, direct and body-group rows next, footer-group rows last.
    Table,
    /// Direct HTML tr children in ordinary order.
    Section,
}

/// One reached element with an exact expanded-name classification.
#[derive(Debug)]
pub struct HtmlTableRowChild<Id> {
    /// The provider's exact node identity.
    pub id: Id,
    /// The canonical HTML kind, if recognized.
    pub kind: Option<KnownHtmlElement>,
}

/// Supplies the provider's checked ordinary-child traversal and accounting.
pub trait HtmlTableRowReader {
    /// A node identity in this reader's topology.
    type Id;
    /// The provider's cursor over one parent's ordinary children.
    type Children;
    /// The provider's first missing fact or refusal.
    type Error;

    /// Admit one reached policy step.
    fn step(&mut self) -> Result<(), Self::Error>;

    /// Open the checked ordinary child sequence of this container.
    fn children(&mut self, container: &Self::Id) -> Result<Self::Children, Self::Error>;

    /// Advance to the next element, admitting any skipped non-elements and cursor exhaustion.
    fn next_element(
        &mut self,
        children: &mut Self::Children,
    ) -> Result<Option<HtmlTableRowChild<Self::Id>>, Self::Error>;
}

impl HtmlTableRowOrder {
    /// Visit rows of an already-qualified container using its ordinary topology.
    ///
    /// The reader supplies exact node identities and fallible observations. A
    /// break finishes the visit without observing later positions or phases.
    pub fn visit<R: HtmlTableRowReader, T>(
        self,
        reader: &mut R,
        container: &R::Id,
        mut visit: impl FnMut(&mut R, R::Id) -> Result<ControlFlow<T>, R::Error>,
    ) -> Result<ControlFlow<T>, R::Error> {
        match self {
            Self::Table => visit_table_rows(reader, container, &mut visit),
            Self::Section => visit_direct_rows(reader, container, &mut visit),
        }
    }
}

#[derive(Clone, Copy)]
enum RowPhase {
    Head,
    Body,
    Foot,
}

fn visit_table_rows<R: HtmlTableRowReader, T>(
    reader: &mut R,
    table: &R::Id,
    visit: &mut impl FnMut(&mut R, R::Id) -> Result<ControlFlow<T>, R::Error>,
) -> Result<ControlFlow<T>, R::Error> {
    for phase in [RowPhase::Head, RowPhase::Body, RowPhase::Foot] {
        reader.step()?;
        let mut children = reader.children(table)?;
        while let Some(HtmlTableRowChild { id, kind }) = reader.next_element(&mut children)? {
            reader.step()?;
            let result = match (phase, kind) {
                (RowPhase::Head, Some(KnownHtmlElement::Thead))
                | (RowPhase::Body, Some(KnownHtmlElement::Tbody))
                | (RowPhase::Foot, Some(KnownHtmlElement::Tfoot)) => {
                    visit_direct_rows(reader, &id, visit)?
                }
                (RowPhase::Body, Some(KnownHtmlElement::Tr)) => visit(reader, id)?,
                _ => continue,
            };
            if let ControlFlow::Break(value) = result {
                return Ok(ControlFlow::Break(value));
            }
        }
    }
    Ok(ControlFlow::Continue(()))
}

fn visit_direct_rows<R: HtmlTableRowReader, T>(
    reader: &mut R,
    parent: &R::Id,
    visit: &mut impl FnMut(&mut R, R::Id) -> Result<ControlFlow<T>, R::Error>,
) -> Result<ControlFlow<T>, R::Error> {
    let mut children = reader.children(parent)?;
    while let Some(HtmlTableRowChild { id, kind }) = reader.next_element(&mut children)? {
        reader.step()?;
        if kind == Some(KnownHtmlElement::Tr)
            && let ControlFlow::Break(value) = visit(reader, id)?
        {
            return Ok(ControlFlow::Break(value));
        }
    }
    Ok(ControlFlow::Continue(()))
}

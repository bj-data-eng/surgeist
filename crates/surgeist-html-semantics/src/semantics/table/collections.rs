//! Immutable direct table membership, HTML 877d6146487f §§4.9.1/5/8.
//! No slot projection, live collection or mutation.
//! Native optional ordinals observe the pinned
//! [rowIndex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tr-rowindex),
//! [sectionRowIndex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tr-sectionrowindex), and
//! [cellIndex](https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-tdth-cellindex)
//! algorithms: None represents source −1, without a WebIDL long conversion.

use super::*;
use crate::table_rows::{
    HtmlTableRowChild, HtmlTableRowIndexKind, HtmlTableRowOrder, HtmlTableRowParentRoute,
    HtmlTableRowReader,
};
use std::ops::ControlFlow;
use surgeist_html::vocabulary::KnownHtmlElement;

/// Direct HTML tbody subjects in a qualified table snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableBodyCollection {
    bodies: Vec<HtmlCaptureSubjectId>,
}
impl HtmlTableBodyCollection {
    pub fn bodies(&self) -> &[HtmlCaptureSubjectId] {
        &self.bodies
    }
    pub fn len(&self) -> usize {
        self.bodies.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }
}
/// HTML tr subjects in source collection order, without implied slot rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableRowCollection {
    rows: Vec<HtmlCaptureSubjectId>,
}
impl HtmlTableRowCollection {
    pub fn rows(&self) -> &[HtmlCaptureSubjectId] {
        &self.rows
    }
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
/// Direct HTML td/th subjects in a qualified row snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableCellCollection {
    cells: Vec<HtmlCaptureSubjectId>,
}
impl HtmlTableCellCollection {
    pub fn cells(&self) -> &[HtmlCaptureSubjectId] {
        &self.cells
    }
    pub fn len(&self) -> usize {
        self.cells.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}
/// A collection of a table: its rows in header, body, footer order, or one that lists a single
/// parent's children directly in source order.
#[derive(Clone, Copy)]
enum Collection {
    Rows(HtmlTableRowOrder),
    Direct(DirectCollection),
}
#[derive(Clone, Copy)]
enum DirectCollection {
    Bodies,
    Cells,
}
enum CollectionProjection<'t> {
    Snapshot(&'t mut Vec<HtmlCaptureSubjectId>),
    Target {
        subject: &'t HtmlCaptureSubjectId,
        index: usize,
        found: bool,
    },
}
#[derive(Clone, Copy)]
enum TableIndex {
    Row(HtmlTableRowIndexKind),
    Cell,
}
impl<'a> HtmlSemanticSession<'a> {
    pub(in crate::semantics) fn table_row_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<usize>>, HtmlSemanticFailure> {
        self.table_index(subject, TableIndex::Row(HtmlTableRowIndexKind::Table))
    }
    pub(in crate::semantics) fn table_section_row_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<usize>>, HtmlSemanticFailure> {
        self.table_index(subject, TableIndex::Row(HtmlTableRowIndexKind::Section))
    }
    pub(in crate::semantics) fn table_cell_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<usize>>, HtmlSemanticFailure> {
        self.table_index(subject, TableIndex::Cell)
    }
    fn table_index(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        query: TableIndex,
    ) -> Result<HtmlSemanticValue<Option<usize>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let kind = self.known(element)?;
        let applicable = match query {
            TableIndex::Row(_) => kind == Some(KnownHtmlElement::Tr),
            TableIndex::Cell => matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th)),
        };
        if !applicable {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        match self.table_index_inner(subject, node, query) {
            Ok(index) => Ok(HtmlSemanticValue::Known(index)),
            Err(TableStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(TableStop::Failure(failure)) => Err(failure),
        }
    }
    fn table_index_inner(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        node: HtmlCapturedNode<'a>,
        query: TableIndex,
    ) -> TableResult<Option<usize>> {
        let Some(parent) = self.parent(node)?.map_err(TableStop::Missing)? else {
            return Ok(None);
        };
        let parent_node = self.node(&parent)?;
        let parent_kind = match parent_node.kind() {
            HtmlCapturedNodeKind::Element(element) => self.known(element)?,
            _ => None,
        };
        let (container, collection) = match query {
            TableIndex::Cell if parent_kind == Some(KnownHtmlElement::Tr) => {
                (parent, Collection::Direct(DirectCollection::Cells))
            }
            TableIndex::Cell => return Ok(None),
            TableIndex::Row(kind) => match kind.parent_route(parent_kind) {
                HtmlTableRowParentRoute::None => return Ok(None),
                HtmlTableRowParentRoute::Direct(order) => (parent, Collection::Rows(order)),
                HtmlTableRowParentRoute::SectionParentTable => {
                    let Some(table) = self.parent(parent_node)?.map_err(TableStop::Missing)? else {
                        return Ok(None);
                    };
                    let table_node = self.node(&table)?;
                    let HtmlCapturedNodeKind::Element(element) = table_node.kind() else {
                        return Ok(None);
                    };
                    if self.known(element)? != Some(KnownHtmlElement::Table) {
                        return Ok(None);
                    }
                    (table, Collection::Rows(HtmlTableRowOrder::Table))
                }
            },
        };
        let mut projection = CollectionProjection::Target {
            subject,
            index: 0,
            found: false,
        };
        self.table_collection_project(&container, collection, &mut projection)?;
        // Capture construction proves coherent ordinary parent/child edges.
        // A qualifying subject must occur in its complete source collection.
        match projection {
            CollectionProjection::Target {
                index, found: true, ..
            } => Ok(Some(index)),
            _ => Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into()),
        }
    }
    pub(in crate::semantics) fn table_bodies(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableBodyCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Direct(DirectCollection::Bodies))?
            .map(|bodies| HtmlTableBodyCollection { bodies }))
    }
    pub(in crate::semantics) fn table_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableRowCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Rows(HtmlTableRowOrder::Table))?
            .map(|rows| HtmlTableRowCollection { rows }))
    }
    pub(in crate::semantics) fn table_section_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableRowCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Rows(HtmlTableRowOrder::Section))?
            .map(|rows| HtmlTableRowCollection { rows }))
    }
    pub(in crate::semantics) fn table_row_cells(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableCellCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Direct(DirectCollection::Cells))?
            .map(|cells| HtmlTableCellCollection { cells }))
    }
    fn table_collection(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        collection: Collection,
    ) -> Result<HtmlSemanticValue<Vec<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        let kind = self.known(element)?;
        let applicable = match collection {
            Collection::Rows(HtmlTableRowOrder::Table)
            | Collection::Direct(DirectCollection::Bodies) => kind == Some(KnownHtmlElement::Table),
            Collection::Rows(HtmlTableRowOrder::Section) => matches!(
                kind,
                Some(KnownHtmlElement::Thead | KnownHtmlElement::Tbody | KnownHtmlElement::Tfoot)
            ),
            Collection::Direct(DirectCollection::Cells) => kind == Some(KnownHtmlElement::Tr),
        };
        if !applicable {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let mut ids = Vec::new();
        let result = self.table_collection_project(
            subject,
            collection,
            &mut CollectionProjection::Snapshot(&mut ids),
        );
        match result {
            Ok(()) => Ok(HtmlSemanticValue::Known(ids)),
            Err(TableStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(TableStop::Failure(failure)) => Err(failure),
        }
    }
    fn table_collection_project(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        collection: Collection,
        projection: &mut CollectionProjection<'_>,
    ) -> TableResult<()> {
        match collection {
            Collection::Rows(order) => {
                let _ = order.visit(&mut CapturedRowRead(self), subject, |reader, row| {
                    Ok(if reader.0.table_collection_emit(projection, row)? {
                        ControlFlow::Break(())
                    } else {
                        ControlFlow::Continue(())
                    })
                })?;
                Ok(())
            }
            Collection::Direct(direct) => self.table_collection_direct(subject, direct, projection),
        }
    }
    fn table_collection_emit(
        &mut self,
        projection: &mut CollectionProjection<'_>,
        id: HtmlCaptureSubjectId,
    ) -> TableResult<bool> {
        match projection {
            CollectionProjection::Snapshot(ids) => {
                self.table_push(ids, id)?;
                Ok(false)
            }
            CollectionProjection::Target {
                subject,
                index,
                found,
            } => {
                self.charge(1)?;
                if &id == *subject {
                    self.charge(1)?;
                    *found = true;
                    Ok(true)
                } else {
                    self.charge(1)?;
                    *index = self.table_add(*index, 1)?;
                    Ok(false)
                }
            }
        }
    }
    fn table_collection_direct(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        collection: DirectCollection,
        projection: &mut CollectionProjection<'_>,
    ) -> TableResult<()> {
        let mut children = self.table_children(subject)?;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            self.charge(1)?;
            let keep = match collection {
                DirectCollection::Bodies => kind == Some(KnownHtmlElement::Tbody),
                DirectCollection::Cells => {
                    matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th))
                }
            };
            if keep && self.table_collection_emit(projection, id)? {
                return Ok(());
            }
        }
        Ok(())
    }
}

struct CapturedRowRead<'session, 'capture>(&'session mut HtmlSemanticSession<'capture>);

impl<'a> HtmlTableRowReader for CapturedRowRead<'_, 'a> {
    type Id = HtmlCaptureSubjectId;
    type Children = HtmlCapturedChildIds<'a>;
    type Error = TableStop;

    fn step(&mut self) -> TableResult<()> {
        self.0.charge(1).map_err(Into::into)
    }

    fn children(&mut self, container: &Self::Id) -> TableResult<Self::Children> {
        self.0.table_children(container)
    }

    fn next_element(
        &mut self,
        children: &mut Self::Children,
    ) -> TableResult<Option<HtmlTableRowChild<Self::Id>>> {
        Ok(self
            .0
            .table_next(children)?
            .map(|(id, kind)| HtmlTableRowChild { id, kind }))
    }
}

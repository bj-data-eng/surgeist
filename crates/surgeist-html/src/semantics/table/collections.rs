//! Immutable direct table membership, HTML 877d6146487f §§4.9.1/5/8.
//! No slot projection, live collection, mutation or index readback.

use super::*;

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
#[derive(Clone, Copy)]
enum Collection {
    Bodies,
    Rows,
    SectionRows,
    Cells,
}
#[derive(Clone, Copy)]
enum RowPhase {
    Head,
    Body,
    Foot,
}
impl<'a> HtmlSemanticSession<'a> {
    pub(in crate::semantics) fn table_bodies(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableBodyCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Bodies)?
            .map(|bodies| HtmlTableBodyCollection { bodies }))
    }
    pub(in crate::semantics) fn table_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableRowCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Rows)?
            .map(|rows| HtmlTableRowCollection { rows }))
    }
    pub(in crate::semantics) fn table_section_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableRowCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::SectionRows)?
            .map(|rows| HtmlTableRowCollection { rows }))
    }
    pub(in crate::semantics) fn table_row_cells(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableCellCollection>, HtmlSemanticFailure> {
        Ok(self
            .table_collection(subject, Collection::Cells)?
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
            Collection::Bodies | Collection::Rows => kind == Some(KnownHtmlElement::Table),
            Collection::SectionRows => matches!(
                kind,
                Some(KnownHtmlElement::Thead | KnownHtmlElement::Tbody | KnownHtmlElement::Tfoot)
            ),
            Collection::Cells => kind == Some(KnownHtmlElement::Tr),
        };
        if !applicable {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let result = if matches!(collection, Collection::Rows) {
            self.table_collection_rows(subject)
        } else {
            self.table_collection_direct(subject, collection)
        };
        match result {
            Ok(ids) => Ok(HtmlSemanticValue::Known(ids)),
            Err(TableStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(TableStop::Failure(failure)) => Err(failure),
        }
    }
    fn table_collection_direct(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        collection: Collection,
    ) -> TableResult<Vec<HtmlCaptureSubjectId>> {
        let mut children = self.table_children(subject)?;
        let mut ids = Vec::new();
        while let Some((id, kind)) = self.table_next(&mut children)? {
            self.charge(1)?;
            let keep = match collection {
                Collection::Bodies => kind == Some(KnownHtmlElement::Tbody),
                Collection::SectionRows => kind == Some(KnownHtmlElement::Tr),
                Collection::Cells => {
                    matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th))
                }
                Collection::Rows => unreachable!("table rows have a separate source order"),
            };
            if keep {
                self.table_push(&mut ids, id)?;
            }
        }
        Ok(ids)
    }
    fn table_collection_append_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        ids: &mut Vec<HtmlCaptureSubjectId>,
    ) -> TableResult<()> {
        let mut children = self.table_children(subject)?;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            self.charge(1)?;
            if kind == Some(KnownHtmlElement::Tr) {
                self.table_push(ids, id)?;
            }
        }
        Ok(())
    }
    fn table_collection_rows(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> TableResult<Vec<HtmlCaptureSubjectId>> {
        let mut ids = Vec::new();
        // Fixed passes preserve source header/body/footer order without
        // temporary section/footer buffers. Every actual visit is charged.
        for phase in [RowPhase::Head, RowPhase::Body, RowPhase::Foot] {
            self.charge(1)?;
            let mut children = self.table_children(subject)?;
            while let Some((id, kind)) = self.table_next(&mut children)? {
                self.charge(1)?;
                match (phase, kind) {
                    (RowPhase::Head, Some(KnownHtmlElement::Thead))
                    | (RowPhase::Body, Some(KnownHtmlElement::Tbody))
                    | (RowPhase::Foot, Some(KnownHtmlElement::Tfoot)) => {
                        self.table_collection_append_rows(&id, &mut ids)?
                    }
                    (RowPhase::Body, Some(KnownHtmlElement::Tr)) => {
                        self.table_push(&mut ids, id)?
                    }
                    _ => {}
                }
            }
        }
        Ok(ids)
    }
}

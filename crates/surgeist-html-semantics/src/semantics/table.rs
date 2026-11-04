//! HTML 877d6146487f §4.9.12: slots, associations and recoverable model errors.
//! Ordinary capture relations are the only topology; no CSS or live tree occurs.

mod collections;
pub use collections::*;
mod axes;
mod headers;
mod occupancy;
mod placement;
mod slots;
use headers::{HeaderMemo, IdTargets};
use placement::{Footprint, Placement, Reach};
use slots::{LeastTree, SlotAxis};

use super::*;
use crate::HtmlCapturedChildIds;
use surgeist_html::attributes::table_header_scope::{
    HtmlTableHeaderScope, HtmlTableHeaderScopeState,
};
use surgeist_html::bounded::{
    attributes::{HtmlTableSpanRule, th_scope_state, th_scope_work_bound},
    lexical::is_ascii_whitespace,
};
use surgeist_html::{
    values::{Attribute, DocumentString},
    vocabulary::KnownHtmlElement,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableCellKind {
    Data,
    Header,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableCell {
    subject: HtmlCaptureSubjectId,
    kind: HtmlTableCellKind,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}
impl HtmlTableCell {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn kind(&self) -> HtmlTableCellKind {
        self.kind
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableRow {
    source: Option<HtmlCaptureSubjectId>,
}
impl HtmlTableRow {
    pub fn source(&self) -> Option<&HtmlCaptureSubjectId> {
        self.source.as_ref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableColumn {
    source: Option<HtmlCaptureSubjectId>,
}
impl HtmlTableColumn {
    pub fn source(&self) -> Option<&HtmlCaptureSubjectId> {
        self.source.as_ref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableRowGroup {
    subject: HtmlCaptureSubjectId,
    y: usize,
    height: usize,
}
impl HtmlTableRowGroup {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn height(&self) -> usize {
        self.height
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableColumnGroup {
    subject: HtmlCaptureSubjectId,
    x: usize,
    width: usize,
}
impl HtmlTableColumnGroup {
    pub fn subject(&self) -> &HtmlCaptureSubjectId {
        &self.subject
    }
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn width(&self) -> usize {
        self.width
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableHeaderAssociation {
    cell: HtmlCaptureSubjectId,
    header: HtmlCaptureSubjectId,
}
impl HtmlTableHeaderAssociation {
    pub fn cell(&self) -> &HtmlCaptureSubjectId {
        &self.cell
    }
    /// Explicit `headers` can identify either kind of cell, as defined by HTML.
    pub fn header(&self) -> &HtmlCaptureSubjectId {
        &self.header
    }
}
/// Source-defined recovered errors; overlapping rectangular coverage is retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlTableModelError {
    Overlap {
        cell: HtmlCaptureSubjectId,
        other: HtmlCaptureSubjectId,
        x: usize,
        y: usize,
    },
    UnanchoredRow {
        y: usize,
    },
    UnanchoredColumn {
        x: usize,
    },
}
/// Complete semantic grid. Row/column positions are their slice indices.
/// Rectangles retain all slot occupants, including overlaps. No dense grid is stored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlTableModel {
    table: HtmlCaptureSubjectId,
    caption: Option<HtmlCaptureSubjectId>,
    cells: Vec<HtmlTableCell>,
    rows: Vec<HtmlTableRow>,
    columns: Vec<HtmlTableColumn>,
    row_groups: Vec<HtmlTableRowGroup>,
    column_groups: Vec<HtmlTableColumnGroup>,
    headers: Vec<HtmlTableHeaderAssociation>,
    errors: Vec<HtmlTableModelError>,
}
impl HtmlTableModel {
    pub fn table(&self) -> &HtmlCaptureSubjectId {
        &self.table
    }
    pub fn caption(&self) -> Option<&HtmlCaptureSubjectId> {
        self.caption.as_ref()
    }
    pub fn width(&self) -> usize {
        self.columns.len()
    }
    pub fn height(&self) -> usize {
        self.rows.len()
    }
    pub fn cells(&self) -> &[HtmlTableCell] {
        &self.cells
    }
    pub fn rows(&self) -> &[HtmlTableRow] {
        &self.rows
    }
    pub fn columns(&self) -> &[HtmlTableColumn] {
        &self.columns
    }
    pub fn row_groups(&self) -> &[HtmlTableRowGroup] {
        &self.row_groups
    }
    pub fn column_groups(&self) -> &[HtmlTableColumnGroup] {
        &self.column_groups
    }
    pub fn headers(&self) -> &[HtmlTableHeaderAssociation] {
        &self.headers
    }
    pub fn errors(&self) -> &[HtmlTableModelError] {
        &self.errors
    }
}

enum TableStop {
    Failure(HtmlSemanticFailure),
    Missing(HtmlSemanticMissing),
}
impl From<HtmlSemanticFailure> for TableStop {
    fn from(value: HtmlSemanticFailure) -> Self {
        Self::Failure(value)
    }
}
type TableResult<T> = Result<T, TableStop>;
#[derive(Default)]
struct Scratch<'a> {
    footers: Vec<HtmlCaptureSubjectId>,
    placement: Placement,
    headers: Vec<usize>,
    /// Per header key, the scan it last hid headers of that key in.
    stamps: Vec<usize>,
    /// Per cell, the principal cell whose header list it last joined.
    listed: Vec<usize>,
    /// The keys of the headers in the current block.
    block: Vec<usize>,
    document: Vec<HtmlCapturedChildIds<'a>>,
    /// What resolves `headers` tokens, built when the first one needs it.
    targets: Option<IdTargets<'a>>,
    memo: Vec<HeaderMemo>,
    /// Lines of a scan that are scanned one at a time.
    specials: Vec<usize>,
    /// The header cells whose `scope` names a row group, and those whose
    /// `scope` names a column group, in model order. Kept only for tables
    /// that have such groups.
    row_group_headers: Vec<usize>,
    column_group_headers: Vec<usize>,
    /// Over the row group headers, in model order, their columns; over the
    /// column group headers, in order of column, their rows.
    row_group_least: LeastTree,
    column_group_least: LeastTree,
    /// Which rows, then columns, hold a cell's top-left slot, and which rows
    /// or columns begin a new line of an axis.
    flags: Vec<bool>,
    /// The lines of an axis under construction: how many cells cross each,
    /// the cells in the order the lines list them, and the cells ordered by
    /// column.
    line_counts: Vec<usize>,
    placed: Vec<usize>,
    order: Vec<usize>,
    column_counts: Vec<usize>,
    columns: SlotAxis,
    rows: SlotAxis,
}
impl Scratch<'_> {
    /// Every capacity admitted for this query's scratch.
    fn backings(&self) -> impl Iterator<Item = ScratchBacking> {
        [
            ScratchBacking::of(&self.footers),
            ScratchBacking::of(&self.headers),
            ScratchBacking::of(&self.stamps),
            ScratchBacking::of(&self.listed),
            ScratchBacking::of(&self.row_group_least.nodes),
            ScratchBacking::of(&self.column_group_least.nodes),
            ScratchBacking::of(&self.block),
            ScratchBacking::of(&self.document),
            ScratchBacking::of(&self.memo),
            ScratchBacking::of(&self.specials),
            ScratchBacking::of(&self.row_group_headers),
            ScratchBacking::of(&self.column_group_headers),
            ScratchBacking::of(&self.flags),
            ScratchBacking::of(&self.line_counts),
            ScratchBacking::of(&self.placed),
            ScratchBacking::of(&self.order),
            ScratchBacking::of(&self.column_counts),
        ]
        .into_iter()
        .chain(self.targets.iter().flat_map(IdTargets::backings))
        .chain(self.placement.backings())
        .chain(self.columns.backings())
        .chain(self.rows.backings())
    }
}
/// The content attribute a span rule reads.
fn table_span_attribute(rule: HtmlTableSpanRule) -> &'static str {
    match rule {
        HtmlTableSpanRule::CellColspan => "colspan",
        HtmlTableSpanRule::CellRowspan => "rowspan",
        HtmlTableSpanRule::ColSpan | HtmlTableSpanRule::ColgroupSpan => "span",
    }
}

impl<'a> HtmlSemanticSession<'a> {
    /// HTML 877d §4.9.1, the first caption element child, or null:
    /// <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-table-caption>.
    /// This getter does not require table-model formation or caption descendants.
    pub(super) fn table_caption(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        self.table_first_child(subject, KnownHtmlElement::Caption)
    }

    /// First direct HTML thead child, or null, HTML 877d §4.9.1:
    /// <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-table-thead>.
    pub(super) fn table_head(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        self.table_first_child(subject, KnownHtmlElement::Thead)
    }

    /// First direct HTML tfoot child, or null, HTML 877d §4.9.1:
    /// <https://html.spec.whatwg.org/commit-snapshots/877d6146487f15b15bc8dd0e3c625063a37b456a/#dom-table-tfoot>.
    pub(super) fn table_foot(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        self.table_first_child(subject, KnownHtmlElement::Tfoot)
    }

    fn table_first_child(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        expected: KnownHtmlElement,
    ) -> Result<HtmlSemanticValue<Option<HtmlCaptureSubjectId>>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Table) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        Ok(match self.first_html_element_child(subject, expected)? {
            Ok(child) => HtmlSemanticValue::Known(child),
            Err(missing) => HtmlSemanticValue::Missing(missing),
        })
    }

    pub(super) fn table_model(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> Result<HtmlSemanticValue<HtmlTableModel>, HtmlSemanticFailure> {
        let node = self.node(subject)?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            return Ok(HtmlSemanticValue::Inapplicable);
        };
        if !self.html_namespace(element)? {
            return Ok(HtmlSemanticValue::Unsupported(
                HtmlSemanticUnsupported::ForeignNamespace,
            ));
        }
        if self.known(element)? != Some(KnownHtmlElement::Table) {
            return Ok(HtmlSemanticValue::Inapplicable);
        }
        let document = node.node_document();
        let mut scratch = Scratch::default();
        let result = self.form_table(subject, document.subject(), &mut scratch);
        // Capacities were admitted together in the one session counter, so
        // their sum is representable. Cleanup does not poll the stopped gate.
        for backing in scratch.backings() {
            self.release_scratch_backing(backing);
        }
        match result {
            Ok(model) => Ok(HtmlSemanticValue::Known(model)),
            Err(TableStop::Missing(missing)) => Ok(HtmlSemanticValue::Missing(missing)),
            Err(TableStop::Failure(failure)) => Err(failure),
        }
    }
    fn table_children(
        &mut self,
        subject: &HtmlCaptureSubjectId,
    ) -> TableResult<HtmlCapturedChildIds<'a>> {
        let node = self.node(subject)?;
        self.charge(1)?;
        match node.children() {
            HtmlCapturedChildren::Complete(children) => Ok(children),
            HtmlCapturedChildren::Unavailable => Err(TableStop::Missing(
                HtmlSemanticMissing::OrderedChildren(subject.clone()),
            )),
        }
    }
    fn table_next(
        &mut self,
        children: &mut HtmlCapturedChildIds<'a>,
    ) -> TableResult<Option<(HtmlCaptureSubjectId, Option<KnownHtmlElement>)>> {
        loop {
            self.charge(1)?;
            let Some(id) = children.next() else {
                return Ok(None);
            };
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                return Ok(Some((id, self.known(element)?)));
            }
        }
    }
    fn table_add(&mut self, left: usize, right: usize) -> TableResult<usize> {
        left.checked_add(right).ok_or_else(|| {
            self.stop(HtmlSemanticFailureKind::RepresentationExhausted)
                .into()
        })
    }
    fn table_push<T>(&mut self, output: &mut Vec<T>, value: T) -> TableResult<()> {
        self.reserve_nested_output(output, 1)?;
        self.charge(1)?;
        output.push(value);
        Ok(())
    }
    fn table_scratch<T>(&mut self, output: &mut Vec<T>, value: T) -> TableResult<()> {
        self.reserve_scratch(output, 1)?;
        self.charge(1)?;
        output.push(value);
        Ok(())
    }
    fn table_element(&mut self, id: &HtmlCaptureSubjectId) -> TableResult<&'a HtmlCapturedElement> {
        match self.node(id)?.kind() {
            HtmlCapturedNodeKind::Element(element) => Ok(element),
            _ => Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into()),
        }
    }
    fn table_raw(
        &mut self,
        element: &'a HtmlCapturedElement,
        local: &str,
    ) -> TableResult<Option<&'a Attribute>> {
        let raw = self.attribute(element, local)?;
        self.interpreter
            .admit_attribute_raw(raw.map(Attribute::value))
            .map_err(|e| TableStop::Failure(self.stop(HtmlSemanticFailureKind::Attribute(e))))?;
        Ok(raw)
    }
    /// Admits an owner-published work bound as one atomic operation.
    fn table_charge_bound(&mut self, bound: Option<usize>) -> TableResult<()> {
        let bound = bound.ok_or_else(|| {
            TableStop::Failure(self.stop(HtmlSemanticFailureKind::RepresentationExhausted))
        })?;
        self.charge(bound)?;
        Ok(())
    }
    /// Reads one span quantity through the HTML-owned rule; the caps, zero
    /// handling and absence default live there, not here. The rule's own
    /// published bound for the raw value is admitted before it runs, and the
    /// read allocates nothing.
    fn table_span(
        &mut self,
        id: &HtmlCaptureSubjectId,
        rule: HtmlTableSpanRule,
    ) -> TableResult<usize> {
        let element = self.table_element(id)?;
        let raw = self.table_raw(element, table_span_attribute(rule))?;
        let value = raw.map(Attribute::value);
        self.table_charge_bound(rule.work_bound(value.map_or(0, DocumentString::len)))?;
        let span = rule.content_value(value);
        self.poll()?;
        Ok(usize::from(span))
    }
    fn table_width(
        &mut self,
        model: &mut HtmlTableModel,
        width: usize,
        source: Option<&HtmlCaptureSubjectId>,
    ) -> TableResult<()> {
        let additional = width.saturating_sub(model.width());
        self.reserve_nested_output(&mut model.columns, additional)?;
        while model.width() < width {
            self.charge(1)?;
            model.columns.push(HtmlTableColumn {
                source: source.cloned(),
            });
        }
        Ok(())
    }
    fn table_height(&mut self, model: &mut HtmlTableModel, height: usize) -> TableResult<()> {
        let additional = height.saturating_sub(model.height());
        self.reserve_nested_output(&mut model.rows, additional)?;
        while model.height() < height {
            self.charge(1)?;
            model.rows.push(HtmlTableRow { source: None });
        }
        Ok(())
    }
    fn table_colgroup(
        &mut self,
        id: &HtmlCaptureSubjectId,
        model: &mut HtmlTableModel,
    ) -> TableResult<()> {
        let mut children = self.table_children(id)?;
        let start = model.width();
        let mut has_column = false;
        while let Some((column, kind)) = self.table_next(&mut children)? {
            if kind == Some(KnownHtmlElement::Col) {
                has_column = true;
                let span = self.table_span(&column, HtmlTableSpanRule::ColSpan)?;
                let width = self.table_add(model.width(), span)?;
                self.table_width(model, width, Some(&column))?;
            }
        }
        if !has_column {
            let span = self.table_span(id, HtmlTableSpanRule::ColgroupSpan)?;
            let width = self.table_add(model.width(), span)?;
            self.table_width(model, width, None)?;
        }
        let width = model.width() - start;
        self.table_push(
            &mut model.column_groups,
            HtmlTableColumnGroup {
                subject: id.clone(),
                x: start,
                width,
            },
        )
    }
    fn table_end_group(
        &mut self,
        model: &mut HtmlTableModel,
        y: &mut usize,
        place: &mut Placement,
    ) -> TableResult<()> {
        while *y < model.height() {
            self.table_grow(place, *y)?;
            self.charge(1)?;
            *y = self.table_add(*y, 1)?;
        }
        self.table_settle(model, place)
    }
    fn table_row(
        &mut self,
        row: &HtmlCaptureSubjectId,
        model: &mut HtmlTableModel,
        y: &mut usize,
        place: &mut Placement,
    ) -> TableResult<()> {
        if model.height() == *y {
            let height = self.table_add(*y, 1)?;
            self.table_height(model, height)?;
        }
        self.charge(1)?;
        model.rows[*y].source = Some(row.clone());
        self.table_grow(place, *y)?;
        let mut children = self.table_children(row)?;
        let mut x = 0;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            if !matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th)) {
                continue;
            }
            self.table_columns_ready(place, model.width())?;
            x = self.table_first_free(place, x, *y, model.width())?;
            if x == model.width() {
                let width = self.table_add(x, 1)?;
                self.table_width(model, width, None)?;
            }
            let width = self.table_span(&id, HtmlTableSpanRule::CellColspan)?;
            let mut height = self.table_span(&id, HtmlTableSpanRule::CellRowspan)?;
            let grows = height == 0;
            if grows {
                height = 1;
            }
            let right = self.table_add(x, width)?;
            let bottom = self.table_add(*y, height)?;
            self.table_width(model, right, None)?;
            self.table_height(model, bottom)?;
            self.table_columns_ready(place, model.width())?;
            let footprint = Footprint { x, y: *y, right };
            self.table_overlaps(model, &id, footprint, place)?;
            let index = model.cells.len();
            self.table_push(
                &mut model.cells,
                HtmlTableCell {
                    subject: id,
                    kind: if kind == Some(KnownHtmlElement::Th) {
                        HtmlTableCellKind::Header
                    } else {
                        HtmlTableCellKind::Data
                    },
                    x,
                    y: *y,
                    width,
                    height,
                },
            )?;
            let reach = if grows {
                Reach::Growing
            } else {
                Reach::Until(bottom)
            };
            self.table_occupy(place, index, footprint, reach)?;
            x = right;
        }
        self.charge(1)?;
        *y = self.table_add(*y, 1)?;
        Ok(())
    }
    fn table_group(
        &mut self,
        group: &HtmlCaptureSubjectId,
        model: &mut HtmlTableModel,
        y: &mut usize,
        place: &mut Placement,
    ) -> TableResult<()> {
        let start = model.height();
        let mut children = self.table_children(group)?;
        while let Some((row, kind)) = self.table_next(&mut children)? {
            if kind == Some(KnownHtmlElement::Tr) {
                self.table_row(&row, model, y, place)?;
            }
        }
        if model.height() > start {
            let height = model.height() - start;
            self.table_push(
                &mut model.row_groups,
                HtmlTableRowGroup {
                    subject: group.clone(),
                    y: start,
                    height,
                },
            )?;
        }
        self.table_end_group(model, y, place)
    }
    fn form_table(
        &mut self,
        subject: &HtmlCaptureSubjectId,
        document: &HtmlCaptureSubjectId,
        scratch: &mut Scratch<'a>,
    ) -> TableResult<HtmlTableModel> {
        self.charge(1)?;
        let mut model = HtmlTableModel {
            table: subject.clone(),
            caption: None,
            cells: Vec::new(),
            rows: Vec::new(),
            columns: Vec::new(),
            row_groups: Vec::new(),
            column_groups: Vec::new(),
            headers: Vec::new(),
            errors: Vec::new(),
        };
        let mut children = self.table_children(subject)?;
        let mut caption_children = children.clone();
        while let Some((id, kind)) = self.table_next(&mut caption_children)? {
            if kind == Some(KnownHtmlElement::Caption) {
                self.charge(1)?;
                model.caption = Some(id);
                break;
            }
        }
        let mut columns_phase = true;
        let mut y = 0;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            match kind {
                Some(KnownHtmlElement::Colgroup) if columns_phase => {
                    self.table_colgroup(&id, &mut model)?
                }
                Some(KnownHtmlElement::Tr) => {
                    columns_phase = false;
                    self.table_row(&id, &mut model, &mut y, &mut scratch.placement)?;
                }
                Some(
                    KnownHtmlElement::Thead | KnownHtmlElement::Tbody | KnownHtmlElement::Tfoot,
                ) => {
                    columns_phase = false;
                    self.table_end_group(&mut model, &mut y, &mut scratch.placement)?;
                    if kind == Some(KnownHtmlElement::Tfoot) {
                        self.table_scratch(&mut scratch.footers, id)?;
                    } else {
                        self.table_group(&id, &mut model, &mut y, &mut scratch.placement)?;
                    }
                }
                _ => {}
            }
        }
        for footer in &scratch.footers {
            self.charge(1)?;
            self.table_group(footer, &mut model, &mut y, &mut scratch.placement)?;
        }
        self.table_settle(&mut model, &mut scratch.placement)?;
        self.table_unanchored(&mut model, &mut scratch.flags)?;
        self.table_headers(&mut model, document, scratch)?;
        self.poll()?;
        Ok(model)
    }
}

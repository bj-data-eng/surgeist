//! HTML 877d6146487f §4.9.12: slots, associations and recoverable model errors.
//! Ordinary capture relations are the only topology; no CSS or live tree occurs.

mod collections;
pub use collections::*;
mod slots;
use slots::{Cover, NO_HEADER, SlotAxis, SlotEntry, SlotLine};

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
/// What formation keeps while it places cells row by row.
#[derive(Default)]
struct Placement {
    downward: Vec<usize>,
    /// The cells that cover the row being placed, as intervals of columns.
    /// Rows are placed top to bottom, so every earlier cell that can overlap
    /// or precede a new one covers that row: this one line answers formation's
    /// slot questions.
    active: Vec<SlotEntry>,
    /// Earlier cells a new cell overlaps, before they are reported in order.
    hits: Vec<usize>,
}

/// What the header scans remember about one cell. A cell's attributes do not
/// change during a query, so each is resolved the first time a header
/// assignment consults it and reused by every assignment that reaches the cell.
#[derive(Clone, Copy, Default)]
struct HeaderMemo {
    /// The th scope state.
    scope: Option<HtmlTableHeaderScopeState>,
    /// Whether no data cell shares a row with the header (scope auto).
    column: Option<bool>,
    /// Whether no data cell shares a column with the header (scope auto).
    row: Option<bool>,
}

#[derive(Default)]
struct Scratch<'a> {
    footers: Vec<HtmlCaptureSubjectId>,
    placement: Placement,
    headers: Vec<usize>,
    opaque: Vec<usize>,
    block: Vec<usize>,
    document: Vec<HtmlCapturedChildIds<'a>>,
    memo: Vec<HeaderMemo>,
    /// The header cells in model order, kept only for tables with groups.
    header_cells: Vec<usize>,
    /// Which rows, then columns, hold a cell's top-left slot, and which rows
    /// or columns begin a new line of an axis.
    flags: Vec<bool>,
    /// Cells ordered by column, and the column counts that order them.
    order: Vec<usize>,
    counts: Vec<usize>,
    columns: SlotAxis,
    rows: SlotAxis,
}
impl Scratch<'_> {
    /// Every capacity admitted for this query's scratch.
    fn capacities(&self) -> [usize; 19] {
        [
            self.footers.capacity(),
            self.placement.downward.capacity(),
            self.placement.active.capacity(),
            self.placement.hits.capacity(),
            self.headers.capacity(),
            self.opaque.capacity(),
            self.block.capacity(),
            self.document.capacity(),
            self.memo.capacity(),
            self.header_cells.capacity(),
            self.flags.capacity(),
            self.order.capacity(),
            self.counts.capacity(),
            self.columns.breaks.capacity(),
            self.columns.lines.capacity(),
            self.columns.entries.capacity(),
            self.rows.breaks.capacity(),
            self.rows.lines.capacity(),
            self.rows.entries.capacity(),
        ]
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
        for capacity in scratch.capacities() {
            self.release_scratch(capacity);
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
    fn table_grow(
        &mut self,
        model: &mut HtmlTableModel,
        y: usize,
        downward: &[usize],
    ) -> TableResult<()> {
        for index in downward {
            self.charge(1)?;
            let index = *index;
            let height = self.table_add(y - model.cells[index].y, 1)?;
            self.charge(1)?;
            model.cells[index].height = height;
        }
        Ok(())
    }
    fn table_end_group(
        &mut self,
        model: &mut HtmlTableModel,
        y: &mut usize,
        downward: &mut Vec<usize>,
    ) -> TableResult<()> {
        while *y < model.height() {
            self.table_grow(model, *y, downward)?;
            self.charge(1)?;
            *y = self.table_add(*y, 1)?;
        }
        self.charge(1)?;
        downward.clear();
        Ok(())
    }
    /// Drops the cells that no longer reach row `y` from the active line and
    /// repairs its running reach. Growing cells were extended to `y` first.
    fn table_expire(
        &mut self,
        model: &HtmlTableModel,
        y: usize,
        active: &mut Vec<SlotEntry>,
    ) -> TableResult<()> {
        self.charge(active.len().saturating_add(1))?;
        let mut reach = 0;
        active.retain_mut(|entry| {
            let live = model
                .cells
                .get(entry.cell)
                .is_some_and(|cell| cell.y.saturating_add(cell.height) > y);
            if live {
                reach = reach.max(entry.end);
                entry.reach = reach;
            }
            live
        });
        Ok(())
    }
    /// Records the recovered overlap errors of a cell about to be placed at
    /// `[x, right)` of row `y`: one per earlier cell that shares a slot with it,
    /// in model order. Every such cell covers row `y`, because rows are placed
    /// in order and a cell reaches only downward from its own row.
    fn table_overlaps(
        &mut self,
        model: &mut HtmlTableModel,
        cell: &HtmlCaptureSubjectId,
        (x, y, right): (usize, usize, usize),
        place: &mut Placement,
    ) -> TableResult<()> {
        let (first, last, work) = slots::overlap_window(&place.active, x, right);
        self.charge(work)?;
        let window = place.active.get(first..last).unwrap_or_default();
        let mut overlapping = 0;
        for entry in window {
            self.charge(1)?;
            if entry.end > x {
                overlapping += 1;
            }
        }
        place.hits.clear();
        self.reserve_scratch(&mut place.hits, overlapping)?;
        for entry in window {
            if entry.end > x {
                place.hits.push(entry.cell);
            }
        }
        // Sorting charges one unit per element for each bit of the count.
        let bits = overlapping
            .checked_ilog2()
            .and_then(|bits| usize::try_from(bits).ok())
            .unwrap_or(0);
        self.charge(overlapping.saturating_mul(1 + bits))?;
        place.hits.sort_unstable();
        for &index in &place.hits {
            self.charge(1)?;
            let Some(other) = model.cells.get(index) else {
                return Err(self
                    .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                    .into());
            };
            let error = HtmlTableModelError::Overlap {
                cell: cell.clone(),
                other: other.subject.clone(),
                x: x.max(other.x),
                y: y.max(other.y),
            };
            self.table_push(&mut model.errors, error)?;
        }
        Ok(())
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
        self.table_grow(model, *y, &place.downward)?;
        self.table_expire(model, *y, &mut place.active)?;
        let mut children = self.table_children(row)?;
        let mut x = 0;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            if !matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th)) {
                continue;
            }
            let (free, work) = slots::first_free(&place.active, x, model.width());
            self.charge(work)?;
            x = free;
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
            self.table_overlaps(model, &id, (x, *y, right), place)?;
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
            self.reserve_scratch(&mut place.active, 1)?;
            let work = slots::insert(
                &mut place.active,
                SlotEntry {
                    start: x,
                    end: right,
                    cell: index,
                    reach: right,
                    header: NO_HEADER,
                },
            );
            self.charge(work)?;
            if grows {
                self.table_scratch(&mut place.downward, index)?;
            }
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
        self.table_end_group(model, y, &mut place.downward)
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
                    self.table_end_group(&mut model, &mut y, &mut scratch.placement.downward)?;
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
        self.table_unanchored(&mut model, &mut scratch.flags)?;
        self.table_headers(&mut model, document, scratch)?;
        self.poll()?;
        Ok(model)
    }
    /// Reports the rows, then the columns, in which no cell has its top-left
    /// slot. One pass over the cells marks the anchored lines.
    fn table_unanchored(
        &mut self,
        model: &mut HtmlTableModel,
        flags: &mut Vec<bool>,
    ) -> TableResult<()> {
        for vertical in [false, true] {
            let lines = if vertical {
                model.width()
            } else {
                model.height()
            };
            self.table_flags(flags, lines)?;
            for cell in &model.cells {
                self.charge(1)?;
                let line = if vertical { cell.x } else { cell.y };
                if let Some(mark) = flags.get_mut(line) {
                    *mark = true;
                }
            }
            for line in 0..lines {
                self.charge(1)?;
                if !flags.get(line).copied().unwrap_or(true) {
                    let error = if vertical {
                        HtmlTableModelError::UnanchoredColumn { x: line }
                    } else {
                        HtmlTableModelError::UnanchoredRow { y: line }
                    };
                    self.table_push(&mut model.errors, error)?;
                }
            }
        }
        Ok(())
    }
    /// Admits and clears `lines + 1` flags: one per row or column and one past
    /// the last.
    fn table_flags(&mut self, flags: &mut Vec<bool>, lines: usize) -> TableResult<()> {
        let count = self.table_add(lines, 1)?;
        flags.clear();
        self.reserve_scratch(flags, count)?;
        flags.resize(count, false);
        self.charge(count)?;
        Ok(())
    }
    /// Builds the row and column lines of the finished model: the cells that
    /// cover a run of identical rows as intervals of columns sorted by start,
    /// and for each run of identical columns the intervals of rows. The entry
    /// tables hold one record per line a cell crosses, admitted from the
    /// cells' spans before anything is allocated.
    fn table_axes(&mut self, model: &HtmlTableModel, scratch: &mut Scratch<'_>) -> TableResult<()> {
        self.table_axis(model, true, scratch)?;
        self.table_axis(model, false, scratch)
    }
    /// One axis. Its lines are filled back to front so that each line lists
    /// its cells in ascending order of start: columns take the cells in model
    /// order, which is ascending in row; rows take them ordered by column.
    fn table_axis(
        &mut self,
        model: &HtmlTableModel,
        vertical: bool,
        scratch: &mut Scratch<'_>,
    ) -> TableResult<()> {
        let Scratch {
            flags,
            order,
            counts,
            columns,
            rows,
            ..
        } = scratch;
        let axis = if vertical { columns } else { rows };
        let extent = if vertical {
            model.width()
        } else {
            model.height()
        };
        // Where a cell starts and how far it reaches across this axis, and the
        // interval it covers along each line.
        let span = |cell: &HtmlTableCell| {
            if vertical {
                (cell.x, cell.width, cell.y, cell.height)
            } else {
                (cell.y, cell.height, cell.x, cell.width)
            }
        };
        // A new line begins wherever a cell starts or ends.
        self.table_flags(flags, extent)?;
        if let Some(first) = flags.first_mut() {
            *first = true;
        }
        for cell in &model.cells {
            self.charge(1)?;
            let (first, count, ..) = span(cell);
            let end = self.table_add(first, count)?;
            for position in [first, end] {
                if let Some(mark) = flags.get_mut(position) {
                    *mark = true;
                }
            }
        }
        let mut runs = 0;
        for position in 0..extent {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                runs += 1;
            }
        }
        self.reserve_scratch(&mut axis.breaks, runs)?;
        for position in 0..extent {
            self.charge(1)?;
            if flags.get(position).copied().unwrap_or(false) {
                axis.breaks.push(position);
            }
        }
        let closing = self.table_add(runs, 1)?;
        self.reserve_scratch(&mut axis.lines, closing)?;
        axis.lines.resize(closing, SlotLine::default());
        self.charge(closing)?;
        let mut total = 0usize;
        for cell in &model.cells {
            let (first, count, ..) = span(cell);
            let (low, high, probes) = axis.lines_of(first, first.saturating_add(count));
            self.charge(probes + 1)?;
            total = self.table_add(total, high - low)?;
            for line in low..high {
                self.charge(1)?;
                if let Some(line) = axis.lines.get_mut(line) {
                    line.offset += 1;
                }
            }
        }
        self.reserve_scratch(&mut axis.entries, total)?;
        let mut running = 0;
        for line in axis.lines.iter_mut() {
            self.charge(1)?;
            running += line.offset;
            line.offset = running;
        }
        axis.entries.resize(total, SlotEntry::default());
        self.charge(total)?;
        if vertical {
            for index in (0..model.cells.len()).rev() {
                self.table_fill(model, index, vertical, axis)?;
            }
        } else {
            let width = model.width();
            let counted = self.table_add(width, 1)?;
            self.reserve_scratch(counts, counted)?;
            counts.resize(counted, 0);
            self.reserve_scratch(order, model.cells.len())?;
            order.resize(model.cells.len(), 0);
            self.charge(counted)?;
            for cell in &model.cells {
                self.charge(1)?;
                if let Some(count) = counts.get_mut(cell.x + 1) {
                    *count += 1;
                }
            }
            let mut running = 0;
            for count in counts.iter_mut() {
                self.charge(1)?;
                running += *count;
                *count = running;
            }
            // `counts[x]` is now the number of cells left of column `x`.
            for (index, cell) in model.cells.iter().enumerate() {
                self.charge(1)?;
                if let Some(slot) = counts.get_mut(cell.x)
                    && let Some(place) = order.get_mut(*slot)
                {
                    *place = index;
                    *slot += 1;
                }
            }
            for position in (0..order.len()).rev() {
                let index = order.get(position).copied().unwrap_or(0);
                self.table_fill(model, index, vertical, axis)?;
            }
        }
        // Finish the lines: the running reach, whether the line's entries are
        // disjoint, the nearest header at or before each entry, and whether
        // any entry is a data cell.
        for line in 0..runs {
            let first = axis.lines.get(line).map_or(0, |l| l.offset);
            let last = axis.lines.get(line + 1).map_or(0, |l| l.offset);
            let (mut reach, mut header, mut regular, mut data) = (0, NO_HEADER, true, false);
            for position in first..last {
                self.charge(1)?;
                let Some(entry) = axis.entries.get_mut(position) else {
                    break;
                };
                if position > first && entry.start < reach {
                    regular = false;
                }
                reach = reach.max(entry.end);
                entry.reach = reach;
                if model
                    .cells
                    .get(entry.cell)
                    .is_some_and(|cell| cell.kind == HtmlTableCellKind::Header)
                {
                    header = position;
                } else {
                    data = true;
                }
                entry.header = header;
            }
            if let Some(line) = axis.lines.get_mut(line) {
                line.regular = regular;
                line.data = data;
            }
        }
        Ok(())
    }
    /// Places cell `index` at the back of each line it crosses.
    fn table_fill(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        vertical: bool,
        axis: &mut SlotAxis,
    ) -> TableResult<()> {
        let Some(cell) = model.cells.get(index) else {
            return Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into());
        };
        let (first, count, start, extent) = if vertical {
            (cell.x, cell.width, cell.y, cell.height)
        } else {
            (cell.y, cell.height, cell.x, cell.width)
        };
        let end = self.table_add(start, extent)?;
        let (low, high, probes) = axis.lines_of(first, first.saturating_add(count));
        self.charge(probes + 1)?;
        for line in low..high {
            self.charge(1)?;
            if let Some(line) = axis.lines.get_mut(line)
                && let Some(place) = line.offset.checked_sub(1)
                && let Some(slot) = axis.entries.get_mut(place)
            {
                line.offset = place;
                *slot = SlotEntry {
                    start,
                    end,
                    cell: index,
                    reach: end,
                    header: NO_HEADER,
                };
            }
        }
        Ok(())
    }
    /// The th scope state of cell `index`, resolved through the HTML-owned rule
    /// the first time this query consults it and reused afterwards. The first
    /// consultation admits the memo table (one entry per cell) and the rule's
    /// published bound for the raw value; later ones charge a single lookup.
    fn table_scope(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<HtmlTableHeaderScopeState> {
        if memo.len() != model.cells.len() {
            self.reserve_scratch(memo, model.cells.len())?;
            self.charge(model.cells.len())?;
            memo.resize(model.cells.len(), HeaderMemo::default());
        }
        self.charge(1)?;
        if let Some(state) = memo.get(index).and_then(|m| m.scope) {
            return Ok(state);
        }
        let element = self.table_element(&model.cells[index].subject)?;
        let raw = self.table_raw(element, "scope")?;
        let value = raw.map(Attribute::value);
        self.table_charge_bound(th_scope_work_bound(value.map_or(0, DocumentString::len)))?;
        let state = th_scope_state(value);
        self.poll()?;
        if let Some(entry) = memo.get_mut(index) {
            entry.scope = Some(state);
        }
        Ok(state)
    }
    /// Whether header `index` is a column header: its `scope` says column, or
    /// it is auto and no data cell shares a row with it. The auto answer is
    /// read from the row lines once per header.
    fn table_column_header(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        rows: &SlotAxis,
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<bool> {
        match self.table_scope(model, index, memo)? {
            HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Column) => Ok(true),
            HtmlTableHeaderScopeState::Auto => {
                self.charge(1)?;
                if let Some(known) = memo.get(index).and_then(|m| m.column) {
                    return Ok(known);
                }
                let header = &model.cells[index];
                let bottom = self.table_add(header.y, header.height)?;
                let (first, last, probes) = rows.lines_of(header.y, bottom);
                self.charge(probes)?;
                let mut shared = false;
                for row in first..last {
                    self.charge(1)?;
                    if rows.lines.get(row).is_some_and(|line| line.data) {
                        shared = true;
                        break;
                    }
                }
                if let Some(entry) = memo.get_mut(index) {
                    entry.column = Some(!shared);
                }
                Ok(!shared)
            }
            _ => Ok(false),
        }
    }
    /// Whether header `index` is a row header: its `scope` says row, or it is
    /// auto, not a column header, and no data cell shares a column with it.
    fn table_row_header(
        &mut self,
        model: &HtmlTableModel,
        index: usize,
        (rows, columns): (&SlotAxis, &SlotAxis),
        memo: &mut Vec<HeaderMemo>,
    ) -> TableResult<bool> {
        match self.table_scope(model, index, memo)? {
            HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Row) => Ok(true),
            HtmlTableHeaderScopeState::Auto
                if !self.table_column_header(model, index, rows, memo)? =>
            {
                self.charge(1)?;
                if let Some(known) = memo.get(index).and_then(|m| m.row) {
                    return Ok(known);
                }
                let header = &model.cells[index];
                let right = self.table_add(header.x, header.width)?;
                let (first, last, probes) = columns.lines_of(header.x, right);
                self.charge(probes)?;
                let mut shared = false;
                for column in first..last {
                    self.charge(1)?;
                    if columns.lines.get(column).is_some_and(|line| line.data) {
                        shared = true;
                        break;
                    }
                }
                if let Some(entry) = memo.get_mut(index) {
                    entry.row = Some(!shared);
                }
                Ok(!shared)
            }
            _ => Ok(false),
        }
    }
    /// A data cell reached by a scan ends the header block it follows: the
    /// block's cells become opaque to earlier headers of the same extent.
    fn table_end_block(&mut self, scan: &mut Scan<'_>) -> TableResult<()> {
        if !scan.in_block {
            return Ok(());
        }
        scan.in_block = false;
        for index in scan.block.iter() {
            self.charge(1)?;
            self.table_scratch(scan.opaque, *index)?;
        }
        self.charge(1)?;
        scan.block.clear();
        Ok(())
    }
    /// A header cell reached by a scan joins the current block, and is a
    /// header of the principal cell unless an opaque header of the same extent
    /// hides it or it is not a header of the scan's kind.
    fn table_reach_header(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        index: usize,
        vertical: bool,
    ) -> TableResult<()> {
        scan.in_block = true;
        self.table_scratch(scan.block, index)?;
        let cell = &model.cells[index];
        let mut blocked = false;
        for opaque in scan.opaque.iter() {
            self.charge(1)?;
            let old = &model.cells[*opaque];
            if (vertical && old.x == cell.x && old.width == cell.width)
                || (!vertical && old.y == cell.y && old.height == cell.height)
            {
                blocked = true;
                break;
            }
        }
        if !blocked {
            blocked = if vertical {
                !self.table_column_header(model, index, scan.rows, scan.memo)?
            } else {
                !self.table_row_header(model, index, (scan.rows, scan.columns), scan.memo)?
            };
        }
        if !blocked {
            self.table_scratch(scan.headers, index)?;
        }
        Ok(())
    }
    /// Scans from the principal cell toward the start of one row (leftward,
    /// from column `from`) or one column (upward, from row `from`), taking the
    /// occupant of each slot in turn. A slot with no occupant or several is
    /// skipped. A line whose intervals are disjoint lists its occupants in
    /// order, so the scan visits each cell once and jumps over runs of data
    /// cells: reaching one only ends the current block, however many follow.
    /// Visiting a cell once instead of once per slot it covers changes
    /// nothing, because a repeated visit re-adds a header already found
    /// (duplicates are dropped when associations are made), a block member
    /// already present, and a data cell with no block left to end.
    fn table_scan(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        principal: usize,
        (line, from): (usize, usize),
        vertical: bool,
    ) -> TableResult<()> {
        self.charge(1)?;
        scan.opaque.clear();
        scan.block.clear();
        scan.in_block = model.cells[principal].kind == HtmlTableCellKind::Header;
        if scan.in_block {
            self.table_scratch(scan.block, principal)?;
        }
        let axis = if vertical { scan.columns } else { scan.rows };
        let Some((entries, facts, base)) = axis.line(line) else {
            return Err(self
                .stop(HtmlSemanticFailureKind::IntrinsicUnavailable)
                .into());
        };
        if facts.regular {
            let (mut before, probes) = slots::leading(entries, |entry| entry.start < from);
            self.charge(probes + 1)?;
            while before > 0 {
                self.charge(1)?;
                let last = before - 1;
                let reached = entries.get(last).map_or(NO_HEADER, |entry| entry.header);
                let Some(position) = reached.checked_sub(base).filter(|_| reached != NO_HEADER)
                else {
                    // Every remaining entry is a data cell.
                    return self.table_end_block(scan);
                };
                if position < last {
                    self.table_end_block(scan)?;
                }
                let cell = entries.get(position).map_or(principal, |entry| entry.cell);
                self.table_reach_header(model, scan, cell, vertical)?;
                before = position;
            }
            return Ok(());
        }
        let mut slot = from;
        loop {
            self.charge(1)?;
            if slot == 0 {
                return Ok(());
            }
            slot -= 1;
            let (cover, work) = slots::cover(entries, slot);
            self.charge(work)?;
            let Cover::One(index) = cover else {
                continue;
            };
            if model.cells[index].kind == HtmlTableCellKind::Header {
                self.table_reach_header(model, scan, index, vertical)?;
            } else {
                self.table_end_block(scan)?;
            }
        }
    }
    fn table_id_match(&mut self, left: &[u16], right: &[u16]) -> TableResult<bool> {
        self.charge(1)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.charge(2)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn table_find_id(
        &mut self,
        document: &HtmlCaptureSubjectId,
        token: &[u16],
        frames: &mut Vec<HtmlCapturedChildIds<'a>>,
    ) -> TableResult<Option<HtmlCaptureSubjectId>> {
        self.charge(1)?;
        frames.clear();
        let children = self.table_children(document)?;
        self.table_scratch(frames, children)?;
        while !frames.is_empty() {
            self.charge(1)?;
            let Some(id) = frames.last_mut().and_then(Iterator::next) else {
                frames.pop();
                continue;
            };
            let node = self.node(&id)?;
            if let HtmlCapturedNodeKind::Element(element) = node.kind() {
                let raw = self.table_raw(element, "id")?;
                if let Some(raw) = raw
                    && !raw.value().is_empty()
                    && self.table_id_match(raw.value().code_units(), token)?
                {
                    return Ok(Some(id));
                }
            }
            let children = self.table_children(&id)?;
            if children.len() != 0 {
                self.table_scratch(frames, children)?;
            }
        }
        Ok(None)
    }
    fn table_empty(&mut self, model: &HtmlTableModel, index: usize) -> TableResult<bool> {
        let mut children = self.table_children(&model.cells[index].subject)?;
        loop {
            self.charge(1)?;
            let Some(id) = children.next() else {
                return Ok(true);
            };
            match self.node(&id)?.kind() {
                HtmlCapturedNodeKind::Element(_) => return Ok(false),
                HtmlCapturedNodeKind::Text(text) => {
                    for unit in text.code_units() {
                        self.charge(1)?;
                        if !is_ascii_whitespace(*unit) {
                            return Ok(false);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn table_headers(
        &mut self,
        model: &mut HtmlTableModel,
        document: &HtmlCaptureSubjectId,
        scratch: &mut Scratch<'a>,
    ) -> TableResult<()> {
        if model.cells.is_empty() {
            return Ok(());
        }
        self.table_axes(model, scratch)?;
        if !model.row_groups.is_empty() || !model.column_groups.is_empty() {
            for (index, cell) in model.cells.iter().enumerate() {
                if cell.kind == HtmlTableCellKind::Header {
                    self.table_scratch(&mut scratch.header_cells, index)?;
                }
            }
        }
        let Scratch {
            document: frames,
            headers,
            opaque,
            block,
            memo,
            header_cells,
            columns,
            rows,
            ..
        } = scratch;
        let mut scan = Scan {
            headers,
            opaque,
            block,
            memo,
            rows,
            columns,
            in_block: false,
        };
        for principal in 0..model.cells.len() {
            self.charge(1)?;
            scan.headers.clear();
            let cell = &model.cells[principal];
            let (cell_x, cell_y) = (cell.x, cell.y);
            let element = self.table_element(&cell.subject)?;
            if let Some(raw) = self.table_raw(element, "headers")? {
                let units = raw.value().code_units();
                let mut position = 0;
                while position < units.len() {
                    self.charge(1)?;
                    if is_ascii_whitespace(units[position]) {
                        position += 1;
                        continue;
                    }
                    let start = position;
                    while position < units.len() {
                        self.charge(1)?;
                        if is_ascii_whitespace(units[position]) {
                            break;
                        }
                        position += 1;
                    }
                    if let Some(id) =
                        self.table_find_id(document, &units[start..position], frames)?
                    {
                        for (index, cell) in model.cells.iter().enumerate() {
                            self.charge(1)?;
                            if cell.subject == id && index != principal {
                                self.table_scratch(scan.headers, index)?;
                                break;
                            }
                        }
                    }
                }
            } else {
                let right = self.table_add(cell_x, cell.width)?;
                let bottom = self.table_add(cell_y, cell.height)?;
                // Rows (or columns) of one line are covered by the same cells in
                // the same way, so one scan of the line serves all of them.
                let (first, last, probes) = scan.rows.lines_of(cell_y, bottom);
                self.charge(probes)?;
                for line in first..last {
                    self.table_scan(model, &mut scan, principal, (line, cell_x), false)?;
                }
                let (first, last, probes) = scan.columns.lines_of(cell_x, right);
                self.charge(probes)?;
                for line in first..last {
                    self.table_scan(model, &mut scan, principal, (line, cell_y), true)?;
                }
                self.table_group_scopes(
                    model,
                    &mut scan,
                    header_cells,
                    principal,
                    (right, bottom),
                )?;
            }
            for (position, header) in scan.headers.iter().enumerate() {
                self.charge(1)?;
                let header = *header;
                if self.table_empty(model, header)? {
                    continue;
                }
                let mut duplicate = false;
                for earlier in &scan.headers[..position] {
                    self.charge(1)?;
                    if *earlier == header {
                        duplicate = true;
                        break;
                    }
                }
                if duplicate || header == principal {
                    continue;
                }
                self.table_push(
                    &mut model.headers,
                    HtmlTableHeaderAssociation {
                        cell: model.cells[principal].subject.clone(),
                        header: model.cells[header].subject.clone(),
                    },
                )?;
            }
        }
        Ok(())
    }
    /// The headers a row group or column group gives its cells: a th whose
    /// `scope` is `rowgroup` (or `colgroup`) in the principal cell's group and
    /// not below or right of the principal cell's far edges. Row groups and
    /// column groups are disjoint and sorted, so one group at most holds the
    /// principal cell, and the header cells sorted by row bound the candidates.
    fn table_group_scopes(
        &mut self,
        model: &HtmlTableModel,
        scan: &mut Scan<'_>,
        header_cells: &[usize],
        principal: usize,
        (right, bottom): (usize, usize),
    ) -> TableResult<()> {
        let Some(cell) = model.cells.get(principal) else {
            return Ok(());
        };
        let (cell_x, cell_y) = (cell.x, cell.y);
        let (group, probes) = slots::containing(&model.row_groups, cell_y, |g| (g.y, g.height));
        self.charge(probes)?;
        if let Some(group) = group.and_then(|index| model.row_groups.get(index)) {
            let limit = self.table_add(group.y, group.height)?.min(bottom);
            let (first, a) = slots::leading_by(header_cells.len(), |i| {
                model.cells[header_cells[i]].y < group.y
            });
            let (last, b) = slots::leading_by(header_cells.len(), |i| {
                model.cells[header_cells[i]].y < limit
            });
            self.charge(a + b)?;
            for &index in header_cells.get(first..last).unwrap_or_default() {
                self.charge(1)?;
                if model.cells[index].x < right
                    && self.table_scope(model, index, scan.memo)?
                        == HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::RowGroup)
                {
                    self.table_scratch(scan.headers, index)?;
                }
            }
        }
        let (group, probes) = slots::containing(&model.column_groups, cell_x, |g| (g.x, g.width));
        self.charge(probes)?;
        if let Some(group) = group.and_then(|index| model.column_groups.get(index)) {
            let (last, probes) = slots::leading_by(header_cells.len(), |i| {
                model.cells[header_cells[i]].y < bottom
            });
            self.charge(probes)?;
            for &index in header_cells.get(..last).unwrap_or_default() {
                self.charge(1)?;
                let header = &model.cells[index];
                if header.x >= group.x
                    && header.x - group.x < group.width
                    && header.x < right
                    && self.table_scope(model, index, scan.memo)?
                        == HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::ColumnGroup)
                {
                    self.table_scratch(scan.headers, index)?;
                }
            }
        }
        Ok(())
    }
}

/// The scratch a header scan works in: the headers found for the principal
/// cell, the block and opaque cells of the scan in progress, what is known of
/// each header, and the finished row and column lines.
struct Scan<'s> {
    headers: &'s mut Vec<usize>,
    opaque: &'s mut Vec<usize>,
    block: &'s mut Vec<usize>,
    memo: &'s mut Vec<HeaderMemo>,
    rows: &'s SlotAxis,
    columns: &'s SlotAxis,
    in_block: bool,
}

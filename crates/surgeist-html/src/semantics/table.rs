//! HTML 877d6146487f §4.9.12: slots, associations and recoverable model errors.
//! Ordinary capture relations are the only topology; no CSS or live tree occurs.

mod collections;
pub use collections::*;

use super::*;
use crate::{HtmlCapturedChildIds, numeric};

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
    fn covers(&self, x: usize, y: usize) -> bool {
        x >= self.x && x - self.x < self.width && y >= self.y && y - self.y < self.height
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
    downward: Vec<usize>,
    headers: Vec<usize>,
    opaque: Vec<usize>,
    block: Vec<usize>,
    document: Vec<HtmlCapturedChildIds<'a>>,
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum Scope {
    Auto,
    Row,
    Column,
    RowGroup,
    ColumnGroup,
}

impl<'a> HtmlSemanticSession<'a> {
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
        for capacity in [
            scratch.footers.capacity(),
            scratch.downward.capacity(),
            scratch.headers.capacity(),
            scratch.opaque.capacity(),
            scratch.block.capacity(),
            scratch.document.capacity(),
        ] {
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
    fn table_span(
        &mut self,
        id: &HtmlCaptureSubjectId,
        local: &str,
        cap: usize,
        zero: bool,
    ) -> TableResult<usize> {
        let element = self.table_element(id)?;
        let Some(raw) = self.table_raw(element, local)? else {
            return Ok(1);
        };
        let work = raw
            .value()
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(8))
            .ok_or_else(|| {
                TableStop::Failure(self.stop(HtmlSemanticFailureKind::RepresentationExhausted))
            })?;
        self.charge(work)?;
        let value = numeric::parse_nonnegative_integer_capped(raw.value(), cap).unwrap_or(1);
        self.poll()?;
        Ok(if value == 0 && !zero { 1 } else { value })
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
                let span = self.table_span(&column, "span", 1000, false)?;
                let width = self.table_add(model.width(), span)?;
                self.table_width(model, width, Some(&column))?;
            }
        }
        if !has_column {
            let span = self.table_span(id, "span", 1000, false)?;
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
    fn table_occupant(
        &mut self,
        model: &HtmlTableModel,
        x: usize,
        y: usize,
    ) -> TableResult<Option<usize>> {
        let mut occupant = None;
        for (index, cell) in model.cells.iter().enumerate() {
            self.charge(1)?;
            if cell.covers(x, y) {
                if occupant.is_some() {
                    return Ok(None);
                }
                occupant = Some(index);
            }
        }
        Ok(occupant)
    }
    fn table_occupied(&mut self, model: &HtmlTableModel, x: usize, y: usize) -> TableResult<bool> {
        for cell in &model.cells {
            self.charge(1)?;
            if cell.covers(x, y) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn table_row(
        &mut self,
        row: &HtmlCaptureSubjectId,
        model: &mut HtmlTableModel,
        y: &mut usize,
        downward: &mut Vec<usize>,
    ) -> TableResult<()> {
        if model.height() == *y {
            let height = self.table_add(*y, 1)?;
            self.table_height(model, height)?;
        }
        self.charge(1)?;
        model.rows[*y].source = Some(row.clone());
        self.table_grow(model, *y, downward)?;
        let mut children = self.table_children(row)?;
        let mut x = 0;
        while let Some((id, kind)) = self.table_next(&mut children)? {
            if !matches!(kind, Some(KnownHtmlElement::Td | KnownHtmlElement::Th)) {
                continue;
            }
            while x < model.width() && self.table_occupied(model, x, *y)? {
                self.charge(1)?;
                x = self.table_add(x, 1)?;
            }
            if x == model.width() {
                let width = self.table_add(x, 1)?;
                self.table_width(model, width, None)?;
            }
            let width = self.table_span(&id, "colspan", 1000, false)?;
            let mut height = self.table_span(&id, "rowspan", 65534, true)?;
            let grows = height == 0;
            if grows {
                height = 1;
            }
            let right = self.table_add(x, width)?;
            let bottom = self.table_add(*y, height)?;
            self.table_width(model, right, None)?;
            self.table_height(model, bottom)?;
            for other in &model.cells {
                self.charge(1)?;
                let other_right = self.table_add(other.x, other.width)?;
                let other_bottom = self.table_add(other.y, other.height)?;
                if x < other_right && other.x < right && *y < other_bottom && other.y < bottom {
                    self.table_push(
                        &mut model.errors,
                        HtmlTableModelError::Overlap {
                            cell: id.clone(),
                            other: other.subject.clone(),
                            x: x.max(other.x),
                            y: (*y).max(other.y),
                        },
                    )?;
                }
            }
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
            if grows {
                self.table_scratch(downward, index)?;
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
        downward: &mut Vec<usize>,
    ) -> TableResult<()> {
        let start = model.height();
        let mut children = self.table_children(group)?;
        while let Some((row, kind)) = self.table_next(&mut children)? {
            if kind == Some(KnownHtmlElement::Tr) {
                self.table_row(&row, model, y, downward)?;
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
        self.table_end_group(model, y, downward)
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
                    self.table_row(&id, &mut model, &mut y, &mut scratch.downward)?;
                }
                Some(
                    KnownHtmlElement::Thead | KnownHtmlElement::Tbody | KnownHtmlElement::Tfoot,
                ) => {
                    columns_phase = false;
                    self.table_end_group(&mut model, &mut y, &mut scratch.downward)?;
                    if kind == Some(KnownHtmlElement::Tfoot) {
                        self.table_scratch(&mut scratch.footers, id)?;
                    } else {
                        self.table_group(&id, &mut model, &mut y, &mut scratch.downward)?;
                    }
                }
                _ => {}
            }
        }
        for footer in &scratch.footers {
            self.charge(1)?;
            self.table_group(footer, &mut model, &mut y, &mut scratch.downward)?;
        }
        for y in 0..model.height() {
            let mut anchored = false;
            for cell in &model.cells {
                self.charge(1)?;
                if cell.y == y {
                    anchored = true;
                    break;
                }
            }
            if !anchored {
                self.table_push(&mut model.errors, HtmlTableModelError::UnanchoredRow { y })?;
            }
        }
        for x in 0..model.width() {
            let mut anchored = false;
            for cell in &model.cells {
                self.charge(1)?;
                if cell.x == x {
                    anchored = true;
                    break;
                }
            }
            if !anchored {
                self.table_push(
                    &mut model.errors,
                    HtmlTableModelError::UnanchoredColumn { x },
                )?;
            }
        }
        self.table_headers(&mut model, document, scratch)?;
        self.poll()?;
        Ok(model)
    }
    fn table_scope(&mut self, model: &HtmlTableModel, index: usize) -> TableResult<Scope> {
        let element = self.table_element(&model.cells[index].subject)?;
        let Some(raw) = self.table_raw(element, "scope")? else {
            return Ok(Scope::Auto);
        };
        for (keyword, scope) in [
            ("row", Scope::Row),
            ("col", Scope::Column),
            ("rowgroup", Scope::RowGroup),
            ("colgroup", Scope::ColumnGroup),
        ] {
            if raw.value().len() != keyword.len() {
                continue;
            }
            let mut equal = true;
            for (unit, byte) in raw.value().code_units().iter().zip(keyword.bytes()) {
                self.charge(1)?;
                if *unit > 127 || (*unit as u8).to_ascii_lowercase() != byte {
                    equal = false;
                    break;
                }
            }
            if equal {
                return Ok(scope);
            }
        }
        Ok(Scope::Auto)
    }
    fn table_column_header(&mut self, model: &HtmlTableModel, index: usize) -> TableResult<bool> {
        match self.table_scope(model, index)? {
            Scope::Column => Ok(true),
            Scope::Auto => {
                let header = &model.cells[index];
                let bottom = self.table_add(header.y, header.height)?;
                for cell in &model.cells {
                    self.charge(1)?;
                    if cell.kind == HtmlTableCellKind::Data
                        && cell.y < bottom
                        && header.y < self.table_add(cell.y, cell.height)?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn table_row_header(&mut self, model: &HtmlTableModel, index: usize) -> TableResult<bool> {
        match self.table_scope(model, index)? {
            Scope::Row => Ok(true),
            Scope::Auto if !self.table_column_header(model, index)? => {
                let header = &model.cells[index];
                let right = self.table_add(header.x, header.width)?;
                for cell in &model.cells {
                    self.charge(1)?;
                    if cell.kind == HtmlTableCellKind::Data
                        && cell.x < right
                        && header.x < self.table_add(cell.x, cell.width)?
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn table_scan(
        &mut self,
        model: &HtmlTableModel,
        principal: usize,
        mut x: usize,
        mut y: usize,
        vertical: bool,
        scratch: &mut Scratch<'a>,
    ) -> TableResult<()> {
        self.charge(1)?;
        scratch.opaque.clear();
        scratch.block.clear();
        let mut in_block = model.cells[principal].kind == HtmlTableCellKind::Header;
        if in_block {
            self.table_scratch(&mut scratch.block, principal)?;
        }
        loop {
            self.charge(1)?;
            if vertical {
                if y == 0 {
                    break;
                }
                y -= 1;
            } else {
                if x == 0 {
                    break;
                }
                x -= 1;
            }
            let Some(index) = self.table_occupant(model, x, y)? else {
                continue;
            };
            let cell = &model.cells[index];
            if cell.kind == HtmlTableCellKind::Header {
                in_block = true;
                self.table_scratch(&mut scratch.block, index)?;
                let mut blocked = false;
                for opaque in &scratch.opaque {
                    self.charge(1)?;
                    let opaque = *opaque;
                    let old = &model.cells[opaque];
                    if (vertical && old.x == cell.x && old.width == cell.width)
                        || (!vertical && old.y == cell.y && old.height == cell.height)
                    {
                        blocked = true;
                        break;
                    }
                }
                if !blocked {
                    blocked = if vertical {
                        !self.table_column_header(model, index)?
                    } else {
                        !self.table_row_header(model, index)?
                    };
                }
                if !blocked {
                    self.table_scratch(&mut scratch.headers, index)?;
                }
            } else if in_block {
                in_block = false;
                for index in &scratch.block {
                    self.charge(1)?;
                    self.table_scratch(&mut scratch.opaque, *index)?;
                }
                self.charge(1)?;
                scratch.block.clear();
            }
        }
        Ok(())
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
                        if !table_whitespace(*unit) {
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
        for principal in 0..model.cells.len() {
            self.charge(1)?;
            scratch.headers.clear();
            let cell = &model.cells[principal];
            let element = self.table_element(&cell.subject)?;
            if let Some(raw) = self.table_raw(element, "headers")? {
                let units = raw.value().code_units();
                let mut position = 0;
                while position < units.len() {
                    self.charge(1)?;
                    if table_whitespace(units[position]) {
                        position += 1;
                        continue;
                    }
                    let start = position;
                    while position < units.len() {
                        self.charge(1)?;
                        if table_whitespace(units[position]) {
                            break;
                        }
                        position += 1;
                    }
                    if let Some(id) = self.table_find_id(
                        document,
                        &units[start..position],
                        &mut scratch.document,
                    )? {
                        for (index, cell) in model.cells.iter().enumerate() {
                            self.charge(1)?;
                            if cell.subject == id && index != principal {
                                self.table_scratch(&mut scratch.headers, index)?;
                                break;
                            }
                        }
                    }
                }
            } else {
                let right = self.table_add(cell.x, cell.width)?;
                let bottom = self.table_add(cell.y, cell.height)?;
                for y in cell.y..bottom {
                    self.table_scan(model, principal, cell.x, y, false, scratch)?;
                }
                for x in cell.x..right {
                    self.table_scan(model, principal, x, cell.y, true, scratch)?;
                }
                for group in &model.row_groups {
                    self.charge(1)?;
                    if cell.y >= group.y && cell.y - group.y < group.height {
                        for (index, header) in model.cells.iter().enumerate() {
                            self.charge(1)?;
                            if header.kind == HtmlTableCellKind::Header
                                && header.y >= group.y
                                && header.y - group.y < group.height
                                && header.x < right
                                && header.y < bottom
                                && self.table_scope(model, index)? == Scope::RowGroup
                            {
                                self.table_scratch(&mut scratch.headers, index)?;
                            }
                        }
                    }
                }
                for group in &model.column_groups {
                    self.charge(1)?;
                    if cell.x >= group.x && cell.x - group.x < group.width {
                        for (index, header) in model.cells.iter().enumerate() {
                            self.charge(1)?;
                            if header.kind == HtmlTableCellKind::Header
                                && header.x >= group.x
                                && header.x - group.x < group.width
                                && header.x < right
                                && header.y < bottom
                                && self.table_scope(model, index)? == Scope::ColumnGroup
                            {
                                self.table_scratch(&mut scratch.headers, index)?;
                            }
                        }
                    }
                }
            }
            for (position, header) in scratch.headers.iter().enumerate() {
                self.charge(1)?;
                let header = *header;
                if self.table_empty(model, header)? {
                    continue;
                }
                let mut duplicate = false;
                for earlier in &scratch.headers[..position] {
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
}

fn table_whitespace(unit: u16) -> bool {
    matches!(unit, 9 | 10 | 12 | 13 | 32)
}

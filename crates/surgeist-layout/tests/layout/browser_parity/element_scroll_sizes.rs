//! Requested DOM integer sizes, separate from legacy operational range spans.
use super::{
    ElementScrollSizeField, Error, Node, NumericComparisonSink, NumericMismatch, RootContext,
    Scalar, TestNode, layout,
};

const PROFILE: &str = "horizontal-zero-edge-divs";
const MAX_SIZE: u32 = i32::MAX as u32;

#[derive(Clone, Debug, PartialEq)]
pub struct ElementScrollSizeMismatch {
    path: String,
    field: ElementScrollSizeField,
    browser_value: u32,
    native_value: u32,
}
impl ElementScrollSizeMismatch {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub const fn field(&self) -> ElementScrollSizeField {
        self.field
    }
    pub const fn browser_value(&self) -> u32 {
        self.browser_value
    }
    pub const fn native_value(&self) -> u32 {
        self.native_value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sizes {
    client_width: u32,
    client_height: u32,
    scroll_width: u32,
    scroll_height: u32,
}
impl Sizes {
    fn validate(self, path: &str) -> Result<Self, Error> {
        if self.client_width > self.scroll_width || self.client_height > self.scroll_height {
            return Err(Error::new(format!(
                "{path}: element scroll sizes require client <= scroll"
            )));
        }
        Ok(self)
    }
    fn fields(self) -> [(ElementScrollSizeField, u32); 4] {
        [
            (ElementScrollSizeField::ClientWidth, self.client_width),
            (ElementScrollSizeField::ClientHeight, self.client_height),
            (ElementScrollSizeField::ScrollWidth, self.scroll_width),
            (ElementScrollSizeField::ScrollHeight, self.scroll_height),
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Row {
    source_path: String,
    sizes: Sizes,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Table {
    rows: Vec<Row>,
}

fn invalid(path: &str, detail: &str) -> Error {
    Error::new(format!(
        "{path}: unsupported or invalid element-scroll-sizes: {detail}"
    ))
}

fn source_paths(
    node: &Node,
    path: &str,
    parent_mode: &str,
    out: &mut Vec<String>,
) -> Result<(), Error> {
    let mode = node.style.get("writing-mode").unwrap_or(parent_mode);
    if node.style.get("source-tag") != Some("div")
        || node.inline_text.is_some()
        || node.prepared_occurrence.is_some()
        || node.prepared.is_some()
        || node.prepared_origin.is_some()
        || node.prepared_control.is_some()
        || node.inline_boundary.is_some()
        || node.atomic_inline_participation.is_some()
        || node.atomic_line_strut.is_some()
        || node.layout_ready_inline_root
        || node.atomic_paragraph_metrics.is_some()
        || node.anonymous_grid_text_wrapper
        || node.text.is_some() && !node.children.is_empty()
        || [
            "zoom",
            "transform",
            "translate",
            "rotate",
            "scale",
            "perspective",
        ]
        .iter()
        .any(|property| node.style.get(property).is_some())
        || mode != "horizontal-tb"
    {
        return Err(invalid(
            path,
            "profile requires an ordinary horizontal source-tag DIV box",
        ));
    }
    out.push(path.to_owned());
    for (index, child) in node.children.iter().enumerate() {
        source_paths(child, &format!("{path}.{index}"), mode, out)?;
    }
    Ok(())
}

fn integer(row: roxmltree::Node<'_, '_>, field: &str) -> Result<u32, Error> {
    let raw = row
        .attribute(field)
        .ok_or_else(|| invalid("element", &format!("missing {field}")))?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid(
            "element",
            &format!("{field} requires an unsigned decimal CSSOM long"),
        ));
    }
    raw.parse::<u32>()
        .ok()
        .filter(|value| *value <= MAX_SIZE)
        .ok_or_else(|| {
            invalid(
                "element",
                &format!("{field} exceeds nonnegative CSSOM long"),
            )
        })
}

pub(super) fn parse(
    test: roxmltree::Node<'_, '_>,
    root: &Node,
    context: &RootContext,
) -> Result<Option<Table>, Error> {
    let tables: Vec<_> = test
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "element-scroll-sizes")
        .collect();
    if tables.is_empty() {
        return Ok(None);
    }
    if tables.len() != 1
        || tables[0].parent() != Some(test)
        || tables[0].tag_name().namespace().is_some()
        || test.attribute("measurement-version") != Some("4")
        || !matches!(
            context,
            RootContext::Qualified(
                super::root_context::QualifiedRootContext::AbsoluteRootLocal { .. }
            )
        )
    {
        return Err(invalid(
            "element-scroll-sizes",
            "requires one V4 absolute-root-local sibling table",
        ));
    }
    let table = tables[0];
    if table.attributes().len() != 1
        || table.attribute("profile") != Some(PROFILE)
        || table
            .attributes()
            .any(|attribute| attribute.namespace().is_some())
        || table
            .children()
            .any(|node| node.is_text() && node.text().is_some_and(|text| !text.trim().is_empty()))
    {
        return Err(invalid(
            "element-scroll-sizes",
            "unknown profile or table payload",
        ));
    }
    let mut paths = Vec::new();
    source_paths(root, "0", "horizontal-tb", &mut paths)?;
    let mut rows = Vec::new();
    for row in table.children().filter(roxmltree::Node::is_element) {
        if !row.has_tag_name("element")
            || row.tag_name().namespace().is_some()
            || row.attributes().len() != 5
            || row.attributes().any(|attribute| {
                attribute.namespace().is_some()
                    || ![
                        "node-path",
                        "client-width",
                        "client-height",
                        "scroll-width",
                        "scroll-height",
                    ]
                    .contains(&attribute.name())
            })
            || row.children().any(|node| {
                node.is_element() || node.text().is_some_and(|text| !text.trim().is_empty())
            })
        {
            return Err(invalid(
                "element-scroll-sizes",
                "invalid element row payload",
            ));
        }
        let path = row
            .attribute("node-path")
            .ok_or_else(|| invalid("element", "missing node-path"))?;
        if paths.get(rows.len()).map(String::as_str) != Some(path) {
            return Err(invalid(
                path,
                "row must match the complete canonical source-order box census",
            ));
        }
        rows.push(Row {
            source_path: path.to_owned(),
            sizes: Sizes {
                client_width: integer(row, "client-width")?,
                client_height: integer(row, "client-height")?,
                scroll_width: integer(row, "scroll-width")?,
                scroll_height: integer(row, "scroll-height")?,
            }
            .validate(path)?,
        });
    }
    if rows.len() != paths.len() {
        return Err(invalid(
            "element-scroll-sizes",
            "rows must cover every source box",
        ));
    }
    Ok(Some(Table { rows }))
}

fn supported_overflow(overflow: layout::Overflow) -> bool {
    matches!(
        overflow,
        layout::Overflow::Visible | layout::Overflow::Clip | layout::Overflow::Hidden
    )
}

fn projected(path: &str, field: ElementScrollSizeField, value: Scalar) -> Result<u32, Error> {
    // Widen the represented extent exactly before rounding. In f32, adding0.5
    // first would already change16777215 into16777216.
    let rounded = f64::from(value).round();
    if !value.is_finite() || value < 0.0 || rounded < 0.0 || rounded > f64::from(MAX_SIZE) {
        return Err(invalid(
            path,
            &format!(
                "{} projection exceeds nonnegative CSSOM long",
                field.as_str()
            ),
        ));
    }
    Ok(rounded as u32)
}

fn project_box_input<'a>(path: &str, node: &'a TestNode) -> Result<&'a layout::NodeInput, Error> {
    let input = node
        .layout_input
        .as_box()
        .ok_or_else(|| invalid(path, "requires a Native box"))?;
    if input.display == layout::Display::None
        || input.display.is_inline_level()
        || input.item_is_table
        || input.item_is_replaced
        || input.writing_mode != layout::WritingMode::HorizontalTb
        || input.padding != layout::Edges::all(layout::Length::ZERO)
        || input.border != layout::Edges::all(layout::Length::ZERO)
        || input.scrollbar_gutter != layout::ScrollbarGutter::Auto
        || !supported_overflow(input.overflow.x())
        || !supported_overflow(input.overflow.y())
    {
        return Err(invalid(
            path,
            "requires non-inline Htb DIV with zero edges, Auto gutter and Visible/Clip/Hidden",
        ));
    }
    if !node.unrounded_present {
        return Err(invalid(path, "missing actual unrounded box output"));
    }
    Ok(input)
}

pub(super) fn compare_projected_box_sizes(
    path: &str,
    node: &TestNode,
    expected: [u32; 4],
) -> Result<(), Error> {
    let actual = project(path, node)?.fields().map(|(_, value)| value);
    if actual != expected {
        return Err(invalid(
            path,
            &format!("browser sizes {expected:?}, Native {actual:?}"),
        ));
    }
    Ok(())
}

fn project(path: &str, node: &TestNode) -> Result<Sizes, Error> {
    let input = project_box_input(path, node)?;
    let geometry = node
        .unrounded
        .scroll_geometry
        .ok_or_else(|| invalid(path, "missing actual unrounded canonical scroll geometry"))?;
    let gutters = geometry.gutters();
    let no_gutters = gutters.top().is_none_or(|rect| rect.size().height == 0.0)
        && gutters
            .bottom()
            .is_none_or(|rect| rect.size().height == 0.0)
        && gutters.left().is_none_or(|rect| rect.size().width == 0.0)
        && gutters.right().is_none_or(|rect| rect.size().width == 0.0);
    if geometry.flow_axes()
        != layout::FlowAxes::new(layout::WritingMode::HorizontalTb, input.direction)
        || !supported_overflow(geometry.used_overflow_x())
        || !supported_overflow(geometry.used_overflow_y())
        || !no_gutters
        || geometry.scrollbar_size() != layout::Size::ZERO
    {
        return Err(invalid(
            path,
            "actual flow/overflow/gutters exceed the requested profile",
        ));
    }
    let port = geometry.scrollport();
    let overflow = geometry.scrollable_overflow();
    let left = port.origin().x;
    let top = port.origin().y;
    let right = left + port.size().width;
    let bottom = top + port.size().height;
    let overflow_right = overflow.origin().x + overflow.size().width;
    let overflow_bottom = overflow.origin().y + overflow.size().height;
    if [
        left,
        top,
        right,
        bottom,
        overflow.origin().x,
        overflow.origin().y,
        overflow_right,
        overflow_bottom,
    ]
    .iter()
    .any(|value| !value.is_finite())
    {
        return Err(invalid(path, "nonfinite canonical edge arithmetic"));
    }
    let width = match input.direction {
        layout::Direction::Ltr => right.max(overflow_right) - left,
        layout::Direction::Rtl => right - left.min(overflow.origin().x),
    };
    Sizes {
        client_width: projected(path, ElementScrollSizeField::ClientWidth, port.size().width)?,
        client_height: projected(
            path,
            ElementScrollSizeField::ClientHeight,
            port.size().height,
        )?,
        scroll_width: projected(path, ElementScrollSizeField::ScrollWidth, width)?,
        scroll_height: projected(
            path,
            ElementScrollSizeField::ScrollHeight,
            bottom.max(overflow_bottom) - top,
        )?,
    }
    .validate(path)
}

impl Table {
    pub(super) fn validate_source(&self, root: &Node, context: &RootContext) -> Result<(), Error> {
        if !matches!(
            context,
            RootContext::Qualified(
                super::root_context::QualifiedRootContext::AbsoluteRootLocal { .. }
            )
        ) {
            return Err(invalid(
                "element-scroll-sizes",
                "requires current absolute-root-local authority",
            ));
        }
        context.validate_root(root)?;
        let mut paths = Vec::new();
        source_paths(root, "0", "horizontal-tb", &mut paths)?;
        if paths.len() != self.rows.len()
            || paths
                .iter()
                .zip(&self.rows)
                .any(|(path, row)| path != &row.source_path)
        {
            return Err(invalid(
                "element-scroll-sizes",
                "source changed after the complete box census was captured",
            ));
        }
        Ok(())
    }

    pub(super) fn compare_next(
        &self,
        index: usize,
        source_path: &str,
        path: &str,
        node: &TestNode,
        collapsed_flex: bool,
        sink: &mut NumericComparisonSink,
    ) -> Result<(), Error> {
        let row = self
            .rows
            .get(index)
            .filter(|row| row.source_path == source_path)
            .ok_or_else(|| {
                invalid(
                    source_path,
                    "Native traversal lost source box correspondence",
                )
            })?;
        let actual =
            if collapsed_flex {
                let input = project_box_input(path, node)?;
                if input.display != layout::Display::Flex
                    || input.flex_item_collapse != layout::FlexItemCollapse::Collapsed
                    || node.unrounded.kind
                        != super::observation::Kind::NonBox(layout::NonBoxKind::Hidden)
                    || !node.unrounded.matches_numeric_fields(
                        &layout::NodeOutput::with_source_index(node.unrounded.source_index),
                    )
                {
                    return Err(invalid(
                        path,
                        "requires the validated collapsed Flex source packet",
                    ));
                }
                Sizes {
                    client_width: 0,
                    client_height: 0,
                    scroll_width: 0,
                    scroll_height: 0,
                }
            } else {
                project(path, node)?
            };
        for ((field, native), (_, browser)) in actual.fields().into_iter().zip(row.sizes.fields()) {
            if let NumericComparisonSink::CollectAll { root, fields } = sink {
                let source = super::exact_differences::source_path_from_diagnostic(root, path)?;
                fields.push(super::exact_differences::ComparedField::element(
                    source, field, browser, native,
                ));
                continue;
            }
            if native == browser
                && !matches!(
                    sink,
                    NumericComparisonSink::PreparedRanges(_)
                        | NumericComparisonSink::PreparedAtomicPercentage(_)
                )
            {
                continue;
            }
            match sink {
                NumericComparisonSink::Strict => {
                    return Err(Error::new(format!(
                        "{path}: {} mismatch, expected {browser}, got {native}",
                        field.as_str()
                    )));
                }
                NumericComparisonSink::PreparedRanges(_)
                | NumericComparisonSink::PreparedAtomicPercentage(_) => {
                    return Err(invalid(
                        path,
                        "prepared Range comparison excludes element sizes",
                    ));
                }
                NumericComparisonSink::CollectAll { .. } => {
                    unreachable!("complete field was collected above")
                }
                NumericComparisonSink::CollectNumeric(mismatches) => mismatches.push(
                    NumericMismatch::ElementScrollSize(ElementScrollSizeMismatch {
                        path: path.to_owned(),
                        field,
                        browser_value: browser,
                        native_value: native,
                    }),
                ),
            }
        }
        Ok(())
    }
    pub(super) fn finish(&self, consumed: usize) -> Result<(), Error> {
        if consumed == self.rows.len() {
            Ok(())
        } else {
            Err(invalid(
                "element-scroll-sizes",
                "Native traversal did not consume every requested source box",
            ))
        }
    }
}

#[cfg(test)]
#[path = "element_scroll_sizes_tests.rs"]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::pinned_font;
use surgeist_layout as layout;
#[path = "prepared_source.rs"]
mod prepared_source;
use surgeist_layout::{ComputeInput, ComputeOutput, GridSpan, LayoutTree as _};

type Scalar = layout::Scalar;

#[derive(Clone, Copy, Debug)]
struct ComparisonTolerance {
    value: Scalar,
}

impl ComparisonTolerance {
    const fn browser_parity() -> Self {
        Self { value: 0.1 }
    }

    fn contains(self, delta: Scalar) -> bool {
        delta.abs() <= self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Golden {
    pub name: String,
    pub use_rounding: bool,
    pub viewport: Viewport,
    pub root: Node,
    pub expectations: Expectation,
}

impl Golden {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let raw = std::fs::read_to_string(path.as_ref())
            .map_err(|source| Error::new(source.to_string()))?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self, Error> {
        let document = roxmltree::Document::parse(raw).map_err(|source| Error {
            message: source.to_string(),
        })?;
        let test = document.root_element();
        expect_tag(test, "test")?;

        let viewport = one_child(test, "viewport")?;
        let input = one_child(test, "input")?;
        let expectations = one_child(test, "expectations")?;

        if test
            .descendants()
            .any(|node| node.has_tag_name("prepared-paragraph"))
            && test.attribute("measurement-version") != Some("2")
        {
            return Err(Error::new(
                "native source capture requires measurement-version=2",
            ));
        }
        let name = required_attr(test, "name")?.to_string();
        let root = parse_node(one_element_child(input)?)?;
        validate_fri06_c08r_explicit_input(&root, true)?;
        let expectations = parse_expectation(one_element_child(expectations)?)?;

        Ok(Self {
            name,
            use_rounding: parse_bool(test.attribute("use-rounding").unwrap_or("true"))?,
            viewport: Viewport {
                width: parse_available(required_attr(viewport, "width")?)?,
                height: parse_available(required_attr(viewport, "height")?)?,
                root_context: parse_root_context(viewport)?,
            },
            root,
            expectations,
        })
    }
}

pub fn fixture_files(relative_dir: &str) -> Result<Vec<PathBuf>, Error> {
    let root = fixture_root(relative_dir)?;
    fixture_files_in(&root, "xml")
}

pub fn fixture_files_in(root: &Path, extension: &str) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    collect_files_with_extension(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

pub const fn fixture_skip_policy_mentions_x_prefix() -> bool {
    false
}

pub const fn fixture_skip_policy_filters_unsupported_constructs() -> bool {
    false
}

fn fixture_root(relative_dir: &str) -> Result<PathBuf, Error> {
    let root = relative_crate_root()
        .ok_or_else(|| Error::new("failed to locate surgeist-layout crate from relative paths"))?;
    Ok(root.join("tests/layout/browser_parity").join(relative_dir))
}

fn relative_crate_root() -> Option<PathBuf> {
    [Path::new("."), Path::new("crates/surgeist-layout")]
        .into_iter()
        .find(|root| root.join("tests/layout/browser_parity/xml").is_dir())
        .map(Path::to_path_buf)
}

pub fn assert_surgeist_matches(golden: &Golden) -> Result<(), Error> {
    assert_surgeist_matches_with_endpoint_accounting(golden).map(|_| ())
}

pub fn assert_surgeist_matches_with_endpoint_accounting(
    golden: &Golden,
) -> Result<ComparisonResult, Error> {
    let mut tree = TestTree::from_golden(&golden.root)?;
    let available = layout::Size::new(
        to_layout_available(golden.viewport.width),
        to_layout_available(golden.viewport.height),
    );

    let request = root_request(available, golden.viewport.root_context)?;
    let batch = if tree.paragraphs.is_empty() {
        layout::compute_layout(&tree, 0, request)
    } else {
        let capture = layout::LayoutFlowInput::try_new(
            &tree,
            0,
            tree.paragraphs.clone(),
            surgeist_layout::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .map_err(|error| Error::new(format!("{}: capture failed: {error:?}", golden.name)))?;
        layout::compute_layout_with_flow(&capture, 0, request)
    }
    .map_err(|error| Error::new(format!("{}: layout failed: {error:?}", golden.name)))?;
    tree.apply_completed_batch(&batch);

    compare_expectation_with_endpoint_accounting(
        &tree,
        0,
        &golden.expectations,
        &golden.name,
        golden.use_rounding,
        batch.unrounded_inline_fragments(),
        batch.final_inline_fragments(),
    )
}

fn root_request(
    available: layout::Size<layout::Available>,
    root_context: RootContext,
) -> Result<layout::LayoutRootRequest, Error> {
    match root_context {
        RootContext::Root => layout::LayoutRootRequest::viewport(available),
        RootContext::FlexItem {
            parent_axes,
            host_inline_size,
        } => {
            let context = layout::FlexItemRootContext::under_viewport(available, parent_axes)
                .map_err(|error| Error::new(format!("invalid flex viewport: {error:?}")))?;
            let host_available = match parent_axes.inline_axis() {
                layout::PhysicalAxis::Horizontal => layout::Size::new(
                    layout::Available::Definite(host_inline_size),
                    layout::Available::MaxContent,
                ),
                layout::PhysicalAxis::Vertical => layout::Size::new(
                    layout::Available::MaxContent,
                    layout::Available::Definite(host_inline_size),
                ),
            };
            layout::LayoutRootRequest::flex_item_under_viewport(host_available, context)
        }
    }
    .map_err(|error| Error::new(format!("invalid layout root request: {error:?}")))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Available {
    Definite(Scalar),
    MinContent,
    MaxContent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub width: Available,
    pub height: Available,
    pub root_context: RootContext,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RootContext {
    Root,
    FlexItem {
        parent_axes: layout::FlowAxes,
        host_inline_size: Scalar,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub style: StyleAttrs,
    pub text: Option<String>,
    pub children: Vec<Node>,
    // Historical measured tuples remain schema data only, never production Text.
    inline_text: Option<MeasuredInlineText>,
    prepared_occurrence: Option<usize>,
    prepared_origin: Option<surgeist_text::PreparedOrigin>,
    prepared_control: Option<surgeist_text::PreparedControl>,
    prepared: Option<prepared_source::Capture>,
    atomic_inline_participation: Option<layout::AtomicInlineParticipation>,
    shape_bands: Option<Vec<FixtureShapeBand>>,
    inline_boundary: Option<layout::InlineBoundaryKind>,
    atomic_line_strut: Option<layout::InlineMetrics>,
    anonymous_grid_text_wrapper: bool,
    layout_ready_inline_root: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct MeasuredInlineText {
    segments: Vec<MeasuredSegment>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct MeasuredSegment {
    id: u64,
    extent: Scalar,
    metrics: layout::InlineMetrics,
    level: surgeist_text::BidiLevel,
    edge: surgeist_text::InlineWhitespaceEdge,
    following: MeasuredBreak,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum MeasuredBreak {
    Prohibited,
    Allowed,
    Mandatory,
    Replacement(Scalar),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FixtureShapeBand {
    band_minimum: Scalar,
    band_maximum: Scalar,
    response: FixtureShapeResponse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FixtureShapeResponse {
    Empty,
    Interval {
        minimum: Scalar,
        maximum: Scalar,
        originating_band: Option<(Scalar, Scalar)>,
    },
    Failure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Div,
    Text,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleAttrs {
    attrs: BTreeMap<String, String>,
}

impl StyleAttrs {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(String::as_str)
    }

    pub fn display(&self) -> Option<String> {
        self.get("display").map(ToOwned::to_owned)
    }

    pub fn width(&self) -> Option<String> {
        self.get("width").map(ToOwned::to_owned)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expectation {
    pub x: Option<Scalar>,
    pub y: Option<Scalar>,
    pub width: Option<Scalar>,
    pub height: Option<Scalar>,
    pub scroll_size: Option<Size>,
    pub fragments: Option<Vec<InlineFragmentExpectation>>,
    pub range_rectangles: Option<Vec<InlineRangeRectangleExpectation>>,
    pub browser_control: Option<BrowserControlExpectation>,
    pub children: Vec<Expectation>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineFragmentExpectation {
    pub source_segment_id: u64,
    pub line_index: usize,
    /// Historical numeric XML only; native Text has no fragment-table visual ordinal.
    pub historical_visual_index: Option<usize>,
    pub x: Scalar,
    pub y: Scalar,
    pub width: Scalar,
    pub height: Scalar,
    pub baseline_x: Scalar,
    pub baseline_y: Scalar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PhysicalStartEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InlineRangeRectangleExpectation {
    pub source_segment_id: u64,
    pub line_index: usize,
    pub physical_start_edge: PhysicalStartEdge,
    pub start: Scalar,
    pub advance: Scalar,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BrowserControlExpectation {
    pub source_index: usize,
    pub terminal_visual_slot: Option<usize>,
    pub previous_line: BrowserNeighborLine,
    pub next_line: BrowserNeighborLine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserNeighborLine {
    Absent,
    Unobserved,
    Earlier,
    Same,
    Later,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserControlComparisonField {
    NextLine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointUnobservableField {
    paths: Vec<String>,
    field: BrowserControlComparisonField,
    browser_value: BrowserNeighborLine,
    model_value: BrowserNeighborLine,
}

impl EndpointUnobservableField {
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    pub const fn field(&self) -> BrowserControlComparisonField {
        self.field
    }

    pub const fn browser_value(&self) -> BrowserNeighborLine {
        self.browser_value
    }

    pub const fn model_value(&self) -> BrowserNeighborLine {
        self.model_value
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComparisonResult {
    endpoint_unobservable_fields: Vec<EndpointUnobservableField>,
}

impl ComparisonResult {
    pub fn endpoint_unobservable_fields(&self) -> &[EndpointUnobservableField] {
        &self.endpoint_unobservable_fields
    }

    fn record_endpoint_unobservable(&mut self, field: EndpointUnobservableField) {
        if let Some(existing) = self
            .endpoint_unobservable_fields
            .iter_mut()
            .find(|existing| {
                existing.field == field.field
                    && existing.browser_value == field.browser_value
                    && existing.model_value == field.model_value
            })
        {
            existing.paths.extend(field.paths);
        } else {
            self.endpoint_unobservable_fields.push(field);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Size {
    pub width: Scalar,
    pub height: Scalar,
}

impl Size {
    pub const fn new(width: Scalar, height: Scalar) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

fn parse_node(xml: roxmltree::Node<'_, '_>) -> Result<Node, Error> {
    if xml.has_tag_name("inline-boundary") {
        return parse_inline_boundary_node(xml);
    }
    let kind = match xml.tag_name().name() {
        "div" => NodeKind::Div,
        "text" => NodeKind::Text,
        tag => return Err(Error::new(format!("unsupported input node `<{tag}>`"))),
    };

    if let Some(layout_input) = xml.attribute("layout-input") {
        if kind == NodeKind::Text && layout_input == "prepared-text" {
            if xml
                .attributes()
                .any(|attr| !["layout-input", "occurrence"].contains(&attr.name()))
                || xml.children().any(|child| {
                    child.is_element() || child.text().is_some_and(|value| !value.trim().is_empty())
                })
            {
                return Err(Error::new(
                    "prepared text leaf accepts only its original occurrence reference",
                ));
            }
            return Ok(Node {
                kind,
                style: StyleAttrs::default(),
                text: None,
                children: Vec::new(),
                inline_text: None,
                prepared_occurrence: Some(parse_inline_integer(xml, "occurrence")?),
                prepared_origin: None,
                prepared_control: None,
                prepared: None,
                atomic_inline_participation: None,
                shape_bands: None,
                inline_boundary: None,
                atomic_line_strut: None,
                anonymous_grid_text_wrapper: false,
                layout_ready_inline_root: false,
            });
        }
        if kind != NodeKind::Text || layout_input != "inline-text" {
            return Err(Error::new(format!(
                "invalid `layout-input` on `<{}>`: `{layout_input}`",
                xml.tag_name().name()
            )));
        }
        return parse_inline_text_node(xml);
    }

    let layout_ready_inline_root = match xml.attribute("layout-ready-inline-root") {
        Some(value) if parse_bool(value)? => true,
        Some(_) => {
            return Err(Error::new(
                "`layout-ready-inline-root` must be `true` when present",
            ));
        }
        None => false,
    };
    let anonymous_grid_text_wrapper =
        match xml.attribute("layout-ready-anonymous-grid-text-wrapper") {
            Some("true") => true,
            Some(_) => {
                return Err(Error::new(
                    "`layout-ready-anonymous-grid-text-wrapper` must be exactly `true`",
                ));
            }
            None => false,
        };
    let mut attrs = BTreeMap::new();
    for attr in xml.attributes() {
        if attr.name() == "layout-ready-anonymous-grid-text-wrapper" {
            continue;
        }
        if attr.name().starts_with("layout-ready-") && attr.name() != "layout-ready-inline-root" {
            return Err(Error::new(format!(
                "unsupported layout-ready input attribute `{}`",
                attr.name()
            )));
        }
        attrs.insert(attr.name().to_string(), attr.value().to_string());
    }

    let mut shape_bands = None;
    for table in xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name("shape-bands"))
    {
        if shape_bands.is_some() {
            return Err(Error::new(
                "expected at most one `<shape-bands>` child on an input node",
            ));
        }
        shape_bands = Some(parse_shape_bands(table)?);
    }

    if anonymous_grid_text_wrapper
        && xml.children().any(|child| {
            child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty())
        })
    {
        return Err(Error::new(
            "anonymous grid text wrapper rejects raw text fallback",
        ));
    }

    let text = xml
        .text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned);
    let mut children = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| {
            !child.has_tag_name("atomic-placeholder")
                && !child.has_tag_name("shape-bands")
                && !child.has_tag_name("prepared-paragraph")
        })
        .map(parse_node)
        .collect::<Result<Vec<_>, _>>()?;
    let mut atomic_indices = std::collections::BTreeSet::new();
    for placeholder in xml
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name("atomic-placeholder"))
    {
        let (child_index, participation) = parse_atomic_placeholder(placeholder)?;
        if !atomic_indices.insert(child_index) {
            return Err(Error::new(format!(
                "duplicate atomic child index `{child_index}`"
            )));
        }
        let child = children
            .get_mut(child_index)
            .ok_or_else(|| Error::new(format!("unmatched atomic child index `{child_index}`")))?;
        if child.inline_text.is_some() || child.atomic_inline_participation.is_some() {
            return Err(Error::new(format!(
                "unmatched atomic child index `{child_index}`"
            )));
        }
        child.atomic_inline_participation = Some(participation);
    }

    let captures: Vec<_> = xml
        .children()
        .filter(|child| child.has_tag_name("prepared-paragraph"))
        .collect();
    if captures.len() > 1 {
        return Err(Error::new(
            "only one prepared paragraph capture is allowed per container",
        ));
    }
    let prepared = captures
        .first()
        .map(|capture| {
            if capture.attributes().any(|attr| attr.name() != "capture")
                || capture.children().any(|child| {
                    child.is_element() || child.text().is_some_and(|text| !text.trim().is_empty())
                })
            {
                return Err(Error::new(
                    "prepared paragraph capture has unexpected fields or payload",
                ));
            }
            let capture = prepared_source::Capture::parse(
                required_attr(*capture, "capture")?,
                children.len(),
            )?;
            for (index, (child, origin)) in capture.origins.iter().enumerate() {
                if children[*child].prepared_occurrence != Some(index)
                    || children[*child].inline_text.is_some()
                {
                    return Err(Error::new(
                        "prepared source origin does not match its exact native leaf",
                    ));
                }
                children[*child].prepared_origin = Some(origin.clone());
            }
            for (child, control) in &capture.controls {
                children[*child].prepared_control = Some(control.clone());
            }
            Ok(capture)
        })
        .transpose()?;
    if prepared.is_none()
        && children
            .iter()
            .any(|child| child.prepared_occurrence.is_some())
    {
        return Err(Error::new("prepared leaf has no containing source capture"));
    }
    Ok(Node {
        prepared_occurrence: None,
        prepared_origin: None,
        prepared_control: None,
        prepared,
        kind,
        style: StyleAttrs { attrs },
        text,
        children,
        inline_text: None,
        atomic_inline_participation: None,
        shape_bands,
        inline_boundary: None,
        atomic_line_strut: None,
        anonymous_grid_text_wrapper,
        layout_ready_inline_root,
    })
}

fn parse_inline_boundary_node(xml: roxmltree::Node<'_, '_>) -> Result<Node, Error> {
    const ATTRIBUTES: [&str; 3] = ["kind", "inline-baseline", "inline-line-height"];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported inline boundary attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;
    let kind = match required_attr(xml, "kind")? {
        "start" => layout::InlineBoundaryKind::Start,
        "end" => layout::InlineBoundaryKind::End,
        raw => return Err(Error::new(format!("invalid inline boundary kind `{raw}`"))),
    };
    let baseline = xml.attribute("inline-baseline");
    let line_height = xml.attribute("inline-line-height");
    let atomic_line_strut = match (baseline, line_height) {
        (None, None) => None,
        (Some(_), None) | (None, Some(_)) => {
            return Err(Error::new(
                "inline boundary metrics require both `inline-baseline` and `inline-line-height`",
            ));
        }
        (Some(baseline), Some(line_height)) => {
            if kind != layout::InlineBoundaryKind::Start {
                return Err(Error::new("only a start inline boundary may carry metrics"));
            }
            let baseline = parse_number(baseline)?;
            let line_height = parse_number(line_height)?;
            if baseline < 0.0 || line_height <= 0.0 || baseline > line_height {
                return Err(Error::new(
                    "inline boundary metrics require 0 <= baseline <= positive line height",
                ));
            }
            Some(
                layout::InlineMetrics::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    baseline,
                    line_height,
                )
                .map_err(|error| {
                    Error::new(format!("invalid inline boundary metrics: {error:?}"))
                })?,
            )
        }
    };
    Ok(Node {
        prepared_occurrence: None,
        prepared_origin: None,
        prepared_control: None,
        prepared: None,
        kind: NodeKind::Div,
        style: StyleAttrs::default(),
        text: None,
        children: Vec::new(),
        inline_text: None,
        atomic_inline_participation: None,
        shape_bands: None,
        inline_boundary: Some(kind),
        atomic_line_strut,
        anonymous_grid_text_wrapper: false,
        layout_ready_inline_root: false,
    })
}

fn validate_native_control_topology(node: &Node) -> Result<(), Error> {
    use surgeist_text::ParagraphControlKind;
    let mut depth = 0_usize;
    for child in &node.children {
        let Some(control) = &child.prepared_control else {
            if child.inline_boundary.is_some() {
                return Err(Error::new(
                    "native inline boundary has no captured control occurrence",
                ));
            }
            validate_fri06_c08r_explicit_input(child, false)?;
            continue;
        };
        if child.atomic_line_strut.is_some()
            || child.prepared_occurrence.is_some()
            || child.inline_text.is_some()
            || child.atomic_inline_participation.is_some()
            || !child.children.is_empty()
        {
            return Err(Error::new(
                "native control cannot carry a second payload or invented strut",
            ));
        }
        match control.view().kind() {
            ParagraphControlKind::InlineBoundaryStart
                if child.inline_boundary == Some(layout::InlineBoundaryKind::Start) =>
            {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| Error::new("native boundary depth overflow"))?;
            }
            ParagraphControlKind::InlineBoundaryEnd
                if child.inline_boundary == Some(layout::InlineBoundaryKind::End) =>
            {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| Error::new("misplaced native end inline boundary"))?;
            }
            ParagraphControlKind::ForcedBreak
                if child.inline_boundary.is_none()
                    && child.style.get("source-tag") == Some("br")
                    && child.style.get("display") == Some("inline") => {}
            _ => {
                return Err(Error::new(
                    "captured control role does not match its exact native leaf",
                ));
            }
        }
    }
    if depth != 0 {
        return Err(Error::new("unclosed native start inline boundary"));
    }
    Ok(())
}

fn validate_fri06_c08r_explicit_input(node: &Node, is_root: bool) -> Result<bool, Error> {
    if node.inline_boundary.is_some() {
        if is_root {
            return Err(Error::new("inline boundary cannot be the fixture root"));
        }
        return Ok(true);
    }

    let mut explicit = node.anonymous_grid_text_wrapper;
    if node.anonymous_grid_text_wrapper
        && (!matches!(
            node.style.get("display"),
            Some("grid" | "inline-grid" | "grid-lanes" | "inline-grid-lanes")
        ) || node.text.is_some()
            || node.children.is_empty()
            || node.children.iter().any(|child| {
                child.inline_text.is_none()
                    || child.inline_boundary.is_some()
                    || child.atomic_inline_participation.is_some()
                    || !child.children.is_empty()
            }))
    {
        return Err(Error::new(
            "anonymous grid text wrapper requires only direct shaped-text children in a grid role",
        ));
    }

    let has_boundary = node
        .children
        .iter()
        .any(|child| child.inline_boundary.is_some());
    if has_boundary && !node.layout_ready_inline_root {
        return Err(Error::new(
            "explicit inline boundaries require a layout-ready containing root",
        ));
    }

    if node.prepared.is_some() {
        if !node.layout_ready_inline_root {
            return Err(Error::new(
                "native paragraph requires its explicit layout-ready containing root",
            ));
        }
        validate_native_control_topology(node)?;
        return Ok(true);
    }

    let mut index = 0;
    while index < node.children.len() {
        let child = &node.children[index];
        let Some(kind) = child.inline_boundary else {
            explicit |= validate_fri06_c08r_explicit_input(child, false)?;
            index += 1;
            continue;
        };
        explicit = true;
        match (kind, child.atomic_line_strut) {
            (layout::InlineBoundaryKind::Start, Some(_)) => {
                let target = node.children.get(index + 1).ok_or_else(|| {
                    Error::new("metric start inline boundary requires a following atomic child")
                })?;
                if target.atomic_inline_participation.is_none()
                    || target.inline_text.is_some()
                    || target.inline_boundary.is_some()
                {
                    return Err(Error::new(
                        "metric start inline boundary must immediately precede one typed atomic child",
                    ));
                }
                index += 1;
            }
            (layout::InlineBoundaryKind::Start, None) => {
                let text = node.children.get(index + 1).ok_or_else(|| {
                    Error::new(
                        "start inline boundary requires a typed-text payload and end boundary",
                    )
                })?;
                let end = node.children.get(index + 2).ok_or_else(|| {
                    Error::new(
                        "start inline boundary requires a typed-text payload and end boundary",
                    )
                })?;
                if (text.inline_text.is_none() && text.prepared_occurrence.is_none())
                    || text.inline_boundary.is_some()
                    || !text.children.is_empty()
                    || end.inline_boundary != Some(layout::InlineBoundaryKind::End)
                    || end.atomic_line_strut.is_some()
                {
                    return Err(Error::new(
                        "transparent inline boundary topology must be start, typed text, end",
                    ));
                }
                index += 2;
            }
            (layout::InlineBoundaryKind::End, _) => {
                return Err(Error::new("misplaced end inline boundary"));
            }
        }
        index += 1;
    }
    Ok(explicit)
}

fn parse_inline_text_node(xml: roxmltree::Node<'_, '_>) -> Result<Node, Error> {
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| attribute.name() != "layout-input")
    {
        if attribute.name() == "display" {
            return Err(Error::new(
                "inline text must not specify box attribute `display`",
            ));
        }
        return Err(Error::new(format!(
            "unsupported inline text attribute `{}`",
            attribute.name()
        )));
    }
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new("unsupported non-whitespace text in inline text"));
        }
        if child.is_element() && !child.has_tag_name("segment") {
            let tag = child.tag_name().name();
            if matches!(tag, "div" | "text") {
                return Err(Error::new(format!(
                    "inline text must not contain layout child `<{tag}>`"
                )));
            }
            return Err(Error::new(format!(
                "unsupported inline text child `<{tag}>`"
            )));
        }
    }
    let segments = xml
        .children()
        .filter(roxmltree::Node::is_element)
        .map(parse_inline_segment)
        .collect::<Result<Vec<_>, _>>()?;
    if segments.is_empty() {
        return Err(Error::new("inline text requires at least one `<segment>`"));
    }
    let mut ids = BTreeSet::new();
    for segment in &segments {
        if !ids.insert(segment.id) {
            return Err(Error::new(format!("duplicate segment id `{}`", segment.id)));
        }
    }
    let inline_text = MeasuredInlineText { segments };
    Ok(Node {
        prepared_occurrence: None,
        prepared_origin: None,
        prepared_control: None,
        prepared: None,
        kind: NodeKind::Text,
        style: StyleAttrs::default(),
        text: None,
        children: Vec::new(),
        inline_text: Some(inline_text),
        atomic_inline_participation: None,
        shape_bands: None,
        inline_boundary: None,
        atomic_line_strut: None,
        anonymous_grid_text_wrapper: false,
        layout_ready_inline_root: false,
    })
}

fn parse_inline_segment(xml: roxmltree::Node<'_, '_>) -> Result<MeasuredSegment, Error> {
    expect_tag(xml, "segment")?;
    const ATTRIBUTES: &[&str] = &[
        "id",
        "inline-extent",
        "inline-baseline",
        "inline-line-height",
        "bidi-level",
        "whitespace-edge",
        "following-break",
        "replacement-inline-extent",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<segment>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;
    let id = parse_inline_integer(xml, "id")?;
    let inline_extent = parse_inline_nonnegative_number(xml, "inline-extent")?;
    let baseline = parse_inline_nonnegative_number(xml, "inline-baseline")?;
    let line_height = parse_inline_nonnegative_number(xml, "inline-line-height")?;
    let metrics = layout::InlineMetrics::try_new(surgeist_layout::InlineBaselineContext::Physical, baseline, line_height).map_err(|_| {
        Error::new(format!(
            "invalid inline metrics on `<segment>`: baseline `{baseline}`, line height `{line_height}`"
        ))
    })?;
    let bidi_level = parse_bidi_level(xml, "segment")?;
    let whitespace_edge = match required_attr(xml, "whitespace-edge")? {
        "preserve" => surgeist_text::InlineWhitespaceEdge::Preserve,
        "discard-at-line-start" => surgeist_text::InlineWhitespaceEdge::DiscardAtLineStart,
        "discard-at-line-end" => surgeist_text::InlineWhitespaceEdge::DiscardAtLineEnd,
        "discard-at-both" => surgeist_text::InlineWhitespaceEdge::DiscardAtBoth,
        raw => {
            return Err(Error::new(format!(
                "invalid `whitespace-edge` on `<segment>`: `{raw}`"
            )));
        }
    };
    let following_break = parse_inline_break(xml, "segment")?;
    if matches!(following_break, MeasuredBreak::Replacement(_))
        && whitespace_edge != surgeist_text::InlineWhitespaceEdge::Preserve
    {
        return Err(Error::new(
            "invalid segment replacement: replacement requires preserved whitespace",
        ));
    }
    Ok(MeasuredSegment {
        id,
        extent: inline_extent,
        metrics,
        level: bidi_level,
        edge: whitespace_edge,
        following: following_break,
    })
}

fn parse_atomic_placeholder(
    xml: roxmltree::Node<'_, '_>,
) -> Result<(usize, layout::AtomicInlineParticipation), Error> {
    const ATTRIBUTES: &[&str] = &[
        "child-index",
        "bidi-level",
        "following-break",
        "replacement-inline-extent",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<atomic-placeholder>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;
    let child_index = parse_inline_integer(xml, "child-index")?;
    let bidi_level = parse_bidi_level(xml, "atomic-placeholder")?;
    let following_break = parse_inline_break(xml, "atomic-placeholder")?;
    let following = match following_break {
        MeasuredBreak::Prohibited => layout::InlineBreakOpportunity::Prohibited,
        MeasuredBreak::Allowed => layout::InlineBreakOpportunity::Allowed,
        MeasuredBreak::Mandatory => layout::InlineBreakOpportunity::Mandatory,
        MeasuredBreak::Replacement(_) => {
            return Err(Error::new(
                "atomic placeholder break replacement is not allowed",
            ));
        }
    };
    let participation = layout::AtomicInlineParticipation::new(bidi_level, following);
    Ok((child_index, participation))
}

fn parse_inline_break(xml: roxmltree::Node<'_, '_>, tag: &str) -> Result<MeasuredBreak, Error> {
    let raw = required_attr(xml, "following-break")?;
    let replacement = xml.attribute("replacement-inline-extent");
    match (raw, replacement) {
        ("prohibited", None) => Ok(MeasuredBreak::Prohibited),
        ("allowed", None) => Ok(MeasuredBreak::Allowed),
        ("mandatory", None) => Ok(MeasuredBreak::Mandatory),
        ("allowed-with-replacement", Some(_)) => {
            let extent = parse_inline_nonnegative_number(xml, "replacement-inline-extent")?;
            Ok(MeasuredBreak::Replacement(extent))
        }
        ("allowed-with-replacement", None) => Err(Error::new(
            "missing `replacement-inline-extent` for replacement break",
        )),
        ("prohibited" | "allowed" | "mandatory", Some(_)) => Err(Error::new(
            "replacement-inline-extent requires `allowed-with-replacement`",
        )),
        _ => Err(Error::new(format!(
            "invalid `following-break` on `<{tag}>`: `{raw}`"
        ))),
    }
}

fn parse_bidi_level(
    xml: roxmltree::Node<'_, '_>,
    tag: &str,
) -> Result<surgeist_text::BidiLevel, Error> {
    let raw = required_attr(xml, "bidi-level")?;
    let level = parse_ascii_integer::<u8>(raw)
        .and_then(|level| surgeist_text::BidiLevel::try_new(level).ok())
        .ok_or_else(|| Error::new(format!("invalid `bidi-level` on `<{tag}>`: `{raw}`")))?;
    Ok(level)
}

fn parse_inline_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    parse_ascii_integer(raw).ok_or_else(|| {
        Error::new(format!(
            "invalid `{name}` on `<{}>`: `{raw}`",
            xml.tag_name().name()
        ))
    })
}

fn parse_ascii_integer<T: std::str::FromStr>(raw: &str) -> Option<T> {
    (!raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| raw.parse().ok())
        .flatten()
}

fn parse_inline_nonnegative_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(format!(
            "invalid `{name}` on `<{}>`: `{raw}`",
            xml.tag_name().name()
        )));
    }
    Ok(value)
}

fn validate_inline_payload(xml: roxmltree::Node<'_, '_>) -> Result<(), Error> {
    let tag = xml.tag_name().name();
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(format!(
                "unsupported non-whitespace text in `<{tag}>`"
            )));
        }
        if child.is_element() {
            return Err(Error::new(format!(
                "unsupported `<{tag}>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }
    Ok(())
}

fn parse_shape_bands(xml: roxmltree::Node<'_, '_>) -> Result<Vec<FixtureShapeBand>, Error> {
    expect_tag(xml, "shape-bands")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<shape-bands>` attribute `{}`",
            attribute.name()
        )));
    }
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(
                "unsupported non-whitespace text in `<shape-bands>`",
            ));
        }
        if child.is_element() && !child.has_tag_name("shape-band") {
            return Err(Error::new(format!(
                "unsupported `<shape-bands>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }

    let mut bands = Vec::new();
    for band in xml.children().filter(roxmltree::Node::is_element) {
        let band = parse_shape_band(band)?;
        if bands.iter().any(|existing: &FixtureShapeBand| {
            existing.band_minimum == band.band_minimum && existing.band_maximum == band.band_maximum
        }) {
            return Err(Error::new(format!(
                "duplicate shape query band `{}..{}`",
                band.band_minimum, band.band_maximum
            )));
        }
        bands.push(band);
    }
    Ok(bands)
}

fn parse_shape_band(xml: roxmltree::Node<'_, '_>) -> Result<FixtureShapeBand, Error> {
    expect_tag(xml, "shape-band")?;
    const ATTRIBUTES: &[&str] = &[
        "band-minimum",
        "band-maximum",
        "interval-minimum",
        "interval-maximum",
        "origin-band-minimum",
        "origin-band-maximum",
        "provider-result",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<shape-band>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_inline_payload(xml)?;

    let band_minimum = parse_number(required_attr(xml, "band-minimum")?)?;
    let band_maximum = parse_number(required_attr(xml, "band-maximum")?)?;
    let validation_margin_box =
        layout::ScrollRect::try_new(layout::Point::ZERO, layout::Size::ZERO)
            .expect("zero validation margin box is valid");
    let query = layout::FloatExclusionQuery::try_new(
        validation_margin_box,
        layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
        band_minimum,
        band_maximum,
    )
    .map_err(|error| Error::new(format!("invalid shape band query: {error:?}")))?;

    let interval = match (
        xml.attribute("interval-minimum"),
        xml.attribute("interval-maximum"),
    ) {
        (Some(minimum), Some(maximum)) => Some((parse_number(minimum)?, parse_number(maximum)?)),
        (None, None) => None,
        _ => {
            return Err(Error::new("shape interval endpoints must appear together"));
        }
    };
    let originating_band = match (
        xml.attribute("origin-band-minimum"),
        xml.attribute("origin-band-maximum"),
    ) {
        (Some(minimum), Some(maximum)) => {
            let minimum = parse_number(minimum)?;
            let maximum = parse_number(maximum)?;
            layout::FloatExclusionQuery::try_new(
                validation_margin_box,
                query.flow_axes(),
                minimum,
                maximum,
            )
            .map_err(|error| {
                Error::new(format!("invalid originating shape band query: {error:?}"))
            })?;
            Some((minimum, maximum))
        }
        (None, None) => None,
        _ => {
            return Err(Error::new(
                "originating shape band endpoints must appear together",
            ));
        }
    };

    let response = match xml.attribute("provider-result") {
        Some("failure") => {
            if interval.is_some() || originating_band.is_some() {
                return Err(Error::new(
                    "provider failure must not include an exclusion interval",
                ));
            }
            FixtureShapeResponse::Failure
        }
        Some(value) => {
            return Err(Error::new(format!(
                "unsupported shape provider result `{value}`"
            )));
        }
        None => match interval {
            Some((minimum, maximum)) => {
                layout::FloatExclusionInterval::try_new(query, minimum, maximum).map_err(
                    |error| Error::new(format!("invalid shape exclusion interval: {error:?}")),
                )?;
                FixtureShapeResponse::Interval {
                    minimum,
                    maximum,
                    originating_band,
                }
            }
            None if originating_band.is_some() => {
                return Err(Error::new(
                    "originating shape band requires an exclusion interval",
                ));
            }
            None => FixtureShapeResponse::Empty,
        },
    };

    Ok(FixtureShapeBand {
        band_minimum: query.band_minimum(),
        band_maximum: query.band_maximum(),
        response,
    })
}

fn parse_expectation(xml: roxmltree::Node<'_, '_>) -> Result<Expectation, Error> {
    expect_tag(xml, "node")?;
    let scroll_size = match (
        xml.attribute("scroll_width"),
        xml.attribute("scroll_height"),
    ) {
        (Some(width), Some(height)) => Some(Size::new(parse_number(width)?, parse_number(height)?)),
        (None, None) => None,
        _ => {
            return Err(Error::new(
                "expected scroll_width and scroll_height to be present together",
            ));
        }
    };

    let mut fragments = None;
    let mut range_rectangles = None;
    let mut browser_control = None;
    let mut children = Vec::new();
    for child in xml.children().filter(roxmltree::Node::is_element) {
        match child.tag_name().name() {
            "node" => children.push(parse_expectation(child)?),
            "fragments" if fragments.is_none() => {
                fragments = Some(parse_fragment_expectations(child)?);
            }
            "fragments" => {
                return Err(Error::new(
                    "expected at most one `<fragments>` child on `<node>`",
                ));
            }
            "range-fragments" if range_rectangles.is_none() => {
                range_rectangles = Some(parse_native_range_fragments(child)?);
            }
            "range-fragments" => return Err(Error::new("duplicate range rectangle category")),
            "range-inks" if range_rectangles.is_none() => {
                range_rectangles = Some(parse_range_rectangle_expectations(child)?);
            }
            "range-inks" => {
                return Err(Error::new(
                    "expected at most one `<range-inks>` child on `<node>`",
                ));
            }
            "browser-control" if browser_control.is_none() => {
                browser_control = Some(parse_browser_control_expectation(child)?);
            }
            "browser-control" => {
                return Err(Error::new(
                    "expected at most one `<browser-control>` child on `<node>`",
                ));
            }
            tag => {
                return Err(Error::new(format!(
                    "unsupported expectation child `<{tag}>`"
                )));
            }
        }
    }
    if fragments.is_some() && range_rectangles.is_some() {
        return Err(Error::new(
            "model fragments and Range rectangle are distinct expectation categories",
        ));
    }
    if browser_control.is_some() && (fragments.is_some() || range_rectangles.is_some()) {
        return Err(Error::new(
            "browser control observations and fragment observations are distinct categories",
        ));
    }

    let expectation = Expectation {
        x: optional_number_attr(xml, "x")?,
        y: optional_number_attr(xml, "y")?,
        width: optional_number_attr(xml, "width")?,
        height: optional_number_attr(xml, "height")?,
        scroll_size,
        fragments,
        range_rectangles,
        browser_control,
        children,
    };
    validate_observation_expectation_category(&expectation)?;
    if (expectation.range_rectangles.is_some() || expectation.browser_control.is_some())
        && let Some(attribute) = xml.attributes().find(|attribute| {
            !["x", "y", "width", "height", "scroll_width", "scroll_height"]
                .contains(&attribute.name())
        })
    {
        return Err(Error::new(format!(
            "unsupported Range rectangle node expectation attribute `{}`",
            attribute.name()
        )));
    }
    Ok(expectation)
}

fn validate_observation_expectation_category(expectation: &Expectation) -> Result<(), Error> {
    if expectation.range_rectangles.is_none() && expectation.browser_control.is_none() {
        return Ok(());
    }
    if expectation.x.is_some()
        || expectation.y.is_some()
        || expectation.width.is_some()
        || expectation.height.is_some()
    {
        return Err(Error::new(if expectation.range_rectangles.is_some() {
            "Range rectangle must not include node geometry"
        } else {
            "browser control observations must not include model control geometry"
        }));
    }
    if expectation.scroll_size.is_some() {
        return Err(Error::new(if expectation.range_rectangles.is_some() {
            "Range rectangle must not include scroll geometry"
        } else {
            "browser control observations must not include scroll geometry"
        }));
    }
    if expectation.fragments.is_some() {
        let message = if expectation.range_rectangles.is_some() {
            "model fragments and Range rectangle are distinct expectation categories"
        } else {
            "browser control observations and fragment observations are distinct categories"
        };
        return Err(Error::new(message));
    }
    if expectation.range_rectangles.is_some() && expectation.browser_control.is_some() {
        return Err(Error::new(
            "browser control observations and fragment observations are distinct categories",
        ));
    }
    if !expectation.children.is_empty() {
        return Err(Error::new(if expectation.range_rectangles.is_some() {
            "Range rectangle must not include child geometry expectations"
        } else {
            "browser control observations must not include child geometry expectations"
        }));
    }
    Ok(())
}

fn parse_browser_control_expectation(
    xml: roxmltree::Node<'_, '_>,
) -> Result<BrowserControlExpectation, Error> {
    expect_tag(xml, "browser-control")?;
    let expectation = BrowserControlExpectation {
        source_index: parse_observation_integer(xml, "source_index", "browser-control")?,
        terminal_visual_slot: match required_attr(xml, "terminal_visual_slot")? {
            "unobserved" => None,
            _ => Some(parse_observation_integer(
                xml,
                "terminal_visual_slot",
                "browser-control",
            )?),
        },
        previous_line: parse_browser_neighbor_line(xml, "previous_line")?,
        next_line: parse_browser_neighbor_line(xml, "next_line")?,
    };
    const ATTRIBUTES: &[&str] = &[
        "source_index",
        "terminal_visual_slot",
        "previous_line",
        "next_line",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<browser-control>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, None)?;
    Ok(expectation)
}

fn parse_browser_neighbor_line(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
) -> Result<BrowserNeighborLine, Error> {
    match required_attr(xml, name)? {
        "absent" => Ok(BrowserNeighborLine::Absent),
        "unobserved" => Ok(BrowserNeighborLine::Unobserved),
        "earlier" => Ok(BrowserNeighborLine::Earlier),
        "same" => Ok(BrowserNeighborLine::Same),
        "later" => Ok(BrowserNeighborLine::Later),
        raw => Err(Error::new(format!(
            "invalid `{name}` on `<browser-control>`: `{raw}`"
        ))),
    }
}

fn parse_observation_integer<T>(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
    tag: &str,
) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<{tag}>`: `{raw}`"
        )));
    }
    raw.parse()
        .map_err(|_| Error::new(format!("invalid `{name}` on `<{tag}>`: `{raw}`")))
}

fn parse_native_range_fragments(
    xml: roxmltree::Node<'_, '_>,
) -> Result<Vec<InlineRangeRectangleExpectation>, Error> {
    if xml.attributes().len() != 0 {
        return Err(Error::new("range-fragments has unexpected attributes"));
    }
    let mut result = Vec::new();
    for node in xml.children() {
        if node.is_text() && node.text().is_some_and(|value| !value.trim().is_empty()) {
            return Err(Error::new("range-fragments has unexpected text"));
        }
        if !node.is_element() {
            continue;
        }
        if !node.has_tag_name("range-fragment") {
            return Err(Error::new("range-fragments has unexpected child"));
        }
        const KEYS: &[&str] = &["origin", "line", "physical_start_edge", "start", "advance"];
        if node.attributes().len() != KEYS.len()
            || node
                .attributes()
                .any(|attribute| !KEYS.contains(&attribute.name()))
        {
            return Err(Error::new(
                "range-fragment requires its complete closed rectangle association",
            ));
        }
        validate_fragment_payload(node, None)?;
        let integer = |name| parse_inline_integer(node, name);
        result.push(InlineRangeRectangleExpectation {
            source_segment_id: integer("origin")?,
            line_index: usize::try_from(integer("line")?)
                .map_err(|_| Error::new("range fragment line index out of range"))?,
            physical_start_edge: match required_attr(node, "physical_start_edge")? {
                "left" => PhysicalStartEdge::Left,
                "right" => PhysicalStartEdge::Right,
                "top" => PhysicalStartEdge::Top,
                "bottom" => PhysicalStartEdge::Bottom,
                _ => return Err(Error::new("invalid physical Range rectangle edge")),
            },
            start: parse_number(required_attr(node, "start")?)?,
            advance: parse_inline_nonnegative_number(node, "advance")?,
        });
    }
    Ok(result)
}

fn parse_range_rectangle_expectations(
    xml: roxmltree::Node<'_, '_>,
) -> Result<Vec<InlineRangeRectangleExpectation>, Error> {
    expect_tag(xml, "range-inks")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<range-inks>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, Some("range-ink"))?;
    xml.children()
        .filter(roxmltree::Node::is_element)
        .map(parse_range_rectangle_expectation)
        .collect()
}

fn parse_range_rectangle_expectation(
    xml: roxmltree::Node<'_, '_>,
) -> Result<InlineRangeRectangleExpectation, Error> {
    expect_tag(xml, "range-ink")?;
    let physical_start_edge = match required_attr(xml, "physical_start_edge")? {
        "left" => PhysicalStartEdge::Left,
        "right" => PhysicalStartEdge::Right,
        "top" => PhysicalStartEdge::Top,
        "bottom" => PhysicalStartEdge::Bottom,
        raw => {
            return Err(Error::new(format!(
                "invalid `physical_start_edge` on `<range-ink>`: `{raw}`"
            )));
        }
    };
    let expectation = InlineRangeRectangleExpectation {
        source_segment_id: parse_range_rectangle_integer(xml, "source_segment_id")?,
        line_index: parse_range_rectangle_integer(xml, "line_index")?,
        physical_start_edge,
        start: parse_range_rectangle_number(xml, "start", false)?,
        advance: parse_range_rectangle_number(xml, "advance", true)?,
    };
    const ATTRIBUTES: &[&str] = &[
        "source_segment_id",
        "line_index",
        "physical_start_edge",
        "start",
        "advance",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<range-ink>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, None)?;
    Ok(expectation)
}

fn parse_range_rectangle_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<range-ink>`: `{raw}`"
        )));
    }
    raw.parse()
        .map_err(|_| Error::new(format!("invalid `{name}` on `<range-ink>`: `{raw}`")))
}

fn parse_range_rectangle_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
    nonnegative: bool,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || (nonnegative && value < 0.0) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<range-ink>`: `{raw}`"
        )));
    }
    Ok(value)
}

fn parse_fragment_expectations(
    xml: roxmltree::Node<'_, '_>,
) -> Result<Vec<InlineFragmentExpectation>, Error> {
    expect_tag(xml, "fragments")?;
    if let Some(attribute) = xml.attributes().next() {
        return Err(Error::new(format!(
            "unsupported `<fragments>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, Some("fragment"))?;
    xml.children()
        .filter(roxmltree::Node::is_element)
        .map(parse_fragment_expectation)
        .collect()
}

fn parse_fragment_expectation(
    xml: roxmltree::Node<'_, '_>,
) -> Result<InlineFragmentExpectation, Error> {
    expect_tag(xml, "fragment")?;
    let expectation = InlineFragmentExpectation {
        source_segment_id: parse_fragment_integer(xml, "source_segment_id")?,
        line_index: parse_fragment_integer(xml, "line_index")?,
        historical_visual_index: Some(parse_fragment_integer(xml, "visual_index")?),
        x: parse_fragment_number(xml, "x", false)?,
        y: parse_fragment_number(xml, "y", false)?,
        width: parse_fragment_number(xml, "width", true)?,
        height: parse_fragment_number(xml, "height", true)?,
        baseline_x: parse_fragment_number(xml, "baseline_x", false)?,
        baseline_y: parse_fragment_number(xml, "baseline_y", false)?,
    };
    const ATTRIBUTES: &[&str] = &[
        "source_segment_id",
        "line_index",
        "visual_index",
        "x",
        "y",
        "width",
        "height",
        "baseline_x",
        "baseline_y",
    ];
    if let Some(attribute) = xml
        .attributes()
        .find(|attribute| !ATTRIBUTES.contains(&attribute.name()))
    {
        return Err(Error::new(format!(
            "unsupported `<fragment>` attribute `{}`",
            attribute.name()
        )));
    }
    validate_fragment_payload(xml, None)?;
    Ok(expectation)
}

fn validate_fragment_payload(
    xml: roxmltree::Node<'_, '_>,
    allowed_child: Option<&str>,
) -> Result<(), Error> {
    let tag = xml.tag_name().name();
    for child in xml.children() {
        if child.is_text() && child.text().is_some_and(|text| !text.trim().is_empty()) {
            return Err(Error::new(format!(
                "unsupported non-whitespace text in `<{tag}>`"
            )));
        }
        if child.is_element() && Some(child.tag_name().name()) != allowed_child {
            return Err(Error::new(format!(
                "unsupported `<{tag}>` child `<{}>`",
                child.tag_name().name()
            )));
        }
    }
    Ok(())
}

fn parse_fragment_integer<T>(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<T, Error>
where
    T: std::str::FromStr,
{
    let raw = required_attr(xml, name)?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<fragment>`: `{raw}`"
        )));
    }
    raw.parse()
        .map_err(|_| Error::new(format!("invalid `{name}` on `<fragment>`: `{raw}`")))
}

fn parse_fragment_number(
    xml: roxmltree::Node<'_, '_>,
    name: &str,
    nonnegative: bool,
) -> Result<Scalar, Error> {
    let raw = required_attr(xml, name)?;
    let value = parse_number(raw)?;
    if !value.is_finite() || (nonnegative && value < 0.0) {
        return Err(Error::new(format!(
            "invalid `{name}` on `<fragment>`: `{raw}`"
        )));
    }
    Ok(value)
}

fn parse_available(raw: &str) -> Result<Available, Error> {
    match raw {
        "min-content" => Ok(Available::MinContent),
        "max-content" => Ok(Available::MaxContent),
        value => {
            let value = value.strip_suffix("px").unwrap_or(value);
            Ok(Available::Definite(parse_number(value)?))
        }
    }
}

fn parse_root_context(viewport: roxmltree::Node<'_, '_>) -> Result<RootContext, Error> {
    let raw = viewport.attribute("root-context").unwrap_or("root");
    let parent_writing_mode = viewport.attribute("parent-writing-mode");
    let parent_direction = viewport.attribute("parent-direction");
    let host_inline_size = viewport.attribute("host-inline-size");

    match raw {
        "root" => {
            if parent_writing_mode.is_some()
                || parent_direction.is_some()
                || host_inline_size.is_some()
            {
                return Err(Error::new(
                    "root viewport must not specify flex-item metadata",
                ));
            }
            Ok(RootContext::Root)
        }
        "flex-item" => {
            let writing_mode =
                parse_writing_mode(Some(required_attr(viewport, "parent-writing-mode")?))?;
            let direction = parse_direction(required_attr(viewport, "parent-direction")?)?;
            let host_inline_size =
                parse_host_inline_size(required_attr(viewport, "host-inline-size")?)?;
            Ok(RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(writing_mode, direction),
                host_inline_size,
            })
        }
        _ => Err(Error::new(format!("unsupported root context `{raw}`"))),
    }
}

fn parse_host_inline_size(raw: &str) -> Result<Scalar, Error> {
    let value = raw
        .strip_suffix("px")
        .ok_or_else(|| Error::new(format!("invalid host inline size `{raw}`")))
        .and_then(parse_number)?;
    if !value.is_finite() || value < 0.0 {
        return Err(Error::new(format!("invalid host inline size `{raw}`")));
    }
    Ok(value)
}

fn parse_item_order(raw: &str) -> Result<layout::ItemOrder, Error> {
    let bytes = raw.as_bytes();
    let digits = match bytes {
        [b'0'] => &bytes[1..],
        [b'-', first, rest @ ..] if first.is_ascii_digit() && *first != b'0' => rest,
        [first, rest @ ..] if first.is_ascii_digit() && *first != b'0' => rest,
        _ => return Err(Error::new(format!("invalid item order `{raw}`"))),
    };
    if !digits.iter().all(u8::is_ascii_digit) {
        return Err(Error::new(format!("invalid item order `{raw}`")));
    }

    raw.parse::<i32>()
        .map(layout::ItemOrder::new)
        .map_err(|_| Error::new(format!("invalid item order `{raw}`")))
}

fn parse_bool(raw: &str) -> Result<bool, Error> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::new(format!("invalid boolean `{raw}`"))),
    }
}

fn parse_number(raw: &str) -> Result<Scalar, Error> {
    // Browser parity XML is a default-precision fixture boundary. The layout
    // engine has generic scalar APIs, but checked-in browser fixtures parse
    // through layout::Scalar/layout::DefaultScalar so legacy XML remains stable.
    raw.parse()
        .map_err(|_| Error::new(format!("invalid number `{raw}`")))
}

fn optional_number_attr(xml: roxmltree::Node<'_, '_>, name: &str) -> Result<Option<Scalar>, Error> {
    xml.attribute(name).map(parse_number).transpose()
}

fn required_attr<'a>(node: roxmltree::Node<'a, '_>, name: &str) -> Result<&'a str, Error> {
    node.attribute(name).ok_or_else(|| {
        Error::new(format!(
            "missing `{name}` on `<{}>`",
            node.tag_name().name()
        ))
    })
}

fn one_child<'a>(
    node: roxmltree::Node<'a, '_>,
    tag: &str,
) -> Result<roxmltree::Node<'a, 'a>, Error> {
    let mut matches = node
        .children()
        .filter(roxmltree::Node::is_element)
        .filter(|child| child.has_tag_name(tag));
    let child = matches
        .next()
        .ok_or_else(|| Error::new(format!("missing `<{tag}>` child")))?;
    if matches.next().is_some() {
        return Err(Error::new(format!("expected exactly one `<{tag}>` child")));
    }
    Ok(child)
}

fn one_element_child<'a>(node: roxmltree::Node<'a, '_>) -> Result<roxmltree::Node<'a, 'a>, Error> {
    let mut children = node.children().filter(roxmltree::Node::is_element);
    let child = children
        .next()
        .ok_or_else(|| Error::new(format!("missing child for `<{}>`", node.tag_name().name())))?;
    if children.next().is_some() {
        return Err(Error::new(format!(
            "expected exactly one child for `<{}>`",
            node.tag_name().name()
        )));
    }
    Ok(child)
}

fn expect_tag(node: roxmltree::Node<'_, '_>, tag: &str) -> Result<(), Error> {
    if node.has_tag_name(tag) {
        Ok(())
    } else {
        Err(Error::new(format!(
            "expected `<{tag}>`, found `<{}>`",
            node.tag_name().name()
        )))
    }
}

#[derive(Clone, Debug)]
struct TestNode {
    non_box_input: layout::NodeInput,
    layout_input: layout::LayoutInput,
    is_br_source: bool,
    font_family: FontFamily,
    font_size: Scalar,
    line_height: Scalar,
    text: Option<String>,
    children: Vec<usize>,
    synthetic: bool,
    parent: Option<usize>,
    range_root: Option<usize>,
    explicit_range_root: bool,
    preserve_fractional_min_content: bool,
    use_tighter_monospace_wrap: bool,
    cache: layout::Cache,
    unrounded: layout::NodeOutput,
    unrounded_present: bool,
    final_layout: layout::NodeOutput,
    final_layout_present: bool,
    unrounded_inline_fragments: Option<Vec<layout::InlineFragmentOutput>>,
    final_inline_fragments: Option<Vec<layout::InlineFragmentOutput>>,
    shape_bands: Option<Vec<FixtureShapeBand>>,
}

#[derive(Clone, Debug, Default)]
struct TestTree {
    nodes: Vec<TestNode>,
    paragraphs: Vec<layout::TextParagraphBinding<usize>>,
}

#[derive(Clone, Copy, Debug)]
struct InheritedTextContext {
    font_family: FontFamily,
    font_size: Scalar,
    line_height: LineHeightState,
    grid_lanes_text: bool,
    inline_level_text: bool,
    containing_flow: Option<layout::FlowAxes>,
    parent: Option<usize>,
    range_root: Option<usize>,
}

impl TestTree {
    fn unrounded_inline_fragments(&self, node: usize) -> Option<&[layout::InlineFragmentOutput]> {
        self.nodes[node].unrounded_inline_fragments.as_deref()
    }

    fn from_golden(root: &Node) -> Result<Self, Error> {
        let mut tree = Self::default();
        tree.push_node(
            root,
            InheritedTextContext {
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: LineHeightState::Normal,
                grid_lanes_text: false,
                inline_level_text: false,
                containing_flow: None,
                parent: None,
                range_root: None,
            },
        )?;
        Ok(tree)
    }

    fn push_node(&mut self, node: &Node, inherited: InheritedTextContext) -> Result<usize, Error> {
        let id = self.nodes.len();
        let range_root = if node.layout_ready_inline_root {
            Some(id)
        } else {
            inherited.range_root
        };
        let font_family = font_family(&node.style)?.unwrap_or(inherited.font_family);
        let font_size = font_size(&node.style)?.unwrap_or(inherited.font_size);
        let line_height = match line_height(&node.style)? {
            Some(value) => LineHeightState::Px(value),
            None => inherited.line_height,
        };
        let resolved_line_height = line_height.resolve(font_size);
        let mut layout_input = if let Some(origin) = &node.prepared_origin {
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origin.clone()))
        } else if node.prepared_control.is_some() {
            if let Some(kind) = node.inline_boundary {
                let flow = inherited.containing_flow.ok_or_else(|| {
                    Error::new("captured neutral boundary requires a containing flow")
                })?;
                // Native fixture validation forbids an authored strut here,
                // and the boundary schema admits no decoration properties.
                // These are genuine zero contributions, not absent Style facts.
                let metrics = layout::InlineMetrics::try_new(
                    surgeist_layout::InlineBaselineContext::Physical,
                    0.0,
                    0.0,
                )
                .map_err(|error| Error::new(format!("invalid neutral metrics: {error:?}")))?;
                layout::LayoutInput::inline_boundary(
                    layout::InlineBoundaryInput::new(
                        kind,
                        metrics,
                        layout::InlineBoundaryEdge::ZERO,
                    )
                    .with_direction(flow.direction())
                    .with_writing_mode(flow.writing_mode()),
                )
            } else {
                to_layout_input_in_flow(&node.style, inherited.containing_flow)?
            }
        } else {
            match (node.inline_boundary, &node.inline_text) {
                (Some(kind), None) => {
                    let flow = inherited.containing_flow.ok_or_else(|| {
                        Error::new("synthetic inline boundary requires a containing flow")
                    })?;
                    let metrics = node.atomic_line_strut.map(Ok).unwrap_or_else(|| {
                        fixture_inline_metrics(font_size, resolved_line_height)
                    })?;
                    layout::LayoutInput::inline_boundary(
                        layout::InlineBoundaryInput::new(
                            kind,
                            metrics,
                            layout::InlineBoundaryEdgeOf::ZERO,
                        )
                        .with_direction(flow.direction())
                        .with_writing_mode(flow.writing_mode()),
                    )
                }
                (None, Some(_)) => {
                    return Err(Error::new(
                        "historical measured-width text has no original source/font capture; it cannot enter production Text",
                    ));
                }
                (None, None) => to_layout_input_in_flow(&node.style, inherited.containing_flow)?,
                (Some(_), Some(_)) => {
                    return Err(Error::new(
                        "fixture node cannot be both inline text and an inline boundary",
                    ));
                }
            }
        };
        if let Some(participation) = node.atomic_inline_participation {
            let Some(input) = layout_input.as_box() else {
                return Err(Error::new("atomic placeholder must reference a box child"));
            };
            let mut input = input.clone();
            input.atomic_inline_participation = Some(participation);
            layout_input = layout::LayoutInput::box_input(input);
        }
        if node.shape_bands.is_some()
            && !layout_input.as_box().is_some_and(|input| {
                input.float_exclusion == layout::FloatExclusion::Shape
                    && input.display != layout::Display::None
                    && input.position != layout::Position::Absolute
                    && matches!(input.float, layout::Float::Left | layout::Float::Right)
            })
        {
            return Err(Error::new(
                "shape band table requires a visible in-flow left/right shape float",
            ));
        }
        let box_display = layout_input.as_box().map(|input| input.display);
        let containing_flow = layout_input
            .as_box()
            .map(|input| layout::FlowAxes::new(input.writing_mode, input.direction));
        let grid_lanes_text = inherited.grid_lanes_text
            || box_display.is_some_and(layout::Display::establishes_grid_lanes_formatting_context);
        let inline_level_text = inherited.inline_level_text
            || box_display.is_some_and(layout::Display::is_inline_level);
        self.nodes.push(TestNode {
            non_box_input: layout::NodeInput::non_box(),
            layout_input,
            is_br_source: node.style.get("source-tag") == Some("br")
                && matches!(node.style.get("display"), Some("inline" | "none")),
            font_family,
            font_size,
            line_height: resolved_line_height,
            text: node.text.clone(),
            children: Vec::new(),
            synthetic: false,
            parent: inherited.parent,
            range_root,
            explicit_range_root: node.layout_ready_inline_root,
            preserve_fractional_min_content: inherited.grid_lanes_text,
            use_tighter_monospace_wrap: !inherited.inline_level_text,
            cache: layout::Cache::new(),
            unrounded: layout::NodeOutput::new(),
            unrounded_present: false,
            final_layout: layout::NodeOutput::new(),
            final_layout_present: false,
            unrounded_inline_fragments: None,
            final_inline_fragments: None,
            shape_bands: node.shape_bands.clone(),
        });

        let mut children = node
            .children
            .iter()
            .map(|child| {
                self.push_node(
                    child,
                    InheritedTextContext {
                        font_family,
                        font_size,
                        line_height,
                        grid_lanes_text,
                        inline_level_text,
                        containing_flow,
                        parent: Some(id),
                        range_root,
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        if node.anonymous_grid_text_wrapper {
            if children.is_empty()
                || children.iter().any(|child| {
                    !matches!(
                        self.nodes[*child].layout_input,
                        layout::LayoutInput::InlineText(_)
                    )
                })
            {
                return Err(Error::new(
                    "anonymous grid text wrapper requires direct shaped-text children",
                ));
            }
            containing_flow
                .ok_or_else(|| Error::new("anonymous grid text wrapper requires a grid flow"))?;
            children = vec![self.push_synthetic_grid_text_wrapper(
                children,
                InheritedTextContext {
                    font_family,
                    font_size,
                    line_height: LineHeightState::Px(resolved_line_height),
                    grid_lanes_text,
                    inline_level_text,
                    containing_flow,
                    parent: Some(id),
                    range_root,
                },
            )];
        }
        if let Some(text) = &node.text
            && box_display.is_some_and(grid_text_container_needs_anonymous_child)
        {
            children.push(self.push_synthetic_text(
                text,
                InheritedTextContext {
                    font_family,
                    font_size,
                    line_height: LineHeightState::Px(resolved_line_height),
                    grid_lanes_text,
                    inline_level_text,
                    containing_flow,
                    parent: Some(id),
                    range_root,
                },
            )?);
            self.nodes[id].text = None;
        }
        if let Some(prepared) = &node.prepared {
            self.paragraphs.push(prepared.binding(id, &children)?);
        }
        self.nodes[id].children = children;
        Ok(id)
    }

    fn push_synthetic_grid_text_wrapper(
        &mut self,
        text_children: Vec<usize>,
        inherited: InheritedTextContext,
    ) -> usize {
        let id = self.nodes.len();
        let flow = inherited
            .containing_flow
            .expect("wrapper caller validates its containing flow");
        let line_height = inherited.line_height.resolve(inherited.font_size);
        let node_input = layout::NodeInput {
            display: layout::Display::Block,
            writing_mode: flow.writing_mode(),
            direction: flow.direction(),
            ..layout::NodeInput::default()
        };
        self.nodes.push(TestNode {
            non_box_input: layout::NodeInput::non_box(),
            layout_input: layout::LayoutInput::box_input(node_input),
            is_br_source: false,
            font_family: inherited.font_family,
            font_size: inherited.font_size,
            line_height,
            text: None,
            children: text_children,
            synthetic: true,
            parent: inherited.parent,
            range_root: inherited.range_root,
            explicit_range_root: false,
            preserve_fractional_min_content: false,
            use_tighter_monospace_wrap: false,
            cache: layout::Cache::new(),
            unrounded: layout::NodeOutput::new(),
            unrounded_present: false,
            final_layout: layout::NodeOutput::new(),
            final_layout_present: false,
            unrounded_inline_fragments: None,
            final_inline_fragments: None,
            shape_bands: None,
        });
        let wrapper = id;
        for child in self.nodes[wrapper].children.clone() {
            self.nodes[child].parent = Some(wrapper);
            self.nodes[child].range_root = inherited.range_root;
        }
        id
    }

    fn push_synthetic_text(
        &mut self,
        text: &str,
        inherited: InheritedTextContext,
    ) -> Result<usize, Error> {
        let id = self.nodes.len();
        let line_height = inherited.line_height.resolve(inherited.font_size);
        self.nodes.push(TestNode {
            non_box_input: layout::NodeInput::non_box(),
            layout_input: layout::LayoutInput::box_input(layout::NodeInput::default()),
            is_br_source: false,
            font_family: inherited.font_family,
            font_size: inherited.font_size,
            line_height,
            text: Some(text.to_string()),
            children: Vec::new(),
            synthetic: true,
            parent: inherited.parent,
            range_root: inherited.range_root,
            explicit_range_root: false,
            preserve_fractional_min_content: inherited.grid_lanes_text,
            use_tighter_monospace_wrap: !inherited.inline_level_text,
            cache: layout::Cache::new(),
            unrounded: layout::NodeOutput::new(),
            unrounded_present: false,
            final_layout: layout::NodeOutput::new(),
            final_layout_present: false,
            unrounded_inline_fragments: None,
            final_inline_fragments: None,
            shape_bands: None,
        });
        Ok(id)
    }

    fn measure(&self, node: usize, input: layout::LeafMeasureInput, text: &str) -> layout::Size {
        let known = input.known_content_size();
        let available = input
            .available_content_size()
            .map(layout::MeasurementAvailable::into_available);
        let text = TextMeasure::new(
            text,
            self.nodes[node].font_family,
            self.nodes[node].font_size,
            self.nodes[node].line_height,
            self.nodes[node].preserve_fractional_min_content,
            self.nodes[node].use_tighter_monospace_wrap,
        );
        if self.box_node_input(node).writing_mode.is_vertical() {
            let height = known.height.unwrap_or_else(|| match available.height {
                layout::Available::Definite(height) => height,
                layout::Available::MinContent => text.min_content_width(),
                layout::Available::MaxContent => text.max_content_width(),
            });
            let width = known.width.unwrap_or_else(|| text.width_for_height(height));
            return layout::Size::new(width, height);
        }

        let width = known.width.unwrap_or_else(|| match available.width {
            layout::Available::Definite(width) => text
                .max_content_width()
                .min(width)
                .max(text.min_content_width()),
            layout::Available::MinContent => text.min_content_width(),
            layout::Available::MaxContent => text.max_content_width(),
        });
        let height = known.height.unwrap_or_else(|| text.height_for_width(width));
        layout::Size::new(width, height)
    }

    fn apply_completed_batch(&mut self, batch: &layout::CompletedLayoutBatch<usize>) {
        batch.apply_to(self).unwrap_or_else(|error| match error {});
    }

    fn box_node_input(&self, node: usize) -> &layout::NodeInput {
        self.nodes[node]
            .layout_input
            .as_box()
            .unwrap_or(&self.nodes[node].non_box_input)
    }
}

fn fixture_inline_metrics(
    font_size: Scalar,
    line_height: Scalar,
) -> Result<layout::InlineMetrics, Error> {
    let leading = (line_height - font_size).max(0.0);
    let baseline = (leading / 2.0 + font_size * 0.8).min(line_height);
    layout::InlineMetrics::try_new(
        surgeist_layout::InlineBaselineContext::Physical,
        baseline,
        line_height,
    )
    .map_err(|error| Error::new(format!("invalid finite fixture inline metrics: {error:?}")))
}

fn grid_text_container_needs_anonymous_child(display: layout::Display) -> bool {
    matches!(
        display,
        layout::Display::Grid
            | layout::Display::InlineGrid
            | layout::Display::GridLanes
            | layout::Display::InlineGridLanes
    )
}

#[derive(Clone, Copy)]
struct TextMeasure<'a> {
    text: &'a str,
    font_family: FontFamily,
    font_size: Scalar,
    line_height: Scalar,
    preserve_fractional_min_content: bool,
    use_tighter_monospace_wrap: bool,
}

impl<'a> TextMeasure<'a> {
    const LINE_HEIGHT: Scalar = 10.0;

    fn new(
        text: &'a str,
        font_family: FontFamily,
        font_size: Scalar,
        line_height: Scalar,
        preserve_fractional_min_content: bool,
        use_tighter_monospace_wrap: bool,
    ) -> Self {
        Self {
            text,
            font_family,
            font_size,
            line_height,
            preserve_fractional_min_content,
            use_tighter_monospace_wrap,
        }
    }

    fn max_content_width(self) -> Scalar {
        self.char_count() as Scalar * self.advance()
    }

    fn min_content_width(self) -> Scalar {
        let width = self
            .wrap_words()
            .into_iter()
            .map(|word| word.chars)
            .max()
            .unwrap_or(0) as Scalar
            * self.advance();
        if self.preserve_fractional_min_content {
            width
        } else {
            width.floor()
        }
    }

    fn height_for_width(self, width: Scalar) -> Scalar {
        let advance = self.wrap_advance();
        let width = width.max(advance);
        let max_chars_per_line = (width / advance).floor().max(1.0) as usize;
        let mut line_count = 1;
        let mut line_chars = 0;
        for word in self.wrap_words() {
            let separator = usize::from(word.space_before && line_chars > 0);
            if line_chars == 0 {
                line_chars = word.chars;
            } else if line_chars + separator + word.chars <= max_chars_per_line {
                line_chars += separator + word.chars;
            } else {
                line_count += 1;
                line_chars = word.chars;
            }
        }
        line_count as Scalar * self.line_height()
    }

    fn width_for_height(self, height: Scalar) -> Scalar {
        let line_height = self.line_height();
        let height = height.max(line_height);
        let max_chars_per_column = (height / line_height).floor().max(1.0) as usize;
        let column_count = self.char_count().div_ceil(max_chars_per_column);
        column_count as Scalar * self.advance()
    }

    fn char_count(self) -> usize {
        self.text.chars().filter(|ch| *ch != '\u{200b}').count()
    }

    fn advance(self) -> Scalar {
        self.font_size * self.font_family.advance_factor()
    }

    fn wrap_advance(self) -> Scalar {
        if self.use_tighter_monospace_wrap {
            self.font_size * self.font_family.wrap_advance_factor()
        } else {
            self.advance()
        }
    }

    fn line_height(self) -> Scalar {
        self.line_height
    }

    fn wrap_words(self) -> Vec<WrapWord> {
        let mut words = Vec::new();
        let mut chars = 0;
        let mut space_before_next = false;
        let mut current_space_before = false;
        for ch in self.text.chars() {
            if ch == '\u{200b}' || ch.is_whitespace() {
                if chars > 0 {
                    words.push(WrapWord {
                        chars,
                        space_before: current_space_before,
                    });
                    chars = 0;
                }
                current_space_before = ch.is_whitespace() || space_before_next;
                space_before_next = current_space_before;
            } else {
                chars += 1;
            }
        }
        if chars > 0 {
            words.push(WrapWord {
                chars,
                space_before: current_space_before,
            });
        }
        words
    }
}

#[derive(Clone, Copy)]
struct WrapWord {
    chars: usize,
    space_before: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FontFamily {
    Ahem,
    Monospace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LineHeightState {
    Normal,
    Px(Scalar),
}

impl LineHeightState {
    fn resolve(self, font_size: Scalar) -> Scalar {
        match self {
            Self::Normal => font_size,
            Self::Px(value) => value,
        }
    }
}

impl FontFamily {
    fn advance_factor(self) -> Scalar {
        match self {
            Self::Ahem => 1.0,
            // Chromium's default monospace font in these fixtures is narrower
            // than Ahem's square glyphs. Keep this deliberately small and local
            // to parity XML text measurement until a real font backend exists.
            Self::Monospace => 0.602_083_3,
        }
    }

    fn wrap_advance_factor(self) -> Scalar {
        match self {
            Self::Ahem => self.advance_factor(),
            Self::Monospace => 0.593_75,
        }
    }
}

impl layout::Traverse for TestTree {
    type Node = usize;

    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        self.nodes[node].children.iter().copied()
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.nodes[node].children.len()
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.nodes[node].children[index]
    }
}

impl layout::LayoutTree for TestTree {
    type MeasureError = Error;

    fn node_input(&self, node: Self::Node) -> &layout::NodeInput {
        self.box_node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> layout::LayoutInputView<'_> {
        self.nodes[node].layout_input.as_view()
    }

    fn cache_get(
        &self,
        node: Self::Node,
        input: &ComputeInput,
        context: layout::CacheKeyContext,
    ) -> Option<ComputeOutput> {
        self.nodes[node].cache.get_with_context(input, context)
    }

    fn unrounded_layout(&self, node: Self::Node) -> Option<layout::NodeOutput> {
        self.nodes[node]
            .unrounded_present
            .then_some(self.nodes[node].unrounded)
    }

    fn float_exclusion_interval(
        &self,
        node: Self::Node,
        query: layout::FloatExclusionQuery,
    ) -> Option<Result<Option<layout::FloatExclusionInterval>, Self::MeasureError>> {
        let table = self.nodes[node].shape_bands.as_ref()?;
        let Some(band) = table.iter().find(|band| {
            band.band_minimum == query.band_minimum() && band.band_maximum == query.band_maximum()
        }) else {
            return Some(Err(Error::new(format!(
                "missing fixture shape response for query band `{}..{}`",
                query.band_minimum(),
                query.band_maximum()
            ))));
        };

        match band.response {
            FixtureShapeResponse::Empty => Some(Ok(None)),
            FixtureShapeResponse::Failure => Some(Err(Error::new(format!(
                "fixture shape provider failure for query band `{}..{}`",
                query.band_minimum(),
                query.band_maximum()
            )))),
            FixtureShapeResponse::Interval {
                minimum,
                maximum,
                originating_band,
            } => {
                let originating_query = match originating_band {
                    Some((band_minimum, band_maximum)) => {
                        match layout::FloatExclusionQuery::try_new(
                            query.margin_box(),
                            query.flow_axes(),
                            band_minimum,
                            band_maximum,
                        ) {
                            Ok(query) => query,
                            Err(error) => {
                                return Some(Err(Error::new(format!(
                                    "invalid fixture originating shape query: {error:?}"
                                ))));
                            }
                        }
                    }
                    None => query,
                };
                Some(
                    layout::FloatExclusionInterval::try_new(originating_query, minimum, maximum)
                        .map_err(|error| {
                            Error::new(format!("invalid fixture shape response: {error:?}"))
                        }),
                )
            }
        }
    }

    fn has_leaf_measurement(&self, node: Self::Node) -> bool {
        let node = &self.nodes[node];
        // Only fixture text supplies intrinsic content measurements. Empty
        // ordinary boxes retain their native container formatting algorithm.
        node.layout_input.as_box().is_some() && node.children.is_empty() && node.text.is_some()
    }

    fn measure_leaf(
        &self,
        node: Self::Node,
        input: layout::LeafMeasureInput,
    ) -> Option<Result<layout::Size, Self::MeasureError>> {
        if !self.has_leaf_measurement(node) {
            return None;
        }
        let text = self.nodes[node].text.as_deref()?;
        Some(Ok(self.measure(node, input, text)))
    }
}

impl layout::LayoutBatchSink<usize, Scalar> for TestTree {
    type Error = std::convert::Infallible;
    type Prepared = Vec<TestNode>;

    fn prepare_layout_batch(
        &self,
        batch: &layout::CompletedLayoutBatch<usize>,
    ) -> Result<Self::Prepared, Self::Error> {
        let mut prepared = self.nodes.clone();
        for node in batch.invalidated_nodes() {
            let state = &mut prepared[*node];
            state.cache.clear();
            state.unrounded_present = false;
            state.final_layout_present = false;
            state.unrounded_inline_fragments = None;
            state.final_inline_fragments = None;
        }
        for entry in batch.cache_clear_entries() {
            prepared[entry.node()].cache.clear();
        }
        for entry in batch.unrounded_entries() {
            let state = &mut prepared[entry.node()];
            state.unrounded = entry.output();
            state.unrounded_present = true;
            if matches!(state.layout_input, layout::LayoutInput::InlineText(_)) {
                state.unrounded_inline_fragments = Some(Vec::new());
            }
        }
        for entry in batch.final_entries() {
            let state = &mut prepared[entry.node()];
            state.final_layout = entry.output();
            state.final_layout_present = true;
            if matches!(state.layout_input, layout::LayoutInput::InlineText(_)) {
                state.final_inline_fragments = Some(Vec::new());
            }
        }
        for entry in batch.unrounded_inline_fragments() {
            let layout::InlineFragmentOwner::Node(node) = entry.owner() else {
                panic!("fixture contains node fragments")
            };
            prepared[*node]
                .unrounded_inline_fragments
                .get_or_insert_with(Vec::new)
                .push(entry.fragment().clone());
        }
        for entry in batch.final_inline_fragments() {
            let layout::InlineFragmentOwner::Node(node) = entry.owner() else {
                panic!("fixture contains node fragments")
            };
            prepared[*node]
                .final_inline_fragments
                .get_or_insert_with(Vec::new)
                .push(entry.fragment().clone());
        }
        for entry in batch.cache_store_entries() {
            prepared[entry.node()].cache.store_with_context(
                entry.input(),
                entry.context(),
                entry.output(),
            );
        }
        Ok(prepared)
    }

    fn commit_layout_batch(&mut self, prepared: Self::Prepared) {
        self.nodes = prepared;
    }
}

fn compare_expectation(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
    unrounded_inline_fragments: &[layout::InlineFragmentOutputEntry<usize>],
    final_inline_fragments: &[layout::InlineFragmentOutputEntry<usize>],
) -> Result<(), Error> {
    compare_expectation_with_endpoint_accounting(
        tree,
        node,
        expected,
        path,
        use_rounding,
        unrounded_inline_fragments,
        final_inline_fragments,
    )
    .map(|_| ())
}

fn compare_expectation_with_endpoint_accounting(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
    unrounded_inline_fragments: &[layout::InlineFragmentOutputEntry<usize>],
    final_inline_fragments: &[layout::InlineFragmentOutputEntry<usize>],
) -> Result<ComparisonResult, Error> {
    let mut comparison_state = ComparisonState {
        fragment_cursors: FragmentCursors {
            unrounded: FragmentCursor {
                entries: unrounded_inline_fragments,
                next: 0,
            },
            final_output: FragmentCursor {
                entries: final_inline_fragments,
                next: 0,
            },
        },
        strict_geometry_nodes: BTreeSet::new(),
        pending_browser_controls: Vec::new(),
    };
    compare_expectation_in_source_order(
        tree,
        node,
        expected,
        path,
        use_rounding,
        &mut comparison_state,
        None,
    )?;
    if let Some(entry) = comparison_state
        .fragment_cursors
        .unrounded
        .entries
        .get(comparison_state.fragment_cursors.unrounded.next)
    {
        return Err(Error::new(format!(
            "{path}: unexpected unrounded fragment source association at owner {:?}",
            entry.owner()
        )));
    }
    if let Some(entry) = comparison_state
        .fragment_cursors
        .final_output
        .entries
        .get(comparison_state.fragment_cursors.final_output.next)
    {
        return Err(Error::new(format!(
            "{path}: unexpected fragment source association at owner {:?}",
            entry.owner()
        )));
    }
    let mut result = ComparisonResult::default();
    for pending in std::mem::take(&mut comparison_state.pending_browser_controls) {
        if let Some(field) = compare_browser_control_expectation(
            tree,
            pending.node,
            &pending.path,
            pending.expected,
            pending.context.as_ref(),
            &comparison_state.strict_geometry_nodes,
        )? {
            result.record_endpoint_unobservable(field);
        }
    }
    Ok(result)
}

struct FragmentCursor<'a> {
    entries: &'a [layout::InlineFragmentOutputEntry<usize>],
    next: usize,
}

struct FragmentCursors<'a> {
    unrounded: FragmentCursor<'a>,
    final_output: FragmentCursor<'a>,
}

struct ComparisonState<'a> {
    fragment_cursors: FragmentCursors<'a>,
    strict_geometry_nodes: BTreeSet<usize>,
    pending_browser_controls: Vec<PendingBrowserControl>,
}

impl<'a> FragmentCursor<'a> {
    fn take_for_node(&mut self, node: usize) -> &'a [layout::InlineFragmentOutputEntry<usize>] {
        let start = self.next;
        while self
            .entries
            .get(self.next)
            .is_some_and(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(node))
        {
            self.next += 1;
        }
        &self.entries[start..self.next]
    }
}

#[derive(Clone, Copy)]
struct BrowserControlContext<'a> {
    parent: usize,
    source_index: usize,
    siblings: &'a [usize],
}

#[derive(Clone)]
struct OwnedBrowserControlContext {
    parent: usize,
    source_index: usize,
    siblings: Vec<usize>,
}

impl From<BrowserControlContext<'_>> for OwnedBrowserControlContext {
    fn from(context: BrowserControlContext<'_>) -> Self {
        Self {
            parent: context.parent,
            source_index: context.source_index,
            siblings: context.siblings.to_vec(),
        }
    }
}

struct PendingBrowserControl {
    node: usize,
    path: String,
    expected: BrowserControlExpectation,
    context: Option<OwnedBrowserControlContext>,
}

fn compare_expectation_in_source_order(
    tree: &TestTree,
    node: usize,
    expected: &Expectation,
    path: &str,
    use_rounding: bool,
    comparison_state: &mut ComparisonState<'_>,
    browser_control_context: Option<BrowserControlContext<'_>>,
) -> Result<(), Error> {
    validate_observation_expectation_category(expected)
        .map_err(|error| Error::new(format!("{path}: {error}")))?;
    let selected_output_is_present = if use_rounding {
        tree.nodes[node].final_layout_present
    } else {
        tree.nodes[node].unrounded_present
    };
    let browser_control_in_flex = expected.browser_control.is_some()
        && browser_control_context.is_some_and(|context| {
            tree.box_node_input(context.parent).display == layout::Display::Flex
        });
    if !selected_output_is_present
        && !browser_control_in_flex
        && matches!(
            tree.nodes[node].layout_input,
            layout::LayoutInput::LineBreak(_) | layout::LayoutInput::InlineBoundary(_)
        )
    {
        let phase = if use_rounding { "final" } else { "unrounded" };
        return Err(Error::new(format!(
            "{path}: control geometry mismatch, expected {phase} output"
        )));
    }

    let actual = if use_rounding {
        tree.nodes[node].final_layout
    } else {
        tree.nodes[node].unrounded
    };
    compare_optional_number(path, "x", actual.location.x, expected.x)?;
    compare_optional_number(path, "y", actual.location.y, expected.y)?;
    compare_optional_number(path, "width", actual.size.width, expected.width)?;
    compare_optional_number(path, "height", actual.size.height, expected.height)?;
    if expected.x.is_some()
        && expected.y.is_some()
        && expected.width.is_some()
        && expected.height.is_some()
    {
        comparison_state.strict_geometry_nodes.insert(node);
    }

    if let Some(expected_scroll_size) = expected.scroll_size {
        let scroll_geometry = actual.scroll_geometry.ok_or_else(|| {
            Error::new(format!(
                "{path}: scroll geometry mismatch, expected canonical geometry"
            ))
        })?;
        let range = scroll_geometry.physical_range();
        let x_span = range.x().maximum() - range.x().minimum();
        let y_span = range.y().maximum() - range.y().minimum();
        compare_number(path, "scroll width", x_span, expected_scroll_size.width)?;
        compare_number(path, "scroll height", y_span, expected_scroll_size.height)?;
    }

    let unrounded_fragments = comparison_state
        .fragment_cursors
        .unrounded
        .take_for_node(node);
    let final_fragments = comparison_state
        .fragment_cursors
        .final_output
        .take_for_node(node);
    if let Some(expected_fragments) = &expected.fragments {
        compare_fragment_expectations(path, final_fragments, expected_fragments)?;
    }
    if let Some(expected_range_rectangles) = &expected.range_rectangles {
        compare_range_rectangle_expectations(
            tree,
            node,
            path,
            unrounded_fragments,
            expected_range_rectangles,
        )?;
    }
    if let Some(expected_browser_control) = expected.browser_control {
        comparison_state
            .pending_browser_controls
            .push(PendingBrowserControl {
                node,
                path: path.to_string(),
                expected: expected_browser_control,
                context: browser_control_context.map(OwnedBrowserControlContext::from),
            });
    }

    let children = comparison_children(tree, node);
    let expected_children = comparison_expected_children(tree, node, &expected.children);
    if children.len() != expected_children.len() {
        return Err(Error::new(format!(
            "{path}: expected {} children, got {}",
            expected_children.len(),
            children.len()
        )));
    }

    let mut index = 0;
    for child in comparison_source_children(tree, node) {
        if unobserved_inline_boundary(&tree.nodes[child].layout_input) {
            consume_unobserved_boundary(tree, child, path, comparison_state)?;
            continue;
        }
        compare_expectation_in_source_order(
            tree,
            child,
            expected_children[index],
            &format!("{path}/{index}"),
            use_rounding,
            comparison_state,
            Some(BrowserControlContext {
                parent: node,
                source_index: index,
                siblings: &children,
            }),
        )?;
        index += 1;
    }

    Ok(())
}

fn unobserved_inline_boundary(input: &layout::LayoutInput) -> bool {
    match input {
        layout::LayoutInput::InlineBoundary(_) => true,
        layout::LayoutInput::TextControl(control) => matches!(
            control.occurrence().view().kind(),
            surgeist_text::ParagraphControlKind::InlineBoundaryStart
                | surgeist_text::ParagraphControlKind::InlineBoundaryEnd
        ),
        _ => false,
    }
}

fn consume_unobserved_boundary(
    tree: &TestTree,
    node: usize,
    path: &str,
    comparison: &mut ComparisonState<'_>,
) -> Result<(), Error> {
    let unrounded = comparison.fragment_cursors.unrounded.take_for_node(node);
    let final_output = comparison.fragment_cursors.final_output.take_for_node(node);
    if let Some(control) = tree
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find(|row| row.node == node)
    {
        let actual_control = |entry: &layout::InlineFragmentOutputEntry<usize>| {
            matches!(entry.fragment().finalized_fragment().view().owner(),
                surgeist_text::FinalizedFragmentOwner::Control(owner)
                    if owner == control.occurrence)
        };
        if unrounded.is_empty()
            || !unrounded.iter().all(actual_control)
            || !final_output.iter().all(actual_control)
            || unrounded.iter().any(|entry| {
                !final_output.iter().any(|output| {
                    output.fragment().finalized_fragment() == entry.fragment().finalized_fragment()
                })
            })
            || final_output.iter().any(|entry| {
                !unrounded.iter().any(|output| {
                    output.fragment().finalized_fragment() == entry.fragment().finalized_fragment()
                })
            })
        {
            return Err(Error::new(format!(
                "{path}: unobserved boundary lost its actual finalized control custody"
            )));
        }
    } else if !unrounded.is_empty() || !final_output.is_empty() {
        return Err(Error::new(format!(
            "{path}: legacy boundary cannot own native Text fragments"
        )));
    }
    Ok(())
}

fn comparison_source_children(tree: &TestTree, node: usize) -> Vec<usize> {
    let mut children = Vec::new();
    for child in tree.nodes[node].children.iter().copied() {
        if tree.nodes[child].synthetic {
            children.extend(comparison_source_children(tree, child));
        } else {
            children.push(child);
        }
    }
    children
}

fn comparison_children(tree: &TestTree, node: usize) -> Vec<usize> {
    comparison_source_children(tree, node)
        .into_iter()
        .filter(|child| !unobserved_inline_boundary(&tree.nodes[*child].layout_input))
        .collect()
}

fn comparison_expected_children<'a>(
    tree: &TestTree,
    node: usize,
    expected: &'a [Expectation],
) -> Vec<&'a Expectation> {
    let source_children = &tree.nodes[node].children;
    if source_children.len() != expected.len() {
        return expected.iter().collect();
    }
    source_children
        .iter()
        .zip(expected)
        .filter_map(|(&child, expectation)| {
            (!unobserved_inline_boundary(&tree.nodes[child].layout_input)).then_some(expectation)
        })
        .collect()
}

/// Fixture identity is the actual origin's ordinal in its retained preparation;
/// observed rectangles and numeric IDs cannot mint that association.
fn fragment_source_id(fragment: &layout::InlineFragmentOutput) -> Result<u64, Error> {
    let issued = fragment.finalized_fragment();
    let origin = issued
        .view()
        .origin()
        .ok_or_else(|| Error::new("Range rectangle requires an actual source-owned fragment"))?;
    let prepared = issued.finalized_line().prepared_text();
    let index = prepared
        .origin_views()
        .position(|view| view.handle() == origin)
        .ok_or_else(|| Error::new("fragment origin does not belong to its retained preparation"))?;
    u64::try_from(index).map_err(|_| Error::new("origin index exceeds fixture integer range"))
}

fn compare_fragment_expectations(
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
    expected: &[InlineFragmentExpectation],
) -> Result<(), Error> {
    if expected
        .iter()
        .any(|fragment| fragment.historical_visual_index.is_some())
    {
        return Err(Error::new(format!(
            "{path}: historical fragment visual_index has no native ordering equivalent"
        )));
    }
    if actual.len() != expected.len() {
        return Err(Error::new(format!(
            "{path}: fragment count mismatch, expected {}, got {}",
            expected.len(),
            actual.len()
        )));
    }

    for (index, (entry, expected)) in actual.iter().zip(expected).enumerate() {
        let fragment = entry.fragment();
        compare_fragment_identity(
            path,
            index,
            "source segment id",
            fragment_source_id(fragment)?,
            expected.source_segment_id,
        )?;
        compare_fragment_identity(
            path,
            index,
            "line index",
            fragment.line_index(),
            expected.line_index,
        )?;

        let rect = fragment
            .rect()
            .ok_or_else(|| Error::new("source fragment has no range rectangle"))?;
        compare_number(
            path,
            &format!("fragment[{index}] rect x"),
            rect.origin().x,
            expected.x,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect y"),
            rect.origin().y,
            expected.y,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect width"),
            rect.size().width,
            expected.width,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] rect height"),
            rect.size().height,
            expected.height,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] baseline x"),
            fragment.baseline().x,
            expected.baseline_x,
        )?;
        compare_number(
            path,
            &format!("fragment[{index}] baseline y"),
            fragment.baseline().y,
            expected.baseline_y,
        )?;
    }

    Ok(())
}

fn compare_range_rectangle_expectations(
    tree: &TestTree,
    owner: usize,
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
    expected: &[InlineRangeRectangleExpectation],
) -> Result<(), Error> {
    let root = range_rectangle_root(tree, owner)?;
    let binding = tree
        .paragraphs
        .iter()
        .find(|binding| binding.container == root)
        .ok_or_else(|| Error::new("Range inline root has no authenticated captured paragraph"))?;
    if actual.iter().any(|entry| {
        entry
            .fragment()
            .finalized_fragment()
            .finalized_line()
            .prepared_text()
            != binding.prepared
    }) {
        return Err(Error::new(
            "Range fragment belongs to another captured paragraph",
        ));
    }
    compare_range_rectangle_expectations_with_translation(path, actual, expected, |edge| {
        range_rectangle_ancestor_translation(tree, owner, edge)
    })
}

/// One browser Range rectangle assembled from genuine source intersections.
/// These are allocation/range rectangles, not glyph ink bounds.
struct NativeRangeRectangle {
    origin: surgeist_text::PreparedOrigin,
    line: surgeist_text::FinalizedLine,
    source: surgeist_text::Range,
    source_segment_id: u64,
    line_index: usize,
    rect: layout::ScrollRect,
}

struct NativeRangeLender<'a> {
    origin: surgeist_text::PreparedOrigin,
    line: surgeist_text::FinalizedLine,
    entries: Vec<&'a layout::InlineFragmentOutputEntry<usize>>,
}

/// The pinned Ahem profile retains a glyph row for a preserved U+0020 even
/// when logical spacing makes its allocation zero. An executed edge discard
/// instead retains its source fragment but emits no glyph. Read that result;
/// do not repeat Text's whitespace winner or selected-edge algorithm here.
fn unpainted_source_space(
    fragment: &surgeist_text::FinalizedFragment,
    line: &surgeist_text::FinalizedLine,
) -> bool {
    let view = fragment.view();
    let (Some(origin), Some(processed), Some(source)) = (
        view.origin(),
        view.processed_range(),
        view.aggregate_range(),
    ) else {
        return false;
    };
    let analyzed = origin.prepared_text().analyzed_source();
    let units = analyzed.processed_source().processed_text().code_units();
    units.get(processed.start().get()..processed.end().get()) == Some(&[0x20][..])
        && !line.run_views().any(|run| {
            run.glyphs().any(|glyph| {
                let glyph_source = glyph.range();
                glyph_source.start < source.end && source.start < glyph_source.end
            })
        })
}

fn native_range_rectangles(
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
) -> Result<Vec<NativeRangeRectangle>, Error> {
    let mut groups = Vec::<NativeRangeLender<'_>>::new();
    let mut seen = Vec::new();
    for entry in actual {
        let issued = entry.fragment().finalized_fragment();
        if seen.contains(&issued) {
            return Err(Error::new(format!(
                "{path}: duplicate actual Range source fragment"
            )));
        }
        seen.push(issued);
        let surgeist_text::FinalizedFragmentOwner::Origin(origin) = issued.view().owner() else {
            return Err(Error::new(
                "Range rectangle requires an actual source-owned fragment",
            ));
        };
        let line = issued.finalized_line();
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.origin == origin && group.line == line)
        {
            group.entries.push(entry);
        } else {
            groups.push(NativeRangeLender {
                origin,
                line,
                entries: vec![entry],
            });
        }
    }
    let mut rectangles: Vec<NativeRangeRectangle> = Vec::new();
    for NativeRangeLender {
        origin,
        line,
        mut entries,
    } in groups
    {
        let canonical = line.fragments().filter(|fragment|
            matches!(fragment.view().owner(), surgeist_text::FinalizedFragmentOwner::Origin(owner)
                if owner == origin)).collect::<Vec<_>>();
        if canonical.iter().any(|issued| {
            !entries
                .iter()
                .any(|entry| entry.fragment().finalized_fragment() == issued)
        }) {
            return Err(Error::new(format!(
                "{path}: incomplete actual Range source coverage"
            )));
        }
        entries.sort_by_key(|entry| {
            let source = entry
                .fragment()
                .finalized_fragment()
                .view()
                .aggregate_range()
                .unwrap();
            (source.start, source.end)
        });
        let mut invisible = None::<surgeist_text::Range>;
        for entry in entries {
            let fragment = entry.fragment();
            let mut source = fragment
                .finalized_fragment()
                .view()
                .aggregate_range()
                .ok_or_else(|| Error::new("Range source fragment has no original interval"))?;
            let rect = fragment.rect().filter(|rect| {
                rect.size().width != 0.0
                    || !unpainted_source_space(fragment.finalized_fragment(), &line)
            });
            let Some(rect) = rect else {
                // Genuine non-geometric source intervals retain custody and may
                // bridge adjacent rectangles; absent records never do. The same
                // holds for a source space with actual executed glyph suppression.
                if let Some(last) = rectangles.last_mut()
                    && last.origin == origin
                    && last.line == line
                    && last.source.end == source.start
                {
                    last.source.end = source.end;
                } else if let Some(pending) = &mut invisible
                    && pending.end == source.start
                {
                    pending.end = source.end;
                } else {
                    invisible = Some(source);
                }
                continue;
            };
            if let Some(pending) = invisible.take()
                && pending.end == source.start
            {
                source.start = pending.start;
            }
            let next = NativeRangeRectangle {
                origin: origin.clone(),
                line: line.clone(),
                source,
                source_segment_id: fragment_source_id(fragment)?,
                line_index: fragment.line_index(),
                rect,
            };
            if let Some(last) = rectangles.last_mut()
                && merge_native_range_rectangles(last, &next)?
            {
                continue;
            }
            rectangles.push(next);
        }
    }
    Ok(rectangles)
}

fn merge_native_range_rectangles(
    previous: &mut NativeRangeRectangle,
    next: &NativeRangeRectangle,
) -> Result<bool, Error> {
    if previous.origin != next.origin
        || previous.line != next.line
        || previous.line_index != next.line_index
        || previous.source.end != next.source.start
    {
        return Ok(false);
    }
    let a = previous.rect;
    let b = next.rect;
    // Preserve zero-width directional-control rectangles. In this closed
    // horizontal profile only positive, exactly adjacent rectangles in the
    // same physical band coalesce; overlapping/gapped bands never become a hull.
    if a.size().width <= 0.0
        || b.size().width <= 0.0
        || a.origin().y != b.origin().y
        || a.size().height != b.size().height
        || !(a.origin().x + a.size().width == b.origin().x
            || b.origin().x + b.size().width == a.origin().x)
    {
        return Ok(false);
    }
    previous.rect = layout::ScrollRect::try_new(
        layout::Point::new(a.origin().x.min(b.origin().x), a.origin().y),
        layout::Size::new(a.size().width + b.size().width, a.size().height),
    )
    .map_err(|_| Error::new("coalesced Range rectangle is not finite"))?;
    previous.source.end = next.source.end;
    Ok(true)
}

fn range_rectangle_values(rect: layout::ScrollRect, edge: PhysicalStartEdge) -> (Scalar, Scalar) {
    match edge {
        PhysicalStartEdge::Left => (rect.origin().x, rect.size().width),
        PhysicalStartEdge::Right => (rect.origin().x + rect.size().width, rect.size().width),
        PhysicalStartEdge::Top => (rect.origin().y, rect.size().height),
        PhysicalStartEdge::Bottom => (rect.origin().y + rect.size().height, rect.size().height),
    }
}

fn compare_range_rectangle_expectations_with_translation(
    path: &str,
    actual: &[layout::InlineFragmentOutputEntry<usize>],
    expected: &[InlineRangeRectangleExpectation],
    mut translation: impl FnMut(PhysicalStartEdge) -> Result<Scalar, Error>,
) -> Result<(), Error> {
    let actual = native_range_rectangles(path, actual)?;
    if actual.len() != expected.len() {
        return Err(Error::new(format!(
            "{path}: Range rectangle count mismatch, expected {}, got {}",
            expected.len(),
            actual.len()
        )));
    }
    let mut matched = vec![false; actual.len()];
    for (index, expected) in expected.iter().enumerate() {
        let offset = translation(expected.physical_start_edge)?;
        let identity = |row: &NativeRangeRectangle| {
            row.source_segment_id == expected.source_segment_id
                && row.line_index == expected.line_index
        };
        let actual_index = actual
            .iter()
            .enumerate()
            .find(|(i, row)| {
                let (start, advance) =
                    range_rectangle_values(row.rect, expected.physical_start_edge);
                !matched[*i]
                    && identity(row)
                    && ComparisonTolerance::browser_parity()
                        .contains(start + offset - expected.start)
                    && ComparisonTolerance::browser_parity().contains(advance - expected.advance)
            })
            .or_else(|| {
                actual
                    .iter()
                    .enumerate()
                    .find(|(i, row)| !matched[*i] && identity(row))
            })
            .map(|(i, _)| i)
            .or_else(|| matched.iter().position(|matched| !matched))
            .expect("equal Range observation counts leave an unmatched source rectangle");
        matched[actual_index] = true;
        let row = &actual[actual_index];
        compare_range_rectangle_identity(
            path,
            index,
            "source segment id",
            row.source_segment_id,
            expected.source_segment_id,
        )?;
        compare_range_rectangle_identity(
            path,
            index,
            "line index",
            row.line_index,
            expected.line_index,
        )?;
        let (start, advance) = range_rectangle_values(row.rect, expected.physical_start_edge);
        compare_number(
            path,
            &format!("Range rectangle[{index}] physical flow-inline start"),
            start + offset,
            expected.start,
        )?;
        compare_number(
            path,
            &format!("Range rectangle[{index}] advance"),
            advance,
            expected.advance,
        )?;
    }
    Ok(())
}

fn range_rectangle_root(tree: &TestTree, owner: usize) -> Result<usize, Error> {
    let owner_node = tree
        .nodes
        .get(owner)
        .ok_or_else(|| Error::new(format!("Range owner node `{owner}` is missing")))?;
    let Some(root) = owner_node.range_root else {
        return Err(Error::new(
            "Range observation requires an explicit inline root marker",
        ));
    };
    let root_node = tree
        .nodes
        .get(root)
        .ok_or_else(|| Error::new(format!("Range inline root node `{root}` is missing")))?;
    if !root_node.explicit_range_root || root_node.range_root != Some(root) {
        return Err(Error::new(format!(
            "Range inline root identity `{root}` does not name its explicit marker"
        )));
    }
    Ok(root)
}

fn range_rectangle_ancestor_translation(
    tree: &TestTree,
    owner: usize,
    edge: PhysicalStartEdge,
) -> Result<Scalar, Error> {
    let owner_node = tree
        .nodes
        .get(owner)
        .ok_or_else(|| Error::new(format!("Range owner node `{owner}` is missing")))?;
    let root = range_rectangle_root(tree, owner)?;

    let mut current = owner_node.parent.ok_or_else(|| {
        Error::new(format!(
            "Range owner node `{owner}` has no parent before explicit root `{root}`"
        ))
    })?;
    let mut visited = std::collections::BTreeSet::new();
    let mut translation = 0.0;
    while current != root {
        if !visited.insert(current) {
            return Err(Error::new(format!(
                "Range ancestry cycle before explicit root `{root}` at node `{current}`"
            )));
        }
        let ancestor = tree.nodes.get(current).ok_or_else(|| {
            Error::new(format!(
                "Range ancestor node `{current}` is missing before explicit root `{root}`"
            ))
        })?;
        if ancestor.range_root != Some(root) || ancestor.explicit_range_root {
            return Err(Error::new(format!(
                "Range ancestor node `{current}` has the wrong explicit root identity"
            )));
        }
        if !ancestor.unrounded_present {
            return Err(Error::new(format!(
                "Range ancestor node `{current}` has no unrounded location"
            )));
        }
        translation += match edge {
            PhysicalStartEdge::Left | PhysicalStartEdge::Right => ancestor.unrounded.location.x,
            PhysicalStartEdge::Top | PhysicalStartEdge::Bottom => ancestor.unrounded.location.y,
        };
        current = ancestor.parent.ok_or_else(|| {
            Error::new(format!(
                "Range ancestry ended before explicit root `{root}` at node `{current}`"
            ))
        })?;
    }
    Ok(translation)
}

fn compare_browser_control_expectation(
    tree: &TestTree,
    node: usize,
    path: &str,
    expected: BrowserControlExpectation,
    context: Option<&OwnedBrowserControlContext>,
    strict_geometry_nodes: &BTreeSet<usize>,
) -> Result<Option<EndpointUnobservableField>, Error> {
    let context = context.ok_or_else(|| {
        Error::new(format!(
            "{path}: browser control source mismatch, expected a source child"
        ))
    })?;
    compare_browser_control_identity(
        path,
        "source index",
        context.source_index,
        expected.source_index,
    )?;
    let parent_input = tree.box_node_input(context.parent);
    if parent_input.display == layout::Display::Flex {
        if parent_input.flex_wrap != layout::FlexWrap::NoWrap {
            return Err(Error::new(format!(
                "{path}: wrapped flex browser control observations are unsupported"
            )));
        }
        if !tree.nodes[node].is_br_source {
            return Err(Error::new(format!(
                "{path}: browser control source mismatch, expected a BR source"
            )));
        }

        if let Some(expected_slot) = expected.terminal_visual_slot {
            compare_browser_control_identity(
                path,
                "terminal visual slot",
                context.source_index,
                expected_slot,
            )?;
        }
        if expected.previous_line != BrowserNeighborLine::Unobserved {
            let actual_previous = if context.source_index == 0 {
                BrowserNeighborLine::Absent
            } else {
                BrowserNeighborLine::Same
            };
            compare_browser_control_identity(
                path,
                "previous neighbor line",
                actual_previous,
                expected.previous_line,
            )?;
        }
        if expected.next_line != BrowserNeighborLine::Unobserved {
            let actual_next = if context.source_index + 1 == context.siblings.len() {
                BrowserNeighborLine::Absent
            } else {
                BrowserNeighborLine::Same
            };
            compare_browser_control_identity(
                path,
                "next neighbor line",
                actual_next,
                expected.next_line,
            )?;
        }
        return Ok(None);
    }

    let line_break = match tree.nodes[node].layout_input {
        layout::LayoutInput::LineBreak(line_break) => line_break,
        _ => {
            return Err(Error::new(format!(
                "{path}: browser control source mismatch, expected a line break"
            )));
        }
    };
    if !tree.nodes[node].unrounded_present {
        return Err(Error::new(format!(
            "{path}: browser control observation mismatch, expected unrounded output"
        )));
    }

    let flow = layout::FlowAxes::new(line_break.writing_mode(), line_break.direction());
    let control_interval = output_block_interval(tree.nodes[node].unrounded, flow);
    let actual_slot = context.siblings[..context.source_index]
        .iter()
        .rev()
        .take_while(|sibling| {
            !matches!(
                tree.nodes[**sibling].layout_input,
                layout::LayoutInput::LineBreak(_)
            )
        })
        .filter(|sibling| {
            tree.nodes[**sibling].unrounded_present
                && block_relation(
                    control_interval,
                    output_block_interval(tree.nodes[**sibling].unrounded, flow),
                    flow,
                ) == BrowserNeighborLine::Same
        })
        .count();
    if let Some(expected_slot) = expected.terminal_visual_slot {
        compare_browser_control_identity(path, "terminal visual slot", actual_slot, expected_slot)?;
    }

    let previous = context
        .source_index
        .checked_sub(1)
        .map(|index| context.siblings[index]);
    let next = context.siblings.get(context.source_index + 1).copied();
    if expected.previous_line != BrowserNeighborLine::Unobserved {
        let actual_previous =
            browser_neighbor_relation(tree, previous, control_interval, flow, path)?;
        compare_browser_control_identity(
            path,
            "previous neighbor line",
            actual_previous,
            expected.previous_line,
        )?;
    }
    if expected.next_line != BrowserNeighborLine::Unobserved {
        let actual_next = browser_neighbor_relation(tree, next, control_interval, flow, path)?;
        let actual_previous = if expected.previous_line == BrowserNeighborLine::Same {
            browser_neighbor_relation(tree, previous, control_interval, flow, path)?
        } else {
            expected.previous_line
        };
        if actual_next != expected.next_line
            && endpoint_is_unobservable(EndpointObservability {
                tree,
                node,
                previous,
                next,
                expected,
                actual_previous,
                actual_next,
                control_interval,
                flow,
                strict_geometry_nodes,
            })
        {
            return Ok(Some(EndpointUnobservableField {
                paths: vec![path.to_string()],
                field: BrowserControlComparisonField::NextLine,
                browser_value: expected.next_line,
                model_value: actual_next,
            }));
        }
        compare_browser_control_identity(
            path,
            "next neighbor line",
            actual_next,
            expected.next_line,
        )?;
    }
    Ok(None)
}

struct EndpointObservability<'a> {
    tree: &'a TestTree,
    node: usize,
    previous: Option<usize>,
    next: Option<usize>,
    expected: BrowserControlExpectation,
    actual_previous: BrowserNeighborLine,
    actual_next: BrowserNeighborLine,
    control_interval: (Scalar, Scalar),
    flow: layout::FlowAxes,
    strict_geometry_nodes: &'a BTreeSet<usize>,
}

fn endpoint_is_unobservable(observation: EndpointObservability<'_>) -> bool {
    let EndpointObservability {
        tree,
        node,
        previous,
        next,
        expected,
        actual_previous,
        actual_next,
        control_interval,
        flow,
        strict_geometry_nodes,
    } = observation;
    let (Some(previous), Some(next)) = (previous, next) else {
        return false;
    };
    let layout::LayoutInput::LineBreak(line_break) = tree.nodes[node].layout_input else {
        return false;
    };
    if line_break.display() != layout::LineBreakDisplay::Break
        || tree.nodes[node].unrounded.size != layout::Size::ZERO
        || !tree.nodes[previous].unrounded_present
        || !tree.nodes[next].unrounded_present
        || !strict_geometry_nodes.contains(&previous)
        || !strict_geometry_nodes.contains(&next)
        || expected.previous_line != BrowserNeighborLine::Same
        || expected.next_line != BrowserNeighborLine::Later
        || actual_previous != BrowserNeighborLine::Same
        || actual_next != BrowserNeighborLine::Same
    {
        return false;
    }

    let previous_interval = output_block_interval(tree.nodes[previous].unrounded, flow);
    let next_interval = output_block_interval(tree.nodes[next].unrounded, flow);
    let shared_endpoint = match flow.block_start() {
        layout::PhysicalSide::Top | layout::PhysicalSide::Left
            if previous_interval.1 == next_interval.0 =>
        {
            previous_interval.1
        }
        layout::PhysicalSide::Right | layout::PhysicalSide::Bottom
            if previous_interval.0 == next_interval.1 =>
        {
            previous_interval.0
        }
        _ => return false,
    };
    control_interval.0 == control_interval.1
        && ComparisonTolerance::browser_parity().contains(control_interval.0 - shared_endpoint)
}

fn browser_neighbor_relation(
    tree: &TestTree,
    neighbor: Option<usize>,
    control_interval: (Scalar, Scalar),
    flow: layout::FlowAxes,
    path: &str,
) -> Result<BrowserNeighborLine, Error> {
    let Some(neighbor) = neighbor else {
        return Ok(BrowserNeighborLine::Absent);
    };
    if !tree.nodes[neighbor].unrounded_present {
        return Err(Error::new(format!(
            "{path}: browser control neighbor-line mismatch, expected unrounded neighbor output"
        )));
    }
    Ok(block_relation(
        control_interval,
        output_block_interval(tree.nodes[neighbor].unrounded, flow),
        flow,
    ))
}

fn output_block_interval(output: layout::NodeOutput, flow: layout::FlowAxes) -> (Scalar, Scalar) {
    match flow.block_axis() {
        layout::PhysicalAxis::Horizontal => {
            (output.location.x, output.location.x + output.size.width)
        }
        layout::PhysicalAxis::Vertical => {
            (output.location.y, output.location.y + output.size.height)
        }
    }
}

fn block_relation(
    control: (Scalar, Scalar),
    neighbor: (Scalar, Scalar),
    flow: layout::FlowAxes,
) -> BrowserNeighborLine {
    let tolerance = ComparisonTolerance::browser_parity().value;
    if neighbor.1 + tolerance >= control.0 && control.1 + tolerance >= neighbor.0 {
        return BrowserNeighborLine::Same;
    }
    let control_center = control.0 + (control.1 - control.0) / 2.0;
    let neighbor_center = neighbor.0 + (neighbor.1 - neighbor.0) / 2.0;
    let neighbor_is_earlier = match flow.block_start() {
        layout::PhysicalSide::Top | layout::PhysicalSide::Left => neighbor_center < control_center,
        layout::PhysicalSide::Right | layout::PhysicalSide::Bottom => {
            neighbor_center > control_center
        }
    };
    if neighbor_is_earlier {
        BrowserNeighborLine::Earlier
    } else {
        BrowserNeighborLine::Later
    }
}

fn compare_browser_control_identity<T>(
    path: &str,
    field: &str,
    actual: T,
    expected: T,
) -> Result<(), Error>
where
    T: Copy + std::fmt::Debug + Eq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: browser control {field} mismatch, expected {expected:?}, got {actual:?}"
        )))
    }
}

fn compare_range_rectangle_identity<T>(
    path: &str,
    index: usize,
    field: &str,
    actual: T,
    expected: T,
) -> Result<(), Error>
where
    T: Copy + std::fmt::Display + Eq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: Range rectangle[{index}] {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_fragment_identity<T>(
    path: &str,
    index: usize,
    field: &str,
    actual: T,
    expected: T,
) -> Result<(), Error>
where
    T: Copy + std::fmt::Display + Eq,
{
    if actual == expected {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: fragment[{index}] {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_number(path: &str, field: &str, actual: Scalar, expected: Scalar) -> Result<(), Error> {
    if ComparisonTolerance::browser_parity().contains(actual - expected) {
        Ok(())
    } else {
        Err(Error::new(format!(
            "{path}: {field} mismatch, expected {expected}, got {actual}"
        )))
    }
}

fn compare_optional_number(
    path: &str,
    field: &str,
    actual: Scalar,
    expected: Option<Scalar>,
) -> Result<(), Error> {
    match expected {
        Some(expected) => compare_number(path, field, actual, expected),
        None => Ok(()),
    }
}

fn to_layout_input(attrs: &StyleAttrs) -> Result<layout::LayoutInput, Error> {
    to_layout_input_in_flow(attrs, None)
}

fn to_layout_input_in_flow(
    attrs: &StyleAttrs,
    containing_flow: Option<layout::FlowAxes>,
) -> Result<layout::LayoutInput, Error> {
    let input = to_node_input_in_flow(attrs, containing_flow)?;
    let br_is_inline_control = attrs.get("source-tag") == Some("br")
        && matches!(attrs.get("display"), Some("inline" | "none"));
    if let Some(kind) = attrs.get("line-control")
        && (kind != "forced-break" || !br_is_inline_control)
    {
        return Err(Error::new(format!(
            "unsupported explicit fixture line control `{kind}`"
        )));
    }
    if br_is_inline_control {
        let flow = containing_flow
            .unwrap_or_else(|| layout::FlowAxes::new(input.writing_mode, input.direction));
        let mut br = layout::LineBreakInput::new()
            .with_direction(flow.direction())
            .with_writing_mode(flow.writing_mode())
            .with_vertical_align(input.vertical_align)
            .with_clear(input.clear);
        if input.display == layout::Display::None {
            br = br.hidden();
        }
        if let Some(metrics) = inline_metrics(attrs)? {
            br = br.with_metrics(metrics);
        }
        Ok(layout::LayoutInput::line_break(br))
    } else {
        Ok(layout::LayoutInput::box_input(input))
    }
}

fn to_node_input(attrs: &StyleAttrs) -> Result<layout::NodeInput, Error> {
    to_node_input_in_flow(attrs, None)
}

fn to_node_input_in_flow(
    attrs: &StyleAttrs,
    containing_flow: Option<layout::FlowAxes>,
) -> Result<layout::NodeInput, Error> {
    for name in [
        "overflow",
        "scroll-padding",
        "scroll-margin",
        "transform",
        "visibility",
    ] {
        if attrs.get(name).is_some() {
            return Err(Error::new(format!(
                "unsupported authored fixture attribute `{name}`"
            )));
        }
    }
    let mut input = layout::NodeInput {
        overflow: parse_computed_overflow(attrs)?,
        ..layout::NodeInput::default()
    };
    let source_tag = attrs.get("source-tag");
    input.display = match attrs.get("display") {
        Some("inline") if source_tag == Some("br") => layout::Display::Block,
        Some(value) => parse_display(value)?,
        None => match source_tag {
            Some("div") => layout::Display::Block,
            _ => input.display,
        },
    };
    if let Some(value) = attrs.get("order") {
        input.item_order = parse_item_order(value)?;
    }
    if let Some(value) = attrs.get("flex-item-collapse") {
        input.flex_item_collapse = match value {
            "collapsed" => layout::FlexItemCollapse::Collapsed,
            _ => {
                return Err(Error::new(format!(
                    "unsupported normalized flex-item collapse `{value}`"
                )));
            }
        };
    }
    if let Some(value) = attrs.get("box-sizing") {
        input.box_sizing = parse_box_sizing(value)?;
    }
    if let Some(value) = attrs.get("direction") {
        input.direction = parse_direction(value)?;
    }
    if let Some(value) = attrs.get("position") {
        input.position = parse_position(value)?;
    }
    if let Some(value) = attrs.get("float-exclusion") {
        input.float_exclusion = match value {
            "shape" => layout::FloatExclusion::Shape,
            _ => {
                return Err(Error::new(format!(
                    "unsupported fixture float exclusion `{value}`"
                )));
            }
        };
    }
    if let Some(value) = attrs.get("scrollbar-width") {
        input.scrollbar_width = layout::ScrollbarWidth::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("overflow-clip-margin") {
        input.overflow_clip_margin = parse_overflow_clip_margin(value)?;
    }
    if let Some(value) = attrs.get("scrollbar-gutter") {
        input.scrollbar_gutter = parse_scrollbar_gutter(value)?;
    }
    input.scroll_padding = layout::ScrollPadding::new(
        parse_scroll_padding(attrs.get("scroll-padding-top"))?,
        parse_scroll_padding(attrs.get("scroll-padding-right"))?,
        parse_scroll_padding(attrs.get("scroll-padding-bottom"))?,
        parse_scroll_padding(attrs.get("scroll-padding-left"))?,
    );
    input.scroll_margin = layout::ScrollMargin::try_new(
        parse_scroll_margin(attrs.get("scroll-margin-top"))?,
        parse_scroll_margin(attrs.get("scroll-margin-right"))?,
        parse_scroll_margin(attrs.get("scroll-margin-bottom"))?,
        parse_scroll_margin(attrs.get("scroll-margin-left"))?,
    )
    .map_err(|source| Error::new(source.to_string()))?;
    if let Some(value) = attrs.get("scroll-snap-type") {
        input.scroll_snap_type = parse_scroll_snap_type(value)?;
    }
    if let Some(value) = attrs.get("scroll-snap-align") {
        input.scroll_snap_align = parse_scroll_snap_align(value)?;
    }
    if let Some(value) = attrs.get("scroll-snap-stop") {
        input.scroll_snap_stop = parse_scroll_snap_stop(value)?;
    }
    if let Some(value) = attrs.get("text-align") {
        input.text_align = parse_text_align(value)?;
    }
    if let Some(value) = attrs.get("vertical-align") {
        input.vertical_align = parse_vertical_align(value)?;
    }
    input.writing_mode = parse_writing_mode(attrs.get("writing-mode"))?;
    let containing_flow = containing_flow
        .unwrap_or_else(|| layout::FlowAxes::new(input.writing_mode, input.direction));
    if let Some(value) = attrs.get("float") {
        input.float = parse_float(value, containing_flow)?;
    }
    if let Some(value) = attrs.get("clear") {
        input.clear = parse_clear(value, containing_flow)?;
    }
    if let Some(value) = attrs.get("flex-direction") {
        input.flex_direction = parse_flex_direction(value)?;
    }
    if let Some(value) = attrs.get("flex-wrap") {
        input.flex_wrap = parse_flex_wrap(value)?;
    }
    if let Some(value) = attrs.get("flex-grow") {
        input.flex_grow = layout::FlexGrow::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("flex-shrink") {
        input.flex_shrink = layout::FlexShrink::try_new(parse_number(value)?)
            .map_err(|source| Error::new(source.to_string()))?;
    }
    if let Some(value) = attrs.get("flex-basis") {
        input.flex_basis = parse_flex_basis(value)?;
    }
    if let Some(value) = attrs.get("width") {
        input.size.width = parse_preferred_size(value)?;
    }
    if let Some(value) = attrs.get("height") {
        input.size.height = parse_preferred_size(value)?;
    }
    if let Some(value) = attrs.get("min-width") {
        input.min_size.width = parse_min_size(value)?;
    }
    if let Some(value) = attrs.get("min-height") {
        input.min_size.height = parse_min_size(value)?;
    }
    if let Some(value) = attrs.get("max-width") {
        input.max_size.width = parse_max_size(value)?;
    }
    if let Some(value) = attrs.get("max-height") {
        input.max_size.height = parse_max_size(value)?;
    }
    if let Some(value) = attrs.get("aspect-ratio") {
        let value = parse_number(value)?;
        input.aspect_ratio = layout::AspectRatio::new(value);
        if input.aspect_ratio.is_none() {
            return Err(Error::new(format!("invalid aspect-ratio `{value}`")));
        }
    }
    let flow_axes = layout::FlowAxes::new(input.writing_mode, input.direction);
    let (default_inline_gap, default_block_gap) = match flow_axes.inline_axis() {
        layout::PhysicalAxis::Horizontal => (input.gap.width, input.gap.height),
        layout::PhysicalAxis::Vertical => (input.gap.height, input.gap.width),
    };
    let inline_gap = match attrs.get("column-gap") {
        Some(value) => parse_length_with_calc(value)?,
        None => default_inline_gap,
    };
    let block_gap = match attrs.get("row-gap") {
        Some(value) => parse_length_with_calc(value)?,
        None => default_block_gap,
    };
    input.gap = match flow_axes.inline_axis() {
        layout::PhysicalAxis::Horizontal => layout::Size::new(inline_gap, block_gap),
        layout::PhysicalAxis::Vertical => layout::Size::new(block_gap, inline_gap),
    };

    apply_edges_auto(
        &mut input.margin,
        attrs,
        [
            ("margin-top", 0),
            ("margin-right", 1),
            ("margin-bottom", 2),
            ("margin-left", 3),
        ],
        layout::LengthAuto::ZERO,
    )?;
    apply_edges(
        &mut input.padding,
        attrs,
        [
            ("padding-top", 0),
            ("padding-right", 1),
            ("padding-bottom", 2),
            ("padding-left", 3),
        ],
        layout::Length::ZERO,
    )?;
    apply_edges(
        &mut input.border,
        attrs,
        [
            ("border-top", 0),
            ("border-right", 1),
            ("border-bottom", 2),
            ("border-left", 3),
        ],
        layout::Length::ZERO,
    )?;
    apply_edges_auto(
        &mut input.inset,
        attrs,
        [("top", 0), ("right", 1), ("bottom", 2), ("left", 3)],
        layout::LengthAuto::AUTO,
    )?;

    if let Some(value) = attrs.get("align-items") {
        input.align_items = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("align-self") {
        input.align_self = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("justify-items") {
        input.justify_items = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("justify-self") {
        input.justify_self = Some(parse_align_items(value)?);
    }
    if let Some(value) = attrs.get("align-content") {
        input.align_content = Some(parse_align_content(value)?);
    }
    if let Some(value) = attrs.get("justify-content") {
        input.justify_content = Some(parse_align_content(value)?);
    }
    if let Some(value) = attrs.get("grid-auto-flow") {
        input.grid_auto_flow = parse_grid_auto_flow(value)?;
    }
    if let Some(value) = attrs.get("grid-template-columns") {
        input.grid_template_columns = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-template-rows") {
        input.grid_template_rows = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-template-areas") {
        input.grid_template_areas = parse_grid_template_areas(value)?;
    }
    if let Some(value) = attrs.get("grid-auto-columns") {
        input.grid_auto_columns = parse_track_component_list_with_calc(value)?;
    }
    if let Some(value) = attrs.get("grid-auto-rows") {
        input.grid_auto_rows = parse_track_component_list_with_calc(value)?;
    }
    let (grid_column, raw_grid_column) =
        parse_grid_placement(attrs.get("grid-column-start"), attrs.get("grid-column-end"))?;
    input.grid_column = grid_column;
    input.raw_grid_column = raw_grid_column;
    let (grid_row, raw_grid_row) =
        parse_grid_placement(attrs.get("grid-row-start"), attrs.get("grid-row-end"))?;
    input.grid_row = grid_row;
    input.raw_grid_row = raw_grid_row;

    Ok(input)
}

fn apply_edges(
    edges: &mut layout::Edges<layout::Length>,
    attrs: &StyleAttrs,
    names: [(&str, usize); 4],
    default: layout::Length,
) -> Result<(), Error> {
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            set_edge(edges, side, parse_length_with_calc(value)?);
        } else {
            set_edge(edges, side, default);
        }
    }
    Ok(())
}

fn apply_edges_auto(
    edges: &mut layout::Edges<layout::LengthAuto>,
    attrs: &StyleAttrs,
    names: [(&str, usize); 4],
    default: layout::LengthAuto,
) -> Result<(), Error> {
    for (name, side) in names {
        if let Some(value) = attrs.get(name) {
            set_edge(edges, side, parse_length_auto_with_calc(value)?);
        } else {
            set_edge(edges, side, default);
        }
    }
    Ok(())
}

fn set_edge<T>(edges: &mut layout::Edges<T>, side: usize, value: T) {
    match side {
        0 => edges.top = value,
        1 => edges.right = value,
        2 => edges.bottom = value,
        3 => edges.left = value,
        _ => unreachable!("edge side index is fixed by caller"),
    }
}

fn font_size(attrs: &StyleAttrs) -> Result<Option<Scalar>, Error> {
    match attrs.get("font-size") {
        Some(value) => Ok(Some(parse_px_dimension(value, "font-size")?)),
        None => Ok(None),
    }
}

fn line_height(attrs: &StyleAttrs) -> Result<Option<Scalar>, Error> {
    match attrs.get("line-height") {
        Some(value) => Ok(Some(parse_px_dimension(value, "line-height")?)),
        None => Ok(None),
    }
}

fn inline_metrics(attrs: &StyleAttrs) -> Result<Option<layout::InlineMetrics>, Error> {
    match (
        attrs.get("inline-baseline"),
        attrs.get("inline-line-height"),
    ) {
        (None, None) => Ok(None),
        (Some(baseline), Some(line_height)) => {
            layout::InlineMetrics::from_line_height_and_baseline(
                surgeist_layout::InlineBaselineContext::Physical,
                parse_px_dimension(line_height, "inline-line-height")?,
                parse_px_dimension(baseline, "inline-baseline")?,
            )
            .map(Some)
            .map_err(|error| Error::new(format!("{error:?}")))
        }
        _ => Err(Error::new(
            "inline metrics require inline-baseline and inline-line-height",
        )),
    }
}

fn font_family(attrs: &StyleAttrs) -> Result<Option<FontFamily>, Error> {
    match attrs.get("font-family") {
        None => Ok(None),
        Some("ahem") => Ok(Some(FontFamily::Ahem)),
        Some("monospace") => Ok(Some(FontFamily::Monospace)),
        Some(value) => Err(Error::new(format!(
            "unsupported parity fixture font-family `{value}`"
        ))),
    }
}

fn parse_px_dimension(raw: &str, name: &str) -> Result<Scalar, Error> {
    let value = raw
        .strip_suffix("px")
        .ok_or_else(|| Error::new(format!("{name} must use px units, got `{raw}`")))?;
    parse_number(value)
}

fn parse_raw_grid_line(raw: &str) -> Result<layout::RawGridLine, Error> {
    let tokens = split_top_level_whitespace(raw);
    match tokens.as_slice() {
        [token] if token == "auto" => Ok(layout::RawGridLine::Auto),
        [token] if token == "span" => Err(Error::new("invalid grid span `span`")),
        [token] => match parse_style_line_index(token) {
            Ok(0) => Err(Error::new(format!("grid line cannot be zero in `{raw}`"))),
            Ok(line) => Ok(layout::RawGridLine::Line(line.into())),
            Err(_) => Ok(layout::RawGridLine::BareIdent(
                parse_custom_ident(token)?.to_owned(),
            )),
        },
        [span, token] if span == "span" => {
            if let Ok(index) = parse_style_span_index(token) {
                Ok(layout::RawGridLine::Span(index.into()))
            } else {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(token)?.to_owned(),
                    index: 1,
                })
            }
        }
        [name, index] if parse_style_line_index(index).is_ok() => named_raw_line(name, index, raw),
        [index, name] if parse_style_line_index(index).is_ok() => named_raw_line(name, index, raw),
        [span, first, second] if span == "span" => {
            if let Ok(index) = parse_style_span_index(first) {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(second)?.to_owned(),
                    index: index.into(),
                })
            } else {
                Ok(layout::RawGridLine::NamedSpan {
                    name: parse_custom_ident(first)?.to_owned(),
                    index: parse_style_span_index(second)?.into(),
                })
            }
        }
        _ => Err(Error::new(format!("unsupported grid line `{raw}`"))),
    }
}

fn parse_grid_placement(
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(layout::GridPlacement, layout::RawGridPlacement), Error> {
    let start = start.unwrap_or("auto");
    let end = end.unwrap_or("auto");
    let mut span = None;
    let legacy_start = parse_grid_line_or_span(start, &mut span).unwrap_or(None);
    let legacy_end = parse_grid_line_or_span(end, &mut span).unwrap_or(None);
    let legacy = match (legacy_start, legacy_end, span) {
        (Some(start), Some(end), None) => layout::GridPlacement::try_lines(start, end),
        (Some(start), None, Some(span)) => layout::GridPlacement::try_line_span(start, span),
        (None, Some(end), Some(span)) => layout::GridPlacement::try_span_line(span, end),
        (Some(start), None, None) => layout::GridPlacement::try_line(start),
        (None, Some(end), None) => layout::GridPlacement::try_end_line(end),
        (None, None, Some(span)) => layout::GridPlacement::try_span(span),
        (None, None, None) => Some(layout::GridPlacement::AUTO),
        (Some(_), Some(_), Some(_)) => None,
    }
    .unwrap_or(layout::GridPlacement::AUTO);
    let raw = layout::RawGridPlacement::new(parse_raw_grid_line(start)?, parse_raw_grid_line(end)?);
    Ok((legacy, raw))
}

fn parse_grid_template_areas(raw: &str) -> Result<layout::GridTemplateAreas, Error> {
    if raw.trim().is_empty() || raw.trim() == "none" {
        return Err(Error::new(format!(
            "unsupported explicit grid template areas `{raw}`"
        )));
    }
    let rows = raw
        .split('/')
        .map(str::trim)
        .map(parse_grid_template_area_row)
        .collect::<Result<Vec<_>, _>>()?;
    let column_count = rows
        .first()
        .map(|row| row.cells.len())
        .filter(|count| *count > 0)
        .ok_or_else(|| Error::new("grid template areas require at least one non-empty row"))?;
    if rows.iter().any(|row| row.cells.len() != column_count) {
        return Err(Error::new(
            "grid template area rows must have equal column counts",
        ));
    }
    validate_grid_template_area_rectangles(&rows)?;
    Ok(layout::GridTemplateAreas { rows })
}

fn parse_grid_template_area_row(raw: &str) -> Result<layout::GridTemplateAreaRow, Error> {
    if raw.is_empty() {
        return Err(Error::new("grid template area rows must not be empty"));
    }
    let cells = split_top_level_whitespace(raw)
        .into_iter()
        .map(|cell| {
            if is_grid_template_area_null_cell(&cell) {
                Ok(None)
            } else {
                parse_grid_template_area_ident(&cell).map(|name| Some(name.to_owned()))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layout::GridTemplateAreaRow { cells })
}

fn parse_grid_template_area_ident(raw: &str) -> Result<&str, Error> {
    if matches!(
        raw,
        "auto"
            | "default"
            | "inherit"
            | "initial"
            | "none"
            | "revert"
            | "revert-layer"
            | "span"
            | "unset"
    ) {
        return Err(Error::new(format!(
            "invalid grid template area name `{raw}`"
        )));
    }
    let mut bytes = raw.strip_prefix('-').unwrap_or(raw).bytes();
    let valid_start = match bytes.next() {
        Some(first) if first.is_ascii_alphabetic() || first == b'_' => true,
        Some(b'-') => bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_'),
        _ => false,
    };
    if !valid_start
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(Error::new(format!(
            "invalid grid template area name `{raw}`"
        )));
    }
    Ok(raw)
}

fn validate_grid_template_area_rectangles(
    rows: &[layout::GridTemplateAreaRow],
) -> Result<(), Error> {
    let mut areas = BTreeMap::<&str, (usize, usize, usize, usize, usize)>::new();
    for (row_index, row) in rows.iter().enumerate() {
        for (column_index, cell) in row.cells.iter().enumerate() {
            let Some(name) = cell.as_deref() else {
                continue;
            };
            let area =
                areas
                    .entry(name)
                    .or_insert((row_index, row_index, column_index, column_index, 0));
            area.0 = area.0.min(row_index);
            area.1 = area.1.max(row_index);
            area.2 = area.2.min(column_index);
            area.3 = area.3.max(column_index);
            area.4 += 1;
        }
    }
    for (name, (row_start, row_end, column_start, column_end, count)) in areas {
        let rectangle_cells = (row_end - row_start + 1) * (column_end - column_start + 1);
        if rectangle_cells != count {
            return Err(Error::new(format!(
                "grid template area `{name}` must form one rectangle"
            )));
        }
    }
    Ok(())
}

fn is_grid_template_area_null_cell(cell: &str) -> bool {
    !cell.is_empty() && cell.bytes().all(|byte| byte == b'.')
}

fn named_raw_line(name: &str, index: &str, raw: &str) -> Result<layout::RawGridLine, Error> {
    let index = parse_style_line_index(index)?;
    if index == 0 {
        return Err(Error::new(format!(
            "named grid line occurrence cannot be zero in `{raw}`"
        )));
    }
    Ok(layout::RawGridLine::NamedLine {
        name: parse_custom_ident(name)?.to_owned(),
        index: index.into(),
    })
}

fn parse_style_line_index(raw: &str) -> Result<i16, Error> {
    raw.parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))
}

fn parse_style_span_index(raw: &str) -> Result<u16, Error> {
    let index: u16 = raw
        .parse()
        .map_err(|_| Error::new(format!("invalid grid span `{raw}`")))?;
    if index == 0 {
        return Err(Error::new("grid span cannot be zero"));
    }
    Ok(index)
}

fn parse_custom_ident(raw: &str) -> Result<&str, Error> {
    if matches!(raw, "auto" | "span") || raw.parse::<i64>().is_ok() {
        return Err(Error::new(format!("invalid grid custom-ident `{raw}`")));
    }
    Ok(raw)
}

fn to_layout_available(value: Available) -> layout::Available {
    match value {
        Available::Definite(value) => layout::Available::definite(value),
        Available::MinContent => layout::Available::MIN_CONTENT,
        Available::MaxContent => layout::Available::MAX_CONTENT,
    }
}

fn parse_display(raw: &str) -> Result<layout::Display, Error> {
    match raw {
        "block" => Ok(layout::Display::Block),
        "inline-block" => Ok(layout::Display::InlineBlock),
        "flex" => Ok(layout::Display::Flex),
        "grid" => Ok(layout::Display::Grid),
        "inline-grid" => Ok(layout::Display::InlineGrid),
        "grid-lanes" => Ok(layout::Display::GridLanes),
        "inline-grid-lanes" => Ok(layout::Display::InlineGridLanes),
        "none" => Ok(layout::Display::None),
        _ => Err(Error::new(format!("unsupported display `{raw}`"))),
    }
}

fn parse_box_sizing(raw: &str) -> Result<layout::BoxSizing, Error> {
    match raw {
        "border-box" => Ok(layout::BoxSizing::BorderBox),
        "content-box" => Ok(layout::BoxSizing::ContentBox),
        _ => Err(Error::new(format!("unsupported box-sizing `{raw}`"))),
    }
}

fn parse_direction(raw: &str) -> Result<layout::Direction, Error> {
    match raw {
        "ltr" => Ok(layout::Direction::Ltr),
        "rtl" => Ok(layout::Direction::Rtl),
        _ => Err(Error::new(format!("unsupported direction `{raw}`"))),
    }
}

fn parse_position(raw: &str) -> Result<layout::Position, Error> {
    match raw {
        "relative" => Ok(layout::Position::Relative),
        "absolute" => Ok(layout::Position::Absolute),
        _ => Err(Error::new(format!("unsupported position `{raw}`"))),
    }
}

fn parse_float(raw: &str, containing_flow: layout::FlowAxes) -> Result<layout::Float, Error> {
    match raw {
        "none" => Ok(layout::Float::None),
        "inline-start" => Ok(layout::Float::Left),
        "inline-end" => Ok(layout::Float::Right),
        "left" => lower_physical_fixture_side(
            layout::PhysicalSide::Left,
            containing_flow,
            "float",
            raw,
            layout::Float::Left,
            layout::Float::Right,
        ),
        "right" => lower_physical_fixture_side(
            layout::PhysicalSide::Right,
            containing_flow,
            "float",
            raw,
            layout::Float::Left,
            layout::Float::Right,
        ),
        _ => Err(Error::new(format!("unsupported float `{raw}`"))),
    }
}

fn parse_clear(raw: &str, containing_flow: layout::FlowAxes) -> Result<layout::Clear, Error> {
    match raw {
        "none" => Ok(layout::Clear::None),
        "inline-start" => Ok(layout::Clear::Left),
        "inline-end" => Ok(layout::Clear::Right),
        "left" => lower_physical_fixture_side(
            layout::PhysicalSide::Left,
            containing_flow,
            "clear",
            raw,
            layout::Clear::Left,
            layout::Clear::Right,
        ),
        "right" => lower_physical_fixture_side(
            layout::PhysicalSide::Right,
            containing_flow,
            "clear",
            raw,
            layout::Clear::Left,
            layout::Clear::Right,
        ),
        "both" => Ok(layout::Clear::Both),
        _ => Err(Error::new(format!("unsupported clear `{raw}`"))),
    }
}

fn lower_physical_fixture_side<T>(
    physical_side: layout::PhysicalSide,
    containing_flow: layout::FlowAxes,
    property: &str,
    raw: &str,
    line_start: T,
    line_end: T,
) -> Result<T, Error> {
    if physical_side == containing_flow.inline_start() {
        Ok(line_start)
    } else if physical_side == containing_flow.inline_end() {
        Ok(line_end)
    } else {
        Err(Error::new(format!(
            "unsupported physical fixture-lowering {property} `{raw}` for {:?} {:?}",
            containing_flow.writing_mode(),
            containing_flow.direction(),
        )))
    }
}

fn parse_overflow(raw: &str) -> Result<layout::Overflow, Error> {
    match raw {
        "visible" => Ok(layout::Overflow::Visible),
        "clip" => Ok(layout::Overflow::Clip),
        "hidden" => Ok(layout::Overflow::Hidden),
        "scroll" => Ok(layout::Overflow::Scroll),
        "auto" => Ok(layout::Overflow::Auto),
        _ => Err(Error::new(format!("unsupported overflow `{raw}`"))),
    }
}

fn parse_computed_overflow(attrs: &StyleAttrs) -> Result<layout::ComputedOverflow, Error> {
    let x = attrs
        .get("overflow-x")
        .map(parse_overflow)
        .transpose()?
        .unwrap_or(layout::Overflow::Visible);
    let y = attrs
        .get("overflow-y")
        .map(parse_overflow)
        .transpose()?
        .unwrap_or(layout::Overflow::Visible);

    layout::ComputedOverflow::try_new(x, y)
        .map_err(|error| Error::new(format!("invalid computed overflow pair: {error}")))
}

fn parse_overflow_clip_margin(raw: &str) -> Result<layout::OverflowClipMargin, Error> {
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let (clip_box, length) = match parts.as_slice() {
        [length] => match *length {
            "content-box" => (layout::OverflowClipBox::ContentBox, "0px"),
            "padding-box" => (layout::OverflowClipBox::PaddingBox, "0px"),
            "border-box" => (layout::OverflowClipBox::BorderBox, "0px"),
            length => (layout::OverflowClipBox::PaddingBox, length),
        },
        [clip_box, length] => {
            let clip_box = match *clip_box {
                "content-box" => layout::OverflowClipBox::ContentBox,
                "padding-box" => layout::OverflowClipBox::PaddingBox,
                "border-box" => layout::OverflowClipBox::BorderBox,
                _ => {
                    return Err(Error::new(format!(
                        "unsupported overflow clip box `{clip_box}`"
                    )));
                }
            };
            (clip_box, *length)
        }
        _ => {
            return Err(Error::new(format!(
                "unsupported overflow clip margin `{raw}`"
            )));
        }
    };
    let margin = parse_px(length, "overflow clip margin")?;
    layout::OverflowClipMargin::try_new(clip_box, margin)
        .map_err(|source| Error::new(source.to_string()))
}

fn parse_scrollbar_gutter(raw: &str) -> Result<layout::ScrollbarGutter, Error> {
    match raw {
        "auto" => Ok(layout::ScrollbarGutter::Auto),
        "stable" => Ok(layout::ScrollbarGutter::Stable),
        "stable both-edges" => Ok(layout::ScrollbarGutter::StableBothEdges),
        _ => Err(Error::new(format!("unsupported scrollbar gutter `{raw}`"))),
    }
}

fn parse_scroll_padding(raw: Option<&str>) -> Result<layout::ScrollPaddingValue, Error> {
    let Some(raw) = raw else {
        return Ok(layout::ScrollPaddingValue::AUTO);
    };
    if raw == "auto" {
        return Ok(layout::ScrollPaddingValue::AUTO);
    }
    let value = if raw.starts_with("calc(") {
        parse_calc_expression(raw)?
    } else if let Some(px) = raw.strip_suffix("px") {
        length_percentage_px(parse_number(px)?, raw)?
    } else if let Some(percent) = raw.strip_suffix('%') {
        length_percentage_percent(parse_number(percent)? / 100.0, raw)?
    } else {
        return Err(Error::new(format!("unsupported scroll padding `{raw}`")));
    };
    Ok(layout::ScrollPaddingValue::value(value))
}

fn parse_scroll_margin(raw: Option<&str>) -> Result<Scalar, Error> {
    raw.map_or(Ok(0.0), |raw| parse_px(raw, "scroll margin"))
}

fn parse_px(raw: &str, property: &str) -> Result<Scalar, Error> {
    raw.strip_suffix("px")
        .ok_or_else(|| Error::new(format!("unsupported {property} `{raw}`")))
        .and_then(parse_number)
}

fn parse_scroll_snap_type(raw: &str) -> Result<layout::ScrollSnapType, Error> {
    if raw == "none" {
        return Ok(layout::ScrollSnapType::None);
    }
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let [axis, strictness] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported scroll snap type `{raw}`")));
    };
    let axis = match *axis {
        "x" => layout::ScrollSnapAxis::X,
        "y" => layout::ScrollSnapAxis::Y,
        "block" => layout::ScrollSnapAxis::Block,
        "inline" => layout::ScrollSnapAxis::Inline,
        "both" => layout::ScrollSnapAxis::Both,
        _ => return Err(Error::new(format!("unsupported scroll snap axis `{axis}`"))),
    };
    let strictness = match *strictness {
        "proximity" => layout::ScrollSnapStrictness::Proximity,
        "mandatory" => layout::ScrollSnapStrictness::Mandatory,
        _ => {
            return Err(Error::new(format!(
                "unsupported scroll snap strictness `{strictness}`"
            )));
        }
    };
    Ok(layout::ScrollSnapType::Enabled { axis, strictness })
}

fn parse_scroll_snap_align(raw: &str) -> Result<layout::ScrollSnapAlign, Error> {
    let parts = raw.split_whitespace().collect::<Vec<_>>();
    let [block, inline] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported scroll snap align `{raw}`")));
    };
    let parse_value = |value| match value {
        "none" => Ok(layout::ScrollSnapAlignValue::None),
        "start" => Ok(layout::ScrollSnapAlignValue::Start),
        "end" => Ok(layout::ScrollSnapAlignValue::End),
        "center" => Ok(layout::ScrollSnapAlignValue::Center),
        _ => Err(Error::new(format!(
            "unsupported scroll snap alignment `{value}`"
        ))),
    };
    Ok(layout::ScrollSnapAlign::new(
        parse_value(block)?,
        parse_value(inline)?,
    ))
}

fn parse_scroll_snap_stop(raw: &str) -> Result<layout::ScrollSnapStop, Error> {
    match raw {
        "normal" => Ok(layout::ScrollSnapStop::Normal),
        "always" => Ok(layout::ScrollSnapStop::Always),
        _ => Err(Error::new(format!("unsupported scroll snap stop `{raw}`"))),
    }
}

fn parse_text_align(raw: &str) -> Result<layout::TextAlign, Error> {
    match raw {
        "left" | "-webkit-left" => Ok(layout::TextAlign::LegacyLeft),
        "right" | "-webkit-right" => Ok(layout::TextAlign::LegacyRight),
        "center" | "-webkit-center" => Ok(layout::TextAlign::LegacyCenter),
        _ => Err(Error::new(format!("unsupported text-align `{raw}`"))),
    }
}

fn parse_vertical_align(raw: &str) -> Result<layout::VerticalAlign, Error> {
    match raw {
        "baseline" => Ok(layout::VerticalAlign::Baseline),
        "top" => Ok(layout::VerticalAlign::Top),
        "bottom" => Ok(layout::VerticalAlign::Bottom),
        _ => Err(Error::new(format!(
            "unsupported parity fixture vertical-align `{raw}`"
        ))),
    }
}

fn parse_writing_mode(raw: Option<&str>) -> Result<layout::WritingMode, Error> {
    match raw {
        None | Some("horizontal-tb") => Ok(layout::WritingMode::HorizontalTb),
        Some("vertical-lr") => Ok(layout::WritingMode::VerticalLr),
        Some("vertical-rl") => Ok(layout::WritingMode::VerticalRl),
        Some("sideways-rl") => Ok(layout::WritingMode::SidewaysRl),
        Some("sideways-lr") => Ok(layout::WritingMode::SidewaysLr),
        Some(value) => Err(Error::new(format!("unsupported writing-mode `{value}`"))),
    }
}

fn parse_flex_direction(raw: &str) -> Result<layout::FlexDirection, Error> {
    match raw {
        "row" => Ok(layout::FlexDirection::Row),
        "column" => Ok(layout::FlexDirection::Column),
        "row-reverse" => Ok(layout::FlexDirection::RowReverse),
        "column-reverse" => Ok(layout::FlexDirection::ColumnReverse),
        _ => Err(Error::new(format!("unsupported flex-direction `{raw}`"))),
    }
}

fn parse_flex_wrap(raw: &str) -> Result<layout::FlexWrap, Error> {
    match raw {
        "nowrap" => Ok(layout::FlexWrap::NoWrap),
        "wrap" => Ok(layout::FlexWrap::Wrap),
        "wrap-reverse" => Ok(layout::FlexWrap::WrapReverse),
        _ => Err(Error::new(format!("unsupported flex-wrap `{raw}`"))),
    }
}

fn parse_align_items(raw: &str) -> Result<layout::AlignItems, Error> {
    let safe = raw.starts_with("safe ");
    let has_overflow_prefix = safe || raw.starts_with("unsafe ");
    let raw = alignment_keyword(raw);
    match (safe, has_overflow_prefix, raw) {
        (true, _, "end") => Ok(layout::AlignItems::SafeEnd),
        (true, _, "flex-end") => Ok(layout::AlignItems::SafeFlexEnd),
        (true, _, "center") => Ok(layout::AlignItems::SafeCenter),
        (_, _, "start") => Ok(layout::AlignItems::Start),
        (_, _, "end") => Ok(layout::AlignItems::End),
        (_, _, "flex-start") => Ok(layout::AlignItems::FlexStart),
        (_, _, "flex-end") => Ok(layout::AlignItems::FlexEnd),
        (_, _, "center") => Ok(layout::AlignItems::Center),
        (_, false, "baseline") => Ok(layout::AlignItems::Baseline),
        (_, false, "first baseline") => Ok(layout::AlignItems::Baseline),
        (_, false, "last baseline") => Ok(layout::AlignItems::LastBaseline),
        (_, _, "stretch") => Ok(layout::AlignItems::Stretch),
        _ => Err(Error::new(format!("unsupported alignment `{raw}`"))),
    }
}

fn parse_align_content(raw: &str) -> Result<layout::AlignContent, Error> {
    let safe = raw.starts_with("safe ");
    let raw = alignment_keyword(raw);
    match (safe, raw) {
        (true, "end") => Ok(layout::AlignContent::SafeEnd),
        (true, "flex-end") => Ok(layout::AlignContent::SafeFlexEnd),
        (true, "center") => Ok(layout::AlignContent::SafeCenter),
        (_, "start") => Ok(layout::AlignContent::Start),
        (_, "end") => Ok(layout::AlignContent::End),
        (_, "flex-start") => Ok(layout::AlignContent::FlexStart),
        (_, "flex-end") => Ok(layout::AlignContent::FlexEnd),
        (_, "center") => Ok(layout::AlignContent::Center),
        (_, "stretch") => Ok(layout::AlignContent::Stretch),
        (_, "space-between") => Ok(layout::AlignContent::SpaceBetween),
        (_, "space-evenly") => Ok(layout::AlignContent::SpaceEvenly),
        (_, "space-around") => Ok(layout::AlignContent::SpaceAround),
        _ => Err(Error::new(format!("unsupported content alignment `{raw}`"))),
    }
}

fn alignment_keyword(raw: &str) -> &str {
    raw.strip_prefix("safe ")
        .or_else(|| raw.strip_prefix("unsafe "))
        .unwrap_or(raw)
}

fn parse_grid_auto_flow(raw: &str) -> Result<layout::GridAutoFlow, Error> {
    match raw {
        "row" => Ok(layout::GridAutoFlow::Row),
        "column" => Ok(layout::GridAutoFlow::Column),
        "row dense" => Ok(layout::GridAutoFlow::RowDense),
        "column dense" => Ok(layout::GridAutoFlow::ColumnDense),
        _ => Err(Error::new(format!("unsupported grid-auto-flow `{raw}`"))),
    }
}

fn parse_calc_expression(raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let body = raw
        .strip_prefix("calc(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| Error::new(format!("unsupported calc expression `{raw}`")))?;
    parse_calc_sum(body.trim(), raw)
}

fn parse_calc_sum(body: &str, raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let parts = body.split_whitespace().collect::<Vec<_>>();
    let [first, operator, second] = parts.as_slice() else {
        return Err(Error::new(format!("unsupported calc expression `{raw}`")));
    };

    let left = parse_calc_term(first, 1.0)?;
    let right = match *operator {
        "+" => parse_calc_term(second, 1.0)?,
        "-" => parse_calc_term(second, -1.0)?,
        _ => return Err(Error::new(format!("unsupported calc expression `{raw}`"))),
    };

    layout::LengthPercentageOf::from_coefficients(
        left.absolute_px() + right.absolute_px(),
        left.percent_fraction() + right.percent_fraction(),
    )
    .map_err(|error| Error::new(format!("invalid calc expression `{raw}`: {error}")))
}

fn parse_calc_term(raw: &str, sign: Scalar) -> Result<layout::LengthPercentageOf, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return length_percentage_px(parse_number(px)? * sign, raw);
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return length_percentage_percent(parse_number(percent)? / 100.0 * sign, raw);
    }
    Err(Error::new(format!(
        "unsupported calc expression term `{raw}`"
    )))
}

fn parse_length_with_calc(raw: &str) -> Result<layout::Length, Error> {
    if raw.trim_start().starts_with("calc(") {
        return Ok(layout::Length::value(parse_calc_expression(raw)?));
    }
    parse_length(raw)
}

fn parse_length_auto_with_calc(raw: &str) -> Result<layout::LengthAuto, Error> {
    if raw == "auto" {
        return Ok(layout::LengthAuto::AUTO);
    }
    Ok(parse_length_with_calc(raw)?.into())
}

fn parse_preferred_size(raw: &str) -> Result<layout::PreferredSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::PreferredSize::AUTO),
        "min-content" => Ok(layout::PreferredSize::MIN_CONTENT),
        "max-content" => Ok(layout::PreferredSize::MAX_CONTENT),
        "stretch" => Ok(layout::PreferredSize::STRETCH),
        "fit-content" => Ok(layout::PreferredSize::FIT_CONTENT),
        "contain" => Ok(layout::PreferredSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::PreferredSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_preferred_calc_size_basis(basis, raw)?;
                layout::PreferredSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid preferred-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::PreferredSize::calculation(
                parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
            )),
        },
    }
}

fn parse_min_size(raw: &str) -> Result<layout::MinSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::MinSize::AUTO),
        "min-content" => Ok(layout::MinSize::MIN_CONTENT),
        "max-content" => Ok(layout::MinSize::MAX_CONTENT),
        "stretch" => Ok(layout::MinSize::STRETCH),
        "fit-content" => Ok(layout::MinSize::FIT_CONTENT),
        "contain" => Ok(layout::MinSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::MinSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_min_calc_size_basis(basis, raw)?;
                layout::MinSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid minimum-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::MinSize::calculation(
                parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
            )),
        },
    }
}

fn parse_max_size(raw: &str) -> Result<layout::MaxSize, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "none" => Ok(layout::MaxSize::NONE),
        "min-content" => Ok(layout::MaxSize::MIN_CONTENT),
        "max-content" => Ok(layout::MaxSize::MAX_CONTENT),
        "stretch" => Ok(layout::MaxSize::STRETCH),
        "fit-content" => Ok(layout::MaxSize::FIT_CONTENT),
        "contain" => Ok(layout::MaxSize::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::MaxSize::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_max_calc_size_basis(basis, raw)?;
                layout::MaxSize::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid maximum-size fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::MaxSize::calculation(
                parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
            )),
        },
    }
}

fn parse_flex_basis(raw: &str) -> Result<layout::FlexBasis, Error> {
    let raw = checked_sizing_fixture(raw)?;
    match raw {
        "auto" => Ok(layout::FlexBasis::AUTO),
        "content" => Ok(layout::FlexBasis::CONTENT),
        "min-content" => Ok(layout::FlexBasis::MIN_CONTENT),
        "max-content" => Ok(layout::FlexBasis::MAX_CONTENT),
        "stretch" => Ok(layout::FlexBasis::STRETCH),
        "fit-content" => Ok(layout::FlexBasis::FIT_CONTENT),
        "contain" => Ok(layout::FlexBasis::CONTAIN),
        _ => match parse_sizing_function(raw)? {
            Some(("fit-content", body)) => Ok(layout::FlexBasis::fit_content_function(
                parse_fit_content_argument(body, raw)?,
            )),
            Some(("calc-size", body)) => {
                let (basis, calculation) = parse_calc_size_arguments(body, raw)?;
                let basis = parse_flex_calc_size_basis(basis, raw)?;
                layout::FlexBasis::calc_size(basis, calculation).map_err(|error| {
                    Error::new(format!("invalid flex-basis fixture `{raw}`: {error}"))
                })
            }
            _ => Ok(layout::FlexBasis::calculation(
                parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
            )),
        },
    }
}

fn parse_length(raw: &str) -> Result<layout::Length, Error> {
    if let Some(px) = raw.strip_suffix("px") {
        return length_px(parse_number(px)?, raw);
    }
    if let Some(percent) = raw.strip_suffix('%') {
        return length_percent(parse_number(percent)? / 100.0, raw);
    }
    // Browser parity XML is a typed fixture format. Unitless fixture numbers
    // represent layout lengths; app-facing CSS parsing stays outside layout math.
    if let Ok(value) = parse_number(raw) {
        return length_px(value, raw);
    }
    Err(Error::new(format!("unsupported length `{raw}`")))
}

const MAX_SIZING_FUNCTION_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug)]
struct FixtureSizingCoefficients {
    absolute_px: Scalar,
    percent_fraction: Scalar,
    size_fraction: Scalar,
}

fn checked_sizing_fixture(raw: &str) -> Result<&str, Error> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::new("empty sizing fixture value"));
    }

    let mut depth = 0usize;
    for ch in raw.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth > MAX_SIZING_FUNCTION_DEPTH {
                    return Err(Error::new(format!(
                        "sizing function nesting depth {depth} exceeds {MAX_SIZING_FUNCTION_DEPTH} in `{raw}`"
                    )));
                }
            }
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture delimiters in `{raw}`"
                    )));
                };
                depth = next_depth;
            }
            '[' | ']' => {
                return Err(Error::new(format!(
                    "unsupported sizing fixture delimiter in `{raw}`"
                )));
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(Error::new(format!(
            "unbalanced sizing fixture delimiters in `{raw}`"
        )));
    }
    Ok(raw)
}

fn parse_sizing_function(raw: &str) -> Result<Option<(&str, &str)>, Error> {
    let Some(open_index) = raw.find('(') else {
        return Ok(None);
    };
    let name = &raw[..open_index];
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
    {
        return Err(Error::new(format!(
            "malformed sizing fixture function `{raw}`"
        )));
    }

    let mut depth = 0usize;
    let mut close_index = None;
    for (offset, ch) in raw[open_index..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture delimiters in `{raw}`"
                    )));
                };
                depth = next_depth;
                if depth == 0 {
                    close_index = Some(open_index + offset);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(close_index) = close_index else {
        return Err(Error::new(format!(
            "unbalanced sizing fixture delimiters in `{raw}`"
        )));
    };
    if close_index + 1 != raw.len() {
        return Err(Error::new(format!(
            "trailing input after sizing fixture function in `{raw}`"
        )));
    }
    Ok(Some((name, &raw[open_index + 1..close_index])))
}

fn split_sizing_arguments<'a>(body: &'a str, raw: &str) -> Result<Vec<&'a str>, Error> {
    let mut arguments = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, ch) in body.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return Err(Error::new(format!(
                        "unbalanced sizing fixture arguments in `{raw}`"
                    )));
                };
                depth = next_depth;
            }
            ',' if depth == 0 => {
                arguments.push(body[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(Error::new(format!(
            "unbalanced sizing fixture arguments in `{raw}`"
        )));
    }
    arguments.push(body[start..].trim());
    if arguments.iter().any(|argument| argument.is_empty()) {
        return Err(Error::new(format!(
            "empty sizing fixture function argument in `{raw}`"
        )));
    }
    Ok(arguments)
}

fn parse_fit_content_argument(body: &str, raw: &str) -> Result<layout::SizingCalculation, Error> {
    let arguments = split_sizing_arguments(body, raw)?;
    let [argument] = arguments.as_slice() else {
        return Err(Error::new(format!(
            "fit-content() requires exactly one argument in `{raw}`"
        )));
    };
    parse_fixture_sizing_expression(argument)?.lower_sizing_calculation()
}

fn parse_calc_size_arguments<'a>(
    body: &'a str,
    raw: &str,
) -> Result<(&'a str, layout::CalcSizeCalculation), Error> {
    let arguments = split_sizing_arguments(body, raw)?;
    let [basis, calculation] = arguments.as_slice() else {
        return Err(Error::new(format!(
            "calc-size() requires exactly two arguments in `{raw}`"
        )));
    };
    Ok((
        basis,
        parse_fixture_sizing_expression(calculation)?.lower_calc_size_calculation()?,
    ))
}

fn parse_preferred_calc_size_basis(
    basis: &str,
    raw: &str,
) -> Result<layout::PreferredSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::PreferredSizeCalcBasis::Any),
        "100%" => Ok(layout::PreferredSizeCalcBasis::FullPercentage),
        "auto" => Ok(layout::PreferredSizeCalcBasis::Auto),
        "min-content" => Ok(layout::PreferredSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::PreferredSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::PreferredSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::PreferredSizeCalcBasis::FitContent),
        "contain" => Ok(layout::PreferredSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid preferred-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_min_calc_size_basis(basis: &str, raw: &str) -> Result<layout::MinSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::MinSizeCalcBasis::Any),
        "100%" => Ok(layout::MinSizeCalcBasis::FullPercentage),
        "auto" => Ok(layout::MinSizeCalcBasis::Auto),
        "min-content" => Ok(layout::MinSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::MinSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::MinSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::MinSizeCalcBasis::FitContent),
        "contain" => Ok(layout::MinSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid minimum-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_max_calc_size_basis(basis: &str, raw: &str) -> Result<layout::MaxSizeCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::MaxSizeCalcBasis::Any),
        "100%" => Ok(layout::MaxSizeCalcBasis::FullPercentage),
        "none" => Ok(layout::MaxSizeCalcBasis::None),
        "min-content" => Ok(layout::MaxSizeCalcBasis::MinContent),
        "max-content" => Ok(layout::MaxSizeCalcBasis::MaxContent),
        "stretch" => Ok(layout::MaxSizeCalcBasis::Stretch),
        "fit-content" => Ok(layout::MaxSizeCalcBasis::FitContent),
        "contain" => Ok(layout::MaxSizeCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid maximum-size calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

fn parse_flex_calc_size_basis(basis: &str, raw: &str) -> Result<layout::FlexBasisCalcBasis, Error> {
    match basis {
        "any" => Ok(layout::FlexBasisCalcBasis::Any),
        "100%" => Ok(layout::FlexBasisCalcBasis::FullPercentage),
        "auto" => Ok(layout::FlexBasisCalcBasis::Auto),
        "content" => Ok(layout::FlexBasisCalcBasis::Content),
        "min-content" => Ok(layout::FlexBasisCalcBasis::MinContent),
        "max-content" => Ok(layout::FlexBasisCalcBasis::MaxContent),
        "stretch" => Ok(layout::FlexBasisCalcBasis::Stretch),
        "fit-content" => Ok(layout::FlexBasisCalcBasis::FitContent),
        "contain" => Ok(layout::FlexBasisCalcBasis::Contain),
        _ => Err(Error::new(format!(
            "invalid flex-basis calc-size basis `{basis}` in `{raw}`"
        ))),
    }
}

#[derive(Debug)]
enum FixtureSizingExpression<'a> {
    Leaf(&'a str),
    Calc {
        raw: &'a str,
        body: &'a str,
    },
    Min {
        raw: &'a str,
        arguments: Vec<&'a str>,
    },
    Max {
        raw: &'a str,
        arguments: Vec<&'a str>,
    },
    Clamp {
        minimum: Option<&'a str>,
        preferred: &'a str,
        maximum: Option<&'a str>,
    },
    UnsupportedFunction {
        name: &'a str,
        raw: &'a str,
    },
}

fn parse_fixture_sizing_expression(raw: &str) -> Result<FixtureSizingExpression<'_>, Error> {
    let Some((name, body)) = parse_sizing_function(raw)? else {
        return Ok(FixtureSizingExpression::Leaf(raw));
    };

    match name {
        "calc" => Ok(FixtureSizingExpression::Calc { raw, body }),
        "min" | "max" => {
            let arguments = split_sizing_arguments(body, raw)?;
            if name == "min" {
                Ok(FixtureSizingExpression::Min { raw, arguments })
            } else {
                Ok(FixtureSizingExpression::Max { raw, arguments })
            }
        }
        "clamp" => {
            let arguments = split_sizing_arguments(body, raw)?;
            let [minimum, preferred, maximum] = arguments.as_slice() else {
                return Err(Error::new(format!(
                    "clamp() requires exactly three arguments in `{raw}`"
                )));
            };
            if *preferred == "none" {
                return Err(Error::new(format!(
                    "clamp() preferred argument cannot be omitted in `{raw}`"
                )));
            }
            let minimum = (*minimum != "none").then_some(*minimum);
            let maximum = (*maximum != "none").then_some(*maximum);
            Ok(FixtureSizingExpression::Clamp {
                minimum,
                preferred,
                maximum,
            })
        }
        _ => Ok(FixtureSizingExpression::UnsupportedFunction { name, raw }),
    }
}

impl FixtureSizingExpression<'_> {
    fn lower_sizing_calculation(self) -> Result<layout::SizingCalculation, Error> {
        match self {
            Self::Leaf(raw) => Ok(layout::SizingCalculation::value(parse_sizing_leaf_value(
                raw,
            )?)),
            Self::Calc { body, .. } => Ok(layout::SizingCalculation::value(
                parse_sizing_affine_value(body, false)?,
            )),
            Self::Min { raw, arguments } => {
                let calculations = arguments
                    .into_iter()
                    .map(|argument| {
                        parse_fixture_sizing_expression(argument)?.lower_sizing_calculation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                layout::SizingCalculation::min(calculations).map_err(|error| {
                    Error::new(format!("invalid sizing fixture function `{raw}`: {error}"))
                })
            }
            Self::Max { raw, arguments } => {
                let calculations = arguments
                    .into_iter()
                    .map(|argument| {
                        parse_fixture_sizing_expression(argument)?.lower_sizing_calculation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                layout::SizingCalculation::max(calculations).map_err(|error| {
                    Error::new(format!("invalid sizing fixture function `{raw}`: {error}"))
                })
            }
            Self::Clamp {
                minimum,
                preferred,
                maximum,
            } => Ok(layout::SizingCalculation::clamp(
                minimum
                    .map(|value| parse_fixture_sizing_expression(value)?.lower_sizing_calculation())
                    .transpose()?,
                parse_fixture_sizing_expression(preferred)?.lower_sizing_calculation()?,
                maximum
                    .map(|value| parse_fixture_sizing_expression(value)?.lower_sizing_calculation())
                    .transpose()?,
            )),
            Self::UnsupportedFunction { name, raw } => Err(Error::new(format!(
                "unsupported sizing fixture function `{name}` in `{raw}`"
            ))),
        }
    }

    fn lower_calc_size_calculation(self) -> Result<layout::CalcSizeCalculation, Error> {
        match self {
            Self::Leaf(raw) => calc_size_calculation_from_coefficients(
                parse_fixture_affine_coefficients(raw, true, true)?,
                raw,
            ),
            Self::Calc { raw, body } => calc_size_calculation_from_coefficients(
                parse_fixture_affine_coefficients(body, true, false)?,
                raw,
            ),
            Self::Min { raw, arguments } => {
                let calculations = arguments
                    .into_iter()
                    .map(|argument| {
                        parse_fixture_sizing_expression(argument)?.lower_calc_size_calculation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                layout::CalcSizeCalculation::min(calculations).map_err(|error| {
                    Error::new(format!(
                        "invalid calc-size fixture function `{raw}`: {error}"
                    ))
                })
            }
            Self::Max { raw, arguments } => {
                let calculations = arguments
                    .into_iter()
                    .map(|argument| {
                        parse_fixture_sizing_expression(argument)?.lower_calc_size_calculation()
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                layout::CalcSizeCalculation::max(calculations).map_err(|error| {
                    Error::new(format!(
                        "invalid calc-size fixture function `{raw}`: {error}"
                    ))
                })
            }
            Self::Clamp {
                minimum,
                preferred,
                maximum,
            } => Ok(layout::CalcSizeCalculation::clamp(
                minimum
                    .map(|value| {
                        parse_fixture_sizing_expression(value)?.lower_calc_size_calculation()
                    })
                    .transpose()?,
                parse_fixture_sizing_expression(preferred)?.lower_calc_size_calculation()?,
                maximum
                    .map(|value| {
                        parse_fixture_sizing_expression(value)?.lower_calc_size_calculation()
                    })
                    .transpose()?,
            )),
            Self::UnsupportedFunction { name, raw } => Err(Error::new(format!(
                "unsupported calc-size fixture function `{name}` in `{raw}`"
            ))),
        }
    }
}

fn parse_sizing_leaf_value(raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    let [atom] = tokens.as_slice() else {
        return Err(Error::new(format!(
            "unsupported sizing fixture leaf `{raw}`"
        )));
    };
    let coefficients = parse_fixture_affine_atom(atom, false, true, raw)?;
    layout::LengthPercentageOf::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
    )
    .map_err(|error| Error::new(format!("invalid sizing fixture `{raw}`: {error}")))
}

fn parse_sizing_affine_value(
    raw: &str,
    allow_unitless: bool,
) -> Result<layout::LengthPercentageOf, Error> {
    let coefficients = parse_fixture_affine_coefficients(raw, false, allow_unitless)?;
    layout::LengthPercentageOf::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
    )
    .map_err(|error| Error::new(format!("invalid sizing fixture `{raw}`: {error}")))
}

fn calc_size_calculation_from_coefficients(
    coefficients: FixtureSizingCoefficients,
    raw: &str,
) -> Result<layout::CalcSizeCalculation, Error> {
    layout::CalcSizeCalculation::from_coefficients(
        coefficients.absolute_px,
        coefficients.percent_fraction,
        coefficients.size_fraction,
    )
    .map_err(|error| Error::new(format!("invalid calc-size fixture `{raw}`: {error}")))
}

fn parse_fixture_affine_coefficients(
    raw: &str,
    allow_size: bool,
    allow_unitless: bool,
) -> Result<FixtureSizingCoefficients, Error> {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return Err(Error::new(format!("empty affine sizing fixture `{raw}`")));
    }

    let mut coefficients = FixtureSizingCoefficients {
        absolute_px: 0.0,
        percent_fraction: 0.0,
        size_fraction: 0.0,
    };
    let mut start = 0usize;
    let mut sign = 1.0;
    loop {
        let end = tokens[start..]
            .iter()
            .position(|token| *token == "+" || *token == "-")
            .map_or(tokens.len(), |offset| start + offset);
        if end == start {
            return Err(Error::new(format!("missing affine sizing term in `{raw}`")));
        }
        let term = parse_fixture_affine_term(&tokens[start..end], allow_size, allow_unitless, raw)?;
        coefficients.absolute_px += term.absolute_px * sign;
        coefficients.percent_fraction += term.percent_fraction * sign;
        coefficients.size_fraction += term.size_fraction * sign;

        if end == tokens.len() {
            break;
        }
        sign = if tokens[end] == "+" { 1.0 } else { -1.0 };
        start = end + 1;
        if start == tokens.len() {
            return Err(Error::new(format!("missing affine sizing term in `{raw}`")));
        }
    }
    Ok(coefficients)
}

fn parse_fixture_affine_term(
    tokens: &[&str],
    allow_size: bool,
    allow_unitless: bool,
    raw: &str,
) -> Result<FixtureSizingCoefficients, Error> {
    match tokens {
        [atom] => parse_fixture_affine_atom(atom, allow_size, allow_unitless, raw),
        [left, "*", right] if allow_size => {
            let size_factor = match (*left, *right) {
                ("size", factor) => parse_number(factor)?,
                (factor, "size") => parse_number(factor)?,
                _ => {
                    return Err(Error::new(format!(
                        "unsupported affine sizing product in `{raw}`"
                    )));
                }
            };
            Ok(FixtureSizingCoefficients {
                absolute_px: 0.0,
                percent_fraction: 0.0,
                size_fraction: size_factor,
            })
        }
        _ => Err(Error::new(format!(
            "unsupported affine sizing term in `{raw}`"
        ))),
    }
}

fn parse_fixture_affine_atom(
    atom: &str,
    allow_size: bool,
    allow_unitless: bool,
    raw: &str,
) -> Result<FixtureSizingCoefficients, Error> {
    let mut coefficients = FixtureSizingCoefficients {
        absolute_px: 0.0,
        percent_fraction: 0.0,
        size_fraction: 0.0,
    };
    if let Some(px) = atom.strip_suffix("px") {
        coefficients.absolute_px = parse_number(px)?;
        return Ok(coefficients);
    }
    if let Some(percent) = atom.strip_suffix('%') {
        coefficients.percent_fraction = parse_number(percent)? / 100.0;
        return Ok(coefficients);
    }
    if allow_size {
        if atom == "size" {
            coefficients.size_fraction = 1.0;
            return Ok(coefficients);
        }
        if let Some(factor) = atom.strip_suffix("*size") {
            coefficients.size_fraction = parse_number(factor)?;
            return Ok(coefficients);
        }
        if let Some(factor) = atom.strip_prefix("size*") {
            coefficients.size_fraction = parse_number(factor)?;
            return Ok(coefficients);
        }
    }
    if allow_unitless {
        coefficients.absolute_px = parse_number(atom)?;
        return Ok(coefficients);
    }
    Err(Error::new(format!(
        "unsupported affine sizing term `{atom}` in `{raw}`"
    )))
}

fn length_percentage_px(value: Scalar, raw: &str) -> Result<layout::LengthPercentageOf, Error> {
    layout::LengthPercentageOf::px(value)
        .map_err(|error| Error::new(format!("invalid length `{raw}`: {error}")))
}

fn length_percentage_percent(
    value: Scalar,
    raw: &str,
) -> Result<layout::LengthPercentageOf, Error> {
    layout::LengthPercentageOf::from_percent_fraction(value)
        .map_err(|error| Error::new(format!("invalid length `{raw}`: {error}")))
}

fn length_px(value: Scalar, raw: &str) -> Result<layout::Length, Error> {
    Ok(layout::Length::value(length_percentage_px(value, raw)?))
}

fn length_percent(value: Scalar, raw: &str) -> Result<layout::Length, Error> {
    Ok(layout::Length::value(length_percentage_percent(
        value, raw,
    )?))
}

#[cfg(test)]
fn preferred_size_px(value: Scalar) -> layout::PreferredSize {
    layout::PreferredSize::value(
        layout::LengthPercentageOf::px(value).expect("finite test dimension px"),
    )
}

#[cfg(test)]
fn min_track_px(value: Scalar) -> layout::MinTrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test min track px")
        .into()
}

#[cfg(test)]
fn max_track_px(value: Scalar) -> layout::MaxTrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test max track px")
        .into()
}

#[cfg(test)]
fn track_px(value: Scalar) -> layout::TrackSizing {
    layout::LengthPercentageOf::px(value)
        .expect("finite test track px")
        .into()
}

#[cfg(test)]
fn track_component_px(value: Scalar) -> layout::TrackComponent {
    layout::TrackComponent::Track(track_px(value))
}

fn parse_track_component_list(raw: &str) -> Result<Vec<layout::TrackComponent>, Error> {
    parse_track_component_list_with_calc(raw)
}

fn parse_track_component_list_with_calc(raw: &str) -> Result<Vec<layout::TrackComponent>, Error> {
    if raw.trim_start().starts_with("subgrid") {
        return Ok(vec![parse_subgrid_track_component(raw)?]);
    }
    split_top_level_whitespace(raw)
        .into_iter()
        .map(|part| parse_track_component_with_calc(&part))
        .collect()
}

fn parse_track_component(raw: &str) -> Result<layout::TrackComponent, Error> {
    parse_track_component_with_calc(raw)
}

fn parse_track_component_with_calc(raw: &str) -> Result<layout::TrackComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, tracks) = split_once_top_level_comma(body)?;
        let repeat = match count.trim() {
            "auto-fill" => layout::TrackRepetition::auto_fill_components(
                parse_track_component_list_with_calc(tracks)?,
            ),
            "auto-fit" => layout::TrackRepetition::auto_fit_components(
                parse_track_component_list_with_calc(tracks)?,
            ),
            raw => layout::TrackRepetition::count_components(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
                parse_track_component_list_with_calc(tracks)?,
            ),
        };
        let repeat =
            repeat.map_err(|error| Error::new(format!("invalid track repeat: {error}")))?;
        return Ok(layout::TrackComponent::Repeat(repeat));
    }
    if raw.starts_with('[') {
        return Ok(layout::TrackComponent::LineNames(parse_subgrid_line_names(
            raw,
        )?));
    }
    Ok(layout::TrackComponent::Track(parse_track_sizing_with_calc(
        raw,
    )?))
}

fn parse_subgrid_track_component(raw: &str) -> Result<layout::TrackComponent, Error> {
    let mut parts = split_top_level_whitespace(raw);
    if parts.first().map(String::as_str) != Some("subgrid") {
        return Err(Error::new(format!("invalid subgrid track list `{raw}`")));
    }
    let name_components = parts
        .drain(1..)
        .map(|part| parse_subgrid_line_name_component(&part))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layout::TrackComponent::Subgrid(layout::SubgridTrack {
        name_components,
    }))
}

fn parse_subgrid_line_name_component(raw: &str) -> Result<layout::SubgridLineNameComponent, Error> {
    if let Some(body) = function_body(raw, "repeat") {
        let (count, names) = split_once_top_level_comma(body)?;
        let line_name_sets = split_top_level_whitespace(names)
            .into_iter()
            .map(|part| parse_subgrid_line_names(&part))
            .collect::<Result<Vec<_>, _>>()?;
        let count = match count.trim() {
            "auto-fill" => layout::SubgridLineNameRepeatCount::AutoFill,
            raw => layout::SubgridLineNameRepeatCount::Count(
                raw.parse()
                    .map_err(|_| Error::new(format!("invalid repeat count `{raw}`")))?,
            ),
        };
        return Ok(layout::SubgridLineNameComponent::Repeat {
            count,
            line_name_sets,
        });
    }
    Ok(layout::SubgridLineNameComponent::LineNames(
        parse_subgrid_line_names(raw)?,
    ))
}

fn parse_subgrid_line_names(raw: &str) -> Result<Vec<String>, Error> {
    let body = raw
        .strip_prefix('[')
        .and_then(|raw| raw.strip_suffix(']'))
        .ok_or_else(|| Error::new(format!("invalid subgrid line-name list `{raw}`")))?;
    body.split_whitespace()
        .map(|name| parse_custom_ident(name).map(ToOwned::to_owned))
        .collect::<Result<Vec<_>, _>>()
}

fn parse_track_sizing_with_calc(raw: &str) -> Result<layout::TrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    if let Some(("minmax", body)) = parse_sizing_function(raw)? {
        let arguments = split_sizing_arguments(body, raw)?;
        let [min, max] = arguments.as_slice() else {
            return Err(Error::new(format!(
                "minmax() requires exactly two arguments in `{raw}`"
            )));
        };
        return Ok(layout::TrackSizing::minmax(
            parse_min_track_sizing_inner(min)?,
            parse_max_track_sizing_inner(max)?,
        ));
    }
    if let Some(("fit-content", body)) = parse_sizing_function(raw)? {
        return Ok(layout::TrackSizing::fit_content(
            parse_fit_content_argument(body, raw)?,
        ));
    }
    match raw {
        "auto" => Ok(layout::TrackSizing::AUTO),
        "min-content" => Ok(layout::TrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::TrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => Ok(layout::TrackSizing::flex(parse_track_flex(raw)?)),
        _ => Ok(layout::TrackSizing::calculation(
            parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
        )),
    }
}

fn parse_min_track_sizing_with_calc(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    parse_min_track_sizing_inner(raw)
}

fn parse_min_track_sizing_inner(raw: &str) -> Result<layout::MinTrackSizing, Error> {
    match raw {
        "auto" => Ok(layout::MinTrackSizing::AUTO),
        "min-content" => Ok(layout::MinTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MinTrackSizing::MAX_CONTENT),
        _ => Ok(layout::MinTrackSizing::Calculation(
            parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
        )),
    }
}

fn parse_max_track_sizing_with_calc(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    let raw = checked_sizing_fixture(raw)?;
    parse_max_track_sizing_inner(raw)
}

fn parse_max_track_sizing_inner(raw: &str) -> Result<layout::MaxTrackSizing, Error> {
    if let Some(("fit-content", body)) = parse_sizing_function(raw)? {
        return Ok(layout::MaxTrackSizing::fit_content(
            parse_fit_content_argument(body, raw)?,
        ));
    }
    match raw {
        "auto" => Ok(layout::MaxTrackSizing::AUTO),
        "min-content" => Ok(layout::MaxTrackSizing::MIN_CONTENT),
        "max-content" => Ok(layout::MaxTrackSizing::MAX_CONTENT),
        _ if raw.ends_with("fr") => Ok(layout::MaxTrackSizing::flex(parse_track_flex(raw)?)),
        _ => Ok(layout::MaxTrackSizing::Calculation(
            parse_fixture_sizing_expression(raw)?.lower_sizing_calculation()?,
        )),
    }
}

fn parse_track_flex(raw: &str) -> Result<layout::TrackFlexFactor, Error> {
    let value = raw
        .strip_suffix("fr")
        .ok_or_else(|| Error::new(format!("invalid track flex `{raw}`")))?;
    layout::TrackFlexFactor::try_new(parse_number(value)?)
        .map_err(|error| Error::new(format!("invalid track flex `{raw}`: {error}")))
}

fn function_body<'a>(raw: &'a str, name: &str) -> Option<&'a str> {
    raw.strip_prefix(name)
        .and_then(|raw| raw.strip_prefix('('))
        .and_then(|raw| raw.strip_suffix(')'))
}

fn split_once_top_level_comma(raw: &str) -> Result<(&str, &str), Error> {
    let mut depth = 0usize;
    for (index, ch) in raw.char_indices() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Ok((&raw[..index], &raw[index + 1..])),
            _ => {}
        }
    }
    Err(Error::new(format!("expected top-level comma in `{raw}`")))
}

fn split_top_level_whitespace(raw: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, ch) in raw.char_indices() {
        match ch {
            '(' | '[' => {
                depth += 1;
                start.get_or_insert(index);
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                start.get_or_insert(index);
            }
            ch if ch.is_whitespace() && depth == 0 => {
                if let Some(start_index) = start.take() {
                    parts.push(raw[start_index..index].to_string());
                }
            }
            _ => {
                start.get_or_insert(index);
            }
        }
    }
    if let Some(start_index) = start {
        parts.push(raw[start_index..].to_string());
    }
    parts
}

fn parse_grid_line(raw: &str) -> Result<Option<isize>, Error> {
    if raw == "auto" || raw.starts_with("span ") {
        return Ok(None);
    }
    parse_grid_line_number(raw).map(Some)
}

fn parse_grid_line_or_span(raw: &str, span: &mut Option<usize>) -> Result<Option<isize>, Error> {
    if raw == "auto" {
        return Ok(None);
    }
    if let Some(raw_span) = raw.strip_prefix("span ") {
        *span = Some(parse_nonnegative_line(raw_span)?);
        return Ok(None);
    }
    parse_grid_line_number(raw).map(Some)
}

fn parse_grid_line_number(raw: &str) -> Result<isize, Error> {
    raw.parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))
}

fn parse_nonnegative_line(raw: &str) -> Result<usize, Error> {
    let value: i64 = raw
        .parse()
        .map_err(|_| Error::new(format!("invalid grid line `{raw}`")))?;
    if value < 0 {
        return Err(Error::new(format!(
            "negative grid line `{raw}` is not supported by Surgeist layout yet"
        )));
    }
    usize::try_from(value).map_err(|_| Error::new(format!("grid line `{raw}` is too large")))
}

fn collect_files_with_extension(
    dir: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), Error> {
    for entry in std::fs::read_dir(dir).map_err(|source| Error::new(source.to_string()))? {
        let entry = entry.map_err(|source| Error::new(source.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
        } else if path
            .extension()
            .and_then(|actual| actual.to_str())
            .is_some_and(|actual| actual == extension)
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// Authored source/style facts for reader tests, never measured widths.
    fn native_test_golden(parts: &[(&str, f32, f64)], boundaries: bool) -> Golden {
        native_test_golden_with_word_spacing(parts, boundaries, 0.0)
    }

    fn native_test_golden_with_word_spacing(
        parts: &[(&str, f32, f64)],
        boundaries: bool,
        word_spacing: f64,
    ) -> Golden {
        use serde_json::json;
        let style = |size: f32, spacing: f64| json!({"fontSize":size,"lineHeight":10.5,"whitespace":"preserve","wrap":"word","letterSpacing":spacing,"wordSpacing":word_spacing,"color":[0.0,0.0,0.0,1.0],"locale":null});
        let mut origins = Vec::new();
        let mut controls = Vec::new();
        let mut order = Vec::new();
        let mut children = String::new();
        let mut at = 0;
        let mut child = 0;
        for (index, (source, size, spacing)) in parts.iter().enumerate() {
            if boundaries && index == 1 {
                controls.push(json!({"child":child,"at":at,"kind":"inline-boundary-start"}));
                order.push(json!({"kind":"control","index":0,"affinity":"before"}));
                children.push_str("<inline-boundary kind=\"start\"/>");
                child += 1;
            }
            let units: Vec<_> = source.encode_utf16().collect();
            origins.push(json!({"child":child,"units":units,"start":at,"end":at+units.len(),"style":style(*size,*spacing)}));
            order.push(json!({"kind":"origin","index":index,"affinity":"before"}));
            at += units.len();
            child += 1;
            children.push_str(&format!(
                "<text layout-input=\"prepared-text\" occurrence=\"{index}\"/>"
            ));
            if boundaries && index == 1 {
                controls.push(json!({"child":child,"at":at,"kind":"inline-boundary-end"}));
                order.push(json!({"kind":"control","index":1,"affinity":"before"}));
                children.push_str("<inline-boundary kind=\"end\"/>");
                child += 1;
            }
        }
        let connections:Vec<_>=(1..parts.len()).map(|index|json!({"preceding":index-1,"following":index,"kind":"continuous","precedingWrap":"word","followingWrap":"word","ancestorWrap":"word"})).collect();
        let capture = json!({"font":{"id":pinned_font::ID,"sha256":pinned_font::SHA256,"faceIndex":0,"family":"SurgeistParityAhem","style":"normal","weight":400,"stretch":"normal","features":"normal","variations":"normal","synthesis":"none","loaded":true},
            "direction":"ltr","writingMode":"horizontal-tb","baseStyle":style(10.0,0.0),"origins":origins,"connections":connections,"atomics":[],"controls":controls,"order":order});
        let capture = capture
            .to_string()
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;");
        Golden::parse(&format!("<test name=\"native-reader\" measurement-version=\"2\" use-rounding=\"true\"><viewport width=\"100\" height=\"max-content\"/><input><div display=\"block\" layout-ready-inline-root=\"true\"><prepared-paragraph capture=\"{capture}\"/>{children}</div></input><expectations><node/></expectations></test>")).unwrap()
    }
    #[test]
    fn prepared_empty_transparent_controls_round_trip_without_fabricated_origin() {
        // Exact producer-shaped source channel for the genuinely empty marked
        // span: two controls, no text occurrence, and no observed metrics.
        let style = serde_json::json!({"fontSize":16.0,"lineHeight":20.0,"whitespace":"collapse","wrap":"word","letterSpacing":0.0,"wordSpacing":0.0,"color":[0.0,0.0,0.0,1.0],"locale":null});
        let capture = serde_json::json!({"font":{"id":pinned_font::ID,"sha256":pinned_font::SHA256,"faceIndex":0,"family":"SurgeistParityAhem","style":"normal","weight":400,"stretch":"normal","features":"normal","variations":"normal","synthesis":"none","loaded":true},"direction":"ltr","writingMode":"horizontal-tb","baseStyle":style,"origins":[],"connections":[],"atomics":[],"controls":[{"child":0,"at":0,"kind":"inline-boundary-start"},{"child":1,"at":0,"kind":"inline-boundary-end"}],"order":[{"kind":"control","index":0,"affinity":"before"},{"kind":"control","index":1,"affinity":"before"}]});
        let capture = capture
            .to_string()
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;");
        let golden=Golden::parse(&format!("<test name=\"native-empty-transparent\" measurement-version=\"2\" use-rounding=\"false\"><viewport width=\"100\" height=\"max-content\"/><input><div display=\"block\" layout-ready-inline-root=\"true\"><prepared-paragraph capture=\"{capture}\"/><inline-boundary kind=\"start\"/><inline-boundary kind=\"end\"/></div></input><expectations><node/></expectations></test>"))
            .expect("authenticated native empty control topology must parse");
        let captured = golden.root.prepared.as_ref().unwrap();
        assert!(captured.origins.is_empty());
        assert_eq!(captured.controls.len(), 2);
        let mut wrong_role = golden.root.clone();
        wrong_role.children[0].inline_boundary = Some(layout::InlineBoundaryKind::End);
        assert_eq!(
            validate_fri06_c08r_explicit_input(&wrong_role, true)
                .unwrap_err()
                .to_string(),
            "captured control role does not match its exact native leaf"
        );
        let mut unbound = golden.root.clone();
        unbound.children[0].prepared_control = None;
        assert_eq!(
            validate_fri06_c08r_explicit_input(&unbound, true)
                .unwrap_err()
                .to_string(),
            "native inline boundary has no captured control occurrence"
        );
        let tree = TestTree::from_golden(&golden.root).unwrap();
        let batch = native_tree_compute(&tree, fri06_c06_inline_request());
        assert!(batch.unrounded_inline_fragments().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        let placed = batch.placed_text()[0].layout();
        assert!(placed.lines().is_empty());
        let receipts = placed.nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(receipts.len(), 1);
        let line = receipts[0].line.finalized_line();
        assert!(!line.has_in_flow_content());
        assert_eq!(line.fragments().len(), 2);
        for row in &line.selected_line().request().controls {
            assert_eq!(
                row.boundary,
                Some(surgeist_text::InlineBoundaryAllocation::ZERO)
            );
            let strut = row.strut.unwrap();
            assert_eq!((strut.baseline, strut.line_extent), (0.0, 0.0));
        }
        for fragment in line.fragments() {
            let surgeist_text::FinalizedFragmentOwner::Control(control) = fragment.view().owner()
            else {
                panic!("the genuine nonvisual control keeps its actual owner")
            };
            assert!(
                tree.paragraphs[0]
                    .controls
                    .iter()
                    .any(|row| row.occurrence == control)
            );
            assert!(fragment.view().origin().is_none());
        }
    }

    fn generated_native_golden(name: &str) -> Golden {
        Golden::parse_file(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/xml/block")
                .join(format!("{name}.xml")),
        )
        .unwrap()
    }

    fn generated_native_batch(golden: &Golden) -> (TestTree, layout::CompletedLayoutBatch<usize>) {
        let tree = TestTree::from_golden(&golden.root).unwrap();
        let request = root_request(
            layout::Size::new(
                to_layout_available(golden.viewport.width),
                to_layout_available(golden.viewport.height),
            ),
            golden.viewport.root_context,
        )
        .unwrap();
        let batch = native_tree_compute(&tree, request);
        (tree, batch)
    }

    #[test]
    fn native_browser_boundaries_keep_control_custody_without_observed_children() {
        let golden = generated_native_golden("prepared_text_continuity_bidi__border_box_ltr");
        let (tree, batch) = generated_native_batch(&golden);
        let controls = tree
            .paragraphs
            .iter()
            .flat_map(|paragraph| {
                paragraph
                    .controls
                    .iter()
                    .map(|row| (row.node, &row.occurrence))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            controls
                .iter()
                .map(|(_, control)| control.view().kind().clone())
                .collect::<Vec<_>>(),
            vec![
                surgeist_text::ParagraphControlKind::InlineBoundaryStart,
                surgeist_text::ParagraphControlKind::InlineBoundaryEnd
            ]
        );
        for (node, control) in controls {
            let input = tree.nodes[node].layout_input.as_inline_boundary().unwrap();
            assert_eq!(input.edge(), layout::InlineBoundaryEdge::ZERO);
            assert_eq!(input.metrics().baseline(), 0.0);
            assert_eq!(input.metrics().line_extent(), 0.0);
            assert!(
                batch.unrounded_inline_fragments().iter().any(|entry| {
                    entry.owner() == &layout::InlineFragmentOwner::Node(node)
                        && matches!(entry.fragment().finalized_fragment().view().owner(),
                    surgeist_text::FinalizedFragmentOwner::Control(actual) if actual == *control)
                }),
                "a transparent semantic control must retain its actual finalized owner"
            );
        }
        // Chrome observes the three source occurrences, not rectangles for the
        // two metricless boundary controls. Their custody was checked above.
        assert_surgeist_matches(&golden).unwrap();
    }

    #[test]
    fn native_range_observation_uses_actual_font_band_instead_of_authored_line_strut() {
        // Authored input, not measured/source-substituted data: the maintained
        // helper captures original X, pinned Ahem10, LH10.5, no tracking/word spacing,
        // no controls/atomics, one genuine source origin and an ordinary inline root.
        let golden = native_test_golden(&[("X", 10.0, 0.0)], false);
        let (tree, batch) = generated_native_batch(&golden);
        let [binding] = tree.paragraphs.as_slice() else {
            panic!("one actual prepared paragraph")
        };
        let [bound_origin] = binding.origins.as_slice() else {
            panic!("one actual original")
        };
        let origin = bound_origin.origin.clone();
        assert_eq!(origin.view().source().text().code_units(), [0x0058]);
        assert_eq!(origin.view().local_range(), surgeist_text::Range::new(0, 1));
        assert_eq!(
            origin.view().aggregate_range(),
            surgeist_text::Range::new(0, 1)
        );

        let placed = batch
            .placed_text()
            .iter()
            .find(|output| output.container() == binding.container)
            .expect("the same computed batch retains actual placed Text")
            .layout();
        assert_eq!(
            placed.analyzed_source().identity(),
            origin.prepared_text().analyzed_source().identity()
        );
        assert_eq!(placed.lines().len(), 1);
        let baseline = placed.lines()[0].baseline();
        let actual = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .as_ref()
                    == Some(&origin)
            })
            .cloned()
            .collect::<Vec<_>>();
        let [entry] = actual.as_slice() else {
            panic!("one actual source-owned X fragment")
        };
        let fragment = entry.fragment();
        let finalized = fragment.finalized_fragment().finalized_line();
        assert_eq!(finalized.prepared_text(), origin.prepared_text());
        assert_eq!(fragment.line_index(), 0);

        // Qualify the real unchanged resource/instance independently before either
        // Range observation. Pinned hhea800/-200 at10px gives ascent8/descent-2,
        // whereas authored extra leading0.5 distributes0.25 on each strut side.
        let runs = finalized.run_views().collect::<Vec<_>>();
        let [run] = runs.as_slice() else {
            panic!("one real pinned-font run")
        };
        assert_eq!(run.face().bytes(), pinned_font::BYTES);
        assert_eq!(run.face().index(), 0);
        assert_eq!(run.font_size(), 10.0);
        let instance = run.effective_instance().unwrap();
        let metrics = instance
            .metrics(instance.face().catalogue_identity())
            .unwrap();
        assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
        assert_eq!(
            run.glyphs()
                .map(|glyph| (glyph.range(), glyph.id(), glyph.advance(), glyph.pen()))
                .collect::<Vec<_>>(),
            [(surgeist_text::Range::new(0, 1), 58, 10.0, 0.0)]
        );
        let strut = fragment
            .rect()
            .expect("genuine source-fragment strut control");
        assert_eq!(strut.size().height, 10.5);
        assert_eq!(f64::from(strut.origin().y), baseline - 8.25);
        assert_eq!((strut.origin().x, strut.size().width), (0.0, 10.0));

        // Independent oracle from original X's admitted font metrics and actual
        // placement: top=baseline-8, height8+2=10, x0/advance10. The canonical
        // public query is an ADDITIONAL consistency/custody control, never the
        // source of these expected numbers and never a substitute Source string.
        let query = placed
            .source_range_fragments(&origin, surgeist_text::Range::new(0, 1))
            .expect("canonical query over the very same real placed result");
        assert_eq!(query.origin(), &origin);
        assert_eq!(query.local_range(), surgeist_text::Range::new(0, 1));
        let units = query.units().collect::<Vec<_>>();
        let [unit] = units.as_slice() else {
            panic!("one real original X EGC")
        };
        assert_eq!(unit.finalized_line(), finalized);
        let pieces = unit.pieces().collect::<Vec<_>>();
        let [surgeist_text::SourceRangeGeometry::Line(piece)] = pieces.as_slice() else {
            panic!("actual ordinary font-metric Range contribution")
        };
        assert_eq!(
            piece.effective_instance().face().bytes(),
            pinned_font::BYTES
        );
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        assert_eq!(piece.baseline(), baseline);
        assert_eq!(
            piece.rect(),
            surgeist_text::Rect::new(0.0, baseline - 8.0, 10.0, 10.0)
        );

        // Existing-callable consumer under correction. Its current fragment-based
        // reconstruction instead returns the LH10.5 strut rectangle checked above.
        // Keep strict browser XML and its inline-only comparison policy unchanged.
        let observed = native_range_rectangles("canonical-native-font-band", &actual).unwrap();
        let [observation] = observed.as_slice() else {
            panic!("one source/line Range observation")
        };
        assert_eq!(observation.origin, origin);
        assert_eq!(observation.line, finalized);
        assert_eq!(observation.source, surgeist_text::Range::new(0, 1));
        assert_eq!(observation.line_index, 0);
        assert_eq!(
            observation.rect.size().height,
            10.0,
            "native Range observation must expose the actual font band, not line-height leading"
        );
        assert_eq!(f64::from(observation.rect.origin().y), baseline - 8.0);
        assert_eq!(
            (observation.rect.origin().x, observation.rect.size().width),
            (0.0, 10.0)
        );
    }

    #[test]
    fn native_browser_range_rectangles_cover_actual_origin_intersections() {
        let golden = generated_native_golden("fri06_inline_mixed_text_atomic_wrap__border_box_ltr");
        let (_, batch) = generated_native_batch(&golden);
        let origin = golden.root.children[0].prepared_origin.as_ref().unwrap();
        let mut covered = Vec::new();
        let mut line = None;
        for entry in batch.unrounded_inline_fragments() {
            let issued = entry.fragment().finalized_fragment();
            let view = issued.view();
            if view.origin().as_ref() != Some(origin) {
                continue;
            }
            let source = view.aggregate_range().unwrap();
            let rect = entry.fragment().rect().unwrap();
            // Original "text" is four covered Ahem letters at16px, one em each.
            // This qualifies every authentic source intersection independently
            // of whether Text chooses one or several fragment rows.
            assert_eq!(rect.origin().x, 16.0 * source.start as f32);
            assert_eq!(rect.size().width, 16.0 * source.len() as f32);
            if let Some(previous) = &line {
                assert_eq!(previous, &issued.finalized_line());
            } else {
                line = Some(issued.finalized_line());
            }
            covered.push((source.start, source.end));
        }
        covered.sort_unstable();
        let mut end = 0;
        for (start, next) in covered {
            assert_eq!(start, end);
            end = next;
        }
        assert_eq!(end, 4);
        // One browser Range rectangle may cover several contiguous, genuinely
        // associated Text intersections on the same finalized line.
        assert_surgeist_matches(&golden).unwrap();
    }

    #[test]
    fn native_browser_range_normalization_rejects_wrong_owner_extent_and_source_gaps() {
        let golden = generated_native_golden("fri06_inline_mixed_text_atomic_wrap__border_box_ltr");
        let (_, batch) = generated_native_batch(&golden);
        let origin = golden.root.children[0].prepared_origin.as_ref().unwrap();
        let actual = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .as_ref()
                    == Some(origin)
            })
            .cloned()
            .collect::<Vec<_>>();
        let expected = golden.expectations.children[0]
            .range_rectangles
            .as_ref()
            .unwrap();
        compare_range_rectangle_expectations_with_translation(
            "actual-range",
            &actual,
            expected,
            |_| Ok(0.0),
        )
        .unwrap();
        for field in ["owner", "extent"] {
            let mut wrong = expected.clone();
            if field == "owner" {
                wrong[0].source_segment_id += 1;
            } else {
                wrong[0].advance -= 16.0;
            }
            assert!(
                compare_range_rectangle_expectations_with_translation(
                    "wrong-range",
                    &actual,
                    &wrong,
                    |_| Ok(0.0)
                )
                .is_err()
            );
        }
        let first = actual
            .iter()
            .find(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .aggregate_range()
                    == Some(surgeist_text::Range::new(0, 1))
            })
            .unwrap();
        let third = actual
            .iter()
            .find(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .aggregate_range()
                    == Some(surgeist_text::Range::new(2, 3))
            })
            .unwrap();
        // Real nonadjacent source cells cannot be replaced by their48px hull.
        // The canonical finalized lender independently exposes the omitted rows.
        let missing = vec![first.clone(), third.clone()];
        let mut hull = expected.clone();
        hull[0].advance = 48.0;
        assert!(
            compare_range_rectangle_expectations_with_translation(
                "missing-source",
                &missing,
                &hull,
                |_| Ok(0.0)
            )
            .unwrap_err()
            .to_string()
            .contains("source coverage")
        );
        let one = |entry: &layout::InlineFragmentOutputEntry<usize>| NativeRangeRectangle {
            origin: origin.clone(),
            line: entry.fragment().finalized_fragment().finalized_line(),
            source: entry
                .fragment()
                .finalized_fragment()
                .view()
                .aggregate_range()
                .unwrap(),
            source_segment_id: 0,
            line_index: entry.fragment().line_index(),
            rect: entry.fragment().rect().unwrap(),
        };
        let mut previous = one(first);
        assert!(!merge_native_range_rectangles(&mut previous, &one(third)).unwrap());
        assert_eq!(previous.rect.size().width, 16.0);
    }

    #[test]
    fn native_browser_range_normalization_keeps_actual_lines_and_zero_width_controls() {
        let golden = generated_native_golden("prepared_text_fractional_fit__border_box_ltr");
        let (_, batch) = generated_native_batch(&golden);
        let origin = golden.root.children[0].prepared_origin.as_ref().unwrap();
        let actual = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .as_ref()
                    == Some(origin)
            })
            .cloned()
            .collect::<Vec<_>>();
        let rectangles = native_range_rectangles("actual-lines", &actual).unwrap();
        assert_eq!(rectangles.len(), 2);
        assert_eq!(rectangles[0].origin, rectangles[1].origin);
        assert_ne!(rectangles[0].line, rectangles[1].line);
        assert_ne!(rectangles[0].rect.origin().y, rectangles[1].rect.origin().y);
        let mut missing_band = golden.expectations.children[0]
            .range_rectangles
            .clone()
            .unwrap();
        missing_band.truncate(1);
        assert!(
            compare_range_rectangle_expectations_with_translation(
                "different-bands",
                &actual,
                &missing_band,
                |_| Ok(0.0)
            )
            .is_err()
        );

        let golden = generated_native_golden("prepared_text_continuity_bidi__border_box_ltr");
        let (_, batch) = generated_native_batch(&golden);
        let origin = golden.root.children[2].prepared_origin.as_ref().unwrap();
        let actual = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .as_ref()
                    == Some(origin)
            })
            .cloned()
            .collect::<Vec<_>>();
        let rectangles = native_range_rectangles("directional-controls", &actual).unwrap();
        assert_eq!(
            rectangles
                .iter()
                .map(|row| row.rect.size().width)
                .collect::<Vec<_>>(),
            vec![0.0, 32.0, 0.0]
        );
        // Actual RLI and PDI source positions keep separate zero-width Range
        // observations around the two real16px Ahem glyphs.
        compare_range_rectangle_expectations_with_translation(
            "directional-controls",
            &actual,
            golden.expectations.children[1]
                .range_rectangles
                .as_ref()
                .unwrap(),
            |_| Ok(0.0),
        )
        .unwrap();
    }

    #[test]
    fn native_browser_range_normalization_keeps_preserved_zero_advance_space() {
        // Ahem's real space advances one em. Authored word spacing cancels
        // that cell, while Preserve still emits the actual space glyph. A lone
        // unit has no character boundary on which tracking could cancel it.
        let golden = native_test_golden_with_word_spacing(&[(" ", 16.0, 0.0)], false, -16.0);
        let (_, batch) = generated_native_batch(&golden);
        let actual = batch.unrounded_inline_fragments();
        assert_eq!(actual.len(), 1);
        let issued = actual[0].fragment().finalized_fragment();
        let line = issued.finalized_line();
        let glyphs = line
            .run_views()
            .flat_map(|run| {
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance(), glyph.range()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, vec![(3, 16.0, surgeist_text::Range::new(0, 1))]);
        assert_eq!(actual[0].fragment().rect().unwrap().size().width, 0.0);
        assert!(!unpainted_source_space(issued, &line));
        let rectangles = native_range_rectangles("preserved-zero-space", actual).unwrap();
        assert_eq!(rectangles.len(), 1);
        assert_eq!(rectangles[0].source, surgeist_text::Range::new(0, 1));
        assert_eq!(rectangles[0].rect.size().width, 0.0);
    }

    #[test]
    fn native_rtl_pdi_observation_difference_retains_source_line_and_placement_custody() {
        // This records an executed observation difference, not a parity pass or
        // unsupported production input. The strict parity runner still fails
        // these two unchanged XML records. Text's owning test qualifies L1/L2.
        for variant in ["border_box_rtl", "content_box_rtl"] {
            let name = format!("prepared_text_continuity_bidi__{variant}");
            let golden = generated_native_golden(&name);
            let (_, batch) = generated_native_batch(&golden);
            let origin = golden.root.children[2].prepared_origin.as_ref().unwrap();
            assert_eq!(
                origin.view().source().text().code_units(),
                [0x2067, 88, 88, 0x2069]
            );
            let prepared = origin.prepared_text();
            let pdi = prepared
                .unit_views()
                .find(|unit| unit.source_range() == surgeist_text::Range::new(4, 5))
                .unwrap();
            assert_eq!(
                pdi.level().get(),
                2,
                "actual paragraph level precedes selected L1"
            );
            let entry = batch
                .unrounded_inline_fragments()
                .iter()
                .find(|entry| {
                    let view = entry.fragment().finalized_fragment().view();
                    view.origin().as_ref() == Some(origin)
                        && view.aggregate_range() == Some(surgeist_text::Range::new(4, 5))
                })
                .unwrap();
            let issued = entry.fragment().finalized_fragment();
            let finalized = issued.finalized_line();
            assert_eq!(finalized.prepared_text(), prepared);
            assert!(finalized.fragments().any(|actual| actual == *issued));
            assert_eq!(entry.fragment().line_index(), 0);
            let rect = entry.fragment().rect().unwrap();
            assert_eq!(
                (rect.origin().x, rect.size().width, rect.size().height),
                (24.0, 0.0, 20.0)
            );
            let placed = batch
                .placed_text()
                .iter()
                .find(|placed| placed.container() == 0)
                .unwrap()
                .layout();
            assert_eq!(
                placed.analyzed_source().identity(),
                prepared.analyzed_source().identity()
            );
            let line = placed.lines()[0];
            let local = issued.view().bounds().unwrap();
            assert_eq!(
                rect.origin().y,
                (line.baseline() + (local.origin().y - finalized.feedback().baseline)) as Scalar
            );
            let position = placed
                .positions()
                .find(|position| {
                    position.source_cursor()
                        == surgeist_text::Cursor::new(4, surgeist_text::Affinity::After)
                })
                .unwrap();
            assert_eq!(
                placed
                    .position_geometry(&position)
                    .unwrap()
                    .rect()
                    .origin()
                    .x,
                24.0
            );
            let retained = placed.clone();
            assert_eq!(
                retained.position_geometry(&position).unwrap(),
                placed.position_geometry(&position).unwrap()
            );
            let (_, other) = generated_native_batch(&golden);
            assert!(
                other.placed_text()[0]
                    .layout()
                    .position_geometry(&position)
                    .is_err(),
                "equal inputs do not manufacture placement identity"
            );
            assert!(
                placed
                    .run_views()
                    .all(|run| run.face().bytes() == pinned_font::BYTES && run.face().index() == 0)
            );
            let glyphs = placed
                .glyph_runs()
                .into_iter()
                .filter(|run| run.line() == 0)
                .flat_map(|run| {
                    let origin = run.origin().x;
                    run.glyphs()
                        .iter()
                        .map(|glyph| {
                            (
                                glyph.range(),
                                glyph.id(),
                                origin + glyph.pen() + f64::from(glyph.x()),
                                glyph.advance(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            // Residual PDI/RLI glyph rows are suppressed at Text's provider
            // boundary. Their original zero Range/source custody remains;
            // the three real Ahem X glyphs retain these canonical positions.
            assert_eq!(
                glyphs,
                vec![
                    (surgeist_text::Range::new(0, 1), 58, 24.0, 16.0),
                    (surgeist_text::Range::new(2, 3), 58, 40.0, 16.0),
                    (surgeist_text::Range::new(3, 4), 58, 56.0, 16.0)
                ]
            );
            let browser = golden.expectations.children[1]
                .range_rectangles
                .as_ref()
                .unwrap()[2];
            assert_eq!(
                (
                    browser.source_segment_id,
                    browser.line_index,
                    browser.start,
                    browser.advance
                ),
                (1, 0, 72.0, 0.0)
            );
            let error = assert_surgeist_matches(&golden).unwrap_err();
            assert!(error.to_string().contains(
                "Range rectangle[2] physical flow-inline start mismatch, expected 72, got 24"
            ));
            println!(
                "{name}: original PDI4..5, prepared level2, actual issued line0/placement; native zero Range source bounds={rect:?}; browser zero Range observation x72; retained glyphs={glyphs:?}"
            );
        }
    }

    fn native_tree_compute(
        tree: &TestTree,
        request: layout::LayoutRootRequest,
    ) -> layout::CompletedLayoutBatch<usize> {
        let capture = layout::LayoutFlowInput::try_new(
            tree,
            0,
            tree.paragraphs.clone(),
            surgeist_layout::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        layout::compute_layout_with_flow(&capture, 0, request).unwrap()
    }
    use super::*;

    fn test_node_input(attrs: StyleAttrs) -> Result<layout::NodeInput, Error> {
        to_node_input(&attrs)
    }

    fn overflow_attrs(x: Option<&str>, y: Option<&str>) -> StyleAttrs {
        let mut attrs = BTreeMap::new();
        if let Some(x) = x {
            attrs.insert("overflow-x".to_string(), x.to_string());
        }
        if let Some(y) = y {
            attrs.insert("overflow-y".to_string(), y.to_string());
        }
        StyleAttrs { attrs }
    }

    fn observed_overflow_axes(input: &layout::NodeInput) -> (layout::Overflow, layout::Overflow) {
        (input.overflow.x(), input.overflow.y())
    }

    fn fri05_c06_attrs(values: &[(&str, &str)]) -> StyleAttrs {
        StyleAttrs {
            attrs: values
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        }
    }

    fn fixture_overflow_clip_margin(value: &str) -> Result<layout::OverflowClipMargin, Error> {
        let raw = format!(
            concat!(
                "<test name=\"overflow-clip-margin\">",
                "<viewport width=\"max-content\" height=\"max-content\"/>",
                "<input><div overflow-clip-margin=\"{}\"/></input>",
                "<expectations><node/></expectations>",
                "</test>"
            ),
            value
        );
        let golden = Golden::parse(&raw)?;
        let tree = TestTree::from_golden(&golden.root)?;
        Ok(tree.box_node_input(0).overflow_clip_margin)
    }

    #[test]
    fn fri07_c04_overflow_clip_margin_parser_lowers_one_token_clip_boxes_with_zero_margin() {
        let mut rejected = Vec::new();
        for (value, expected_box) in [
            ("content-box", layout::OverflowClipBox::ContentBox),
            ("padding-box", layout::OverflowClipBox::PaddingBox),
            ("border-box", layout::OverflowClipBox::BorderBox),
        ] {
            let Ok(actual) = fixture_overflow_clip_margin(value) else {
                rejected.push(value);
                continue;
            };

            assert_eq!(actual.clip_box(), expected_box);
            assert_eq!(actual.margin(), 0.0);
        }
        assert!(
            rejected.is_empty(),
            "rejected one-token overflow clip boxes: {rejected:?}"
        );
    }

    #[test]
    fn fri07_c04_overflow_clip_margin_parser_preserves_length_forms_and_invalid_tokens() {
        for (value, expected_box, expected_margin) in [
            ("6px", layout::OverflowClipBox::PaddingBox, 6.0),
            (
                "content-box 3.5px",
                layout::OverflowClipBox::ContentBox,
                3.5,
            ),
        ] {
            let actual = fixture_overflow_clip_margin(value)
                .unwrap_or_else(|error| panic!("rejected overflow-clip-margin={value:?}: {error}"));
            assert_eq!(actual.clip_box(), expected_box);
            assert_eq!(actual.margin(), expected_margin);
        }

        for value in [
            "",
            "inherit",
            "padding-box -1px",
            "padding-box 1em",
            "padding-box var(--clip)",
            "content-box 0px extra",
        ] {
            assert!(
                fixture_overflow_clip_margin(value).is_err(),
                "accepted overflow-clip-margin={value:?}"
            );
        }
    }

    #[test]
    fn fri07_c04_collapse_parser_maps_only_collapsed_and_defaults_absence() {
        let collapsed = test_node_input(fri05_c06_attrs(&[("flex-item-collapse", "collapsed")]))
            .expect("normalized collapsed fixture fact should parse");
        assert_eq!(
            collapsed.flex_item_collapse,
            layout::FlexItemCollapse::Collapsed
        );

        let normal = test_node_input(fri05_c06_attrs(&[]))
            .expect("absent collapse fixture fact should use the public default");
        assert_eq!(normal.flex_item_collapse, layout::FlexItemCollapse::Normal);
    }

    #[test]
    fn fri07_c04_collapse_parser_rejects_noncanonical_and_authored_states() {
        for value in [
            "normal",
            "visible",
            "hidden",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "",
            "collapse",
            "COLLAPSED",
            " collapsed ",
        ] {
            assert!(
                test_node_input(fri05_c06_attrs(&[("flex-item-collapse", value)])).is_err(),
                "parser accepted flex-item-collapse={value:?}"
            );
        }
        assert!(
            test_node_input(fri05_c06_attrs(&[("visibility", "collapse")])).is_err(),
            "parser accepted authored visibility"
        );
    }

    #[test]
    fn fri07_c04_collapse_parser_rejects_duplicate_normalized_attributes() {
        let error = Golden::parse(concat!(
            "<test name=\"duplicate\">",
            "<viewport width=\"max-content\" height=\"max-content\"/>",
            "<input><div flex-item-collapse=\"collapsed\" flex-item-collapse=\"collapsed\"/></input>",
            "<expectations><node/></expectations>",
            "</test>"
        ))
        .expect_err("duplicate collapse attributes must be rejected");
        assert!(error.to_string().contains("attribute"), "{error}");
    }

    #[test]
    fn fri07_c04_collapse_parser_ignores_fixture_name_and_expected_geometry() {
        fn parsed_input(name: &str, width: &str) -> layout::NodeInput {
            let raw = format!(
                concat!(
                    "<test name=\"{}\">",
                    "<viewport width=\"max-content\" height=\"max-content\"/>",
                    "<input><div flex-item-collapse=\"collapsed\"/></input>",
                    "<expectations><node width=\"{}\"/></expectations>",
                    "</test>"
                ),
                name, width
            );
            let golden = Golden::parse(&raw).expect("collapse fixture should parse");
            TestTree::from_golden(&golden.root)
                .expect("fixture should lower")
                .box_node_input(0)
                .clone()
        }

        assert_eq!(
            parsed_input("collapsed-name", "10"),
            parsed_input("renamed-control", "999")
        );
    }

    #[test]
    fn fri05_c06_parser_lowers_finite_scroll_fields_through_production_types() {
        let input = test_node_input(fri05_c06_attrs(&[
            ("overflow-x", "hidden"),
            ("overflow-y", "auto"),
            ("overflow-clip-margin", "content-box 3.5px"),
            ("scrollbar-gutter", "stable both-edges"),
            ("scroll-padding-top", "auto"),
            ("scroll-padding-right", "12px"),
            ("scroll-padding-bottom", "25%"),
            ("scroll-padding-left", "calc(4px + 10%)"),
            ("scroll-margin-top", "-1px"),
            ("scroll-margin-right", "2px"),
            ("scroll-margin-bottom", "3.5px"),
            ("scroll-margin-left", "-4px"),
            ("scroll-snap-type", "inline mandatory"),
            ("scroll-snap-align", "start center"),
            ("scroll-snap-stop", "always"),
        ]))
        .expect("finite computed scroll fixture fields should lower");

        assert_eq!(
            observed_overflow_axes(&input),
            (layout::Overflow::Hidden, layout::Overflow::Auto)
        );
        assert_eq!(
            input.overflow_clip_margin.clip_box(),
            layout::OverflowClipBox::ContentBox
        );
        assert_eq!(input.overflow_clip_margin.margin(), 3.5);
        assert_eq!(
            input.scrollbar_gutter,
            layout::ScrollbarGutter::StableBothEdges
        );
        assert!(input.scroll_padding.top().is_auto());
        for (actual, px, percent) in [
            (input.scroll_padding.right(), 12.0, 0.0),
            (input.scroll_padding.bottom(), 0.0, 0.25),
            (input.scroll_padding.left(), 4.0, 0.1),
        ] {
            let layout::ScrollPaddingValue::Value(actual) = actual else {
                panic!("expected numeric scroll padding, got {actual:?}");
            };
            assert_eq!(
                (actual.absolute_px(), actual.percent_fraction()),
                (px, percent)
            );
        }
        assert_eq!(
            (
                input.scroll_margin.top(),
                input.scroll_margin.right(),
                input.scroll_margin.bottom(),
                input.scroll_margin.left(),
            ),
            (-1.0, 2.0, 3.5, -4.0)
        );
        assert_eq!(
            input.scroll_snap_type,
            layout::ScrollSnapType::Enabled {
                axis: layout::ScrollSnapAxis::Inline,
                strictness: layout::ScrollSnapStrictness::Mandatory,
            }
        );
        assert_eq!(
            input.scroll_snap_align,
            layout::ScrollSnapAlign::new(
                layout::ScrollSnapAlignValue::Start,
                layout::ScrollSnapAlignValue::Center,
            )
        );
        assert_eq!(input.scroll_snap_stop, layout::ScrollSnapStop::Always);
    }

    #[test]
    fn fri05_c06_parser_accepts_exact_keyword_domains_and_initials() {
        for overflow in ["visible", "clip", "hidden", "scroll", "auto"] {
            assert!(
                parse_overflow(overflow).is_ok(),
                "rejected overflow {overflow}"
            );
        }
        for (name, value) in [
            ("overflow-clip-margin", "border-box 0px"),
            ("scrollbar-gutter", "stable"),
            ("scroll-snap-type", "both proximity"),
            ("scroll-snap-align", "none end"),
            ("scroll-snap-stop", "normal"),
        ] {
            test_node_input(fri05_c06_attrs(&[(name, value)]))
                .unwrap_or_else(|error| panic!("rejected {name}={value:?}: {error}"));
        }
    }

    #[test]
    fn fri05_c06_parser_rejects_ambiguous_or_non_computed_scroll_syntax() {
        for (name, value) in [
            ("overflow", "hidden"),
            ("scroll-padding", "1px 2px"),
            ("scroll-margin", "1px"),
            ("transform", "translateX(1px)"),
            ("overflow-clip-margin", "inherit"),
            ("overflow-clip-margin", "padding-box -1px"),
            ("overflow-clip-margin", "padding-box 1em"),
            ("overflow-clip-margin", "padding-box var(--clip)"),
            ("scrollbar-gutter", "stable force"),
            ("scroll-padding-top", "initial"),
            ("scroll-padding-top", "1em"),
            ("scroll-padding-top", "var(--padding)"),
            ("scroll-padding-top", "NaNpx"),
            ("scroll-margin-left", "10%"),
            ("scroll-margin-left", "infpx"),
            ("scroll-snap-type", "x"),
            ("scroll-snap-type", "x mandatory extra"),
            ("scroll-snap-align", "start"),
            ("scroll-snap-align", "start center end"),
            ("scroll-snap-stop", "inherit"),
        ] {
            assert!(
                test_node_input(fri05_c06_attrs(&[(name, value)])).is_err(),
                "accepted {name}={value:?}"
            );
        }
    }

    #[test]
    fn fri05_c06_parser_rejects_noncanonical_computed_overflow_constructor_pairs() {
        assert!(
            layout::ComputedOverflow::try_new(layout::Overflow::Visible, layout::Overflow::Auto)
                .is_err()
        );
        assert!(
            layout::ComputedOverflow::try_new(layout::Overflow::Clip, layout::Overflow::Scroll)
                .is_err()
        );
    }

    #[test]
    fn fri05_c06_computed_overflow_transition_accepts_direct_valid_computed_pairs() {
        use layout::Overflow::{Auto, Clip, Hidden, Scroll, Visible};

        for (x_token, y_token, expected) in [
            ("visible", "visible", (Visible, Visible)),
            ("visible", "clip", (Visible, Clip)),
            ("clip", "visible", (Clip, Visible)),
            ("clip", "clip", (Clip, Clip)),
            ("hidden", "hidden", (Hidden, Hidden)),
            ("hidden", "scroll", (Hidden, Scroll)),
            ("hidden", "auto", (Hidden, Auto)),
            ("scroll", "hidden", (Scroll, Hidden)),
            ("scroll", "scroll", (Scroll, Scroll)),
            ("scroll", "auto", (Scroll, Auto)),
            ("auto", "hidden", (Auto, Hidden)),
            ("auto", "scroll", (Auto, Scroll)),
            ("auto", "auto", (Auto, Auto)),
        ] {
            let input = test_node_input(overflow_attrs(Some(x_token), Some(y_token)))
                .unwrap_or_else(|error| {
                    panic!("rejected valid pair ({x_token}, {y_token}): {error}")
                });
            assert_eq!(observed_overflow_axes(&input), expected);
        }
    }

    #[test]
    fn fri05_c06_computed_overflow_transition_rejects_direct_invalid_computed_pairs() {
        for (x, y) in [
            ("visible", "hidden"),
            ("visible", "scroll"),
            ("visible", "auto"),
            ("clip", "hidden"),
            ("clip", "scroll"),
            ("clip", "auto"),
            ("hidden", "visible"),
            ("hidden", "clip"),
            ("scroll", "visible"),
            ("scroll", "clip"),
            ("auto", "visible"),
            ("auto", "clip"),
        ] {
            assert!(
                test_node_input(overflow_attrs(Some(x), Some(y))).is_err(),
                "accepted invalid computed pair ({x}, {y})"
            );
        }
    }

    fn fri05_c06_scroll_expectation(width: Scalar, height: Scalar) -> Golden {
        let mut golden = Golden::parse(
            r#"
            <test name="fri05-c06-nonzero" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="flex" overflow-x="scroll" overflow-y="scroll"
                         scrollbar-width="15" align-items="start" justify-content="end"
                         width="50px" height="50px">
                        <div display="flex" flex-shrink="0" width="100px" height="100px" />
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="50"
                          scroll_width="65" scroll_height="65">
                        <node x="-65" y="0" width="100" height="100" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("scroll expectation fixture should parse");
        golden.expectations.scroll_size = Some(Size::new(width, height));
        golden
    }

    #[test]
    fn fri05_c06_comparator_signed_range_uses_span_instead_of_maximum_endpoint() {
        assert_surgeist_matches(&fri05_c06_scroll_expectation(65.0, 65.0))
            .expect("canonical signed range span should match");
    }

    #[test]
    fn fri05_c06_comparator_explicit_zero_range_spans_pass() {
        let golden = Golden::parse(
            r#"
            <test name="fri05-c06-zero" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div overflow-x="hidden" overflow-y="hidden" width="50px" height="50px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="50"
                          scroll_width="0" scroll_height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("zero scroll expectation fixture should parse");

        assert_surgeist_matches(&golden).expect("explicit zero range spans should match");
    }

    #[test]
    fn fri05_c06_comparator_wrong_x_range_span_names_scroll_width_mismatch() {
        let error = assert_surgeist_matches(&fri05_c06_scroll_expectation(64.0, 65.0))
            .expect_err("wrong x range span should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll width mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_wrong_y_range_span_names_scroll_height_mismatch() {
        let error = assert_surgeist_matches(&fri05_c06_scroll_expectation(65.0, 64.0))
            .expect_err("wrong y range span should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll height mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_wrong_scroll_span_precedes_invalid_child_mismatch() {
        let mut golden = fri05_c06_scroll_expectation(64.0, 65.0);
        golden.expectations.children[0].x = Some(-64.0);

        let error = assert_surgeist_matches(&golden)
            .expect_err("wrong root scroll span should fail before the invalid child");

        assert_eq!(
            error.to_string(),
            "fri05-c06-nonzero: scroll width mismatch, expected 64, got 65"
        );
    }

    #[test]
    fn fri05_c06_comparator_absent_geometry_names_scroll_geometry_mismatch() {
        let golden = Golden::parse(
            r#"
            <test name="fri05-c06-missing-geometry" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="none" />
                </input>
                <expectations>
                    <node scroll_width="0" scroll_height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("missing geometry fixture should parse");
        let error = assert_surgeist_matches(&golden)
            .expect_err("missing canonical scroll geometry should fail");

        assert_eq!(
            error.to_string(),
            "fri05-c06-missing-geometry: scroll geometry mismatch, expected canonical geometry"
        );
    }

    fn line_break_tree(input: layout::LineBreakInput) -> TestTree {
        TestTree {
            paragraphs: Vec::new(),
            nodes: vec![TestNode {
                non_box_input: layout::NodeInput::non_box(),
                layout_input: layout::LayoutInput::LineBreak(input),
                is_br_source: true,
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children: Vec::new(),
                synthetic: false,
                parent: None,
                range_root: None,
                explicit_range_root: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: layout::NodeOutput::new(),
                unrounded_present: true,
                final_layout: layout::NodeOutput::new(),
                final_layout_present: true,
                unrounded_inline_fragments: None,
                final_inline_fragments: None,
                shape_bands: None,
            }],
        }
    }

    fn fri06_c12_t07_endpoint_control_comparison_fixture() -> (TestTree, Expectation) {
        fn block_input() -> layout::NodeInput {
            layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::default()
            }
        }

        fn output(x: Scalar, y: Scalar, width: Scalar, height: Scalar) -> layout::NodeOutput {
            let mut output = layout::NodeOutput::new();
            output.location = layout::Point::new(x, y);
            output.size = layout::Size::new(width, height);
            output
        }

        fn node(
            layout_input: layout::LayoutInput,
            parent: Option<usize>,
            children: Vec<usize>,
            output: layout::NodeOutput,
        ) -> TestNode {
            TestNode {
                non_box_input: layout::NodeInput::non_box(),
                is_br_source: matches!(layout_input, layout::LayoutInput::LineBreak(_)),
                layout_input,
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children,
                synthetic: false,
                parent,
                range_root: None,
                explicit_range_root: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: output,
                unrounded_present: true,
                final_layout: output,
                final_layout_present: true,
                unrounded_inline_fragments: None,
                final_inline_fragments: None,
                shape_bands: None,
            }
        }

        fn geometry(x: Scalar, y: Scalar, width: Scalar, height: Scalar) -> Expectation {
            Expectation {
                x: Some(x),
                y: Some(y),
                width: Some(width),
                height: Some(height),
                scroll_size: None,
                fragments: None,
                range_rectangles: None,
                browser_control: None,
                children: Vec::new(),
            }
        }

        let tree = TestTree {
            paragraphs: Vec::new(),
            nodes: vec![
                node(
                    layout::LayoutInput::box_input(block_input()),
                    None,
                    vec![1, 2, 3],
                    output(0.0, 0.0, 100.0, 45.0),
                ),
                node(
                    layout::LayoutInput::box_input(block_input()),
                    Some(0),
                    Vec::new(),
                    output(0.0, 5.0, 20.0, 20.0),
                ),
                node(
                    layout::LayoutInput::line_break(layout::LineBreakInput::new()),
                    Some(0),
                    Vec::new(),
                    output(20.0, 25.0, 0.0, 0.0),
                ),
                node(
                    layout::LayoutInput::box_input(block_input()),
                    Some(0),
                    Vec::new(),
                    output(0.0, 25.0, 20.0, 20.0),
                ),
            ],
        };
        let mut control = geometry(0.0, 0.0, 0.0, 0.0);
        control.x = None;
        control.y = None;
        control.width = None;
        control.height = None;
        control.browser_control = Some(BrowserControlExpectation {
            source_index: 1,
            terminal_visual_slot: Some(1),
            previous_line: BrowserNeighborLine::Same,
            next_line: BrowserNeighborLine::Later,
        });
        let expected = Expectation {
            children: vec![
                geometry(0.0, 5.0, 20.0, 20.0),
                control,
                geometry(0.0, 25.0, 20.0, 20.0),
            ],
            ..geometry(0.0, 0.0, 100.0, 45.0)
        };
        (tree, expected)
    }

    #[test]
    fn fri06_c06_comparator_wrong_control_x_names_x_mismatch() {
        let tree = line_break_tree(layout::LineBreakInput::new());
        let expected = Expectation {
            x: Some(1.0),
            y: Some(0.0),
            width: Some(0.0),
            height: Some(0.0),
            scroll_size: None,
            fragments: None,
            range_rectangles: None,
            browser_control: None,
            children: vec![Expectation {
                x: Some(99.0),
                y: None,
                width: None,
                height: None,
                scroll_size: None,
                fragments: None,
                range_rectangles: None,
                browser_control: None,
                children: Vec::new(),
            }],
        };

        let error = compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[], &[])
            .expect_err("wrong control x should fail before unrelated child comparison");

        assert_eq!(
            error.to_string(),
            "fri06-c06-control: x mismatch, expected 1, got 0"
        );
    }

    #[test]
    fn fri06_c06_comparator_zero_control_geometry_passes_and_missing_output_is_named() {
        let expected = Expectation {
            x: Some(0.0),
            y: Some(0.0),
            width: Some(0.0),
            height: Some(0.0),
            scroll_size: None,
            fragments: None,
            range_rectangles: None,
            browser_control: None,
            children: Vec::new(),
        };
        let mut tree = line_break_tree(layout::LineBreakInput::new());
        compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[], &[])
            .expect("published zero control geometry should compare");

        tree.nodes[0].final_layout_present = false;
        let error = compare_expectation(&tree, 0, &expected, "fri06-c06-control", true, &[], &[])
            .expect_err("missing control output should fail");
        assert_eq!(
            error.to_string(),
            "fri06-c06-control: control geometry mismatch, expected final output"
        );
    }

    fn fri06_c06_fragment_xml(fragment_body: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-fragment-parser" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input><div display="block" /></input>
                <expectations>
                    <node>{fragment_body}</node>
                </expectations>
            </test>
            "#
        )
    }

    fn fri06_c08_range_rectangle_golden() -> Golden {
        let mut golden = native_test_golden(&[("X", 10.0, 0.0)], false);
        golden.name = "fri06-c08-range-ink".to_string();
        golden.expectations.children = vec![Expectation {
            x: None,
            y: None,
            width: None,
            height: None,
            scroll_size: None,
            fragments: None,
            range_rectangles: Some(vec![InlineRangeRectangleExpectation {
                source_segment_id: 0,
                line_index: 0,
                physical_start_edge: PhysicalStartEdge::Left,
                start: 0.0,
                advance: 10.0,
            }]),
            browser_control: None,
            children: Vec::new(),
        }];
        golden
    }

    #[test]
    fn fri06_c08_range_rectangle_parser_and_comparator_ignore_browser_block_ink_geometry() {
        let golden = fri06_c08_range_rectangle_golden();

        assert_surgeist_matches(&golden).expect(
            "Range rectangle should compare source/line/visual and flow-inline facts without browser block ink geometry",
        );
    }

    #[test]
    fn fri06_c08_range_rectangle_rounded_fixture_compares_unrounded_model_advance() {
        // Ahem's one-em advance at the exact canonical size 10 - 23/64 is
        // 9.640625. A lone unit has no tracking gap; the fractional font metric
        // independently distinguishes this unrounded Range from its 10px box.
        let mut golden = native_test_golden(&[("X", 9.640625, 0.0)], false);
        golden.expectations.children = vec![Expectation {
            x: None,
            y: None,
            width: None,
            height: None,
            scroll_size: None,
            fragments: None,
            range_rectangles: Some(vec![InlineRangeRectangleExpectation {
                source_segment_id: 0,
                line_index: 0,
                physical_start_edge: PhysicalStartEdge::Left,
                start: 0.0,
                advance: 9.640625,
            }]),
            browser_control: None,
            children: Vec::new(),
        }];

        assert_surgeist_matches(&golden)
            .expect("browser Range advance must compare with the unrounded model fragment");
    }

    #[test]
    fn fri06_c08_range_rectangle_parser_does_not_require_browser_visual_order() {
        let xml = fri06_c06_fragment_xml(
            r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="0" advance="10" /></range-inks>"#,
        );

        Golden::parse(&xml).expect(
            "Range rectangle should retain source/line/inline facts without a visual index",
        );
    }

    #[test]
    fn fri06_c08r_empty_range_parser_preserves_explicit_zero_observations() {
        let golden = Golden::parse(&fri06_c06_fragment_xml("<range-inks/>"))
            .expect("explicit empty Range evidence should parse");

        assert_eq!(golden.expectations.range_rectangles, Some(Vec::new()));
    }

    #[test]
    fn fri06_c08r_empty_range_comparator_rejects_unexpected_model_fragment() {
        let mut golden = fri06_c08_range_rectangle_golden();
        golden.expectations.children[0].range_rectangles = Some(Vec::new());

        let error = assert_surgeist_matches(&golden)
            .expect_err("explicit empty Range evidence must enforce zero model fragments");
        assert_eq!(
            error.to_string(),
            "fri06-c08-range-ink/0: Range rectangle count mismatch, expected 0, got 1"
        );
    }

    #[test]
    fn fri06_c08_r0_range_observation_order_does_not_supply_model_visual_order() {
        let golden = native_test_golden(&[("X", 10.0, 0.0), ("X", 5.0, 0.0)], false);
        let tree = TestTree::from_golden(&golden.root).unwrap();
        let batch = native_tree_compute(&tree, fri06_c06_inline_request());
        let actual = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry
                    .fragment()
                    .finalized_fragment()
                    .view()
                    .origin()
                    .is_some()
            })
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), 2);
        let mut expected = actual
            .iter()
            .map(|entry| {
                let fragment = entry.fragment();
                InlineRangeRectangleExpectation {
                    source_segment_id: fragment_source_id(fragment).unwrap(),
                    line_index: fragment.line_index(),
                    physical_start_edge: PhysicalStartEdge::Left,
                    start: fragment.rect().unwrap().origin().x,
                    advance: fragment.rect().unwrap().size().width,
                }
            })
            .collect::<Vec<_>>();
        expected.reverse();

        compare_range_rectangle_expectations_with_translation(
            "fri06-c08-r0-range",
            &actual,
            &expected,
            |_| Ok(0.0),
        )
        .expect("Range source order must not become a model visual-index assertion");

        for field in ["source", "line", "start", "advance"] {
            let mut changed = expected.clone();
            match field {
                "source" => changed[0].source_segment_id += 1,
                "line" => changed[0].line_index += 1,
                "start" => changed[0].start += 1.0,
                "advance" => changed[0].advance += 1.0,
                _ => unreachable!(),
            }
            assert!(
                compare_range_rectangle_expectations_with_translation(
                    "fri06-c08-r0-range",
                    &actual,
                    &changed,
                    |_| Ok(0.0),
                )
                .is_err(),
                "Range {field} association must remain strict"
            );
        }
    }

    fn fri06_c08_r0_flex_control_golden(flex_wrap: &str) -> Golden {
        Golden::parse(&format!(
            r#"
            <test name="fri06-c08-r0-flex-control" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input>
                    <div display="flex" flex-wrap="{flex_wrap}">
                        <div display="block" />
                        <div source-tag="br" display="inline" />
                        <div display="block" />
                    </div>
                </input>
                <expectations>
                    <node>
                        <node />
                        <node><browser-control source_index="1" terminal_visual_slot="1" previous_line="same" next_line="same" /></node>
                        <node />
                    </node>
                </expectations>
            </test>
            "#,
        ))
        .expect("finite flex control fixture should parse")
    }

    #[test]
    fn fri06_c08_r0_nonwrapping_flex_control_uses_source_and_line_membership_without_geometry() {
        let golden = fri06_c08_r0_flex_control_golden("nowrap");
        let mut tree = TestTree::from_golden(&golden.root).unwrap();

        compare_expectation(
            &tree,
            0,
            &golden.expectations,
            &golden.name,
            golden.use_rounding,
            &[],
            &[],
        )
        .expect("nonwrapping flex controls must not require model control-point geometry");

        for field in ["source", "terminal", "previous", "next"] {
            let mut changed = golden.clone();
            let control = changed.expectations.children[1]
                .browser_control
                .as_mut()
                .unwrap();
            match field {
                "source" => control.source_index = 0,
                "terminal" => control.terminal_visual_slot = Some(0),
                "previous" => control.previous_line = BrowserNeighborLine::Earlier,
                "next" => control.next_line = BrowserNeighborLine::Later,
                _ => unreachable!(),
            }
            assert!(
                compare_expectation(
                    &tree,
                    0,
                    &changed.expectations,
                    &changed.name,
                    changed.use_rounding,
                    &[],
                    &[],
                )
                .is_err(),
                "nonwrapping flex browser-control {field} must remain strict"
            );
        }

        tree.nodes[2].is_br_source = false;
        let error = compare_expectation(
            &tree,
            0,
            &golden.expectations,
            &golden.name,
            golden.use_rounding,
            &[],
            &[],
        )
        .expect_err("browser-control observations require a BR source association");
        assert!(error.to_string().contains("expected a BR source"));
    }

    #[test]
    fn fri06_c08_r0_wrapped_flex_control_is_rejected() {
        let golden = fri06_c08_r0_flex_control_golden("wrap");
        let tree = TestTree::from_golden(&golden.root).unwrap();

        let error = compare_expectation(
            &tree,
            0,
            &golden.expectations,
            &golden.name,
            golden.use_rounding,
            &[],
            &[],
        )
        .expect_err("wrapped flex browser-control observations must fail closed");
        assert!(
            error
                .to_string()
                .contains("wrapped flex browser control observations are unsupported"),
            "unexpected wrapped-flex diagnostic: {error}"
        );
    }

    #[test]
    fn fri06_c08_r0_float_and_clear_token_tables_are_finite() {
        let flow = layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr);
        for (raw, expected) in [
            ("none", layout::Float::None),
            ("left", layout::Float::Left),
            ("right", layout::Float::Right),
            ("inline-start", layout::Float::Left),
            ("inline-end", layout::Float::Right),
        ] {
            assert_eq!(
                parse_float(raw, flow).unwrap(),
                expected,
                "float token {raw}"
            );
        }
        for (raw, expected) in [
            ("none", layout::Clear::None),
            ("left", layout::Clear::Left),
            ("right", layout::Clear::Right),
            ("inline-start", layout::Clear::Left),
            ("inline-end", layout::Clear::Right),
            ("both", layout::Clear::Both),
        ] {
            assert_eq!(
                parse_clear(raw, flow).unwrap(),
                expected,
                "clear token {raw}"
            );
        }
        for raw in [
            "both",
            "start",
            "end",
            "inline_start",
            "INLINE-START",
            " inline-start",
            "inline-end ",
            "",
        ] {
            assert!(
                parse_float(raw, flow).is_err(),
                "invalid float token {raw:?}"
            );
        }
        for raw in [
            "start",
            "end",
            "inline_start",
            "INLINE-END",
            " inline-start",
            "inline-end ",
            "",
        ] {
            assert!(
                parse_clear(raw, flow).is_err(),
                "invalid clear token {raw:?}"
            );
        }
    }

    #[test]
    fn fri06_c08_browser_control_category_observes_slots_and_neighbor_lines_not_br_rects() {
        let golden = Golden::parse(
            r#"
            <test name="fri06-c08-browser-controls" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="block" direction="ltr" font-family="monospace" font-size="16px" line-height="24px" width="140px">
                        <div display="inline-block" width="32px" height="12px" />
                        <div source-tag="br" display="inline" inline-baseline="16.8px" inline-line-height="24px" />
                        <div source-tag="br" display="inline" inline-baseline="16.8px" inline-line-height="24px" />
                        <div display="inline-block" width="48px" height="12px" />
                        <atomic-placeholder child-index="0" bidi-level="0" following-break="prohibited" />
                        <atomic-placeholder child-index="3" bidi-level="0" following-break="prohibited" />
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="140" height="72">
                        <node x="0" y="5" width="32" height="12" />
                        <node><browser-control source_index="1" terminal_visual_slot="1" previous_line="same" next_line="later" /></node>
                        <node><browser-control source_index="2" terminal_visual_slot="0" previous_line="earlier" next_line="later" /></node>
                        <node x="0" y="53" width="48" height="12" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("browser controls should parse as a distinct observation category");

        assert_surgeist_matches(&golden).expect(
            "browser BR observations should compare source, terminal slot, and neighbor-line effects without comparing the BR ink rectangle",
        );

        for (field, diagnostic) in [
            ("source", "browser control source index mismatch"),
            (
                "terminal slot",
                "browser control terminal visual slot mismatch",
            ),
            (
                "previous line",
                "browser control previous neighbor line mismatch",
            ),
            ("next line", "browser control next neighbor line mismatch"),
        ] {
            let mut changed = golden.clone();
            let control = changed.expectations.children[1]
                .browser_control
                .as_mut()
                .expect("first browser control");
            match field {
                "source" => control.source_index += 1,
                "terminal slot" => control.terminal_visual_slot = Some(2),
                "previous line" => control.previous_line = BrowserNeighborLine::Later,
                "next line" => control.next_line = BrowserNeighborLine::Same,
                _ => unreachable!(),
            }
            let error = assert_surgeist_matches(&changed)
                .expect_err("wrong browser control observation should fail");
            assert!(
                error.to_string().contains(diagnostic),
                "unexpected {field} diagnostic: {error}"
            );
        }

        let mixed = r#"
            <test name="fri06-c08-mixed-browser-control" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input><div source-tag="br" display="inline" /></input>
                <expectations>
                    <node x="1"><browser-control source_index="0" terminal_visual_slot="0" previous_line="absent" next_line="absent" /></node>
                </expectations>
            </test>
        "#;
        let error = Golden::parse(mixed)
            .expect_err("browser control observations must not carry model control geometry");
        assert_eq!(
            error.to_string(),
            "browser control observations must not include model control geometry"
        );
    }

    #[test]
    fn fri06_c12_t07_zero_size_shared_endpoint_is_same_in_both_block_orientations() {
        let forward =
            layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr);
        let control = (25.0, 25.0);
        assert_eq!(
            block_relation(control, (5.0, 25.0), forward),
            BrowserNeighborLine::Same,
            "the touching previous neighbor remains on the control line"
        );
        assert_eq!(
            block_relation(control, (25.0, 45.0), forward),
            BrowserNeighborLine::Same,
            "the touching next neighbor shares the closed control interval"
        );

        let reverse =
            layout::FlowAxes::new(layout::WritingMode::VerticalRl, layout::Direction::Ltr);
        assert_eq!(
            block_relation(control, (25.0, 45.0), reverse),
            BrowserNeighborLine::Same,
            "the physical-right touching interval is previous in reverse block flow"
        );
        assert_eq!(
            block_relation(control, (5.0, 25.0), reverse),
            BrowserNeighborLine::Same,
            "the physical-left touching interval shares the closed control interval"
        );

        assert_eq!(
            block_relation((20.0, 30.0), (25.0, 45.0), forward),
            BrowserNeighborLine::Same,
            "nonzero overlap remains same-line"
        );
        assert_eq!(
            block_relation(control, (25.05, 45.0), forward),
            BrowserNeighborLine::Same,
            "the existing browser tolerance remains unchanged away from a shared endpoint"
        );
        assert_eq!(
            block_relation(control, (30.0, 45.0), forward),
            BrowserNeighborLine::Later,
            "a real five-pixel gap remains on the later line"
        );
    }

    #[test]
    fn fri06_c12_t07_endpoint_unobservable_requires_exact_shared_endpoint() {
        let (tree, expected) = fri06_c12_t07_endpoint_control_comparison_fixture();
        let result = compare_expectation_with_endpoint_accounting(
            &tree,
            0,
            &expected,
            "fri06-c12-t07-endpoint",
            true,
            &[],
            &[],
        )
        .expect("an exact shared endpoint should be typed as endpoint-unobservable");
        assert_eq!(result.endpoint_unobservable_fields().len(), 1);
        let field = &result.endpoint_unobservable_fields()[0];
        assert_eq!(field.paths(), ["fri06-c12-t07-endpoint/1"]);
        assert_eq!(field.field(), BrowserControlComparisonField::NextLine);
        assert_eq!(field.browser_value(), BrowserNeighborLine::Later);
        assert_eq!(field.model_value(), BrowserNeighborLine::Same);

        let mut non_endpoint = tree.clone();
        non_endpoint.nodes[3].unrounded.location.y += 0.05;
        non_endpoint.nodes[3].final_layout.location.y += 0.05;
        let error = compare_expectation_with_endpoint_accounting(
            &non_endpoint,
            0,
            &expected,
            "fri06-c12-t07-endpoint",
            true,
            &[],
            &[],
        )
        .expect_err("neighbors without one exact shared endpoint must remain strict");
        assert!(
            error
                .to_string()
                .contains("browser control next neighbor line mismatch")
        );

        let mut hidden_break = tree.clone();
        hidden_break.nodes[2].layout_input =
            layout::LayoutInput::line_break(layout::LineBreakInput::new().hidden());
        assert!(
            compare_expectation_with_endpoint_accounting(
                &hidden_break,
                0,
                &expected,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("a hidden break must not qualify")
            .to_string()
            .contains("browser control next neighbor line mismatch")
        );

        let mut nonzero_control = tree.clone();
        nonzero_control.nodes[2].unrounded.size.width = 1.0;
        nonzero_control.nodes[2].final_layout.size.width = 1.0;
        assert!(
            compare_expectation_with_endpoint_accounting(
                &nonzero_control,
                0,
                &expected,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("a nonzero control must not qualify")
            .to_string()
            .contains("browser control next neighbor line mismatch")
        );

        let mut missing_neighbor = tree.clone();
        missing_neighbor.nodes[3].unrounded_present = false;
        assert!(
            compare_expectation_with_endpoint_accounting(
                &missing_neighbor,
                0,
                &expected,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("missing unrounded neighbor geometry must fail")
            .to_string()
            .contains("expected unrounded neighbor output")
        );

        let mut incomplete_neighbor = expected.clone();
        incomplete_neighbor.children[2].height = None;
        assert!(
            compare_expectation_with_endpoint_accounting(
                &tree,
                0,
                &incomplete_neighbor,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("incomplete strict neighbor comparison must not qualify")
            .to_string()
            .contains("browser control next neighbor line mismatch")
        );

        let mut wrong_neighbor = tree.clone();
        wrong_neighbor.nodes[3].final_layout.location.y += 1.0;
        assert!(
            compare_expectation_with_endpoint_accounting(
                &wrong_neighbor,
                0,
                &expected,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("wrong ordinary neighbor geometry must fail first")
            .to_string()
            .contains("y mismatch")
        );

        let mut widened_field = expected.clone();
        widened_field.children[1]
            .browser_control
            .as_mut()
            .expect("control expectation")
            .previous_line = BrowserNeighborLine::Later;
        assert!(
            compare_expectation_with_endpoint_accounting(
                &tree,
                0,
                &widened_field,
                "fri06-c12-t07-endpoint",
                true,
                &[],
                &[],
            )
            .expect_err("only next_line may be typed endpoint-unobservable")
            .to_string()
            .contains("browser control previous neighbor line mismatch")
        );
    }

    #[test]
    fn fri06_c08_range_rectangle_wrong_identity_or_inline_interval_still_fails() {
        for (field, expected_diagnostic) in [
            (
                "source",
                "Range rectangle[0] source segment id mismatch, expected 1, got 0",
            ),
            (
                "line",
                "Range rectangle[0] line index mismatch, expected 1, got 0",
            ),
            (
                "start",
                "Range rectangle[0] physical flow-inline start mismatch, expected 1, got 0",
            ),
            (
                "advance",
                "Range rectangle[0] advance mismatch, expected 11, got 10",
            ),
        ] {
            let mut golden = fri06_c08_range_rectangle_golden();
            let range_rectangle = &mut golden.expectations.children[0]
                .range_rectangles
                .as_mut()
                .unwrap()[0];
            match field {
                "source" => range_rectangle.source_segment_id += 1,
                "line" => range_rectangle.line_index += 1,
                "start" => range_rectangle.start += 1.0,
                "advance" => range_rectangle.advance += 1.0,
                _ => unreachable!(),
            }

            let error = assert_surgeist_matches(&golden)
                .expect_err("wrong Range-rectangle identity or inline interval should fail");
            assert!(
                error.to_string().contains(expected_diagnostic),
                "unexpected {field} diagnostic: {error}"
            );
        }
    }

    #[test]
    fn fri06_c08_range_rectangle_parser_is_finite_complete_and_category_exclusive() {
        let complete = r#"<range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="0" advance="10" />"#;
        for (body, diagnostic) in [
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="0" /></range-inks>"#.to_string(),
                "missing `advance` on `<range-ink>`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="inline" start="0" advance="10" /></range-inks>"#.to_string(),
                "invalid `physical_start_edge` on `<range-ink>`: `inline`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="NaN" advance="10" /></range-inks>"#.to_string(),
                "invalid `start` on `<range-ink>`: `NaN`",
            ),
            (
                r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="0" advance="-1" /></range-inks>"#.to_string(),
                "invalid `advance` on `<range-ink>`: `-1`",
            ),
            (
                format!(r#"<range-inks>{complete}</range-inks><fragments>{}</fragments>"#, fri06_c06_complete_fragment("")),
                "model fragments and Range rectangle are distinct expectation categories",
            ),
        ] {
            let error = Golden::parse(&fri06_c06_fragment_xml(&body))
                .expect_err("invalid or mixed Range-rectangle category should fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c08_range_rectangle_parser_rejects_mixed_node_scroll_and_nested_geometry() {
        let range_rectangles = r#"<range-inks><range-ink source_segment_id="11" line_index="0" physical_start_edge="left" start="0" advance="10" /></range-inks>"#;
        for (node_attributes, nested, diagnostic) in [
            (
                "x=\"1\"",
                "",
                "Range rectangle must not include node geometry",
            ),
            (
                "y=\"1\"",
                "",
                "Range rectangle must not include node geometry",
            ),
            (
                "width=\"1\"",
                "",
                "Range rectangle must not include node geometry",
            ),
            (
                "height=\"1\"",
                "",
                "Range rectangle must not include node geometry",
            ),
            (
                "scroll_width=\"1\" scroll_height=\"2\"",
                "",
                "Range rectangle must not include scroll geometry",
            ),
            (
                "content_width=\"1\"",
                "",
                "unsupported Range rectangle node expectation attribute `content_width`",
            ),
            (
                "",
                "<fragments />",
                "model fragments and Range rectangle are distinct expectation categories",
            ),
            (
                "",
                "<node x=\"1\" />",
                "Range rectangle must not include child geometry expectations",
            ),
        ] {
            let xml = format!(
                r#"
                <test name="fri06-c08-range-ink-parser" use-rounding="true">
                    <viewport width="100" height="max-content" />
                    <input><div display="block" /></input>
                    <expectations>
                        <node {node_attributes}>{range_rectangles}{nested}</node>
                    </expectations>
                </test>
                "#,
            );
            let error = Golden::parse(&xml).expect_err(
                "Range rectangle mixed with another geometry category should fail closed",
            );
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c08_range_rectangle_comparator_rejects_mixed_geometry_before_comparison() {
        for (category, diagnostic) in [
            ("node", "Range rectangle must not include node geometry"),
            ("scroll", "Range rectangle must not include scroll geometry"),
            (
                "fragments",
                "model fragments and Range rectangle are distinct expectation categories",
            ),
            (
                "children",
                "Range rectangle must not include child geometry expectations",
            ),
        ] {
            let mut golden = fri06_c08_range_rectangle_golden();
            let expected = &mut golden.expectations.children[0];
            match category {
                "node" => expected.x = Some(99.0),
                "scroll" => expected.scroll_size = Some(Size::new(99.0, 99.0)),
                "fragments" => expected.fragments = Some(Vec::new()),
                "children" => expected.children.push(Expectation {
                    x: Some(99.0),
                    y: None,
                    width: None,
                    height: None,
                    scroll_size: None,
                    fragments: None,
                    range_rectangles: None,
                    browser_control: None,
                    children: Vec::new(),
                }),
                _ => unreachable!(),
            }

            let error = assert_surgeist_matches(&golden)
                .expect_err("mixed Range-rectangle comparator state should fail closed");
            assert!(
                error.to_string().contains(diagnostic),
                "unexpected {category} diagnostic: {error}"
            );
        }
    }

    #[test]
    fn fri06_c06_comparator_parser_distinguishes_absent_and_explicit_empty_fragments() {
        let absent = Golden::parse(&fri06_c06_fragment_xml(""))
            .expect("legacy expectation without fragments should parse");
        let empty = Golden::parse(&fri06_c06_fragment_xml("<fragments />"))
            .expect("explicit empty fragment state should parse");

        assert_eq!(absent.expectations.fragments, None);
        assert_eq!(empty.expectations.fragments, Some(Vec::new()));
    }

    #[test]
    fn fri06_c06_comparator_parser_requires_every_fragment_field() {
        let fields = [
            ("source_segment_id", "11"),
            ("line_index", "0"),
            ("visual_index", "2"),
            ("x", "1.25"),
            ("y", "2.5"),
            ("width", "10.25"),
            ("height", "10"),
            ("baseline_x", "1.25"),
            ("baseline_y", "10.5"),
        ];

        for (missing, _) in fields {
            let attrs = fields
                .iter()
                .filter(|(name, _)| *name != missing)
                .map(|(name, value)| format!(r#"{name}="{value}""#))
                .collect::<Vec<_>>()
                .join(" ");
            let xml =
                fri06_c06_fragment_xml(&format!("<fragments><fragment {attrs} /></fragments>"));
            let error = Golden::parse(&xml).expect_err("missing fragment field should fail");

            assert_eq!(
                error.to_string(),
                format!("missing `{missing}` on `<fragment>`")
            );
        }
    }

    #[test]
    fn fri06_c06_comparator_parser_rejects_nonfinite_negative_and_unknown_fragment_facts() {
        for (fragment, diagnostic) in [
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="NaN" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5" />"#,
                "invalid `x` on `<fragment>`: `NaN`",
            ),
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="-1" height="10" baseline_x="1.25" baseline_y="10.5" />"#,
                "invalid `width` on `<fragment>`: `-1`",
            ),
            (
                r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5" fallback="true" />"#,
                "unsupported `<fragment>` attribute `fallback`",
            ),
        ] {
            let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
            let error = Golden::parse(&xml).expect_err("invalid fragment fact should fail");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_complete_fragment(payload: &str) -> String {
        format!(
            r#"<fragment source_segment_id="11" line_index="0" visual_index="2" x="1.25" y="2.5" width="10.25" height="10" baseline_x="1.25" baseline_y="10.5">{payload}</fragment>"#
        )
    }

    #[test]
    fn fri06_c06_comparator_fragments_reject_non_whitespace_text() {
        let xml = fri06_c06_fragment_xml("<fragments>payload</fragments>");
        let error = Golden::parse(&xml).expect_err("non-whitespace text in fragments should fail");
        assert_eq!(
            error.to_string(),
            "unsupported non-whitespace text in `<fragments>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragments_reject_nested_element() {
        let xml = fri06_c06_fragment_xml("<fragments><nested /></fragments>");
        let error = Golden::parse(&xml).expect_err("unknown fragments child should fail");
        assert_eq!(
            error.to_string(),
            "unsupported `<fragments>` child `<nested>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragment_rejects_non_whitespace_text() {
        let fragment = fri06_c06_complete_fragment("payload");
        let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
        let error = Golden::parse(&xml).expect_err("non-whitespace text in fragment should fail");
        assert_eq!(
            error.to_string(),
            "unsupported non-whitespace text in `<fragment>`"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragment_rejects_nested_element() {
        let fragment = fri06_c06_complete_fragment("<nested />");
        let xml = fri06_c06_fragment_xml(&format!("<fragments>{fragment}</fragments>"));
        let error = Golden::parse(&xml).expect_err("unknown fragment child should fail");
        assert_eq!(
            error.to_string(),
            "unsupported `<fragment>` child `<nested>`"
        );
    }

    fn fri06_c06_fragment_observation()
    -> (TestTree, layout::CompletedLayoutBatch<usize>, Expectation) {
        let golden = native_test_golden(&[("X", 10.0, 0.25), ("X", 5.0, 0.0)], true);
        let mut tree = TestTree::from_golden(&golden.root).unwrap();
        let batch = native_tree_compute(&tree, fri06_c06_inline_request());
        tree.apply_completed_batch(&batch);
        fn expected_for(
            tree: &TestTree,
            batch: &layout::CompletedLayoutBatch<usize>,
            node: usize,
        ) -> Expectation {
            let fragments = batch
                .final_inline_fragments()
                .iter()
                .filter(|entry| {
                    entry.owner() == &layout::InlineFragmentOwner::Node(node)
                        && entry
                            .fragment()
                            .finalized_fragment()
                            .view()
                            .origin()
                            .is_some()
                })
                .map(|entry| {
                    let fragment = entry.fragment();
                    let rect = fragment
                        .rect()
                        .expect("authored source has actual range geometry");
                    InlineFragmentExpectation {
                        source_segment_id: fragment_source_id(fragment).unwrap(),
                        line_index: fragment.line_index(),
                        historical_visual_index: None,
                        x: rect.origin().x,
                        y: rect.origin().y,
                        width: rect.size().width,
                        height: rect.size().height,
                        baseline_x: fragment.baseline().x,
                        baseline_y: fragment.baseline().y,
                    }
                })
                .collect();
            Expectation {
                x: None,
                y: None,
                width: None,
                height: None,
                scroll_size: None,
                fragments: tree.nodes[node]
                    .layout_input
                    .as_inline_text()
                    .is_some()
                    .then_some(fragments),
                range_rectangles: None,
                browser_control: None,
                children: tree.nodes[node]
                    .children
                    .iter()
                    .copied()
                    .map(|child| expected_for(tree, batch, child))
                    .collect(),
            }
        }

        let expected = expected_for(&tree, &batch, 0);
        (tree, batch, expected)
    }

    fn fri06_c06_compare_fragments(
        tree: &TestTree,
        batch: &layout::CompletedLayoutBatch<usize>,
        expected: &Expectation,
    ) -> Result<(), Error> {
        compare_expectation(
            tree,
            0,
            expected,
            "fri06-c06-fragments",
            false,
            batch.unrounded_inline_fragments(),
            batch.final_inline_fragments(),
        )
    }

    #[test]
    fn native_comparator_retains_actual_owners_and_finalized_physical_paint_order() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();
        let mut line = None;
        // Actual neighboring full tracking values .25/0 contribute their
        // mean .125 to the preceding10px cell; the outer halves are suppressed.
        for (node, source_id, x, width) in [(1, 0, 0.0, 10.125), (3, 1, 10.125, 5.0)] {
            let fragment = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| entry.owner() == &layout::InlineFragmentOwner::Node(node))
                .unwrap()
                .fragment();
            let issued = fragment.finalized_fragment();
            assert_eq!(
                issued.view().origin(),
                Some(tree.paragraphs[0].origins[source_id].origin.clone())
            );
            assert_eq!(fragment_source_id(fragment).unwrap(), source_id as u64);
            let rect = fragment.rect().unwrap();
            assert_eq!((rect.origin().x, rect.size().width), (x, width));
            let actual_line = issued.finalized_line();
            if let Some(previous) = &line {
                assert_eq!(previous, &actual_line);
            } else {
                line = Some(actual_line);
            }
        }
        // Actual finalized paint rows, not association-table enumeration. Ahem X
        // is glyph58. Logical spacing changes the first run's final advance but
        // does not rewrite its canonical provider advance or raw offsets.
        let line = line.unwrap();
        let paints = line
            .run_views()
            .map(|run| {
                (
                    run.range(),
                    run.origin().x,
                    run.advance(),
                    run.glyphs()
                        .map(|glyph| {
                            (
                                glyph.id(),
                                glyph.pen(),
                                glyph.x(),
                                glyph.y(),
                                glyph.advance(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            paints,
            vec![
                (
                    surgeist_text::Range::new(0, 1),
                    0.0,
                    10.125,
                    vec![(58, 0.0, 0.0, 0.0, 10.0)]
                ),
                (
                    surgeist_text::Range::new(1, 2),
                    10.125,
                    5.0,
                    vec![(58, 0.0, 0.0, 0.0, 5.0)]
                ),
            ]
        );
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("actual source associations and physical geometry should match");
    }

    #[test]
    fn historical_fragment_ordinal_stays_schema_only_and_cannot_certify_native_order() {
        let raw = fri06_c06_fragment_xml(&format!(
            "<fragments>{}</fragments>",
            fri06_c06_complete_fragment("")
        ));
        let historical = Golden::parse(&raw).unwrap();
        assert_eq!(
            historical.expectations.fragments.as_ref().unwrap()[0].historical_visual_index,
            Some(2)
        );
        let (tree, batch, mut native) = fri06_c06_fragment_observation();
        native.children[0].fragments.as_mut().unwrap()[0].historical_visual_index = Some(2);
        assert_eq!(
            fri06_c06_compare_fragments(&tree, &batch, &native)
                .unwrap_err()
                .to_string(),
            "fri06-c06-fragments/0: historical fragment visual_index has no native ordering equivalent"
        );
    }

    #[test]
    fn fri06_c06_comparator_fragment_fields_have_stable_diagnostics() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();
        let actual = batch.final_inline_fragments()[0].fragment();
        let rect = actual.rect().unwrap();
        let baseline = actual.baseline();

        let mut cases = Vec::new();
        let mut changed = expected.clone();
        changed.children[0].fragments.as_mut().unwrap()[0].source_segment_id = 12;
        cases.push((
            changed,
            "fri06-c06-fragments/0: fragment[0] source segment id mismatch, expected 12, got 0"
                .to_string(),
        ));
        let mut changed = expected.clone();
        changed.children[0].fragments.as_mut().unwrap()[0].line_index += 1;
        cases.push((
            changed,
            format!(
                "fri06-c06-fragments/0: fragment[0] line index mismatch, expected {}, got {}",
                actual.line_index() + 1,
                actual.line_index()
            ),
        ));
        for (field, actual_value) in [
            ("rect x", rect.origin().x),
            ("rect y", rect.origin().y),
            ("rect width", rect.size().width),
            ("rect height", rect.size().height),
            ("baseline x", baseline.x),
            ("baseline y", baseline.y),
        ] {
            let mut changed = expected.clone();
            let fragment = &mut changed.children[0].fragments.as_mut().unwrap()[0];
            match field {
                "rect x" => fragment.x += 1.0,
                "rect y" => fragment.y += 1.0,
                "rect width" => fragment.width += 1.0,
                "rect height" => fragment.height += 1.0,
                "baseline x" => fragment.baseline_x += 1.0,
                "baseline y" => fragment.baseline_y += 1.0,
                _ => unreachable!(),
            }
            cases.push((
                changed,
                format!(
                    "fri06-c06-fragments/0: fragment[0] {field} mismatch, expected {}, got {actual_value}",
                    actual_value + 1.0
                ),
            ));
        }

        for (changed, diagnostic) in cases {
            let error = fri06_c06_compare_fragments(&tree, &batch, &changed)
                .expect_err("wrong fragment field should fail");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn native_range_rectangles_do_not_relax_line_block_or_baseline_geometry() {
        let (tree, batch, expected) = fri06_c06_fragment_observation();
        for (field, diagnostic) in [
            ("line", "fragment[0] line index mismatch"),
            ("block", "fragment[0] rect y mismatch"),
            ("baseline", "fragment[0] baseline y mismatch"),
        ] {
            let mut changed = expected.clone();
            let fragment = &mut changed.children[0].fragments.as_mut().unwrap()[0];
            match field {
                "line" => fragment.line_index += 1,
                "block" => fragment.y += 1.0,
                "baseline" => fragment.baseline_y += 1.0,
                _ => unreachable!(),
            }
            let error = fri06_c06_compare_fragments(&tree, &batch, &changed)
                .expect_err("wrong explicit model-line block geometry should fail");
            assert!(
                error.to_string().contains(diagnostic),
                "unexpected {field} diagnostic: {error}"
            );
        }
    }

    #[test]
    fn fri06_c06_comparator_fragment_numeric_tolerance_does_not_relax_identity() {
        let (tree, batch, mut expected) = fri06_c06_fragment_observation();
        let fragment = &mut expected.children[0].fragments.as_mut().unwrap()[0];
        fragment.x += 0.05;
        fragment.baseline_y += 0.05;
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("fragment geometry and baseline should use browser tolerance");

        expected.children[0].fragments.as_mut().unwrap()[0].source_segment_id += 1;
        let error = fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect_err("actual source occurrence identity remains exact");
        assert!(
            error
                .to_string()
                .contains("fragment[0] source segment id mismatch")
        );
    }

    #[test]
    fn fri06_c06_comparator_explicit_empty_checks_while_absent_skips_fragments() {
        let (tree, batch, mut expected) = fri06_c06_fragment_observation();
        expected.children[0].fragments = None;
        fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect("absent legacy fragment expectation should preserve current meaning");

        expected.children[0].fragments = Some(Vec::new());
        let error = fri06_c06_compare_fragments(&tree, &batch, &expected)
            .expect_err("explicit empty fragment state should compare exactly");
        assert_eq!(
            error.to_string(),
            "fri06-c06-fragments/0: fragment count mismatch, expected 0, got 1"
        );
    }

    #[test]
    fn fri06_c06_comparator_missing_fragment_output_is_named() {
        let (_tree, _batch, expected) = fri06_c06_fragment_observation();
        let expected_fragments = expected.children[0].fragments.as_ref().unwrap();
        let error = compare_fragment_expectations("fri06-c06-fragments/0", &[], expected_fragments)
            .expect_err("missing final fragment output should fail");
        assert_eq!(
            error.to_string(),
            "fri06-c06-fragments/0: fragment count mismatch, expected 1, got 0"
        );
    }

    #[test]
    fn fri06_c06_comparator_old_xml_without_fragments_keeps_legacy_semantics() {
        let golden = Golden::parse(
            r#"
            <test name="fri06-c06-old-xml" use-rounding="true">
                <viewport width="100px" height="80px" />
                <input><div width="10px" height="20px" /></input>
                <expectations><node x="0" y="0" width="10" height="20" /></expectations>
            </test>
            "#,
        )
        .expect("legacy XML should parse");

        assert_surgeist_matches(&golden).expect("legacy XML meaning should remain unchanged");
    }

    #[test]
    fn layout_input_returns_browser_parity_line_break_node() {
        let input = layout::LineBreakInput::new().hidden();
        let tree = line_break_tree(input);

        assert_eq!(
            tree.layout_input(0),
            layout::LayoutInputView::LineBreak(input)
        );
        assert_eq!(tree.layout_input(0).as_line_break(), Some(input));
    }

    #[test]
    fn node_input_returns_canonical_non_box_for_browser_parity_line_break_node() {
        let tree = line_break_tree(layout::LineBreakInput::new());

        assert_eq!(tree.node_input(0), &layout::NodeInput::non_box());
    }

    #[test]
    fn parse_available_accepts_css_pixel_viewport_values() {
        assert_eq!(
            parse_available("400px").expect("px viewport should parse"),
            Available::Definite(400.0)
        );
        assert_eq!(
            parse_available("max-content").expect("keyword viewport should parse"),
            Available::MaxContent
        );
    }

    #[test]
    fn browser_parity_xml_numbers_use_default_layout_scalar() {
        fn require_default_scalar(value: layout::DefaultScalar) -> layout::DefaultScalar {
            value
        }

        let parsed: Scalar =
            parse_number("137.203125").expect("fixture number should parse as default scalar");
        // Exact default-scalar value for 137.203125, written without a
        // decimal literal that trips clippy's excessive-precision lint.
        let expected = layout::DefaultScalar::from_bits(0x4309_3400);

        assert_eq!(require_default_scalar(parsed), expected);
    }

    #[test]
    fn item_order_parser_is_canonical_and_scalar_independent() {
        fn parse_fixture_order(raw: Option<&str>) -> Result<layout::ItemOrder, Error> {
            let order = raw
                .map(|value| format!(r#" order="{value}""#))
                .unwrap_or_default();
            let golden = Golden::parse(&format!(
                r#"
                <test name="item-order" use-rounding="true">
                    <viewport width="100px" height="100px" />
                    <input>
                        <div{order} />
                    </input>
                    <expectations>
                        <node x="0" y="0" width="0" height="0" />
                    </expectations>
                </test>
                "#
            ))?;
            let tree = TestTree::from_golden(&golden.root)?;
            Ok(tree.box_node_input(0).item_order)
        }

        assert_eq!(
            parse_fixture_order(None).expect("missing order should default"),
            layout::ItemOrder::ZERO
        );
        for (raw, expected) in [
            ("-2147483648", i32::MIN),
            ("0", 0),
            ("2147483647", i32::MAX),
        ] {
            assert_eq!(
                parse_fixture_order(Some(raw)).expect("canonical i32 order should parse"),
                layout::ItemOrder::new(expected),
                "{raw}"
            );
        }

        for raw in [
            "+1",
            "01",
            "-0",
            "1.0",
            "1e0",
            "order",
            " 1",
            "1 ",
            "2147483648",
            "-2147483649",
        ] {
            assert!(
                parse_fixture_order(Some(raw)).is_err(),
                "noncanonical order `{raw}` should fail"
            );
        }
    }

    #[test]
    fn viewport_parent_axes_schema_is_strict() -> Result<(), Error> {
        fn parse_viewport(attrs: &str, root_attrs: &str) -> Result<RootContext, Error> {
            Golden::parse(&format!(
                r#"
                <test name="viewport-parent-axes" use-rounding="true">
                    <viewport width="100px" height="100px" {attrs} />
                    <input>
                        <div {root_attrs} />
                    </input>
                    <expectations>
                        <node x="0" y="0" width="0" height="0" />
                    </expectations>
                </test>
                "#
            ))
            .map(|golden| golden.viewport.root_context)
        }

        assert_eq!(parse_viewport("", "")?, RootContext::Root);
        for attrs in [
            r#"host-inline-size="100px""#,
            r#"parent-writing-mode="horizontal-tb""#,
            r#"parent-direction="ltr""#,
            r#"parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="100px""#,
        ] {
            assert!(
                parse_viewport(attrs, "").is_err(),
                "root viewport metadata `{attrs}` should be rejected"
            );
        }

        let vertical_rtl = parse_viewport(
            r#"root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="37.5px""#,
            r#"writing-mode="horizontal-tb" direction="ltr""#,
        )?;
        let horizontal_ltr = parse_viewport(
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="0px""#,
            r#"writing-mode="vertical-lr" direction="rtl""#,
        )?;
        assert_ne!(
            vertical_rtl, horizontal_ltr,
            "fixture root context should retain the parsed parent axes"
        );

        for attrs in [
            r#"root-context="flex-item""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb""#,
            r#"root-context="flex-item" parent-direction="ltr""#,
            r#"root-context="flex-item" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-direction="ltr" host-inline-size="100px""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="left-to-right""#,
            r#"root-context="flex-item" parent-writing-mode=" horizontal-tb" parent-direction="ltr""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr ""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="100""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="max-content""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="NaNpx""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="infpx""#,
            r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="-0.5px""#,
        ] {
            assert!(
                parse_viewport(attrs, r#"writing-mode="vertical-lr" direction="rtl""#).is_err(),
                "invalid flex-item viewport metadata `{attrs}` should be rejected"
            );
        }

        Ok::<(), Error>(())
    }

    #[test]
    fn flex_item_root_separates_host_inline_allocation_from_viewport_context() -> Result<(), Error>
    {
        fn request(attrs: &str) -> Result<layout::LayoutRootRequest, Error> {
            let golden = Golden::parse(&format!(
                r#"
                <test name="flex-host-allocation" use-rounding="true">
                    <viewport width="400px" height="60px" {attrs} />
                    <input><div display="grid" /></input>
                    <expectations><node x="0" y="0" width="0" height="0" /></expectations>
                </test>
                "#
            ))?;
            let viewport_available = layout::Size::new(
                to_layout_available(golden.viewport.width),
                to_layout_available(golden.viewport.height),
            );
            root_request(viewport_available, golden.viewport.root_context)
        }

        for (attrs, expected_available) in [
            (
                r#"root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="160px""#,
                layout::Size::new(
                    layout::Available::Definite(160.0),
                    layout::Available::MaxContent,
                ),
            ),
            (
                r#"root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="80.5px""#,
                layout::Size::new(
                    layout::Available::MaxContent,
                    layout::Available::Definite(80.5),
                ),
            ),
        ] {
            let request = request(attrs)?;
            assert_eq!(request.available(), expected_available);
            let layout::LayoutRootContext::FlexItemUnderViewport(context) = request.context()
            else {
                panic!("expected flex-item root context");
            };
            assert_eq!(
                context.viewport_available(),
                layout::Size::new(
                    layout::Available::Definite(400.0),
                    layout::Available::Definite(60.0),
                )
            );
        }

        Ok(())
    }

    #[test]
    fn parses_viewport_root_context_metadata() {
        let golden = Golden::parse(
            r#"
            <test name="viewport-flex-item" use-rounding="true">
                <viewport width="400px" height="max-content" root-context="flex-item" parent-writing-mode="vertical-rl" parent-direction="rtl" host-inline-size="20px" />
                <input>
                    <div display="grid" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("viewport root context should parse");

        assert_eq!(
            golden.viewport.root_context,
            RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(
                    layout::WritingMode::VerticalRl,
                    layout::Direction::Rtl,
                ),
                host_inline_size: 20.0,
            }
        );
    }

    #[test]
    fn flex_item_root_uses_the_public_compute_request() {
        let golden = Golden::parse(
            r#"
            <test name="viewport-flex-item" use-rounding="true">
                <viewport width="400px" height="80px" root-context="flex-item" parent-writing-mode="horizontal-tb" parent-direction="ltr" host-inline-size="400px" />
                <input>
                    <div display="flex" width="50%" height="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="200" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("flex-item root fixture should parse");

        assert_eq!(
            golden.viewport.root_context,
            RootContext::FlexItem {
                parent_axes: layout::FlowAxes::new(
                    layout::WritingMode::HorizontalTb,
                    layout::Direction::Ltr,
                ),
                host_inline_size: 400.0,
            }
        );

        assert_surgeist_matches(&golden)
            .expect("flex-item root should be computed through the public request");
    }

    #[test]
    fn completed_batch_application_stages_outputs_and_cache_changes() {
        let golden = Golden::parse(
            r#"
            <test name="batch-application" use-rounding="false">
                <viewport width="100px" height="80px" />
                <input>
                    <div width="10.25px" height="20.5px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="10.25" height="20.5" />
                </expectations>
            </test>
            "#,
        )
        .expect("batch fixture should parse");
        let mut tree = TestTree::from_golden(&golden.root).expect("test tree should build");
        let available = layout::Size::new(
            to_layout_available(golden.viewport.width),
            to_layout_available(golden.viewport.height),
        );
        let request = root_request(available, golden.viewport.root_context)
            .expect("root request should be valid");
        let batch = layout::compute_layout(&tree, 0, request)
            .expect("layout should produce a completed batch");

        assert_eq!(batch.unrounded_entries().len(), 1);
        assert_eq!(batch.final_entries().len(), 1);
        assert_eq!(batch.cache_store_entries().len(), 1);
        tree.apply_completed_batch(&batch);
        assert_eq!(tree.nodes[0].unrounded.size, layout::Size::new(10.25, 20.5));
        assert_eq!(
            tree.nodes[0].final_layout.size,
            layout::Size::new(10.0, 21.0)
        );
        assert!(!tree.nodes[0].cache.is_empty());

        let cached_batch = layout::compute_layout(&tree, 0, request)
            .expect("identical layout should produce a completed batch");

        assert_eq!(cached_batch.unrounded_entries().len(), 1);
        assert_eq!(cached_batch.final_entries().len(), 1);
        assert!(cached_batch.cache_store_entries().is_empty());
        assert_surgeist_matches(&golden).expect("unrounded expectations should remain observable");

        let hidden_input = layout::NodeInput {
            display: layout::Display::None,
            ..layout::NodeInput::default()
        };
        tree.nodes[0].layout_input = layout::LayoutInput::box_input(hidden_input);
        let hidden_request = root_request(available, RootContext::Root)
            .expect("hidden root request should be valid");
        let hidden_batch = layout::compute_layout(&tree, 0, hidden_request)
            .expect("hidden root should produce a completed batch");

        assert_eq!(hidden_batch.cache_clear_entries().len(), 1);
        tree.apply_completed_batch(&hidden_batch);
        assert!(tree.nodes[0].cache.is_empty());
    }

    #[test]
    fn parses_partial_geometry_expectations() {
        let golden = Golden::parse(
            r#"
            <test name="partial-geometry" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div width="10px" height="20px" />
                </input>
                <expectations>
                    <node height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("partial geometry should parse");

        assert_eq!(golden.expectations.height, Some(20.0));
        assert_eq!(golden.expectations.width, None);
        assert_eq!(golden.expectations.x, None);
        assert_eq!(golden.expectations.y, None);
    }

    #[test]
    fn parse_text_align_accepts_chromium_webkit_aliases() {
        assert_eq!(
            parse_text_align("-webkit-left").expect("webkit left should parse"),
            layout::TextAlign::LegacyLeft
        );
        assert_eq!(
            parse_text_align("-webkit-right").expect("webkit right should parse"),
            layout::TextAlign::LegacyRight
        );
        assert_eq!(
            parse_text_align("-webkit-center").expect("webkit center should parse"),
            layout::TextAlign::LegacyCenter
        );
    }

    #[test]
    fn property_field_migration_preferred_parser_accepts_browser_fixture_unitless_lengths() {
        assert_eq!(
            parse_preferred_size("40").expect("unitless fixture length should parse"),
            preferred_size_px(40.0)
        );
        assert_eq!(
            parse_preferred_size("0").expect("unitless zero fixture length should parse"),
            preferred_size_px(0.0)
        );
    }

    #[test]
    fn parse_length_rejects_non_fixture_css_units() {
        assert!(parse_length("1em").is_err());
        assert!(parse_length("calc(100% - 1px)").is_err());
    }

    #[test]
    fn parse_length_rejects_non_finite_fixture_numbers_without_panicking() {
        let px_error = parse_length("NaNpx").expect_err("NaN px should be rejected");
        assert!(
            px_error.to_string().contains("scalar must be finite"),
            "unexpected error: {px_error}"
        );

        let percent_error = parse_length("inf%").expect_err("infinite percent should be rejected");
        assert!(
            percent_error.to_string().contains("scalar must be finite"),
            "unexpected error: {percent_error}"
        );

        let calc_error =
            parse_length_with_calc("calc(infpx + 10%)").expect_err("infinite calc should fail");
        assert!(
            calc_error.to_string().contains("scalar must be finite"),
            "unexpected error: {calc_error}"
        );
    }

    #[test]
    fn parse_length_accepts_fixture_calc_px_plus_percent() {
        let length = parse_length_with_calc("calc(12px + 25%)").expect("fixture calc should parse");
        let layout::Length::Value(value) = length else {
            panic!("expected affine calc length, got {length:?}");
        };
        assert_eq!(value.absolute_px(), 12.0);
        assert_eq!(value.percent_fraction(), 0.25);
        assert_eq!(length.resolve_optional(Some(200.0)), Some(62.0));
    }

    #[test]
    fn property_field_migration_preferred_parser_accepts_fixture_calc_percent_minus_px() {
        let preferred = parse_preferred_size("calc(50% - 8px)")
            .expect("fixture calc preferred size should parse");
        let expected = layout::PreferredSize::value(
            layout::LengthPercentageOf::from_coefficients(-8.0, 0.5)
                .expect("finite expected fixture calculation"),
        );
        assert_eq!(preferred, expected);
    }

    #[test]
    fn property_field_migration_parser_routes_simple_values_by_destination_property() {
        let value = layout::LengthPercentageOf::px(12.0).expect("finite fixture value");
        assert_eq!(
            parse_preferred_size("12px").expect("preferred size should parse"),
            layout::PreferredSize::value(value),
        );
        assert_eq!(
            parse_min_size("12px").expect("minimum size should parse"),
            layout::MinSize::value(value),
        );
        assert_eq!(
            parse_max_size("none").expect("maximum initial value should parse"),
            layout::MaxSize::NONE,
        );
        assert_eq!(
            parse_flex_basis("12px").expect("flex basis should parse"),
            layout::FlexBasis::value(value),
        );
        assert!(parse_max_size("auto").is_err());
        assert!(parse_preferred_size("none").is_err());
        assert!(parse_min_size("1fr").is_err());
        assert!(parse_flex_basis("1fr").is_err());
    }

    #[test]
    fn fri04_c05_parser_accepts_property_keywords_nested_calculations_and_fit_content() {
        for (raw, expected) in [
            ("auto", layout::PreferredSize::AUTO),
            ("min-content", layout::PreferredSize::MIN_CONTENT),
            ("max-content", layout::PreferredSize::MAX_CONTENT),
            ("stretch", layout::PreferredSize::STRETCH),
            ("fit-content", layout::PreferredSize::FIT_CONTENT),
            ("contain", layout::PreferredSize::CONTAIN),
        ] {
            assert_eq!(
                parse_preferred_size(raw).expect("preferred keyword should parse"),
                expected,
                "preferred keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("auto", layout::MinSize::AUTO),
            ("min-content", layout::MinSize::MIN_CONTENT),
            ("max-content", layout::MinSize::MAX_CONTENT),
            ("stretch", layout::MinSize::STRETCH),
            ("fit-content", layout::MinSize::FIT_CONTENT),
            ("contain", layout::MinSize::CONTAIN),
        ] {
            assert_eq!(
                parse_min_size(raw).expect("minimum keyword should parse"),
                expected,
                "minimum keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("none", layout::MaxSize::NONE),
            ("min-content", layout::MaxSize::MIN_CONTENT),
            ("max-content", layout::MaxSize::MAX_CONTENT),
            ("stretch", layout::MaxSize::STRETCH),
            ("fit-content", layout::MaxSize::FIT_CONTENT),
            ("contain", layout::MaxSize::CONTAIN),
        ] {
            assert_eq!(
                parse_max_size(raw).expect("maximum keyword should parse"),
                expected,
                "maximum keyword {raw}"
            );
        }
        for (raw, expected) in [
            ("auto", layout::FlexBasis::AUTO),
            ("content", layout::FlexBasis::CONTENT),
            ("min-content", layout::FlexBasis::MIN_CONTENT),
            ("max-content", layout::FlexBasis::MAX_CONTENT),
            ("stretch", layout::FlexBasis::STRETCH),
            ("fit-content", layout::FlexBasis::FIT_CONTENT),
            ("contain", layout::FlexBasis::CONTAIN),
        ] {
            assert_eq!(
                parse_flex_basis(raw).expect("flex keyword should parse"),
                expected,
                "flex keyword {raw}"
            );
        }

        let px = |value| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::px(value).expect("finite expected px"),
            )
        };
        let percent = |value| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::from_percent_fraction(value)
                    .expect("finite expected percentage"),
            )
        };
        let affine = layout::SizingCalculation::value(
            layout::LengthPercentageOf::from_coefficients(5.0, 0.1)
                .expect("finite expected affine value"),
        );
        let nested_min = layout::SizingCalculation::min(vec![percent(0.25), affine])
            .expect("nonempty expected minimum");
        let nested_max = layout::SizingCalculation::max(vec![px(10.0), nested_min])
            .expect("nonempty expected maximum");
        let expected = layout::PreferredSize::calculation(layout::SizingCalculation::clamp(
            None,
            nested_max,
            Some(px(90.0)),
        ));
        assert_eq!(
            parse_preferred_size("clamp(none, max(10px, min(25%, calc(5px + 10%))), 90px)")
                .expect("nested preferred calculation should parse"),
            expected
        );
        assert_eq!(
            parse_min_size("clamp(none, 20px, none)")
                .expect("omitted clamp endpoints should parse"),
            layout::MinSize::calculation(layout::SizingCalculation::clamp(None, px(20.0), None))
        );
        assert!(
            parse_flex_basis("fit-content(max(10px, 25%))")
                .expect("flex fit-content function should parse")
                .is_fit_content_function()
        );
        assert!(matches!(
            parse_max_track_sizing_with_calc("fit-content(min(40px, 50%))")
                .expect("maximum track fit-content should parse"),
            layout::MaxTrackSizing::FitContent(_)
        ));
    }

    #[test]
    fn fri04_c05_parser_accepts_canonical_calc_size_bases_and_affine_programs() {
        let size = layout::CalcSizeCalculation::size();
        for (raw, basis) in [
            ("100%", layout::PreferredSizeCalcBasis::FullPercentage),
            ("auto", layout::PreferredSizeCalcBasis::Auto),
            ("min-content", layout::PreferredSizeCalcBasis::MinContent),
            ("max-content", layout::PreferredSizeCalcBasis::MaxContent),
            ("stretch", layout::PreferredSizeCalcBasis::Stretch),
            ("fit-content", layout::PreferredSizeCalcBasis::FitContent),
            ("contain", layout::PreferredSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_preferred_size(&format!("calc-size({raw}, size)"))
                    .expect("preferred calc-size basis should parse"),
                layout::PreferredSize::calc_size(basis, size.clone())
                    .expect("expected preferred calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::MinSizeCalcBasis::FullPercentage),
            ("auto", layout::MinSizeCalcBasis::Auto),
            ("min-content", layout::MinSizeCalcBasis::MinContent),
            ("max-content", layout::MinSizeCalcBasis::MaxContent),
            ("stretch", layout::MinSizeCalcBasis::Stretch),
            ("fit-content", layout::MinSizeCalcBasis::FitContent),
            ("contain", layout::MinSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_min_size(&format!("calc-size({raw}, size)"))
                    .expect("minimum calc-size basis should parse"),
                layout::MinSize::calc_size(basis, size.clone())
                    .expect("expected minimum calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::MaxSizeCalcBasis::FullPercentage),
            ("none", layout::MaxSizeCalcBasis::None),
            ("min-content", layout::MaxSizeCalcBasis::MinContent),
            ("max-content", layout::MaxSizeCalcBasis::MaxContent),
            ("stretch", layout::MaxSizeCalcBasis::Stretch),
            ("fit-content", layout::MaxSizeCalcBasis::FitContent),
            ("contain", layout::MaxSizeCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_max_size(&format!("calc-size({raw}, size)"))
                    .expect("maximum calc-size basis should parse"),
                layout::MaxSize::calc_size(basis, size.clone())
                    .expect("expected maximum calc-size should construct")
            );
        }
        for (raw, basis) in [
            ("100%", layout::FlexBasisCalcBasis::FullPercentage),
            ("auto", layout::FlexBasisCalcBasis::Auto),
            ("content", layout::FlexBasisCalcBasis::Content),
            ("min-content", layout::FlexBasisCalcBasis::MinContent),
            ("max-content", layout::FlexBasisCalcBasis::MaxContent),
            ("stretch", layout::FlexBasisCalcBasis::Stretch),
            ("fit-content", layout::FlexBasisCalcBasis::FitContent),
            ("contain", layout::FlexBasisCalcBasis::Contain),
        ] {
            assert_eq!(
                parse_flex_basis(&format!("calc-size({raw}, size)"))
                    .expect("flex calc-size basis should parse"),
                layout::FlexBasis::calc_size(basis, size.clone())
                    .expect("expected flex calc-size should construct")
            );
        }

        let independent = layout::CalcSizeCalculation::from_coefficients(10.0, 0.25, 0.0)
            .expect("finite independent calc-size coefficients");
        assert_eq!(
            parse_preferred_size("calc-size(any, 10px + 25%)")
                .expect("independent Any calc-size should parse"),
            layout::PreferredSize::calc_size(
                layout::PreferredSizeCalcBasis::Any,
                independent.clone()
            )
            .expect("independent Any calc-size should construct")
        );
        assert_eq!(
            parse_min_size("calc-size(any, 10px + 25%)")
                .expect("minimum Any calc-size should parse"),
            layout::MinSize::calc_size(layout::MinSizeCalcBasis::Any, independent.clone())
                .expect("minimum Any calc-size should construct")
        );
        assert_eq!(
            parse_max_size("calc-size(any, 10px + 25%)")
                .expect("maximum Any calc-size should parse"),
            layout::MaxSize::calc_size(layout::MaxSizeCalcBasis::Any, independent.clone())
                .expect("maximum Any calc-size should construct")
        );
        assert_eq!(
            parse_flex_basis("calc-size(any, 10px + 25%)")
                .expect("flex Any calc-size should parse"),
            layout::FlexBasis::calc_size(layout::FlexBasisCalcBasis::Any, independent)
                .expect("flex Any calc-size should construct")
        );

        let nested_min = layout::CalcSizeCalculation::min(vec![
            layout::CalcSizeCalculation::from_coefficients(0.0, 0.0, 0.5)
                .expect("finite size coefficient"),
            layout::CalcSizeCalculation::from_coefficients(0.0, 0.8, 0.0)
                .expect("finite percentage coefficient"),
        ])
        .expect("nonempty calc-size minimum");
        let nested_max = layout::CalcSizeCalculation::max(vec![
            layout::CalcSizeCalculation::from_coefficients(10.0, 0.25, 0.0)
                .expect("finite affine coefficient"),
            nested_min,
        ])
        .expect("nonempty calc-size maximum");
        let nested = layout::CalcSizeCalculation::clamp(
            None,
            nested_max,
            Some(
                layout::CalcSizeCalculation::from_coefficients(100.0, 0.0, 0.0)
                    .expect("finite maximum coefficient"),
            ),
        );
        assert_eq!(
            parse_preferred_size(
                "calc-size(auto, clamp(none, max(10px + 25%, min(size * 0.5, 80%)), 100px))"
            )
            .expect("nested calc-size program should parse"),
            layout::PreferredSize::calc_size(layout::PreferredSizeCalcBasis::Auto, nested)
                .expect("nested calc-size program should construct")
        );
    }

    #[test]
    fn fri04_c05_parser_accepts_track_only_flex_and_rejects_flex_in_minimum_track() {
        let factor = layout::TrackFlexFactor::try_new(2.0).expect("finite track flex");
        assert_eq!(
            parse_max_track_sizing_with_calc("2fr").expect("maximum track flex should parse"),
            layout::MaxTrackSizing::flex(factor)
        );
        assert_eq!(
            parse_track_sizing_with_calc("2fr").expect("complete track flex should parse"),
            layout::TrackSizing::flex(factor)
        );
        assert!(parse_min_track_sizing_with_calc("2fr").is_err());
        assert!(parse_track_sizing_with_calc("minmax(2fr, 20px)").is_err());
        assert!(parse_preferred_size("2fr").is_err());
        assert!(parse_min_size("2fr").is_err());
        assert!(parse_max_size("2fr").is_err());
        assert!(parse_flex_basis("2fr").is_err());

        assert!(parse_min_track_sizing_with_calc("min(10px, max(20%, 30px))").is_ok());
        assert!(parse_max_track_sizing_with_calc("max(10px, min(20%, 30px))").is_ok());
        assert!(
            parse_track_sizing_with_calc("minmax(min(10px, 20%), fit-content(max(30px, 40%)))")
                .is_ok()
        );
    }

    #[test]
    fn fri04_c05_parser_rejects_malformed_arity_nonfinite_and_cross_property_values() {
        for raw in [
            "min()",
            "max()",
            "min(10px,)",
            "clamp(10px, 20px)",
            "clamp(10px, 20px, 30px, 40px)",
            "fit-content()",
            "fit-content(10px, 20px)",
            "calc-size(any)",
            "calc-size(any, 10px, 20px)",
            "min(10px, max(20px, 30px)",
            "min(10px)), 20px)",
            "min(10px) trailing",
            "10px + 20%",
            "min(10px + 20%, 30px)",
            "NaNpx",
            "inf%",
            "calc(1e38px + 3e38px)",
            "size",
            "calc(size + 1px)",
            "min(size, 1px)",
            "calc-size(any, size)",
            "calc-size(any, max(1px, size))",
        ] {
            assert!(
                parse_preferred_size(raw).is_err(),
                "invalid preferred fixture unexpectedly parsed: {raw}"
            );
        }

        for raw in ["none", "content", "1fr"] {
            assert!(parse_preferred_size(raw).is_err(), "preferred {raw}");
            assert!(parse_min_size(raw).is_err(), "minimum {raw}");
        }
        for raw in ["auto", "content", "1fr"] {
            assert!(parse_max_size(raw).is_err(), "maximum {raw}");
        }
        for raw in ["none", "1fr"] {
            assert!(parse_flex_basis(raw).is_err(), "flex basis {raw}");
        }
        assert!(parse_min_track_sizing_with_calc("content").is_err());
        assert!(parse_min_track_sizing_with_calc("fit-content(10px)").is_err());
        assert!(parse_max_track_sizing_with_calc("content").is_err());
        assert!(parse_max_track_sizing_with_calc("none").is_err());
        assert!(parse_max_track_sizing_with_calc("-1fr").is_err());
        assert!(parse_max_track_sizing_with_calc("NaNfr").is_err());
        assert!(parse_max_track_sizing_with_calc("inffr").is_err());
    }

    #[test]
    fn fri04_c05_parser_accepts_depth_64_and_rejects_depth_65_before_descent() {
        let nested_min = |depth: usize| {
            let mut raw = "min(".repeat(depth);
            raw.push_str("10px");
            raw.push_str(&")".repeat(depth));
            raw
        };

        assert!(
            parse_preferred_size(&nested_min(64)).is_ok(),
            "fixture nesting at the documented limit should parse"
        );
        let error = parse_preferred_size(&nested_min(65))
            .expect_err("fixture nesting beyond the documented limit should fail");
        assert!(
            error.to_string().contains("exceeds 64"),
            "unexpected excessive-depth error: {error}"
        );
    }

    #[test]
    fn fri08_c08_t02_fixture_expression_leaves_and_functions_lower_to_both_destinations() {
        let sizing_value = |absolute_px, percent_fraction| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::from_coefficients(absolute_px, percent_fraction)
                    .unwrap_or_else(|error| panic!("finite sizing coefficients failed: {error}")),
            )
        };
        for (raw, calculation) in [
            ("12px", sizing_value(12.0, 0.0)),
            ("25%", sizing_value(0.0, 0.25)),
            ("7", sizing_value(7.0, 0.0)),
            ("calc(5px + 10%)", sizing_value(5.0, 0.1)),
            (
                "min(10px, 25%)",
                layout::SizingCalculation::min(vec![
                    sizing_value(10.0, 0.0),
                    sizing_value(0.0, 0.25),
                ])
                .unwrap_or_else(|error| panic!("finite sizing minimum failed: {error}")),
            ),
            (
                "max(10px, 25%)",
                layout::SizingCalculation::max(vec![
                    sizing_value(10.0, 0.0),
                    sizing_value(0.0, 0.25),
                ])
                .unwrap_or_else(|error| panic!("finite sizing maximum failed: {error}")),
            ),
            (
                "clamp(10px, 25%, 90px)",
                layout::SizingCalculation::clamp(
                    Some(sizing_value(10.0, 0.0)),
                    sizing_value(0.0, 0.25),
                    Some(sizing_value(90.0, 0.0)),
                ),
            ),
        ] {
            assert_eq!(
                parse_preferred_size(raw)
                    .unwrap_or_else(|error| panic!("ordinary expression `{raw}` failed: {error}")),
                layout::PreferredSize::calculation(calculation),
                "ordinary expression `{raw}`",
            );
        }

        let calc_value = |absolute_px, percent_fraction, size_fraction| {
            layout::CalcSizeCalculation::from_coefficients(
                absolute_px,
                percent_fraction,
                size_fraction,
            )
            .unwrap_or_else(|error| panic!("finite calc-size coefficients failed: {error}"))
        };
        for (raw, calculation) in [
            ("12px", calc_value(12.0, 0.0, 0.0)),
            ("25%", calc_value(0.0, 0.25, 0.0)),
            ("7", calc_value(7.0, 0.0, 0.0)),
            ("size", calc_value(0.0, 0.0, 1.0)),
            ("2*size", calc_value(0.0, 0.0, 2.0)),
            ("size*3", calc_value(0.0, 0.0, 3.0)),
            ("4 * size", calc_value(0.0, 0.0, 4.0)),
            ("size * 5", calc_value(0.0, 0.0, 5.0)),
            ("calc(5px + 10% + size * 0.5)", calc_value(5.0, 0.1, 0.5)),
            (
                "min(10px, size)",
                layout::CalcSizeCalculation::min(vec![
                    calc_value(10.0, 0.0, 0.0),
                    calc_value(0.0, 0.0, 1.0),
                ])
                .unwrap_or_else(|error| panic!("finite calc-size minimum failed: {error}")),
            ),
            (
                "max(25%, size)",
                layout::CalcSizeCalculation::max(vec![
                    calc_value(0.0, 0.25, 0.0),
                    calc_value(0.0, 0.0, 1.0),
                ])
                .unwrap_or_else(|error| panic!("finite calc-size maximum failed: {error}")),
            ),
            (
                "clamp(10px, size, 90px)",
                layout::CalcSizeCalculation::clamp(
                    Some(calc_value(10.0, 0.0, 0.0)),
                    calc_value(0.0, 0.0, 1.0),
                    Some(calc_value(90.0, 0.0, 0.0)),
                ),
            ),
        ] {
            let parsed = parse_preferred_size(&format!("calc-size(auto, {raw})"))
                .unwrap_or_else(|error| panic!("calc-size expression `{raw}` failed: {error}"));
            let wanted =
                layout::PreferredSize::calc_size(layout::PreferredSizeCalcBasis::Auto, calculation)
                    .unwrap_or_else(|error| {
                        panic!("finite calc-size construction failed: {error}")
                    });
            assert_eq!(parsed, wanted, "calc-size expression `{raw}`");
        }
    }

    #[test]
    fn fri08_c08_t02_fixture_expression_depth_and_optional_clamp_bounds_are_stable() {
        let nested_min = |depth: usize| {
            let mut raw = "min(".repeat(depth);
            raw.push_str("10px");
            raw.push_str(&")".repeat(depth));
            raw
        };

        assert!(parse_preferred_size(&nested_min(64)).is_ok());
        let ordinary_65 = nested_min(65);
        let ordinary_error = parse_preferred_size(&ordinary_65).unwrap_err().to_string();
        assert_eq!(
            ordinary_error,
            format!("sizing function nesting depth 65 exceeds 64 in `{ordinary_65}`")
        );

        let calc_64 = format!("calc-size(auto, {})", nested_min(63));
        assert!(parse_preferred_size(&calc_64).is_ok());
        let calc_65 = format!("calc-size(auto, {})", nested_min(64));
        let calc_error = parse_preferred_size(&calc_65).unwrap_err().to_string();
        assert_eq!(
            calc_error,
            format!("sizing function nesting depth 65 exceeds 64 in `{calc_65}`")
        );

        let px = |value| {
            layout::SizingCalculation::value(
                layout::LengthPercentageOf::px(value)
                    .unwrap_or_else(|error| panic!("finite clamp value failed: {error}")),
            )
        };
        assert_eq!(
            parse_preferred_size("clamp(none, 20px, none)").unwrap_or_else(|error| {
                panic!("ordinary omitted clamp bounds failed: {error}")
            }),
            layout::PreferredSize::calculation(layout::SizingCalculation::clamp(
                None,
                px(20.0),
                None,
            )),
        );
        let size = layout::CalcSizeCalculation::size();
        assert_eq!(
            parse_preferred_size("calc-size(auto, clamp(none, size, none))")
                .unwrap_or_else(|error| panic!("calc-size omitted clamp bounds failed: {error}"),),
            layout::PreferredSize::calc_size(
                layout::PreferredSizeCalcBasis::Auto,
                layout::CalcSizeCalculation::clamp(None, size, None),
            )
            .unwrap_or_else(|error| panic!("finite omitted clamp construction failed: {error}")),
        );
    }

    #[test]
    fn fri08_c08_t02_fixture_expression_errors_keep_precedence_and_diagnostic_identity() {
        let ordinary = |raw| parse_preferred_size(raw).unwrap_err().to_string();
        let calc_size = |raw: &str| {
            parse_preferred_size(&format!("calc-size(auto, {raw})"))
                .unwrap_err()
                .to_string()
        };

        assert_eq!(
            ordinary("Min(10px)"),
            "malformed sizing fixture function `Min(10px)`"
        );
        assert_eq!(
            calc_size("Min(10px)"),
            "malformed sizing fixture function `Min(10px)`"
        );
        assert_eq!(
            ordinary("clamp(10px, 20px)"),
            "clamp() requires exactly three arguments in `clamp(10px, 20px)`"
        );
        assert_eq!(
            calc_size("clamp(10px, 20px)"),
            "clamp() requires exactly three arguments in `clamp(10px, 20px)`"
        );
        assert_eq!(
            ordinary("NaNpx"),
            "invalid sizing fixture `NaNpx`: absolute length coefficient must be finite"
        );
        assert_eq!(
            calc_size("NaNpx"),
            "invalid calc-size fixture `NaNpx`: absolute length coefficient must be finite"
        );
        assert_eq!(ordinary("size"), "invalid number `size`");
        assert_eq!(
            ordinary("calc(size + 1px)"),
            "unsupported affine sizing term `size` in `size + 1px`"
        );
        assert_eq!(
            parse_preferred_size("calc-size(any, size)")
                .unwrap_err()
                .to_string(),
            "invalid preferred-size fixture `calc-size(any, size)`: an Any calc-size basis cannot be combined with a size reference"
        );
        assert_eq!(
            ordinary("mystery(1px)"),
            "unsupported sizing fixture function `mystery` in `mystery(1px)`"
        );
        assert_eq!(
            calc_size("mystery(1px)"),
            "unsupported calc-size fixture function `mystery` in `mystery(1px)`"
        );
        assert_eq!(ordinary("min(size, Min(1px))"), "invalid number `size`");
        assert_eq!(calc_size("min(wat, Min(1px))"), "invalid number `wat`");
        assert_eq!(
            ordinary("clamp(size, 10px, Min(1px))"),
            "invalid number `size`"
        );
        assert_eq!(
            calc_size("clamp(wat, 10px, Min(1px))"),
            "invalid number `wat`"
        );
    }

    #[test]
    fn fri04_c05_parser_preserves_existing_unitless_fixture_lengths() {
        assert_eq!(
            parse_preferred_size("40").expect("unitless preferred length should parse"),
            preferred_size_px(40.0)
        );
        assert_eq!(
            parse_min_size("0").expect("unitless minimum length should parse"),
            layout::MinSize::ZERO
        );
        assert_eq!(
            parse_max_track_sizing_with_calc("12")
                .expect("unitless maximum track length should parse"),
            max_track_px(12.0)
        );
    }

    #[test]
    fn parse_length_rejects_unsupported_calc_fixture_syntax() {
        let error =
            parse_length_with_calc("calc(100% / 2)").expect_err("division is not supported yet");
        assert!(
            error.to_string().contains("unsupported calc expression"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn to_node_input_lowers_calc_margin_as_affine_value() {
        let golden = Golden::parse(
            r#"
            <test name="calc-margin" use-rounding="true">
                <viewport width="200px" height="max-content" />
                <input>
                    <div display="block" margin-left="calc(10% - 4px)" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("calc margin XML should parse");

        let tree = TestTree::from_golden(&golden.root).expect("calc margin should lower");

        let layout::LengthAuto::Value(value) = tree.box_node_input(0).margin.left else {
            panic!(
                "expected affine calc margin-left, got {:?}",
                tree.box_node_input(0).margin.left
            );
        };
        assert_eq!(value.absolute_px(), -4.0);
        assert_eq!(value.percent_fraction(), 0.1);
        assert_eq!(
            tree.box_node_input(0)
                .margin
                .left
                .resolve_optional(Some(200.0)),
            Some(16.0)
        );
    }

    #[test]
    fn to_node_input_lowers_calc_grid_track_as_affine_value() {
        let golden = Golden::parse(
            r#"
            <test name="calc-grid-track" use-rounding="true">
                <viewport width="240px" height="max-content" />
                <input>
                    <div display="grid" grid-template-columns="calc(25% + 20px)" />
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("calc grid track XML should parse");

        let tree = TestTree::from_golden(&golden.root).expect("calc grid track should lower");

        let [layout::TrackComponent::Track(track)] =
            tree.box_node_input(0).grid_template_columns.as_slice()
        else {
            panic!(
                "expected one grid track, got {:?}",
                tree.box_node_input(0).grid_template_columns
            );
        };
        let layout::MinTrackSizing::Calculation(min) = &track.min else {
            panic!("expected affine calc min track, got {:?}", track.min);
        };
        let layout::MaxTrackSizing::Calculation(max) = &track.max else {
            panic!("expected affine calc max track, got {:?}", track.max);
        };
        assert_eq!(min, max);
        assert_eq!(
            min.resolve_against(layout::PercentageBasisOf::definite(240.0).unwrap())
                .value,
            Some(80.0)
        );
    }

    #[test]
    fn parse_track_component_list_accepts_rich_grid_tracks() {
        let tracks = parse_track_component_list(
            "40px minmax(20px,40px) fit-content(50%) repeat(2, 1fr auto)",
        )
        .expect("rich grid track list should parse");

        assert_eq!(tracks.len(), 4);
        assert_eq!(tracks[0], track_component_px(40.0));
        assert_eq!(
            tracks[1],
            layout::TrackComponent::minmax(min_track_px(20.0), max_track_px(40.0))
        );
        assert_eq!(
            tracks[2],
            layout::TrackComponent::fit_content(layout::SizingCalculation::value(
                layout::LengthPercentageOf::from_percent_fraction(0.5).unwrap()
            ))
        );
        assert_eq!(
            tracks[3],
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::count(
                    2,
                    vec![
                        layout::TrackSizing::flex(layout::TrackFlexFactor::try_new(1.0).unwrap()),
                        layout::TrackSizing::AUTO
                    ]
                )
                .expect("valid track repetition")
            )
        );
    }

    #[test]
    fn track_sizing_parser_accepts_role_valid_values_and_rejects_invalid_flex() {
        assert!(parse_track_sizing_with_calc("calc(20px + 25%)").is_ok());
        assert!(parse_track_sizing_with_calc("fit-content(50%)").is_ok());
        assert!(parse_track_sizing_with_calc("1fr").is_ok());
        assert!(parse_track_sizing_with_calc("minmax(10px,2fr)").is_ok());

        assert!(parse_track_sizing_with_calc("minmax(1fr,20px)").is_err());
        assert!(parse_track_sizing_with_calc("-1fr").is_err());
        assert!(parse_track_sizing_with_calc("NaNfr").is_err());
        assert!(parse_track_sizing_with_calc("inffr").is_err());
    }

    #[test]
    fn parse_track_component_list_accepts_auto_repeat() {
        assert_eq!(
            parse_track_component("repeat(auto-fill, minmax(150px,1fr))")
                .expect("auto-fill should parse"),
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::auto_fill(vec![layout::TrackSizing::minmax(
                    min_track_px(150.0),
                    layout::MaxTrackSizing::Flex(layout::TrackFlexFactor::try_new(1.0).unwrap())
                )])
                .expect("valid track repetition")
            )
        );
        assert_eq!(
            parse_track_component("repeat(auto-fit, 40px)").expect("auto-fit should parse"),
            layout::TrackComponent::Repeat(
                layout::TrackRepetition::auto_fit(vec![track_px(40.0)])
                    .expect("valid track repetition")
            )
        );
    }

    #[test]
    fn parse_track_component_list_accepts_explicit_line_names() {
        let parsed =
            parse_track_component_list("[a] 10px [b c] 20px [d]").expect("line names should parse");

        assert_eq!(
            parsed,
            vec![
                layout::TrackComponent::LineNames(vec!["a".to_string()]),
                layout::TrackComponent::Track(track_px(10.0)),
                layout::TrackComponent::LineNames(vec!["b".to_string(), "c".to_string()]),
                layout::TrackComponent::Track(track_px(20.0)),
                layout::TrackComponent::LineNames(vec!["d".to_string()]),
            ]
        );
    }

    #[test]
    fn parse_track_component_list_rejects_reserved_line_names() {
        assert!(parse_track_component_list("[auto] 10px").is_err());
        assert!(parse_track_component_list("[span] 10px").is_err());
    }

    #[test]
    fn to_node_input_preserves_named_grid_syntax() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                (
                    "grid-template-columns".to_string(),
                    "[a] 10px [b]".to_string(),
                ),
                ("grid-column-start".to_string(), "a 2".to_string()),
            ]),
        })
        .expect("named grid syntax should parse to layout input");

        assert_eq!(
            input.raw_grid_column,
            layout::RawGridPlacement::new(
                layout::RawGridLine::NamedLine {
                    name: "a".to_string(),
                    index: 2,
                },
                layout::RawGridLine::Auto,
            )
        );
        assert_eq!(
            input.grid_template_columns,
            vec![
                layout::TrackComponent::LineNames(vec!["a".to_string()]),
                layout::TrackComponent::Track(track_px(10.0)),
                layout::TrackComponent::LineNames(vec!["b".to_string()]),
            ]
        );
    }

    #[test]
    fn to_node_input_leaves_untagged_default_display_to_layout_default() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::new(),
        })
        .expect("empty attrs should parse");

        assert_eq!(input.display, layout::NodeInput::default().display);
    }

    #[test]
    fn to_node_input_applies_html_source_tag_display_defaults() {
        let div = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("source-tag".to_string(), "div".to_string())]),
        })
        .expect("source-tag div should parse");

        assert_eq!(div.display, layout::Display::Block);
    }

    #[test]
    fn source_tag_br_lowers_to_line_break_input() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "inline".to_string()),
                ("direction".to_string(), "rtl".to_string()),
                ("writing-mode".to_string(), "vertical-rl".to_string()),
                ("vertical-align".to_string(), "top".to_string()),
                ("clear".to_string(), "both".to_string()),
            ]),
        })
        .expect("source-tag br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.direction(), layout::Direction::Rtl);
        assert_eq!(input.writing_mode(), layout::WritingMode::VerticalRl);
        assert_eq!(input.vertical_align(), layout::VerticalAlign::Top);
        assert_eq!(input.clear(), layout::Clear::Both);
    }

    #[test]
    fn source_tag_br_uses_containing_inline_flow_when_descendant_style_differs() {
        let golden = Golden::parse(
            r#"
            <test name="br-containing-flow" use-rounding="true">
                <viewport width="100px" height="100px" />
                <input>
                    <div direction="rtl" writing-mode="vertical-rl">
                        <div source-tag="br" display="inline" direction="ltr" writing-mode="horizontal-tb" />
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="0" height="0">
                        <node x="0" y="0" width="0" height="0" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("fixture should parse");

        let tree = TestTree::from_golden(&golden.root).expect("test tree should build");
        let layout::LayoutInput::LineBreak(input) = tree.nodes[1].layout_input else {
            panic!("br should lower to line break");
        };

        assert_eq!(input.direction(), layout::Direction::Rtl);
        assert_eq!(input.writing_mode(), layout::WritingMode::VerticalRl);
        assert_eq!(golden.root.children[0].style.get("direction"), Some("ltr"));
        assert_eq!(
            golden.root.children[0].style.get("writing-mode"),
            Some("horizontal-tb")
        );
    }

    #[test]
    fn source_tag_br_display_inline_lowers_to_visible_line_break() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "inline".to_string()),
                ("direction".to_string(), "rtl".to_string()),
            ]),
        })
        .expect("display inline br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.display(), layout::LineBreakDisplay::Break);
        assert_eq!(input.direction(), layout::Direction::Rtl);
    }

    #[test]
    fn source_tag_br_display_none_lowers_to_hidden_line_break() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "none".to_string()),
            ]),
        })
        .expect("display none br should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.display(), layout::LineBreakDisplay::None);
    }

    #[test]
    fn source_tag_br_lowers_explicit_inline_metrics() {
        let input = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "inline".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
                ("inline-line-height".to_string(), "20px".to_string()),
            ]),
        })
        .expect("source-tag br with inline metrics should lower");

        let layout::LayoutInput::LineBreak(input) = input else {
            panic!("br should lower to line break");
        };
        assert_eq!(input.metrics().baseline(), 15.0);
        assert_eq!(input.metrics().line_extent(), 20.0);
        assert_eq!(input.metrics().after_baseline(), 5.0);
    }

    #[test]
    fn sideways_writing_mode_parses_without_normalization() {
        for (raw, expected) in [
            ("horizontal-tb", layout::WritingMode::HorizontalTb),
            ("vertical-rl", layout::WritingMode::VerticalRl),
            ("vertical-lr", layout::WritingMode::VerticalLr),
            ("sideways-rl", layout::WritingMode::SidewaysRl),
            ("sideways-lr", layout::WritingMode::SidewaysLr),
        ] {
            assert_eq!(
                parse_writing_mode(Some(raw)).expect("known writing mode should parse"),
                expected,
                "{raw} must preserve its exact layout writing mode"
            );
        }

        assert!(parse_writing_mode(Some("sideways")).is_err());
    }

    #[test]
    fn source_tag_br_rejects_partial_inline_metrics() {
        let error = to_layout_input(&StyleAttrs {
            attrs: BTreeMap::from([
                ("source-tag".to_string(), "br".to_string()),
                ("display".to_string(), "inline".to_string()),
                ("inline-baseline".to_string(), "15px".to_string()),
            ]),
        })
        .expect_err("partial inline metrics should be rejected");

        assert!(
            error.to_string().contains("inline metrics require"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn to_node_input_preserves_grid_template_areas() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "head head / nav main".to_string(),
            )]),
        })
        .expect("grid template areas should parse to layout input");

        assert_eq!(
            input.grid_template_areas,
            layout::GridTemplateAreas {
                rows: vec![
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("head".to_string()), Some("head".to_string())],
                    },
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("nav".to_string()), Some("main".to_string())],
                    },
                ],
            }
        );
    }

    #[test]
    fn to_node_input_treats_grid_template_area_dot_runs_as_null_cells() {
        let input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([(
                "grid-template-areas".to_string(),
                "... main / footer ...".to_string(),
            )]),
        })
        .expect("grid template areas should parse to layout input");

        assert_eq!(
            input.grid_template_areas,
            layout::GridTemplateAreas {
                rows: vec![
                    layout::GridTemplateAreaRow {
                        cells: vec![None, Some("main".to_string())],
                    },
                    layout::GridTemplateAreaRow {
                        cells: vec![Some("footer".to_string()), None],
                    },
                ],
            }
        );
    }

    #[test]
    fn parse_track_component_list_accepts_subgrid() {
        assert_eq!(
            parse_track_component_list("subgrid [] [start end]").expect("subgrid should parse"),
            vec![layout::TrackComponent::Subgrid(layout::SubgridTrack {
                name_components: vec![
                    layout::SubgridLineNameComponent::LineNames(Vec::new()),
                    layout::SubgridLineNameComponent::LineNames(vec![
                        "start".to_string(),
                        "end".to_string(),
                    ]),
                ],
            })]
        );
    }

    #[test]
    fn parse_display_preserves_inline_grid_variants() {
        assert_eq!(
            parse_display("inline-grid").expect("inline-grid should parse"),
            layout::Display::InlineGrid
        );
        assert_eq!(
            parse_display("inline-grid-lanes").expect("inline-grid-lanes should parse"),
            layout::Display::InlineGridLanes
        );
    }

    #[test]
    fn parse_display_preserves_inline_block() {
        assert_eq!(
            parse_display("inline-block").expect("inline-block should parse"),
            layout::Display::InlineBlock
        );
    }

    #[test]
    fn empty_inline_grid_display_uses_grid_tracks_instead_of_leaf_measurement() {
        let node_input = layout::NodeInput {
            display: layout::Display::InlineGrid,
            grid_template_columns: vec![track_component_px(40.0)],
            grid_template_rows: vec![track_component_px(20.0)],
            ..layout::NodeInput::default()
        };
        let mut tree = TestTree {
            paragraphs: Vec::new(),
            nodes: vec![TestNode {
                non_box_input: layout::NodeInput::non_box(),
                layout_input: layout::LayoutInput::box_input(node_input),
                is_br_source: false,
                font_family: FontFamily::Ahem,
                font_size: TextMeasure::LINE_HEIGHT,
                line_height: TextMeasure::LINE_HEIGHT,
                text: None,
                children: Vec::new(),
                synthetic: false,
                parent: None,
                range_root: None,
                explicit_range_root: false,
                preserve_fractional_min_content: false,
                use_tighter_monospace_wrap: false,
                cache: layout::Cache::new(),
                unrounded: layout::NodeOutput::new(),
                unrounded_present: false,
                final_layout: layout::NodeOutput::new(),
                final_layout_present: false,
                unrounded_inline_fragments: None,
                final_inline_fragments: None,
                shape_bands: None,
            }],
        };

        let request = root_request(
            layout::Size::splat(layout::Available::MaxContent),
            RootContext::Root,
        )
        .expect("root request should be valid");
        let batch =
            layout::compute_layout(&tree, 0, request).expect("inline grid layout should complete");
        tree.apply_completed_batch(&batch);

        assert_eq!(
            tree.nodes[0].final_layout.size,
            layout::Size::new(40.0, 20.0)
        );
    }

    #[test]
    fn empty_inline_grid_xml_uses_grid_tracks_instead_of_leaf_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="empty-inline-grid" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" grid-template-columns="40px" grid-template-rows="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="40" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("empty inline-grid should size from its grid tracks");
    }

    #[test]
    fn grid_text_container_uses_anonymous_text_child_for_layout() {
        let golden = Golden::parse(
            r#"
            <test name="grid-text-container" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" grid-template-columns="50px">
                        <div display="grid">hello</div>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="50" height="10">
                        <node x="0" y="0" width="50" height="10" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("grid text fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("grid container text should contribute through anonymous text child");
    }

    #[test]
    fn text_inline_grid_xml_uses_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="text-inline-grid" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <text display="inline-grid" font-size="15px">X</text>
                </input>
                <expectations>
                    <node x="0" y="0" width="15" height="15" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid text parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("text inline-grid should use fixture text measurement");
    }

    #[test]
    fn font_size_attr_scales_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="font-size-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <text font-size="12px">x</text>
                </input>
                <expectations>
                    <node x="0" y="0" width="12" height="12" />
                </expectations>
            </test>
            "#,
        )
        .expect("font-size parity fixture should parse");

        assert_surgeist_matches(&golden).expect("font-size should scale fixture text measurement");
    }

    #[test]
    fn monospace_text_measurement_wraps_at_spaces() {
        let text = TextMeasure::new(
            "The cat can not be separated from milk",
            FontFamily::Monospace,
            16.0,
            16.0,
            false,
            true,
        );

        assert_eq!(text.height_for_width(80.0), 96.0);
    }

    #[test]
    fn monospace_text_measurement_matches_chromium_subgrid_wrap_width() {
        let text = TextMeasure::new(
            "The cat can not be separated from milk",
            FontFamily::Monospace,
            16.0,
            16.0,
            false,
            true,
        );

        assert_eq!(text.height_for_width(86.0), 80.0);
    }

    #[test]
    fn monospace_min_content_keeps_browser_fractional_width() {
        let text = TextMeasure::new("Number 1", FontFamily::Monospace, 15.0, 15.0, true, true);

        assert_eq!(text.min_content_width(), 54.1875);
    }

    #[test]
    fn font_size_attr_inherits_to_fixture_text_descendants() {
        let golden = Golden::parse(
            r#"
            <test name="inherited-font-size-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div font-size="12px">
                        <text>x</text>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="12" height="12">
                        <node x="0" y="0" width="12" height="12" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("inherited font-size parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("font-size should inherit to fixture text descendants");
    }

    #[test]
    fn line_height_attr_controls_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="line-height-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-block" line-height="0px">
                        x
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="10" height="0" />
                </expectations>
            </test>
            "#,
        )
        .expect("line-height parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("line-height should affect fixture text measurement");
    }

    #[test]
    fn font_family_attr_uses_monospace_fixture_text_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="monospace-font-family-text" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div font-family="monospace" font-size="24px">
                        <text>x</text>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="14" height="24">
                        <node x="0" y="0" width="14" height="24" />
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("monospace font-family parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("font-family should select fixture text measurement");
    }

    #[test]
    fn orthogonal_nested_subgrid_text_leaf_preserves_lowered_physical_size() {
        let golden = Golden::parse(
            r#"
            <test name="orthogonal-nested-subgrid-text-sized" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" row-gap="20px" column-gap="20px" border-top="3px" border-left="3px" border-bottom="3px" border-right="3px" grid-template-rows="100px auto" grid-template-columns="100px auto">
                        <div display="grid" box-sizing="content-box" direction="ltr" writing-mode="vertical-rl" font-family="monospace" font-size="24px" line-height="24px" row-gap="100px" column-gap="100px" grid-template-rows="subgrid" grid-template-columns="subgrid" grid-row-start="span 2" grid-column-start="span 2">
                            <div display="grid" box-sizing="content-box" direction="ltr" writing-mode="horizontal-tb" font-family="monospace" font-size="24px" line-height="24px" row-gap="100px" column-gap="100px" grid-template-rows="subgrid" grid-template-columns="100px" grid-column-start="span 2">
                                <text display="block" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" width="24px" height="24px">a</text>
                                <text display="block" box-sizing="content-box" direction="ltr" font-family="monospace" font-size="24px" line-height="24px" width="24px" height="24px" grid-row-start="2">b</text>
                            </div>
                            <text display="block" box-sizing="content-box" direction="ltr" writing-mode="vertical-rl" font-family="monospace" font-size="24px" line-height="24px" width="72px" height="24px" grid-row-start="1" grid-column-start="2">ccc</text>
                        </div>
                    </div>
                </input>
                <expectations>
                    <node x="0" y="0" width="238" height="190">
                        <node>
                            <node>
                                <node />
                                <node />
                            </node>
                            <node width="72" height="24" />
                        </node>
                    </node>
                </expectations>
            </test>
            "#,
        )
        .expect("orthogonal nested subgrid text fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("text leaf should preserve its lowered physical size and writing mode");
    }

    #[test]
    fn empty_inline_grid_lanes_fixture_uses_grid_lanes_tracks_instead_of_leaf_measurement() {
        let golden = Golden::parse(
            r#"
            <test name="empty-inline-grid-lanes" use-rounding="true">
                <viewport width="max-content" height="max-content" />
                <input>
                    <div display="inline-grid-lanes" grid-template-columns="40px" grid-template-rows="20px" />
                </input>
                <expectations>
                    <node x="0" y="0" width="40" height="20" />
                </expectations>
            </test>
            "#,
        )
        .expect("inline-grid-lanes parity fixture should parse");

        assert_surgeist_matches(&golden)
            .expect("empty inline-grid-lanes should size from its grid-lanes tracks");
    }

    #[test]
    fn parse_alignment_accepts_safe_and_unsafe_prefixes() {
        assert_eq!(
            parse_align_items("safe center").expect("safe item alignment should parse"),
            layout::AlignItems::SafeCenter
        );
        assert_eq!(
            parse_align_items("baseline").expect("baseline alignment should parse"),
            layout::AlignItems::Baseline
        );
        assert_eq!(
            parse_align_items("first baseline").expect("first baseline alignment should parse"),
            layout::AlignItems::Baseline
        );
        assert_eq!(
            parse_align_items("last baseline").expect("last baseline alignment should parse"),
            layout::AlignItems::LastBaseline
        );
        assert_eq!(
            parse_align_content("unsafe end").expect("unsafe content alignment should parse"),
            layout::AlignContent::End
        );
    }

    #[test]
    fn parse_alignment_rejects_prefixed_baselines() {
        assert!(parse_align_items("safe baseline").is_err());
        assert!(parse_align_items("unsafe first baseline").is_err());
        assert!(parse_align_items("safe last baseline").is_err());
    }

    #[test]
    fn to_node_input_lowers_baseline_and_subgrid_parity_attrs() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                ("display".to_string(), "inline-grid".to_string()),
                ("align-items".to_string(), "last baseline".to_string()),
                ("align-self".to_string(), "first baseline".to_string()),
                (
                    "grid-template-columns".to_string(),
                    "subgrid [a] [b]".to_string(),
                ),
            ]),
        })
        .expect("baseline subgrid attrs should parse");

        assert_eq!(node_input.display, layout::Display::InlineGrid);
        assert_eq!(
            node_input.align_items,
            Some(layout::AlignItems::LastBaseline)
        );
        assert_eq!(node_input.align_self, Some(layout::AlignItems::Baseline));
        assert_eq!(
            node_input.grid_template_columns,
            vec![layout::TrackComponent::Subgrid(layout::SubgridTrack {
                name_components: vec![
                    layout::SubgridLineNameComponent::LineNames(vec!["a".to_string()]),
                    layout::SubgridLineNameComponent::LineNames(vec!["b".to_string()]),
                ],
            })]
        );
    }

    #[test]
    fn to_node_input_preserves_end_only_grid_placement_for_abspos() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-column-end".to_string(), "1".to_string())]),
        })
        .expect("end-only grid placement should parse");

        assert_eq!(
            node_input.grid_column,
            layout::GridPlacement::try_end_line(1).expect("valid grid line")
        );
        assert_eq!(
            node_input.raw_grid_column,
            layout::RawGridPlacement::new(layout::RawGridLine::Auto, layout::RawGridLine::Line(1))
        );
    }

    #[test]
    fn to_node_input_preserves_physical_auto_inline_margins() {
        let node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([
                ("margin-left".to_string(), "auto".to_string()),
                ("margin-right".to_string(), "12px".to_string()),
            ]),
        })
        .expect("physical margin attrs should parse");

        assert_eq!(node_input.margin.left, layout::LengthAuto::AUTO);
        assert_eq!(
            node_input.margin.right,
            length_px(12.0, "12px").unwrap().into()
        );
    }

    #[test]
    fn fixture_gaps_project_logical_css_axes() {
        let expected_physical_gap = layout::Size::new(
            length_px(7.0, "7px").expect("valid row gap"),
            length_px(11.0, "11px").expect("valid column gap"),
        );

        for (writing_mode, direction) in [
            ("vertical-rl", "ltr"),
            ("vertical-lr", "rtl"),
            ("sideways-rl", "rtl"),
            ("sideways-lr", "ltr"),
        ] {
            let node_input = test_node_input(StyleAttrs {
                attrs: BTreeMap::from([
                    ("writing-mode".to_string(), writing_mode.to_string()),
                    ("direction".to_string(), direction.to_string()),
                    ("column-gap".to_string(), "11px".to_string()),
                    ("row-gap".to_string(), "7px".to_string()),
                ]),
            })
            .expect("logical CSS gaps should parse");

            assert_eq!(
                node_input.gap, expected_physical_gap,
                "{writing_mode} {direction} must store column-gap as inline and row-gap as block"
            );
        }
    }

    #[test]
    fn parse_grid_line_accepts_negative_lines() {
        assert_eq!(
            parse_grid_line("-1").expect("negative line should parse"),
            Some(-1)
        );
        let mut span = None;
        assert_eq!(
            parse_grid_line_or_span("-3", &mut span).expect("negative end line should parse"),
            Some(-3)
        );
        assert_eq!(span, None);
    }

    #[test]
    fn parse_grid_line_accepts_named_line() {
        assert_eq!(
            parse_raw_grid_line("a").unwrap(),
            layout::RawGridLine::BareIdent("a".to_string())
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_line_with_occurrence() {
        assert_eq!(
            parse_raw_grid_line("a 8").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 8
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_integer_before_named_line() {
        assert_eq!(
            parse_raw_grid_line("2 a").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_negative_named_line_occurrence() {
        assert_eq!(
            parse_raw_grid_line("b -1").unwrap(),
            layout::RawGridLine::NamedLine {
                name: "b".to_string(),
                index: -1
            }
        );
    }

    #[test]
    fn parse_grid_line_rejects_zero_numeric_line() {
        assert!(parse_raw_grid_line("0").is_err());
    }

    #[test]
    fn parse_grid_line_accepts_named_span() {
        assert_eq!(
            parse_raw_grid_line("span a").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 1
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_count() {
        assert_eq!(
            parse_raw_grid_line("span 2 a").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_accepts_named_span_with_reversed_count_order() {
        assert_eq!(
            parse_raw_grid_line("span a 2").unwrap(),
            layout::RawGridLine::NamedSpan {
                name: "a".to_string(),
                index: 2
            }
        );
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_line_occurrence() {
        assert!(parse_raw_grid_line("a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_zero_named_span_count() {
        assert!(parse_raw_grid_line("span 0 a").is_err());
        assert!(parse_raw_grid_line("span a 0").is_err());
    }

    #[test]
    fn parse_grid_line_rejects_reserved_named_custom_ident() {
        assert!(parse_raw_grid_line("auto 1").is_err());
        assert!(parse_raw_grid_line("1 auto").is_err());
        assert!(parse_raw_grid_line("span auto").is_err());
        assert!(parse_raw_grid_line("span 1 auto").is_err());
    }

    #[test]
    fn comparison_tolerance_is_named_policy() {
        let tolerance = ComparisonTolerance::browser_parity();

        assert!(tolerance.contains(0.05));
        assert!(!tolerance.contains(0.2));
    }

    #[test]
    fn parse_grid_start_line_accepts_span() {
        let mut node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-column-start".to_string(), "span 2".to_string())]),
        })
        .expect("span start should parse");

        assert_eq!(node_input.grid_column.start(), None);
        assert_eq!(node_input.grid_column.span(), GridSpan::new(2));

        node_input = test_node_input(StyleAttrs {
            attrs: BTreeMap::from([("grid-row-start".to_string(), "span 3".to_string())]),
        })
        .expect("row span start should parse");

        assert_eq!(node_input.grid_row.start(), None);
        assert_eq!(node_input.grid_row.span(), GridSpan::new(3));
    }

    fn fri06_c06_inline_input_xml(input: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-inline-input" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input>{input}</input>
                <expectations><node /></expectations>
            </test>
            "#
        )
    }

    fn fri06_c06_segment_xml(
        id: &str,
        extent: &str,
        metrics: (&str, &str),
        bidi: &str,
        whitespace: &str,
        following_break: &str,
        replacement: Option<&str>,
    ) -> String {
        let (baseline, line_height) = metrics;
        let replacement = replacement
            .map(|value| format!(r#" replacement-inline-extent="{value}""#))
            .unwrap_or_default();
        format!(
            r#"<segment id="{id}" inline-extent="{extent}" inline-baseline="{baseline}" inline-line-height="{line_height}" bidi-level="{bidi}" whitespace-edge="{whitespace}" following-break="{following_break}"{replacement} />"#
        )
    }

    fn fri06_c06_inline_text(segments: &str) -> String {
        format!(r#"<text layout-input="inline-text">{segments}</text>"#)
    }

    fn fri06_c06_lower(input: &str) -> Result<TestTree, Error> {
        let golden = Golden::parse(&fri06_c06_inline_input_xml(input))?;
        TestTree::from_golden(&golden.root)
    }

    fn fri06_c06_valid_segment() -> String {
        fri06_c06_segment_xml(
            "11",
            "10.25",
            ("8", "10"),
            "1",
            "preserve",
            "allowed-with-replacement",
            Some("1.5"),
        )
    }

    #[test]
    fn historical_inline_schema_preserves_numeric_tuples_without_production_admission() {
        let segments = [
            fri06_c06_segment_xml(
                "11",
                "10.25",
                ("8", "10"),
                "0",
                "preserve",
                "prohibited",
                None,
            ),
            fri06_c06_segment_xml(
                "22",
                "5",
                ("7", "9"),
                "1",
                "discard-at-line-start",
                "allowed",
                None,
            ),
            fri06_c06_segment_xml(
                "33",
                "4",
                ("6", "8"),
                "2",
                "discard-at-line-end",
                "mandatory",
                None,
            ),
            fri06_c06_segment_xml(
                "44",
                "3",
                ("5", "7"),
                "3",
                "preserve",
                "allowed-with-replacement",
                Some("1.5"),
            ),
        ]
        .join("");
        let golden = Golden::parse(&fri06_c06_inline_input_xml(&format!(
            r#"<div display="block">{}</div>"#,
            fri06_c06_inline_text(&segments)
        )))
        .unwrap();
        let input = golden.root.children[0].inline_text.as_ref().unwrap();
        let error = TestTree::from_golden(&golden.root).unwrap_err();
        assert!(error.to_string().contains("historical measured-width text"));
        let segments = &input.segments;
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].id, 11);
        assert_eq!(segments[0].extent, 10.25);
        assert_eq!(segments[0].metrics.baseline(), 8.0);
        assert_eq!(segments[0].metrics.line_extent(), 10.0);
        assert_eq!(segments[1].level.get(), 1);
        assert_eq!(
            segments[1].edge,
            surgeist_text::InlineWhitespaceEdge::DiscardAtLineStart
        );
        assert_eq!(segments[2].following, MeasuredBreak::Mandatory);
        assert_eq!(
            match segments[3].following {
                MeasuredBreak::Replacement(extent) => Some(extent),
                _ => None,
            },
            Some(1.5)
        );
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_empty_and_duplicate_ids() {
        let empty = fri06_c06_valid_segment().replace("id=\"11\"", "id=\"\"");
        let error = fri06_c06_lower(&fri06_c06_inline_text(&empty))
            .expect_err("an empty segment ID must be rejected");
        assert!(error.to_string().contains("invalid `id` on `<segment>`"));

        let duplicate = format!("{}{}", fri06_c06_valid_segment(), fri06_c06_valid_segment());
        let error = fri06_c06_lower(&fri06_c06_inline_text(&duplicate))
            .expect_err("duplicate caller-local segment IDs must be rejected");
        assert!(error.to_string().contains("duplicate segment id `11`"));

        let error = fri06_c06_lower(&fri06_c06_inline_text(""))
            .expect_err("an empty shaped text input must be rejected");
        assert!(
            error
                .to_string()
                .contains("inline text requires at least one `<segment>`")
        );
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_partial_and_nonfinite_tuples() {
        for missing in [
            "id",
            "inline-extent",
            "inline-baseline",
            "inline-line-height",
            "bidi-level",
            "whitespace-edge",
            "following-break",
        ] {
            let needle = format!(r#" {missing}=""#);
            let segment = fri06_c06_valid_segment();
            let start = segment.find(&needle).expect("field exists");
            let value_start = start + needle.len();
            let value_end = value_start
                + segment[value_start..]
                    .find('"')
                    .expect("field value terminates");
            let mut partial = segment;
            partial.replace_range(start..=value_end, "");
            let error = fri06_c06_lower(&fri06_c06_inline_text(&partial))
                .expect_err("a partial shaped tuple must be rejected");
            assert_eq!(
                error.to_string(),
                format!("missing `{missing}` on `<segment>`")
            );
        }

        for (field, invalid) in [
            ("inline-extent", "NaN"),
            ("inline-baseline", "inf"),
            ("inline-line-height", "NaN"),
            ("replacement-inline-extent", "-inf"),
        ] {
            let mut segment = fri06_c06_valid_segment();
            let field_prefix = format!(r#"{field}=""#);
            let start = segment.find(&field_prefix).expect("field exists") + field_prefix.len();
            let end = start + segment[start..].find('"').expect("field value terminates");
            segment.replace_range(start..end, invalid);
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("non-finite shaped metrics must be rejected");
            assert!(error.to_string().contains(&format!("invalid `{field}`")));
        }
    }

    #[test]
    fn historical_schema_retains_resolved_bidi_126_without_claiming_native_analysis() {
        let segment = fri06_c06_valid_segment().replace("bidi-level=\"1\"", "bidi-level=\"126\"");
        let golden = Golden::parse(&fri06_c06_inline_input_xml(&format!(
            r#"<div display="block">{}</div>"#,
            fri06_c06_inline_text(&segment)
        )))
        .unwrap();
        assert_eq!(
            golden.root.children[0]
                .inline_text
                .as_ref()
                .unwrap()
                .segments[0]
                .level
                .get(),
            126
        );
        assert!(TestTree::from_golden(&golden.root).is_err());
    }

    #[test]
    fn fri06_c06_inline_input_segment_validation_rejects_out_of_domain_choices() {
        for (field, valid, invalid) in [
            ("bidi-level", "1", "127"),
            ("whitespace-edge", "preserve", "collapse"),
            ("following-break", "allowed-with-replacement", "fallback"),
        ] {
            let segment = fri06_c06_valid_segment().replace(
                &format!(r#"{field}="{valid}""#),
                &format!(r#"{field}="{invalid}""#),
            );
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("out-of-domain inline fact must be rejected");
            assert!(error.to_string().contains(&format!("invalid `{field}`")));
        }

        for segment in [
            fri06_c06_valid_segment().replace(" replacement-inline-extent=\"1.5\"", ""),
            fri06_c06_valid_segment().replace(
                "following-break=\"allowed-with-replacement\"",
                "following-break=\"allowed\"",
            ),
            fri06_c06_valid_segment().replace(
                "whitespace-edge=\"preserve\"",
                "whitespace-edge=\"discard-at-both\"",
            ),
        ] {
            let error = fri06_c06_lower(&fri06_c06_inline_text(&segment))
                .expect_err("contradictory break tuple must be rejected");
            assert!(error.to_string().contains("replacement"));
        }
    }

    #[test]
    fn fri06_c06_inline_input_schema_validation_rejects_unknown_attributes_and_payload() {
        for (shaped_text, diagnostic) in [
            (
                fri06_c06_inline_text(
                    &fri06_c06_valid_segment().replace(" />", " glyphs=\"x\" />"),
                ),
                "unsupported `<segment>` attribute `glyphs`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text" font-family="Ahem">{}</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported inline text attribute `font-family`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}authored text</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported non-whitespace text in inline text",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}<unknown /></text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported inline text child `<unknown>`",
            ),
            (
                r#"<text layout-input="inline-text"><segment id="11" inline-extent="1" inline-baseline="1" inline-line-height="1" bidi-level="0" whitespace-edge="preserve" following-break="prohibited">payload</segment></text>"#.to_string(),
                "unsupported non-whitespace text in `<segment>`",
            ),
        ] {
            let error = fri06_c06_lower(&shaped_text)
                .expect_err("unknown shaped attributes or payload must fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    #[test]
    fn fri06_c06_inline_input_schema_validation_rejects_box_children_and_measurement() {
        for (shaped_text, diagnostic) in [
            (
                format!(
                    r#"<text layout-input="inline-text" display="block">{}</text>"#,
                    fri06_c06_valid_segment()
                ),
                "inline text must not specify box attribute `display`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}<div /></text>"#,
                    fri06_c06_valid_segment()
                ),
                "inline text must not contain layout child `<div>`",
            ),
            (
                format!(
                    r#"<text layout-input="inline-text">{}measured leaf text</text>"#,
                    fri06_c06_valid_segment()
                ),
                "unsupported non-whitespace text in inline text",
            ),
        ] {
            let error = fri06_c06_lower(&shaped_text)
                .expect_err("inline text must reject contradictory box or measurement state");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_atomic_fixture(atomic_facts: &str) -> String {
        format!(
            r#"<div display="block"><div display="inline-block" width="4" height="6"/><div display="block" width="5" height="6"/>{atomic_facts}</div>"#
        )
    }

    #[test]
    fn fri06_c06_inline_input_atomic_binding_uses_exact_child_index_and_preserves_order() {
        let tree = fri06_c06_lower(&fri06_c06_atomic_fixture(
            r#"<atomic-placeholder child-index="0" bidi-level="3" following-break="mandatory" />"#,
        ))
        .expect("reviewed atomic placeholder should bind");

        assert_eq!(tree.nodes[0].children, vec![1, 2]);
        let participation = tree
            .node_input(1)
            .atomic_inline_participation
            .expect("the referenced atomic child receives participation");
        assert_eq!(participation.bidi_level().get(), 3);
        assert_eq!(
            participation.following_break(),
            layout::InlineBreakOpportunity::Mandatory
        );
        assert_eq!(tree.node_input(2).display, layout::Display::Block);
    }

    #[test]
    fn fri06_c06_inline_input_atomic_binding_rejects_unmatched_duplicate_and_invalid_facts() {
        for (atomic_facts, diagnostic) in [
            (
                r#"<atomic-placeholder child-index="2" bidi-level="0" following-break="allowed" />"#,
                "unmatched atomic child index `2`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed" /><atomic-placeholder child-index="0" bidi-level="1" following-break="mandatory" />"#,
                "duplicate atomic child index `0`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="127" following-break="allowed" />"#,
                "invalid `bidi-level` on `<atomic-placeholder>`: `127`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="fallback" />"#,
                "invalid `following-break` on `<atomic-placeholder>`: `fallback`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed-with-replacement" replacement-inline-extent="1" />"#,
                "atomic placeholder break replacement is not allowed",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" />"#,
                "missing `following-break` on `<atomic-placeholder>`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed" glyph="x" />"#,
                "unsupported `<atomic-placeholder>` attribute `glyph`",
            ),
            (
                r#"<atomic-placeholder child-index="0" bidi-level="0" following-break="allowed">payload</atomic-placeholder>"#,
                "unsupported non-whitespace text in `<atomic-placeholder>`",
            ),
        ] {
            let error = fri06_c06_lower(&fri06_c06_atomic_fixture(atomic_facts))
                .expect_err("invalid or unmatched atomic facts must fail closed");
            assert_eq!(error.to_string(), diagnostic);
        }
    }

    fn fri06_c06_inline_request() -> layout::LayoutRootRequest {
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(100.0),
            layout::Available::MaxContent,
        ))
        .expect("valid inline fixture request")
    }

    fn assert_native_cached_source_geometry(
        cold: &layout::CompletedLayoutBatch<usize>,
        warm: &layout::CompletedLayoutBatch<usize>,
        origin: &surgeist_text::PreparedOrigin,
    ) {
        assert_eq!(
            cold.unrounded_inline_fragments().len(),
            warm.unrounded_inline_fragments().len()
        );
        assert_eq!(
            cold.final_inline_fragments().len(),
            warm.final_inline_fragments().len()
        );
        for (first, second) in cold
            .unrounded_inline_fragments()
            .iter()
            .zip(warm.unrounded_inline_fragments())
            .chain(
                cold.final_inline_fragments()
                    .iter()
                    .zip(warm.final_inline_fragments()),
            )
        {
            assert_eq!(first.owner().clone(), second.owner().clone());
            let first = first.fragment();
            let second = second.fragment();
            assert_eq!(first.rect(), second.rect());
            assert_eq!(first.baseline(), second.baseline());
            assert_eq!(first.line_index(), second.line_index());
            // A fresh computation may issue new selected/finalized owners. Both
            // must retain the SAME actual prepared origin and font/source basis.
            for fragment in [first, second] {
                assert_eq!(
                    fragment.finalized_fragment().view().origin(),
                    Some(origin.clone())
                );
                assert_eq!(
                    fragment
                        .finalized_fragment()
                        .finalized_line()
                        .prepared_text(),
                    origin.prepared_text()
                );
            }
        }
    }

    #[test]
    fn fri06_c06_inline_input_fragment_cache_commits_and_restores_nonempty_slices() {
        // Two raw10px glyphs and one real internal gap(-9.75) produce10.25px.
        // The same font/LH preserves the fractional vertical cache rectangle.
        let golden = native_test_golden(&[("XX", 10.0, -9.75)], false);
        let mut tree = TestTree::from_golden(&golden.root).unwrap();
        let request = fri06_c06_inline_request();
        let cold = native_tree_compute(&tree, request);
        let origin = tree.paragraphs[0].origins[0].origin.clone();
        let assert_slices = |batch: &layout::CompletedLayoutBatch<usize>| {
            for entries in [
                batch.unrounded_inline_fragments(),
                batch.final_inline_fragments(),
            ] {
                // Selected presentation units retain their own intersections;
                // the two real source slices together cover this whole origin.
                assert_eq!(entries.len(), 2);
                let line = entries[0].fragment().finalized_fragment().finalized_line();
                let mut coverage = Vec::new();
                for (index, entry) in entries.iter().enumerate() {
                    assert_eq!(entry.owner(), &layout::InlineFragmentOwner::Node(1));
                    let fragment = entry.fragment();
                    assert_eq!(fragment.line_index(), 0);
                    let issued = fragment.finalized_fragment();
                    assert_eq!(issued.view().origin(), Some(origin.clone()));
                    assert_eq!(issued.finalized_line(), line);
                    assert_eq!(line.prepared_text(), origin.prepared_text());
                    let range = surgeist_text::Range::new(index, index + 1);
                    assert_eq!(issued.view().local_range(), Some(range));
                    assert_eq!(issued.view().aggregate_range(), Some(range));
                    coverage.push(range);
                    let rect = fragment.rect().unwrap();
                    let (x, width) = if index == 0 {
                        (0.0, 0.25)
                    } else {
                        (0.25, 10.0)
                    };
                    assert_eq!(
                        (
                            rect.origin().x,
                            rect.origin().y,
                            rect.size().width,
                            rect.size().height,
                        ),
                        (x, -0.25, width, 10.5)
                    );
                }
                assert_eq!(
                    coverage,
                    [
                        surgeist_text::Range::new(0, 1),
                        surgeist_text::Range::new(1, 2),
                    ]
                );
                assert_eq!(origin.view().local_range(), surgeist_text::Range::new(0, 2));
                assert_eq!(line.feedback().advance, 10.25);
                assert_eq!(
                    line.run_views()
                        .flat_map(|run| {
                            assert_eq!(run.face().bytes(), pinned_font::BYTES);
                            assert_eq!(run.font_size(), 10.0);
                            run.glyphs()
                                .map(|glyph| {
                                    (glyph.range(), glyph.id(), glyph.advance(), glyph.pen())
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                    [
                        (surgeist_text::Range::new(0, 1), 58, 10.0, 0.0),
                        (surgeist_text::Range::new(1, 2), 58, 10.0, 0.25),
                    ]
                );
            }
        };
        assert_slices(&cold);
        tree.apply_completed_batch(&cold);
        let stored = cold
            .unrounded_inline_fragments()
            .iter()
            .map(|entry| entry.fragment().clone())
            .collect::<Vec<_>>();
        assert_eq!(tree.unrounded_inline_fragments(1), Some(stored.as_slice()));

        let warm = native_tree_compute(&tree, request);
        assert_slices(&warm);
        assert_native_cached_source_geometry(&cold, &warm, &origin);
    }

    #[test]
    fn native_empty_origin_cache_preserves_real_occurrence_instead_of_fabricating_absence() {
        let golden = native_test_golden(&[("", 10.0, 0.0)], false);
        let mut tree = TestTree::from_golden(&golden.root).unwrap();
        let origin = tree.paragraphs[0].origins[0].origin.clone();
        let snapshot = |batch: &layout::CompletedLayoutBatch<usize>| {
            assert!(batch.unrounded_inline_fragments().is_empty());
            assert!(batch.final_inline_fragments().is_empty());
            let placed = batch.placed_text()[0].layout();
            assert!(placed.lines().is_empty());
            let receipts = placed.nonformatted_lines().cloned().collect::<Vec<_>>();
            assert_eq!(receipts.len(), 1);
            let line = receipts[0].line.finalized_line();
            assert_eq!(line.prepared_text(), origin.prepared_text());
            assert!(!line.has_in_flow_content());
            let fragments = line
                .fragment_views()
                .map(|fragment| {
                    assert_eq!(fragment.origin(), Some(origin.clone()));
                    assert_eq!(
                        fragment.local_range(),
                        Some(surgeist_text::Range::new(0, 0))
                    );
                    assert_eq!(
                        fragment.aggregate_range(),
                        Some(surgeist_text::Range::new(0, 0))
                    );
                    (
                        fragment.owner(),
                        fragment.local_range(),
                        fragment.aggregate_range(),
                        fragment.bounds(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                fragments.len(),
                1,
                "the real EmptyOrigin remains in the receipt"
            );
            (receipts, fragments)
        };
        assert_eq!(tree.unrounded_inline_fragments(1), None);
        let cold = native_tree_compute(&tree, fri06_c06_inline_request());
        let retained = snapshot(&cold);
        tree.apply_completed_batch(&cold);
        assert!(tree.unrounded_inline_fragments(1).unwrap().is_empty());
        let warm = native_tree_compute(&tree, fri06_c06_inline_request());
        assert_native_cached_source_geometry(&cold, &warm, &tree.paragraphs[0].origins[0].origin);
        assert_eq!(snapshot(&warm), retained);
    }

    fn fri06_c06_shape_input_xml(float_attrs: &str, provider: &str) -> String {
        format!(
            r#"
            <test name="fri06-c06-shape-input" use-rounding="true">
                <viewport width="100" height="max-content" />
                <input>
                    <div display="block" width="100">
                        <div width="80" height="20" {float_attrs}>
                            {provider}
                        </div>
                        <div display="block" width="30" height="10" float="left" />
                    </div>
                </input>
                <expectations><node /></expectations>
            </test>
            "#
        )
    }

    fn fri06_c06_shape_input_lower(float_attrs: &str, provider: &str) -> Result<TestTree, Error> {
        let golden = Golden::parse(&fri06_c06_shape_input_xml(float_attrs, provider))?;
        TestTree::from_golden(&golden.root)
    }

    fn fri06_c06_shape_input_query(
        band_minimum: Scalar,
        band_maximum: Scalar,
    ) -> layout::FloatExclusionQuery {
        layout::FloatExclusionQuery::try_new(
            layout::ScrollRect::try_new(layout::Point::ZERO, layout::Size::new(80.0, 20.0))
                .expect("finite test margin box"),
            layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
            band_minimum,
            band_maximum,
        )
        .expect("finite test query")
    }

    fn fri06_c06_shape_input_request() -> layout::LayoutRootRequest {
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::definite(100.0),
            layout::Available::MaxContent,
        ))
        .expect("finite shape fixture request")
    }

    #[test]
    fn fri06_c06_shape_input_bottom_alignment_lowers_exactly_without_widening() {
        let bottom = fri06_c06_shape_input_lower(r#"float="left" vertical-align="bottom""#, "")
            .expect("bottom is the reviewed finite alignment addition");
        assert_eq!(
            bottom.node_input(1).vertical_align,
            layout::VerticalAlign::Bottom
        );

        let error = fri06_c06_shape_input_lower(r#"float="left" vertical-align="middle""#, "")
            .expect_err("later-owned vertical alignment must remain rejected");
        assert_eq!(
            error.to_string(),
            "unsupported parity fixture vertical-align `middle`"
        );
    }

    #[test]
    fn fri06_c06_shape_input_valid_table_returns_empty_partial_and_full_intervals() {
        let provider = r#"
            <shape-bands>
                <shape-band band-minimum="0" band-maximum="10" />
                <shape-band band-minimum="10" band-maximum="20"
                    interval-minimum="0" interval-maximum="40" />
                <shape-band band-minimum="20" band-maximum="30"
                    interval-minimum="0" interval-maximum="80" />
            </shape-bands>
        "#;
        let tree = fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, provider)
            .expect("finite shape table lowers through production constructors");
        assert_eq!(
            tree.node_input(1).float_exclusion,
            layout::FloatExclusion::Shape
        );

        let empty = tree
            .float_exclusion_interval(1, fri06_c06_shape_input_query(0.0, 10.0))
            .expect("shape table is a provider")
            .expect("empty intersection is a successful response");
        assert_eq!(empty, None);

        for (band, expected) in [((10.0, 20.0), (0.0, 40.0)), ((20.0, 30.0), (0.0, 80.0))] {
            let interval = tree
                .float_exclusion_interval(1, fri06_c06_shape_input_query(band.0, band.1))
                .expect("shape table is a provider")
                .expect("finite interval is a successful response")
                .expect("partial and full intervals remain nonempty");
            assert_eq!((interval.minimum(), interval.maximum()), expected);
        }
    }

    #[test]
    fn fri06_c06_shape_input_schema_rejects_partial_nonfinite_duplicate_and_unknown_facts() {
        let cases = [
            (
                r#"<shape-bands><shape-band band-minimum="0" /></shape-bands>"#,
                "missing `band-maximum` on `<shape-band>`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="NaN" band-maximum="10" /></shape-bands>"#,
                "invalid shape band query",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="10" band-maximum="0" /></shape-bands>"#,
                "invalid shape band query",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="2" /></shape-bands>"#,
                "shape interval endpoints must appear together",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="8" interval-maximum="2" /></shape-bands>"#,
                "invalid shape exclusion interval",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" origin-band-minimum="1" interval-minimum="0" interval-maximum="5" /></shape-bands>"#,
                "originating shape band endpoints must appear together",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" /><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#,
                "duplicate shape query band `0..10`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" path="M0 0" /></shape-bands>"#,
                "unsupported `<shape-band>` attribute `path`",
            ),
            (
                r#"<shape-bands geometry="path"><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#,
                "unsupported `<shape-bands>` attribute `geometry`",
            ),
            (
                r#"<shape-bands><path /></shape-bands>"#,
                "unsupported `<shape-bands>` child `<path>`",
            ),
            (
                r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" provider-result="failure" interval-minimum="0" interval-maximum="5" /></shape-bands>"#,
                "provider failure must not include an exclusion interval",
            ),
        ];

        for (provider, diagnostic) in cases {
            let error =
                fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, provider)
                    .expect_err("strict finite shape schema must fail closed");
            assert!(
                error.to_string().contains(diagnostic),
                "expected `{diagnostic}`, got `{error}`"
            );
        }
    }

    #[test]
    fn fri06_c06_shape_input_table_binds_only_visible_in_flow_left_or_right_shape_float() {
        let provider =
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" /></shape-bands>"#;
        for attrs in [
            r#"float="none" float-exclusion="shape""#,
            r#"display="none" float="left" float-exclusion="shape""#,
            r#"position="absolute" float="right" float-exclusion="shape""#,
            r#"float="left""#,
        ] {
            let error = fri06_c06_shape_input_lower(attrs, provider)
                .expect_err("shape table must bind only to the reviewed shape-float role");
            assert_eq!(
                error.to_string(),
                "shape band table requires a visible in-flow left/right shape float"
            );
        }

        for side in ["left", "right"] {
            fri06_c06_shape_input_lower(
                &format!(r#"float="{side}" float-exclusion="shape""#),
                provider,
            )
            .expect("both visible in-flow shape-float sides accept a table");
        }
    }

    #[test]
    fn fri06_c06_shape_input_compute_consumes_partial_table_through_production_provider() {
        let tree = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" interval-minimum="0" interval-maximum="40" /></shape-bands>"#,
        )
        .expect("valid partial fixture response lowers");
        let batch = layout::compute_layout(&tree, 0, fri06_c06_shape_input_request())
            .expect("production compute consumes the finite provider response");
        let second_float = batch
            .final_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .expect("second float publishes output")
            .output();
        assert_eq!(second_float.location, layout::Point::new(40.0, 0.0));
    }

    #[test]
    fn fri06_c06_shape_input_compute_reports_missing_mismatch_and_provider_failure() {
        let missing = fri06_c06_shape_input_lower(r#"float="left" float-exclusion="shape""#, "")
            .expect("shape without a table represents a missing fixture provider");
        let error = layout::compute_layout(&missing, 0, fri06_c06_shape_input_request())
            .expect_err("requested missing provider must fail");
        assert_eq!(
            error.site(),
            layout::LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(
            error.operation(),
            layout::LayoutOperation::FloatExclusionQuery
        );
        assert!(matches!(
            error.kind(),
            layout::LayoutErrorKind::MissingContext(
                layout::LayoutMissingContext::FloatExclusionProvider
            )
        ));

        let mismatch = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" origin-band-minimum="10" origin-band-maximum="20" interval-minimum="0" interval-maximum="40" /></shape-bands>"#,
        )
        .expect("representable mismatched origin lowers");
        let error = layout::compute_layout(&mismatch, 0, fri06_c06_shape_input_request())
            .expect_err("production compute must reject a mismatched originating query");
        let layout::LayoutErrorKind::InvalidInput(
            layout::LayoutInvalidInput::FloatExclusionProviderOutput {
                error: layout::FloatExclusionIntervalError::QueryMismatch { expected, actual },
            },
        ) = error.kind()
        else {
            panic!("unexpected mismatch diagnostic: {error:?}");
        };
        assert_eq!(
            (expected.band_minimum(), expected.band_maximum()),
            (0.0, 10.0)
        );
        assert_eq!((actual.band_minimum(), actual.band_maximum()), (10.0, 20.0));

        let failure = fri06_c06_shape_input_lower(
            r#"float="left" float-exclusion="shape""#,
            r#"<shape-bands><shape-band band-minimum="0" band-maximum="10" provider-result="failure" /></shape-bands>"#,
        )
        .expect("representable provider failure lowers");
        let error = layout::compute_layout(&failure, 0, fri06_c06_shape_input_request())
            .expect_err("production compute must preserve fixture provider failure");
        assert_eq!(
            error.site(),
            layout::LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            }
        );
        assert_eq!(
            error.operation(),
            layout::LayoutOperation::FloatExclusionQuery
        );
        let layout::LayoutErrorKind::Measurement(provider_error) = error.kind() else {
            panic!("unexpected provider failure diagnostic: {error:?}");
        };
        assert_eq!(
            provider_error.to_string(),
            "fixture shape provider failure for query band `0..10`"
        );
    }
}

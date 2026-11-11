//! Closed browser wire data and its validated layout-owned representation.

#[path = "measurement/prepared_text.rs"]
mod prepared_text;
#[path = "measurement/style.rs"]
mod style;

#[path = "measurement/root_context.rs"]
mod root_context;
use root_context::WireViewport;

#[path = "measurement/element_scroll_sizes.rs"]
mod element_scroll_sizes;

#[path = "measurement/flex_content_text.rs"]
mod flex_content_text;
use crate::fri07_intrinsic_flex_profile;
#[path = "measurement/measured_leaf.rs"]
mod measured_leaf;
use crate::horizontal_br_font_band_profile;
use crate::original_subgrid_body_profile;
use crate::prepared_atomic_percentage_profile;
use crate::{empty_line_br_font_band_profile, vertical_br_font_band_profile};
use crate::{
    original_block_axes_profile, original_flex_axes_profile, original_flex_parent_profile,
    original_grid_axes_profile, original_subgrid_axes_profile,
};
pub(super) use flex_content_text::FlexContentText;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BoxObservationRequest {
    Geometry,
    ElementScrollSizes,
    MeasuredLeaf,
    FlexContentText,
    Fri07IntrinsicFlexText,
    PreparedRanges,
    FlexComposition,
    OriginalFlexParent,
    OriginalPercentageFlexParent,
    OriginalBlockAxes,
    OriginalFlexAxes,
    OriginalGridAxes,
    OriginalSubgridAxes,
    OriginalSubgridBody,
    PreparedAtomicPercentage,
    HorizontalBrFontBand,
    VerticalBrFontBand,
    EmptyLineBrFontBand,
}

pub(super) const FLEX_COMPOSITION_SOURCE: &str = "html/flex/fri07_flex_composition.html";
pub(super) const FLEX_COMPOSITION_SOURCE_SHA256: &str =
    "19acedc9894eed500890b91f48f9eb0e7f19a3db440f06cbaa6356e7ce73192e";
pub(super) const OBSERVATION_REQUEST_INPUT: &str = "observation-request.json";
pub(super) const MEASURED_LEAF_SCRIPT_INPUT: &str = "scripts/gentest/measured_leaf.js";
pub(super) const MEASURED_LEAF_SCRIPT: &str = include_str!("measurement/measured_leaf.js");

impl BoxObservationRequest {
    pub fn from_input_bytes(input: Option<&[u8]>) -> Result<Self, &'static str> {
        match input {
            None => Ok(Self::Geometry),
            Some(bytes) if Some(bytes) == Self::ElementScrollSizes.input_bytes() => {
                Ok(Self::ElementScrollSizes)
            }
            Some(bytes) if Some(bytes) == Self::MeasuredLeaf.input_bytes() => {
                Ok(Self::MeasuredLeaf)
            }
            Some(bytes) if Some(bytes) == Self::FlexContentText.input_bytes() => {
                Ok(Self::FlexContentText)
            }
            Some(bytes) if Some(bytes) == Self::Fri07IntrinsicFlexText.input_bytes() => {
                Ok(Self::Fri07IntrinsicFlexText)
            }
            Some(bytes) if Some(bytes) == Self::PreparedRanges.input_bytes() => {
                Ok(Self::PreparedRanges)
            }
            Some(bytes) if Some(bytes) == Self::FlexComposition.input_bytes() => {
                Ok(Self::FlexComposition)
            }
            Some(bytes) if Some(bytes) == Self::OriginalFlexParent.input_bytes() => {
                Ok(Self::OriginalFlexParent)
            }
            Some(bytes) if Some(bytes) == Self::OriginalPercentageFlexParent.input_bytes() => {
                Ok(Self::OriginalPercentageFlexParent)
            }
            Some(bytes) if Some(bytes) == Self::OriginalBlockAxes.input_bytes() => {
                Ok(Self::OriginalBlockAxes)
            }
            Some(bytes) if Some(bytes) == Self::OriginalFlexAxes.input_bytes() => {
                Ok(Self::OriginalFlexAxes)
            }
            Some(bytes) if Some(bytes) == Self::OriginalGridAxes.input_bytes() => {
                Ok(Self::OriginalGridAxes)
            }
            Some(bytes) if Some(bytes) == Self::OriginalSubgridAxes.input_bytes() => {
                Ok(Self::OriginalSubgridAxes)
            }
            Some(bytes) if Some(bytes) == Self::OriginalSubgridBody.input_bytes() => {
                Ok(Self::OriginalSubgridBody)
            }
            Some(bytes) if Some(bytes) == Self::PreparedAtomicPercentage.input_bytes() => {
                Ok(Self::PreparedAtomicPercentage)
            }
            Some(bytes) if Some(bytes) == Self::HorizontalBrFontBand.input_bytes() => {
                Ok(Self::HorizontalBrFontBand)
            }
            Some(bytes) if Some(bytes) == Self::VerticalBrFontBand.input_bytes() => {
                Ok(Self::VerticalBrFontBand)
            }
            Some(bytes) if Some(bytes) == Self::EmptyLineBrFontBand.input_bytes() => {
                Ok(Self::EmptyLineBrFontBand)
            }
            Some(_) => Err("observation request must match a registered exact input"),
        }
    }

    pub fn proof_script(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::MeasuredLeaf => Some((MEASURED_LEAF_SCRIPT_INPUT, MEASURED_LEAF_SCRIPT)),
            Self::FlexContentText => Some((
                "scripts/gentest/flex_content_text.js",
                include_str!("measurement/flex_content_text.js"),
            )),
            Self::Fri07IntrinsicFlexText => Some((
                fri07_intrinsic_flex_profile::SCRIPT_PATH,
                fri07_intrinsic_flex_profile::SCRIPT,
            )),
            Self::OriginalFlexParent => Some((
                original_flex_parent_profile::SCRIPT_PATH,
                original_flex_parent_profile::SCRIPT,
            )),
            Self::OriginalPercentageFlexParent => Some((
                original_flex_parent_profile::percentage::SCRIPT_PATH,
                original_flex_parent_profile::percentage::SCRIPT,
            )),
            Self::OriginalBlockAxes => Some((
                original_block_axes_profile::SCRIPT_PATH,
                original_block_axes_profile::SCRIPT,
            )),
            Self::OriginalFlexAxes => Some((
                original_flex_axes_profile::SCRIPT_PATH,
                original_flex_axes_profile::SCRIPT,
            )),
            Self::OriginalGridAxes => Some((
                original_grid_axes_profile::SCRIPT_PATH,
                original_grid_axes_profile::SCRIPT,
            )),
            Self::OriginalSubgridAxes => Some((
                original_subgrid_axes_profile::SCRIPT_PATH,
                original_subgrid_axes_profile::SCRIPT,
            )),
            Self::OriginalSubgridBody => Some((
                original_subgrid_body_profile::SCRIPT_PATH,
                original_subgrid_body_profile::SCRIPT,
            )),
            Self::PreparedAtomicPercentage => Some((
                prepared_atomic_percentage_profile::SCRIPT_PATH,
                prepared_atomic_percentage_profile::SCRIPT,
            )),
            Self::HorizontalBrFontBand => Some((
                horizontal_br_font_band_profile::SCRIPT_PATH,
                horizontal_br_font_band_profile::SCRIPT,
            )),
            Self::VerticalBrFontBand => Some((
                vertical_br_font_band_profile::SCRIPT_PATH,
                vertical_br_font_band_profile::SCRIPT,
            )),
            Self::EmptyLineBrFontBand => Some((
                empty_line_br_font_band_profile::SCRIPT_PATH,
                empty_line_br_font_band_profile::SCRIPT,
            )),
            Self::Geometry
            | Self::ElementScrollSizes
            | Self::PreparedRanges
            | Self::FlexComposition => None,
        }
    }

    pub fn input_bytes(self) -> Option<&'static [u8]> {
        match self {
            Self::Geometry => None,
            Self::ElementScrollSizes => Some(include_bytes!(
                "measurement/element_scroll_sizes_request.json"
            )),
            Self::PreparedRanges => Some(include_bytes!("measurement/prepared_range_request.json")),
            Self::FlexComposition => {
                Some(include_bytes!("measurement/flex_composition_request.json"))
            }
            Self::MeasuredLeaf => Some(include_bytes!("measurement/measured_leaf_request.json")),
            Self::FlexContentText => {
                Some(include_bytes!("measurement/flex_content_text_request.json"))
            }
            Self::Fri07IntrinsicFlexText => Some(fri07_intrinsic_flex_profile::REQUEST),
            Self::OriginalFlexParent => Some(original_flex_parent_profile::REQUEST),
            Self::OriginalPercentageFlexParent => {
                Some(original_flex_parent_profile::percentage::REQUEST)
            }
            Self::OriginalBlockAxes => Some(original_block_axes_profile::REQUEST),
            Self::OriginalFlexAxes => Some(original_flex_axes_profile::REQUEST),
            Self::OriginalGridAxes => Some(original_grid_axes_profile::REQUEST),
            Self::OriginalSubgridAxes => Some(original_subgrid_axes_profile::REQUEST),
            Self::OriginalSubgridBody => Some(original_subgrid_body_profile::REQUEST),
            Self::PreparedAtomicPercentage => Some(prepared_atomic_percentage_profile::REQUEST),
            Self::HorizontalBrFontBand => Some(horizontal_br_font_band_profile::REQUEST),
            Self::VerticalBrFontBand => Some(vertical_br_font_band_profile::REQUEST),
            Self::EmptyLineBrFontBand => Some(empty_line_br_font_band_profile::REQUEST),
        }
    }
}

#[derive(Debug)]
pub(super) struct ElementScrollSize {
    node_path: String,
    client_width: u32,
    client_height: u32,
    scroll_width: u32,
    scroll_height: u32,
}

impl ElementScrollSize {
    pub fn node_path(&self) -> &str {
        self.node_path.as_str()
    }
    pub fn client_width(&self) -> u32 {
        self.client_width
    }
    pub fn client_height(&self) -> u32 {
        self.client_height
    }
    pub fn scroll_width(&self) -> u32 {
        self.scroll_width
    }
    pub fn scroll_height(&self) -> u32 {
        self.scroll_height
    }
}

use serde::Deserialize;

use crate::{atomic_paragraph_metrics, grid_alignment_reference};

use std::fmt;
#[cfg(test)]
use style::Dimension;
use style::Style;

pub(super) type Attributes = Vec<(&'static str, String)>;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum MeasurementErrorKind {
    Decode,
    MissingField,
    InvalidValue,
    ContradictoryFields,
    OutOfRange,
}

#[derive(Debug)]
pub(super) struct MeasurementError {
    pub case_id: String,
    pub variant: String,
    pub node_path: String,
    pub field_path: String,
    pub kind: MeasurementErrorKind,
    detail: Box<ErrorDetail>,
}
#[derive(Debug)]
struct ErrorDetail {
    message: String,
    source: Option<serde_json::Error>,
}
impl fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}) {}.{} [{:?}]: {}",
            self.case_id,
            self.variant,
            self.node_path,
            self.field_path,
            self.kind,
            self.detail.message
        )
    }
}
impl std::error::Error for MeasurementError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.detail.source.as_ref().map(|source| source as _)
    }
}

struct Context<'a> {
    case_id: &'a str,
    variant: &'a str,
    node_path: String,
}
impl<'a> Context<'a> {
    fn error(
        &self,
        field: impl Into<String>,
        kind: MeasurementErrorKind,
        detail: impl Into<String>,
    ) -> MeasurementError {
        MeasurementError {
            case_id: self.case_id.into(),
            variant: self.variant.into(),
            node_path: self.node_path.clone(),
            field_path: field.into(),
            kind,
            detail: Box::new(ErrorDetail {
                message: detail.into(),
                source: None,
            }),
        }
    }
    fn required<T>(&self, value: Option<T>, field: &str) -> Result<T, MeasurementError> {
        value.ok_or_else(|| {
            self.error(
                field,
                MeasurementErrorKind::MissingField,
                "required measurement field is absent",
            )
        })
    }
    fn ensure(
        &self,
        valid: bool,
        field: impl Into<String>,
        detail: &str,
    ) -> Result<(), MeasurementError> {
        if valid {
            Ok(())
        } else {
            Err(self.error(field, MeasurementErrorKind::InvalidValue, detail))
        }
    }
    fn compatible(&self, valid: bool, field: &str, detail: &str) -> Result<(), MeasurementError> {
        if valid {
            Ok(())
        } else {
            Err(self.error(field, MeasurementErrorKind::ContradictoryFields, detail))
        }
    }
    fn child(&self, index: usize) -> Self {
        Context {
            case_id: self.case_id,
            variant: self.variant,
            node_path: format!("{}.children[{index}]", self.node_path),
        }
    }
}

/// Optional fields reject an explicit null unless their wire contract says otherwise.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireNode {
    #[serde(default, deserialize_with = "present")]
    tag_name: Option<String>,
    layout_input: LayoutInput,
    #[serde(default, deserialize_with = "present")]
    measured_leaf: Option<measured_leaf::WireMeasuredLeaf>,
    #[serde(default, deserialize_with = "present")]
    flex_content_text: Option<flex_content_text::WireFlexContentText>,
    #[serde(default, deserialize_with = "present")]
    fri07_intrinsic_flex_text: Option<flex_content_text::WireIntrinsicFlexText>,
    #[serde(default, deserialize_with = "present")]
    prepared_range_frame: Option<PreparedRangeFrame>,
    #[serde(default, deserialize_with = "present")]
    flex_composition_image: Option<FlexCompositionImage>,
    #[serde(default, deserialize_with = "present")]
    unsupported_reason: Option<String>,
    #[serde(default, deserialize_with = "present")]
    style: Option<Style>,
    #[serde(default, deserialize_with = "present")]
    use_rounding: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    viewport: Option<WireViewport>,
    #[serde(default, deserialize_with = "present")]
    layout_ready_inline_root: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    layout_ready_anonymous_grid_text_wrapper: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    text_content: Option<String>,
    #[serde(default, deserialize_with = "present")]
    unrounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    smart_rounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    world_rounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    naively_rounded_layout: Option<WireGeometry>,
    #[serde(default, deserialize_with = "present")]
    inline_segments: Option<Vec<WireSegment>>,
    #[serde(default, deserialize_with = "present")]
    prepared_paragraph: Option<prepared_text::Paragraph>,
    #[serde(default, deserialize_with = "present")]
    atomic_paragraph_metrics: Option<atomic_paragraph_metrics::Capture>,
    #[serde(default, deserialize_with = "present")]
    grid_alignment_profile: Option<grid_alignment_reference::Profile>,
    #[serde(default, deserialize_with = "present")]
    grid_alignment_reference: Option<grid_alignment_reference::Capture>,
    #[serde(default, deserialize_with = "present")]
    prepared_occurrence: Option<usize>,
    #[serde(default, deserialize_with = "present")]
    inline_boundary: Option<WireBoundary>,
    #[serde(default, deserialize_with = "present")]
    line_control_participation: Option<WireControl>,
    #[serde(default, deserialize_with = "present")]
    browser_br_observation: Option<BrowserBrObservation>,
    #[serde(default, deserialize_with = "present")]
    atomic_inline_participation: Option<WireAtomic>,
    #[serde(default, deserialize_with = "present")]
    shape_bands: Option<Vec<WireBand>>,
    #[serde(default, deserialize_with = "present")]
    fragments: Option<Vec<WireFragment>>,
    #[serde(default, deserialize_with = "present")]
    #[serde(rename = "rangeInks")]
    range_rectangles: Option<Vec<WireRange>>,
    #[serde(default, deserialize_with = "present")]
    range_fragments: Option<Vec<WireRangeFragment>>,
    #[serde(default, deserialize_with = "present")]
    children: Option<Vec<WireNode>>,
}
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FlexCompositionImage {
    pub(super) natural_width: u32,
    pub(super) natural_height: u32,
}

#[derive(Deserialize)]
enum PreparedRangeFrame {
    #[serde(rename = "prepared-range-root-border-v1")]
    RootBorder,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum LayoutInput {
    Box,
    InlineText,
    PreparedText,
    InlineBoundary,
    Unsupported,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BrowserBrObservation {
    FontBand,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireGeometry {
    #[serde(default, deserialize_with = "present")]
    x: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    y: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    height: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    scroll_width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    scroll_height: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    client_width: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    client_height: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireBoundary {
    kind: BoundaryKind,
    #[serde(default, deserialize_with = "present")]
    baseline: Option<f64>,
    #[serde(default, deserialize_with = "present")]
    line_height: Option<f64>,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum BoundaryKind {
    Start,
    End,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireControl {
    kind: ControlKind,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ControlKind {
    ForcedBreak,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireSegment {
    id: u64,
    inline_extent: f64,
    inline_baseline: f64,
    inline_line_height: f64,
    bidi_level: u64,
    whitespace_edge: WhitespaceEdge,
    following_break: BreakKind,
    #[serde(default, deserialize_with = "present")]
    replacement_inline_extent: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireAtomic {
    bidi_level: u64,
    following_break: BreakKind,
    #[serde(default, deserialize_with = "present")]
    replacement_inline_extent: Option<f64>,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum WhitespaceEdge {
    Preserve,
    DiscardAtStart,
    DiscardAtEnd,
    DiscardAtBoth,
}
#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum BreakKind {
    Prohibited,
    Allowed,
    Mandatory,
    AllowedWithReplacement,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireBand {
    band_minimum: f64,
    band_maximum: f64,
    #[serde(default, deserialize_with = "present")]
    interval: Option<WireInterval>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireInterval {
    minimum: f64,
    maximum: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireFragment {
    source_segment_id: u64,
    line_index: u64,
    visual_index: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    baseline_x: f64,
    baseline_y: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireRange {
    source_segment_id: u64,
    line_index: u64,
    physical_start_edge: PhysicalEdge,
    start: f64,
    advance: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireRangeFragment {
    origin: u64,
    line: u64,
    physical_start_edge: PhysicalEdge,
    start: f64,
    advance: f64,
}
impl WireRangeFragment {
    fn rectangle_fields(self) -> WireRange {
        WireRange {
            source_segment_id: self.origin,
            line_index: self.line,
            physical_start_edge: self.physical_start_edge,
            start: self.start,
            advance: self.advance,
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PhysicalEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug)]
pub(super) enum DecodedMeasurement {
    Unsupported { reason: String },
    Supported(ValidatedMeasurement),
}
#[derive(Debug)]
pub(super) struct ValidatedMeasurement {
    use_rounding: bool,
    viewport: Attributes,
    root_local_positioned: bool,
    root: Box<Node>,
    element_scroll_sizes: Option<Vec<ElementScrollSize>>,
    measured_leaf: Option<Attributes>,
    flex_content_text: Option<FlexContentText>,
    prepared_range_frame: bool,
    flex_composition_image: Option<FlexCompositionImage>,
}
#[derive(Debug)]
pub(super) struct Node {
    kind: NodeKind,
    prepared_paragraph: Option<String>,
    attrs: Attributes,
    children: Vec<Node>,
    text: Option<String>,
    observation: Observation,
    world_rounded_layout: Option<Attributes>,
    atomic: Option<Attributes>,
    bands: Option<Vec<Attributes>>,
    control_expectation: Option<Attributes>,
    br_font_band: Option<Geometry>,
}
#[derive(Debug)]
pub(super) enum NodeKind {
    Box,
    Text,
    InlineText(Vec<Attributes>),
    PreparedText(usize),
    Boundary(Attributes),
    Control,
}
#[derive(Debug)]
pub(super) enum Observation {
    Geometry {
        unrounded: Geometry,
        selected: Geometry,
        scroll: Option<(f64, f64)>,
        fragments: Option<Vec<Attributes>>,
    },
    RangeRectangles(Vec<Attributes>),
    RangeFragments(Vec<Attributes>),
    Boundary,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Geometry {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub(super) fn decode(
    raw: &str,
    case_id: &str,
    variant: &str,
    request: BoxObservationRequest,
) -> Result<DecodedMeasurement, MeasurementError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let wire: WireNode = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
        let (node_path, field_path) = split_decode_path(&error.path().to_string());
        let source = error.into_inner();
        MeasurementError {
            case_id: case_id.into(),
            variant: variant.into(),
            node_path,
            field_path,
            kind: MeasurementErrorKind::Decode,
            detail: Box::new(ErrorDetail {
                message: source.to_string(),
                source: Some(source),
            }),
        }
    })?;
    deserializer.end().map_err(|source| MeasurementError {
        case_id: case_id.into(),
        variant: variant.into(),
        node_path: "root".into(),
        field_path: String::new(),
        kind: MeasurementErrorKind::Decode,
        detail: Box::new(ErrorDetail {
            message: source.to_string(),
            source: Some(source),
        }),
    })?;
    let context = Context {
        case_id,
        variant,
        node_path: "root".into(),
    };
    wire.validate_wire(&context)?;
    let prepared_atomic_percentage = request == BoxObservationRequest::PreparedAtomicPercentage;
    context.compatible(
        prepared_atomic_percentage || !wire.has_authored_atomic_size(),
        "preparedParagraph.atomics",
        "authored atomic sizes require their registered capture request",
    )?;
    context.compatible(
        request == BoxObservationRequest::MeasuredLeaf || wire.measured_leaf.is_none(),
        "measuredLeaf",
        "measured leaf evidence requires its registered request",
    )?;
    context.compatible(
        request == BoxObservationRequest::FlexContentText || wire.flex_content_text.is_none(),
        "flexContentText",
        "Flex content evidence requires its registered request",
    )?;
    context.compatible(
        request == BoxObservationRequest::Fri07IntrinsicFlexText
            || wire.fri07_intrinsic_flex_text.is_none(),
        "fri07IntrinsicFlexText",
        "intrinsic Flex evidence requires its registered request",
    )?;
    let prepared_range_frame =
        request == BoxObservationRequest::PreparedRanges || prepared_atomic_percentage;
    context.compatible(
        prepared_range_frame || wire.prepared_range_frame.is_none(),
        "preparedRangeFrame",
        "prepared Range evidence requires its registered request",
    )?;
    context.compatible(
        request == BoxObservationRequest::FlexComposition || wire.flex_composition_image.is_none(),
        "flexCompositionImage",
        "Flex composition image evidence requires its registered request",
    )?;
    if let Some(reason) = wire.unsupported_reason() {
        return Ok(DecodedMeasurement::Unsupported {
            reason: reason.into(),
        });
    }
    let use_rounding = context.required(wire.use_rounding, "useRounding")?;
    if let Some(viewport) = &wire.viewport {
        viewport.validate_root_style(wire.style.as_ref(), &context)?;
    }
    let root_local_positioned = wire
        .viewport
        .as_ref()
        .is_some_and(WireViewport::root_local_positioned);
    let viewport = wire
        .viewport
        .as_ref()
        .ok_or_else(|| {
            context.error(
                "viewport",
                MeasurementErrorKind::MissingField,
                "root requires viewport",
            )
        })?
        .validate(&context)?;
    wire.validate_grid_alignment_profile(&context)?;
    let block_offset = wire
        .viewport
        .as_ref()
        .and_then(WireViewport::hypothetical_block_offset);
    let requires_block_offset = matches!(
        request,
        BoxObservationRequest::VerticalBrFontBand
            | BoxObservationRequest::OriginalBlockAxes
            | BoxObservationRequest::OriginalFlexAxes
            | BoxObservationRequest::OriginalGridAxes
            | BoxObservationRequest::OriginalSubgridAxes
    ) || (request == BoxObservationRequest::EmptyLineBrFontBand
        && wire
            .style
            .as_ref()
            .and_then(|style| style.writing_mode.as_deref())
            == Some("vertical-rl"));
    let valid_block_offset = block_offset == requires_block_offset.then_some(0.0);
    context.compatible(
        valid_block_offset,
        "viewport.rootContext.hypotheticalBlockOffset",
        "explicit zero static block offset must match a qualified first-block profile",
    )?;
    if request == BoxObservationRequest::HorizontalBrFontBand {
        wire.validate_horizontal_br_font_band(&context, root_local_positioned)?;
    }
    if request == BoxObservationRequest::VerticalBrFontBand {
        wire.validate_vertical_br_font_band(&context, root_local_positioned)?;
    }
    if request == BoxObservationRequest::EmptyLineBrFontBand {
        wire.validate_empty_line_br_font_band(&context, root_local_positioned)?;
    } else {
        context.compatible(
            !wire.uses_stylesheet_break_font(),
            "preparedParagraph.font",
            "stylesheet break font requires the original empty-line observation profile",
        )?;
    }
    if request == BoxObservationRequest::OriginalFlexParent {
        context.compatible(
            root_local_positioned && use_rounding,
            "viewport",
            "original Flex parent requires an absolute root-local observation",
        )?;
        wire.validate_original_flex_parent(&context)?;
    }
    if request == BoxObservationRequest::OriginalPercentageFlexParent {
        context.compatible(
            root_local_positioned && use_rounding,
            "viewport",
            "original percentage Flex parent requires an absolute root-local observation",
        )?;
        wire.validate_original_percentage_flex_parent(&context)?;
    }
    if request == BoxObservationRequest::OriginalSubgridBody {
        context.compatible(
            !root_local_positioned && use_rounding,
            "viewport",
            "original subgrid requires the normal in-flow body frame",
        )?;
        wire.validate_original_subgrid_body(&context)?;
    }
    if prepared_range_frame {
        context.required(wire.prepared_range_frame.as_ref(), "preparedRangeFrame")?;
        context.compatible(
            root_local_positioned
                && wire.layout_ready_inline_root == Some(true)
                && wire.prepared_paragraph.is_some(),
            "preparedRangeFrame",
            "prepared Range evidence requires an absolute root-local prepared inline root",
        )?;
        wire.validate_prepared_range_rows(&context)?;
    }
    if prepared_atomic_percentage {
        wire.validate_prepared_atomic_percentage(&context)?;
    }
    let flex_composition_image = if request == BoxObservationRequest::FlexComposition {
        let image = context.required(wire.flex_composition_image, "flexCompositionImage")?;
        context.compatible(
            root_local_positioned
                && wire.style.as_ref().is_some_and(|style| {
                    style.display.as_deref() == Some("flex")
                        && style.writing_mode.as_deref() == Some("sideways-rl")
                })
                && (image.natural_width, image.natural_height) == (80, 40)
                && wire.children.as_deref().is_some_and(|children| {
                    let [_, child, _, _] = children else {
                        return false;
                    };
                    child.layout_input == LayoutInput::Box
                        && child.tag_name.as_deref() == Some("img")
                        && child.children.as_ref().is_some_and(Vec::is_empty)
                }),
            "flexCompositionImage",
            "Flex composition requires the source-bound replaced image in its root frame",
        )?;
        Some(image)
    } else {
        None
    };
    let element_scroll_sizes = match request {
        BoxObservationRequest::Geometry
        | BoxObservationRequest::Fri07IntrinsicFlexText
        | BoxObservationRequest::PreparedRanges
        | BoxObservationRequest::FlexComposition
        | BoxObservationRequest::OriginalFlexParent
        | BoxObservationRequest::OriginalPercentageFlexParent
        | BoxObservationRequest::OriginalBlockAxes
        | BoxObservationRequest::OriginalFlexAxes
        | BoxObservationRequest::OriginalGridAxes
        | BoxObservationRequest::OriginalSubgridAxes
        | BoxObservationRequest::OriginalSubgridBody
        | BoxObservationRequest::PreparedAtomicPercentage
        | BoxObservationRequest::HorizontalBrFontBand
        | BoxObservationRequest::VerticalBrFontBand
        | BoxObservationRequest::EmptyLineBrFontBand => None,
        BoxObservationRequest::ElementScrollSizes
        | BoxObservationRequest::MeasuredLeaf
        | BoxObservationRequest::FlexContentText => {
            if !root_local_positioned {
                return Ok(DecodedMeasurement::Unsupported {
                    reason: "root: element scroll sizes require an authenticated absolute-root-local capture".into(),
                });
            }
            if let Some(reason) = element_scroll_sizes::unsupported_reason(&wire, &context)? {
                return Ok(DecodedMeasurement::Unsupported { reason });
            }
            Some(element_scroll_sizes::collect(
                &wire,
                use_rounding,
                &context,
            )?)
        }
    };
    let measured_leaf = if request == BoxObservationRequest::MeasuredLeaf {
        let witness = context.required(wire.measured_leaf.as_ref(), "measuredLeaf")?;
        Some(witness.validate(
            &wire,
            element_scroll_sizes.as_deref().unwrap_or_default(),
            &context,
        )?)
    } else {
        None
    };
    let flex_content_text = match request {
        BoxObservationRequest::FlexContentText => {
            let witness = context.required(wire.flex_content_text.as_ref(), "flexContentText")?;
            Some(witness.validate(&wire, &context)?)
        }
        BoxObservationRequest::Fri07IntrinsicFlexText => {
            context.compatible(
                root_local_positioned && use_rounding,
                "viewport",
                "intrinsic Flex text requires the original absolute root-local frame",
            )?;
            let witness = context.required(
                wire.fri07_intrinsic_flex_text.as_ref(),
                "fri07IntrinsicFlexText",
            )?;
            Some(witness.validate(&wire, &context)?)
        }
        _ => None,
    };
    let root = wire.validate(&context, "horizontal-tb", use_rounding, true)?;
    if matches!(
        request,
        BoxObservationRequest::OriginalBlockAxes
            | BoxObservationRequest::OriginalFlexAxes
            | BoxObservationRequest::OriginalGridAxes
            | BoxObservationRequest::OriginalSubgridAxes
    ) {
        context.compatible(
            root_local_positioned
                && use_rounding
                && ["containing-direction", "hypothetical-direction"]
                    .into_iter()
                    .all(|key| has_attribute(&viewport, key, "ltr"))
                && has_attribute(&viewport, "hypothetical-inline-offset", "0px"),
            "viewport",
            "original axes require their first-block root-local frame",
        )?;
        match request {
            BoxObservationRequest::OriginalGridAxes => {
                validate_original_grid_axes(&root, &context)?
            }
            BoxObservationRequest::OriginalSubgridAxes => {
                validate_original_subgrid_axes(&root, &context)?
            }
            _ => validate_original_axes(&root, &context, request)?,
        }
    }
    if root_local_positioned {
        let Observation::Geometry {
            unrounded,
            selected,
            ..
        } = root.observation()
        else {
            return Err(context.error(
                "unroundedLayout",
                MeasurementErrorKind::ContradictoryFields,
                "absolute root-local capture requires a root box",
            ));
        };
        context.compatible(
            unrounded.x == 0.0 && unrounded.y == 0.0 && selected.x == 0.0 && selected.y == 0.0,
            "unroundedLayout",
            "absolute root-local observations require root origin (0, 0)",
        )?;
    }
    Ok(DecodedMeasurement::Supported(ValidatedMeasurement {
        use_rounding,
        viewport,
        root_local_positioned,
        root: Box::new(root),
        element_scroll_sizes,
        measured_leaf,
        flex_content_text,
        prepared_range_frame,
        flex_composition_image,
    }))
}

fn has_attribute(attrs: &Attributes, name: &str, value: &str) -> bool {
    attrs
        .iter()
        .any(|(key, actual)| *key == name && actual == value)
}

fn validate_original_axes(
    root: &Node,
    context: &Context<'_>,
    request: BoxObservationRequest,
) -> Result<(), MeasurementError> {
    let (root_display, root_width, root_height, second_height) = match request {
        BoxObservationRequest::OriginalBlockAxes => ("block", "100px", "100px", "30px"),
        BoxObservationRequest::OriginalFlexAxes => ("flex", "180px", "140px", "35px"),
        _ => unreachable!("only the original axis profiles validate their fixed box trees"),
    };
    let [first, second] = root.children.as_slice() else {
        return Err(context.error(
            "children",
            MeasurementErrorKind::ContradictoryFields,
            "original axes require two source children",
        ));
    };
    for (index, (node, display, width, height)) in [
        (root, root_display, root_width, root_height),
        (first, "block", "30px", "20px"),
        (second, "block", "20px", second_height),
    ]
    .into_iter()
    .enumerate()
    {
        let place = if index == 0 {
            Context {
                node_path: context.node_path.clone(),
                ..*context
            }
        } else {
            context.child(index - 1)
        };
        place.compatible(
            matches!(node.kind, NodeKind::Box)
                && node.text.is_none()
                && (index == 0 || node.children.is_empty())
                && has_attribute(&node.attrs, "source-tag", "div")
                && has_attribute(&node.attrs, "display", display)
                && has_attribute(&node.attrs, "width", width)
                && has_attribute(&node.attrs, "height", height),
            "style",
            "original axes require fixed source boxes without text",
        )?;
    }
    if request == BoxObservationRequest::OriginalFlexAxes {
        context.compatible(
            has_attribute(&root.attrs, "align-items", "flex-end")
                && has_attribute(&root.attrs, "justify-content", "space-between"),
            "style",
            "original Flex axes require their source alignment",
        )?;
    }
    Ok(())
}

fn validate_original_grid_axes(root: &Node, context: &Context<'_>) -> Result<(), MeasurementError> {
    let [first, second] = root.children.as_slice() else {
        return Err(context.error(
            "children",
            MeasurementErrorKind::ContradictoryFields,
            "original Grid axes require two source children",
        ));
    };
    context.compatible(
        auto_source_div(root)
            && has_attribute(&root.attrs, "display", "grid")
            && has_attribute(&root.attrs, "position", "absolute")
            && has_attribute(&root.attrs, "grid-template-columns", "30px 40px")
            && has_attribute(&root.attrs, "grid-template-rows", "50px 60px"),
        "style",
        "original Grid axes require an absolute Auto root with fixed authored tracks",
    )?;
    for (index, (child, start, end)) in [(first, "1", "2"), (second, "2", "3")]
        .into_iter()
        .enumerate()
    {
        let place = context.child(index);
        place.compatible(
            relative_auto_axis_box(child, "flex")
                && child.children.is_empty()
                && diagonal_grid_placement(child, start, end),
            "style",
            "original Grid axes require authored diagonal child placement",
        )?;
    }
    Ok(())
}

fn auto_source_div(node: &Node) -> bool {
    matches!(node.kind, NodeKind::Box)
        && node.text.is_none()
        && has_attribute(&node.attrs, "source-tag", "div")
        && node
            .attrs
            .iter()
            .all(|(name, value)| !matches!(*name, "width" | "height") || value == "auto")
}

fn relative_auto_axis_box(node: &Node, display: &str) -> bool {
    // The normalized attributes omit the base relative position.
    auto_source_div(node)
        && has_attribute(&node.attrs, "display", display)
        && node.attrs.iter().all(|(name, _)| *name != "position")
}

fn diagonal_grid_placement(node: &Node, start: &str, end: &str) -> bool {
    [
        ("grid-column-start", start),
        ("grid-column-end", end),
        ("grid-row-start", start),
        ("grid-row-end", end),
    ]
    .into_iter()
    .all(|(key, value)| has_attribute(&node.attrs, key, value))
}

fn validate_original_subgrid_axes(
    root: &Node,
    context: &Context<'_>,
) -> Result<(), MeasurementError> {
    let [columns, rows] = root.children.as_slice() else {
        return Err(context.error(
            "children",
            MeasurementErrorKind::ContradictoryFields,
            "original Subgrid axes require two parent grids",
        ));
    };
    context.compatible(
        auto_source_div(root)
            && has_attribute(&root.attrs, "display", "block")
            && has_attribute(&root.attrs, "position", "absolute")
            && root
                .attrs
                .iter()
                .all(|(name, value)| *name != "writing-mode" || value == "horizontal-tb"),
        "style",
        "original Subgrid axes require a horizontal absolute Auto Block",
    )?;
    let authored_grid = |node: &Node, columns, rows| {
        relative_auto_axis_box(node, "grid")
            && [
                ("grid-template-columns", columns),
                ("grid-template-rows", rows),
                ("row-gap", "7px"),
                ("column-gap", "11px"),
            ]
            .into_iter()
            .all(|(key, value)| has_attribute(&node.attrs, key, value))
    };
    for (index, (parent, columns, rows)) in [
        (columns, "subgrid", "50px 60px"),
        (rows, "30px 40px", "subgrid"),
    ]
    .into_iter()
    .enumerate()
    {
        let parent_context = context.child(index);
        parent_context.compatible(
            authored_grid(parent, "30px 40px", "50px 60px"),
            "style",
            "original Subgrid axes require fixed parent tracks and gaps",
        )?;
        let [subgrid] = parent.children.as_slice() else {
            return Err(parent_context.error(
                "children",
                MeasurementErrorKind::ContradictoryFields,
                "original Subgrid parent requires one subgrid",
            ));
        };
        let subgrid_context = parent_context.child(0);
        subgrid_context.compatible(
            authored_grid(subgrid, columns, rows) && diagonal_grid_placement(subgrid, "1", "3"),
            "style",
            "original Subgrid requires its authored inherited axis and full span",
        )?;
        let [first, second] = subgrid.children.as_slice() else {
            return Err(subgrid_context.error(
                "children",
                MeasurementErrorKind::ContradictoryFields,
                "original Subgrid requires two source cells",
            ));
        };
        for (index, (cell, start, end)) in [(first, "1", "2"), (second, "2", "3")]
            .into_iter()
            .enumerate()
        {
            subgrid_context.child(index).compatible(
                relative_auto_axis_box(cell, "flex")
                    && cell.children.is_empty()
                    && diagonal_grid_placement(cell, start, end),
                "style",
                "original Subgrid requires authored diagonal cells",
            )?;
        }
    }
    Ok(())
}

pub(super) fn number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
fn finite(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    context.ensure(value.is_finite(), field, "number must be finite")?;
    Ok(value)
}
fn nonnegative(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    finite(value, context, field)?;
    context.ensure(value >= 0.0, field, "extent must be nonnegative")?;
    Ok(value)
}
fn layout_number(value: f64, context: &Context<'_>, field: &str) -> Result<f64, MeasurementError> {
    finite(value, context, field)?;
    if (value as f32).is_finite() {
        Ok(value)
    } else {
        Err(context.error(
            field,
            MeasurementErrorKind::OutOfRange,
            "geometry must be representable as f32",
        ))
    }
}
fn bidi(value: u64, context: &Context<'_>, field: &str) -> Result<String, MeasurementError> {
    if value <= 126 {
        Ok(value.to_string())
    } else {
        Err(context.error(
            field,
            MeasurementErrorKind::OutOfRange,
            "resolved bidi level exceeds 126",
        ))
    }
}

impl WireNode {
    fn unsupported_reason(&self) -> Option<&str> {
        self.unsupported_reason.as_deref().or_else(|| {
            self.children
                .as_deref()
                .unwrap_or_default()
                .iter()
                .find_map(Self::unsupported_reason)
        })
    }
    fn validate_grid_alignment_profile(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        fn visit(
            node: &WireNode,
            path: String,
            in_region: bool,
            demanded: &mut Vec<String>,
            context: &Context<'_>,
        ) -> Result<(), MeasurementError> {
            context.compatible(
                path == "/" || node.grid_alignment_profile.is_none(),
                "gridAlignmentProfile",
                "Grid reference profile belongs only to the source root",
            )?;
            let in_region = in_region || node.layout_ready_inline_root == Some(true);
            // This predicate demands complete own typography proof; it does not
            // derive a baseline name. Raw Style owns its normalized own mode.
            let needs_reference = in_region
                && node.layout_input == LayoutInput::Box
                && node.style.as_ref().is_some_and(|style| {
                    matches!(style.display.as_deref(), Some("grid" | "inline-grid"))
                        && matches!(style.writing_mode(), "vertical-lr" | "vertical-rl")
                });
            context.compatible(
                node.grid_alignment_reference.is_some() == needs_reference,
                "gridAlignmentReference",
                "Grid record coverage disagrees with actual layout-ready region, own box role or writing mode",
            )?;
            if needs_reference {
                demanded.push(path.clone());
            }
            for (index, child) in node.children.iter().flatten().enumerate() {
                let next = if path == "/" {
                    format!("/{index}")
                } else {
                    format!("{path}/{index}")
                };
                visit(child, next, in_region, demanded, &context.child(index))?;
            }
            Ok(())
        }
        let profile =
            context.required(self.grid_alignment_profile.as_ref(), "gridAlignmentProfile")?;
        let mut demanded = Vec::new();
        visit(self, "/".into(), false, &mut demanded, context)?;
        profile.validate(&demanded).map_err(|detail| {
            context.error(
                "gridAlignmentProfile",
                MeasurementErrorKind::InvalidValue,
                detail,
            )
        })
    }
    fn validate_prepared_range_rows(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        if let Some(occurrence) = self.prepared_occurrence {
            let rows = context.required(self.range_fragments.as_ref(), "rangeFragments")?;
            context.compatible(
                !rows.is_empty()
                    && rows.iter().all(|row| {
                        usize::try_from(row.origin) == Ok(occurrence)
                            && matches!(row.physical_start_edge, PhysicalEdge::Left)
                    }),
                "rangeFragments",
                "prepared Range rows require their owning occurrence and the physical left edge",
            )?;
        }
        for (index, child) in self.children.iter().flatten().enumerate() {
            child.validate_prepared_range_rows(&context.child(index))?;
        }
        Ok(())
    }

    fn validate_vertical_br_font_band(
        &self,
        context: &Context<'_>,
        root_local_positioned: bool,
    ) -> Result<(), MeasurementError> {
        let prepared = context.required(self.prepared_paragraph.as_ref(), "preparedParagraph")?;
        let children = context.required(self.children.as_deref(), "children")?;
        context.compatible(
            root_local_positioned
                && self.layout_ready_inline_root == Some(true)
                && self.style.as_ref().is_some_and(|style| {
                    style.display.as_deref() == Some("block")
                        && matches!(style.writing_mode.as_deref(), Some("vertical-lr" | "vertical-rl"))
                })
                && prepared.origins.len() == 2
                && prepared.atomics.len() == 2
                && prepared.controls.len() == 1
                && prepared.controls[0].child == 2
                && matches!(children, [_, _, br, _, _] if br.browser_br_observation.is_some())
                && children.iter().filter(|child| child.browser_br_observation.is_some()).count() == 1,
            "preparedParagraph",
            "vertical break font-band evidence requires the original root, prepared source and selected break",
        )
    }

    fn uses_stylesheet_break_font(&self) -> bool {
        self.prepared_paragraph
            .as_ref()
            .is_some_and(prepared_text::Paragraph::uses_stylesheet_ahem)
            || self
                .children
                .iter()
                .flatten()
                .any(Self::uses_stylesheet_break_font)
    }

    fn validate_empty_line_br_font_band(
        &self,
        context: &Context<'_>,
        root_local_positioned: bool,
    ) -> Result<(), MeasurementError> {
        let prepared = context.required(self.prepared_paragraph.as_ref(), "preparedParagraph")?;
        let children = context.required(self.children.as_deref(), "children")?;
        context.compatible(
            root_local_positioned
                && self.layout_ready_inline_root == Some(true)
                && self.style.as_ref().is_some_and(|style| {
                    style.display.as_deref() == Some("block")
                        && matches!(style.writing_mode.as_deref(), None | Some("horizontal-tb" | "vertical-rl"))
                })
                && prepared.is_empty_line_source()
                && matches!(children, [first, second]
                    if first.browser_br_observation.is_some() && second.browser_br_observation.is_some()
                    && !first.uses_stylesheet_break_font() && !second.uses_stylesheet_break_font()),
            "preparedParagraph",
            "empty-line font bands require the original root and both prepared breaks",
        )
    }

    fn validate_horizontal_br_font_band(
        &self,
        context: &Context<'_>,
        root_local_positioned: bool,
    ) -> Result<(), MeasurementError> {
        let prepared = context.required(self.prepared_paragraph.as_ref(), "preparedParagraph")?;
        let children = context.required(self.children.as_deref(), "children")?;
        context.compatible(
            root_local_positioned
                && self.layout_ready_inline_root == Some(true)
                && self.style.as_ref().is_some_and(|style| {
                    style.display.as_deref() == Some("block")
                        && style.writing_mode.as_deref().unwrap_or("horizontal-tb") == "horizontal-tb"
                })
                && prepared.origins.is_empty()
                && prepared.atomics.len() == 2
                && prepared.controls.len() == 1
                && prepared.controls[0].child == 1
                && matches!(children, [before, br, after]
                    if before.browser_br_observation.is_none()
                        && br.browser_br_observation.is_some()
                        && after.browser_br_observation.is_none()),
            "preparedParagraph",
            "horizontal break font-band evidence requires the original root, prepared source and selected break",
        )
    }

    fn validate_prepared_atomic_percentage(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        let paragraph = context.required(self.prepared_paragraph.as_ref(), "preparedParagraph")?;
        context.compatible(
            self.children.as_deref().is_some_and(|children| {
                let [before, atomic, after] = children else {
                    return false;
                };
                before.prepared_occurrence == Some(0)
                    && atomic.layout_input == LayoutInput::Box
                    && atomic.tag_name.as_deref() == Some("span")
                    && atomic.children.as_ref().is_some_and(Vec::is_empty)
                    && after.prepared_occurrence == Some(1)
                    && !children.iter().any(Self::has_authored_atomic_size)
            }) && matches!(paragraph.atomics.as_slice(), [atomic] if
                atomic.child == 1 && atomic.authored_size.is_some()),
            "preparedParagraph.atomics",
            "original percentage capture requires one authored atomic between its source occurrences",
        )
    }

    fn has_authored_atomic_size(&self) -> bool {
        self.prepared_paragraph.as_ref().is_some_and(|paragraph| {
            paragraph
                .atomics
                .iter()
                .any(|atomic| atomic.authored_size.is_some())
        }) || self
            .children
            .iter()
            .flatten()
            .any(Self::has_authored_atomic_size)
    }

    fn validate_wire(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.compatible(
            context.node_path == "root" || self.fri07_intrinsic_flex_text.is_none(),
            "fri07IntrinsicFlexText",
            "intrinsic Flex evidence belongs to the root capture",
        )?;
        context.compatible(
            context.node_path == "root" || self.flex_composition_image.is_none(),
            "flexCompositionImage",
            "Flex composition evidence belongs to the capture root",
        )?;
        context.compatible(
            context.node_path == "root" || self.prepared_range_frame.is_none(),
            "preparedRangeFrame",
            "prepared Range evidence belongs to the capture root",
        )?;
        context.compatible(
            context.node_path == "root" || self.measured_leaf.is_none(),
            "measuredLeaf",
            "measured leaf evidence belongs to the capture root",
        )?;
        context.compatible(
            context.node_path == "root" || self.flex_content_text.is_none(),
            "flexContentText",
            "Flex content evidence belongs to the capture root",
        )?;
        if let Some(capture) = &self.grid_alignment_reference {
            capture.validate().map_err(|detail| {
                context.error(
                    "gridAlignmentReference",
                    MeasurementErrorKind::InvalidValue,
                    detail,
                )
            })?;
            let style = context.required(self.style.as_ref(), "style")?;
            context.compatible(
                self.layout_input == LayoutInput::Box
                    && matches!(self.tag_name.as_deref(), Some("div" | "span"))
                    && style.display.as_deref() == Some(capture.display.as_str())
                    && style.writing_mode.as_deref() == Some(capture.writing_mode.as_str())
                    && style.direction.as_deref() == Some(capture.direction.as_str())
                    && style.has_reference_font(),
                "gridAlignmentReference",
                "Grid reference disagrees with its actual own role/frame/font",
            )?;
        }
        if let Some(capture) = &self.atomic_paragraph_metrics {
            if let Err(detail) = capture.validate() {
                return Err(context.error(
                    "atomicParagraphMetrics",
                    MeasurementErrorKind::InvalidValue,
                    detail,
                ));
            }
            let style = context.required(self.style.as_ref(), "style")?;
            context.compatible(
                self.layout_input == LayoutInput::Box
                    && self.tag_name.as_deref() == Some("div")
                    && self.prepared_paragraph.is_none()
                    && self.prepared_occurrence.is_none()
                    && self.inline_segments.is_none()
                    && self.text_content.is_none()
                    && style.display.as_deref() == Some("block")
                    && style.writing_mode.as_deref() == Some(capture.writing_mode.as_str())
                    && style.direction.as_deref() == Some(capture.direction.as_str())
                    && style.vertical_align.as_deref().unwrap_or("baseline")
                        == capture.vertical_align.as_str()
                    && style.has_atomic_metric_font(),
                "atomicParagraphMetrics",
                "named paragraph certificate disagrees with actual containing box",
            )?;
            let children = context.required(self.children.as_ref(), "children")?;
            context.compatible(
                children.len() == 3,
                "atomicParagraphMetrics.order",
                "certificate requires all three actual atomic/BR occurrences",
            )?;
            for (index, row) in capture.order.iter().enumerate() {
                let child = &children[index];
                let own = context.required(child.style.as_ref(), "children.style")?;
                let same_frame = own.writing_mode == style.writing_mode
                    && own.direction == style.direction
                    && own.vertical_align.as_deref().unwrap_or("baseline")
                        == capture.vertical_align.as_str()
                    && own.has_atomic_metric_font();
                let empty_box = child.layout_input == LayoutInput::Box
                    && child.children.as_ref().is_some_and(Vec::is_empty)
                    && child.text_content.is_none()
                    && child.prepared_paragraph.is_none()
                    && child.atomic_paragraph_metrics.is_none();
                let role = match row {
                    atomic_paragraph_metrics::Occurrence::Atomic { bidi_level, .. } => {
                        child.tag_name.as_deref() == Some("span")
                            && own.display.as_deref() == Some("inline-block")
                            && child.line_control_participation.is_none()
                            && child
                                .atomic_inline_participation
                                .as_ref()
                                .is_some_and(|atomic| {
                                    atomic.bidi_level == u64::from(*bidi_level)
                                        && atomic.following_break == BreakKind::Prohibited
                                })
                    }
                    atomic_paragraph_metrics::Occurrence::Break { .. } => {
                        child.tag_name.as_deref() == Some("br")
                            && own.display.as_deref() == Some("inline")
                            && child.line_control_participation.is_some()
                            && child.atomic_inline_participation.is_none()
                            && own.has_zero_inline_metrics()
                    }
                };
                context.compatible(
                    same_frame && empty_box && role,
                    "atomicParagraphMetrics.order",
                    "certificate does not match actual same-reference atomic/BR child",
                )?;
            }
        }

        context.compatible(
            self.world_rounded_layout.is_none() || self.layout_input == LayoutInput::Box,
            "worldRoundedLayout",
            "world-corner diagnostic requires an actual measured box",
        )?;
        context.compatible(
            self.browser_br_observation.is_none()
                || (self.layout_input == LayoutInput::Box
                    && self.tag_name.as_deref() == Some("br")
                    && self
                        .style
                        .as_ref()
                        .and_then(|style| style.display.as_deref())
                        == Some("inline")
                    && self.children.as_ref().is_none_or(Vec::is_empty)
                    && self.text_content.is_none()
                    && self.inline_segments.is_none()
                    && self.prepared_paragraph.is_none()
                    && self.prepared_occurrence.is_none()
                    && self.inline_boundary.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.range_rectangles.is_none()
                    && self.range_fragments.is_none()
                    && self.fragments.is_none()
                    && self.shape_bands.is_none()),
            "browserBrObservation",
            "a BR font-band request requires an actual visible inline BR leaf",
        )?;
        if let Some(paragraph) = &self.prepared_paragraph {
            paragraph.validate(context, self.children.as_ref().map_or(0, Vec::len))?;
        }
        context.compatible(
            self.range_fragments.is_none() || self.layout_input == LayoutInput::PreparedText,
            "rangeFragments",
            "source Range rectangles require prepared-text role",
        )?;
        if let Some(reason) = &self.unsupported_reason {
            context.compatible(
                matches!(
                    self.layout_input,
                    LayoutInput::Box | LayoutInput::Unsupported
                ),
                "unsupportedReason",
                "only measured boxes or unsupported descriptors carry reasons",
            )?;
            context.ensure(
                !reason.trim().is_empty(),
                "unsupportedReason",
                "unsupported reason must be nonempty",
            )?;
        }
        if self.layout_input == LayoutInput::Unsupported {
            context.required(self.unsupported_reason.as_ref(), "unsupportedReason")?;
            context.compatible(
                self.tag_name.is_none()
                    && self.style.is_none()
                    && self.use_rounding.is_none()
                    && self.viewport.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.text_content.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && self.world_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && self.prepared_paragraph.is_none()
                    && self.atomic_paragraph_metrics.is_none()
                    && self.grid_alignment_profile.is_none()
                    && self.grid_alignment_reference.is_none()
                    && self.prepared_occurrence.is_none()
                    && self.inline_segments.is_none()
                    && self.inline_boundary.is_none()
                    && self.line_control_participation.is_none()
                    && self.browser_br_observation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.fragments.is_none()
                    && self.range_rectangles.is_none()
                    && self.range_fragments.is_none()
                    && self.children.is_none(),
                "layoutInput",
                "unsupported descriptors carry only a reason",
            )?;
        }
        if self.layout_input == LayoutInput::Box {
            context.compatible(
                self.inline_segments.is_none()
                    && self.inline_boundary.is_none()
                    && self.range_rectangles.is_none(),
                "layoutInput",
                "box roles cannot carry inline text or boundary payload",
            )?;
        }
        context.compatible(
            self.prepared_paragraph.is_none() || self.layout_input == LayoutInput::Box,
            "preparedParagraph",
            "only an actual containing box owns a prepared paragraph",
        )?;
        context.compatible(
            self.prepared_occurrence.is_none() || self.layout_input == LayoutInput::PreparedText,
            "preparedOccurrence",
            "an original occurrence requires the prepared-text role",
        )?;
        if self.layout_input == LayoutInput::PreparedText {
            context.required(self.prepared_occurrence, "preparedOccurrence")?;
            context.compatible(
                self.inline_segments.is_none(),
                "inlineSegments",
                "measured segments cannot be a prepared-text input",
            )?;
        }
        if matches!(
            self.layout_input,
            LayoutInput::InlineText | LayoutInput::PreparedText
        ) {
            context.compatible(
                self.range_rectangles.is_none() || self.layout_input == LayoutInput::InlineText,
                "rangeInks",
                "historical measured observations cannot masquerade as native source observations",
            )?;
            if self.range_rectangles.is_some() || self.range_fragments.is_some() {
                context.compatible(self.fragments.is_none() && self.unrounded_layout.is_none() && self.smart_rounded_layout.is_none()
                    && self.world_rounded_layout.is_none() && self.naively_rounded_layout.is_none() && self.children.as_ref().is_none_or(Vec::is_empty) && self.style.as_ref().is_none_or(|style| !style.has_scroll_fields()), "rangeInks", "Range observations cannot carry geometry, fragments, children, or scroll state")?;
            }
            context.compatible(
                self.line_control_participation.is_none(),
                "lineControlParticipation",
                "inline text cannot carry line-control participation",
            )?;
            context.compatible(
                self.inline_boundary.is_none()
                    && self.tag_name.is_none()
                    && self.style.is_none()
                    && self.text_content.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.children.as_ref().is_none_or(Vec::is_empty),
                "layoutInput",
                "inline text cannot carry box or boundary payload",
            )?;
        }
        if self.layout_input == LayoutInput::InlineBoundary {
            context.compatible(
                self.tag_name.is_none()
                    && self.style.is_none()
                    && self.use_rounding.is_none()
                    && self.viewport.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()
                    && self.text_content.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && self.world_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && self.inline_segments.is_none()
                    && self.line_control_participation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.fragments.is_none()
                    && self.range_rectangles.is_none()
                    && self.children.as_ref().is_none_or(Vec::is_empty),
                "inlineBoundary",
                "inline boundaries cannot carry box, text, or observation payload",
            )?;
        }
        if let Some(style) = &self.style {
            style.validate_wire(context)?;
        }
        if let Some(viewport) = &self.viewport {
            viewport.validate_alternative(context)?;
        }
        if let Some(boundary) = &self.inline_boundary {
            boundary.validate_alternative(context)?;
        }
        for (index, segment) in self
            .inline_segments
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            validate_break_alternative(
                &segment.following_break,
                segment.replacement_inline_extent,
                context,
                &format!("inlineSegments[{index}]"),
            )?;
        }
        if let Some(atomic) = &self.atomic_inline_participation {
            atomic.validate_alternative(context)?;
        }
        for (index, child) in self
            .children
            .as_deref()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            child.validate_wire(&context.child(index))?;
        }
        Ok(())
    }
    fn validate(
        self,
        context: &Context<'_>,
        parent_writing_mode: &str,
        use_rounding: bool,
        is_root: bool,
    ) -> Result<Node, MeasurementError> {
        let mut attrs = Vec::new();
        if let Some(profile) = &self.grid_alignment_profile {
            let encoded = serde_json::to_string(profile).map_err(|_| {
                context.error(
                    "gridAlignmentProfile",
                    MeasurementErrorKind::InvalidValue,
                    "cannot encode complete Grid owner profile",
                )
            })?;
            attrs.push(("grid-alignment-profile", encoded));
        }
        if let Some(capture) = &self.grid_alignment_reference {
            let encoded = serde_json::to_string(capture).map_err(|_| {
                context.error(
                    "gridAlignmentReference",
                    MeasurementErrorKind::InvalidValue,
                    "cannot encode own Grid reference",
                )
            })?;
            attrs.push(("grid-alignment-reference", encoded));
        }
        if let Some(capture) = &self.atomic_paragraph_metrics {
            let encoded = serde_json::to_string(capture).map_err(|_| {
                context.error(
                    "atomicParagraphMetrics",
                    MeasurementErrorKind::InvalidValue,
                    "cannot encode finite metric certificate",
                )
            })?;
            attrs.push(("atomic-paragraph-metrics", encoded));
        }
        if let Some(tag) = &self.tag_name {
            validate_xml_text(tag, context, "tagName")?;
        }
        if let Some(text) = &self.text_content {
            validate_xml_text(text, context, "textContent")?;
        }
        let children = context.required(self.children, "children")?;
        let style = self.style.unwrap_or_default();
        let writing_mode = style.writing_mode().to_string();
        if self.layout_input == LayoutInput::InlineBoundary {
            let boundary = context.required(self.inline_boundary, "inlineBoundary")?;
            return Ok(Node {
                prepared_paragraph: None,
                kind: NodeKind::Boundary(boundary.validate(context)?),
                attrs,
                children: Vec::new(),
                text: None,
                observation: Observation::Boundary,
                world_rounded_layout: None,
                atomic: None,
                bands: None,
                control_expectation: None,
                br_font_band: None,
            });
        }
        context.compatible(
            self.inline_boundary.is_none(),
            "inlineBoundary",
            "boundary descriptor requires inline-boundary role",
        )?;
        let inline_text = matches!(
            self.layout_input,
            LayoutInput::InlineText | LayoutInput::PreparedText
        );
        context.compatible(
            inline_text
                || (self.inline_segments.is_none()
                    && self.range_rectangles.is_none()
                    && self.range_fragments.is_none()),
            "layoutInput",
            "text segments and Range observations require inline-text role",
        )?;
        let control = self.line_control_participation.is_some();
        let display = style.display.as_deref();
        let is_br = self.tag_name.as_deref() == Some("br");
        let br_control = is_br && matches!(display, Some("inline" | "none"));
        let br_box = is_br && display == Some("block");
        if let Some(WireControl {
            kind: ControlKind::ForcedBreak,
        }) = &self.line_control_participation
        {
            context.compatible(
                !inline_text && is_br && display == Some("inline") && children.is_empty(),
                "lineControlParticipation",
                "forced break requires an inline BR without children",
            )?;
        }
        context.compatible(
            !br_box || self.atomic_inline_participation.is_none(),
            "atomicInlineParticipation",
            "block BR cannot participate as an atomic inline",
        )?;
        context.compatible(
            !inline_text
                || (self.tag_name.is_none()
                    && children.is_empty()
                    && self.text_content.is_none()
                    && self.line_control_participation.is_none()
                    && self.atomic_inline_participation.is_none()
                    && self.shape_bands.is_none()
                    && self.layout_ready_inline_root.is_none()
                    && self.layout_ready_anonymous_grid_text_wrapper.is_none()),
            "layoutInput",
            "inline text cannot carry box participation or children",
        )?;
        if (!is_br || br_control || br_box)
            && let Some(tag) = self.tag_name
        {
            attrs.push(("source-tag", tag));
        }
        if let Some(marker) = self.layout_ready_inline_root {
            context.ensure(
                marker,
                "layoutReadyInlineRoot",
                "marker must be true when present",
            )?;
            attrs.push(("layout-ready-inline-root", "true".into()));
        }
        if let Some(marker) = self.layout_ready_anonymous_grid_text_wrapper {
            context.ensure(
                marker,
                "layoutReadyAnonymousGridTextWrapper",
                "marker must be true when present",
            )?;
            context.compatible(
                style.is_grid()
                    && !children.is_empty()
                    && children.iter().all(|child| {
                        child.layout_input == LayoutInput::InlineText
                            && child.children.as_ref().is_none_or(Vec::is_empty)
                    })
                    && self.text_content.is_none(),
                "layoutReadyAnonymousGridTextWrapper",
                "anonymous grid text wrapper requires only direct typed text",
            )?;
            attrs.push(("layout-ready-anonymous-grid-text-wrapper", "true".into()));
        }
        if control {
            attrs.push(("line-control", "forced-break".into()));
        }
        let shape = self
            .shape_bands
            .map(|bands| {
                context.ensure(
                    !bands.is_empty(),
                    "shapeBands",
                    "shape bands must be nonempty",
                )?;
                let mut queries = Vec::new();
                bands
                    .into_iter()
                    .enumerate()
                    .map(|(index, band)| {
                        let query = (band.band_minimum, band.band_maximum);
                        context.ensure(
                            !queries.contains(&query),
                            format!("shapeBands[{index}]"),
                            "shape query endpoints must be unique",
                        )?;
                        queries.push(query);
                        band.validate(context, &format!("shapeBands[{index}]"))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        attrs.extend(style.attrs(
            context,
            parent_writing_mode,
            shape.is_some(),
            !is_br || br_control,
        )?);
        context.compatible(
            !is_root || self.atomic_inline_participation.is_none(),
            "atomicInlineParticipation",
            "atomic participation requires a parent input slot",
        )?;
        let atomic = self
            .atomic_inline_participation
            .map(|value| value.validate(context))
            .transpose()?;
        let kind = if self.layout_input == LayoutInput::PreparedText {
            NodeKind::PreparedText(
                context.required(self.prepared_occurrence, "preparedOccurrence")?,
            )
        } else if inline_text {
            let segments = context.required(self.inline_segments, "inlineSegments")?;
            context.ensure(
                !segments.is_empty(),
                "inlineSegments",
                "inline text requires at least one segment",
            )?;
            let mut seen = std::collections::BTreeSet::new();
            let segments = segments
                .into_iter()
                .enumerate()
                .map(|(index, segment)| {
                    context.ensure(
                        seen.insert(segment.id),
                        format!("inlineSegments[{index}].id"),
                        "segment identifier must be unique within its text node",
                    )?;
                    segment.validate(context, &format!("inlineSegments[{index}]"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            NodeKind::InlineText(segments)
        } else if control {
            NodeKind::Control
        } else if self.text_content.is_some()
            && !style.is_grid()
            && !children
                .iter()
                .any(|child| child.layout_input == LayoutInput::InlineText)
        {
            NodeKind::Text
        } else {
            NodeKind::Box
        };
        let world_rounded_layout = if self.layout_input == LayoutInput::Box {
            Some(
                context
                    .required(self.world_rounded_layout, "worldRoundedLayout")?
                    .validate_diagnostic(context)?,
            )
        } else {
            None
        };
        let native_ranges = self.range_fragments.is_some();
        let native_rectangles = self.range_fragments.map(|rows| {
            rows.into_iter()
                .map(WireRangeFragment::rectangle_fields)
                .collect()
        });
        let observation = if let Some(ranges) = self.range_rectangles.or(native_rectangles) {
            context.compatible(
                self.fragments.is_none()
                    && self.unrounded_layout.is_none()
                    && self.smart_rounded_layout.is_none()
                    && world_rounded_layout.is_none()
                    && self.naively_rounded_layout.is_none()
                    && !style.has_scroll_fields(),
                "rangeInks",
                "Range observations cannot carry geometry, fragments, or scroll state",
            )?;
            let ranges = ranges
                .into_iter()
                .enumerate()
                .map(|(index, range)| range.validate(context, &format!("rangeFragments[{index}]")))
                .collect::<Result<Vec<_>, _>>()?;
            if native_ranges {
                let ranges = ranges
                    .into_iter()
                    .map(|mut attributes: Attributes| {
                        for (name, _) in &mut attributes {
                            if *name == "source_segment_id" {
                                *name = "origin";
                            } else if *name == "line_index" {
                                *name = "line";
                            }
                        }
                        attributes
                    })
                    .collect();
                Observation::RangeFragments(ranges)
            } else {
                Observation::RangeRectangles(ranges)
            }
        } else {
            let unrounded = context.required(self.unrounded_layout, "unroundedLayout")?;
            let geometry = unrounded.validate(context, "unroundedLayout")?;
            let selected_scroll = if use_rounding {
                self.smart_rounded_layout
                    .as_ref()
                    .map(|value| (value.scroll_width, value.scroll_height))
            } else {
                Some((unrounded.scroll_width, unrounded.scroll_height))
            };
            let rounded = self
                .smart_rounded_layout
                .as_ref()
                .map(|value| value.validate(context, "smartRoundedLayout"))
                .transpose()?;
            let selected = if use_rounding {
                context.required(rounded, "smartRoundedLayout")?
            } else {
                geometry
            };
            if br_box {
                attrs.retain(|(key, _)| !matches!(*key, "width" | "height"));
                let index = attrs
                    .iter()
                    .position(|(key, _)| {
                        matches!(
                            *key,
                            "min-width"
                                | "min-height"
                                | "max-width"
                                | "max-height"
                                | "aspect-ratio"
                                | "row-gap"
                                | "column-gap"
                                | "top"
                                | "right"
                                | "bottom"
                                | "left"
                        ) || key.starts_with("margin-")
                            || key.starts_with("padding-")
                            || key.starts_with("border-")
                            || key.starts_with("grid-")
                    })
                    .unwrap_or(attrs.len());
                attrs.splice(
                    index..index,
                    [
                        ("width", format!("{}px", number(geometry.width))),
                        ("height", format!("{}px", number(geometry.height))),
                    ],
                );
            }
            let scroll = if style.has_scroll_overflow() {
                let naive =
                    context.required(self.naively_rounded_layout, "naivelyRoundedLayout")?;
                naive.validate_supplied(context, "naivelyRoundedLayout")?;
                let (scroll_width, scroll_height) = selected_scroll.ok_or_else(|| {
                    context.error(
                        "smartRoundedLayout",
                        MeasurementErrorKind::MissingField,
                        "rounded scroll observations are absent",
                    )
                })?;
                let sw = context.required(
                    scroll_width,
                    if use_rounding {
                        "smartRoundedLayout.scrollWidth"
                    } else {
                        "unroundedLayout.scrollWidth"
                    },
                )?;
                let sh = context.required(
                    scroll_height,
                    if use_rounding {
                        "smartRoundedLayout.scrollHeight"
                    } else {
                        "unroundedLayout.scrollHeight"
                    },
                )?;
                let cw =
                    context.required(naive.client_width, "naivelyRoundedLayout.clientWidth")?;
                let ch =
                    context.required(naive.client_height, "naivelyRoundedLayout.clientHeight")?;
                for (field, value) in [
                    ("unroundedLayout.scrollWidth", sw),
                    ("unroundedLayout.scrollHeight", sh),
                    ("naivelyRoundedLayout.clientWidth", cw),
                    ("naivelyRoundedLayout.clientHeight", ch),
                ] {
                    nonnegative(value, context, field)?;
                }
                Some((
                    layout_number((sw - cw).max(0.0), context, "scrollWidth")?,
                    layout_number((sh - ch).max(0.0), context, "scrollHeight")?,
                ))
            } else {
                if let Some(naive) = self.naively_rounded_layout {
                    naive.validate_supplied(context, "naivelyRoundedLayout")?;
                }
                None
            };
            let fragments = self
                .fragments
                .map(|fragments| {
                    fragments
                        .into_iter()
                        .enumerate()
                        .map(|(index, fragment)| {
                            fragment.validate(context, &format!("fragments[{index}]"))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?;
            Observation::Geometry {
                unrounded: geometry,
                selected,
                scroll,
                fragments,
            }
        };
        let br_font_band = if self.browser_br_observation.is_some() {
            context.ensure(
                !is_root,
                "browserBrObservation",
                "a BR font-band request requires an inline child in its actual container",
            )?;
            let Observation::Geometry {
                selected,
                scroll: None,
                fragments: None,
                ..
            } = &observation
            else {
                return Err(context.error(
                    "browserBrObservation",
                    MeasurementErrorKind::InvalidValue,
                    "a BR font-band observation requires measured geometry",
                ));
            };
            Some(*selected)
        } else {
            None
        };
        let prepared_paragraph = self
            .prepared_paragraph
            .as_ref()
            .map(|paragraph| {
                paragraph.validate_frame(
                    context,
                    &writing_mode,
                    style.direction.as_deref().unwrap_or("ltr"),
                )?;
                let encoded = paragraph.validate(context, children.len())?;
                for (origin, row) in paragraph.origins.iter().enumerate() {
                    context.ensure(
                        children[row.child].layout_input == LayoutInput::PreparedText
                            && children[row.child].prepared_occurrence == Some(origin),
                        "preparedParagraph.origins",
                        "each input leaf must lend its exact original occurrence",
                    )?;
                }
                for row in &paragraph.atomics {
                    let child = &children[row.child];
                    let style = child.style.as_ref();
                    let display = style.and_then(|style| style.display.as_deref());
                    let absolute = style.and_then(|style| style.position.as_deref())
                        == Some("absolute");
                    let floating = style
                        .and_then(|style| style.css_float.as_deref())
                        .is_some_and(|value| value != "none");
                    // The independent reader lowers actual inline BRs to break
                    // controls even without optional lineControlParticipation.
                    let inline_br = child.tag_name.as_deref() == Some("br")
                        && matches!(display, Some("inline" | "none"));
                    let matches_flow = match row.kind {
                        prepared_text::AtomicKind::InFlow => {
                            matches!(display,
                                Some("inline-block" | "inline-grid" | "inline-grid-lanes"))
                                && !absolute
                                && !floating
                        }
                        prepared_text::AtomicKind::OutOfFlow => {
                            absolute && display != Some("none")
                        }
                    };
                    context.ensure(
                        child.layout_input == LayoutInput::Box
                            && child.line_control_participation.is_none()
                            && child.atomic_inline_participation.is_none()
                            && !inline_br
                            && matches_flow,
                        "preparedParagraph.atomics",
                        "captured atomic flow role requires the corresponding actual box without control or measured-level authority",
                    )?;
                    if !matches!(paragraph.writing_mode, prepared_text::WritingMode::HorizontalTb) {
                        context.ensure(
                            display == Some("inline-block"),
                            "preparedParagraph.atomics",
                            "vertical capture requires an actual inline-block",
                        )?;
                    }
                }
                for row in &paragraph.controls {
                    let child = &children[row.child];
                    let matches_role = match row.kind {
                        prepared_text::ControlKind::ForcedBreak => {
                            child.layout_input == LayoutInput::Box
                                && child.tag_name.as_deref() == Some("br")
                                && child.style.as_ref().and_then(|style| style.display.as_deref())
                                    == Some("inline")
                                && child.children.as_ref().is_none_or(Vec::is_empty)
                                && matches!(
                                    child.line_control_participation.as_ref(),
                                    Some(WireControl { kind: ControlKind::ForcedBreak })
                                )
                        }
                        prepared_text::ControlKind::InlineBoundaryStart => {
                            child.layout_input == LayoutInput::InlineBoundary
                                && matches!(child.inline_boundary.as_ref(),
                                    Some(boundary) if boundary.kind == BoundaryKind::Start)
                        }
                        prepared_text::ControlKind::InlineBoundaryEnd => {
                            child.layout_input == LayoutInput::InlineBoundary
                                && matches!(child.inline_boundary.as_ref(),
                                    Some(boundary) if boundary.kind == BoundaryKind::End)
                        }
                    };
                    context.ensure(
                        matches_role,
                        "preparedParagraph.controls",
                        "each captured control must match its actual child role",
                    )?;
                }
                Ok(encoded)
            })
            .transpose()?;
        let has_typed_text = children.iter().any(|child| {
            matches!(
                child.layout_input,
                LayoutInput::InlineText | LayoutInput::PreparedText
            )
        });
        let mut children = children
            .into_iter()
            .enumerate()
            .map(|(index, child)| {
                child.validate(&context.child(index), &writing_mode, use_rounding, false)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !is_root && let Some(viewport) = self.viewport {
            viewport.validate(context)?;
        }
        prepare_control_expectations(&writing_mode, &mut children, context)?;
        Ok(Node {
            kind,
            prepared_paragraph,
            attrs,
            children,
            text: if has_typed_text {
                None
            } else {
                self.text_content
            },
            observation,
            world_rounded_layout,
            atomic,
            bands: shape,
            control_expectation: None,
            br_font_band,
        })
    }
}
impl WireNode {
    fn validate_original_flex_parent(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        let parent = context.required(self.style.as_ref(), "style")?;
        let children = context.required(self.children.as_deref(), "children")?;
        context.compatible(
            self.tag_name.as_deref() == Some("div")
                && self.layout_input == LayoutInput::Box
                && self.text_content.is_none()
                && parent.position.as_deref() == Some("absolute")
                && parent.display.as_deref() == Some("flex")
                && children.len() == 1,
            "children",
            "original capture requires the actual Flex parent and its sole Grid child",
        )?;
        let grid = &children[0];
        let grid_context = context.child(0);
        let style = grid_context.required(grid.style.as_ref(), "style")?;
        let leaves = grid_context.required(grid.children.as_deref(), "children")?;
        grid_context.compatible(
            grid.tag_name.as_deref() == Some("div")
                && grid.layout_input == LayoutInput::Box
                && grid.text_content.is_none()
                && style.display.as_deref() == Some("grid")
                && leaves.len() == 2,
            "children",
            "original Grid requires two source text boxes",
        )?;
        for (index, leaf) in leaves.iter().enumerate() {
            let leaf_context = grid_context.child(index);
            let style = leaf_context.required(leaf.style.as_ref(), "style")?;
            let text = leaf_context.required(leaf.text_content.as_deref(), "textContent")?;
            leaf_context.compatible(
                leaf.tag_name.as_deref() == Some("div")
                    && leaf.layout_input == LayoutInput::Box
                    && leaf.children.as_ref().is_some_and(Vec::is_empty)
                    && style.display.as_deref() == Some("flex")
                    && style.has_direct_flex_text_style()
                    && text.contains('\u{200b}')
                    && text
                        .split('\u{200b}')
                        .all(|word| !word.is_empty() && word.bytes().all(|byte| byte == b'H')),
                "textContent",
                "original Grid text boxes require captured Ahem break opportunities",
            )?;
        }
        Ok(())
    }

    fn validate_original_percentage_flex_parent(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        let mut node = self;
        let mut place = Context {
            node_path: context.node_path.clone(),
            ..*context
        };
        let displays = ["flex", "grid", "flex", "flex"];
        for (index, display) in displays.into_iter().enumerate() {
            let style = place.required(node.style.as_ref(), "style")?;
            let children = place.required(node.children.as_deref(), "children")?;
            let last = index == displays.len() - 1;
            place.compatible(
                node.layout_input == LayoutInput::Box
                    && node.tag_name.as_deref() == Some("div")
                    && node.text_content.is_none()
                    && style.display.as_deref() == Some(display)
                    && (index != 0 || style.position.as_deref() == Some("absolute"))
                    && children.len() == usize::from(!last),
                "children",
                "original percentage capture requires the full Flex, Grid and ratio-box source tree",
            )?;
            let unrounded = place.required(node.unrounded_layout.as_ref(), "unroundedLayout")?;
            let rounded =
                place.required(node.smart_rounded_layout.as_ref(), "smartRoundedLayout")?;
            place.compatible(
                unrounded.validate(&place, "unroundedLayout")?
                    == rounded.validate(&place, "smartRoundedLayout")?,
                "unroundedLayout",
                "original percentage capture requires identical unrounded and rounded rectangles",
            )?;
            if let Some(child) = children.first() {
                node = child;
                place = place.child(0);
            }
        }
        Ok(())
    }

    fn validate_original_subgrid_body(
        &self,
        context: &Context<'_>,
    ) -> Result<(), MeasurementError> {
        let body = context.required(self.style.as_ref(), "style")?;
        let children = context.required(self.children.as_deref(), "children")?;
        context.compatible(
            self.tag_name.as_deref() == Some("body")
                && self.layout_input == LayoutInput::Box
                && body.position.as_deref() == Some("static")
                && body.display.as_deref() == Some("block")
                && children.len() == 1,
            "children",
            "original subgrid requires the actual body and its sole Grid child",
        )?;
        let grid = &children[0];
        context.compatible(
            grid.style
                .as_ref()
                .and_then(|style| style.display.as_deref())
                == Some("grid"),
            "children[0].style.display",
            "the observed child must be a Grid box",
        )?;
        let grid_context = context.child(0);
        for (node, place) in [(self, context), (grid, &grid_context)] {
            let raw = place.required(node.unrounded_layout.as_ref(), "unroundedLayout")?;
            place.ensure(
                raw.x == Some(0.0) && raw.y == Some(0.0),
                "unroundedLayout",
                "body and Grid must each start at their real parent origin",
            )?;
        }
        let geometry =
            context.required(self.world_rounded_layout.as_ref(), "worldRoundedLayout")?;
        geometry.validate_diagnostic(context)?;
        context.ensure(
            grid.validate_original_subgrid_boxes(&grid_context)? == 16,
            "children",
            "the Grid observation must contain every source box",
        )
    }

    fn validate_original_subgrid_boxes(
        &self,
        context: &Context<'_>,
    ) -> Result<usize, MeasurementError> {
        context.compatible(
            self.layout_input == LayoutInput::Box
                && self.tag_name.as_deref() == Some("div")
                && self
                    .style
                    .as_ref()
                    .is_some_and(|style| style.position.is_none()),
            "layoutInput",
            "the Grid observation admits only static source boxes",
        )?;
        let geometry =
            context.required(self.world_rounded_layout.as_ref(), "worldRoundedLayout")?;
        geometry.validate_diagnostic(context)?;
        let mut count = 1;
        for (index, child) in self.children.iter().flatten().enumerate() {
            count += child.validate_original_subgrid_boxes(&context.child(index))?;
        }
        Ok(count)
    }
}

impl WireGeometry {
    fn validate_diagnostic(&self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        let mut attrs = Vec::new();
        for (name, value, extent) in [
            ("x", self.x, false),
            ("y", self.y, false),
            ("width", self.width, true),
            ("height", self.height, true),
        ] {
            let field = format!("worldRoundedLayout.{name}");
            let value = context.required(value, &field)?;
            if extent {
                nonnegative(value, context, &field)?;
            } else {
                finite(value, context, &field)?;
            }
            attrs.push((name, number(value)));
        }
        for (first, second, a, b) in [
            (
                "scroll-width",
                "scroll-height",
                self.scroll_width,
                self.scroll_height,
            ),
            (
                "client-width",
                "client-height",
                self.client_width,
                self.client_height,
            ),
        ] {
            context.compatible(
                a.is_some() == b.is_some(),
                "worldRoundedLayout",
                "diagnostic extent pairs must be supplied together",
            )?;
            if let (Some(a), Some(b)) = (a, b) {
                attrs.push((
                    first,
                    number(nonnegative(a, context, "worldRoundedLayout")?),
                ));
                attrs.push((
                    second,
                    number(nonnegative(b, context, "worldRoundedLayout")?),
                ));
            }
        }
        Ok(attrs)
    }

    fn validate(&self, context: &Context<'_>, prefix: &str) -> Result<Geometry, MeasurementError> {
        self.validate_supplied(context, prefix)?;
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(Geometry {
            x: layout_number(context.required(self.x, &field("x"))?, context, &field("x"))?,
            y: layout_number(context.required(self.y, &field("y"))?, context, &field("y"))?,
            width: layout_number(
                nonnegative(
                    context.required(self.width, &field("width"))?,
                    context,
                    &field("width"),
                )?,
                context,
                &field("width"),
            )?,
            height: layout_number(
                nonnegative(
                    context.required(self.height, &field("height"))?,
                    context,
                    &field("height"),
                )?,
                context,
                &field("height"),
            )?,
        })
    }
    fn validate_supplied(
        &self,
        context: &Context<'_>,
        prefix: &str,
    ) -> Result<(), MeasurementError> {
        for (name, value) in [
            ("x", self.x),
            ("y", self.y),
            ("width", self.width),
            ("height", self.height),
            ("scrollWidth", self.scroll_width),
            ("scrollHeight", self.scroll_height),
            ("clientWidth", self.client_width),
            ("clientHeight", self.client_height),
        ] {
            if let Some(value) = value {
                finite(value, context, &format!("{prefix}.{name}"))?;
                if !matches!(name, "x" | "y") {
                    nonnegative(value, context, &format!("{prefix}.{name}"))?;
                }
            }
        }
        Ok(())
    }
}
impl WireBoundary {
    fn validate_alternative(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.compatible(
            self.baseline.is_some() == self.line_height.is_some(),
            "inlineBoundary",
            "boundary metrics must be paired",
        )?;
        context.compatible(
            self.kind == BoundaryKind::Start || self.baseline.is_none(),
            "inlineBoundary.kind",
            "only boundary starts carry metrics",
        )
    }
    fn validate(self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        let kind = match self.kind {
            BoundaryKind::Start => "start",
            BoundaryKind::End => "end",
        };
        let mut attrs = vec![("kind", kind.into())];
        match (self.baseline, self.line_height) {
            (None, None) => {}
            (Some(baseline), Some(height)) => {
                context.compatible(
                    self.kind == BoundaryKind::Start,
                    "inlineBoundary.kind",
                    "only boundary starts carry metrics",
                )?;
                nonnegative(baseline, context, "inlineBoundary.baseline")?;
                nonnegative(height, context, "inlineBoundary.lineHeight")?;
                context.ensure(
                    height > 0.0 && baseline <= height,
                    "inlineBoundary.lineHeight",
                    "boundary strut must be positive and cover its baseline",
                )?;
                attrs.extend([
                    ("inline-baseline", number(baseline)),
                    ("inline-line-height", number(height)),
                ]);
            }
            _ => {
                return Err(context.error(
                    "inlineBoundary",
                    MeasurementErrorKind::ContradictoryFields,
                    "boundary metrics must be paired",
                ));
            }
        }
        Ok(attrs)
    }
}
fn break_attrs(
    kind: BreakKind,
    replacement: Option<f64>,
    context: &Context<'_>,
    prefix: &str,
) -> Result<Attributes, MeasurementError> {
    validate_break_alternative(&kind, replacement, context, prefix)?;
    let name = match kind {
        BreakKind::Prohibited => "prohibited",
        BreakKind::Allowed => "allowed",
        BreakKind::Mandatory => "mandatory",
        BreakKind::AllowedWithReplacement => "allowed-with-replacement",
    };
    let mut attrs = vec![("following-break", name.into())];
    if let Some(value) = replacement {
        attrs.push((
            "replacement-inline-extent",
            number(nonnegative(
                value,
                context,
                &format!("{prefix}.replacementInlineExtent"),
            )?),
        ));
    }
    Ok(attrs)
}
impl WireSegment {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        let mut attrs = vec![
            ("id", self.id.to_string()),
            (
                "inline-extent",
                number(nonnegative(
                    self.inline_extent,
                    context,
                    &field("inlineExtent"),
                )?),
            ),
            (
                "inline-baseline",
                number(nonnegative(
                    self.inline_baseline,
                    context,
                    &field("inlineBaseline"),
                )?),
            ),
            (
                "inline-line-height",
                number(nonnegative(
                    self.inline_line_height,
                    context,
                    &field("inlineLineHeight"),
                )?),
            ),
            (
                "bidi-level",
                bidi(self.bidi_level, context, &field("bidiLevel"))?,
            ),
            (
                "whitespace-edge",
                match self.whitespace_edge {
                    WhitespaceEdge::Preserve => "preserve",
                    WhitespaceEdge::DiscardAtStart => "discard-at-start",
                    WhitespaceEdge::DiscardAtEnd => "discard-at-end",
                    WhitespaceEdge::DiscardAtBoth => "discard-at-both",
                }
                .into(),
            ),
        ];
        context.ensure(
            self.inline_baseline <= self.inline_line_height,
            field("inlineLineHeight"),
            "line height must cover the baseline",
        )?;
        attrs.extend(break_attrs(
            self.following_break,
            self.replacement_inline_extent,
            context,
            prefix,
        )?);
        Ok(attrs)
    }
}
impl WireAtomic {
    fn validate_alternative(&self, context: &Context<'_>) -> Result<(), MeasurementError> {
        context.compatible(
            self.following_break != BreakKind::AllowedWithReplacement
                && self.replacement_inline_extent.is_none(),
            "atomicInlineParticipation",
            "atomic breaks cannot carry text replacements",
        )
    }
    fn validate(self, context: &Context<'_>) -> Result<Attributes, MeasurementError> {
        context.compatible(
            self.following_break != BreakKind::AllowedWithReplacement
                && self.replacement_inline_extent.is_none(),
            "atomicInlineParticipation",
            "atomic breaks cannot carry text replacements",
        )?;
        let mut attrs = vec![(
            "bidi-level",
            bidi(
                self.bidi_level,
                context,
                "atomicInlineParticipation.bidiLevel",
            )?,
        )];
        attrs.extend(break_attrs(
            self.following_break,
            self.replacement_inline_extent,
            context,
            "atomicInlineParticipation",
        )?);
        Ok(attrs)
    }
}
impl WireBand {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let mut attrs = vec![
            (
                "band-minimum",
                number(finite(
                    self.band_minimum,
                    context,
                    &format!("{prefix}.bandMinimum"),
                )?),
            ),
            (
                "band-maximum",
                number(finite(
                    self.band_maximum,
                    context,
                    &format!("{prefix}.bandMaximum"),
                )?),
            ),
        ];
        context.ensure(
            self.band_minimum <= self.band_maximum,
            prefix,
            "band interval must not decrease",
        )?;
        match self.interval {
            None => {}
            Some(WireInterval { minimum, maximum }) => {
                finite(minimum, context, &format!("{prefix}.interval.minimum"))?;
                finite(maximum, context, &format!("{prefix}.interval.maximum"))?;
                context.ensure(
                    minimum <= maximum,
                    prefix,
                    "shape interval must not decrease",
                )?;
                attrs.extend([
                    ("interval-minimum", number(minimum)),
                    ("interval-maximum", number(maximum)),
                ]);
            }
        }
        Ok(attrs)
    }
}
impl WireFragment {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(vec![
            ("source_segment_id", self.source_segment_id.to_string()),
            (
                "line_index",
                bounded_index(self.line_index, context, &field("lineIndex"))?,
            ),
            (
                "visual_index",
                bounded_index(self.visual_index, context, &field("visualIndex"))?,
            ),
            ("x", number(finite(self.x, context, &field("x"))?)),
            ("y", number(finite(self.y, context, &field("y"))?)),
            (
                "width",
                number(nonnegative(self.width, context, &field("width"))?),
            ),
            (
                "height",
                number(nonnegative(self.height, context, &field("height"))?),
            ),
            (
                "baseline_x",
                number(finite(self.baseline_x, context, &field("baselineX"))?),
            ),
            (
                "baseline_y",
                number(finite(self.baseline_y, context, &field("baselineY"))?),
            ),
        ])
    }
}
impl WireRange {
    fn validate(self, context: &Context<'_>, prefix: &str) -> Result<Attributes, MeasurementError> {
        let field = |name: &str| format!("{prefix}.{name}");
        Ok(vec![
            ("source_segment_id", self.source_segment_id.to_string()),
            (
                "line_index",
                bounded_index(self.line_index, context, &field("lineIndex"))?,
            ),
            (
                "physical_start_edge",
                match self.physical_start_edge {
                    PhysicalEdge::Left => "left",
                    PhysicalEdge::Right => "right",
                    PhysicalEdge::Top => "top",
                    PhysicalEdge::Bottom => "bottom",
                }
                .into(),
            ),
            (
                "start",
                number(finite(self.start, context, &field("start"))?),
            ),
            (
                "advance",
                number(nonnegative(self.advance, context, &field("advance"))?),
            ),
        ])
    }
}

impl ValidatedMeasurement {
    pub fn flex_composition_image(&self) -> Option<FlexCompositionImage> {
        self.flex_composition_image
    }
    pub fn has_prepared_range_frame(&self) -> bool {
        self.prepared_range_frame
    }
    pub fn flex_content_text(&self) -> Option<&FlexContentText> {
        self.flex_content_text.as_ref()
    }
    pub fn measured_leaf(&self) -> Option<&Attributes> {
        self.measured_leaf.as_ref()
    }
    pub fn element_scroll_sizes(&self) -> Option<&[ElementScrollSize]> {
        self.element_scroll_sizes.as_deref()
    }
    pub fn observation_frame(&self) -> &'static str {
        if self.root_local_positioned {
            "root-border-local"
        } else {
            "root-border"
        }
    }
    pub fn use_rounding(&self) -> bool {
        self.use_rounding
    }
    pub fn viewport(&self) -> &Attributes {
        &self.viewport
    }
    pub fn root(&self) -> &Node {
        &self.root
    }
}
impl Node {
    pub fn world_rounded_layout(&self) -> Option<&Attributes> {
        self.world_rounded_layout.as_ref()
    }
    pub fn prepared_paragraph(&self) -> Option<&str> {
        self.prepared_paragraph.as_deref()
    }
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }
    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }
    pub fn children(&self) -> &[Node] {
        &self.children
    }
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
    pub fn observation(&self) -> &Observation {
        &self.observation
    }
    pub fn atomic(&self) -> Option<&Attributes> {
        self.atomic.as_ref()
    }
    pub fn control_expectation(&self) -> Option<&Attributes> {
        self.control_expectation.as_ref()
    }
    pub fn br_font_band(&self) -> Option<&Geometry> {
        self.br_font_band.as_ref()
    }
    pub fn bands(&self) -> Option<&[Attributes]> {
        self.bands.as_deref()
    }
}
impl Geometry {
    pub fn x(&self) -> f64 {
        self.x
    }
    pub fn y(&self) -> f64 {
        self.y
    }
    pub fn width(&self) -> f64 {
        self.width
    }
    pub fn height(&self) -> f64 {
        self.height
    }
}

#[cfg(test)]
pub(super) fn style_attributes(raw: &str) -> Result<Attributes, MeasurementError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct StyleFixture {
        #[serde(default, deserialize_with = "present")]
        tag_name: Option<String>,
        style: Style,
    }
    let fixture: StyleFixture = serde_json::from_str(raw).map_err(test_decode_error)?;
    let context = Context {
        case_id: "style-test",
        variant: "test",
        node_path: "root".into(),
    };
    fixture.style.validate_wire(&context)?;
    let mut attrs = fixture
        .tag_name
        .map(|value| vec![("source-tag", value)])
        .unwrap_or_default();
    attrs.extend(
        fixture
            .style
            .attrs(&context, "horizontal-tb", false, true)?,
    );
    Ok(attrs)
}
#[cfg(test)]
fn test_decode_error(source: serde_json::Error) -> MeasurementError {
    MeasurementError {
        case_id: "style-test".into(),
        variant: "test".into(),
        node_path: "root".into(),
        field_path: String::new(),
        kind: MeasurementErrorKind::Decode,
        detail: Box::new(ErrorDetail {
            message: source.to_string(),
            source: Some(source),
        }),
    }
}
#[cfg(test)]
pub(super) fn dimension_attribute(raw: &str) -> Result<String, MeasurementError> {
    let value: Dimension = serde_json::from_str(raw).map_err(test_decode_error)?;
    value.validate(
        &Context {
            case_id: "style-test",
            variant: "test",
            node_path: "root".into(),
        },
        "dimension",
    )
}
#[cfg(test)]
pub(super) fn track_attribute(raw: &str) -> Result<String, MeasurementError> {
    style::test_track(raw)
}
#[cfg(test)]
pub(super) fn position_attribute(raw: &str) -> Result<Option<String>, MeasurementError> {
    style::test_position(raw)
}

#[cfg(test)]
#[path = "measurement/tests.rs"]
mod tests;

/// Project the browser-only line-control facts before the infallible XML phase.
fn prepare_control_expectations(
    writing_mode: &str,
    children: &mut [Node],
    context: &Context<'_>,
) -> Result<(), MeasurementError> {
    let indices = children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (!matches!(node.kind, NodeKind::Boundary(_))).then_some(index))
        .collect::<Vec<_>>();
    for (source_index, &index) in indices.iter().enumerate() {
        if !matches!(children[index].kind, NodeKind::Control)
            && children[index].br_font_band.is_none()
        {
            continue;
        }
        let control = block_interval(writing_mode, &children[index]).ok_or_else(|| {
            context.child(index).error(
                "unroundedLayout",
                MeasurementErrorKind::MissingField,
                "browser controls require geometry",
            )
        })?;
        let candidates = indices[..source_index]
            .iter()
            .rev()
            .take_while(|&&index| {
                !matches!(children[index].kind, NodeKind::Control)
                    && children[index].br_font_band.is_none()
            })
            .map(|&index| &children[index])
            .collect::<Vec<_>>();
        let terminal = candidates
            .iter()
            .all(|node| block_interval(writing_mode, node).is_some())
            .then(|| {
                candidates
                    .iter()
                    .filter(|node| {
                        block_interval(writing_mode, node).is_some_and(|interval| {
                            block_relation(writing_mode, control, interval) == "same"
                        })
                    })
                    .count()
            });
        let previous = source_index
            .checked_sub(1)
            .map(|index| block_interval(writing_mode, &children[indices[index]]))
            .map_or("absent", |interval| {
                interval.map_or("unobserved", |interval| {
                    block_relation(writing_mode, control, interval)
                })
            });
        let next = indices
            .get(source_index + 1)
            .map(|&index| block_interval(writing_mode, &children[index]))
            .map_or("absent", |interval| {
                interval.map_or("unobserved", |interval| {
                    block_relation(writing_mode, control, interval)
                })
            });
        children[index].control_expectation = Some(vec![
            ("source_index", source_index.to_string()),
            (
                "terminal_visual_slot",
                terminal.map_or_else(|| "unobserved".into(), |value| value.to_string()),
            ),
            ("previous_line", previous.into()),
            ("next_line", next.into()),
        ]);
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct BlockInterval {
    minimum: f64,
    maximum: f64,
}
fn block_interval(writing_mode: &str, node: &Node) -> Option<BlockInterval> {
    let Observation::Geometry { unrounded, .. } = node.observation else {
        return None;
    };
    let (minimum, extent) = if writing_mode == "horizontal-tb" {
        (unrounded.y, unrounded.height)
    } else {
        (unrounded.x, unrounded.width)
    };
    // Both operands have already been bounded to finite f32 geometry. Their
    // sum and the center computation below are therefore representable in f64.
    Some(BlockInterval {
        minimum,
        maximum: minimum + extent,
    })
}
fn block_relation(
    writing_mode: &str,
    control: BlockInterval,
    neighbor: BlockInterval,
) -> &'static str {
    if neighbor.maximum >= control.minimum && control.maximum >= neighbor.minimum {
        return "same";
    }
    let control_center = control.minimum + (control.maximum - control.minimum) / 2.0;
    let neighbor_center = neighbor.minimum + (neighbor.maximum - neighbor.minimum) / 2.0;
    let earlier = if matches!(writing_mode, "vertical-rl" | "sideways-rl") {
        neighbor_center > control_center
    } else {
        neighbor_center < control_center
    };
    if earlier { "earlier" } else { "later" }
}

fn split_decode_path(path: &str) -> (String, String) {
    let mut node = "root".to_string();
    let mut remainder = path;
    while let Some(child) = remainder.strip_prefix("children[") {
        let Some(end) = child.find(']') else {
            break;
        };
        node.push_str(".children[");
        node.push_str(&child[..=end]);
        remainder = child[end + 1..]
            .strip_prefix('.')
            .unwrap_or(&child[end + 1..]);
    }
    (node, remainder.to_string())
}

fn bounded_index(
    value: u64,
    context: &Context<'_>,
    field: &str,
) -> Result<String, MeasurementError> {
    usize::try_from(value)
        .map(|index| index.to_string())
        .map_err(|_| {
            context.error(
                field,
                MeasurementErrorKind::OutOfRange,
                "index exceeds the fixture consumer capacity",
            )
        })
}

fn validate_break_alternative(
    kind: &BreakKind,
    replacement: Option<f64>,
    context: &Context<'_>,
    prefix: &str,
) -> Result<(), MeasurementError> {
    context.compatible(
        (*kind == BreakKind::AllowedWithReplacement) == replacement.is_some(),
        &format!("{prefix}.replacementInlineExtent"),
        "replacement extent belongs only to allowed-with-replacement breaks",
    )
}

fn validate_xml_text(
    value: &str,
    context: &Context<'_>,
    field: &str,
) -> Result<(), MeasurementError> {
    context.ensure(value.chars().all(|value|matches!(value,'\u{9}'|'\u{A}'|'\u{D}'|'\u{20}'..='\u{D7FF}'|'\u{E000}'..='\u{FFFD}'|'\u{10000}'..='\u{10FFFF}')),field,"text must contain only XML 1.0 characters")
}

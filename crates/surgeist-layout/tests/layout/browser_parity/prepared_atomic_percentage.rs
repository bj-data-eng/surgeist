//! Admission and measurement of the original authored atomic percentage.

use super::{
    Error, Node, RootContext, Scalar, StyleAttrs, capture_authority, layout, root_request,
};
use crate::{pinned_font, prepared_atomic_percentage_profile as profile};
use capture_authority::{CaptureMetadata, CapturedArtifact, sha256};

pub(super) struct Capture<'a> {
    pub report: &'a str,
    pub source_path: &'a str,
    pub source_bytes: &'a [u8],
    pub output_path: &'a str,
}

fn invalid(detail: impl std::fmt::Display) -> Error {
    Error::new(format!("Prepared atomic percentage: {detail}"))
}

#[derive(Clone, Copy)]
pub(super) struct Variant {
    box_sizing: layout::BoxSizing,
    direction: layout::Direction,
}

pub(super) fn admit(capture: Capture<'_>, name: &str, xml: &str) -> Result<Variant, Error> {
    if capture.source_path != profile::SOURCE
        || sha256(capture.source_bytes) != profile::SOURCE_SHA256
    {
        return Err(invalid("original source changed"));
    }
    let report = capture_authority::unique_json(capture.report).map_err(invalid)?;
    let metadata: CaptureMetadata =
        serde_json::from_value(report["metadata"].clone()).map_err(invalid)?;
    let inputs = [
        (profile::SOURCE, profile::SOURCE_SHA256.to_owned()),
        ("observation-request.json", sha256(profile::REQUEST)),
        (profile::SCRIPT_PATH, sha256(profile::SCRIPT.as_bytes())),
        ("fonts/Ahem.ttf", pinned_font::SHA256.to_owned()),
        (
            "scripts/gentest/test_base_style.css",
            pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256.to_owned(),
        ),
    ];
    if metadata.schema_version != 4
        || !metadata
            .inputs
            .get(capture_authority::ROOT_HELPER_INPUT)
            .is_some_and(|digest| profile::accepts_helper(digest))
        || inputs
            .iter()
            .any(|(path, digest)| metadata.inputs.get(*path) != Some(digest))
    {
        return Err(invalid("source or capture inputs changed"));
    }
    let artifacts = report["generated"]
        .as_array()
        .ok_or_else(|| invalid("missing generated artifacts"))?;
    let mut matches = artifacts
        .iter()
        .filter(|artifact| artifact["name"] == name || artifact["output"] == capture.output_path);
    let artifact: CapturedArtifact = serde_json::from_value(
        matches
            .next()
            .ok_or_else(|| invalid("missing captured output"))?
            .clone(),
    )
    .map_err(invalid)?;
    if matches.next().is_some()
        || artifact.name != name
        || artifact.source != profile::SOURCE
        || artifact.output != capture.output_path
        || artifact.output != format!("xml/block/{name}.xml")
        || artifact.name
            != format!(
                "fri06_atomic_inline_percentage_block_size__{}",
                artifact.variant
            )
        || artifact.source_sha256 != profile::SOURCE_SHA256
        || artifact.output_sha256 != sha256(xml.as_bytes())
        || !artifact.linked_resources.is_empty()
    {
        return Err(invalid("capture source or output binding changed"));
    }
    use layout::{
        BoxSizing::{BorderBox, ContentBox},
        Direction::{Ltr, Rtl},
    };
    let (box_sizing, direction) = match artifact.variant.as_str() {
        "border_box_ltr" => (BorderBox, Ltr),
        "border_box_rtl" => (BorderBox, Rtl),
        "content_box_ltr" => (ContentBox, Ltr),
        "content_box_rtl" => (ContentBox, Rtl),
        _ => return Err(invalid("unregistered capture variant")),
    };
    Ok(Variant {
        box_sizing,
        direction,
    })
}

pub(super) fn validate_frame(context: &RootContext) -> Result<(), Error> {
    use super::root_context::QualifiedRootContext;
    let RootContext::Qualified(QualifiedRootContext::AbsoluteRootLocal {
        containing_size,
        containing_direction: layout::Direction::Ltr,
        hypothetical_direction: layout::Direction::Ltr,
        hypothetical_display,
        hypothetical_inline_offset: 0.0,
        ..
    }) = context
    else {
        return Err(invalid("containing frame differs from the fixed capture"));
    };
    // The V4 reader checks the ICB against the containing extent. The fixed
    // browser edition supplies the independent extent and reference sides.
    if containing_size.width.get() != 800.0
        || containing_size.height.get() != 600.0
        || hypothetical_display != "block"
    {
        return Err(invalid("containing frame differs from the fixed capture"));
    }
    Ok(())
}

pub(super) fn validate_source(raw: &str) -> Result<(), Error> {
    let raw = capture_authority::unique_json(raw).map_err(invalid)?;
    let before: Vec<_> = "\n  basis ".encode_utf16().collect();
    let after: Vec<_> = " definite\n".encode_utf16().collect();
    let occurrences = serde_json::json!([before, after]);
    let [first, second] = raw["origins"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
    else {
        return Err(invalid("source occurrence census changed"));
    };
    if first["units"] != occurrences[0]
        || second["units"] != occurrences[1]
        || first["child"] != 0
        || second["child"] != 2
        || raw["baseStyle"]["fontSize"].as_f64() != Some(16.0)
        || raw["baseStyle"]["lineHeight"].as_f64() != Some(20.0)
        || first["style"] != raw["baseStyle"]
        || second["style"] != raw["baseStyle"]
        || raw["atomics"].as_array().map(Vec::len) != Some(1)
        || raw["atomics"][0]["authoredSize"] != serde_json::json!({"width":"20px","height":"50%"})
    {
        return Err(invalid("authored source or font metrics changed"));
    }
    Ok(())
}

const ROOT: usize = 0;
const ATOMIC: usize = 2;
const CHILDREN: [usize; 3] = [1, ATOMIC, 3];

// The source occurrences retain their actual child identities. They have no
// Layout input until Text is prepared. The size-only query reads two boxes.
struct BoxProjection {
    container: layout::NodeInput,
    atomic: layout::NodeInput,
}

impl layout::Traverse for BoxProjection {
    type Node = usize;
    type Scalar = Scalar;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children = if node == ROOT {
            CHILDREN.as_slice()
        } else {
            &[]
        };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        self.children(node).len()
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).expect("actual child index")
    }
}

impl layout::LayoutTree for BoxProjection {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        match node {
            ROOT => &self.container,
            ATOMIC => &self.atomic,
            _ => unreachable!("unprepared text has no box input"),
        }
    }

    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        layout::LayoutInputView::Box(self.node_input(node))
    }
}

pub(super) fn measure(
    context: &RootContext,
    variant: Variant,
    root: &StyleAttrs,
    children: &[Node],
    child: usize,
    width: &str,
    height: &str,
) -> Result<surgeist_text::Size, Error> {
    let [before, atomic, after] = children else {
        return Err(invalid(
            "original containing frame has three direct children",
        ));
    };
    if child != 1
        || before.prepared_occurrence != Some(0)
        || after.prepared_occurrence != Some(1)
        || !atomic.children.is_empty()
        || atomic.style.get("source-tag") != Some("span")
        || atomic.atomic_inline_participation.is_some()
    {
        return Err(invalid(
            "authored atomic does not name the original in-flow child",
        ));
    }
    let container = super::to_node_input(root)?;
    let atomic = super::to_node_input_in_flow(
        &atomic.style,
        Some(layout::FlowAxes::new(
            container.writing_mode,
            container.direction,
        )),
    )?;
    if container.box_sizing != variant.box_sizing
        || atomic.box_sizing != variant.box_sizing
        || container.direction != variant.direction
        || atomic.direction != variant.direction
    {
        return Err(invalid(
            "effective root or atomic styles differ from the capture variant",
        ));
    }
    if container.size
        != layout::Size::new(
            super::parse_preferred_size("180px")?,
            super::parse_preferred_size("80px")?,
        )
        || atomic.size
            != layout::Size::new(
                super::parse_preferred_size(width)?,
                super::parse_preferred_size(height)?,
            )
    {
        return Err(invalid(
            "actual authored box sizing differs from the source",
        ));
    }
    let size = layout::measure_definite_inline_atomic_size(
        &BoxProjection { container, atomic },
        ROOT,
        ATOMIC,
        root_request(context)?,
    )
    .map_err(|error| invalid(format!("{error:?}")))?;
    Ok(surgeist_text::Size::new(
        f64::from(size.width),
        f64::from(size.height),
    ))
}

use super::exact_differences::{
    QualificationError, QualifiedComparison, QualifiedVerdict, RawVerdict,
};
use super::prepared_range_qualification::Comparisons;
use super::{Expectation, Golden, NumericComparisonSink, PhysicalStartEdge};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Basis {
    root: Box<Node>,
    context: RootContext,
}

impl Basis {
    pub(super) fn new(root: &Node, context: &RootContext) -> Self {
        Self {
            root: Box::new(root.clone()),
            context: context.clone(),
        }
    }

    pub(super) fn validate(&self, golden: &Golden) -> Result<(), Error> {
        if *self.root != golden.root || self.context != golden.viewport.root_context {
            return Err(invalid("prepared atomic sizing frame changed"));
        }
        Ok(())
    }
}

fn dimensions(row: &Expectation) -> (Option<Scalar>, Option<Scalar>) {
    (row.width, row.height)
}

pub(crate) fn verify_historical(current: &Golden, historical: &Golden) -> Result<(), Error> {
    let [_, current_atomic, _] = current.expectations.children.as_slice() else {
        return Err(invalid("current atomic observation is missing"));
    };
    let [_, historical_atomic, _] = historical.expectations.children.as_slice() else {
        return Err(invalid("historical atomic observation is missing"));
    };
    // The new face may change line breaks and positions. The authored root
    // and atomic have the same observed size in both captures.
    if current.name != historical.name
        || !historical.use_rounding
        || dimensions(&current.expectations) != dimensions(&historical.expectations)
        || dimensions(current_atomic) != dimensions(historical_atomic)
    {
        return Err(invalid("historical box size observation changed"));
    }
    Ok(())
}

fn validate_observations(golden: &Golden) -> Result<(), Error> {
    let root = &golden.expectations;
    let [first, atomic, second] = root.children.as_slice() else {
        return Err(invalid("original observation child census changed"));
    };
    let complete_box = |row: &Expectation| {
        row.x.is_some()
            && row.y.is_some()
            && row.width.is_some()
            && row.height.is_some()
            && row.range_rectangles.is_none()
    };
    if golden.prepared_atomic_basis.is_none()
        || !golden.has_prepared_range_frame()
        || !golden.use_rounding
        || !complete_box(root)
        || !complete_box(atomic)
        || ![first, atomic, second].iter().all(|row| row.children.is_empty())
        || ![first, second].iter().enumerate().all(|(index, row)| {
            row.range_rectangles.as_deref().is_some_and(|rows| {
                matches!(rows, [row] if row.source_segment_id == index as u64
                    && row.line_index == index && row.physical_start_edge == PhysicalStartEdge::Left)
            })
        })
    {
        return Err(invalid("incomplete box, line or Range observations"));
    }
    Ok(())
}

fn verify_native<'batch>(
    tree: &super::TestTree,
    batch: &'batch layout::CompletedLayoutBatch<usize>,
) -> Result<&'batch super::ObservedPlacement, Error> {
    let [binding] = tree.paragraphs.as_slice() else {
        return Err(invalid("expected one actual prepared paragraph"));
    };
    let [bound] = binding.atomics.as_slice() else {
        return Err(invalid("expected one actual bound atomic"));
    };
    let [placed] = batch.placed_text() else {
        return Err(invalid("expected one actual placed paragraph"));
    };
    let text = placed.layout();
    let atomics = text.inline_boxes();
    let [atomic] = atomics.as_slice() else {
        return Err(invalid("expected one actual allocated atomic"));
    };
    let original = *bound.occurrence.view().original_box();
    let native = &tree.nodes[ATOMIC];
    if bound.node != ATOMIC
        || placed.container() != ROOT
        || placed.nodes() != CHILDREN
        || original.size() != surgeist_text::Size::new(20.0, 40.0)
        || atomic.id() != original.id()
        || atomic.index() != original.index()
        || atomic.rect().size() != original.size()
        || atomic.line() != 0
        || text.lines().len() != 2
        || text.analyzed_source().identity() != binding.prepared.analyzed_source().identity()
        || text
            .run_views()
            .any(|run| run.face().bytes() != pinned_font::BYTES || run.face().index() != 0)
        || !native.unrounded_present
        || native.unrounded.size != layout::Size::new(20.0, 40.0)
    {
        return Err(invalid(
            "prepared, allocated and actual atomic or font custody changed",
        ));
    }
    Ok(placed)
}

pub(crate) fn compare(
    golden: &Golden,
    observe_placement: impl FnOnce(&super::ObservedPlacement) -> Result<(), Error>,
) -> Result<(Vec<String>, Result<QualifiedComparison, QualificationError>), Error> {
    validate_observations(golden)?;
    let mut numeric =
        NumericComparisonSink::PreparedAtomicPercentage(Comparisons::new(golden.name.clone()));
    let mut custody = None;
    let result = super::compare_captured_with_batch(golden, &mut numeric, |tree, batch| {
        custody = Some(verify_native(tree, batch).and_then(observe_placement));
        Ok(())
    })?;
    if !result.endpoint_unobservable_fields().is_empty() {
        return Err(invalid("incomplete Native observations"));
    }
    let NumericComparisonSink::PreparedAtomicPercentage(comparisons) = numeric else {
        unreachable!("prepared atomic percentage collector remains selected");
    };
    let fields = comparisons.differences();
    let qualified = custody
        .unwrap_or_else(|| Err(invalid("missing completed Native batch")))
        .map_err(|error| QualificationError::ComparisonFailure {
            reason: error.to_string(),
        })
        .and_then(|()| {
            if fields.is_empty() {
                Ok(QualifiedComparison {
                    raw: RawVerdict::Matches,
                    qualified: QualifiedVerdict::MatchingControl,
                })
            } else {
                Err(QualificationError::DifferenceSetChanged {
                    missing: Vec::new(),
                    extra: fields.clone(),
                    changed: Vec::new(),
                })
            }
        });
    Ok((fields, qualified))
}

#[cfg(test)]
#[path = "prepared_atomic_percentage_tests.rs"]
mod tests;

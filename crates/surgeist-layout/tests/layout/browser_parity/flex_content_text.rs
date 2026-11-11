//! Prepared Text inputs for authenticated Flex captures.

use super::{
    Error, Golden, InheritedTextContext, LineHeightState, TestTree, capture_authority, layout,
    pinned_font,
};
use surgeist_text as text;

use crate::fri07_intrinsic_flex_profile as intrinsic;
use crate::fri08_grid_lanes_text_profile as grid_lanes;
use crate::fri08_wrapped_subgrid_profile as wrapped;

pub(super) const PROFILE: &str = "ahem-10px-flex-direct-text";
const STYLESHEET: &[u8] = include_bytes!("scripts/gentest/test_base_style.css");
pub(super) const REQUEST: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/flex_content_text_request.json");
const SCRIPT: &[u8] =
    include_bytes!("../../bin/surgeist-layout-generate/measurement/flex_content_text.js");
pub(super) const SCRIPT_PATH: &str = "scripts/gentest/flex_content_text.js";
pub(super) const CAPTURE_INPUTS: [(&str, &[u8]); 3] = [
    ("scripts/gentest/test_base_style.css", STYLESHEET),
    ("observation-request.json", REQUEST),
    (SCRIPT_PATH, SCRIPT),
];
pub(super) const GRID_LANES_TEXT_CAPTURE_INPUTS: [(&str, &[u8]); 3] = [
    ("scripts/gentest/test_base_style.css", STYLESHEET),
    ("observation-request.json", grid_lanes::REQUEST),
    (grid_lanes::SCRIPT_PATH, grid_lanes::SCRIPT.as_bytes()),
];
pub(super) const WRAPPED_SUBGRID_CAPTURE_INPUTS: [(&str, &[u8]); 3] = [
    ("scripts/gentest/test_base_style.css", STYLESHEET),
    ("observation-request.json", wrapped::REQUEST),
    (wrapped::SCRIPT_PATH, wrapped::SCRIPT.as_bytes()),
];
pub(super) const INTRINSIC_CAPTURE_INPUTS: [(&str, &[u8]); 3] = [
    ("scripts/gentest/test_base_style.css", STYLESHEET),
    ("observation-request.json", intrinsic::REQUEST),
    (intrinsic::SCRIPT_PATH, intrinsic::SCRIPT.as_bytes()),
];

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Proof {
    occurrences: Vec<SourceOccurrence>,
    source: Box<super::Node>,
    viewport: super::Viewport,
}

impl Proof {
    pub(super) fn is_wrapped_subgrid(&self) -> bool {
        self.occurrences
            .first()
            .is_some_and(|occurrence| matches!(occurrence.path, SourcePath::SubgridLeaf { .. }))
    }

    pub(super) fn is_grid_lanes_text(&self) -> bool {
        self.occurrences
            .first()
            .is_some_and(|occurrence| matches!(occurrence.path, SourcePath::GridLanesItem { .. }))
    }

    pub(super) fn matches_source(&self, golden: &Golden) -> bool {
        golden.root == *self.source && golden.viewport == self.viewport
    }

    pub(super) fn occurrences(&self) -> &[SourceOccurrence] {
        &self.occurrences
    }

    pub(super) fn is_intrinsic(&self) -> bool {
        self.occurrences
            .first()
            .is_some_and(|occurrence| matches!(occurrence.path, SourcePath::RowItem { .. }))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct SourceOccurrence {
    pub(super) path: SourcePath,
    pub(super) direction: layout::Direction,
    pub(super) words: Vec<intrinsic::WordRange>,
    pub(super) clip: Option<[u32; 4]>,
    pub(super) sizes: Option<grid_lanes::Sizes>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SourcePath {
    Child(usize),
    RowItem { row: usize, item: usize },
    GridLanesItem { row: usize, item: usize },
    SubgridLeaf { card: usize, item: usize },
}

impl std::fmt::Display for SourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Child(index) => write!(f, "/{index}"),
            Self::RowItem { row, item } | Self::GridLanesItem { row, item } => {
                write!(f, "/{row}/{item}")
            }
            Self::SubgridLeaf { card, item: 0 } => write!(f, "/{card}/0"),
            Self::SubgridLeaf { card, item: 1 } => write!(f, "/{card}/1/0"),
            Self::SubgridLeaf { card, item } => write!(f, "/{card}/invalid-{item}"),
        }
    }
}

impl SourcePath {
    pub(super) fn locate(self, tree: &TestTree) -> Option<(usize, usize, usize)> {
        let (parent, index) = match self {
            Self::Child(index) => (0, index),
            Self::RowItem { row, item } | Self::GridLanesItem { row, item } => {
                (*tree.nodes.first()?.children.get(row)?, item)
            }
            Self::SubgridLeaf { card, item } => {
                let card = *tree.nodes.first()?.children.get(card)?;
                match item {
                    0 => (card, 0),
                    1 => (*tree.nodes.get(card)?.children.get(1)?, 0),
                    _ => return None,
                }
            }
        };
        let source = *tree.nodes.get(parent)?.children.get(index)?;
        Some((parent, index, source))
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LoweredOccurrence {
    pub(super) path: SourcePath,
    pub(super) source: usize,
    pub(super) wrapper: usize,
    pub(super) text: usize,
}

fn invalid(reason: impl std::fmt::Display) -> Error {
    Error::new(format!("Flex content Text: {reason}"))
}

pub(super) fn bind(
    golden: &Golden,
    xml: &str,
    report: &str,
    source_path: &str,
    source_bytes: &[u8],
    output_path: &str,
) -> Result<Proof, Error> {
    let doc = roxmltree::Document::parse(xml).map_err(invalid)?;
    let root = doc.root_element();
    let mut proofs = root
        .descendants()
        .filter(|node| node.has_tag_name("flex-content-text"));
    let proof = proofs
        .next()
        .ok_or_else(|| invalid("missing occurrence proof"))?;
    let profile = proof.attribute("profile");
    let inputs = match profile {
        Some(PROFILE) => CAPTURE_INPUTS,
        Some(intrinsic::PROFILE) => INTRINSIC_CAPTURE_INPUTS,
        Some(wrapped::PROFILE) => WRAPPED_SUBGRID_CAPTURE_INPUTS,
        Some(grid_lanes::PROFILE) => GRID_LANES_TEXT_CAPTURE_INPUTS,
        _ => return Err(invalid("unsupported Flex Text profile")),
    };
    let direct = profile == Some(PROFILE);
    let wrapped_subgrid = profile == Some(wrapped::PROFILE);
    let intrinsic = profile == Some(intrinsic::PROFILE);
    let measured_grid_lanes = profile == Some(grid_lanes::PROFILE);
    let root_display = if measured_grid_lanes {
        "block"
    } else if wrapped_subgrid {
        "grid"
    } else {
        "flex"
    };
    let valid_font = if measured_grid_lanes {
        proof.attributes().len() == 1
    } else {
        proof.attributes().len() == 2
            && proof.attribute("font-resource-sha256")
                == Some(pinned_font::AHEM_STYLESHEET_WOFF2_SHA256)
            && capture_authority::sha256(pinned_font::AHEM_STYLESHEET_WOFF2)
                == pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
    };
    if proofs.next().is_some()
        || proof.parent() != Some(root)
        || root.attribute("measurement-version") != Some("4")
        || root.attribute("observation-frame") != Some("root-border-local")
        || !valid_font
        || !golden.use_rounding
        || golden.has_element_scroll_sizes() != direct
        || golden.root.style.get("display") != Some(root_display)
        || golden.root.text.is_some()
    {
        return Err(invalid("unsupported occurrence proof or root profile"));
    }
    let report = capture_authority::unique_json(report).map_err(invalid)?;
    let metadata: capture_authority::CaptureMetadata =
        serde_json::from_value(report["metadata"].clone()).map_err(invalid)?;
    if metadata.schema_version != 4
        || !metadata
            .inputs
            .get(capture_authority::ROOT_HELPER_INPUT)
            .is_some_and(|hash| capture_authority::root_helper_profile(hash).is_ok())
        || (!direct
            && !metadata
                .inputs
                .get(capture_authority::ROOT_HELPER_INPUT)
                .is_some_and(|hash| {
                    hash == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
                }))
        || (intrinsic
            && (source_path != intrinsic::SOURCE
                || capture_authority::sha256(source_bytes) != intrinsic::SOURCE_SHA256))
        || (wrapped_subgrid
            && wrapped::source_digest(source_path)
                != Some(capture_authority::sha256(source_bytes).as_str()))
        || (measured_grid_lanes
            && grid_lanes::source_digest(source_path)
                != Some(capture_authority::sha256(source_bytes).as_str()))
        || capture_authority::sha256(STYLESHEET) != pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256
        || inputs
            .into_iter()
            .chain([(source_path, source_bytes)])
            .any(|(path, bytes)| {
                metadata.inputs.get(path) != Some(&capture_authority::sha256(bytes))
            })
    {
        return Err(invalid("source, font, or capture inputs changed"));
    }
    let artifacts = report["generated"]
        .as_array()
        .ok_or_else(|| invalid("missing capture report artifacts"))?;
    let mut matches = artifacts
        .iter()
        .filter(|artifact| artifact["name"] == golden.name || artifact["output"] == output_path);
    let artifact: capture_authority::CapturedArtifact = serde_json::from_value(
        matches
            .next()
            .ok_or_else(|| invalid("missing captured output"))?
            .clone(),
    )
    .map_err(invalid)?;
    if matches.next().is_some()
        || artifact.name != golden.name
        || artifact.source != source_path
        || artifact.output != output_path
        || artifact.source_sha256 != capture_authority::sha256(source_bytes)
        || artifact.output_sha256 != capture_authority::sha256(xml.as_bytes())
        || !artifact.linked_resources.is_empty()
    {
        return Err(invalid("capture source or output binding changed"));
    }
    let occurrences = if measured_grid_lanes {
        bind_grid_lanes_occurrences(golden, proof)?
    } else if wrapped_subgrid {
        bind_wrapped_occurrences(golden, proof)?
    } else if intrinsic {
        bind_intrinsic_occurrences(golden, proof)?
    } else {
        bind_direct_occurrences(golden, proof)?
    };
    Ok(Proof {
        occurrences,
        source: Box::new(golden.root.clone()),
        viewport: golden.viewport.clone(),
    })
}

fn bind_direct_occurrences(
    golden: &Golden,
    proof: roxmltree::Node<'_, '_>,
) -> Result<Vec<SourceOccurrence>, Error> {
    let rows = proof
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    if rows.is_empty() || rows.len() != golden.root.children.len() {
        return Err(invalid("incomplete occurrence census"));
    }
    rows.into_iter()
        .zip(&golden.root.children)
        .enumerate()
        .map(|(index, (row, child))| {
            let direction = row.attribute("direction");
            if !row.has_tag_name("occurrence")
                || row.attributes().len() != 2
                || row.children().any(|node| node.is_element())
                || row.attribute("child") != Some(index.to_string().as_str())
                || child.style.get("direction") != direction
                || child.style.get("display") != Some("flex")
                || child.style.get("source-tag") != Some("div")
                || !child.children.is_empty()
                || child.prepared.is_some()
                || !child.text.as_ref().is_some_and(|value| {
                    value.split(' ').all(|word| {
                        !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_alphabetic())
                    })
                })
            {
                return Err(invalid(
                    "occurrence does not identify its direct source label",
                ));
            }
            Ok(SourceOccurrence {
                path: SourcePath::Child(index),
                direction: occurrence_direction(direction)?,
                words: Vec::new(),
                clip: None,
                sizes: None,
            })
        })
        .collect()
}

fn occurrence_direction(direction: Option<&str>) -> Result<layout::Direction, Error> {
    match direction {
        Some("ltr") => Ok(layout::Direction::Ltr),
        Some("rtl") => Ok(layout::Direction::Rtl),
        _ => Err(invalid("unsupported occurrence direction")),
    }
}

fn bind_intrinsic_occurrences(
    golden: &Golden,
    proof: roxmltree::Node<'_, '_>,
) -> Result<Vec<SourceOccurrence>, Error> {
    if golden.root.style.get("flex-direction") != Some("column")
        || golden.root.children.len() != intrinsic::LABELS.len()
    {
        return Err(invalid("unexpected intrinsic Flex root"));
    }
    let mut evidence = proof.children().filter(roxmltree::Node::is_element);
    let mut occurrences = Vec::new();
    for (row_index, (row, label)) in golden
        .root
        .children
        .iter()
        .zip(intrinsic::LABELS)
        .enumerate()
    {
        if row.style.get("display") != Some("flex")
            || !matches!(row.style.get("flex-direction"), None | Some("row"))
            || row.style.get("source-tag") != Some("div")
            || row.children.len() != 2
            || row.text.is_some()
            || row.prepared.is_some()
        {
            return Err(invalid(format!("unexpected original row /{row_index}")));
        }
        for (item_index, item) in row.children.iter().enumerate() {
            let path = SourcePath::RowItem {
                row: row_index,
                item: item_index,
            };
            let occurrence = evidence
                .next()
                .ok_or_else(|| invalid(format!("missing {path}")))?;
            let direction = occurrence.attribute("direction");
            let basis = if item_index == 0 {
                "min-content"
            } else {
                "max-content"
            };
            if !occurrence.has_tag_name("occurrence")
                || occurrence.attributes().len() != 4
                || occurrence.children().next().is_some()
                || occurrence.attribute("row") != Some(row_index.to_string().as_str())
                || occurrence.attribute("item") != Some(item_index.to_string().as_str())
                || item.style.get("direction") != direction
                || item.style.get("display") != Some("flex")
                || item.style.get("source-tag") != Some("div")
                || item.style.get("flex-basis") != Some(basis)
                || item.style.get("min-width") != Some("0px")
                || item.style.get("flex-grow") != Some("1")
                || item.style.get("flex-shrink").unwrap_or("1") != "1"
                || !item.children.is_empty()
                || item.prepared.is_some()
                || item.text.as_deref() != Some(label)
            {
                return Err(invalid(format!(
                    "unexpected original Flex Text occurrence {path}"
                )));
            }
            let words = occurrence
                .attribute("words")
                .ok_or_else(|| invalid(format!("missing word Ranges for {path}")))?;
            let words: Vec<intrinsic::WordRange> = serde_json::from_str(words).map_err(invalid)?;
            if !intrinsic::valid_words(label, &words) {
                return Err(invalid(format!("invalid word Ranges for {path}")));
            }
            occurrences.push(SourceOccurrence {
                path,
                direction: occurrence_direction(direction)?,
                words,
                clip: None,
                sizes: None,
            });
        }
    }
    if evidence.next().is_some() {
        return Err(invalid("extra intrinsic Flex Text occurrence"));
    }
    Ok(occurrences)
}

fn bind_wrapped_occurrences(
    golden: &Golden,
    proof: roxmltree::Node<'_, '_>,
) -> Result<Vec<SourceOccurrence>, Error> {
    if golden.root.children.len() != 3 {
        return Err(invalid("unexpected wrapped Subgrid root census"));
    }
    let mut rows = proof.children().filter(roxmltree::Node::is_element);
    let mut occurrences = Vec::new();
    for (card, node) in golden.root.children.iter().enumerate() {
        let [header, footer] = node.children.as_slice() else {
            return Err(invalid("unexpected wrapped Subgrid card census"));
        };
        let [text_box] = footer.children.as_slice() else {
            return Err(invalid("unexpected wrapped Subgrid footer census"));
        };
        if [node, footer].into_iter().any(|grid| {
            grid.style.get("source-tag") != Some("div")
                || grid.style.get("display") != Some("grid")
                || grid.style.get("grid-template-rows") != Some("subgrid")
                || grid.text.is_some()
        }) {
            return Err(invalid("original row Subgrid inputs changed"));
        }
        for (item, (node, label)) in [header, text_box]
            .into_iter()
            .zip(wrapped::LABELS)
            .enumerate()
        {
            let row = rows
                .next()
                .ok_or_else(|| invalid("missing wrapped Text occurrence"))?;
            let direction = occurrence_direction(row.attribute("direction"))?;
            let words: Vec<intrinsic::WordRange> = serde_json::from_str(
                row.attribute("ranges")
                    .ok_or_else(|| invalid("missing wrapped source Ranges"))?,
            )
            .map_err(invalid)?;
            let clip: Option<[u32; 4]> = row
                .attribute("clip")
                .map(serde_json::from_str)
                .transpose()
                .map_err(invalid)?;
            let expected_overflow = if item == 0 { "visible" } else { "hidden" };
            if !row.has_tag_name("occurrence")
                || row.attributes().len() != if item == 0 { 4 } else { 5 }
                || row.children().any(|node| node.is_element())
                || row.attribute("card") != Some(card.to_string().as_str())
                || row.attribute("item") != Some(item.to_string().as_str())
                || node.style.get("source-tag") != Some("div")
                || node.style.get("display") != Some("flex")
                || node.style.get("direction") != row.attribute("direction")
                || node.style.get("font-family").unwrap_or("ahem") != "ahem"
                || node.style.get("font-size").unwrap_or("10px") != "10px"
                || node.style.get("line-height").unwrap_or("10px") != "10px"
                || node.style.get("writing-mode").unwrap_or("horizontal-tb") != "horizontal-tb"
                || node.style.get("overflow-x").unwrap_or("visible") != expected_overflow
                || node.style.get("overflow-y").unwrap_or("visible") != expected_overflow
                || node.text.as_deref() != Some(label)
                || !node.children.is_empty()
                || node.prepared.is_some()
                || !wrapped::valid_ranges(label, &words)
                || match clip {
                    None => item != 0,
                    Some(clip) => item != 1 || !wrapped::valid_clip(clip),
                }
            {
                return Err(invalid("wrapped source, Range or clip observation changed"));
            }
            occurrences.push(SourceOccurrence {
                path: SourcePath::SubgridLeaf { card, item },
                direction,
                words,
                clip,
                sizes: None,
            });
        }
    }
    if rows.next().is_some() {
        return Err(invalid("extra wrapped Text occurrence"));
    }
    Ok(occurrences)
}

fn bind_grid_lanes_occurrences(
    golden: &Golden,
    proof: roxmltree::Node<'_, '_>,
) -> Result<Vec<SourceOccurrence>, Error> {
    if golden.root.children.len() != grid_lanes::WIDTHS.len() {
        return Err(invalid("unexpected Grid Lanes text group census"));
    }
    let mut rows = proof.children().filter(roxmltree::Node::is_element);
    let mut occurrences = Vec::new();
    for (row, (group, width)) in golden
        .root
        .children
        .iter()
        .zip(grid_lanes::WIDTHS)
        .enumerate()
    {
        if group.style.get("source-tag") != Some("div")
            || group.style.get("display") != Some("grid-lanes")
            || group.style.get("grid-template-rows") != Some("auto auto auto")
            || group.style.get("width") != Some(width)
            || group.text.is_some()
            || group.children.len() != grid_lanes::LABELS.len()
        {
            return Err(invalid("original Grid Lanes sizing group changed"));
        }
        for (item, (node, label)) in group.children.iter().zip(grid_lanes::LABELS).enumerate() {
            let witness = rows
                .next()
                .ok_or_else(|| invalid("missing Grid Lanes text measurement"))?;
            let direction = occurrence_direction(witness.attribute("direction"))?;
            let sizes: grid_lanes::Sizes = serde_json::from_str(
                witness
                    .attribute("sizes")
                    .ok_or_else(|| invalid("missing standalone sizes"))?,
            )
            .map_err(invalid)?;
            if !witness.has_tag_name("occurrence")
                || witness.attributes().len() != 4
                || witness.children().next().is_some()
                || witness.attribute("row") != Some(row.to_string().as_str())
                || witness.attribute("item") != Some(item.to_string().as_str())
                || node.style.get("source-tag") != Some("div")
                || node.style.get("display") != Some("flex")
                || node.style.get("direction") != witness.attribute("direction")
                || node.style.get("font-family") != Some("monospace")
                || node.style.get("font-size") != Some("15px")
                || node.style.get("line-height") != Some("15px")
                || node.style.get("writing-mode").unwrap_or("horizontal-tb") != "horizontal-tb"
                || node.style.get("width") != Some(width)
                || node.text.as_deref() != Some(label)
                || !node.children.is_empty()
                || node.prepared.is_some()
                || !sizes.valid()
            {
                return Err(invalid(
                    "original monospace leaf or standalone sizing changed",
                ));
            }
            occurrences.push(SourceOccurrence {
                path: SourcePath::GridLanesItem { row, item },
                direction,
                words: Vec::new(),
                clip: None,
                sizes: Some(sizes),
            });
        }
    }
    if rows.next().is_some() {
        return Err(invalid("extra Grid Lanes text measurement"));
    }
    Ok(occurrences)
}

pub(super) fn prepare_label(
    system: &mut text::System,
    face: &surgeist_font::FontFace,
    label: &str,
    direction: layout::Direction,
) -> Result<text::PreparedText, Error> {
    let text_style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Collapse,
        wrap: text::Wrap::Word,
        locale: Some("en".to_owned()),
        ..text::Style::default()
    };
    let original = text::Source::new(label);
    let mut source = text::Source::new("");
    let range = source.push(original.text().clone());
    source.span(range, text_style.clone());
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            source,
            base_style: text_style,
            base_direction: match direction {
                layout::Direction::Ltr => text::ParagraphBaseDirection::LeftToRight,
                layout::Direction::Rtl => text::ParagraphBaseDirection::RightToLeft,
            },
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: original,
                local: text::Range::new(0, range.len()),
                aggregate: range,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: Vec::new(),
            controls: Vec::new(),
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            anchor_order: Vec::new(),
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .map_err(invalid)?;
    let wraps = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: text::Wrap::Word,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wraps).map_err(invalid)?;
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .map_err(invalid)?;
    let mut saw_font = false;
    for unit in prepared.unit_views() {
        for instance in unit.font_instances() {
            let instance = instance.map_err(invalid)?;
            if instance.face().admitted_face() != face
                || instance.axis_count() != 0
                || instance.synthesis() != text::FontSynthesis::default()
            {
                return Err(invalid("prepared text selected a different effective face"));
            }
            saw_font = true;
        }
    }
    if !saw_font {
        return Err(invalid("prepared label has no font instances"));
    }
    Ok(prepared)
}

pub(super) fn lower(tree: &mut TestTree, golden: &Golden, proof: &Proof) -> Result<(), Error> {
    if proof.is_wrapped_subgrid()
        || proof.is_grid_lanes_text()
        || golden.root != *proof.source
        || golden.viewport != proof.viewport
    {
        return Err(invalid("parsed source inputs changed after binding"));
    }
    let resource = surgeist_font::FontResource::decode(
        pinned_font::AHEM_STYLESHEET_WOFF2.to_vec(),
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .map_err(invalid)?;
    let face = resource
        .face(0)
        .ok_or_else(|| invalid("missing exact decoded face"))?;
    if resource.face_count() != 1 || proof.occurrences.is_empty() || !tree.paragraphs.is_empty() {
        return Err(invalid("unexpected font or paragraph census"));
    }
    let mut system = text::System::default();
    system.register_font_resource(resource).map_err(invalid)?;
    for occurrence in &proof.occurrences {
        let (path, direction) = (occurrence.path, occurrence.direction);
        let (_, _, node) = path
            .locate(tree)
            .ok_or_else(|| invalid("missing source path"))?;
        let flow = layout::FlowAxes::new(layout::WritingMode::HorizontalTb, direction);
        let style = tree.nodes[node]
            .layout_input
            .as_box()
            .ok_or_else(|| invalid("missing source box"))?;
        if style.display != layout::Display::Flex
            || style.writing_mode != flow.writing_mode()
            || style.direction != flow.direction()
            || tree.nodes[node].font_family != super::FontFamily::Ahem
            || tree.nodes[node].font_size != 10.0
            || tree.nodes[node].line_height != 10.0
            || !tree.nodes[node].children.is_empty()
        {
            return Err(invalid("lowered source does not match its capture"));
        }
        let label = tree.nodes[node]
            .text
            .take()
            .ok_or_else(|| invalid("missing captured label"))?;
        let prepared = prepare_label(&mut system, &face, &label, direction)?;
        let origin = prepared
            .origin_views()
            .next()
            .ok_or_else(|| invalid("missing prepared source occurrence"))?
            .handle();
        let inherited = InheritedTextContext {
            font_family: super::FontFamily::Ahem,
            font_size: 10.0,
            line_height: LineHeightState::Px(10.0),
            grid_lanes_text: false,
            inline_level_text: false,
            containing_flow: Some(flow),
            parent: Some(node),
            range_root: None,
        };
        let wrapper = tree.push_synthetic_text_wrapper(Vec::new(), inherited);
        let text_node = tree.push_synthetic_text(
            &label,
            InheritedTextContext {
                parent: Some(wrapper),
                ..inherited
            },
        )?;
        tree.nodes[text_node].layout_input =
            layout::LayoutInput::inline_text(layout::TextParticipation::new(origin.clone()));
        tree.nodes[text_node].text = None;
        tree.nodes[wrapper].children.push(text_node);
        tree.nodes[node].children.push(wrapper);
        tree.paragraphs.push(layout::TextParagraphBinding {
            inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: wrapper,
            nodes: vec![text_node],
            origins: vec![layout::TextOriginBinding {
                node: text_node,
                origin,
            }],
            atomics: Vec::new(),
            controls: Vec::new(),
            line_strut: None,
        });
        tree.flex_text.push(LoweredOccurrence {
            path,
            source: node,
            wrapper,
            text: text_node,
        });
    }
    Ok(())
}

pub(super) fn verify_word_ranges(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
    proof: &Proof,
) -> Result<(), Error> {
    if !proof.is_intrinsic() {
        return Ok(());
    }
    if proof.occurrences.len() != tree.flex_text.len() {
        return Err(invalid("incomplete placed intrinsic Text census"));
    }
    let root = tree.nodes[0].final_layout.location;
    let tolerance = f64::from(super::ComparisonTolerance::browser_parity().value);
    for (bound, lowered) in proof.occurrences.iter().zip(&tree.flex_text) {
        let path = bound.path;
        if path != lowered.path {
            return Err(invalid(format!("changed prepared origin for {path}")));
        }
        let placement = batch
            .placed_text()
            .iter()
            .find(|placed| placed.container() == lowered.wrapper)
            .ok_or_else(|| invalid(format!("missing placed Text for {path}")))?;
        let layout::LayoutInput::InlineText(participation) = &tree.nodes[lowered.text].layout_input
        else {
            return Err(invalid(format!("missing source origin for {path}")));
        };
        for word in &bound.words {
            let range = text::Range::new(word.start, word.end);
            let selected = placement
                .layout()
                .source_range_fragments(participation.origin(), range)
                .map_err(invalid)?;
            let mut pieces = Vec::new();
            for unit in selected.units() {
                if unit.line_index() != word.line {
                    return Err(invalid(format!(
                        "{path} word {}..{}: browser line {}, Native line {}",
                        word.start,
                        word.end,
                        word.line,
                        unit.line_index(),
                    )));
                }
                for piece in unit.pieces() {
                    let text::SourceRangeGeometry::Line(piece) = piece else {
                        return Err(invalid(format!(
                            "unexpected non-line word geometry for {path}"
                        )));
                    };
                    let rect = piece.rect();
                    let left = rect.origin.x - f64::from(root.x);
                    let top = rect.origin.y - f64::from(root.y);
                    let right = left + rect.size.width;
                    let bottom = top + rect.size.height;
                    if [left, top, right, bottom]
                        .iter()
                        .any(|edge| !edge.is_finite())
                        || left >= right
                        || top >= bottom
                    {
                        return Err(invalid(format!("invalid placed word geometry for {path}")));
                    }
                    pieces.push([left, top, right, bottom]);
                }
            }
            pieces.sort_by(|a, b| a[0].total_cmp(&b[0]));
            let Some((&first, rest)) = pieces.split_first() else {
                return Err(invalid(format!("missing placed word geometry for {path}")));
            };
            let mut native = first;
            for &[left, top, right, bottom] in rest {
                if [left, top, bottom] != [native[2], native[1], native[3]] {
                    return Err(invalid(format!(
                        "discontinuous placed word geometry for {path}"
                    )));
                }
                native[2] = right;
            }
            // Capture and placement share the rounded root-border-local frame.
            // Retain the original browser edges and round each corner as the
            // browser box capture does, before deriving an extent.
            let browser = [word.x, word.y, word.x + word.width, word.y + word.height]
                .map(|edge| (edge + 0.5).floor());
            if native
                .into_iter()
                .zip(browser)
                .any(|(a, b)| (a - b).abs() > tolerance)
            {
                return Err(invalid(format!(
                    "{path} word {}..{}: browser {browser:?}, Native {native:?}",
                    word.start, word.end,
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_nested_flex_items_use_prepared_text_intrinsic_bases() {
        let historical = include_str!("xml/flex/fri07_intrinsic_flex_basis__border_box_ltr.xml");
        let mut golden = Golden::parse(historical).expect("retained source topology");
        let viewport = roxmltree::Document::parse(
            r#"<viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="flex" hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>"#,
        )
        .unwrap();
        golden.viewport.root_context =
            super::super::parse_root_context(viewport.root_element(), Some("4")).unwrap();
        for (property, value) in [
            ("position", "absolute"),
            ("font-family", "ahem"),
            ("font-size", "10px"),
            ("line-height", "10px"),
        ] {
            golden
                .root
                .style
                .attrs
                .insert(property.into(), value.into());
        }
        golden
            .viewport
            .root_context
            .validate_root(&golden.root)
            .unwrap();
        let request = super::super::root_request(&golden.viewport.root_context).unwrap();
        let mut tree =
            TestTree::from_golden(&golden.root, Some(&golden.viewport.root_context)).unwrap();
        let proof = Proof {
            occurrences: (0..2)
                .flat_map(|row| {
                    (0..2).map(move |item| SourceOccurrence {
                        path: SourcePath::RowItem { row, item },
                        direction: layout::Direction::Ltr,
                        words: Vec::new(),
                        clip: None,
                        sizes: None,
                    })
                })
                .collect(),
            source: Box::new(golden.root.clone()),
            viewport: golden.viewport.clone(),
        };
        lower(&mut tree, &golden, &proof)
            .expect("four prepared Text occurrences under two Flex rows");
        let batch = super::super::compute_actual_batch(&tree, &golden.name, request)
            .expect("the original nested Flex tree accepts all four prepared paragraphs");
        tree.apply_completed_batch(&batch);
        let rows = &tree.nodes[0].children;
        assert_eq!(rows.len(), 2);
        for (&row, expected) in rows.iter().zip([
            [(0.0, 75.0, 3), (75.0, 185.0, 1)],
            [(0.0, 36.0, 3), (36.0, 84.0, 3)],
        ]) {
            let items = &tree.nodes[row].children;
            assert_eq!(items.len(), 2);
            for (&item, (x, width, lines)) in items.iter().zip(expected) {
                let occurrence = tree
                    .flex_text
                    .iter()
                    .find(|occurrence| occurrence.source == item)
                    .unwrap();
                assert_eq!(tree.nodes[item].final_layout.location.x, x);
                assert_eq!(tree.nodes[item].final_layout.size.width, width);
                let placement = batch
                    .placed_text()
                    .iter()
                    .find(|placement| placement.container() == occurrence.wrapper)
                    .unwrap();
                assert_eq!(placement.layout().lines().len(), lines);
            }
        }
        super::super::flex_text_projection::validate(&tree, &batch)
            .expect("nested Text placement projects through its original item and row");
    }
}

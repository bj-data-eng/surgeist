//! Prepared Text input for authenticated direct-text Flex captures.

use super::{
    Error, Golden, InheritedTextContext, LineHeightState, TestTree, capture_authority, layout,
    pinned_font,
};
use surgeist_text as text;

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

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Proof {
    rows: Vec<layout::Direction>,
    source: Box<super::Node>,
    viewport: super::Viewport,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LoweredOccurrence {
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
    let report = capture_authority::unique_json(report).map_err(invalid)?;
    let metadata: capture_authority::CaptureMetadata =
        serde_json::from_value(report["metadata"].clone()).map_err(invalid)?;
    if metadata.schema_version != 4
        || !metadata
            .inputs
            .get(capture_authority::ROOT_HELPER_INPUT)
            .is_some_and(|hash| capture_authority::root_helper_profile(hash).is_ok())
        || capture_authority::sha256(STYLESHEET) != pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256
        || capture_authority::sha256(pinned_font::AHEM_STYLESHEET_WOFF2)
            != pinned_font::AHEM_STYLESHEET_WOFF2_SHA256
        || CAPTURE_INPUTS
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
    let doc = roxmltree::Document::parse(xml).map_err(invalid)?;
    let root = doc.root_element();
    let mut proofs = root
        .descendants()
        .filter(|node| node.has_tag_name("flex-content-text"));
    let proof = proofs
        .next()
        .ok_or_else(|| invalid("missing occurrence proof"))?;
    if proofs.next().is_some()
        || proof.parent() != Some(root)
        || root.attribute("measurement-version") != Some("4")
        || root.attribute("observation-frame") != Some("root-border-local")
        || proof.attributes().len() != 2
        || proof.attribute("profile") != Some(PROFILE)
        || proof.attribute("font-resource-sha256")
            != Some(pinned_font::AHEM_STYLESHEET_WOFF2_SHA256)
        || !golden.use_rounding
        || !golden.has_element_scroll_sizes()
        || golden.root.style.get("display") != Some("flex")
        || golden.root.text.is_some()
    {
        return Err(invalid("unsupported occurrence proof or root profile"));
    }
    let rows = proof
        .children()
        .filter(roxmltree::Node::is_element)
        .collect::<Vec<_>>();
    if rows.is_empty() || rows.len() != golden.root.children.len() {
        return Err(invalid("incomplete occurrence census"));
    }
    let rows = rows
        .into_iter()
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
            match direction {
                Some("ltr") => Ok(layout::Direction::Ltr),
                Some("rtl") => Ok(layout::Direction::Rtl),
                _ => Err(invalid("unsupported occurrence direction")),
            }
        })
        .collect::<Result<_, _>>()?;
    Ok(Proof {
        rows,
        source: Box::new(golden.root.clone()),
        viewport: golden.viewport.clone(),
    })
}

pub(super) fn lower(tree: &mut TestTree, golden: &Golden, proof: &Proof) -> Result<(), Error> {
    if golden.root != *proof.source || golden.viewport != proof.viewport {
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
    if resource.face_count() != 1
        || tree.nodes[0].children.len() != proof.rows.len()
        || !tree.paragraphs.is_empty()
    {
        return Err(invalid("unexpected font or paragraph census"));
    }
    let mut system = text::System::default();
    system.register_font_resource(resource).map_err(invalid)?;
    for (node, direction) in tree.nodes[0]
        .children
        .clone()
        .into_iter()
        .zip(proof.rows.iter().copied())
    {
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
        let text_style = text::Style {
            font: text::Font::new().family("Ahem"),
            size: 10.0,
            line_height: text::LineHeight::Absolute(10.0),
            white_space: text::WhiteSpace::Collapse,
            wrap: text::Wrap::Word,
            locale: Some("en".to_owned()),
            ..text::Style::default()
        };
        let original = text::Source::new(label.as_str());
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
                if instance.face().admitted_face() != &face
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
            source: node,
            wrapper,
            text: text_node,
        });
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
            rows: vec![layout::Direction::Ltr; 4],
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

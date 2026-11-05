//! Admission of the decoded image from the fixed sideways Flex capture.

use super::{Error, Golden, Node, TestTree, Viewport, capture_authority, layout, one_child};
use capture_authority::{CaptureMetadata, CapturedArtifact, RootCaptureProfile, sha256};

const SOURCE: &str = "html/flex/fri07_flex_composition.html";
const SOURCE_BYTES: &[u8] = include_bytes!("html/flex/fri07_flex_composition.html");
const RETAINED_REPORT: &str =
    include_str!("captured/flex-composition-differences/xml/generation-reports/all.json");
const REQUEST: &[u8] =
    include_bytes!("captured/flex-composition-differences/observation-request.json");

pub(super) struct Capture<'a> {
    pub report: &'a str,
    pub source_path: &'a str,
    pub source_bytes: &'a [u8],
    pub output_path: &'a str,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Proof {
    root: Box<Node>,
    viewport: Viewport,
    natural_size: layout::Size,
}

fn invalid(detail: impl std::fmt::Display) -> Error {
    Error::new(format!("Flex composition image: {detail}"))
}

pub(super) fn bind(
    capture: Capture<'_>,
    test: roxmltree::Node<'_, '_>,
    golden: &Golden,
    xml: &str,
) -> Result<Proof, Error> {
    let source_digest = sha256(SOURCE_BYTES);
    if capture.source_path != SOURCE || capture.source_bytes != SOURCE_BYTES {
        return Err(invalid("original source changed"));
    }
    let report = capture_authority::unique_json(capture.report).map_err(invalid)?;
    let current = &report["metadata"];
    let metadata: CaptureMetadata = serde_json::from_value(current.clone()).map_err(invalid)?;
    let retained = capture_authority::unique_json(RETAINED_REPORT).map_err(invalid)?;
    let sealed = &retained["metadata"];
    // The qualifier executable may be rebuilt. The browser and capture
    // contract must still match the environment of the gutter control.
    if [
        "generator",
        "engine_version",
        "browser_executable",
        "browser_executable_sha256",
        "browser_provenance",
        "contract_sha256",
        "source_imports",
    ]
    .into_iter()
    .any(|key| current[key] != sealed[key])
        || current["inputs"]["browser-owner.toml"] != sealed["inputs"]["browser-owner.toml"]
    {
        return Err(invalid("capture environment changed"));
    }
    if metadata.schema_version != 4
        || metadata.inputs.get(SOURCE) != Some(&source_digest)
        || metadata.inputs.get("observation-request.json") != Some(&sha256(REQUEST))
        || metadata
            .inputs
            .get("scripts/gentest/test_base_style.css")
            .map(String::as_str)
            != Some(super::pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256)
        || !metadata
            .inputs
            .get(capture_authority::ROOT_HELPER_INPUT)
            .is_some_and(|digest| {
                capture_authority::root_helper_profile(digest).ok()
                    == Some(RootCaptureProfile::RootContextFlexCompositionV1)
            })
    {
        return Err(invalid("source or capture profile changed"));
    }
    let artifacts = report["generated"]
        .as_array()
        .ok_or_else(|| invalid("missing generated artifacts"))?;
    let mut matches = artifacts.iter().filter(|artifact| {
        artifact["name"] == golden.name || artifact["output"] == capture.output_path
    });
    let artifact: CapturedArtifact = serde_json::from_value(
        matches
            .next()
            .ok_or_else(|| invalid("missing captured output"))?
            .clone(),
    )
    .map_err(invalid)?;
    if matches.next().is_some()
        || artifact.name != golden.name
        || artifact.source != SOURCE
        || artifact.output != capture.output_path
        || artifact.output != format!("xml/flex/{}.xml", golden.name)
        || artifact.name != format!("fri07_flex_composition__{}", artifact.variant)
        || artifact.source_sha256 != source_digest
        || artifact.output_sha256 != sha256(xml.as_bytes())
        || !artifact.linked_resources.is_empty()
    {
        return Err(invalid("capture source or output binding changed"));
    }

    // The retained capture pins the complete normalized input independently of
    // the supplied report. Browser expectations remain separate from admission.
    let projection_digest = match artifact.variant.as_str() {
        "border_box_ltr" => "f873c4fc20217022deb5cc35c9e5443010e83e3490f7493a3ac1a6e4d4979fa4",
        "border_box_rtl" => "3ea6eeecce0b70bbf559e7846480b39aaddc7d44c636b16ab8f3ebdc0a11fc46",
        "content_box_ltr" => "dee73b0201e80c039add5f9ddfa1fa2c6359d6d27d296b64fb2c28025352b63d",
        "content_box_rtl" => "017839898a38e0f3dc708643167392995020218ae86d1903ffd027bc325d5f36",
        _ => return Err(invalid("unregistered variant")),
    };
    let viewport = one_child(test, "viewport")?;
    let input = one_child(test, "input")?;
    if !xml
        .get(viewport.range().start..input.range().end)
        .is_some_and(|projection| sha256(projection.as_bytes()) == projection_digest)
    {
        return Err(invalid("normalized source projection changed"));
    }

    let mut witnesses = test
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "flex-composition-image");
    let witness = witnesses
        .next()
        .ok_or_else(|| invalid("missing natural dimensions"))?;
    if witnesses.next().is_some()
        || witness.parent() != Some(test)
        || witness.tag_name().namespace().is_some()
        || witness.attributes().len() != 2
        || witness.attribute("natural-width") != Some("80")
        || witness.attribute("natural-height") != Some("40")
        || witness.children().next().is_some()
        || test.attribute("measurement-version") != Some("4")
        || test.attribute("observation-frame") != Some("root-border-local")
        || !golden.use_rounding
    {
        return Err(invalid("incompatible root or natural dimensions witness"));
    }
    Ok(Proof {
        root: Box::new(golden.root.clone()),
        viewport: golden.viewport.clone(),
        natural_size: layout::Size::new(80.0, 40.0),
    })
}

pub(super) fn lower(tree: &mut TestTree, golden: &Golden, proof: &Proof) -> Result<(), Error> {
    if golden.root != *proof.root || golden.viewport != proof.viewport {
        return Err(invalid("authenticated source projection changed"));
    }
    let node = tree.nodes[0].children[1];
    let leaf = &mut tree.nodes[node];
    if !leaf.children.is_empty() || leaf.text.is_some() {
        return Err(invalid("normalized image is not a leaf"));
    }
    let mut input = leaf
        .layout_input
        .as_box()
        .ok_or_else(|| invalid("missing image box"))?
        .clone();
    let ratio = proof.natural_size.width / proof.natural_size.height;
    input.item_is_replaced = true;
    input.aspect_ratio = layout::AspectRatio::new(ratio);
    leaf.layout_input = layout::LayoutInput::box_input(input);
    tree.replaced_image = Some((node, proof.natural_size));
    Ok(())
}

pub(super) fn measure(natural: layout::Size, input: layout::LeafMeasureInput) -> layout::Size {
    let known = input.known_content_size();
    let ratio = natural.width / natural.height;
    layout::Size::new(
        known
            .width
            .or_else(|| known.height.map(|height| height * ratio))
            .unwrap_or(natural.width),
        known
            .height
            .or_else(|| known.width.map(|width| width / ratio))
            .unwrap_or(natural.height),
    )
}

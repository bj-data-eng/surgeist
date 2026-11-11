//! Admission of the decoded image from the fixed sideways Flex capture.

use super::{Error, Golden, Node, TestTree, Viewport, capture_authority, layout, one_child};
use capture_authority::{CaptureMetadata, CapturedArtifact, sha256};

pub(super) const SOURCE: &str = "html/flex/fri07_flex_composition.html";
const SOURCE_BYTES: &[u8] = include_bytes!("html/flex/fri07_flex_composition.html");
const RETAINED_REPORT: &str =
    include_str!("captured/flex-composition-differences/xml/generation-reports/all.json");
const SCRIPT_CUSTODY_REPORT: &str = include_str!("flex_composition_script_custody_report.json");
const FOCUSED_REPORT: &str = include_str!("flex_composition_focused_report.json");
const FIXED_ORTHOGONAL_REPORT: &str = include_str!("flex_composition_fixed_orthogonal_report.json");
const COMPUTED_MARGIN_REPORT: &str = include_str!("flex_composition_computed_margin_report.json");
const COMPUTED_MARGIN_REPORT_SHA256: &str =
    "b7b9dfd49cacdecb54aae3c032365dcec5e6ac9c4939a1228ee45fef08307066";
const FIXED_ORTHOGONAL_REPORT_SHA256: &str =
    "f69ebad7fd090c3b8c2ba1a4978d361baf21f75ab946efb6b300c212d6f4e862";
const FOCUSED_REPORT_SHA256: &str =
    "60437234e3d3d380bda1cb208409d23ebfa690a3add9d1db9b21c8f94ebc5b82";

// Each report binds an environment qualified by the same collapse-disabled
// gutter control. The original producer and its witness remain immutable.
const RETAINED_REPORT_SHA256: &str =
    "65bb238d03aae80b366fa1f481be42137799466f74c1a35fafdb40d2460478a2";
const SCRIPT_CUSTODY_REPORT_SHA256: &str =
    "cd167f9ecd34d142423885080df4f839ccf94b81c5ba80eb70ea868770547024";
pub(crate) const RETAINED_HELPER_SHA256: &str =
    "afad9adf5a3915d88e94b0e3ad2d4768c242783422a0fb3bbc9a277492c10f9c";

pub(crate) fn accepts_helper(digest: &str) -> bool {
    digest == RETAINED_HELPER_SHA256
        || digest == crate::capture_helper_identity::SCRIPT_CUSTODY_HELPER_SHA256
        || digest == crate::capture_helper_identity::FIXED_ORTHOGONAL_HELPER_SHA256
        || digest == crate::capture_helper_identity::COMPUTED_MARGIN_HELPER_SHA256
}

pub(super) struct Capture<'a> {
    pub report: &'a str,
    pub published: bool,
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
    let projection =
        require_control_environment(current, &metadata, RETAINED_REPORT, capture.published)?;
    if metadata.schema_version != 4
        || metadata.inputs.get(SOURCE) != Some(&source_digest)
        || metadata.inputs.get("observation-request.json")
            != Some(&sha256(capture_authority::FLEX_COMPOSITION_REQUEST_BYTES))
        || metadata
            .inputs
            .get("scripts/gentest/test_base_style.css")
            .map(String::as_str)
            != Some(super::pinned_font::ROOT_CONTEXT_STYLESHEET_SHA256)
        || !metadata
            .inputs
            .get(capture_authority::ROOT_HELPER_INPUT)
            .is_some_and(|digest| accepts_helper(digest))
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

    // Each qualified environment pins the complete normalized input.
    // Browser expectations are compared after admission.
    let projection_digest = projection
        .digest(&artifact.variant)
        .ok_or_else(|| invalid("unregistered variant"))?;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceProjection {
    Retained,
    InheritedTypography,
}

impl SourceProjection {
    fn digest(self, variant: &str) -> Option<&'static str> {
        Some(match (self, variant) {
            (Self::Retained, "border_box_ltr") => {
                "f873c4fc20217022deb5cc35c9e5443010e83e3490f7493a3ac1a6e4d4979fa4"
            }
            (Self::Retained, "border_box_rtl") => {
                "3ea6eeecce0b70bbf559e7846480b39aaddc7d44c636b16ab8f3ebdc0a11fc46"
            }
            (Self::Retained, "content_box_ltr") => {
                "dee73b0201e80c039add5f9ddfa1fa2c6359d6d27d296b64fb2c28025352b63d"
            }
            (Self::Retained, "content_box_rtl") => {
                "017839898a38e0f3dc708643167392995020218ae86d1903ffd027bc325d5f36"
            }
            (Self::InheritedTypography, "border_box_ltr") => {
                "fa5fa96b819ec7c75af5ca1b15dffe67d117cf411119372e7b6fc59dd02e3a8d"
            }
            (Self::InheritedTypography, "border_box_rtl") => {
                "deab14baed8c2eb7d01622f35d9c0937f0242581a5fa415315e77dc6f3f994b1"
            }
            (Self::InheritedTypography, "content_box_ltr") => {
                "f0bbfe27f75f8c2714abfc461e92498103ca894abe698c7679c066d043425909"
            }
            (Self::InheritedTypography, "content_box_rtl") => {
                "dedcdce4d246d7dd66b468c03b11a9ab281663fe057c6527a0ee097f285416cb"
            }
            _ => return None,
        })
    }
}

fn require_control_environment(
    current: &serde_json::Value,
    metadata: &CaptureMetadata,
    control_report: &str,
    published: bool,
) -> Result<SourceProjection, Error> {
    let mut admitted = None;
    for (report, digest, declaration, projection) in [
        (
            control_report,
            RETAINED_REPORT_SHA256,
            "corpus.toml",
            SourceProjection::Retained,
        ),
        (
            SCRIPT_CUSTODY_REPORT,
            SCRIPT_CUSTODY_REPORT_SHA256,
            "corpus.toml",
            SourceProjection::Retained,
        ),
        (
            FOCUSED_REPORT,
            FOCUSED_REPORT_SHA256,
            "focused-declaration.json",
            SourceProjection::Retained,
        ),
        (
            FIXED_ORTHOGONAL_REPORT,
            FIXED_ORTHOGONAL_REPORT_SHA256,
            "focused-declaration.json",
            SourceProjection::Retained,
        ),
        (
            COMPUTED_MARGIN_REPORT,
            COMPUTED_MARGIN_REPORT_SHA256,
            "focused-declaration.json",
            SourceProjection::InheritedTypography,
        ),
    ] {
        if sha256(report.as_bytes()) != digest {
            return Err(invalid("gutter control report identity changed"));
        }
        let witness = capture_authority::unique_json(report).map_err(invalid)?;
        let sealed = &witness["metadata"];
        if environment_matches(current, metadata, sealed, declaration, published)
            && admitted
                .replace(projection)
                .is_some_and(|previous| previous != projection)
        {
            return Err(invalid("capture environment has conflicting projections"));
        }
    }
    admitted.ok_or_else(|| invalid("capture environment changed"))
}

fn environment_matches(
    current: &serde_json::Value,
    metadata: &CaptureMetadata,
    sealed: &serde_json::Value,
    declaration: &str,
    published: bool,
) -> bool {
    let same_environment = [
        "engine_version",
        "browser_executable",
        "browser_executable_sha256",
        "browser_provenance",
    ]
    .into_iter()
    .all(|key| current[key] == sealed[key])
        && [
            "browser-owner.toml",
            "fonts/Ahem.ttf",
            "observation-request.json",
            "scripts/gentest/test_base_style.css",
            capture_authority::ROOT_HELPER_INPUT,
        ]
        .into_iter()
        .all(|key| current["inputs"][key] == sealed["inputs"][key]);
    if published {
        // Publication owns and verifies its distinct declaration and contract.
        return same_environment
            && metadata.generator == capture_authority::FIXED_GENERATOR
            && metadata.source_imports.is_empty();
    }
    let digest = |value: &serde_json::Value| {
        value
            .get("inputs")?
            .get(declaration)?
            .as_str()
            .filter(|value| {
                value.len() == 64
                    && value
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
            .map(str::to_owned)
    };
    let declaration_matches = match (digest(current), digest(sealed)) {
        (Some(current), Some(sealed)) => current == sealed,
        _ => false,
    };
    let other = if declaration == "corpus.toml" {
        "focused-declaration.json"
    } else {
        "corpus.toml"
    };
    same_environment
        && declaration_matches
        && current["inputs"] == sealed["inputs"]
        && current["inputs"].get(other).is_none()
        && sealed["inputs"].get(other).is_none()
        && ["generator", "contract_sha256", "source_imports"]
            .into_iter()
            .all(|key| current[key] == sealed[key])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_capture_refuses_a_replaced_control_report_with_matching_metadata() {
        let report =
            include_str!("captured/flex-composition-qualification/xml/generation-reports/all.json");
        let mut current = capture_authority::unique_json(report).unwrap()["metadata"].clone();
        let metadata = serde_json::from_value(current.clone()).unwrap();
        require_control_environment(&current, &metadata, RETAINED_REPORT, true).unwrap();

        let mut control = capture_authority::unique_json(RETAINED_REPORT).unwrap();
        let substituted = serde_json::json!("0".repeat(64));
        current["browser_executable_sha256"] = substituted.clone();
        control["metadata"]["browser_executable_sha256"] = substituted;
        let metadata = serde_json::from_value(current.clone()).unwrap();
        let error = require_control_environment(&current, &metadata, &control.to_string(), true)
            .expect_err("a replacement report cannot redefine the control environment");
        assert!(
            error
                .to_string()
                .contains("gutter control report identity changed")
        );
    }
    #[test]
    fn published_environment_can_use_a_qualified_focused_witness_without_its_declaration_format() {
        let witness = capture_authority::unique_json(FOCUSED_REPORT).unwrap();
        // This tests environment suitability only; the publication owner checks
        // the report's independent contract before reaching this boundary.
        let mut current = capture_authority::unique_json(include_str!(
            "captured/flex-composition-qualification/xml/generation-reports/all.json"
        ))
        .unwrap()["metadata"]
            .clone();
        current["inputs"][capture_authority::ROOT_HELPER_INPUT] =
            witness["metadata"]["inputs"][capture_authority::ROOT_HELPER_INPUT].clone();
        let metadata = serde_json::from_value(current.clone()).unwrap();
        assert!(environment_matches(
            &current,
            &metadata,
            &witness["metadata"],
            "focused-declaration.json",
            true
        ));
        assert!(!environment_matches(
            &current,
            &metadata,
            &witness["metadata"],
            "focused-declaration.json",
            false
        ));
        current["generator"] = serde_json::json!("unowned-generator");
        let metadata = serde_json::from_value(current.clone()).unwrap();
        assert!(!environment_matches(
            &current,
            &metadata,
            &witness["metadata"],
            "focused-declaration.json",
            true
        ));
    }
}

use super::*;

fn reference() -> Value {
    json!({"profile":1,"display":"grid","writingMode":"vertical-rl","direction":"ltr","textOrientation":"mixed","dominantBaseline":"auto","alignmentBaseline":"auto","font":{"sha256":"e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c","family":"ahem","size":10,"style":"normal","weight":400,"stretch":"normal","loaded":true},"reference":"central"})
}
fn node() -> Value {
    json!({"layoutInput":"box","tagName":"div","layoutReadyInlineRoot":true,"gridAlignmentProfile":{"profile":1,"owners":["/"]},"gridAlignmentReference":reference(),"useRounding":false,"viewport":{"rootContext":{"kind":"block-flow-auto","parentInlineExtent":800,"parentDirection":"ltr","fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}},"style":{"display":"grid","position":"relative","writingMode":"vertical-rl","direction":"ltr","fontFamily":"ahem","fontSize":{"unit":"px","value":10},"lineHeight":{"unit":"px","value":10},"size":{"width":{"unit":"px","value":40},"height":{"unit":"px","value":10}}},"children":[],"worldRoundedLayout":{"x":0,"y":0,"width":40,"height":10},"unroundedLayout":{"x":0,"y":0,"width":40,"height":10}})
}
fn without_attribute(xml: &str, name: &str) -> String {
    let prefix = format!(" {name}=\"");
    let start = xml.find(&prefix).unwrap();
    let end = start + prefix.len() + xml[start + prefix.len()..].find('"').unwrap() + 1;
    format!("{}{}", &xml[..start], &xml[end..])
}
#[test]
fn own_grid_mixed_upright_roundtrip_selects_native_reference_without_zero_strut() {
    for mode in ["vertical-rl", "vertical-lr"] {
        for orientation in ["mixed", "upright"] {
            for direction in ["ltr", "rtl"] {
                let mut value = node();
                value["style"]["writingMode"] = json!(mode);
                value["style"]["direction"] = json!(direction);
                value["gridAlignmentReference"]["writingMode"] = json!(mode);
                value["gridAlignmentReference"]["direction"] = json!(direction);
                value["gridAlignmentReference"]["textOrientation"] = json!(orientation);
                let xml = generate_xml("own-grid", &value);
                assert!(!xml.contains("atomic-paragraph-metrics"));
                let golden = browser_parity_support::Golden::parse(&xml).unwrap();
                browser_parity_support::assert_grid_alignment_reference_inputs(
                    &golden,
                    &["/".to_owned()],
                )
                .unwrap();
                browser_parity_support::assert_surgeist_matches(&golden).unwrap();
            }
        }
    }
}
#[test]
fn required_profile_and_each_actual_owner_record_never_silently_disappear() {
    let value = node();
    for field in ["gridAlignmentProfile", "gridAlignmentReference"] {
        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(measured_xml("missing-grid", &missing).is_err(), "{field}");
    }
    for owners in [json!([]), json!(["/", "/"]), json!(["/0"])] {
        let mut invalid = value.clone();
        invalid["gridAlignmentProfile"]["owners"] = owners;
        assert!(measured_xml("bad-owner-census", &invalid).is_err());
    }
    let xml = generate_xml("grid-profile", &value);
    let no_record = without_attribute(&xml, "grid-alignment-reference");
    assert!(browser_parity_support::Golden::parse(&no_record).is_err());
    let no_profile = without_attribute(&xml, "grid-alignment-profile");
    assert!(browser_parity_support::Golden::parse(&no_profile).is_err());
    let raw = without_attribute(&no_profile, "grid-alignment-reference");
    let raw = browser_parity_support::Golden::parse(&raw).unwrap();
    // Explicit raw Native observation remains Physical. The captured front
    // door independently refuses loss of its mandatory profile authority.
    browser_parity_support::assert_grid_alignment_reference_inputs(&raw, &[]).unwrap();
    assert!(
        browser_parity_support::assert_surgeist_matches(&raw)
            .unwrap_err()
            .to_string()
            .contains("MissingGridAlignmentProfileAuthority")
    );
}
#[test]
fn independent_grid_role_font_frame_override_and_scope_authorities_refuse() {
    let value = node();
    for field in [
        "profile",
        "font",
        "display",
        "writingMode",
        "direction",
        "textOrientation",
        "dominantBaseline",
        "alignmentBaseline",
        "reference",
    ] {
        let mut invalid = value.clone();
        invalid["gridAlignmentReference"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            measured_xml("missing-own-context", &invalid).is_err(),
            "{field}"
        );
    }
    for (field, bad) in [
        ("display", json!("block")),
        ("writingMode", json!("horizontal-tb")),
        ("direction", json!("rtl")),
        ("fontFamily", json!("monospace")),
        ("fontSize", json!({"unit":"px","value":11})),
    ] {
        let mut invalid = value.clone();
        invalid["style"][field] = bad;
        assert!(
            measured_xml("foreign-own-frame", &invalid).is_err(),
            "{field}"
        );
    }
    for (field, bad) in [
        ("reference", json!("alphabetic")),
        ("textOrientation", json!("sideways")),
        ("dominantBaseline", json!("central")),
        ("alignmentBaseline", json!("alphabetic")),
    ] {
        let mut invalid = value.clone();
        invalid["gridAlignmentReference"][field] = bad;
        assert!(
            measured_xml("unsupported-own-context", &invalid).is_err(),
            "{field}"
        );
    }
    let mut invalid = value.clone();
    invalid["gridAlignmentReference"]["font"]["sha256"] = json!("foreign");
    assert!(measured_xml("foreign-resource", &invalid).is_err());
    let mut nested = value.clone();
    nested.as_object_mut().unwrap().remove("viewport");
    nested.as_object_mut().unwrap().remove("useRounding");
    let mut invalid = value;
    invalid["children"] = json!([nested]);
    invalid["gridAlignmentProfile"]["owners"] = json!(["/", "/0"]);
    assert!(measured_xml("foreign-profile-scope", &invalid).is_err());
}
#[test]
fn grid_only_preparation_uses_actual_own_font_without_paragraph_donation() {
    let script=[r#"
const window={};const Node={ELEMENT_NODE:1,TEXT_NODE:3,COMMENT_NODE:8};const CSSRule={STYLE_RULE:1};
const own={display:'grid',writingMode:'vertical-rl',direction:'ltr',textOrientation:'mixed',dominantBaseline:'auto',alignmentBaseline:'auto',textCombineUpright:'none',fontFamily:'ahem',fontSize:'10px',fontStyle:'normal',fontWeight:'400',fontStretch:'100%',fontFeatureSettings:'normal',fontVariationSettings:'normal',fontVariant:'normal',fontSizeAdjust:'none',lineHeight:'10px'};
let override='';
const root={id:'test-root',nodeType:1,tagName:'DIV',namespaceURI:'http://www.w3.org/1999/xhtml',childNodes:[],parentElement:null,style:{getPropertyValue(){return override;}},getAttribute(name){return name==='data-surgeist-layout-ready-inline'?'true':null;},querySelectorAll(){return [];}};
function getComputedStyle(){return own;}
const face={family:'ahem',style:'normal',weight:'normal',stretch:'normal',unicodeRange:'U+0-10FFFF',featureSettings:'normal',variationSettings:'normal',ascentOverride:'normal',descentOverride:'normal',lineGapOverride:'normal',sizeAdjust:'100%',status:'unloaded',loads:0,async load(){this.loads++;this.status='loaded';return this;}};
const faces=[face];faces.ready=Promise.resolve();const document={fonts:faces,styleSheets:[],getElementById(){return root;}};
"#,TEST_HELPER_SOURCE,r#"
(async()=>{
 rootContextCapture={ownedStyle:{}};await prepareNamedReferenceFont();
 if(face.loads!==1||namedReferenceFontState.face!==face)throw new Error('actual Grid-only load authority missing');
 for(const orientation of ['mixed','upright']){own.textOrientation=orientation;const capture=gridAlignmentReferenceForElement(root);if(capture.reference!=='central'||capture.textOrientation!==orientation||capture.before!==undefined||capture.lineHeight!==undefined||capture.order!==undefined)throw new Error('Grid reference borrowed paragraph metrics');}
 await prepareNamedReferenceFont();if(face.loads!==1)throw new Error('duplicate font loader/reload');
 function fails(reason){let got;try{gridAlignmentReferenceForElement(root);}catch(error){got=error.message;}if(got!=='Grid alignment reference: '+reason)throw new Error('missing exact refusal '+reason+': '+got);}
 own.textOrientation='sideways';fails('UnsupportedReferenceContext');own.textOrientation='mixed';
 own.fontFamily='monospace';fails('UnsupportedFirstFontAuthority');own.fontFamily='ahem';
 override='alphabetic';fails('UnsupportedReferenceOverride');override='';
 own.writingMode='horizontal-tb';root.childNodes=[{atomicParagraphMetrics:{reference:'central'}}];if(gridAlignmentReferenceForElement(root)!==undefined)throw new Error('vertical child donated Grid reference');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#].concat();
    run_bundled_helper_script("own-grid-reference", script);
}

#[test]
fn global_profile_on_serialized_text_box_preserves_actual_box_contract() {
    let mut value = node();
    value
        .as_object_mut()
        .unwrap()
        .remove("gridAlignmentReference");
    value["gridAlignmentProfile"]["owners"] = json!([]);
    value["style"]["display"] = json!("block");
    value["style"]["writingMode"] = json!("horizontal-tb");
    value["textContent"] = json!("x");
    let xml = generate_xml("text-box-global-profile", &value);
    let document = roxmltree::Document::parse(&xml).unwrap();
    let input = document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("input"))
        .unwrap();
    let root = input.children().find(roxmltree::Node::is_element).unwrap();
    assert!(root.has_tag_name("text"));
    assert_eq!(root.attribute("source-tag"), Some("div"));
    assert_eq!(root.attribute("display"), Some("block"));
    assert!(root.attribute("grid-alignment-profile").is_some());
    assert_eq!(root.attribute("grid-alignment-reference"), None);
    let golden = browser_parity_support::Golden::parse(&xml).expect(
        "global source census belongs to the serialized HTML text box, not only Grid records",
    );
    assert_eq!(golden.root.kind, browser_parity_support::NodeKind::Text);
    assert_eq!(golden.root.text.as_deref(), Some("x"));
    // This constructs the existing ordinary Box/analytic-leaf input. It does
    // not create prepared or measured-width inline Text participation. Those
    // historical source-admission refusals retain their own existing tests.
    browser_parity_support::assert_grid_alignment_reference_inputs(&golden, &[]).unwrap();
}

#[test]
fn grid_record_on_serialized_text_box_remains_forbidden() {
    let xml = generate_xml("grid-record-role", &node());
    // Preserve a fully valid Grid record, actual own style/font and complete
    // census; change only its private input role from box to text.
    let foreign_role = xml.replacen("<div ", "<text ", 1);
    assert_ne!(foreign_role, xml);
    assert!(browser_parity_support::Golden::parse(&foreign_role).is_err());
}

fn coherent_grid_erasure(xml: &str) -> String {
    let document = roxmltree::Document::parse(xml).unwrap();
    let root = document
        .descendants()
        .find(|node| node.attribute("grid-alignment-profile").is_some())
        .unwrap();
    assert_eq!(root.attribute("layout-ready-inline-root"), Some("true"));
    let raw = root.attribute("grid-alignment-profile").unwrap();
    let mut profile: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(profile["owners"], json!(["/"]));
    profile["owners"] = json!([]);
    let encode = |raw: &str| raw.replace('&', "&amp;").replace('"', "&quot;");
    let old = format!(" grid-alignment-profile=\"{}\"", encode(raw));
    let replacement = format!(
        " grid-alignment-profile=\"{}\"",
        encode(&profile.to_string())
    );
    let missing_record = without_attribute(xml, "grid-alignment-reference");
    let erased = missing_record.replacen(&old, &replacement, 1);
    assert_ne!(erased, missing_record);
    erased
}

#[test]
fn coherent_grid_record_erasure_refuses_at_wire_demand_boundary() {
    let mut value = node();
    measured_xml("marked-grid-control", &value).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("gridAlignmentReference");
    value["gridAlignmentProfile"]["owners"] = json!([]);
    assert!(measured_xml("coherent-grid-erasure", &value).is_err());
}

#[test]
fn coherent_grid_record_erasure_refuses_at_reader_demand_boundary() {
    let xml = generate_xml("marked-grid-control", &node());
    let golden = browser_parity_support::Golden::parse(&xml).unwrap();
    browser_parity_support::assert_grid_alignment_reference_inputs(&golden, &["/".to_owned()])
        .unwrap();
    assert!(browser_parity_support::Golden::parse(&coherent_grid_erasure(&xml)).is_err());
}

#[test]
fn grid_record_outside_actual_region_refuses_at_wire_boundary() {
    let mut value = node();
    measured_xml("marked-grid-control", &value).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .remove("layoutReadyInlineRoot");
    assert!(measured_xml("outside-grid-region", &value).is_err());
}

#[test]
fn grid_record_outside_actual_region_refuses_at_reader_boundary() {
    let xml = generate_xml("marked-grid-control", &node());
    let golden = browser_parity_support::Golden::parse(&xml).unwrap();
    browser_parity_support::assert_grid_alignment_reference_inputs(&golden, &["/".to_owned()])
        .unwrap();
    let outside = without_attribute(&xml, "layout-ready-inline-root");
    assert!(browser_parity_support::Golden::parse(&outside).is_err());
}

fn grid_child() -> Value {
    let mut child = node();
    for field in [
        "viewport",
        "useRounding",
        "gridAlignmentProfile",
        "layoutReadyInlineRoot",
    ] {
        child.as_object_mut().unwrap().remove(field);
    }
    child
}

#[test]
fn marked_ancestor_demands_child_grid_own_context_without_child_marker() {
    let mut root = node();
    root.as_object_mut()
        .unwrap()
        .remove("gridAlignmentReference");
    root["style"]["display"] = json!("block");
    root["style"]["writingMode"] = json!("horizontal-tb");
    root["gridAlignmentProfile"]["owners"] = json!(["/0"]);
    root["children"] = json!([grid_child()]);
    let xml = generate_xml("inherited-grid-region", &root);
    let golden = browser_parity_support::Golden::parse(&xml).unwrap();
    browser_parity_support::assert_grid_alignment_reference_inputs(&golden, &["/0".to_owned()])
        .unwrap();
}

#[test]
fn own_horizontal_reset_does_not_borrow_vertical_ancestor_mode() {
    let mut root = node();
    let mut child = grid_child();
    child
        .as_object_mut()
        .unwrap()
        .remove("gridAlignmentReference");
    // Wire Style's existing absent-mode value is own horizontal-tb. Under
    // the vertical parent, its serializer must preserve that reset explicitly.
    child["style"]
        .as_object_mut()
        .unwrap()
        .remove("writingMode");
    root["children"] = json!([child]);
    let xml = generate_xml("own-grid-mode-reset", &root);
    let golden = browser_parity_support::Golden::parse(&xml).unwrap();
    assert_eq!(
        golden.root.children[0].style.get("writing-mode"),
        Some("horizontal-tb")
    );
    browser_parity_support::assert_grid_alignment_reference_inputs(&golden, &["/".to_owned()])
        .unwrap();
}

#[test]
fn serialized_grid_text_role_cannot_erase_source_reference_demand() {
    let xml = generate_xml("grid-text-coherent-erasure", &node());
    let erased = coherent_grid_erasure(&xml).replacen("<div ", "<text ", 1);
    let document =
        roxmltree::Document::parse(&erased).expect("role substitution remains well-formed XML");
    let input = document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("input"))
        .unwrap();
    let root = input.children().find(roxmltree::Node::is_element).unwrap();
    assert!(root.has_tag_name("text"));
    assert_eq!(root.attribute("display"), Some("grid"));
    assert_eq!(root.attribute("layout-ready-inline-root"), Some("true"));
    assert_eq!(root.attribute("grid-alignment-reference"), None);
    let error = browser_parity_support::Golden::parse(&erased).unwrap_err();
    assert!(
        error.to_string().contains(
            "Grid record coverage disagrees with independently derived source demand at /"
        ),
        "{error}"
    );
}

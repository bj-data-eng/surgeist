use super::*;
fn certificate() -> Value {
    json!({"profile":1,"font":{"sha256":"e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c","family":"ahem","size":10,"style":"normal","weight":400,"stretch":"normal","loaded":true},"writingMode":"vertical-rl","direction":"rtl","textOrientation":"mixed","dominantBaseline":"auto","alignmentBaseline":"auto","verticalAlign":"baseline","lineFit":"leading","reference":"central","before":0,"after":0,"lineHeight":0,"order":[{"kind":"atomic","child":0,"source":1,"bidiLevel":1},{"kind":"break","child":1,"source":2,"reference":"central","before":0,"after":0},{"kind":"atomic","child":2,"source":3,"bidiLevel":1}]})
}
fn node() -> Value {
    let style = json!({"fontFamily":"ahem","fontSize":{"unit":"px","value":10},"lineHeight":{"unit":"px","value":0},"writingMode":"vertical-rl","direction":"rtl","verticalAlign":"baseline"});
    let child = |tag: &str, display: &str| {
        let mut own = style.clone();
        own["display"] = json!(display);
        json!({"layoutInput":"box","tagName":tag,"style":own,"children":[],"worldRoundedLayout":{"x":0,"y":0,"width":20,"height":20},"unroundedLayout":{"x":0,"y":0,"width":20,"height":20}})
    };
    let mut first = child("span", "inline-block");
    first["atomicInlineParticipation"] = json!({"bidiLevel":1,"followingBreak":"prohibited"});
    let mut br = child("br", "inline");
    br["lineControlParticipation"] = json!({"kind":"forced-break"});
    br["style"]["inlineMetrics"] = json!({"baseline":0,"lineHeight":0});
    let mut parent_style = style;
    parent_style["display"] = json!("block");
    parent_style["position"] = json!("relative");
    json!({"layoutInput":"box","tagName":"div","style":parent_style,"atomicParagraphMetrics":certificate(),"gridAlignmentProfile":{"profile":1,"owners":[]},"useRounding":false,"viewport":{"rootContext":{"kind":"block-flow-auto","parentInlineExtent":800,"parentDirection":"ltr","fallback":{"initialContainingBlock":{"width":800,"height":600},"nearestScrollport":"none","containingLimits":{"width":"unbounded","height":"unbounded"}}}},"children":[first.clone(),br,first],"worldRoundedLayout":{"x":0,"y":0,"width":40,"height":20},"unroundedLayout":{"x":0,"y":0,"width":40,"height":20}})
}
#[test]
fn named_atomic_metric_decoder_and_real_reader_keep_explicit_certificate() {
    let value = node();
    let xml = generate_xml("named-atomic", &value);
    assert!(xml.contains("atomic-paragraph-metrics="));
    let golden = browser_parity_support::Golden::parse(&xml).unwrap();
    assert_eq!(golden.root.children.len(), 3);
    for actual in std::iter::once(&golden.root).chain(&golden.root.children) {
        assert_eq!(actual.style.get("font-family"), Some("ahem"));
        assert_eq!(actual.style.get("font-size"), Some("10px"));
    }
    browser_parity_support::assert_named_atomic_paragraph_inputs(&golden.root)
        .expect("actual paragraph and BR must lower with certified Named metrics");
    for invalid in [
        xml.replacen("font-family=\"ahem\"", "font-family=\"monospace\"", 1),
        xml.replacen(" font-size=\"10px\"", "", 1),
        xml.replace("font-size=\"10px\"", "font-size=\"11px\""),
    ] {
        assert!(browser_parity_support::Golden::parse(&invalid).is_err());
    }
    let mut physical = value;
    physical
        .as_object_mut()
        .unwrap()
        .remove("atomicParagraphMetrics");
    let xml = generate_xml("raw-physical", &physical);
    assert!(!xml.contains("atomic-paragraph-metrics="));
    let physical = browser_parity_support::Golden::parse(&xml).unwrap();
    for actual in std::iter::once(&physical.root).chain(&physical.root.children) {
        assert_eq!(actual.style.get("font-family"), Some("ahem"));
        assert_eq!(actual.style.get("font-size"), Some("10px"));
    }
}
#[test]
fn named_atomic_html_default_preserves_raw_alignment_token_through_xml() {
    for raw_default in ["auto", "baseline"] {
        let mut value = node();
        value["atomicParagraphMetrics"]["alignmentBaseline"] = json!(raw_default);
        let xml = generate_xml("html-default-reference", &value);
        let golden = browser_parity_support::Golden::parse(&xml).unwrap();
        let capture: Value =
            serde_json::from_str(golden.root.style.get("atomic-paragraph-metrics").unwrap())
                .unwrap();
        assert_eq!(capture["alignmentBaseline"], raw_default);
        assert_eq!(capture["verticalAlign"], "baseline");
        browser_parity_support::assert_named_atomic_paragraph_inputs(&golden.root).unwrap();
    }
}
#[test]
fn named_atomic_raw_initial_alignment_elision_preserves_actual_inputs() {
    for raw_default in ["auto", "baseline"] {
        let mut explicit = node();
        explicit["atomicParagraphMetrics"]["alignmentBaseline"] = json!(raw_default);
        let expected = generate_xml("html-initial-alignment", &explicit);
        // Every subset of the four raw non-inherited initial style fields
        // must retain the same required computed certificate and Native inputs.
        for omitted in 0_u8..16 {
            let mut value = explicit.clone();
            for occurrence in 0..4 {
                if omitted & (1 << occurrence) != 0 {
                    let style = if occurrence == 0 {
                        &mut value["style"]
                    } else {
                        &mut value["children"][occurrence - 1]["style"]
                    };
                    style.as_object_mut().unwrap().remove("verticalAlign");
                }
            }
            let xml = generate_xml("html-initial-alignment", &value);
            assert_eq!(
                xml, expected,
                "raw default {raw_default}, omitted {omitted}"
            );
            let golden = browser_parity_support::Golden::parse(&xml).unwrap();
            browser_parity_support::assert_named_atomic_paragraph_inputs(&golden.root).unwrap();
        }
    }
}
#[test]
fn named_atomic_decoder_rejects_unowned_reference_resource_and_actual_children() {
    let value = node();
    for occurrence in 0..4 {
        for field in ["fontFamily", "fontSize", "verticalAlign"] {
            if field != "verticalAlign" {
                let mut missing = value.clone();
                let style = if occurrence == 0 {
                    &mut missing["style"]
                } else {
                    &mut missing["children"][occurrence - 1]["style"]
                };
                style.as_object_mut().unwrap().remove(field);
                assert!(
                    measured_xml("missing-actual-font", &missing).is_err(),
                    "{occurrence}.{field}"
                );
            }
            let mut conflicting = value.clone();
            let style = if occurrence == 0 {
                &mut conflicting["style"]
            } else {
                &mut conflicting["children"][occurrence - 1]["style"]
            };
            style[field] = match field {
                "fontFamily" => json!("monospace"),
                "fontSize" => json!({"unit":"px","value":11}),
                "verticalAlign" => json!("middle"),
                _ => unreachable!(),
            };
            assert!(
                measured_xml("conflicting-actual-font", &conflicting).is_err(),
                "{occurrence}.{field}"
            );
        }
    }
    for path in [
        "profile",
        "font",
        "order",
        "reference",
        "before",
        "verticalAlign",
    ] {
        let mut invalid = value.clone();
        invalid["atomicParagraphMetrics"]
            .as_object_mut()
            .unwrap()
            .remove(path);
        assert!(measured_xml("missing", &invalid).is_err(), "{path}");
    }
    for invalid in [
        {
            let mut v = value.clone();
            v["atomicParagraphMetrics"]["font"]["loaded"] = json!(false);
            v
        },
        {
            let mut v = value.clone();
            v["atomicParagraphMetrics"]["after"] = json!(-3);
            v
        },
        {
            let mut v = value.clone();
            v["children"][1]["style"]["inlineMetrics"] = json!({"baseline":1,"lineHeight":1});
            v
        },
        {
            let mut v = value.clone();
            v["children"][0]["atomicInlineParticipation"]["bidiLevel"] = json!(0);
            v
        },
        {
            let mut v = value;
            v["children"].as_array_mut().unwrap().remove(1);
            v
        },
    ] {
        assert!(measured_xml("invalid", &invalid).is_err());
    }
}
#[test]
fn actual_css_face_loading_custody_and_neutral_bidi_are_not_defaults() {
    let script = [r#"
const window = {};
const Node = { ELEMENT_NODE:1, TEXT_NODE:3, COMMENT_NODE:8 };
const CSSRule = { STYLE_RULE:1 };
let direction = 'rtl', mode = 'vertical-rl', breakVerticalAlign = 'baseline';
const own = { fontFamily:'ahem',fontSize:'10px',lineHeight:'0px',fontStyle:'normal',fontWeight:'400',fontStretch:'100%',fontFeatureSettings:'normal',fontVariationSettings:'normal',fontVariant:'normal',fontSizeAdjust:'none',textOrientation:'mixed',dominantBaseline:'auto',alignmentBaseline:'auto',textCombineUpright:'none',unicodeBidi:'normal',whiteSpace:'normal',textEmphasisStyle:'none',verticalAlign:'baseline',cssFloat:'none',position:'relative',top:'auto',right:'auto',bottom:'auto',left:'auto' };
function element(tag) { return { nodeType:1,tagName:tag,childNodes:[],parentElement:null,style:{getPropertyValue(){return '';}},closest(){return null;},getAttribute(){return null;},hasAttribute(){return false;} }; }
const root = element('DIV'); root.id='test-root'; root.getAttribute=name=>name==='data-surgeist-layout-ready-inline'?'true':null;
root.childNodes=[element('SPAN'),element('BR'),element('SPAN')]; for(const child of root.childNodes)child.parentElement=root;
root.querySelectorAll=()=>root.childNodes;
function getComputedStyle(node) { return {...own,direction,writingMode:mode,verticalAlign:node.tagName==='BR'?breakVerticalAlign:own.verticalAlign,display:node===root?'block':node.tagName==='BR'?'inline':'inline-block'}; }
function cssFace() { return {family:'ahem',style:'normal',weight:'normal',stretch:'normal',unicodeRange:'U+0-10FFFF',featureSettings:'normal',variationSettings:'normal',ascentOverride:'normal',descentOverride:'normal',lineGapOverride:'normal',sizeAdjust:'100%',status:'unloaded',async load(){this.status='loaded';return this;}}; }
const face=cssFace(),faces=[face]; faces.ready=Promise.resolve();
const document={fonts:faces,styleSheets:[],getElementById(){return root;}};
"#, TEST_HELPER_SOURCE, r#"
(async()=>{
  rootContextCapture={ownedStyle:{}};
  await prepareNamedReferenceFont(); requireNamedReferenceFont();
  if(namedReferenceFontState.face!==face || face.status!=='loaded')throw new Error('actual CSS face was not retained');
  function described(){return [{atomicInlineParticipation:{bidiLevel:direction==='rtl'?1:0,followingBreak:'prohibited'}},{lineControlParticipation:{kind:'forced-break'},style:{inlineMetrics:{baseline:0,lineHeight:0}}},{atomicInlineParticipation:{bidiLevel:direction==='rtl'?1:0,followingBreak:'prohibited'}}];}
  for(const rawDefault of ['auto','baseline']){own.alignmentBaseline=rawDefault;for(direction of ['ltr','rtl']){const metrics=atomicParagraphMetricsForElement(root,described());if(metrics.alignmentBaseline!==rawDefault||metrics.verticalAlign!=='baseline'||metrics.font.sha256!=='e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c'||metrics.before!==0||metrics.after!==0||metrics.order[2].source!==2||metrics.order[0].bidiLevel!==(direction==='rtl'?1:0))throw new Error('source/reference tuple changed');}}
  function fails(action,expected){let message;try{action();}catch(error){message=error.message;}if(!message?.includes(expected))throw new Error('missing precise refusal '+expected+': '+message);}
  own.alignmentBaseline='central';fails(()=>atomicParagraphMetricsForElement(root,described()),'UnsupportedReferenceContext');own.alignmentBaseline='auto';
  own.verticalAlign='middle';fails(()=>atomicParagraphMetricsForElement(root,described()),'UnsupportedReferenceContext');own.verticalAlign='baseline';
  breakVerticalAlign='middle';fails(()=>atomicParagraphMetricsForElement(root,described()),'MismatchedOccurrenceReference');breakVerticalAlign='baseline';
  for(const edge of ['top','right','bottom','left'])own[edge]='0px';
  atomicParagraphMetricsForElement(root,described());
  own.left='1px';own.right='-1px';fails(()=>atomicParagraphMetricsForElement(root,described()),'UnsupportedOccurrence');
  for(const edge of ['top','right','bottom','left'])own[edge]='auto';
  faces.push(cssFace());fails(requireNamedReferenceFont,'CompetingFontResource');faces.pop();
  faces[0]=cssFace();faces[0].status='loaded';fails(requireNamedReferenceFont,'FontResourceChanged');
  await prepareNamedReferenceFont();fails(requireNamedReferenceFont,'FontResourceChanged');
  faces[0]=face;requireNamedReferenceFont();if(namedReferenceFontState.face!==face)throw new Error('pristine restore lost original face');
  mode='horizontal-tb'; if(atomicParagraphMetricsForElement(root,described())!==undefined)throw new Error('raw Physical context was promoted');
  mode='vertical-rl'; root.closest=()=>({}); if(atomicParagraphMetricsForElement(root,described())!==undefined)throw new Error('Prepared context was promoted'); root.closest=()=>null;
  rootContextCapture={ownedStyle:{}};faces.length=0;await prepareNamedReferenceFont();fails(requireNamedReferenceFont,'MissingFontResource');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#].concat();
    run_bundled_helper_script("named-atomic-font-custody", script);
}

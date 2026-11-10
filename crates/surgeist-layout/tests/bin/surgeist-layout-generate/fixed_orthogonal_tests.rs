//! Exercise the actual local-subtree and analytic-producer gates.
//!
//! The recorder supplies the existing caller-owned source/live equality,
//! stylesheet, horizontal containing frame and unshifted-frame preconditions.
//! It does not exercise those proofs or browser CSS parsing/layout. CLI source
//! controls exercise the complete capture boundary independently.

use super::*;

fn classify(scenario: Value) -> Value {
    helper_probe(&r#"(() => {
      const input = __SCENARIO__;
      const declaration = values => Object.assign({
        getPropertyValue(name) { return values[name] || ''; },
        getPropertyPriority() { return ''; },
        [Symbol.iterator]() { return Object.keys(values)[Symbol.iterator](); },
      }, ...Object.entries(values).map(([key, value]) => ({
        [key.replace(/-([a-z])/g, (_, c) => c.toUpperCase())]: value
      })));
      const box = (mode, display, values) => ({
        tagName: 'DIV', style: declaration(values), children: [], childNodes: [],
        attributes: [], textContent: '', classList: { contains() { return false; } },
        scrollLeft: 0, scrollTop: 0,
        getBoundingClientRect() { throw new Error('a used rectangle cannot prove fixed sizing'); },
        computed: { writingMode: mode, direction: input.direction || 'ltr', display,
          position: 'relative', top: 'auto', right: 'auto', bottom: 'auto', left: 'auto',
          width: '30px', height: '50px', minWidth: '0px', minHeight: '0px',
          maxWidth: 'none', maxHeight: 'none', aspectRatio: 'auto',
          clipPath: 'none', overflowX: 'visible', overflowY: 'visible', content: 'normal',
          marginTop: '0px', marginRight: '0px', marginBottom: '0px', marginLeft: '0px',
          paddingTop: '0px', paddingRight: '0px', paddingBottom: '0px', paddingLeft: '0px',
          borderTopWidth: '0px', borderRightWidth: '0px', borderBottomWidth: '0px', borderLeftWidth: '0px',
        },
      });
      const root = box(input.rootMode || 'horizontal-tb', 'block', {display: 'block'});
      const child = box(input.mode, 'flex', {
        'writing-mode': input.mode, width: '30px', height: '50px', ...input.authored,
      });
      Object.assign(child.computed, input.computed);
      child.tagName = input.tag || 'DIV';
      child.scrollTop = input.scrollTop || 0;
      child.textContent = input.text || '';
      if (child.textContent) child.childNodes.push({nodeType: 3, textContent: child.textContent, length: child.textContent.length});
      if (input.marker) child.attributes.push({name: input.marker, value: 'true'});
      root.children.push(child); root.childNodes.push(child);
      globalThis.getComputedStyle = node => node.computed;
      globalThis.Node = {TEXT_NODE: 3, ELEMENT_NODE: 1};
      try {
        qualifyRootLocalSubtree(root, () => true, false, null);
        const normalized = node => ({style: {writingMode: node.computed.writingMode},
          textContent: node.textContent, children: node.children.map(normalized)});
        qualifyAnalyticCaptureText(normalized(root), 'horizontal-tb');
        return {accepted: true};
      } catch (error) { return {accepted: false, reason: error.message}; }
    })()"#.replace("__SCENARIO__", &scenario.to_string()))
}

#[test]
fn fixed_orthogonal_literal_plain_boxes_accept_all_four_modes_in_both_directions() {
    let mut observed = Vec::new();
    let mut expected = Vec::new();
    for mode in ["vertical-rl", "vertical-lr", "sideways-rl", "sideways-lr"] {
        for direction in ["ltr", "rtl"] {
            observed.push((
                mode,
                direction,
                classify(json!({"mode": mode, "direction": direction})),
            ));
            expected.push((mode, direction, json!({"accepted": true})));
        }
    }
    assert_eq!(observed, expected);
}

#[test]
fn fixed_orthogonal_horizontal_control_keeps_empty_flex_leaf_eligibility() {
    for direction in ["ltr", "rtl"] {
        assert_eq!(
            classify(json!({"mode": "horizontal-tb", "direction": direction})),
            json!({"accepted": true})
        );
    }
}

#[test]
fn fixed_orthogonal_used_sizes_do_not_supply_missing_or_intrinsic_source_dimensions() {
    for property in ["width", "height"] {
        for value in [
            "",
            "auto",
            "min-content",
            "max-content",
            "fit-content(30px)",
            "100%",
            "calc(10px + 20px)",
            "var(--size)",
        ] {
            let mut scenario = json!({"mode": "vertical-rl", "authored": {}});
            scenario["authored"][property] = json!(value);
            assert_eq!(
                classify(scenario)["accepted"],
                false,
                "{property}: {value} with a finite used size"
            );
        }
    }
}

#[test]
fn fixed_orthogonal_source_overrides_and_unsupported_producer_frames_refuse() {
    for authored in [
        json!({"inline-size": "30px"}),
        json!({"block-size": "50px"}),
        json!({"min-width": "10px"}),
        json!({"max-height": "100px"}),
        json!({"aspect-ratio": "1"}),
        json!({"padding-block-start": "1px"}),
        json!({"inset-inline-start": "1px"}),
        json!({"top": "1px"}),
    ] {
        assert_eq!(
            classify(json!({"mode": "vertical-rl", "authored": authored}))["accepted"],
            false
        );
    }
    for extra in [
        json!({"rootMode": "vertical-rl"}),
        json!({"tag": "BR"}),
        json!({"tag": "IMG"}),
        json!({"tag": "SPAN", "computed": {"display": "inline"}}),
        json!({"text": "X"}),
        json!({"marker": "data-surgeist-prepared-text"}),
        json!({"computed": {"position": "fixed"}}),
        json!({"computed": {"position": "absolute"}}),
        json!({"computed": {"clipPath": "inset(1px)"}}),
        json!({"computed": {"overflowX": "hidden"}}),
        json!({"scrollTop": 1}),
    ] {
        let mut scenario = json!({"mode": "vertical-rl"});
        scenario
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert_eq!(classify(scenario)["accepted"], false);
    }
}

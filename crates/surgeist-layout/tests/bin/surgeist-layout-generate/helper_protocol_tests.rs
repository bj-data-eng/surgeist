//! Execute the bundled producer while isolating DOM geometry measurement.

#[path = "fixed_orthogonal_tests.rs"]
mod fixed_orthogonal_tests;

use crate::adapter::TEST_HELPER_SOURCE;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command, Stdio};

fn helper_response(has_root: bool) -> Value {
    // `describeElement` owns geometry and style capture. This recorder leaves
    // the real `getTestData` in charge of missing-source refusal, body mutation, variant
    // order, and JSON transport, which are the protocol contracts under test.
    let script = r#"
const vm = require('node:vm');
const fs = require('node:fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const root = {};
const measurements = [];
const bodyClasses = [];
const rootQueries = [];
let className = '';
const context = vm.createContext({
  window: {},
  document: {
    getElementById(id) {
      rootQueries.push(id);
      return input.hasRoot ? root : null;
    },
    body: {
      get className() { return className; },
      set className(value) { className = value; bodyClasses.push(value); },
    },
  },
});
vm.runInContext(input.helperSource, context);
context.describeElement = element => {
  if (element !== root) throw new Error('measured a different root');
  measurements.push(className);
  return { observedClass: className };
};
const response = vm.runInContext('getTestData()', context);
process.stdout.write(JSON.stringify({ response, measurements, bodyClasses, rootQueries }));
"#;
    run_node(
        script,
        json!({ "helperSource": TEST_HELPER_SOURCE, "hasRoot": has_root }),
    )
}

fn run_node(script: &str, input: Value) -> Value {
    let mut child = Command::new("node")
        .args(["-e", script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node must execute the bundled measurement helper");
    child
        .stdin
        .take()
        .expect("piped Node input")
        .write_all(
            serde_json::to_string(&input)
                .expect("helper input is JSON")
                .as_bytes(),
        )
        .expect("write the captured helper source to Node");
    let output = child
        .wait_with_output()
        .expect("wait for the helper result");
    assert!(
        output.status.success(),
        "helper failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout).expect("helper harness returns JSON")
}

fn decode_envelope(result: &Value) -> Value {
    let response = result["response"]
        .as_str()
        .expect("getTestData retains JSON-string transport");
    let envelope: Value =
        serde_json::from_str(response).expect("measurement envelope is valid JSON");
    assert_eq!(envelope["schemaVersion"].as_u64(), Some(4));
    assert_eq!(
        envelope
            .as_object()
            .expect("measurement envelope is an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "borderBoxLtrData",
            "borderBoxRtlData",
            "contentBoxLtrData",
            "contentBoxRtlData",
            "schemaVersion",
        ],
    );
    envelope
}

#[test]
fn present_root_without_source_authority_returns_four_precise_refusals() {
    let result = helper_response(true);
    let envelope = decode_envelope(&result);
    let expected_classes = json!([
        "border-box ltr",
        "content-box ltr",
        "border-box rtl",
        "content-box rtl",
    ]);
    assert_eq!(result["rootQueries"], json!(["test-root"]));
    assert_eq!(result["bodyClasses"], expected_classes);
    assert_eq!(result["measurements"], json!([]));
    for key in [
        "borderBoxLtrData",
        "contentBoxLtrData",
        "borderBoxRtlData",
        "contentBoxRtlData",
    ] {
        assert_eq!(
            envelope[key],
            json!({ "layoutInput": "unsupported", "unsupportedReason": "Root context: MissingSourceAuthority" })
        );
    }
}

#[test]
fn missing_root_returns_version_four_with_four_unsupported_variants() {
    let result = helper_response(false);
    let envelope = decode_envelope(&result);
    assert_eq!(result["rootQueries"], json!(["test-root"]));
    assert_eq!(result["bodyClasses"], json!([]));
    assert_eq!(result["measurements"], json!([]));
    for key in [
        "borderBoxLtrData",
        "contentBoxLtrData",
        "borderBoxRtlData",
        "contentBoxRtlData",
    ] {
        assert_eq!(
            envelope[key],
            json!({ "layoutInput": "unsupported", "unsupportedReason": "Unsupported missing #test-root fixture root" }),
        );
    }
}

fn helper_probe(probe: &str) -> Value {
    run_node(
        r#"
const vm = require('node:vm');
const fs = require('node:fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const context = vm.createContext({ window: {} });
vm.runInContext(input.helperSource, context);
process.stdout.write(JSON.stringify(vm.runInContext(input.probe, context)));
"#,
        json!({ "helperSource": TEST_HELPER_SOURCE, "probe": probe }),
    )
}

#[test]
fn br_metrics_are_numeric_and_zero_height_is_preserved() {
    let result = helper_probe("brInlineMetricsForElement({tagName: 'BR'}, {lineHeight: '0px'})");
    assert_eq!(result, json!({ "baseline": 0, "lineHeight": 0 }));
}

#[test]
fn shape_measurements_use_an_optional_interval_object() {
    let result = helper_probe(
        r#"layoutReadyShapeBands({ getAttribute: () => JSON.stringify([
          {bandMinimum: 0, bandMaximum: 10, intervalMinimum: 2, intervalMaximum: 8},
          {bandMinimum: 10, bandMaximum: 20}
        ]) })"#,
    );
    assert_eq!(
        result,
        json!([
            { "bandMinimum": 0, "bandMaximum": 10, "interval": { "minimum": 2, "maximum": 8 } },
            { "bandMinimum": 10, "bandMaximum": 20 },
        ])
    );
}

#[test]
fn named_grid_placements_omit_absent_occurrences() {
    let result = helper_probe(
        "['header', 'header -2', 'span header', 'span 3 header'].map(parseGridPosition)",
    );
    assert_eq!(
        result,
        json!([
            { "kind": "named-line", "name": "header" },
            { "kind": "named-line", "name": "header", "occurrence": -2 },
            { "kind": "named-span", "name": "header" },
            { "kind": "named-span", "name": "header", "occurrence": 3 },
        ])
    );
}

#[test]
fn root_script_profiles_preserve_source_absence_and_exact_live_custody() {
    // These controlled DOM facts exercise the actual profile/inventory guards;
    // the real-browser qualifier separately checks parsing and reconfiguration.
    let observed = run_node(
        include_str!("script_custody_tests.js"),
        json!({"helperSource": TEST_HELPER_SOURCE}),
    );
    assert_eq!(
        observed,
        json!({"sourceProfiles": ["absent", "helper"], "restored": true}),
    );
}

#[test]
fn prepared_transparent_inline_profile_preserves_or_refuses_actual_context() {
    // Execute the exact producer with DOM computed-style facts. No Text provider
    // or browser-output geometry is substituted by this focused profile test.
    run_node(
        include_str!("prepared_capture_tests.js"),
        json!({"helperSource":TEST_HELPER_SOURCE, "fontBytes": include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf").as_slice()}),
    );
}

#[test]
fn prepared_atomic_br_profile_preserves_vertical_source_and_rejects_unrepresented_context() {
    // Original atomic/control occurrences and computed styles are supplied to
    // the actual helper. DOM Range replies are controlled observation records;
    // this test makes no browser layout or FontFace-loading claim.
    let observed = run_node(
        include_str!("prepared_capture_tests.js"),
        json!({"suite":"atomic-br", "helperSource":TEST_HELPER_SOURCE, "fontBytes":include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf").as_slice()}),
    );
    assert!(observed.as_array().unwrap().iter().any(|row| {
        row["writingMode"] == "vertical-lr" && row["direction"] == "ltr" && row["admitted"] == true
    }));
}

#[test]
fn explicit_br_font_band_observation_is_independent_of_input_participation() {
    // The actual producer receives controlled DOM facts. These assertions
    // qualify explicit observation intent, not browser metrics or font loading.
    let observed = run_node(
        include_str!("prepared_capture_tests.js"),
        json!({"suite":"br-observation", "helperSource":TEST_HELPER_SOURCE, "fontBytes":include_bytes!("../../layout/browser_parity/fonts/Ahem.ttf").as_slice()}),
    );
    assert_eq!(
        observed,
        json!([
            {"request":null,"participating":null,"unparticipating":null},
            {"request":"font-band","participating":"font-band","unparticipating":"font-band"},
        ]),
    );
}

#[test]
fn analytic_profile_keeps_orthogonal_ancestry_distinct_from_known_glyph_free_boxes() {
    let result = helper_probe(
        r#"(() => {
      const attempt = node => {
        try { qualifyAnalyticCaptureText(node, 'horizontal-tb'); return 'accepted'; }
        catch (error) { return error.message; }
      };
      return [
        attempt({style:{writingMode:'horizontal-tb'},textContent:'X'}),
        attempt({style:{writingMode:'vertical-rl'},textContent:'X'}),
        attempt({style:{writingMode:'vertical-rl'},children:[{style:{writingMode:'horizontal-tb'},textContent:'X'}]}),
        attempt({style:{writingMode:'vertical-rl'},children:[]}),
      ];
    })()"#,
    );
    assert_eq!(
        result,
        json!([
            "accepted",
            "Root context: UnsupportedAnalyticOrthogonalTextProfile",
            "Root context: UnsupportedAnalyticOrthogonalTextProfile",
            "accepted",
        ])
    );
}

#[test]
fn prepared_range_requires_the_unchanged_effective_font_face() {
    let refusals = helper_probe(
        r#"(() => {
          const face = {
            family: 'SurgeistParityAhem', status: 'loaded',
            style: 'normal', weight: '400', stretch: 'normal',
            unicodeRange: 'U+0-10FFFF', featureSettings: 'normal', variationSettings: 'normal',
            ascentOverride: 'normal', descentOverride: 'normal',
            lineGapOverride: 'normal', sizeAdjust: '100%'
          };
          const stylesheetFace = { family: 'ahem' };
          globalThis.document = { fonts: [face, stylesheetFace] };
          pinnedTextFontState = { face };
          qualifyLoadedPreparedRangeFace();
          const attempts = {
            unicodeRange: 'U+0041', ascentOverride: '10%', descentOverride: '10%',
            lineGapOverride: '10%', sizeAdjust: '75%'
          };
          const refused = [];
          const qualify = reason => {
            try { qualifyLoadedPreparedRangeFace(); }
            catch (error) {
              if (error.message !== 'Root context: PreparedRangeFontAuthority') throw error;
              refused.push(reason);
            }
          };
          for (const [descriptor, replacement] of Object.entries(attempts)) {
            const prior = face[descriptor];
            face[descriptor] = replacement;
            qualify(descriptor);
            face[descriptor] = prior;
          }
          for (const family of ['surgeistparityahem', '\u017furgeistparityahem']) {
            document.fonts.push({ ...face, family });
            qualify(family);
            document.fonts.pop();
          }
          document.fonts.pop();
          qualify('missing-stylesheet-face');
          return refused;
        })()"#,
    );
    assert_eq!(
        refusals,
        json!([
            "unicodeRange",
            "ascentOverride",
            "descentOverride",
            "lineGapOverride",
            "sizeAdjust",
            "surgeistparityahem",
            "ſurgeistparityahem",
            "missing-stylesheet-face"
        ])
    );
}

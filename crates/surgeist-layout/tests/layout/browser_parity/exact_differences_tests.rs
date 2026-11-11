use super::*;
use serde_json::{Value, json};

// Independently authored finite arithmetic: an empty 20x10 Flex box, zero
// edges, Visible overflow, no named eligible items or scroll range. No value in
// this oracle is read from Layout or from CompletedComparison.
fn oracle() -> Value {
    let point = json!({"x":0,"y":0});
    let size = json!({"width":20,"height":10});
    let rect = json!({"origin":point,"size":size});
    let edges = json!({"top":0,"right":0,"bottom":0,"left":0});
    let flow = json!({"writing_mode":"horizontal-tb","direction":"ltr"});
    let pair = json!({"first":{"state":"absent"},"last":{"state":"absent"}});
    let facts = json!({
        "source_index":0,"location":point,"size":size,"content_size":size,
        "border":edges,"padding":edges,"margin":edges,
        "named_baselines":{"alphabetic":pair,"central":pair},
        "scroll":{"state":"present","facts":{
            "flow_axes":flow,"used_overflow_x":"visible","used_overflow_y":"visible",
            "border_box":rect,"padding_box":rect,"content_box":rect,"scrollport":rect,
            "clip_x":null,"clip_y":null,"scrollable_overflow":rect,
            "range_x":{"minimum":0,"maximum":0},"range_y":{"minimum":0,"maximum":0},
            "gutter_top":null,"gutter_right":null,"gutter_bottom":null,"gutter_left":null,
            "scrollbar_size":{"width":0,"height":0},"resolved_scroll_padding":edges,
            "optimal_viewing_region":rect,"scroll_snap_type":{"state":"none"},
            "target":{"border_box":rect,"scroll_margin":edges,"flow_axes":flow,
                "block_align":"none","inline_align":"none","snap_stop":"normal"}
        }}
    });
    json!({"path":"/","role":"ordinary-box","children":[],
        "unrounded":{"state":"present","facts":facts},"final_output":{"state":"present","facts":facts}})
}
fn identity() -> Value {
    let hash = "a".repeat(64);
    json!({"source":"html/empty-fixed.html","source_sha256":hash,"variant":"content_box_ltr",
        "historical_sha256":null,"capture_sha256":hash,"record_sha256":hash,
        "generator":"test","engine_version":"test",
        "browser_executable":"chrome","browser_executable_sha256":hash,"browser_provenance":"test",
        "profile_sha256":hash,"source_imports_sha256":hash,"inputs":{"base.css":hash},"linked_resources":{},
        "font_demand":"none","policy_source_sha256":hash})
}
fn declaration() -> Value {
    json!({"profile":"canonical-no-text-boxes","identities":{"focused":identity()},
        "representation":{"published_edition":null},
        "policy":{"id":"fixed-box-mechanism","edition":"1","source":"docs/policy.md","source_sha256":"a".repeat(64)},
        "reason":"Mechanism test of a source-derived fixed box",
        "native":{"source_derivation":"20px by 10px empty Flex box; initial canonical geometry and named absence", "nodes":[oracle()]},
        "differences":[{"domain":"layout","path":"/","field":"width","browser":21,"native":20}]})
}
const FIXED_XML: &str = r#"<test name="exact-fixed" measurement-version="4" observation-frame="root-border" use-rounding="false">
<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div grid-alignment-profile="{&quot;profile&quot;:1,&quot;owners&quot;:[]}" display="flex" position="relative" direction="ltr" width="20px" height="10px"/></input>
<expectations><node x="0" y="0" width="21" height="10"/></expectations>
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="21" height="10"/></diagnostics></test>"#;
fn golden() -> Golden {
    Golden::parse(FIXED_XML).unwrap()
}
fn spec(value: &Value) -> ExactDifferenceSpec {
    ExactDifferenceSpec::parse(&value.to_string()).unwrap()
}
fn qualify(value: &Value, golden: &Golden) -> Result<QualifiedComparison> {
    let spec = spec(value);
    let identity = CaseIdentity::from_verified_projection(identity()).unwrap();
    let resolved = spec.resolve_against(CaptureAuthority::Focused(&identity), golden)?;
    let completed = complete_captured_comparison(golden, &resolved)?;
    resolved.qualify(&completed)
}

#[test]
fn wrapping_world_corner_height_is_checked_separately_from_root_local_height() {
    let mut value: Value = serde_json::from_str(include_str!(
        "captured/canonical-box-differences/semantic/collapsed-flex-wrapping-border_box_ltr.json"
    ))
    .unwrap();
    let edition = json!("wrapping-world-corner-differences");
    value["representation"]["published_edition"] = edition.clone();
    value["representation"]["current"]["selectors"]
        .as_array_mut()
        .unwrap()
        .insert(4, json!({"path":"/","field":"world-corner-height"}));
    value["differences"].as_array_mut().unwrap().push(json!({
        "domain":"world-corner","path":"/","field":"height","browser":157,"native":68
    }));
    let specification = ExactDifferenceSpec::parse(&value.to_string()).unwrap();
    let edition: PublishedEdition = serde_json::from_value(edition).unwrap();
    let identity =
        CaseIdentity::from_verified_projection(value["identities"]["published"].clone()).unwrap();
    let authority = || CaptureAuthority::Published {
        edition,
        identity: &identity,
    };
    let mut golden = Golden::parse(include_str!(
        "captured/canonical-box-differences/xml/flex/fri07_collapsed_strut_wrapping__border_box_ltr.xml"
    ))
    .unwrap();
    let historical = include_bytes!("xml/flex/fri07_collapsed_strut_wrapping__border_box_ltr.xml");
    verify_wrapping_world_predecessor(&golden, historical).unwrap();
    let resolved = specification.resolve_against(authority(), &golden).unwrap();
    let complete = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(
        resolved.qualify(&complete).unwrap().raw,
        RawVerdict::Different { fields: 13 }
    );

    let root_world = golden
        .world_rounding_diagnostics
        .iter_mut()
        .find(|row| row.node_path == "0")
        .unwrap();
    root_world.height -= 1.0;
    assert_eq!(golden.expectations.height, Some(157.0));
    assert!(verify_wrapping_world_predecessor(&golden, historical).is_err());
    let resolved = specification.resolve_against(authority(), &golden).unwrap();
    let complete = complete_captured_comparison(&golden, &resolved).unwrap();
    assert!(matches!(
        resolved.qualify(&complete),
        Err(QualificationError::DifferenceSetChanged { .. })
    ));
}

#[test]
fn complete_source_derived_fixed_box_qualifies_while_raw_still_fails() {
    let golden = golden();
    let spec = spec(&declaration());
    assert_eq!(
        spec.policy_source(),
        ("docs/policy.md", "a".repeat(64).as_str())
    );
    let identity = CaseIdentity::from_verified_projection(identity()).unwrap();
    let resolved = spec
        .resolve_against(CaptureAuthority::Focused(&identity), &golden)
        .unwrap();
    let complete = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(
        complete.observations.len(),
        4,
        "equal requested fields are retained"
    );
    assert_eq!(
        resolved.qualify(&complete).unwrap(),
        QualifiedComparison {
            raw: RawVerdict::Different { fields: 1 },
            qualified: QualifiedVerdict::Intentional {
                policy: "fixed-box-mechanism".into(),
                edition: "1".into()
            },
        }
    );
    assert_eq!(
        collect_surgeist_numeric_mismatches(&golden).unwrap().len(),
        1
    );
    assert_eq!(
        assert_surgeist_matches(&golden).unwrap_err().to_string(),
        "exact-fixed: width mismatch, expected 21, got 20"
    );
}
#[test]
fn later_requested_field_and_unexpected_pass_refuse_a_declared_prefix() {
    let mut golden = golden();
    golden.expectations.height = Some(11.0);
    assert!(
        matches!(qualify(&declaration(), &golden), Err(QualificationError::DifferenceSetChanged {extra,..}) if extra == ["layout:/:Height"])
    );
    golden.expectations.width = Some(20.0);
    golden.expectations.height = Some(10.0);
    assert_eq!(
        qualify(&declaration(), &golden),
        Err(QualificationError::UnexpectedPass)
    );
}
#[test]
fn declared_pair_drift_below_browser_tolerance_is_exactly_rejected() {
    let mut value = declaration();
    value["differences"][0]["browser"] = json!(21.0625);
    assert!(
        matches!(qualify(&value, &golden()), Err(QualificationError::DifferenceSetChanged {changed,..}) if changed == ["layout:/:Width"])
    );
    assert!(numeric_values_match(21.0, 21.0625));
}
#[test]
fn complete_packet_oracle_checks_unobserved_components_and_both_phases() {
    let replacements = [
        (
            "/unrounded/facts/content_size/width",
            json!(19),
            "content_size.width",
        ),
        ("/final_output/facts/margin/left", json!(-1), "margin.left"),
        (
            "/unrounded/facts/named_baselines/central/last/state",
            json!("unknown"),
            "named_baselines.central.last.state",
        ),
        (
            "/unrounded/facts/scroll/facts/range_x",
            json!({"minimum":2,"maximum":2}),
            "range_x.maximum",
        ),
        (
            "/final_output/facts/scroll/facts/scrollable_overflow/origin/x",
            json!(-1),
            "scrollable_overflow.origin.x",
        ),
        (
            "/unrounded/facts/scroll/facts/target/snap_stop",
            json!("always"),
            "target.snap_stop",
        ),
        ("/final_output", json!({"state":"absent"}), "final_output"),
    ];
    for (pointer, replacement, key) in replacements {
        let mut value = declaration();
        *value["native"]["nodes"][0].pointer_mut(pointer).unwrap() = replacement;
        assert!(
            matches!(qualify(&value, &golden()), Err(QualificationError::NativeOracleChanged {key: actual,..}) if actual.ends_with(key)),
            "{pointer}"
        );
    }
}
#[test]
fn source_census_and_closed_observation_profile_refuse_incomplete_or_inline_inputs() {
    let mut value = declaration();
    value["native"]["nodes"][0]["children"] = json!(["/0"]);
    assert!(matches!(
        qualify(&value, &golden()),
        Err(QualificationError::IncompleteNativeOracle { .. })
    ));
    let mut golden = golden();
    golden.root.text = Some("X".into());
    assert!(matches!(
        qualify(&declaration(), &golden),
        Err(QualificationError::UnsupportedObservation { .. })
    ));
    golden.root.text = None;
    golden.expectations.fragments = Some(Vec::new());
    assert!(matches!(
        qualify(&declaration(), &golden),
        Err(QualificationError::UnsupportedObservation { .. })
    ));
}
#[test]
fn identity_drift_rejects_identical_geometry() {
    let spec = spec(&declaration());
    for field in [
        "source_sha256",
        "capture_sha256",
        "browser_executable_sha256",
        "profile_sha256",
        "policy_source_sha256",
    ] {
        let mut value = identity();
        value[field] = json!("b".repeat(64));
        let identity = CaseIdentity::from_verified_projection(value).unwrap();
        assert!(
            matches!(
                spec.resolve_against(CaptureAuthority::Focused(&identity), &golden()),
                Err(QualificationError::IdentityChanged { .. })
            ),
            "{field}"
        );
    }
}
#[test]
fn canonical_parser_preserves_an_exact_widened_binary32_coordinate() {
    let value = prepared_source::unique_json("95.20001220703125").unwrap();
    let actual: Number =
        serde_json::from_value(value).expect("exact widened binary32 must survive JSON decoding");
    assert_eq!(actual, Number(f32::from_bits(0x42be_6668)));

    let off_grid = prepared_source::unique_json("95.20001220703124").unwrap();
    assert!(serde_json::from_value::<Number>(off_grid).is_err());
}

#[test]
fn parser_rejects_duplicates_unknown_missing_noncanonical_and_negative_extents() {
    let raw = declaration().to_string();
    assert!(
        ExactDifferenceSpec::parse(&raw.replacen(
            "\"reason\":",
            "\"reason\":\"duplicate\",\"reason\":",
            1
        ))
        .is_err()
    );
    for pointer in [
        "/identities/focused/historical_sha256",
        "/native/nodes/0/unrounded/facts/scroll/facts/clip_x",
        "/native/nodes/0/final_output/facts/scroll/facts/clip_y",
        "/native/nodes/0/unrounded/facts/scroll/facts/gutter_top",
        "/native/nodes/0/final_output/facts/scroll/facts/gutter_right",
        "/native/nodes/0/unrounded/facts/scroll/facts/gutter_bottom",
        "/native/nodes/0/final_output/facts/scroll/facts/gutter_left",
        "/native/nodes/0/final_output/facts/source_index",
        "/differences/0/native",
    ] {
        let mut value = declaration();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        value
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            ExactDifferenceSpec::parse(&value.to_string()).is_err(),
            "{pointer}"
        );
    }
    let mut value = declaration();
    value["native"]["nodes"][0]["unrounded"]["facts"]["size"]["width"] = json!(-1);
    assert!(ExactDifferenceSpec::parse(&value.to_string()).is_err());
    let mut value = declaration();
    value["differences"][0]["native"] = Value::Null;
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { reason })
            if reason == "absent Native value requires a scroll span"
    ));
    let mut value = declaration();
    value["differences"][0]["browser"] = json!(21.01);
    assert!(ExactDifferenceSpec::parse(&value.to_string()).is_err());
    let mut value = declaration();
    value["unowned"] = json!(true);
    assert!(ExactDifferenceSpec::parse(&value.to_string()).is_err());
    let mut value = declaration();
    value["native"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(oracle());
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::DuplicateKey { .. })
    ));
}
#[test]
fn source_output_change_in_an_equal_field_cannot_preserve_qualification() {
    let mut golden = golden();
    golden
        .root
        .style
        .attrs
        .insert("height".into(), "11px".into());
    golden.expectations.height = Some(11.0);
    assert!(matches!(
        qualify(&declaration(), &golden),
        Err(QualificationError::NativeOracleChanged { .. })
    ));
}
#[test]
fn late_terminal_topology_failure_returns_no_completed_comparison() {
    let mut golden = golden();
    let spec = spec(&declaration());
    let identity = CaseIdentity::from_verified_projection(identity()).unwrap();
    let resolved = spec
        .resolve_against(CaptureAuthority::Focused(&identity), &golden)
        .unwrap();
    golden
        .expectations
        .children
        .push(golden.expectations.clone());
    assert!(matches!(
        complete_captured_comparison(&golden, &resolved),
        Err(QualificationError::ComparisonFailure { .. })
    ));
}
#[test]
fn normalized_signed_zero_is_exact_and_dom_fields_keep_their_domain() {
    assert_eq!(Number::new(-0.0).unwrap(), Number::new(0.0).unwrap());
    let layout = ComparedField::layout(
        "/".into(),
        NumericComparisonField::ScrollWidth,
        1.0,
        Some(0.0),
    )
    .unwrap();
    let dom = ComparedField::element("/".into(), ElementScrollSizeField::ScrollWidth, 1, 0);
    assert_ne!(layout.key(), dom.key());
}

fn element_golden(scroll_width: u32) -> Golden {
    let raw = FIXED_XML
        .replace("observation-frame=\"root-border\"", "observation-frame=\"root-border-local\"")
        .replace("root-context=\"block-flow-auto\" parent-inline-extent=\"800px\" parent-direction=\"ltr\"", "root-context=\"absolute-root-local\" containing-width=\"800px\" containing-height=\"600px\" containing-direction=\"ltr\" hypothetical-direction=\"ltr\" hypothetical-display=\"flex\" hypothetical-anchor=\"inline-start\" hypothetical-inline-offset=\"0px\"")
        .replace("display=\"flex\" position=\"relative\"", "source-tag=\"div\" display=\"flex\" position=\"absolute\"")
        .replace("<diagnostics>", &format!("<element-scroll-sizes profile=\"horizontal-zero-edge-divs\"><element node-path=\"0\" client-width=\"20\" client-height=\"10\" scroll-width=\"{scroll_width}\" scroll-height=\"10\"/></element-scroll-sizes><diagnostics>"));
    Golden::parse(&raw).unwrap()
}
#[test]
fn complete_mixed_domain_ledger_retains_equal_integers_and_rejects_extra_scroll_differences() {
    let value = declaration();
    let spec = spec(&value);
    let identity = CaseIdentity::from_verified_projection(identity()).unwrap();
    let golden = element_golden(20);
    let resolved = spec
        .resolve_against(CaptureAuthority::Focused(&identity), &golden)
        .unwrap();
    let completed = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(
        completed.observations.len(),
        8,
        "all four equal DOM integer fields are retained too"
    );
    resolved.qualify(&completed).unwrap();

    let mut golden = element_golden(21);
    golden.expectations.scroll_size = Some(Size::new(1.0, 0.0));
    assert!(
        matches!(qualify(&value, &golden), Err(QualificationError::DifferenceSetChanged {extra,..}) if extra == ["element-size:/:ScrollWidth", "layout:/:ScrollWidth"])
    );
    let mut value = value;
    value["differences"].as_array_mut().unwrap().extend([
        json!({"domain":"layout","path":"/","field":"scroll-width","browser":1,"native":0}),
        json!({"domain":"element-size","path":"/","field":"scroll-width","browser":21,"native":20}),
    ]);
    assert_eq!(
        qualify(&value, &golden).unwrap().raw,
        RawVerdict::Different { fields: 3 }
    );
}

// These are explicitly authored Native operations. Neither a historical viewport
// nor a browser expectation supplies this standalone request or any Text metric.
fn output_census_native_request() -> layout::LayoutRootRequest {
    let width = layout::NonNegativeFiniteOf::new(100.0).unwrap();
    layout::LayoutRootRequest::standalone(
        layout::SizingRequest::new(
            layout::Size::new(
                layout::AxisSizingRequest::new(
                    layout::AvailableSpace::Definite(width),
                    layout::SizingOperation::Normal,
                ),
                layout::AxisSizingRequest::new(
                    layout::AvailableSpace::Indefinite,
                    layout::SizingOperation::Normal,
                ),
            ),
            layout::OrthogonalFallbackOf::new(
                layout::Size::NONE,
                layout::NearestScrollportOf::Absent,
                layout::Size::splat(layout::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
        layout::FlowAxes::new(layout::WritingMode::HorizontalTb, layout::Direction::Ltr),
        layout::Size::new(Some(width), None),
    )
}

#[test]
fn completed_empty_native_inline_family_is_rejected_even_without_text_fragments_or_pieces() {
    // Same source and request for both real computations. The sole added input
    // is explicit complete Native family demand on an empty Block formatter.
    // This follows the owning native-family empty-completion contract; no
    // output packet or family is constructed by the test.
    let mut golden = golden();
    golden
        .root
        .style
        .attrs
        .insert("display".into(), "block".into());
    golden
        .root
        .style
        .attrs
        .insert("width".into(), "100px".into());
    golden.root.style.attrs.remove("height");
    let source = source_census(
        &golden.root,
        &golden.expectations,
        None,
        FlexTextCensus::None,
    )
    .unwrap();
    let tree = TestTree::from_golden(&golden.root, None).unwrap();
    assert!(tree.paragraphs.is_empty());
    let request = output_census_native_request();

    let ordinary = layout::compute_layout(&tree, 0, request).unwrap();
    assert!(ordinary.placed_text().is_empty());
    assert!(ordinary.unrounded_inline_fragments().is_empty());
    assert!(ordinary.final_inline_fragments().is_empty());
    assert_eq!(ordinary.native_inline_box_families().count(), 0);
    assert_eq!(ordinary.native_inline_box_pieces().count(), 0);
    assert!(snapshot_batch(&tree, &ordinary, &source, false).is_ok());

    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        Vec::new(),
        layout::WrappingInput {
            native_inline_boxes: vec![layout::NativeInlineBoxFamilyInput { formatter: 0 }],
            scopes: Vec::new(),
            gaps: Vec::new(),
            child_order: vec![layout::InlineChildOrderInput {
                parent: layout::GeneratedBoxRef::Node(0),
                children: Vec::new(),
                prefix: layout::ChildOrderCompleteness::Complete,
                suffix: layout::ChildOrderCompleteness::Complete,
                formatting_context: 0,
            }],
        },
        Vec::new(),
    )
    .unwrap();
    let demanded = layout::compute_layout_with_flow(&flow, 0, request).unwrap();
    assert!(demanded.placed_text().is_empty());
    assert!(demanded.unrounded_inline_fragments().is_empty());
    assert!(demanded.final_inline_fragments().is_empty());
    let mut families = demanded.native_inline_box_families();
    assert_eq!(families.next().unwrap().formatter(), 0);
    assert!(families.next().is_none());
    assert_eq!(
        demanded.native_inline_box_pieces().count(),
        0,
        "an empty completion has a family but no formatted piece"
    );
    assert_eq!(
        snapshot_batch(&tree, &demanded, &source, false).unwrap_err(),
        QualificationError::UnsupportedObservation {
            owner: "/".into(),
            category: "Text/fragment/native-inline output census".into(),
        }
    );
}

#[test]
fn completed_prepared_text_and_both_fragment_channels_are_rejected_by_box_snapshot() {
    // Author one actual X occurrence and its font/style inputs. Preparation and
    // placement use the real pinned font and production Text/Layout providers;
    // there are no supplied glyph, width, baseline or rectangle observations.
    let style = json!({"fontSize":10.0,"lineHeight":10.5,"whitespace":"preserve",
        "wrap":"word","letterSpacing":0.0,"wordSpacing":0.0,
        "color":[0.0,0.0,0.0,1.0],"locale":null});
    let capture = json!({
        "font":{"id":pinned_font::ID,"sha256":pinned_font::SHA256,"faceIndex":0,
            "family":"SurgeistParityAhem","style":"normal","weight":400,
            "stretch":"normal","features":"normal","variations":"normal",
            "synthesis":"none","loaded":true},
        "direction":"ltr","writingMode":"horizontal-tb","baseStyle":style,
        "origins":[{"child":0,"units":[88],"start":0,"end":1,"style":style}],
        "connections":[],"atomics":[],"controls":[],
        "order":[{"kind":"origin","index":0,"affinity":"before"}],
    })
    .to_string()
    .replace('&', "&amp;")
    .replace('"', "&quot;")
    .replace('<', "&lt;");
    let golden = Golden::parse(&format!(
        r#"<test name="authored-output-census" measurement-version="2" use-rounding="true">
<viewport width="100" height="max-content"/>
<input><div display="block" layout-ready-inline-root="true"><prepared-paragraph capture="{capture}"/><text layout-input="prepared-text" occurrence="0"/></div></input>
<expectations><node><node/></node></expectations></test>"#
    ))
    .unwrap();
    let tree = TestTree::from_golden(&golden.root, None).unwrap();
    let batch = compute_actual_batch(&tree, &golden.name, output_census_native_request()).unwrap();
    assert!(!batch.placed_text().is_empty());
    assert!(!batch.unrounded_inline_fragments().is_empty());
    assert!(!batch.final_inline_fragments().is_empty());
    assert_eq!(batch.native_inline_box_families().count(), 0);
    assert_eq!(batch.native_inline_box_pieces().count(), 0);
    // This is the authored root plus its one original Text child. It is a direct
    // snapshot guard stimulus, never admission to the no-text source profile.
    let source = vec![
        SourceNode {
            path: "/".to_owned(),
            role: SourceRole::OrdinaryBox,
            children: vec!["/0".to_owned()],
        },
        SourceNode {
            path: "/0".to_owned(),
            role: SourceRole::OrdinaryBox,
            children: Vec::new(),
        },
    ];
    assert_eq!(
        snapshot_batch(&tree, &batch, &source, false).unwrap_err(),
        QualificationError::UnsupportedObservation {
            owner: "/".into(),
            category: "Text/fragment/native-inline output census".into(),
        }
    );
    assert!(matches!(
        source_census(
            &golden.root,
            &golden.expectations,
            None,
            FlexTextCensus::None
        ),
        Err(QualificationError::UnsupportedObservation { .. })
    ));
}

#[test]
fn canonical_spec_accepts_explicit_focused_and_published_capture_lineages() {
    let mut value = declaration();
    value.as_object_mut().unwrap().remove("identity");
    let focused = identity();
    let mut published = focused.clone();
    published["generator"] = json!("source-imported-durable-edition");
    published["source_imports_sha256"] = json!("b".repeat(64));
    published["inputs"]["origin/source-import.json"] = json!("c".repeat(64));
    value["identities"] = json!({"focused": focused, "published": published});
    value["representation"] = json!({"published_edition": "canonical-box-differences"});
    let result = ExactDifferenceSpec::parse(&value.to_string());
    assert!(
        result.is_ok(),
        "one semantic oracle must admit separately authored, explicit capture lineages: {:?}",
        result.err()
    );
}

// Independently authored lineage labels and hashes; the Native source, oracle
// and browser pair remain the fixed-box arithmetic used above.
fn published_identity() -> Value {
    let mut value = identity();
    value["generator"] = json!("source-imported-durable-edition");
    value["capture_sha256"] = json!("b".repeat(64));
    value["profile_sha256"] = json!("c".repeat(64));
    value["source_imports_sha256"] = json!("d".repeat(64));
    value["inputs"]["origin/source-import.json"] = json!("e".repeat(64));
    value
}
fn two_lineage_declaration() -> Value {
    let mut value = declaration();
    value["identities"]["published"] = published_identity();
    value["representation"]["published_edition"] = json!("canonical-box-differences");
    value["representation"]["current"] = json!({
        "case_id":"fixed-published","output":"xml/fixed-published.xml","frame":"root-border-local","domain":"layout",
        "selectors":[{"path":"/","field":"x"},{"path":"/","field":"y"},{"path":"/","field":"width"},{"path":"/","field":"height"}],
        "nodes":["/"],"unrounded_packets":["/"],"final_packets":["/"]
    });
    value
}

#[test]
fn explicit_authority_dispatches_one_lineage_before_comparing_identical_geometry() {
    let value = two_lineage_declaration();
    let spec = spec(&value);
    let focused = CaseIdentity::from_verified_projection(identity()).unwrap();
    let published = CaseIdentity::from_verified_projection(published_identity()).unwrap();
    let mut golden = element_golden(20);
    golden.element_scroll_sizes = None;
    for authority in [
        CaptureAuthority::Focused(&focused),
        CaptureAuthority::Published {
            edition: PublishedEdition::CanonicalBoxes,
            identity: &published,
        },
    ] {
        let resolved = spec.resolve_against(authority, &golden).unwrap();
        let completed = complete_captured_comparison(&golden, &resolved).unwrap();
        assert_eq!(
            resolved.qualify(&completed).unwrap().raw,
            RawVerdict::Different { fields: 1 }
        );
    }
    // Trying another identity until one matches would incorrectly accept each
    // of these calls. The exact same source/oracle/geometry has no such power.
    for authority in [
        CaptureAuthority::Focused(&published),
        CaptureAuthority::Published {
            edition: PublishedEdition::CanonicalBoxes,
            identity: &focused,
        },
    ] {
        assert!(matches!(
            spec.resolve_against(authority, &golden),
            Err(QualificationError::IdentityChanged { component }) if component == "capture_sha256"
        ));
    }
}

#[test]
fn missing_selected_lineage_refuses_even_when_the_other_identity_matches() {
    let focused_only = spec(&declaration());
    let focused = CaseIdentity::from_verified_projection(identity()).unwrap();
    assert!(matches!(
        focused_only.resolve_against(
            CaptureAuthority::Published {
                edition: PublishedEdition::CanonicalBoxes,
                identity: &focused,
            },
            &golden(),
        ),
        Err(QualificationError::IdentityChanged { component }) if component == "capture_lineage.published"
    ));

    let mut value = two_lineage_declaration();
    value["identities"]
        .as_object_mut()
        .unwrap()
        .remove("focused");
    let published_only = spec(&value);
    let published = CaseIdentity::from_verified_projection(published_identity()).unwrap();
    assert!(
        published_only
            .resolve_against(
                CaptureAuthority::Published {
                    edition: PublishedEdition::CanonicalBoxes,
                    identity: &published,
                },
                &{
                    let mut g = element_golden(20);
                    g.element_scroll_sizes = None;
                    g
                },
            )
            .is_ok()
    );
    assert!(matches!(
        published_only.resolve_against(CaptureAuthority::Focused(&published), &golden()),
        Err(QualificationError::IdentityChanged { component }) if component == "capture_lineage.focused"
    ));
}

#[test]
fn capture_lineage_map_rejects_empty_unknown_duplicate_and_legacy_fields() {
    let mut value = declaration();
    value["identities"] = json!({});
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut value = declaration();
    value["identities"]["unregistered"] = identity();
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let raw = declaration().to_string();
    let duplicate = raw.replacen(
        "\"focused\":",
        &format!("\"focused\":{},\"focused\":", identity()),
        1,
    );
    assert!(matches!(
        ExactDifferenceSpec::parse(&duplicate),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut value = declaration();
    value.as_object_mut().unwrap().remove("identities");
    value["identity"] = identity();
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut value = declaration();
    value["identity"] = identity();
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
}

#[test]
fn published_identity_requires_explicit_fixed_edition_and_focused_null_is_required() {
    let mut focused = declaration();
    assert!(ExactDifferenceSpec::parse(&focused.to_string()).is_ok());
    focused["representation"]
        .as_object_mut()
        .unwrap()
        .remove("published_edition");
    assert!(matches!(
        ExactDifferenceSpec::parse(&focused.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut focused = declaration();
    focused.as_object_mut().unwrap().remove("representation");
    assert!(matches!(
        ExactDifferenceSpec::parse(&focused.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    for edition in [Value::Null, json!("other-edition")] {
        let mut value = two_lineage_declaration();
        value["representation"]["published_edition"] = edition;
        assert!(matches!(
            ExactDifferenceSpec::parse(&value.to_string()),
            Err(QualificationError::MalformedDeclaration { .. })
        ));
    }
    let mut value = two_lineage_declaration();
    value["representation"]
        .as_object_mut()
        .unwrap()
        .remove("published_edition");
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut value = declaration();
    value["representation"]["unregistered"] = json!(true);
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
}

#[test]
fn capture_lineages_cannot_disagree_on_shared_semantic_basis() {
    for (field, replacement) in [
        ("source", json!("html/other.html")),
        ("source_sha256", json!("b".repeat(64))),
        ("variant", json!("content_box_rtl")),
        ("record_sha256", json!("b".repeat(64))),
        ("historical_sha256", json!("b".repeat(64))),
        ("font_demand", json!("text")),
        ("policy_source_sha256", json!("b".repeat(64))),
    ] {
        let mut value = two_lineage_declaration();
        value["identities"]["published"][field] = replacement;
        assert!(
            matches!(
                ExactDifferenceSpec::parse(&value.to_string()),
                Err(QualificationError::MalformedDeclaration { .. })
            ),
            "{field}"
        );
    }
}

#[test]
fn shared_resources_and_present_dedicated_input_hashes_must_agree() {
    let mut value = two_lineage_declaration();
    value["identities"]["published"]["inputs"]["base.css"] = json!("b".repeat(64));
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    let mut value = two_lineage_declaration();
    value["identities"]["published"]["linked_resources"]["base.css"] = json!("b".repeat(64));
    assert!(matches!(
        ExactDifferenceSpec::parse(&value.to_string()),
        Err(QualificationError::MalformedDeclaration { .. })
    ));
    for path in ["html/empty-fixed.html", "docs/policy.md"] {
        let mut value = declaration();
        value["identities"]["focused"]["inputs"][path] = json!("b".repeat(64));
        assert!(
            matches!(
                ExactDifferenceSpec::parse(&value.to_string()),
                Err(QualificationError::MalformedDeclaration { .. })
            ),
            "{path}"
        );
    }
}

#[test]
fn selected_lineage_requires_exact_input_and_linked_resource_maps() {
    let mut value = two_lineage_declaration();
    value["identities"]["focused"]["inputs"]["shared.css"] = json!("a".repeat(64));
    let spec = spec(&value);
    // The actual identity lacks an authored entry. It cannot match a subset.
    let missing = CaseIdentity::from_verified_projection(identity()).unwrap();
    assert!(matches!(
        spec.resolve_against(CaptureAuthority::Focused(&missing), &golden()),
        Err(QualificationError::IdentityChanged { component }) if component == "inputs"
    ));
    let mut actual = identity();
    actual["inputs"]["shared.css"] = json!("a".repeat(64));
    actual["inputs"]["unexpected.css"] = json!("a".repeat(64));
    let extra = CaseIdentity::from_verified_projection(actual).unwrap();
    assert!(matches!(
        spec.resolve_against(CaptureAuthority::Focused(&extra), &golden()),
        Err(QualificationError::IdentityChanged { component }) if component == "inputs"
    ));
    let mut actual = identity();
    actual["inputs"]["shared.css"] = json!("a".repeat(64));
    actual["linked_resources"]["unexpected.css"] = json!("a".repeat(64));
    let extra = CaseIdentity::from_verified_projection(actual).unwrap();
    assert!(matches!(
        spec.resolve_against(CaptureAuthority::Focused(&extra), &golden()),
        Err(QualificationError::IdentityChanged { component }) if component == "linked_resources"
    ));
}

#[test]
fn every_declared_lineage_validates_its_own_paths_and_hashes() {
    for lineage in ["focused", "published"] {
        for (field, invalid) in [
            ("source", json!("../outside.html")),
            ("source_sha256", json!("z".repeat(64))),
            ("inputs", json!({})),
        ] {
            let mut value = two_lineage_declaration();
            value["identities"][lineage][field] = invalid;
            assert!(
                matches!(
                    ExactDifferenceSpec::parse(&value.to_string()),
                    Err(QualificationError::MalformedDeclaration { .. })
                ),
                "{lineage}:{field}"
            );
        }
    }
}

fn fixed_current_selection() -> super::super::capture_authority::CaseSelection {
    super::super::capture_authority::CaseSelection {
        name: "fixed-published".into(),
        source: "html/empty-fixed.html".into(),
        source_sha256: "a".repeat(64),
        variant: "content_box_ltr".into(),
        output: "xml/fixed-published.xml".into(),
        record_sha256: "a".repeat(64),
        historical_sha256: None,
    }
}
#[test]
fn published_parse_can_remain_unresolved_but_admission_cannot_execute_it() {
    let mut value = two_lineage_declaration();
    value["representation"]
        .as_object_mut()
        .unwrap()
        .remove("current");
    let parsed = spec(&value);
    assert!(matches!(
        parsed.require_published_context(PublishedEdition::CanonicalBoxes),
        Err(QualificationError::IncompleteRepresentation { .. })
    ));
    let identity = CaseIdentity::from_verified_projection(published_identity()).unwrap();
    let mut golden = element_golden(20);
    golden.element_scroll_sizes = None;
    assert!(matches!(
        parsed.resolve_against(
            CaptureAuthority::Published {
                edition: PublishedEdition::CanonicalBoxes,
                identity: &identity
            },
            &golden
        ),
        Err(QualificationError::IncompleteRepresentation { .. })
    ));
}
#[test]
fn current_prebinding_refuses_case_output_selector_and_either_packet_census_drift() {
    let value = two_lineage_declaration();
    assert!(
        spec(&value)
            .prebind_current(PublishedEdition::CanonicalBoxes, &fixed_current_selection())
            .is_ok()
    );
    for (pointer, replacement) in [
        ("/representation/current/case_id", json!("other")),
        ("/representation/current/output", json!("xml/other.xml")),
        ("/representation/current/selectors", json!([])),
        ("/representation/current/nodes", json!([])),
        ("/representation/current/unrounded_packets", json!([])),
        ("/representation/current/final_packets", json!([])),
    ] {
        let mut changed = value.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            matches!(
                spec(&changed)
                    .prebind_current(PublishedEdition::CanonicalBoxes, &fixed_current_selection()),
                Err(QualificationError::IncompleteRepresentation { .. })
            ),
            "{pointer}"
        );
    }
}
#[test]
fn published_layout_ranges_are_independent_of_dom_element_sizes() {
    let mut value = two_lineage_declaration();
    let current = &mut value["representation"]["current"];
    let selectors = current["selectors"].as_array_mut().unwrap();
    selectors.extend([
        json!({"path":"/","field":"layout-scroll-width"}),
        json!({"path":"/","field":"layout-scroll-height"}),
    ]);
    let identity = CaseIdentity::from_verified_projection(published_identity()).unwrap();
    let authority = || CaptureAuthority::Published {
        edition: PublishedEdition::CanonicalBoxes,
        identity: &identity,
    };
    let mut golden = element_golden(20);
    golden.element_scroll_sizes = None;
    golden.expectations.scroll_size = Some(Size {
        width: 0.0,
        height: 0.0,
    });

    let parsed = spec(&value);
    let resolved = parsed.resolve_against(authority(), &golden).unwrap();
    let complete = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(complete.observations.len(), 6);
    assert_eq!(
        resolved.qualify(&complete).unwrap().raw,
        RawVerdict::Different { fields: 1 }
    );

    let mut incomplete = value;
    incomplete["representation"]["current"]["selectors"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(
        spec(&incomplete)
            .resolve_against(authority(), &golden)
            .is_err()
    );
    golden.expectations.scroll_size = None;
    assert!(parsed.resolve_against(authority(), &golden).is_err());
}

#[test]
fn measured_grid_publication_compares_both_complete_domains_for_the_certified_leaf() {
    const XML: &str = r#"<test name="fixed-published" measurement-version="4" observation-frame="root-border-local" use-rounding="false">
<viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="grid" hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div source-tag="div" grid-alignment-profile="{&quot;profile&quot;:1,&quot;owners&quot;:[]}" display="grid" position="absolute" direction="ltr" width="80px" height="40px" grid-template-columns="80px" grid-template-rows="40px">
<text source-tag="div" display="flex" direction="ltr">HHHH&#x200b;HHHH</text></div></input>
<expectations><node x="0" y="0" width="81" height="40"><node x="0" y="0" width="80" height="40"/></node></expectations>
<element-scroll-sizes profile="horizontal-zero-edge-divs">
<element node-path="0" client-width="80" client-height="40" scroll-width="80" scroll-height="40"/>
<element node-path="0.0" client-width="80" client-height="40" scroll-width="81" scroll-height="40"/>
</element-scroll-sizes>
<measured-leaf profile="ahem-10px-zwsp" node-path="0.0" font-resource-sha256="e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c" first-advance="40" second-advance="40" full-advance="80"/>
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="81" height="40"/><world-rounded-layout node-path="0.0" x="0" y="0" width="80" height="40"/></diagnostics></test>"#;

    let mut value = two_lineage_declaration();
    value["profile"] = json!("canonical-measured-grid");
    value["identities"]
        .as_object_mut()
        .unwrap()
        .remove("focused");
    value["identities"]["published"]["font_demand"] = json!("ahem-10px-zwsp");
    value["representation"]["published_edition"] = json!("measured-grid-differences");
    value["representation"]["current"]["domain"] = json!("layout-and-element-sizes");
    let fields = [
        "x",
        "y",
        "width",
        "height",
        "client-width",
        "client-height",
        "scroll-width",
        "scroll-height",
    ];
    let selectors: Vec<_> = ["/", "/0"]
        .into_iter()
        .flat_map(|path| fields.map(|field| json!({"path":path,"field":field})))
        .collect();
    value["representation"]["current"]["selectors"] = json!(selectors);
    for key in ["nodes", "unrounded_packets", "final_packets"] {
        value["representation"]["current"][key] = json!(["/", "/0"]);
    }
    // Topology and both packet channels are authored. This test verifies
    // admission and observation census before comparing the Native oracle.
    let mut parent = oracle();
    parent["children"] = json!(["/0"]);
    let mut child = oracle();
    child["path"] = json!("/0");
    value["native"]["nodes"] = json!([parent, child]);

    let parsed = ExactDifferenceSpec::parse(&value.to_string())
        .unwrap_or_else(|error| panic!("measured Grid comparison declaration must parse: {error}"));
    let identity =
        CaseIdentity::from_verified_projection(value["identities"]["published"].clone()).unwrap();
    let edition =
        serde_json::from_value(value["representation"]["published_edition"].clone()).unwrap();
    let authority = || CaptureAuthority::Published {
        edition,
        identity: &identity,
    };
    let golden = Golden::parse(XML).unwrap();
    let resolved = parsed.resolve_against(authority(), &golden).unwrap();
    let complete = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(complete.observations.len(), 16);
    let differences: Vec<_> = complete.raw_differences().map(ComparedField::key).collect();
    assert_eq!(
        differences,
        ["layout:/:Width", "element-size:/0:ScrollWidth"]
    );

    let mut missing_executed_field = golden.clone();
    missing_executed_field.expectations.children[0].height = None;
    assert!(matches!(
        complete_captured_comparison(&missing_executed_field, &resolved),
        Err(QualificationError::IncompleteRepresentation { key })
            if key == "complete executed current comparison selectors"
    ));

    let mut without_witness = golden.clone();
    without_witness.measured_leaf = None;
    assert!(
        parsed
            .resolve_against(authority(), &without_witness)
            .is_err()
    );
    let mut changed = value.clone();
    changed["representation"]["current"]["selectors"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(
        spec(&changed)
            .resolve_against(authority(), &golden)
            .is_err()
    );

    let hidden_xml = XML
        .replace(
            "<text source-tag=\"div\" display=\"flex\" direction=\"ltr\">",
            "<text source-tag=\"div\" display=\"flex\" direction=\"ltr\" overflow-x=\"hidden\" overflow-y=\"hidden\">",
        )
        .replace(
            r#"<node x="0" y="0" width="80" height="40"/>"#,
            r#"<node x="0" y="0" width="80" height="40" scroll_width="1" scroll_height="0"/>"#,
        );
    let hidden = Golden::parse(&hidden_xml).unwrap();
    assert!(parsed.resolve_against(authority(), &hidden).is_err());

    let selectors = value["representation"]["current"]["selectors"]
        .as_array_mut()
        .unwrap();
    for field in ["layout-scroll-width", "layout-scroll-height"] {
        selectors.push(json!({"path":"/0","field":field}));
    }
    let with_ranges = ExactDifferenceSpec::parse(&value.to_string())
        .expect("measured Grid declares Layout scroll-range observations");
    let resolved = with_ranges.resolve_against(authority(), &hidden).unwrap();
    let complete = complete_captured_comparison(&hidden, &resolved).unwrap();
    assert_eq!(complete.observations.len(), 18);
    let differences: Vec<_> = complete.raw_differences().map(ComparedField::key).collect();
    assert_eq!(
        differences,
        [
            "layout:/:Width",
            "layout:/0:ScrollWidth",
            "element-size:/0:ScrollWidth",
        ]
    );
    assert!(with_ranges.resolve_against(authority(), &golden).is_err());

    value["representation"]["current"]["selectors"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(spec(&value).resolve_against(authority(), &hidden).is_err());
}

#[test]
fn published_resolution_refuses_a_different_frame_or_requested_census_before_native_execution() {
    let value = two_lineage_declaration();
    let parsed = spec(&value);
    let identity = CaseIdentity::from_verified_projection(published_identity()).unwrap();
    assert!(matches!(
        parsed.resolve_against(
            CaptureAuthority::Published {
                edition: PublishedEdition::CanonicalBoxes,
                identity: &identity
            },
            &golden()
        ),
        Err(QualificationError::IncompleteRepresentation { .. })
    ));
    let mut g = element_golden(20);
    g.element_scroll_sizes = None;
    g.expectations.height = None;
    assert!(matches!(
        parsed.resolve_against(
            CaptureAuthority::Published {
                edition: PublishedEdition::CanonicalBoxes,
                identity: &identity
            },
            &g
        ),
        Err(QualificationError::IncompleteRepresentation { .. })
    ));
}

#[test]
fn completed_batch_keeps_raw_differences_when_native_oracle_qualification_fails() {
    let mut value = declaration();
    value["native"]["nodes"][0]["unrounded"]["facts"]["size"]["width"] = json!(19);
    let parsed = spec(&value);
    let identity = CaseIdentity::from_verified_projection(identity()).unwrap();
    let golden = golden();
    let resolved = parsed
        .resolve_against(CaptureAuthority::Focused(&identity), &golden)
        .unwrap();
    let completed = complete_captured_comparison(&golden, &resolved).unwrap();
    assert_eq!(
        completed
            .raw_differences()
            .map(ComparedField::key)
            .collect::<Vec<_>>(),
        ["layout:/:Width"]
    );
    assert!(matches!(
        resolved.qualify(&completed),
        Err(QualificationError::NativeOracleChanged { .. })
    ));
    assert_eq!(completed.raw_differences().count(), 1);
}

#[test]
fn sideways_flex_composition_refuses_an_unexplained_raw_pair() {
    const SOURCE: &str = "html/flex/fri07_flex_composition.html";
    const SOURCE_BYTES: &[u8] = include_bytes!("html/flex/fri07_flex_composition.html");
    const REPORT: &str =
        include_str!("captured/flex-composition-differences/xml/generation-reports/all.json");
    const XML: &str = include_str!(
        "captured/flex-composition-differences/xml/flex/fri07_flex_composition__border_box_rtl.xml"
    );
    const OUTPUT: &str = "xml/flex/fri07_flex_composition__border_box_rtl.xml";

    let owner = Path::new(env!("CARGO_MANIFEST_DIR"));
    let inputs = flex_composition_qualification::Inputs::load(owner).unwrap();
    let golden =
        Golden::parse_with_flex_composition(XML, REPORT, SOURCE, SOURCE_BYTES, OUTPUT).unwrap();

    for (browser_y, difference_kind) in [(0.0, "unchanged"), (1.0, "changed"), (15.0, "missing")] {
        let mut golden = golden.clone();
        golden.expectations.children[3].children[0].y = Some(browser_y);
        let mut raw = Vec::new();
        let result =
            flex_composition_qualification::qualify(&golden, "border_box_rtl", &inputs, &mut raw);
        let receipt: Value = serde_json::from_slice(&raw).unwrap();
        let selector = "layout:/3/0:Y".to_owned();
        match difference_kind {
            "unchanged" => {
                assert_eq!(result.unwrap(), (9, 1));
                assert_eq!(
                    receipt["qualified"][0]["fields"].as_array().unwrap().len(),
                    9
                );
                assert_eq!(
                    receipt["qualified"][1]["fields"].as_array().unwrap().len(),
                    1
                );
            }
            "changed" => assert!(matches!(
                result,
                Err(QualificationError::DifferenceSetChanged { changed, .. }) if changed == [selector]
            )),
            "missing" => assert!(matches!(
                result,
                Err(QualificationError::DifferenceSetChanged { missing, .. }) if missing == [selector]
            )),
            _ => unreachable!(),
        }
        assert_eq!(receipt["observations"].as_array().unwrap().len(), 34);
        assert_eq!(receipt["native_packets"].as_array().unwrap().len(), 7);
        if difference_kind != "unchanged" {
            assert!(receipt["qualified"].is_null());
        }
    }

    let mut golden = golden;
    golden.expectations.children[3].children[0].width = Some(121.0);
    let mut raw = Vec::new();
    assert!(matches!(
        flex_composition_qualification::qualify(&golden, "border_box_rtl", &inputs, &mut raw),
        Err(QualificationError::DifferenceSetChanged { extra, .. }) if extra == ["layout:/3/0:Width"]
    ));
}

#[test]
fn sideways_fixed_capture_retains_the_gutter_control_environment() {
    const REPORT: &str =
        include_str!("captured/flex-composition-qualification/xml/generation-reports/all.json");
    const XML: &str = include_str!(
        "captured/flex-composition-qualification/xml/flex/fri07_flex_composition__border_box_rtl.xml"
    );
    const SOURCE: &str = "html/flex/fri07_flex_composition.html";
    const SOURCE_BYTES: &[u8] = include_bytes!("html/flex/fri07_flex_composition.html");
    const OUTPUT: &str = "xml/flex/fri07_flex_composition__border_box_rtl.xml";
    let parse = |report: &str| {
        Golden::parse_with_published_flex_composition(XML, report, SOURCE, SOURCE_BYTES, OUTPUT)
    };
    parse(REPORT).unwrap();
    assert!(
        Golden::parse_with_flex_composition(XML, REPORT, SOURCE, SOURCE_BYTES, OUTPUT).is_err()
    );

    let report: serde_json::Value = serde_json::from_str(REPORT).unwrap();
    for pointer in [
        "/metadata/generator",
        "/metadata/engine_version",
        "/metadata/browser_executable",
        "/metadata/browser_executable_sha256",
        "/metadata/browser_provenance",
        "/metadata/inputs/browser-owner.toml",
        "/metadata/inputs/fonts~1Ahem.ttf",
        "/metadata/inputs/observation-request.json",
        "/metadata/inputs/scripts~1gentest~1test_base_style.css",
        "/metadata/inputs/scripts~1gentest~1test_helper.js",
    ] {
        let mut changed = report.clone();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!("changed");
        let error = parse(&changed.to_string()).unwrap_err();
        assert!(
            error.to_string().contains("capture environment changed"),
            "{pointer}: {error}"
        );
    }
}

#[test]
fn matching_geometry_exposes_complete_raw_differences_and_requires_empty_comparison() {
    let historical =
        Golden::parse(include_str!("xml/block/block_basic__border_box_ltr.xml")).unwrap();
    assert!(compare_matching_geometry(&historical).is_err());
    let matching = Golden::parse(&FIXED_XML.replace("width=\"21\"", "width=\"20\"")).unwrap();
    let (raw, qualified) = compare_matching_geometry(&matching).unwrap();
    assert!(raw.is_empty());
    assert_eq!(
        qualified.unwrap(),
        QualifiedComparison {
            raw: RawVerdict::Matches,
            qualified: QualifiedVerdict::MatchingControl
        }
    );
    let (raw, qualified) = compare_matching_geometry(&golden()).unwrap();
    assert_eq!(raw.len(), 1);
    assert!(raw[0].contains("Width"));
    assert!(qualified.is_err());
    for field in ["x", "y", "width", "height"] {
        let mut missing = matching.clone();
        match field {
            "x" => missing.expectations.x = None,
            "y" => missing.expectations.y = None,
            "width" => missing.expectations.width = None,
            _ => missing.expectations.height = None,
        }
        assert!(compare_matching_geometry(&missing).is_err(), "{field}");
    }
    let mut nonfinite = matching.clone();
    nonfinite.expectations.width = Some(Scalar::NAN);
    assert!(compare_matching_geometry(&nonfinite).is_err());
    let mut late = matching;
    late.root.children.push(late.root.clone());
    assert!(compare_matching_geometry(&late).is_err());
}

#[test]
fn fri06_matching_requires_complete_box_and_br_observations() {
    use super::capture_authority::{FRI06_SUBGRID_SOURCES, MatchingCategory};

    let source = FRI06_SUBGRID_SOURCES[2];
    let compare =
        |golden: &Golden| compare_matching_capture(MatchingCategory::Fri06Subgrid, source, golden);
    let capture = Golden::parse(include_str!(
        "captured/fri06-subgrid-representatives/xml/subgrid/subgrid_baseline_vertical_nested_inner_row1_first__border_box_ltr.xml"
    ))
    .unwrap();
    let (raw, qualified) = compare(&capture).unwrap();
    assert!(raw.is_empty() && qualified.is_ok());

    let mut incomplete = capture.clone();
    incomplete.expectations.x = None;
    assert!(compare(&incomplete).is_err());

    fn first_br_observation(
        expectation: &mut Expectation,
    ) -> Option<&mut Option<BrowserControlExpectation>> {
        if expectation.browser_control.is_some() {
            return Some(&mut expectation.browser_control);
        }
        expectation
            .children
            .iter_mut()
            .find_map(first_br_observation)
    }

    let mut incomplete = capture.clone();
    first_br_observation(&mut incomplete.expectations)
        .unwrap()
        .take();
    assert!(compare(&incomplete).is_err());

    let mut incomplete = capture;
    first_br_observation(&mut incomplete.expectations)
        .unwrap()
        .as_mut()
        .unwrap()
        .next_line = BrowserNeighborLine::Unobserved;
    assert!(compare(&incomplete).is_err());
}

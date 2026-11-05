use super::super::{
    Golden, NumericComparisonField, assert_surgeist_matches, collect_surgeist_numeric_mismatches,
};
use super::*;

fn table(rows: &[(&str, [u32; 4])]) -> String {
    let mut out = String::from("<element-scroll-sizes profile=\"horizontal-zero-edge-divs\">");
    for (path, [cw, ch, sw, sh]) in rows {
        out.push_str(&format!("<element node-path=\"{path}\" client-width=\"{cw}\" client-height=\"{ch}\" scroll-width=\"{sw}\" scroll-height=\"{sh}\"/>"));
    }
    out.push_str("</element-scroll-sizes>");
    out
}

fn xml(attrs: &str, child: Option<&str>, observation: &str) -> String {
    let child = child.unwrap_or("");
    let child_expectation = if child.is_empty() { "" } else { "<node/>" };
    let child_diagnostic = if child.is_empty() {
        ""
    } else {
        "<world-rounded-layout node-path=\"0.0\" x=\"0\" y=\"0\" width=\"0\" height=\"0\"/>"
    };
    // Diagnostic rectangles are schema witnesses only and never projection operands.
    format!(
        r#"<test name="element-scroll" measurement-version="4" observation-frame="root-border-local" use-rounding="true">
<viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px" containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="block" hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
<input><div source-tag="div" display="block" position="absolute" grid-alignment-profile="{{&quot;profile&quot;:1,&quot;owners&quot;:[]}}" {attrs}>{child}</div></input>
<expectations><node>{child_expectation}</node></expectations>{observation}
<diagnostics><world-rounded-layout node-path="0" x="0" y="0" width="0" height="0"/>{child_diagnostic}</diagnostics></test>"#
    )
}
fn golden(attrs: &str, child: Option<&str>, rows: &[(&str, [u32; 4])]) -> Golden {
    Golden::parse(&xml(attrs, child, &table(rows)))
        .expect("closed actual DIV source and complete sideband parse")
}
fn integer_rows(golden: &Golden) -> Vec<(String, ElementScrollSizeField, u32, u32)> {
    collect_surgeist_numeric_mismatches(golden)
        .unwrap()
        .into_iter()
        .map(|mismatch| {
            let NumericMismatch::ElementScrollSize(row) = mismatch else {
                panic!("expected integer-only requested differences")
            };
            (
                row.path().to_owned(),
                row.field(),
                row.browser_value(),
                row.native_value(),
            )
        })
        .collect()
}

#[test]
fn measured_leaf_xml_preserves_valid_evidence_and_refuses_mismatched_witnesses() {
    const WITNESS: &str = r#"<measured-leaf profile="ahem-10px-zwsp" node-path="0.0" font-resource-sha256="e450dd6e1bc6fb10ebc6edecef1182dcd8cad6e12e54c363ad4ba26f6b5db46c" first-advance="40" second-advance="40" full-advance="80"/>"#;
    let sizes = table(&[("0", [80, 40, 80, 40]), ("0.0", [80, 40, 80, 40])]);
    let base = xml(
        "",
        Some(r#"<text source-tag="div">HHHH&#x200b;HHHH</text>"#),
        &sizes,
    )
    .replace(r#"display="block""#, r#"display="grid""#);
    let measured = base.replace("</expectations>", &format!("</expectations>{WITNESS}"));
    for (field, broken) in [
        ("resource", measured.replace("e450dd6e", "00000000")),
        (
            "path",
            measured.replace(r#"node-path="0.0" font"#, r#"node-path="0.1" font"#),
        ),
        ("source", measured.replace("HHHH&#x200b;HHHH", "HHHH HHHH")),
        (
            "advance",
            measured.replace(r#"full-advance="80""#, r#"full-advance="81""#),
        ),
        (
            "attribute",
            measured.replace("<measured-leaf ", "<measured-leaf unknown=\"yes\" "),
        ),
        (
            "duplicate",
            measured.replace(WITNESS, &format!("{WITNESS}{WITNESS}")),
        ),
        ("missing DOM sizes", measured.replace(&sizes, "")),
        (
            "normalized root role",
            measured
                .replace("<input><div", "<input><text")
                .replace("</div></input>", "</text></input>"),
        ),
    ] {
        let result = Golden::parse(&broken);
        assert!(
            result.is_err_and(|error| error.to_string().contains("measured-leaf")),
            "{field} mismatch must refuse the measured leaf witness",
        );
    }
    let admitted = Golden::parse(&measured).unwrap();
    assert_eq!(admitted.measured_leaf.as_ref().unwrap().child_index, 0);
    assert_ne!(admitted, Golden::parse(&base).unwrap());
}

#[test]
fn absent_observation_preserves_existing_capture_and_strict_first_error() {
    let mut g = Golden::parse(&xml("width=\"20px\" height=\"30px\"", None, "")).unwrap();
    assert!(!g.has_element_scroll_sizes());
    assert_surgeist_matches(&g).unwrap();
    g.expectations.width = Some(21.0);
    g.expectations.height = Some(31.0);
    assert_eq!(
        assert_surgeist_matches(&g).unwrap_err().to_string(),
        "element-scroll: width mismatch, expected 21, got 20"
    );
}

#[test]
fn semantic_table_is_optional_but_closed_and_covers_source_preorder() {
    let child = "<div source-tag=\"div\" display=\"block\" width=\"5px\" height=\"10px\"/>";
    let rows = [("0", [20, 30, 20, 30]), ("0.0", [5, 10, 5, 10])];
    let raw = xml("width=\"20px\" height=\"30px\"", Some(child), &table(&rows));
    let g = Golden::parse(&raw).unwrap();
    assert!(g.has_element_scroll_sizes());
    assert_surgeist_matches(&g).unwrap();
    for changed in [
        raw.replace("horizontal-zero-edge-divs", "other"),
        raw.replace(
            "profile=\"horizontal-zero-edge-divs\"",
            "profile=\"horizontal-zero-edge-divs\" unknown=\"0\"",
        ),
        raw.replace("client-width=\"5\"", "unknown=\"5\""),
        raw.replace("node-path=\"0.0\" client", "node-path=\"0\" client"),
        raw.replace("node-path=\"0.0\" client", "node-path=\"0.00\" client"),
        raw.replace("node-path=\"0.0\" client", "node-path=\"0.1\" client"),
        raw.replace(&table(&rows), &table(&rows[..1])),
        raw.replace(&table(&rows), &table(&[rows[1], rows[0]])),
        raw.replace(
            "</element-scroll-sizes>",
            "<unexpected/></element-scroll-sizes>",
        ),
        raw.replace("</element-scroll-sizes>", "text</element-scroll-sizes>"),
        raw.replace(
            "<element node-path=\"0\"",
            "<element unexpected=\"1\" node-path=\"0\"",
        ),
        raw.replace(&table(&rows), &format!("{}{}", table(&rows), table(&rows))),
        raw.replace(
            &table(&rows),
            &format!("<metadata>{}</metadata>", table(&rows)),
        ),
    ] {
        assert!(
            Golden::parse(&changed).is_err(),
            "malformed or incomplete sideband must refuse"
        );
    }
}

#[test]
fn cssom_values_are_exact_bounded_integers_and_coupled_pairs() {
    let base = xml(
        "width=\"20px\" height=\"30px\"",
        None,
        &table(&[("0", [20, 30, 20, 30])]),
    );
    for value in ["-1", "+1", "1.5", "NaN", "inf", "2147483648", "4294967295"] {
        let raw = base.replace("scroll-width=\"20\"", &format!("scroll-width=\"{value}\""));
        assert!(Golden::parse(&raw).is_err(), "invalid CSSOM long {value}");
    }
    assert!(Golden::parse(&base.replace("client-width=\"20\"", "client-width=\"21\"")).is_err());
    let large = base.replace("scroll-width=\"20\"", "scroll-width=\"2147483647\"");
    assert!(
        Golden::parse(&large).is_ok(),
        "largest signed long is retained losslessly"
    );
    let zero = golden("width=\"0px\" height=\"0px\"", None, &[("0", [0, 0, 0, 0])]);
    assert_surgeist_matches(&zero).unwrap();
}

#[test]
fn actual_source_role_and_absolute_profile_are_required() {
    let raw = xml(
        "width=\"20px\" height=\"30px\"",
        None,
        &table(&[("0", [20, 30, 20, 30])]),
    );
    for changed in [
        raw.replace("source-tag=\"div\"", "source-tag=\"span\""),
        raw.replace("source-tag=\"div\"", ""),
        raw.replace("measurement-version=\"4\"", "measurement-version=\"2\""),
        raw.replace(
            "<element-scroll-sizes",
            "<x:element-scroll-sizes xmlns:x=\"urn:foreign\"",
        )
        .replace("</element-scroll-sizes>", "</x:element-scroll-sizes>"),
    ] {
        assert!(Golden::parse(&changed).is_err());
    }
}

#[test]
fn unsupported_native_edges_overflow_and_gutter_policy_do_not_become_differences() {
    for extra in [
        "padding-left=\"1px\"",
        "border-left=\"1px\"",
        "writing-mode=\"vertical-rl\"",
        "overflow-x=\"auto\" overflow-y=\"auto\"",
        "overflow-x=\"scroll\" overflow-y=\"scroll\"",
        "scrollbar-gutter=\"stable\"",
    ] {
        let raw = xml(
            &format!("width=\"20px\" height=\"30px\" {extra}"),
            None,
            &table(&[("0", [20, 30, 20, 30])]),
        );
        let error = Golden::parse(&raw)
            .and_then(|g| collect_surgeist_numeric_mismatches(&g))
            .unwrap_err();
        assert!(
            error.to_string().contains("element-scroll-sizes"),
            "unsupported {extra} is fatal: {error}"
        );
    }
}

#[test]
fn inactive_nonzero_scrollbar_environment_does_not_invent_a_gutter() {
    let g = golden(
        "width=\"20px\" height=\"30px\" scrollbar-width=\"15\" overflow-x=\"hidden\" overflow-y=\"hidden\"",
        None,
        &[("0", [20, 30, 20, 30])],
    );
    assert_surgeist_matches(&g).unwrap();
}

#[test]
fn represented_integer_and_half_rounding_are_independent_of_f32_addition() {
    assert_eq!(
        projected("0", ElementScrollSizeField::ClientWidth, 16_777_215.0).unwrap(),
        16_777_215
    );
    assert_eq!(
        projected("0", ElementScrollSizeField::ClientWidth, 10.5).unwrap(),
        11
    );
    assert_eq!(
        projected("0", ElementScrollSizeField::ClientHeight, 20.75).unwrap(),
        21
    );
    assert!(projected("0", ElementScrollSizeField::ScrollWidth, 2_147_483_648.0).is_err());
    for value in [Scalar::NAN, Scalar::INFINITY, -1.0] {
        assert!(projected("0", ElementScrollSizeField::ClientWidth, value).is_err());
    }
    let g = golden(
        "width=\"16777216px\" height=\"30px\"",
        None,
        &[("0", [16_777_217, 30, 16_777_217, 30])],
    );
    assert_eq!(
        integer_rows(&g),
        [
            (
                "element-scroll".into(),
                ElementScrollSizeField::ClientWidth,
                16_777_217,
                16_777_216
            ),
            (
                "element-scroll".into(),
                ElementScrollSizeField::ScrollWidth,
                16_777_217,
                16_777_216
            ),
        ]
    );
}

#[test]
fn all_requested_integer_fields_and_old_fields_accumulate_without_tolerance() {
    let mut g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [21, 31, 22, 32])],
    );
    g.expectations.width = Some(21.0);
    let rows = collect_surgeist_numeric_mismatches(&g).unwrap();
    let NumericMismatch::Layout(first) = &rows[0] else {
        panic!("old field retains layout domain")
    };
    assert_eq!(
        (first.field(), first.browser_value(), first.native_value()),
        (NumericComparisonField::Width, 21.0, 20.0)
    );
    let actual = rows[1..]
        .iter()
        .map(|row| {
            let NumericMismatch::ElementScrollSize(row) = row else {
                panic!("expected CSSOM domain")
            };
            (row.field(), row.browser_value(), row.native_value())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        [
            (ElementScrollSizeField::ClientWidth, 21, 20),
            (ElementScrollSizeField::ClientHeight, 31, 30),
            (ElementScrollSizeField::ScrollWidth, 22, 20),
            (ElementScrollSizeField::ScrollHeight, 32, 30),
        ]
    );
    assert_eq!(
        assert_surgeist_matches(&g).unwrap_err().to_string(),
        "element-scroll: width mismatch, expected 21, got 20"
    );
}

#[test]
fn later_unsupported_child_invalidates_the_complete_case_after_earlier_mismatches() {
    let child = "<div source-tag=\"div\" display=\"block\" width=\"5px\" height=\"10px\" overflow-x=\"auto\" overflow-y=\"auto\"/>";
    let mut g = golden(
        "width=\"20px\" height=\"30px\"",
        Some(child),
        &[("0", [21, 31, 21, 31]), ("0.0", [5, 10, 5, 10])],
    );
    g.expectations.width = Some(21.0);
    let error = collect_surgeist_numeric_mismatches(&g)
        .unwrap_err()
        .to_string();
    assert!(error.contains("element-scroll/0: unsupported or invalid element-scroll-sizes"));
}

#[test]
fn fractional_local_sizes_do_not_borrow_world_corner_phase_or_rounding_mode() {
    let child = "<div source-tag=\"div\" display=\"block\" position=\"absolute\" left=\"0.5px\" top=\"0.5px\" width=\"10.75px\" height=\"20.75px\"/>";
    let raw = xml(
        "width=\"40px\" height=\"40px\"",
        Some(child),
        &table(&[("0", [40, 40, 40, 40]), ("0.0", [11, 21, 11, 21])]),
    );
    for rounded in [true, false] {
        let raw = raw.replace(
            "use-rounding=\"true\"",
            &format!("use-rounding=\"{rounded}\""),
        );
        let g = Golden::parse(&raw).unwrap();
        assert_surgeist_matches(&g).unwrap();
        let changed = Golden::parse(&raw.replace(
            "x=\"0\" y=\"0\" width=\"0\" height=\"0\"",
            "x=\"100.5\" y=\"200.5\" width=\"999\" height=\"999\"",
        ))
        .unwrap();
        assert_surgeist_matches(&changed).unwrap();
    }
}

#[test]
fn origin_anchored_scrolling_area_uses_own_direction_and_preserves_visible_clip_ranges() {
    let child = "<div source-tag=\"div\" display=\"block\" position=\"absolute\" left=\"-8px\" top=\"0px\" width=\"33px\" height=\"40.5px\"/>";
    for overflow in ["visible", "clip", "hidden"] {
        for (direction, sw) in [("ltr", 25), ("rtl", 28)] {
            let g = golden(
                &format!(
                    "direction=\"{direction}\" width=\"20.25px\" height=\"30.25px\" overflow-x=\"{overflow}\" overflow-y=\"{overflow}\""
                ),
                Some(child),
                &[("0", [20, 30, sw, 41]), ("0.0", [33, 41, 33, 41])],
            );
            assert_surgeist_matches(&g).unwrap();
            if overflow != "hidden" {
                let mut ranged = g.clone();
                ranged.expectations.scroll_size = Some(super::super::Size::new(0.0, 0.0));
                assert_surgeist_matches(&ranged).unwrap();
            }
        }
    }
}

#[test]
fn unreachable_origin_side_and_top_do_not_inflate_cssom_sizes() {
    for (direction, left) in [("ltr", -15), ("rtl", 25)] {
        let child = format!(
            "<div source-tag=\"div\" display=\"block\" position=\"absolute\" left=\"{left}px\" top=\"-9px\" width=\"10px\" height=\"8px\"/>"
        );
        for overflow in ["visible", "clip", "hidden"] {
            let g = golden(
                &format!(
                    "direction=\"{direction}\" width=\"20px\" height=\"30px\" overflow-x=\"{overflow}\" overflow-y=\"{overflow}\""
                ),
                Some(&child),
                &[("0", [20, 30, 20, 30]), ("0.0", [10, 8, 10, 8])],
            );
            assert_surgeist_matches(&g).unwrap();
        }
    }
}

#[test]
fn later_descendant_integer_and_layout_fields_survive_an_earlier_difference() {
    let child = "<div source-tag=\"div\" display=\"block\" width=\"5px\" height=\"10px\"/>";
    let mut g = golden(
        "width=\"20px\" height=\"30px\"",
        Some(child),
        &[("0", [21, 30, 21, 30]), ("0.0", [5, 10, 7, 10])],
    );
    g.expectations.children[0].height = Some(11.0);
    let rows = collect_surgeist_numeric_mismatches(&g).unwrap();
    let observations = rows
        .into_iter()
        .map(|row| match row {
            NumericMismatch::Layout(row) => format!(
                "{}:{}:{}:{}",
                row.path(),
                row.field().as_str(),
                row.browser_value(),
                row.native_value()
            ),
            NumericMismatch::ElementScrollSize(row) => format!(
                "{}:{}:{}:{}",
                row.path(),
                row.field().as_str(),
                row.browser_value(),
                row.native_value()
            ),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observations,
        [
            "element-scroll:clientWidth:21:20",
            "element-scroll:scrollWidth:21:20",
            "element-scroll/0:height:11:10",
            "element-scroll/0:scrollWidth:7:5",
        ]
    );
}

#[test]
fn semantic_projection_requires_a_real_unrounded_output_before_observation() {
    let g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [20, 30, 20, 30])],
    );
    let tree =
        super::super::TestTree::from_golden(&g.root, Some(&g.viewport.root_context)).unwrap();
    let error = project("element-scroll", &tree.nodes[0]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("missing actual unrounded box output")
    );
    assert_surgeist_matches(&g).unwrap();
}

#[test]
fn native_anonymous_text_helpers_do_not_change_original_div_path_custody() {
    let children = "<text source-tag=\"div\" display=\"grid\" width=\"20px\" height=\"20px\">H</text><div source-tag=\"div\" display=\"block\" width=\"5px\" height=\"10px\"/>";
    let raw=xml("width=\"50px\" height=\"50px\"",Some(children),&table(&[("0",[50,50,50,50]),("0.0",[20,20,20,20]),("0.1",[5,10,5,10])]))
        .replace("<expectations><node><node/></node></expectations>","<expectations><node><node/><node/></node></expectations>")
        .replace("</diagnostics>","<world-rounded-layout node-path=\"0.1\" x=\"0\" y=\"0\" width=\"0\" height=\"0\"/></diagnostics>");
    let g = Golden::parse(&raw).unwrap();
    assert_surgeist_matches(&g).unwrap();
    assert!(collect_surgeist_numeric_mismatches(&g).unwrap().is_empty());
}

#[test]
fn table_schema_rejects_namespaces_nested_rows_duplicates_and_each_missing_field() {
    let raw = xml(
        "width=\"20px\" height=\"30px\"",
        None,
        &table(&[("0", [20, 30, 20, 30])]),
    );
    let row = "<element node-path=\"0\" client-width=\"20\" client-height=\"30\" scroll-width=\"20\" scroll-height=\"30\"/>";
    for changed in [
        raw.replace(
            row,
            &row.replace("<element ", "<element xmlns=\"urn:foreign\" "),
        ),
        raw.replace(
            row,
            &row.replace("client-width=", "xmlns:x=\"urn:foreign\" x:client-width="),
        ),
        raw.replace(row, &row.replace("/>", "><element/></element>")),
        raw.replace(row, &row.replace("/>", ">unexpected</element>")),
        raw.replace(
            row,
            &row.replace(
                "client-width=\"20\"",
                "client-width=\"20\" client-width=\"20\"",
            ),
        ),
    ] {
        assert!(
            Golden::parse(&changed).is_err(),
            "closed row schema refuses malformed content"
        );
    }
    for attribute in [
        "node-path=\"0\"",
        "client-width=\"20\"",
        "client-height=\"30\"",
        "scroll-width=\"20\"",
        "scroll-height=\"30\"",
    ] {
        assert!(
            Golden::parse(&raw.replace(row, &row.replace(attribute, ""))).is_err(),
            "missing required {attribute}"
        );
    }
}

#[test]
fn source_profile_rejects_transforms_zoom_and_mixed_opaque_text() {
    for extra in [
        "zoom=\"2\"",
        "transform=\"translateX(1px)\"",
        "translate=\"1px\"",
        "rotate=\"1deg\"",
        "scale=\"2\"",
        "perspective=\"100px\"",
    ] {
        let raw = xml(
            &format!("width=\"20px\" height=\"30px\" {extra}"),
            None,
            &table(&[("0", [20, 30, 20, 30])]),
        );
        assert!(
            Golden::parse(&raw).is_err(),
            "outside closed element profile: {extra}"
        );
    }
    let child = "<div source-tag=\"div\" display=\"block\" width=\"5px\" height=\"10px\"/>";
    let raw = xml(
        "width=\"20px\" height=\"30px\"",
        Some(child),
        &table(&[("0", [20, 30, 20, 30]), ("0.0", [5, 10, 5, 10])]),
    );
    assert!(
        Golden::parse(&raw.replace(child, &format!("H{child}"))).is_err(),
        "opaque element text requires an actual leaf"
    );
}

#[test]
fn changing_source_after_parse_cannot_reuse_the_captured_profile_or_census() {
    let mut g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [20, 30, 20, 30])],
    );
    g.root
        .style
        .attrs
        .insert("source-tag".into(), "span".into());
    assert!(
        collect_surgeist_numeric_mismatches(&g)
            .unwrap_err()
            .to_string()
            .contains("element-scroll-sizes")
    );
    let mut g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [20, 30, 20, 30])],
    );
    g.root.children.push(g.root.clone());
    assert!(
        collect_surgeist_numeric_mismatches(&g)
            .unwrap_err()
            .to_string()
            .contains("source changed")
    );
}

#[test]
fn strict_mode_reports_the_first_exact_integer_difference() {
    let g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [21, 31, 22, 32])],
    );
    assert_eq!(
        assert_surgeist_matches(&g).unwrap_err().to_string(),
        "element-scroll: clientWidth mismatch, expected 21, got 20"
    );
}

#[test]
fn half_ties_preserve_local_sizes_in_both_directions_and_placement_phases() {
    for direction in ["ltr", "rtl"] {
        for phase in ["0", "0.5"] {
            let child = format!(
                "<div source-tag=\"div\" display=\"block\" position=\"absolute\" direction=\"{direction}\" left=\"{phase}px\" top=\"{phase}px\" width=\"10.5px\" height=\"20.5px\"/>"
            );
            let g = golden(
                "width=\"40px\" height=\"40px\"",
                Some(&child),
                &[("0", [40, 40, 40, 40]), ("0.0", [11, 21, 11, 21])],
            );
            assert_surgeist_matches(&g).unwrap();
        }
    }
}

#[test]
fn actual_completed_output_without_canonical_scroll_geometry_is_refused() {
    let g = golden(
        "width=\"20px\" height=\"30px\"",
        None,
        &[("0", [20, 30, 20, 30])],
    );
    let mut tree =
        super::super::TestTree::from_golden(&g.root, Some(&g.viewport.root_context)).unwrap();
    let request = super::super::root_request(&g.viewport.root_context).unwrap();
    let batch = super::super::compute_actual_batch(&tree, &g.name, request).unwrap();
    tree.apply_completed_batch(&batch);
    assert!(project("element-scroll", &tree.nodes[0]).is_ok());
    // Remove authority from the completed batch; no numeric substitute is supplied.
    tree.nodes[0].unrounded.scroll_geometry = None;
    assert!(
        project("element-scroll", &tree.nodes[0])
            .unwrap_err()
            .to_string()
            .contains("missing actual unrounded canonical scroll geometry")
    );
}

#[test]
fn replacing_the_public_root_and_viewport_cannot_retain_a_foreign_capture_profile() {
    let attrs = "width=\"20px\" height=\"30px\"";
    let mut captured = golden(attrs, None, &[("0", [20, 30, 20, 30])]);
    let mut raw = xml(attrs, None, "");
    let start = raw.find("<viewport ").unwrap();
    let end = start + raw[start..].find("/>").unwrap() + 2;
    raw.replace_range(start..end, r#"<viewport root-context="block-flow-auto" parent-inline-extent="800px" parent-direction="ltr" fallback-icb-width="800px" fallback-icb-height="600px" fallback-nearest-scrollport="none" fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>"#);
    raw = raw
        .replace(
            "observation-frame=\"root-border-local\"",
            "observation-frame=\"root-border\"",
        )
        .replace("position=\"absolute\"", "position=\"relative\"");
    let replacement = Golden::parse(&raw).unwrap();
    assert_surgeist_matches(&replacement).unwrap();
    captured.viewport = replacement.viewport;
    captured.root = replacement.root;
    assert!(
        collect_surgeist_numeric_mismatches(&captured).is_err(),
        "the retained table requires current absolute-root-local authority"
    );
}

//! Checked-in browser controls that need only the public Layout API.

use super::{portable_qualification, support};

#[test]
fn registered_current_editions_qualify_without_generator() {
    let edition =
        portable_qualification::admit_registered(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect("required current edition must authenticate");
    let selected = edition.outputs().collect();
    let run = edition
        .execute_selected(&selected)
        .expect("complete registered selection");
    for case in &run.cases {
        eprintln!(
            "current/{} raw={:?}; qualified={:?}",
            case.output, case.raw, case.qualified
        );
    }
    assert!(
        run.current_coverage_complete(),
        "all registered current requirements must qualify"
    );
}

#[test]
fn grid_static_position_edition_qualifies_all_original_variants() {
    use support::exact_differences::{PublishedEdition, QualifiedVerdict, RawVerdict};

    let owner = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let registry = portable_qualification::admit_registered(owner).unwrap();
    let edition = PublishedEdition::GridStaticPosition.artifact_root();
    let variants = [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ];
    let mut expected = std::collections::BTreeMap::new();
    for (family, fields) in [
        ("grid_absolute_layout_within_border_static", 8),
        ("grid_absolute_resolved_insets", 4),
    ] {
        for variant in variants {
            let relative = format!("xml/grid/{family}__{variant}.xml");
            expected.insert(format!("{edition}/{relative}"), (relative, fields));
        }
    }
    let prefix = format!("{edition}/");
    let selected = registry
        .outputs()
        .filter(|output| output.starts_with(&prefix))
        .collect();
    assert_eq!(selected, expected.keys().cloned().collect());
    let run = registry.execute_selected(&selected).unwrap();
    assert!(run.selected_cases_passed());
    assert_eq!(run.cases.len(), 8);
    for case in run.cases {
        let (relative, fields) = &expected[&case.output];
        assert_eq!(
            std::fs::read(owner.join(&case.output)).unwrap(),
            std::fs::read(
                owner
                    .join("tests/layout/browser_parity/captured/root-context-v4")
                    .join(relative)
            )
            .unwrap(),
        );
        assert_eq!(case.raw.unwrap().len(), *fields);
        let comparison = case.qualified.unwrap();
        assert_eq!(comparison.raw, RawVerdict::Different { fields: *fields });
        assert_eq!(
            comparison.qualified,
            QualifiedVerdict::Intentional {
                policy: "grid-absolute-child-content-edges".to_owned(),
                edition: "css-grid-1-20250326-native-1".to_owned(),
            }
        );
    }
}

#[test]
fn captured_root_context_edition_matches_native_geometry_without_generator() {
    use support::exact_differences::PublishedEdition;

    let owner = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = owner.join("tests/layout/browser_parity/captured/root-context-v4");
    let fixtures = support::fixture_files_in(&root.join("xml"), "xml")
        .expect("the source-qualified edition is available to ordinary XML consumers");
    assert!(
        !fixtures.is_empty(),
        "the captured edition has Native cases"
    );
    let current = portable_qualification::admit_grid_static_positions(owner).unwrap();
    let selected = current.outputs().map(str::to_owned).collect();
    let run = current.execute_selected(&selected).unwrap();
    assert!(run.registered_coverage_complete());
    let mut qualified: std::collections::BTreeSet<_> = run
        .cases
        .into_iter()
        .map(|case| root.join(case.output))
        .collect();
    let edition = owner.join(PublishedEdition::GridStaticPosition.artifact_root());
    for path in fixtures {
        if qualified.remove(&path) {
            assert_eq!(
                std::fs::read(&path).unwrap(),
                std::fs::read(edition.join(path.strip_prefix(&root).unwrap())).unwrap(),
                "qualified capture must retain {}",
                path.display(),
            );
            continue;
        }
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
    }
    assert!(
        qualified.is_empty(),
        "missing qualified predecessors: {qualified:?}"
    );
}

#[test]
fn fixed_grid_contributions_preserve_baseline_distribution() {
    for family in [
        "grid_align_items_baseline_child_multiline_no_override_on_secondline",
        "grid_align_items_baseline_child_multiline_override",
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/captured/root-context-v4/xml/grid")
                .join(format!("{family}__{variant}.xml"));
            let golden = support::Golden::parse_file(&path)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
            support::assert_surgeist_matches(&golden)
                .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
        }
    }
}

#[test]
fn grid_absolute_percentage_control_matches_public_layout() {
    assert_fri06_c12_t08_complete_fixture_geometry(
        "tests/layout/browser_parity/captured/root-context-v4/xml/grid/absolute_correct_cross_child_size_with_percentage__content_box_ltr.xml",
    );
}

#[test]
fn fri08_c03_intrinsic_checked_in_min_max_container_variants_use_candidate_projection() {
    for family in [
        "grid_lanes_min_content_container_sizing",
        "grid_lanes_max_content_container_sizing",
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/captured/root-context-v4/xml/grid-lanes")
                .join(format!("{family}__{variant}.xml"));
            let golden = support::Golden::parse_file(&path)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
            support::assert_surgeist_matches(&golden)
                .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
        }
    }
}

#[test]
fn fri08_c04_standalone_column_autoflow_measures_the_local_grid_container() {
    for variant in [
        "border_box_ltr",
        "border_box_rtl",
        "content_box_ltr",
        "content_box_rtl",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/layout/browser_parity/captured/root-context-v4/xml/subgrid")
            .join(format!(
                "subgrid_standalone_axis_column_autoflow__{variant}.xml"
            ));
        let golden = support::Golden::parse_file(&path)
            .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
        support::assert_surgeist_matches(&golden)
            .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
    }
}

#[test]
fn fri08_c04_overflow_frozen_grid_and_subgrid_controls_match_public_layout() {
    for (suite, source) in [
        ("grid", "grid_overflow_inline_axis_scroll"),
        ("subgrid", "subgrid_overflow_hidden_does_not_prohibit"),
        (
            "subgrid",
            "subgrid_sibling_overflow_footer_second_matches_first",
        ),
        (
            "subgrid",
            "subgrid_sibling_overflow_footer_third_matches_first",
        ),
    ] {
        for variant in [
            "border_box_ltr",
            "border_box_rtl",
            "content_box_ltr",
            "content_box_rtl",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/layout/browser_parity/captured/root-context-v4/xml")
                .join(suite)
                .join(format!("{source}__{variant}.xml"));
            let golden = support::Golden::parse_file(&path)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.display()));
            support::assert_surgeist_matches(&golden)
                .unwrap_or_else(|error| panic!("{} must match: {error}", path.display()));
        }
    }
}

fn assert_fri06_c12_t08_complete_fixture_geometry(relative_path: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let golden = support::Golden::parse_file(path).expect("reviewed T08 fixture parses");
    let _ = (
        support::fixture_files,
        support::fixture_files_in,
        support::fixture_skip_policy_mentions_x_prefix(),
        support::fixture_skip_policy_filters_unsupported_constructs(),
        golden.root.style.display(),
        golden.root.style.width(),
    );
    support::assert_surgeist_matches(&golden)
        .unwrap_or_else(|error| panic!("{} complete geometry: {error}", golden.name));
}

#[test]
fn horizontal_auto_rows_composed_target_places_later_item_at_y128_without_moving_first_row() {
    assert_fri06_c12_t08_complete_fixture_geometry(
        "tests/layout/browser_parity/captured/root-context-v4/xml/subgrid/subgrid_baseline_auto_rows_inner_row1_first__border_box_ltr.xml",
    );
}

#[test]
fn rtl_inline_column_composed_target_places_first_item_at_x265() {
    assert_fri06_c12_t08_complete_fixture_geometry(
        "tests/layout/browser_parity/captured/root-context-v4/xml/subgrid/subgrid_baseline_inline_column_inner_col1_first__border_box_rtl.xml",
    );
}

#[test]
fn ltr_inline_column_composed_target_remains_x265() {
    assert_fri06_c12_t08_complete_fixture_geometry(
        "tests/layout/browser_parity/captured/root-context-v4/xml/subgrid/subgrid_baseline_inline_column_inner_col1_first__border_box_ltr.xml",
    );
}

#[test]
fn vertical_auto_rows_composed_target_remains_x308() {
    assert_fri06_c12_t08_complete_fixture_geometry(
        "tests/layout/browser_parity/captured/root-context-v4/xml/subgrid/subgrid_baseline_vertical_auto_rows_inner_row1_first__border_box_ltr.xml",
    );
}

#[test]
fn fri08_c06_collapsed_gutter_golden_matches_the_automatic_span_line_offsets() {
    let golden = support::Golden::parse(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/layout/browser_parity/captured/root-context-v4/xml/grid/fri08_auto_fit_occupied_track_collapse__border_box_ltr.xml"
    )))
    .expect("authoritative collapsed-gutter fixture parses");
    support::assert_surgeist_matches(&golden)
        .expect("automatic span uses the coincident-gutter line offsets");
}

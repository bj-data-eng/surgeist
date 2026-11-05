use std::path::Path;

use super::*;
use crate::fixture_variants::fixture_cases;

fn captured_and_historical(variant: &str) -> (Golden, Golden) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/layout/browser_parity");
    let relative = format!("xml/subgrid/{STEM}__{variant}.xml");
    let captured = root
        .join("captured/original-subgrid-body-qualification")
        .join(&relative);
    (
        Golden::parse_file(captured).unwrap(),
        Golden::parse_file(root.join(relative)).unwrap(),
    )
}

#[test]
fn original_subgrid_body_matches_each_variant_and_its_complete_historical_geometry() {
    for (variant, _) in fixture_cases() {
        let (captured, historical) = captured_and_historical(variant);
        verify_historical(&captured, variant, &historical).unwrap();
        let (raw, qualified) = compare(&captured, variant).unwrap();
        assert!(raw.is_empty(), "{variant}: {raw:?}");
        assert_eq!(
            qualified.unwrap().qualified,
            QualifiedVerdict::MatchingControl
        );
    }
}

#[test]
fn original_subgrid_body_preserves_signed_grid_placement_and_late_box_differences() {
    let variant = "content_box_ltr";
    let (mut captured, historical) = captured_and_historical(variant);
    let grid = &mut captured.expectations.children[0];
    grid.x = Some(-2.0);
    grid.children[6].children[3].children[0].width = Some(21.0);
    let (raw, qualified) = compare(&captured, variant).unwrap();
    assert_eq!(raw.len(), 2, "{raw:?}");
    assert!(raw[0].contains("field: X") && raw[0].contains("browser_value: -2.0"));
    assert!(raw[1].contains("field: Width") && raw[1].contains("browser_value: 21.0"));
    assert!(matches!(
        qualified.unwrap_err(),
        QualificationError::DifferenceSetChanged { extra, .. } if extra == raw
    ));
    assert!(matches!(
        verify_historical(&captured, variant, &historical).unwrap_err(),
        QualificationError::DifferenceSetChanged { changed, .. }
            if changed == [
                "/0 x: historical 0, fresh -2",
                "/0/6/3/0 width: historical 20, fresh 21",
            ]
    ));
}

#[test]
fn original_subgrid_body_refuses_omitted_geometry_and_observed_block_extent_as_input() {
    let variant = "border_box_rtl";
    let (mut captured, _) = captured_and_historical(variant);
    let mut sized_body = captured.clone();
    sized_body
        .root
        .style
        .attrs
        .insert("height".into(), "72px".into());
    assert_eq!(
        validate(&sized_body, variant),
        Err(changed("original static Auto body input"))
    );

    captured.expectations.children[0].children[6].children[3].children[0].height = None;
    assert_eq!(
        validate(&captured, variant),
        Err(QualificationError::IncompleteRepresentation {
            key: "/0/6/3/0: expected x, y, width and height".into(),
        })
    );
}

#[test]
fn original_subgrid_body_checks_placement_before_rounding_can_hide_it() {
    let variant = "content_box_ltr";
    let (mut captured, _) = captured_and_historical(variant);
    captured.root.children[0]
        .style
        .attrs
        .insert("margin-left".into(), "0.25px".into());
    let (raw, qualified) = compare(&captured, variant).unwrap();
    assert!(
        raw.iter()
            .any(|field| field.ends_with("/0: unrounded x, browser 0, Native 0.25")),
        "{raw:?}"
    );
    assert!(qualified.is_err());
}

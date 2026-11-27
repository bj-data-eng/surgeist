//! Authored expectations for HTML877d Input step-grid projection.

use surgeist_html::{
    elements::input::{
        HtmlInputStepBounds as Bounds, HtmlInputStepDirection as Direction, HtmlInputStepError,
    },
    microsyntax::numeric::{HtmlNonNegativeDecimal, HtmlNonNegativeInteger, HtmlSignedDecimal},
    values::DocumentString,
};

fn decimal(text: &str) -> HtmlSignedDecimal {
    let negative = text.starts_with('-');
    let text = text.strip_prefix('-').unwrap_or(text);
    let (integer, fraction) = text.split_once('.').unwrap_or((text, ""));
    let integer =
        HtmlNonNegativeInteger::from_decimal_digits(&DocumentString::from(integer)).unwrap();
    let magnitude = if fraction.is_empty() {
        integer.into()
    } else {
        HtmlNonNegativeDecimal::with_fraction_digits(integer, &DocumentString::from(fraction))
            .unwrap()
    };
    HtmlSignedDecimal::from_magnitude(magnitude, negative)
}

fn project(
    current: Option<&str>,
    base: &str,
    step: &str,
    range: (Option<&str>, Option<&str>),
    direction: Direction,
    count: i32,
) -> Option<(bool, Vec<u8>, usize)> {
    let current = current.map(decimal);
    let base = decimal(base);
    let step = decimal(step);
    let min = range.0.map(decimal);
    let max = range.1.map(decimal);
    let bounds = Bounds {
        minimum: min.as_ref(),
        maximum: max.as_ref(),
        base: &base,
        step: step.magnitude(),
    };
    let mut qualification = vec![255; bounds.qualification_cells().unwrap()];
    let grid = bounds
        .qualify(&mut qualification, || Ok::<_, ()>(()))
        .unwrap()?;
    let mut scratch = vec![255; grid.scratch_cells(current.as_ref()).unwrap()];
    grid.project(current.as_ref(), direction, count, &mut scratch, || {
        Ok::<_, ()>(())
    })
    .unwrap()
    .map(|value| {
        (
            value.is_negative(),
            value.coefficient().to_vec(),
            value.scale(),
        )
    })
}

#[test]
fn unaligned_signed_coordinates_select_the_neighbor_in_the_invoked_direction() {
    use Direction::{Down, Up};
    for (current, base, direction, negative, digits) in [
        ("1.001", "-0.125", Up, false, &[5, 7, 3, 1][..]),
        ("1.001", "-0.125", Down, false, &[5, 7, 8][..]),
        ("-1.001", "0.125", Up, true, &[5, 7, 8][..]),
        ("-1.001", "0.125", Down, true, &[5, 7, 3, 1][..]),
    ] {
        assert_eq!(
            project(Some(current), base, "0.5", (None, None), direction, -100),
            Some((negative, digits.to_vec(), 3)),
        );
    }
}

#[test]
fn projections_inside_bounds_reach_the_nearest_legal_grid_point() {
    use Direction::{Down, Up};
    let range = (Some("2"), Some("8"));
    assert_eq!(
        project(None, "1", "3", range, Up, 0),
        Some((false, vec![4], 0))
    );
    assert_eq!(
        project(Some("4"), "1", "3", range, Up, 100),
        Some((false, vec![7], 0))
    );
    assert_eq!(
        project(Some("4"), "1", "3", range, Down, 100),
        Some((false, vec![4], 0))
    );
    assert_eq!(
        project(Some("4"), "1", "3", range, Up, -1),
        Some((false, vec![4], 0))
    );
    assert_eq!(project(Some("7"), "1", "3", range, Up, -1), None);
    assert_eq!(project(None, "0", "3", (Some("1"), Some("2")), Up, 1), None);
    assert_eq!(project(None, "0", "3", (Some("7"), Some("2")), Up, 1), None);
}

#[test]
fn aligned_extents_can_exceed_binary64_and_counts_keep_exact_signed_endpoints() {
    use Direction::{Down, Up};
    assert_eq!(
        project(
            Some("9007199254740992.125"),
            "0",
            "0.125",
            (None, None),
            Up,
            1
        ),
        Some((
            false,
            vec![0, 5, 2, 2, 9, 9, 0, 4, 7, 4, 5, 2, 9, 9, 1, 7, 0, 0, 9],
            3
        )),
    );
    assert_eq!(project(None, "0", "1", (None, None), Up, i32::MIN), None);
    assert_eq!(
        project(None, "0", "1", (None, None), Down, i32::MAX),
        Some((true, vec![7, 4, 6, 3, 8, 4, 7, 4, 1, 2], 0)),
    );
}

#[test]
fn projection_refuses_short_workspace_and_preserves_the_first_work_stop() {
    let base = decimal("0");
    let step = decimal("3");
    let bounds = Bounds {
        minimum: None,
        maximum: None,
        base: &base,
        step: step.magnitude(),
    };
    let grid = bounds
        .qualify(&mut [], || Ok::<_, ()>(()))
        .unwrap()
        .unwrap();
    let count = grid.scratch_cells(None).unwrap();
    let mut scratch = vec![0; count - 1];
    assert_eq!(
        grid.project(None, Direction::Up, 1, &mut scratch, || Ok::<_, u16>(())),
        Err(HtmlInputStepError::Workspace),
    );
    scratch.push(0);
    assert_eq!(
        grid.project(None, Direction::Up, 1, &mut scratch, || Err(123_u16)),
        Err(HtmlInputStepError::Work(123)),
    );
}

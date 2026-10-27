//! Native real-box witnesses; no synthetic Text or measured-font substitutes.
use super::*;

fn geometry<S: l::LayoutScalar>(
    batch: &l::CompletedLayoutBatchOf<u32, S>,
    direction: l::Direction,
    width: f64,
    widths: &[f64],
    rows: &[&[u32]],
) {
    assert!(batch.placed_text().is_empty());
    assert!(batch.final_inline_fragments().is_empty());
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        assert_eq!(entries.len(), widths.len() + 1);
        assert_eq!(
            numeric_node(entries, 0).size,
            l::Size::new(S::from_f64(width), S::from_f64(10.0 * rows.len() as f64))
        );
        for (row, ids) in rows.iter().enumerate() {
            let mut used = 0.0;
            for &id in ids.iter() {
                let actual = numeric_node(entries, id);
                let w = widths[id as usize - 1];
                assert_eq!(entries.iter().filter(|e| e.node() == id).count(), 1);
                assert_eq!(actual.source_index, l::SourceIndex::new(id as usize - 1));
                assert_eq!(actual.size, l::Size::new(S::from_f64(w), S::from_f64(10.0)));
                // Independently supplied uniform levels reverse the physical
                // slots in RTL; Start anchors each row at the true right edge.
                let x = if direction == l::Direction::Ltr {
                    used
                } else {
                    width - used - w
                };
                assert_eq!(
                    actual.location,
                    l::Point::new(S::from_f64(x), S::from_f64(10.0 * row as f64))
                );
                used += w;
            }
        }
    }
}
fn profile_tree<S: l::LayoutScalar>(
    direction: l::Direction,
    width: f64,
    widths: &[f64],
) -> Tree<S> {
    let mut tree = numeric_tree(
        direction,
        widths
            .iter()
            .map(|&w| numeric_atomic(direction, w, 10.0, l::InlineBreakOpportunity::Allowed))
            .collect(),
    );
    let root = tree.inputs[0].as_box().cloned().unwrap();
    tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
        size: l::Size::new(
            l::PreferredSizeOf::value(l::LengthPercentageOf::px(S::from_f64(width)).unwrap()),
            root.size.height,
        ),
        ..root
    });
    tree
}
struct NativeProfile<'a> {
    width: f64,
    widths: &'a [f64],
    ordinary: &'a [&'a [u32]],
    repaired: &'a [&'a [u32]],
}
fn literal_profiles<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let profiles = [
            NativeProfile {
                width: 60.0,
                widths: &[30.0, 30.0, 10.0],
                ordinary: &[&[1, 2], &[3]],
                repaired: &[&[1], &[2, 3]],
            },
            NativeProfile {
                width: 60.0,
                widths: &[10.0, 50.0, 10.0],
                ordinary: &[&[1, 2], &[3]],
                repaired: &[&[1, 2], &[3]],
            },
            // A whole-group30/50/50 program changes frozen first cut2.
            NativeProfile {
                width: 60.0,
                widths: &[30.0, 30.0, 20.0, 50.0],
                ordinary: &[&[1, 2], &[3], &[4]],
                repaired: &[&[1, 2], &[3], &[4]],
            },
            NativeProfile {
                width: 170.0,
                widths: &[160.0, 30.0, 120.0, 50.0],
                ordinary: &[&[1], &[2, 3], &[4]],
                repaired: &[&[1], &[2, 3], &[4]],
            },
        ];
        for NativeProfile {
            width,
            widths,
            ordinary,
            repaired,
        } in profiles
        {
            let mut tree = profile_tree::<S>(direction, width, widths);
            let capacity = l::AvailableOf::definite(S::from_f64(width));
            let auto = numeric_compute(&tree, capacity, &[]);
            geometry(&auto, direction, width, widths, ordinary);
            let actual = numeric_style(&mut tree, l::TextWrapStyle::AvoidShortLastLine, capacity);
            geometry(&actual, direction, width, widths, repaired);
            let replay = numeric_compute(&tree, capacity, &[]);
            numeric_equal(&actual, &replay, tree.inputs.len());
            let mut fresh = profile_tree::<S>(direction, width, widths);
            let cold = numeric_style(&mut fresh, l::TextWrapStyle::AvoidShortLastLine, capacity);
            numeric_equal(&actual, &cold, tree.inputs.len());
            geometry(&auto, direction, width, widths, ordinary);
        }
    }
}
#[test]
fn avoid_short_last_line_native_repair_transfer_and_frozen_prefix_f32() {
    literal_profiles::<f32>();
}
#[test]
fn avoid_short_last_line_native_repair_transfer_and_frozen_prefix_f64() {
    literal_profiles::<f64>();
}

fn forced_and_permission<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let capacity = l::AvailableOf::definite(S::from_f64(60.0));
        let mut tree = partition_tree::<S>(direction, 2, l::InlineBreakOpportunity::Allowed);
        let auto = numeric_compute(&tree, capacity, &[]);
        assert_partition(&auto, direction, 2, false);
        let actual = numeric_style(&mut tree, l::TextWrapStyle::AvoidShortLastLine, capacity);
        for entries in [actual.unrounded_entries(), actual.final_entries()] {
            assert_eq!(
                numeric_node(entries, 0).size,
                l::Size::new(S::from_f64(60.0), S::from_f64(40.0))
            );
            for (ids, top, used) in [
                (&[1, 2][..], 0.0, 60.0),
                (&[3][..], 10.0, 10.0),
                (&[5][..], 20.0, 30.0),
                (&[6, 7][..], 30.0, 40.0),
            ] {
                assert_row(entries, ids, direction, top, used);
            }
            for (id, width) in [
                (1, 30.0),
                (2, 30.0),
                (3, 10.0),
                (5, 30.0),
                (6, 30.0),
                (7, 10.0),
            ] {
                let output = numeric_node(entries, id);
                assert_eq!(output.source_index, l::SourceIndex::new(id as usize - 1));
                assert_eq!(
                    output.size,
                    l::Size::new(S::from_f64(width), S::from_f64(10.0))
                );
            }
            let control = numeric_node(entries, 4);
            assert_eq!(control.source_index, l::SourceIndex::new(3));
            assert_eq!(control.size, l::Size::ZERO);
            assert_eq!(control.location.y, S::from_f64(20.0));
        }
        // A real single-line terminal group cannot borrow across Mandatory.
        let mut single = numeric_tree::<S>(
            direction,
            vec![
                numeric_atomic(direction, 30.0, 10.0, l::InlineBreakOpportunity::Allowed),
                numeric_forced(direction, 10.0, 10.0),
                numeric_atomic(direction, 10.0, 10.0, l::InlineBreakOpportunity::Allowed),
            ],
        );
        let ordinary = numeric_compute(&single, capacity, &[]);
        let selected = numeric_style(&mut single, l::TextWrapStyle::AvoidShortLastLine, capacity);
        numeric_equal(&ordinary, &selected, single.inputs.len());
        for entries in [selected.unrounded_entries(), selected.final_entries()] {
            assert_row(entries, &[1], direction, 0.0, 30.0);
            assert_row(entries, &[3], direction, 10.0, 10.0);
        }
        let mut prohibited =
            partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Prohibited);
        let ordinary = numeric_compute(&prohibited, capacity, &[]);
        let selected = numeric_style(
            &mut prohibited,
            l::TextWrapStyle::AvoidShortLastLine,
            capacity,
        );
        numeric_equal(&ordinary, &selected, prohibited.inputs.len());
        for entries in [selected.unrounded_entries(), selected.final_entries()] {
            assert_row(entries, &[1, 2, 3], direction, 0.0, 70.0);
        }
        assert_partition(&auto, direction, 2, false);
    }
}
#[test]
fn avoid_short_last_line_native_targets_only_paragraph_and_keeps_permissions_f32() {
    forced_and_permission::<f32>();
}
#[test]
fn avoid_short_last_line_native_targets_only_paragraph_and_keeps_permissions_f64() {
    forced_and_permission::<f64>();
}

fn ordinary_domains<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        // A genuine oversized first atom is valid ordinary overflow. Optional
        // quality must not turn advance70/capacity60 into a geometry error.
        let mut overflow = profile_tree::<S>(direction, 60.0, &[70.0, 10.0]);
        let capacity = l::AvailableOf::definite(S::from_f64(60.0));
        let ordinary = numeric_compute(&overflow, capacity, &[]);
        geometry(&ordinary, direction, 60.0, &[70.0, 10.0], &[&[1], &[2]]);
        let selected = numeric_style(
            &mut overflow,
            l::TextWrapStyle::AvoidShortLastLine,
            capacity,
        );
        numeric_equal(&ordinary, &selected, overflow.inputs.len());
        // Zero capacity is independently admitted by actual ordinary layout;
        // all real overflowing boxes and their source entries remain present.
        let mut zero = profile_tree::<S>(direction, 0.0, &[10.0, 10.0]);
        let zero_capacity = l::AvailableOf::definite(S::ZERO);
        let ordinary = numeric_compute(&zero, zero_capacity, &[]);
        geometry(&ordinary, direction, 0.0, &[10.0, 10.0], &[&[1], &[2]]);
        let selected = numeric_style(
            &mut zero,
            l::TextWrapStyle::AvoidShortLastLine,
            zero_capacity,
        );
        numeric_equal(&ordinary, &selected, zero.inputs.len());
        for minimum in [true, false] {
            let mut tree = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
            let root = tree.inputs[0].as_box().cloned().unwrap();
            tree.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
                size: l::Size::new(l::PreferredSizeOf::AUTO, root.size.height),
                ..root
            });
            let available = if minimum {
                l::AvailableOf::MIN_CONTENT
            } else {
                l::AvailableOf::MAX_CONTENT
            };
            let ordinary = numeric_compute(&tree, available, &[]);
            let selected =
                numeric_style(&mut tree, l::TextWrapStyle::AvoidShortLastLine, available);
            numeric_equal(&ordinary, &selected, tree.inputs.len());
            let rows: &[&[u32]] = if minimum {
                &[&[1], &[2], &[3]]
            } else {
                &[&[1, 2, 3]]
            };
            geometry(
                &selected,
                direction,
                if minimum { 30.0 } else { 70.0 },
                &[30.0, 30.0, 10.0],
                rows,
            );
        }
    }
}
#[test]
fn avoid_short_last_line_numeric_outside_optional_domain_and_intrinsics_stay_ordinary_f32() {
    ordinary_domains::<f32>();
}
#[test]
fn avoid_short_last_line_numeric_outside_optional_domain_and_intrinsics_stay_ordinary_f64() {
    ordinary_domains::<f64>();
}

fn long_final_group<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        // Each real60-wide prefix atom fills one actual row. The final three
        // atoms retain the independent60/10 ->30/40 tail witness at rows11/12.
        let mut widths = vec![60.0; 10];
        widths.extend([30.0, 30.0, 10.0]);
        let mut ordinary = (1..=10).map(|id| vec![id]).collect::<Vec<_>>();
        ordinary.extend([vec![11, 12], vec![13]]);
        let mut repaired = ordinary[..10].to_vec();
        repaired.extend([vec![11], vec![12, 13]]);
        let ordinary_rows = ordinary.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let repaired_rows = repaired.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut tree = profile_tree::<S>(direction, 60.0, &widths);
        let capacity = l::AvailableOf::definite(S::from_f64(60.0));
        let auto = numeric_compute(&tree, capacity, &[]);
        geometry(&auto, direction, 60.0, &widths, &ordinary_rows);
        let actual = numeric_style(&mut tree, l::TextWrapStyle::AvoidShortLastLine, capacity);
        geometry(&actual, direction, 60.0, &widths, &repaired_rows);
        for (before, after) in [
            (auto.unrounded_entries(), actual.unrounded_entries()),
            (auto.final_entries(), actual.final_entries()),
        ] {
            for id in 1..=10 {
                assert_eq!(numeric_node(before, id), numeric_node(after, id));
            }
        }
        let repeated = numeric_compute(&tree, capacity, &[]);
        numeric_equal(&actual, &repeated, tree.inputs.len());
        geometry(&auto, direction, 60.0, &widths, &ordinary_rows);
    }
}
#[test]
fn avoid_short_last_line_twelve_line_native_group_preserves_prefix_boxes_f32() {
    long_final_group::<f32>();
}
#[test]
fn avoid_short_last_line_twelve_line_native_group_preserves_prefix_boxes_f64() {
    long_final_group::<f64>();
}

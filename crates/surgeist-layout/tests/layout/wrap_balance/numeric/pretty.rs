//! Real native participants exercise both staged quality operations.
use super::avoid_short_last_line::{geometry, profile_tree};
use super::*;

struct Profile<'a> {
    width: f64,
    widths: &'a [f64],
    auto: &'a [&'a [u32]],
    pretty: &'a [&'a [u32]],
}
fn profiles<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for Profile {
            width,
            widths,
            auto,
            pretty,
        } in [
            Profile {
                width: 60.0,
                widths: &[30.0, 30.0, 10.0],
                auto: &[&[1, 2], &[3]],
                pretty: &[&[1], &[2, 3]],
            },
            // Tail stays put; Body Q2/9 ->5/36 with equal actual terminal50.
            Profile {
                width: 60.0,
                widths: &[30.0, 30.0, 20.0, 50.0],
                auto: &[&[1, 2], &[3], &[4]],
                pretty: &[&[1], &[2, 3], &[4]],
            },
            Profile {
                width: 60.0,
                widths: &[10.0, 50.0, 10.0],
                auto: &[&[1, 2], &[3]],
                pretty: &[&[1, 2], &[3]],
            },
            Profile {
                width: 170.0,
                widths: &[160.0, 30.0, 120.0, 50.0],
                auto: &[&[1], &[2, 3], &[4]],
                pretty: &[&[1], &[2, 3], &[4]],
            },
        ] {
            let mut tree = profile_tree::<S>(direction, width, widths);
            let available = l::AvailableOf::definite(S::from_f64(width));
            let ordinary = numeric_compute(&tree, available, &[]);
            geometry(&ordinary, direction, width, widths, auto);
            let selected = numeric_style(&mut tree, l::TextWrapStyle::Pretty, available);
            geometry(&selected, direction, width, widths, pretty);
            let repeated = numeric_compute(&tree, available, &[]);
            numeric_equal(&selected, &repeated, tree.inputs.len());
            let mut fresh = profile_tree::<S>(direction, width, widths);
            let cold = numeric_style(&mut fresh, l::TextWrapStyle::Pretty, available);
            numeric_equal(&selected, &cold, tree.inputs.len());
            geometry(&ordinary, direction, width, widths, auto);
        }
    }
}
#[test]
fn pretty_native_stages_tail_and_body_with_equal_terminal_guard_f32() {
    profiles::<f32>();
}
#[test]
fn pretty_native_stages_tail_and_body_with_equal_terminal_guard_f64() {
    profiles::<f64>();
}

fn long_body<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let mut widths = vec![60.0; 9];
        widths.extend([30.0, 30.0, 20.0, 50.0]);
        let mut auto = (1..=9).map(|id| vec![id]).collect::<Vec<_>>();
        auto.extend([vec![10, 11], vec![12], vec![13]]);
        let mut expected = auto[..9].to_vec();
        expected.extend([vec![10], vec![11, 12], vec![13]]);
        let ordinary_rows = auto.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let pretty_rows = expected.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut tree = profile_tree::<S>(direction, 60.0, &widths);
        let available = l::AvailableOf::definite(S::from_f64(60.0));
        let ordinary = numeric_compute(&tree, available, &[]);
        geometry(&ordinary, direction, 60.0, &widths, &ordinary_rows);
        let pretty = numeric_style(&mut tree, l::TextWrapStyle::Pretty, available);
        geometry(&pretty, direction, 60.0, &widths, &pretty_rows);
        // Twelve real rows, eleven nonterminal terms: Q4/99 ->5/198.
        for (before, after) in [
            (ordinary.unrounded_entries(), pretty.unrounded_entries()),
            (ordinary.final_entries(), pretty.final_entries()),
        ] {
            for id in 1..=9 {
                assert_eq!(numeric_node(before, id), numeric_node(after, id));
            }
        }
        numeric_equal(
            &pretty,
            &numeric_compute(&tree, available, &[]),
            tree.inputs.len(),
        );
        geometry(&ordinary, direction, 60.0, &widths, &ordinary_rows);
    }
}
#[test]
fn pretty_native_body_repairs_twelve_line_group_without_moving_full_prefix_f32() {
    long_body::<f32>();
}
#[test]
fn pretty_native_body_repairs_twelve_line_group_without_moving_full_prefix_f64() {
    long_body::<f64>();
}

fn forced_and_domains<S: l::LayoutScalar>() {
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let atom =
            |width| numeric_atomic(direction, width, 10.0, l::InlineBreakOpportunity::Allowed);
        let mut tree = numeric_tree::<S>(
            direction,
            vec![
                atom(30.0),
                atom(30.0),
                atom(20.0),
                atom(50.0),
                numeric_forced(direction, 10.0, 10.0),
                atom(30.0),
                atom(30.0),
                atom(10.0),
            ],
        );
        let available = l::AvailableOf::definite(S::from_f64(60.0));
        let pretty = numeric_style(&mut tree, l::TextWrapStyle::Pretty, available);
        for entries in [pretty.unrounded_entries(), pretty.final_entries()] {
            assert_eq!(
                numeric_node(entries, 0).size,
                l::Size::new(S::from_f64(60.0), S::from_f64(50.0))
            );
            // Body is permitted before real Forced; the later Paragraph group
            // independently takes Tail. No participant crosses the control.
            for (ids, top, width) in [
                (&[1][..], 0.0, 30.0),
                (&[2, 3][..], 10.0, 50.0),
                (&[4][..], 20.0, 50.0),
                (&[6][..], 30.0, 30.0),
                (&[7, 8][..], 40.0, 40.0),
            ] {
                assert_row(entries, ids, direction, top, width);
            }
            let forced = numeric_node(entries, 5);
            assert_eq!(forced.size, l::Size::ZERO);
            assert_eq!(forced.source_index, l::SourceIndex::new(4));
            assert_eq!(
                forced.location,
                l::Point::new(
                    S::from_f64(if direction == l::Direction::Ltr {
                        50.0
                    } else {
                        10.0
                    }),
                    S::from_f64(30.0)
                )
            );
            for (id, width) in [
                (1, 30.0),
                (2, 30.0),
                (3, 20.0),
                (4, 50.0),
                (6, 30.0),
                (7, 30.0),
                (8, 10.0),
            ] {
                let output = numeric_node(entries, id);
                assert_eq!(output.source_index, l::SourceIndex::new(id as usize - 1));
                assert_eq!(
                    output.size,
                    l::Size::new(S::from_f64(width), S::from_f64(10.0))
                );
            }
        }
        for (width, widths, rows) in [
            (60.0, &[70.0, 10.0][..], &[&[1][..], &[2][..]][..]),
            (0.0, &[10.0, 10.0][..], &[&[1][..], &[2][..]][..]),
            (60.0, &[0.0][..], &[&[1][..]][..]),
        ] {
            let mut domain = profile_tree::<S>(direction, width, widths);
            let capacity = l::AvailableOf::definite(S::from_f64(width));
            let auto = numeric_compute(&domain, capacity, &[]);
            geometry(&auto, direction, width, widths, rows);
            let selected = numeric_style(&mut domain, l::TextWrapStyle::Pretty, capacity);
            numeric_equal(&auto, &selected, domain.inputs.len());
        }
        let mut nowrap = partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Prohibited);
        let auto = numeric_compute(&nowrap, available, &[]);
        let selected = numeric_style(&mut nowrap, l::TextWrapStyle::Pretty, available);
        numeric_equal(&auto, &selected, nowrap.inputs.len());
        for minimum in [true, false] {
            let mut intrinsic =
                partition_tree::<S>(direction, 1, l::InlineBreakOpportunity::Allowed);
            let root = intrinsic.inputs[0].as_box().cloned().unwrap();
            intrinsic.inputs[0] = l::LayoutInputOf::box_input(l::NodeInputOf {
                size: l::Size::new(l::PreferredSizeOf::AUTO, root.size.height),
                ..root
            });
            let capacity = if minimum {
                l::AvailableOf::MIN_CONTENT
            } else {
                l::AvailableOf::MAX_CONTENT
            };
            let auto = numeric_compute(&intrinsic, capacity, &[]);
            let selected = numeric_style(&mut intrinsic, l::TextWrapStyle::Pretty, capacity);
            numeric_equal(&auto, &selected, intrinsic.inputs.len());
        }
    }
}
#[test]
fn pretty_native_forced_body_and_ordinary_optional_domain_custody_f32() {
    forced_and_domains::<f32>();
}
#[test]
fn pretty_native_forced_body_and_ordinary_optional_domain_custody_f64() {
    forced_and_domains::<f64>();
}

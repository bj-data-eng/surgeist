use super::*;

fn overflow<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for width in [100.0, 120.0] {
            let mut tree = two_rows::<S>(
                mode,
                direction,
                l::TextGroupAlign::Center,
                l::TextAlign::Start,
            );
            tree.inputs[1] = atomic(mode, direction, width);
            let batch = tree.compute(mode, &[]);
            // A zero or negative limiting remainder adds exactly0 padding.
            // Actual start-overflow remains signed; the short line is not centered.
            let first = if direction == l::Direction::Ltr {
                0.0
            } else if width == 120.0 {
                -20.0
            } else {
                0.0
            };
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(node(entries, 1).location.x, n(first));
                assert_eq!(
                    node(entries, 3).location.x,
                    if direction == l::Direction::Ltr {
                        S::ZERO
                    } else {
                        n(60.0)
                    }
                );
                assert_eq!(node(entries, 1).size, l::Size::new(n(width), n(10.0)));
                assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
            }
        }
        // A negative ancestor remainder does not disable a positive descendant
        // subset. Parent120/60/40 adds0; the child's60/40 still adds End40.
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::TextGroupAlign::Center;
        let mut child = block(mode, direction, 100.0);
        child.text_group_align = l::TextGroupAlign::End;
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root),
                atomic(mode, direction, 120.0),
                forced(mode, direction),
                l::LayoutInputOf::box_input(child),
                atomic(mode, direction, 60.0),
                forced(mode, direction),
                atomic(mode, direction, 40.0),
            ],
            &[(0, &[1, 2, 3]), (3, &[4, 5, 6])],
        );
        let batch = tree.compute(mode, &[]);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                node(entries, 1).location.x,
                if direction == l::Direction::Ltr {
                    S::ZERO
                } else {
                    n(-20.0)
                }
            );
            assert_eq!(
                node(entries, 4).location.x,
                if direction == l::Direction::Ltr {
                    n(40.0)
                } else {
                    S::ZERO
                }
            );
            assert_eq!(
                node(entries, 6).location.x,
                if direction == l::Direction::Ltr {
                    n(40.0)
                } else {
                    n(20.0)
                }
            );
        }
    }
}
#[test]
fn zero_and_negative_remainders_do_not_enlarge_or_center_the_content_frame() {
    overflow::<f32>();
    overflow::<f64>();
}

fn indentation_and_margins<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        for indent in [-20.0, 20.0] {
            let mut tree = two_rows::<S>(
                mode,
                direction,
                l::TextGroupAlign::Center,
                l::TextAlign::Start,
            );
            tree.update(0, |root| {
                root.text_indent = l::TextIndentOf {
                    amount: l::LengthPercentageOf::px(n(indent)).unwrap(),
                    each_line: true,
                    hanging: false,
                }
            });
            let batch = tree.compute(mode, &[]);
            // -20: usable120, P60 ->30/30. +20: usable80, P20 ->10/10.
            // Insets never replace the genuine signed20px indentation.
            let expected = match (indent < 0.0, direction) {
                (true, l::Direction::Ltr) => [10.0, 10.0],
                (true, l::Direction::Rtl) => [30.0, 50.0],
                (false, l::Direction::Ltr) => [30.0, 30.0],
                (false, l::Direction::Rtl) => [10.0, 30.0],
            };
            for entries in [batch.unrounded_entries(), batch.final_entries()] {
                assert_eq!(node(entries, 1).location.x, n(expected[0]));
                assert_eq!(node(entries, 3).location.x, n(expected[1]));
                assert_eq!(node(entries, 1).source_index, l::SourceIndex::new(0));
                assert_eq!(node(entries, 3).source_index, l::SourceIndex::new(2));
                assert_eq!(node(entries, 0).size.height, n(20.0));
            }
        }
        let mut tree = two_rows::<S>(
            mode,
            direction,
            l::TextGroupAlign::Center,
            l::TextAlign::Start,
        );
        tree.update(1, |item| {
            let value = l::LengthAutoOf::value(l::LengthPercentageOf::px(n(-10.0)).unwrap());
            if direction == l::Direction::Ltr {
                item.margin.left = value;
            } else {
                item.margin.right = value;
            }
        });
        let batch = tree.compute(mode, &[]);
        // The REAL60px atomic's signed start margin makes natural advance50.
        // P50 ->25/25; its negative margin is retained in actual box placement.
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let first = node(entries, 1);
            assert_eq!(
                first.location.x,
                if direction == l::Direction::Ltr {
                    n(15.0)
                } else {
                    n(25.0)
                }
            );
            assert_eq!(
                node(entries, 3).location.x,
                if direction == l::Direction::Ltr {
                    n(25.0)
                } else {
                    n(35.0)
                }
            );
            assert_eq!(first.size.width, n(60.0));
            assert_eq!(
                if direction == l::Direction::Ltr {
                    first.margin.left
                } else {
                    first.margin.right
                },
                n(-10.0)
            );
        }
    }
}
#[test]
fn genuine_signed_indent_and_atomic_margin_remain_in_the_natural_basis() {
    indentation_and_margins::<f32>();
    indentation_and_margins::<f64>();
}

fn endpoint<S: l::LayoutScalar>(
    kind: l::InlineBoundaryKind,
    direction: l::Direction,
    baseline: f64,
    height: f64,
) -> l::LayoutInputOf<S> {
    l::LayoutInputOf::inline_boundary(
        l::InlineBoundaryInputOf::contribution(
            kind,
            l::InlineMetricsOf::try_new(
                surgeist_layout::InlineBaselineContext::Physical,
                n(baseline),
                n(height),
            )
            .unwrap(),
            l::InlineBoundaryEdgeOf::ZERO,
        )
        .unwrap()
        .with_direction(direction),
    )
}
fn control_custody<S: l::LayoutScalar>() {
    let mode = l::WritingMode::HorizontalTb;
    for direction in [l::Direction::Ltr, l::Direction::Rtl] {
        let mut root = block(mode, direction, 100.0);
        root.text_group_align = l::TextGroupAlign::Center;
        // One real empty Forced line participates (N0/C100 ->P100), followed
        // by genuine34px nominal endpoints in a phantom Paragraph receiver.
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root.clone()),
                forced(mode, direction),
                endpoint(l::InlineBoundaryKind::Start, direction, 30.0, 34.0),
                endpoint(l::InlineBoundaryKind::End, direction, 30.0, 34.0),
            ],
            &[(0, &[1, 2, 3])],
        );
        let batch = tree.compute(mode, &[]);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(10.0)));
            assert_eq!(node(entries, 1).location, l::Point::new(n(50.0), n(10.0)));
            for (id, source) in [(2, 1), (3, 2)] {
                assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
                assert_eq!(node(entries, id).size, l::Size::ZERO);
                // Nonformatted descendant placement has no group assignment
                // and uses block10, never its nominal baseline30+10.
                assert_eq!(node(entries, id).location.y, n(10.0));
            }
        }
        let tree = Tree::<S>::new(
            vec![
                l::LayoutInputOf::box_input(root),
                endpoint(l::InlineBoundaryKind::Start, direction, 10.0, 10.0),
                atomic(mode, direction, 60.0),
                forced(mode, direction),
                atomic(mode, direction, 40.0),
                endpoint(l::InlineBoundaryKind::End, direction, 10.0, 10.0),
            ],
            &[(0, &[1, 2, 3, 4, 5])],
        );
        let batch = tree.compute(mode, &[]);
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            for (id, source) in [(1, 0), (2, 1), (3, 2), (4, 3), (5, 4)] {
                assert_eq!(node(entries, id).source_index, l::SourceIndex::new(source));
            }
            assert_eq!(node(entries, 2).location, l::Point::new(n(20.0), S::ZERO));
            assert_eq!(
                node(entries, 4).location,
                l::Point::new(
                    if direction == l::Direction::Ltr {
                        n(20.0)
                    } else {
                        n(40.0)
                    },
                    n(10.0)
                )
            );
            assert_eq!(
                node(entries, 1).location,
                l::Point::new(
                    if direction == l::Direction::Ltr {
                        n(20.0)
                    } else {
                        n(80.0)
                    },
                    n(10.0)
                )
            );
            assert_eq!(
                node(entries, 5).location,
                l::Point::new(
                    if direction == l::Direction::Ltr {
                        n(60.0)
                    } else {
                        n(40.0)
                    },
                    n(20.0)
                )
            );
            assert_eq!(node(entries, 1).size, l::Size::ZERO);
            assert_eq!(node(entries, 5).size, l::Size::ZERO);
            assert_eq!(node(entries, 0).size, l::Size::new(n(100.0), n(20.0)));
        }
    }
}
#[test]
fn forced_empty_members_and_real_endpoint_nodes_preserve_distinct_custody() {
    control_custody::<f32>();
    control_custody::<f64>();
}

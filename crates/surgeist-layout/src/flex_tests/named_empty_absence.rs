//! Named absence requires the Flex owner's actual empty eligible item set.
use super::*;
use crate::engine::ComputeSession;
use crate::output::{NamedBaselineDemand, NamedBaselineSet};

#[derive(Clone, Copy, Debug)]
enum Children {
    Empty,
    Absolute,
    Hidden,
    InFlow,
    Collapsed,
}

fn fixture<S: LayoutScalar>(
    flow: FlowAxes,
    direction: FlexDirection,
    children: Children,
    fixed: bool,
) -> PublicLayoutTreeOf<S> {
    let size = Size::new(S::from_f64(20.0), S::from_f64(12.0));
    let mut tree = PublicLayoutTreeOf::new().style(
        1,
        NodeInputOf {
            display: Display::Flex,
            writing_mode: flow.writing_mode(),
            direction: flow.direction(),
            flex_direction: direction,
            size: if fixed {
                size.map(PreferredSizeOf::px)
            } else {
                Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
            },
            align_items: Some(AlignItems::FlexStart),
            align_content: Some(AlignContent::FlexStart),
            justify_content: Some(AlignContent::FlexStart),
            ..NodeInputOf::default()
        },
    );
    if matches!(children, Children::Empty) {
        return tree.children(1, []);
    }
    tree = tree.children(1, [2]).children(2, []).style(
        2,
        NodeInputOf {
            display: if matches!(children, Children::Hidden) {
                Display::None
            } else {
                Display::Block
            },
            position: if matches!(children, Children::Absolute) {
                Position::Absolute
            } else {
                Position::Relative
            },
            flex_item_collapse: if matches!(children, Children::Collapsed) {
                FlexItemCollapse::Collapsed
            } else {
                FlexItemCollapse::Normal
            },
            writing_mode: flow.writing_mode(),
            direction: flow.direction(),
            size: size.map(PreferredSizeOf::px),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            flex_grow: FlexGrowOf::ZERO,
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
            ..NodeInputOf::default()
        },
    );
    tree
}

fn request<S: LayoutScalar>(
    flow: FlowAxes,
    run_mode: RunMode,
    demand: NamedBaselineDemand,
) -> ComputeInputOf<S> {
    let extent = NonNegativeFiniteOf::new(S::from_f64(200.0)).unwrap();
    ComputeInputOf::for_child(
        run_mode,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::splat(Some(extent.get())),
        ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
        SizingRequestOf::new(
            Size::splat(if run_mode.is_perform_layout() {
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(extent),
                    SizingOperation::Normal,
                )
            } else {
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MinContent)
            }),
            OrthogonalFallbackOf::new(
                Size::splat(Some(extent)),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
    .with_named_baseline_demand(demand)
}

fn empty_eligible_set<S: LayoutScalar>(children: Children, fixed: bool) {
    for flow in fixtures::fri05_c04_flex_all_flow_axes() {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            let tree = fixture::<S>(flow, direction, children, fixed);
            assert!(
                !tree.has_leaf_measurement(1),
                "the actual Flex container is not an opaque leaf"
            );
            assert_eq!(
                tree.child_count(1),
                usize::from(!matches!(children, Children::Empty))
            );
            for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
                for name in [BaselineName::Alphabetic, BaselineName::Central] {
                    for set in [NamedBaselineSet::First, NamedBaselineSet::Last] {
                        let demand = NamedBaselineDemand::Fact { name, set };
                        let mut session = ComputeSession::new(&tree, Vec::new());
                        let output = session
                            .compute_child(1, request(flow, run_mode, demand))
                            .unwrap();
                        // No eligible item supplies a physical port. Named absence is
                        // separately proved by the Flex collection, never by None alone.
                        assert_eq!(output.baselines(), BaselinesOf::NONE);
                        if fixed {
                            assert_eq!(
                                output.size,
                                Size::new(S::from_f64(20.0), S::from_f64(12.0))
                            );
                        }
                        assert_eq!(
                            demand.selected_fact(output.named_baselines),
                            Some((name, BaselineFactOf::Absent)),
                            "{flow:?} {direction:?} {children:?} fixed={fixed} {run_mode:?} {set:?} {name:?}",
                        );
                    }
                }
            }
        }
    }
}

fn nonempty_remains_unknown<S: LayoutScalar>() {
    for flow in fixtures::fri05_c04_flex_all_flow_axes() {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            for children in [Children::InFlow, Children::Collapsed] {
                for fixed in [false, true] {
                    let tree = fixture::<S>(flow, direction, children, fixed);
                    assert!(!tree.has_leaf_measurement(1));
                    assert!(!tree.has_leaf_measurement(2));
                    for run_mode in [RunMode::ComputeSize, RunMode::PerformLayout] {
                        for name in [BaselineName::Alphabetic, BaselineName::Central] {
                            for set in [NamedBaselineSet::First, NamedBaselineSet::Last] {
                                let demand = NamedBaselineDemand::Fact { name, set };
                                let mut session = ComputeSession::new(&tree, Vec::new());
                                let output = session
                                    .compute_child(1, request(flow, run_mode, demand))
                                    .unwrap();
                                assert_eq!(
                                    demand.selected_fact(output.named_baselines),
                                    Some((name, BaselineFactOf::Unknown)),
                                    "a real in-flow item is not proof of named absence: {flow:?} {direction:?} {children:?} fixed={fixed} {run_mode:?} {set:?} {name:?}",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn known_size_without_named_demand<S: LayoutScalar>() {
    for flow in fixtures::fri05_c04_flex_all_flow_axes() {
        for direction in [FlexDirection::Row, FlexDirection::Column] {
            for children in [
                Children::Empty,
                Children::Absolute,
                Children::Hidden,
                Children::InFlow,
                Children::Collapsed,
            ] {
                let tree = fixture::<S>(flow, direction, children, true);
                let mut session = ComputeSession::new(&tree, Vec::new());
                let output = session
                    .compute_child(
                        1,
                        request(flow, RunMode::ComputeSize, NamedBaselineDemand::None),
                    )
                    .unwrap();
                assert_eq!(output.size, Size::new(S::from_f64(20.0), S::from_f64(12.0)));
                assert_eq!(output.baselines(), BaselinesOf::NONE);
                assert_eq!(
                    output.named_baselines,
                    NamedBaselinesOf::unknown(),
                    "the size-only shortcut supplies no unrequested named proof: {flow:?} {direction:?} {children:?}",
                );
            }
        }
    }
}

fn physical_last_zero_is_preserved<S: LayoutScalar>() {
    // A single real empty Block occupies exactly the Flex border box. Its
    // vertical-rl line-under edge is physical left0; First and Last select that
    // same item. A supplied/synthesized zero port must remain Some(0), not None.
    let flow = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    let tree = fixture::<S>(flow, FlexDirection::Row, Children::InFlow, true);
    for demand in [
        NamedBaselineDemand::None,
        NamedBaselineDemand::Fact {
            name: BaselineName::Central,
            set: NamedBaselineSet::Last,
        },
    ] {
        let mut session = ComputeSession::new(&tree, Vec::new());
        let output = session
            .compute_child(1, request(flow, RunMode::PerformLayout, demand))
            .unwrap();
        assert_eq!(output.size, Size::new(S::from_f64(20.0), S::from_f64(12.0)));
        assert_eq!(output.first_baselines, Point::new(Some(S::ZERO), None));
        assert_eq!(output.last_baselines, Point::new(Some(S::ZERO), None));
        assert_eq!(output.named_baselines, NamedBaselinesOf::unknown());
    }
    // The generic output carrier by itself cannot certify a named family.
    let neutral =
        ComputeOutputOf::<S>::from_outer_size(Size::new(S::from_f64(20.0), S::from_f64(12.0)));
    assert_eq!(neutral.baselines(), BaselinesOf::NONE);
    assert_eq!(neutral.named_baselines, NamedBaselinesOf::unknown());
}

#[test]
fn empty_flex_named_absence_f32() {
    empty_eligible_set::<f32>(Children::Empty, false);
}
#[test]
fn empty_flex_named_absence_f64() {
    empty_eligible_set::<f64>(Children::Empty, false);
}
#[test]
fn definite_empty_flex_named_absence_f32() {
    empty_eligible_set::<f32>(Children::Empty, true);
}
#[test]
fn definite_empty_flex_named_absence_f64() {
    empty_eligible_set::<f64>(Children::Empty, true);
}
#[test]
fn absolute_only_flex_named_absence_f32() {
    empty_eligible_set::<f32>(Children::Absolute, false);
}
#[test]
fn absolute_only_flex_named_absence_f64() {
    empty_eligible_set::<f64>(Children::Absolute, false);
}
#[test]
fn hidden_only_flex_named_absence_f32() {
    empty_eligible_set::<f32>(Children::Hidden, false);
}
#[test]
fn hidden_only_flex_named_absence_f64() {
    empty_eligible_set::<f64>(Children::Hidden, false);
}
#[test]
fn nonempty_flex_named_export_stays_unknown_f32() {
    nonempty_remains_unknown::<f32>();
}
#[test]
fn nonempty_flex_named_export_stays_unknown_f64() {
    nonempty_remains_unknown::<f64>();
}
#[test]
fn known_size_flex_without_named_demand_stays_unknown_f32() {
    known_size_without_named_demand::<f32>();
}
#[test]
fn known_size_flex_without_named_demand_stays_unknown_f64() {
    known_size_without_named_demand::<f64>();
}
#[test]
fn flex_physical_last_zero_and_generic_unknown_f32() {
    physical_last_zero_is_preserved::<f32>();
}
#[test]
fn flex_physical_last_zero_and_generic_unknown_f64() {
    physical_last_zero_is_preserved::<f64>();
}

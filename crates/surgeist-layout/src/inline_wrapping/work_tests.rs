//! Real numeric/flex geometry precedes invocation-scoped comparison bounds.

use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{
    AlignContent, AlignItems, AtomicInlineParticipation, AvailableOf, BidiLevel,
    CompletedLayoutBatchOf, Display, FlexDirection, FlexGrowOf, FlexShrinkOf, FlexWrap,
    InlineBreakOpportunity, LayoutOutputEntryOf, LayoutRootRequestOf, LayoutScalar,
    LengthPercentageOf, NodeInputOf, Point, PreferredSizeOf, Size, SourceIndex, TextWrapStyle,
    WrapEdge, compute_layout,
};
use std::{cell::Cell, marker::PhantomData, rc::Rc};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Work {
    subsets: usize,
    offers: usize,
}

thread_local! {
    static WORK: Cell<Option<Work>> = const { Cell::new(None) };
}

pub(super) fn record_subset() {
    WORK.with(|slot| {
        if let Some(mut work) = slot.get() {
            work.subsets += 1;
            slot.set(Some(work));
        }
    });
}

pub(super) fn record_offer() {
    WORK.with(|slot| {
        if let Some(mut work) = slot.get() {
            work.offers += 1;
            slot.set(Some(work));
        }
    });
}

// Each synchronous public computation owns its recorder. Nested observation
// restores its caller, including on unwind, and cannot move to another thread.
struct Observation {
    prior: Option<Work>,
    thread: PhantomData<Rc<()>>,
}
impl Observation {
    fn enter() -> Self {
        Self {
            prior: WORK.with(|slot| slot.replace(Some(Work::default()))),
            thread: PhantomData,
        }
    }
    fn counts(&self) -> Work {
        WORK.with(|slot| slot.get().unwrap())
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        WORK.with(|slot| slot.set(self.prior));
    }
}

fn observe<T>(f: impl FnOnce() -> T) -> (T, Work) {
    let scope = Observation::enter();
    let result = f();
    let work = scope.counts();
    drop(scope);
    (result, work)
}

#[derive(Clone, Copy, Debug)]
enum Profile {
    Numeric,
    Flex,
}

fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn tree<S: LayoutScalar>(profile: Profile, count: usize, avoid: bool) -> PublicLayoutTreeOf<S> {
    let width = 10.0 * (count - 1) as f64;
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, (1..=count).map(|i| u32::try_from(i).unwrap()))
        .style(
            0,
            NodeInputOf {
                display: match profile {
                    Profile::Numeric => Display::Block,
                    Profile::Flex => Display::Flex,
                },
                size: Size::new(px(width), PreferredSizeOf::AUTO),
                text_wrap_style: TextWrapStyle::Auto,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: Some(AlignItems::FlexStart),
                align_content: Some(AlignContent::FlexStart),
                justify_content: Some(AlignContent::FlexStart),
                ..NodeInputOf::default()
            },
        );
    for index in 1..=count {
        let node = u32::try_from(index).unwrap();
        let mut input = NodeInputOf {
            display: match profile {
                Profile::Numeric => Display::InlineBlock,
                Profile::Flex => Display::Block,
            },
            size: Size::new(px(10.0), px(10.0)),
            flex_grow: FlexGrowOf::ZERO,
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
            atomic_inline_participation: matches!(profile, Profile::Numeric).then_some(
                AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                ),
            ),
            ..NodeInputOf::default()
        };
        if avoid && index < count {
            input.wrap_edges.after = match profile {
                Profile::Numeric => WrapEdge::AvoidLine,
                Profile::Flex => WrapEdge::AvoidFlex,
            };
        }
        // Before and the terminal After remain genuine neutral Auto. There is
        // one independent actual avoided after-side per interior cut.
        tree = tree.children(node, []).style(node, input);
    }
    tree
}

fn assert_entries<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], count: usize) {
    assert_eq!(entries.len(), count + 1);
    let root = entries.iter().find(|e| e.node() == 0).unwrap().output();
    assert_eq!(root.location, Point::ZERO);
    assert_eq!(
        root.size,
        Size::new(S::from_f64(10.0 * (count - 1) as f64), S::from_f64(20.0))
    );
    for index in 1..=count {
        let node = u32::try_from(index).unwrap();
        let matches = entries
            .iter()
            .filter(|e| e.node() == node)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "one actual output per real node");
        let output = matches[0].output();
        let (x, y) = if index == count {
            (0.0, 10.0)
        } else {
            (10.0 * (index - 1) as f64, 0.0)
        };
        assert_eq!(output.location, Point::new(S::from_f64(x), S::from_f64(y)));
        assert_eq!(output.size, Size::splat(S::from_f64(10.0)));
        assert_eq!(output.source_index, SourceIndex::new(index - 1));
    }
}

fn run<S: LayoutScalar>(profile: Profile, count: usize, avoid: bool) {
    let tree = tree::<S>(profile, count, avoid);
    let (batch, work): (CompletedLayoutBatchOf<u32, S>, Work) = observe(|| {
        compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(10.0 * (count - 1) as f64)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap()
    });
    // Distinct singleton avoided-edge sets are incomparable. Ordinary latest
    // fitting cut wins: n-1 real boxes on row0 and the last real box on row1.
    // Qualify complete authentic output before any work assertion.
    assert_entries(batch.unrounded_entries(), count);
    assert_entries(batch.final_entries(), count);
    println!(
        "profile={profile:?} scalar={} n={count} avoid={avoid} offers={} subset_comparisons={}",
        std::any::type_name::<S>(),
        work.offers,
        work.subsets
    );
    if avoid {
        assert!(
            work.offers > 0,
            "actual avoided candidate selection must execute"
        );
    }
    // For these nonempty pairwise-disjoint edge sets, disjointness proves both
    // subset directions false without scanning retained candidates. Allow two
    // subset calls per real offer for incidental inside/equivalent controls.
    // Neutral numeric same-class inside offers also need at most two; neutral
    // flex may bypass the frontier entirely (0offers/0comparisons is valid).
    assert!(
        work.subsets <= 2 * work.offers,
        "disjoint/neutral fixture must not scan retained frontier: profile={profile:?} scalar={} n={count} avoid={avoid} work={work:?}",
        std::any::type_name::<S>()
    );
}

#[test]
fn numeric_disjoint_edges_128_f32() {
    run::<f32>(Profile::Numeric, 128, true);
}
#[test]
fn numeric_disjoint_edges_256_f32() {
    run::<f32>(Profile::Numeric, 256, true);
}
#[test]
fn numeric_disjoint_edges_128_f64() {
    run::<f64>(Profile::Numeric, 128, true);
}
#[test]
fn numeric_disjoint_edges_256_f64() {
    run::<f64>(Profile::Numeric, 256, true);
}
#[test]
fn flex_disjoint_edges_128_f32() {
    run::<f32>(Profile::Flex, 128, true);
}
#[test]
fn flex_disjoint_edges_256_f32() {
    run::<f32>(Profile::Flex, 256, true);
}
#[test]
fn flex_disjoint_edges_128_f64() {
    run::<f64>(Profile::Flex, 128, true);
}
#[test]
fn flex_disjoint_edges_256_f64() {
    run::<f64>(Profile::Flex, 256, true);
}
#[test]
fn numeric_neutral_edges_128_f32() {
    run::<f32>(Profile::Numeric, 128, false);
}
#[test]
fn numeric_neutral_edges_256_f32() {
    run::<f32>(Profile::Numeric, 256, false);
}
#[test]
fn numeric_neutral_edges_128_f64() {
    run::<f64>(Profile::Numeric, 128, false);
}
#[test]
fn numeric_neutral_edges_256_f64() {
    run::<f64>(Profile::Numeric, 256, false);
}
#[test]
fn flex_neutral_edges_128_f32() {
    run::<f32>(Profile::Flex, 128, false);
}
#[test]
fn flex_neutral_edges_256_f32() {
    run::<f32>(Profile::Flex, 256, false);
}
#[test]
fn flex_neutral_edges_128_f64() {
    run::<f64>(Profile::Flex, 128, false);
}
#[test]
fn flex_neutral_edges_256_f64() {
    run::<f64>(Profile::Flex, 256, false);
}

#[test]
fn observation_restores_outer_invocation_after_nested_unwind() {
    assert!(WORK.with(|slot| slot.get().is_none()));
    let outer = Observation::enter();
    record_offer();
    assert!(
        std::panic::catch_unwind(|| {
            let _inner = Observation::enter();
            record_subset();
            panic!("intentional observer cleanup control");
        })
        .is_err()
    );
    assert_eq!(
        outer.counts(),
        Work {
            subsets: 0,
            offers: 1
        }
    );
    drop(outer);
    assert!(WORK.with(|slot| slot.get().is_none()));
}

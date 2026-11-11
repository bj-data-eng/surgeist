use super::contracts::{Round, UnroundedInlineFragmentState};
use crate::error::{
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInternalInvariant, LayoutOperation,
    LayoutResultOf,
};
use crate::output::{
    AtomicReadbackPhase, GeometryStateOf, LayoutOutputEntryOf, NormalContentOf, NormalGeometryOf,
};
use crate::scalar::round_layout_coordinate;
use crate::scroll::rounding::rebuild_rounded_canonical_scroll_geometry;
use crate::{LayoutScalar, Point, Traverse};

pub(crate) fn round_layout<Tree, M>(
    tree: &mut Tree,
    root: <Tree as Traverse>::Node,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    round_layout_inner(
        tree,
        root,
        RoundingFrames {
            visual: Point::ZERO,
            normal: Point::ZERO,
            named: NamedRoundingContext::ROOT,
            fragment_coordinates: false,
        },
    )
}

#[derive(Clone, Copy)]
struct RoundingFrames<S: LayoutScalar> {
    visual: Point<S>,
    normal: Point<S>,
    named: NamedRoundingContext,
    fragment_coordinates: bool,
}

/// Coordinate custody for another actual subtree or an overflow carrier.
#[derive(Clone, Copy)]
pub(crate) struct FragmentAnchor<S: LayoutScalar>(RoundingFrames<S>);
impl<S: LayoutScalar> FragmentAnchor<S> {
    pub(crate) fn external(origin: Point<S>) -> Self {
        let rounded = Point::new(
            round_layout_coordinate(origin.x),
            round_layout_coordinate(origin.y),
        );
        Self(RoundingFrames {
            visual: origin,
            normal: origin,
            named: NamedRoundingContext::ROOT.descend(origin, rounded),
            fragment_coordinates: true,
        })
    }
    pub(crate) fn overflow(
        self,
        geometry: crate::OverflowFragmentGeometryOf<S>,
        normal: Point<S>,
    ) -> Result<(crate::OverflowFragmentGeometryOf<S>, Self), crate::ScrollRectErrorOf<S>> {
        let raw = geometry.origin;
        let cumulative = Point::new(self.0.visual.x + raw.x, self.0.visual.y + raw.y);
        let origin = Point::new(
            round_layout_coordinate(cumulative.x) - round_layout_coordinate(self.0.visual.x),
            round_layout_coordinate(cumulative.y) - round_layout_coordinate(self.0.visual.y),
        );
        let content_bounds = crate::scroll::rounding::round_canonical_source_rect(
            geometry.content_bounds,
            cumulative,
        )?;
        let anchor = Self(RoundingFrames {
            visual: cumulative,
            normal: Point::new(self.0.normal.x + normal.x, self.0.normal.y + normal.y),
            named: self.0.named.descend(raw, origin),
            fragment_coordinates: true,
        });
        Ok((
            crate::OverflowFragmentGeometryOf {
                origin,
                content_bounds,
            },
            anchor,
        ))
    }
}

pub(crate) fn round_fragment_subtree<Tree, M>(
    tree: &mut Tree,
    root: Tree::Node,
    anchor: FragmentAnchor<Tree::Scalar>,
) -> LayoutResultOf<Tree::Node, (), Tree::Scalar, M>
where
    Tree: Round<M>,
{
    round_layout_inner(tree, root, anchor.0)
}

fn round_layout_inner<Tree, M>(
    tree: &mut Tree,
    node: <Tree as Traverse>::Node,
    ancestors: RoundingFrames<Tree::Scalar>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), <Tree as Traverse>::Scalar, M>
where
    Tree: Round<M>,
{
    let entry = tree.unrounded(node)?;
    if entry.node() != node || !entry.geometry_state().is_unrounded() {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::RoundingFinalization,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidOutputGeometryPhase,
            ),
        ));
    }
    let unrounded = entry.output();
    if let GeometryStateOf::CanonicalAtomic {
        normal_origin,
        visual_origin: origin,
        readback: AtomicReadbackPhase::Pending,
        ..
    } = entry.geometry_state()
    {
        // This actual atomic allocation was finalized against the same line
        // anchor. Reapplying independent endpoint snapping would invalidate it.
        // This owning allocation is already canonical. Only its actual CSS
        // pixel phase influences descendant/local scroll endpoint rounding;
        // retaining the outer f64 translation avoids narrowing a large origin.
        let scalar_phase = Point::new(
            Tree::Scalar::from_f64(origin.x - origin.x.floor()),
            Tree::Scalar::from_f64(origin.y - origin.y.floor()),
        );
        let mut layout = unrounded;
        let named_context = NamedRoundingContext::canonical(origin);
        layout.named_baselines = finalize_named_baselines(unrounded.named_baselines, named_context)
            .map_err(|kind| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    kind,
                )
            })?;
        layout.scroll_geometry = unrounded
            .scroll_geometry
            .map(|geometry| rebuild_rounded_canonical_scroll_geometry(geometry, scalar_phase))
            .transpose()
            .map_err(|_| {
                LayoutErrorOf::new(
                    LayoutErrorSiteOf::Node(node),
                    LayoutOperation::RoundingFinalization,
                    LayoutErrorKindOf::InternalInvariant(
                        LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                    ),
                )
            })?;
        let finalized = entry.finalized_atomic(layout).ok_or_else(|| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidOutputGeometryPhase,
                ),
            )
        })?;
        tree.set_final(finalized);
        let child_frame = RoundingFrames {
            visual: scalar_phase,
            normal: Point::new(
                Tree::Scalar::from_f64(normal_origin.x - normal_origin.x.floor()),
                Tree::Scalar::from_f64(normal_origin.y - normal_origin.y.floor()),
            ),
            named: named_context,
            fragment_coordinates: ancestors.fragment_coordinates,
        };
        tree.record_fragment_anchor(node, FragmentAnchor(child_frame));
        for index in 0..tree.child_count(node) {
            round_layout_inner(tree, tree.child(node, index), child_frame)?;
        }
        return Ok(());
    }
    let GeometryStateOf::OrdinaryUnrounded { measured_normal } = entry.geometry_state() else {
        return Err(LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::RoundingFinalization,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidOutputGeometryPhase,
            ),
        ));
    };
    if tree.is_prepared_text(node) {
        // The new Text path supplies geometry from one canonical line anchor.
        // Its local metrics and fragments must never be independently snapped.
        let fragments = match tree.unrounded_inline_fragment_state(node) {
            UnroundedInlineFragmentState::Present(fragments) => fragments.to_vec(),
            UnroundedInlineFragmentState::Absent => Vec::new(),
        };
        tree.set_final(LayoutOutputEntryOf::ordinary_canonical(
            node,
            measured_normal,
            unrounded,
        ));
        tree.set_final_inline_fragments(node, fragments.clone(), fragments);
        return Ok(());
    }
    let mut layout = unrounded;
    let cumulative_x = ancestors.visual.x + unrounded.location.x;
    let cumulative_y = ancestors.visual.y + unrounded.location.y;
    let normal_cumulative = Point::new(
        ancestors.normal.x + measured_normal.location.x,
        ancestors.normal.y + measured_normal.location.y,
    );
    let normal = round_normal_geometry(measured_normal, normal_cumulative).map_err(|_| {
        LayoutErrorOf::new(
            LayoutErrorSiteOf::Node(node),
            LayoutOperation::RoundingFinalization,
            LayoutErrorKindOf::InternalInvariant(
                LayoutInternalInvariant::InvalidRoundedScrollGeometry,
            ),
        )
    })?;

    layout.location.x = if ancestors.fragment_coordinates {
        round_layout_coordinate(cumulative_x) - round_layout_coordinate(ancestors.visual.x)
    } else {
        round_layout_coordinate(unrounded.location.x)
    };
    layout.location.y = if ancestors.fragment_coordinates {
        round_layout_coordinate(cumulative_y) - round_layout_coordinate(ancestors.visual.y)
    } else {
        round_layout_coordinate(unrounded.location.y)
    };
    let named_context = ancestors.named.descend(unrounded.location, layout.location);
    layout.named_baselines = finalize_named_baselines(unrounded.named_baselines, named_context)
        .map_err(|kind| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                kind,
            )
        })?;
    layout.size.width = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x);
    layout.size.height = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y);
    layout.content_size.width =
        round_layout_coordinate(cumulative_x + unrounded.content_size.width)
            - round_layout_coordinate(cumulative_x);
    layout.content_size.height =
        round_layout_coordinate(cumulative_y + unrounded.content_size.height)
            - round_layout_coordinate(cumulative_y);
    layout.border.left = round_layout_coordinate(cumulative_x + unrounded.border.left)
        - round_layout_coordinate(cumulative_x);
    layout.border.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round_layout_coordinate(cumulative_y + unrounded.border.top)
        - round_layout_coordinate(cumulative_y);
    layout.border.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round_layout_coordinate(cumulative_x + unrounded.padding.left)
        - round_layout_coordinate(cumulative_x);
    layout.padding.right = round_layout_coordinate(cumulative_x + unrounded.size.width)
        - round_layout_coordinate(cumulative_x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round_layout_coordinate(cumulative_y + unrounded.padding.top)
        - round_layout_coordinate(cumulative_y);
    layout.padding.bottom = round_layout_coordinate(cumulative_y + unrounded.size.height)
        - round_layout_coordinate(cumulative_y + unrounded.size.height - unrounded.padding.bottom);
    let scroll_geometry = unrounded
        .scroll_geometry
        .map(|geometry| {
            rebuild_rounded_canonical_scroll_geometry(
                geometry,
                Point::new(cumulative_x, cumulative_y),
            )
        })
        .transpose()
        .map_err(|_| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::RoundingFinalization,
                LayoutErrorKindOf::InternalInvariant(
                    LayoutInternalInvariant::InvalidRoundedScrollGeometry,
                ),
            )
        })?;
    layout = layout.with_scroll_geometry(scroll_geometry);

    let fragment_phases = match tree.unrounded_inline_fragment_state(node) {
        UnroundedInlineFragmentState::Absent => None,
        UnroundedInlineFragmentState::Present(fragments) => {
            let unrounded_fragments = fragments.to_vec();
            let final_fragments = fragments.to_vec();
            Some((unrounded_fragments, final_fragments))
        }
    };

    tree.set_final(LayoutOutputEntryOf::ordinary_canonical(
        node, normal, layout,
    ));
    if let Some((unrounded_fragments, final_fragments)) = fragment_phases {
        tree.set_final_inline_fragments(node, unrounded_fragments, final_fragments);
    }

    let child_frame = RoundingFrames {
        visual: Point::new(cumulative_x, cumulative_y),
        normal: normal_cumulative,
        named: named_context,
        fragment_coordinates: ancestors.fragment_coordinates,
    };
    tree.record_fragment_anchor(node, FragmentAnchor(child_frame));
    for index in 0..tree.child_count(node) {
        round_layout_inner(tree, tree.child(node, index), child_frame)?;
    }
    Ok(())
}

fn round_normal_geometry<S: LayoutScalar>(
    measured: NormalGeometryOf<S>,
    origin: Point<S>,
) -> Result<NormalGeometryOf<S>, crate::ScrollRectErrorOf<S>> {
    let extent = |base, size| round_layout_coordinate(base + size) - round_layout_coordinate(base);
    Ok(NormalGeometryOf {
        location: Point::new(
            round_layout_coordinate(measured.location.x),
            round_layout_coordinate(measured.location.y),
        ),
        size: crate::Size::new(
            extent(origin.x, measured.size.width),
            extent(origin.y, measured.size.height),
        ),
        content: match measured.content {
            NormalContentOf::Box(content) => {
                let local = content.origin();
                let size = content.size();
                let x = origin.x + local.x;
                let y = origin.y + local.y;
                NormalContentOf::Box(crate::ScrollRectOf::try_new(
                    Point::new(
                        round_layout_coordinate(x) - round_layout_coordinate(origin.x),
                        round_layout_coordinate(y) - round_layout_coordinate(origin.y),
                    ),
                    crate::Size::new(extent(x, size.width), extent(y, size.height)),
                )?)
            }
            NormalContentOf::Metadata => NormalContentOf::Metadata,
            NormalContentOf::Hidden => NormalContentOf::Hidden,
        },
    })
}

/// Named readback tracks the actual raw and published border frames without
/// adding a small metric to a huge global origin. `integer_shift` is
/// floor(raw global origin) - floor(published global origin); the two phases
/// complete the same raw-to-published displacement. Ordinary location/size
/// rounding remains unchanged. Canonical Text atomics retain their real origin.
#[derive(Clone, Copy)]
struct NamedRoundingContext {
    raw_phase: Point<f64>,
    published_phase: Point<f64>,
    integer_shift: Point<f64>,
}

impl NamedRoundingContext {
    const ROOT: Self = Self {
        raw_phase: Point::ZERO,
        published_phase: Point::ZERO,
        integer_shift: Point::ZERO,
    };

    fn canonical(origin: Point<f64>) -> Self {
        let phase = Point::new(origin.x - origin.x.floor(), origin.y - origin.y.floor());
        Self {
            raw_phase: phase,
            published_phase: phase,
            integer_shift: Point::ZERO,
        }
    }

    fn descend<S: LayoutScalar>(self, raw: Point<S>, published: Point<S>) -> Self {
        let axis = |phase: f64, shift: f64, raw: S, published: S| {
            let raw = raw.to_f64();
            let combined_phase = phase + (raw - raw.floor());
            (
                combined_phase - combined_phase.floor(),
                shift + (raw.floor() - published.to_f64()) + combined_phase.floor(),
            )
        };
        let (x, dx) = axis(self.raw_phase.x, self.integer_shift.x, raw.x, published.x);
        let (y, dy) = axis(self.raw_phase.y, self.integer_shift.y, raw.y, published.y);
        Self {
            raw_phase: Point::new(x, y),
            integer_shift: Point::new(dx, dy),
            published_phase: self.published_phase,
        }
    }
}

fn finalize_named_baselines<S: LayoutScalar, M>(
    facts: crate::NamedBaselinesOf<S>,
    context: NamedRoundingContext,
) -> Result<crate::NamedBaselinesOf<S>, LayoutErrorKindOf<S, M>> {
    let finalize = |fact: crate::BaselineFactOf<S>| {
        let crate::BaselineFactOf::Available(coordinate) = fact else {
            return Ok(fact);
        };
        let (phase, published_phase, shift) = match coordinate.axis() {
            crate::PhysicalAxis::Horizontal => (
                context.raw_phase.x,
                context.published_phase.x,
                context.integer_shift.x,
            ),
            crate::PhysicalAxis::Vertical => (
                context.raw_phase.y,
                context.published_phase.y,
                context.integer_shift.y,
            ),
        };
        let offset = coordinate.offset().to_f64();
        let whole = offset.floor();
        let correction = shift + (phase + (offset - whole) + 0.5).floor() - published_phase;
        let value = whole + correction;
        if !value.is_finite() {
            return Err(LayoutErrorKindOf::InvalidInput(
                crate::LayoutInvalidInputOf::InvalidNumeric {
                    value: S::from_f64(value),
                },
            ));
        }
        let converted = S::from_f64(value);
        if !converted.is_finite()
            || value != 0.0 && converted == S::ZERO
            || correction != 0.0 && (value == whole || converted.to_f64() == whole)
        {
            return Err(LayoutErrorKindOf::InvalidInput(
                crate::LayoutInvalidInputOf::InlineBaseline(
                    crate::InlineBaselineError::UnrepresentableCoordinate,
                ),
            ));
        }
        crate::BaselineCoordinateOf::try_new(coordinate.axis(), converted)
            .map(crate::BaselineFactOf::Available)
            .map_err(|crate::FiniteScalarErrorOf::NonFinite { value }| {
                LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::InvalidNumeric {
                    value,
                })
            })
    };
    let mut result = crate::NamedBaselinesOf::unknown();
    for name in [
        crate::BaselineName::Alphabetic,
        crate::BaselineName::Central,
    ] {
        let pair = facts.get(name);
        result = result.with(
            name,
            crate::BaselinePairOf::new(finalize(pair.first())?, finalize(pair.last())?),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_deep_rounding<S: LayoutScalar>(fragment: bool) {
        use crate::test_support::layout_tree::OracleTreeOf;
        use crate::{NodeOutputOf, Size};

        const NODES: u32 = 1025;
        let mut tree = OracleTreeOf::<S>::new();
        for node in 0..NODES {
            tree = tree.unrounded(
                node,
                NodeOutputOf {
                    location: Point::new(S::from_f64(0.25), S::ZERO),
                    size: Size::splat(S::from_f64(10.25)),
                    ..NodeOutputOf::default()
                },
            );
            if node + 1 < NODES {
                tree = tree.children(node, [node + 1]);
            }
        }
        if fragment {
            let anchor = FragmentAnchor::external(Point::new(S::from_f64(0.25), S::ZERO));
            round_fragment_subtree(&mut tree.borrow_compute(), 0, anchor).unwrap();
        } else {
            round_layout(&mut tree.borrow_compute(), 0).unwrap();
        }

        let (x, width) = if fragment { (1.0, 10.0) } else { (0.0, 11.0) };
        for node in [0, NODES - 1] {
            let rounded = tree.final_layout(node).expect("node was rounded");
            assert_eq!(rounded.location, Point::new(S::from_f64(x), S::ZERO));
            assert_eq!(
                rounded.size,
                Size::new(S::from_f64(width), S::from_f64(10.0))
            );
        }
    }

    #[test]
    fn deep_ordinary_and_fragment_rounding_return_on_bounded_stack() {
        const CHILD: &str = "SURGEIST_TEST_ROUNDING_STACK_CHILD";
        const TEST: &str =
            "engine::rounding::tests::deep_ordinary_and_fragment_rounding_return_on_bounded_stack";
        const STACK_BYTES: usize = 2 * 1024 * 1024;

        if let Ok(case) = std::env::var(CHILD) {
            std::thread::Builder::new()
                .name("bounded-rounding-stack".into())
                .stack_size(STACK_BYTES)
                .spawn(move || match case.as_str() {
                    "ordinary-f32" => assert_deep_rounding::<f32>(false),
                    "ordinary-f64" => assert_deep_rounding::<f64>(false),
                    "fragment-f32" => assert_deep_rounding::<f32>(true),
                    "fragment-f64" => assert_deep_rounding::<f64>(true),
                    _ => panic!("unknown rounding case {case}"),
                })
                .unwrap()
                .join()
                .unwrap();
            return;
        }

        let executable = std::env::current_exe().unwrap();
        let mut failures = Vec::new();
        for case in [
            "ordinary-f32",
            "ordinary-f64",
            "fragment-f32",
            "fragment-f64",
        ] {
            let result = std::process::Command::new(&executable)
                .args([TEST, "--exact", "--nocapture"])
                .env(CHILD, case)
                .output()
                .unwrap();
            let stdout = String::from_utf8_lossy(&result.stdout);
            if !result.status.success() || !stdout.contains("1 passed") {
                failures.push(format!(
                    "{case} failed on a {STACK_BYTES}-byte stack, child {}\n{stdout}\n{}",
                    result.status,
                    String::from_utf8_lossy(&result.stderr),
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    fn assert_layout_round_boundaries<S: LayoutScalar>() {
        for (value, expected) in [
            (-2.0, -2.0),
            (-1.51, -2.0),
            (-1.5, -1.0),
            (-1.49, -1.0),
            (-0.51, -1.0),
            (-0.5, 0.0),
            (-0.49, 0.0),
            (0.0, 0.0),
            (0.49, 0.0),
            (0.5, 1.0),
            (0.51, 1.0),
            (1.49, 1.0),
            (1.5, 2.0),
            (1.51, 2.0),
            (2.0, 2.0),
            (-1_048_576.5, -1_048_576.0),
            (1_048_576.5, 1_048_577.0),
        ] {
            assert_eq!(
                round_layout_coordinate(S::from_f64(value)),
                S::from_f64(expected)
            );
        }
    }

    #[test]
    fn nearest_css_pixel_scalar_boundaries_keep_half_open_ties() {
        assert_layout_round_boundaries::<f32>();
        assert_layout_round_boundaries::<f64>();
    }

    #[test]
    fn actual_text_coordinate_narrowing_refuses_overflow_and_nonzero_underflow() {
        use crate::text_layout::{fit::FitError, run::number};
        assert!(matches!(
            number::<f32>(f64::MAX),
            Err(FitError::Numeric(
                _,
                crate::LayoutTextNumericError::OutOfRange
            ))
        ));
        assert!(matches!(
            number::<f32>(f64::from_bits(1)),
            Err(FitError::Numeric(
                _,
                crate::LayoutTextNumericError::NonZeroLost
            ))
        ));
        assert_eq!(
            number::<f64>(f64::from_bits(1)).ok(),
            Some(f64::from_bits(1))
        );
    }
}

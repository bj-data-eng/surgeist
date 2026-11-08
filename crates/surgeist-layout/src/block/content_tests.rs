use super::*;
use crate::engine::ComputeSession;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn input<S: LayoutScalar>(axes: crate::FlowAxes, inline: f64) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::PerformLayout,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        axes.physical_size(LogicalSizeOf::new(Some(S::from_f64(inline)), None)),
        axes.physical_size(LogicalSizeOf::new(Some(S::from_f64(100.0)), None)),
        ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
        crate::SizingRequestOf::new(
            Size::splat(crate::AxisSizingRequestOf::new(
                crate::AvailableSpaceOf::Indefinite,
                crate::SizingOperation::Normal,
            )),
            crate::OrthogonalFallbackOf::new(
                Size::NONE,
                crate::NearestScrollportOf::Absent,
                Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    )
}
fn content<S: LayoutScalar>() {
    for vertical in [false, true] {
        let axes = crate::FlowAxes::new(
            if vertical {
                crate::WritingMode::VerticalRl
            } else {
                crate::WritingMode::HorizontalTb
            },
            Direction::Ltr,
        );
        let mut owner = crate::NodeInputOf::<S> {
            display: crate::Display::Block,
            writing_mode: axes.writing_mode(),
            ..Default::default()
        };
        let maximum =
            crate::MaxSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(20.0)).unwrap());
        if vertical {
            owner.max_size.width = maximum;
        } else {
            owner.max_size.height = maximum;
        }
        let child = crate::NodeInputOf::<S> {
            display: crate::Display::Block,
            writing_mode: axes.writing_mode(),
            size: axes.physical_size(LogicalSizeOf::new(
                crate::PreferredSizeOf::px(S::from_f64(40.0)),
                crate::PreferredSizeOf::px(S::from_f64(30.0)),
            )),
            ..Default::default()
        };
        let tree = PublicLayoutTreeOf::new()
            .style(0, owner)
            .style(1, child)
            .children(0, [1]);
        let mut session = ComputeSession::new(&tree, Vec::new());
        let context = prepare(
            &mut session,
            0,
            input(axes, 40.0),
            crate::LayoutTree::node_input(&tree, 0),
        )
        .unwrap();
        assert_eq!(
            context.limits,
            Some(Limits {
                minimum: S::ZERO,
                maximum: S::from_f64(20.0)
            })
        );
        assert_eq!(context.inline_outer, S::from_f64(40.0));
        let receipt = produce(
            &mut session,
            0,
            &context,
            Projection {
                block_reference: S::ZERO,
                content_origin: S::ZERO,
            },
            None,
        )
        .unwrap();
        assert_eq!(receipt.block_extent, S::from_f64(30.0));
        assert!(!receipt.through);
        assert_eq!(receipt.start.resolve(), S::ZERO);
        assert_eq!(receipt.end.resolve(), S::ZERO);
        assert!(session.staged_unrounded(0).is_err());
        let actual = session.staged_unrounded(1).unwrap().output();
        assert_eq!(
            actual.size,
            axes.physical_size(LogicalSizeOf::new(S::from_f64(40.0), S::from_f64(30.0)))
        );
        let origin = if vertical {
            Point::new(S::from_f64(-30.0), S::ZERO)
        } else {
            Point::ZERO
        };
        assert_eq!(actual.location, origin);
        let bounds = receipt.bounds.unwrap();
        assert_eq!(bounds.origin(), origin);
        assert_eq!(bounds.size(), actual.size);
    }
    let axes = crate::FlowAxes::new(crate::WritingMode::HorizontalTb, Direction::Ltr);
    let mut owner = crate::NodeInputOf::<S> {
        display: crate::Display::Block,
        ..Default::default()
    };
    owner.min_size.height =
        crate::MinSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(90.0)).unwrap());
    owner.max_size.height =
        crate::MaxSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(70.0)).unwrap());
    let child = crate::NodeInputOf::<S> {
        display: crate::Display::Block,
        size: Size::new(
            crate::PreferredSizeOf::AUTO,
            crate::PreferredSizeOf::value(
                crate::LengthPercentageOf::from_percent_fraction(S::from_f64(0.5)).unwrap(),
            ),
        ),
        ..Default::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .style(0, owner)
        .style(1, child)
        .measure(1, Size::new(S::from_f64(20.0), S::from_f64(10.0)))
        .children(0, [1]);
    let mut session = ComputeSession::new(&tree, Vec::new());
    let context = prepare(
        &mut session,
        0,
        input(axes, 100.0),
        crate::LayoutTree::node_input(&tree, 0),
    )
    .unwrap();
    assert_eq!(
        context.limits,
        Some(Limits {
            minimum: S::from_f64(90.0),
            maximum: S::from_f64(90.0)
        })
    );
    let receipt = produce(
        &mut session,
        0,
        &context,
        Projection {
            block_reference: S::ZERO,
            content_origin: S::ZERO,
        },
        None,
    )
    .unwrap();
    assert_eq!(receipt.block_extent, S::from_f64(45.0));
    assert_eq!(
        session.staged_unrounded(1).unwrap().output().size.height,
        S::from_f64(45.0)
    );
    assert!(session.staged_unrounded(0).is_err());
}
#[test]
fn scope_content_preserves_real_boxes_and_percentage_authority_f32() {
    content::<f32>();
}
#[test]
fn scope_content_preserves_real_boxes_and_percentage_authority_f64() {
    content::<f64>();
}

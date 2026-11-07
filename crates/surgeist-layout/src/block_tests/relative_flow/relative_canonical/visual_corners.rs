//! Independent visual-corner oracles alongside the committed normal-flow REDs.
//! Mount as a private child of block_tests::relative_flow::relative_canonical.
use super::*;
use crate::test_support::readback::{self, Phase};

fn actual_visual_corners_preserve_normal_receipt<S: LayoutScalar>() {
    for (nominal, top, left, trailing, expected) in [
        (
            surgeist_text::Size::new(10.0, 10.4),
            0.0,
            0.0,
            false,
            (10.0, 0.0, 10.0, 10.0),
        ),
        (
            surgeist_text::Size::new(10.0, 10.4),
            0.72,
            0.0,
            false,
            (10.0, 0.0, 10.0, 11.0),
        ),
        (
            surgeist_text::Size::new(10.4, 10.0),
            0.0,
            0.0,
            true,
            (10.0, 0.0, 10.0, 10.0),
        ),
        (
            surgeist_text::Size::new(10.4, 10.0),
            0.0,
            0.32,
            true,
            (10.0, 0.0, 11.0, 10.0),
        ),
    ] {
        let paragraph = fractional_atomic_paragraph(nominal, trailing);
        let batch = fractional_atomic_layout::<S>(
            &paragraph,
            nominal,
            Edges {
                top: LengthAutoOf::px(px::<S>(top)),
                left: LengthAutoOf::px(px::<S>(left)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            trailing,
        );
        fractional_atomic_source_and_font(&batch, trailing);
        // Independent corner arithmetic: the preceding Ahem X uses10px.
        // Empty atomic normal baseline is10; measured top for height10.4 is-.4.
        // At top.72, raw top=.32 and bottom=10.72, snapped to0 and11. This
        // visual11 must coexist with normal height12/baseline10, not feed H13.
        // At left.32, raw x=10.32 and right=20.72 snap to10 and21. Following
        // source X remains at normal20 rather than acquiring visual width11.
        for entries in [
            readback::continuous_custody(&batch, Phase::Unrounded).as_slice(),
            readback::continuous_custody(&batch, Phase::Final).as_slice(),
        ] {
            let node = entries
                .iter()
                .find(|entry| entry.node() == 2)
                .unwrap()
                .output();
            assert_eq!(
                node.location,
                Point::new(px::<S>(expected.0), px::<S>(expected.1))
            );
            assert_eq!(
                node.size,
                Size::new(px::<S>(expected.2), px::<S>(expected.3))
            );
            assert_eq!(
                entries
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output()
                    .size
                    .height,
                px::<S>(12.0)
            );
        }
        let visual = batch.placed_text()[0].layout().inline_boxes()[0].rect();
        assert_eq!(
            (
                visual.origin.x,
                visual.origin.y,
                visual.size.width,
                visual.size.height
            ),
            expected
        );
        assert_eq!(batch.placed_text()[0].layout().lines()[0].baseline(), 10.0);
        if trailing {
            let following = batch
                .unrounded_inline_fragments()
                .iter()
                .find(|entry| *entry.owner() == crate::InlineFragmentOwner::Node(3))
                .unwrap()
                .fragment()
                .rect()
                .unwrap();
            assert_eq!(following.origin().x, px::<S>(20.0));
        }
    }
}

#[test]
fn relative_atomic_visual_corners_keep_independent_normal_receipts_f32() {
    actual_visual_corners_preserve_normal_receipt::<f32>();
}
#[test]
fn relative_atomic_visual_corners_keep_independent_normal_receipts_f64() {
    actual_visual_corners_preserve_normal_receipt::<f64>();
}

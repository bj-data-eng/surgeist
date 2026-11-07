use super::*;
use crate::test_support::readback::{self, Phase};

fn native_fractional_inset_uses_measured_basis<S: LayoutScalar>() {
    for origin in [0.0, 0.72] {
        let container = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [9, 1])
            .children(9, [])
            .children(1, [3])
            .children(3, [])
            .style(0, root_style(Direction::Ltr))
            .style(
                9,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(px::<S>(50.0)),
                        PreferredSizeOf::px(px::<S>(origin)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(1, container)
            .style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    position: Position::Relative,
                    size: Size::new(
                        PreferredSizeOf::px(px::<S>(10.0)),
                        PreferredSizeOf::px(px::<S>(12.5)),
                    ),
                    inset: Edges {
                        top: inset_percent(0.0385),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..NodeInputOf::default()
                },
            );
        let batch = compute(&tree);
        let raw_container = output(&batch, 1);
        assert_eq!(raw_container.location, Point::new(S::ZERO, px::<S>(origin)));
        assert_eq!(raw_container.size.height, px::<S>(12.5));
        let final_container = readback::single_box_output(&batch, 1, Phase::Final);
        // An actual preceding normal sibling changes publication phase only:
        //0..12.5=>0..13, versus.72..13.22=>1..13. The used size stays12.5.
        assert_eq!(
            final_container.size.height,
            px::<S>(if origin == 0.0 { 13.0 } else { 12.0 })
        );
        let resolved = px::<S>(0.0385) * px::<S>(12.5);
        assert_eq!(output(&batch, 3).location, Point::new(S::ZERO, resolved));
        let final_child = readback::single_box_output(&batch, 3, Phase::Final);
        assert_eq!(final_child.location, Point::ZERO);
        assert_eq!(final_child.size, Size::new(px::<S>(10.0), px::<S>(13.0)));
    }
}

#[test]
fn native_percentage_basis_precedes_publication_rounding_f32() {
    native_fractional_inset_uses_measured_basis::<f32>();
}

#[test]
fn native_percentage_basis_precedes_publication_rounding_f64() {
    native_fractional_inset_uses_measured_basis::<f64>();
}

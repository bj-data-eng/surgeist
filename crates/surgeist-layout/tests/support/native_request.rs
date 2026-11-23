//! Declared native viewport inputs for the public Layout integration fixtures.
//!
//! The viewport's physical inner extents are independent of sizing operations
//! and of a root's authored or externally allocated size. A missing dimension
//! records an unknown ICB dimension; it is not an infinite percentage basis.
use surgeist_layout as l;

pub(crate) fn sizing<S: l::scalar::LayoutScalar>(
    viewport_inner_size: l::geometry::Size<Option<S>>,
    operations: l::geometry::Size<l::constraint::SizingOperation>,
) -> l::constraint::SizingRequestOf<S> {
    let icb = viewport_inner_size.map(|extent| {
        extent.map(|value| {
            l::length::NonNegativeFiniteOf::new(value)
                .expect("a declared native viewport extent is finite and nonnegative")
        })
    });
    let spaces = icb.map(|extent| match extent {
        Some(value) => l::constraint::AvailableSpaceOf::Definite(value),
        None => l::constraint::AvailableSpaceOf::Indefinite,
    });
    // These standalone native fixtures have no ancestor scrollport or generated
    // containing-box constraints. The local limits and ICB remain separate;
    // descendants replace the nearest scrollport through Layout's actual owner.
    l::constraint::SizingRequestOf::new(
        l::geometry::Size::new(
            l::constraint::AxisSizingRequestOf::new(spaces.width, operations.width),
            l::constraint::AxisSizingRequestOf::new(spaces.height, operations.height),
        ),
        l::constraint::OrthogonalFallbackOf::new(
            icb,
            l::constraint::NearestScrollportOf::Absent,
            l::geometry::Size::splat(l::constraint::OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
}

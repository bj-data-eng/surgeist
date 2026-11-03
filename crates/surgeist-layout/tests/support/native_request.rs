//! Declared native viewport inputs for the public Layout integration fixtures.
//!
//! The viewport's physical inner extents are independent of sizing operations
//! and of a root's authored or externally allocated size. A missing dimension
//! records an unknown ICB dimension; it is not an infinite percentage basis.
use surgeist_layout as l;

pub(crate) fn sizing<S: l::LayoutScalar>(
    viewport_inner_size: l::Size<Option<S>>,
    operations: l::Size<l::SizingOperation>,
) -> l::SizingRequestOf<S> {
    let icb = viewport_inner_size.map(|extent| {
        extent.map(|value| {
            l::NonNegativeFiniteOf::new(value)
                .expect("a declared native viewport extent is finite and nonnegative")
        })
    });
    let spaces = icb.map(|extent| match extent {
        Some(value) => l::AvailableSpaceOf::Definite(value),
        None => l::AvailableSpaceOf::Indefinite,
    });
    // These standalone native fixtures have no ancestor scrollport or generated
    // containing-box constraints. The local limits and ICB remain separate;
    // descendants replace the nearest scrollport through Layout's actual owner.
    l::SizingRequestOf::new(
        l::Size::new(
            l::AxisSizingRequestOf::new(spaces.width, operations.width),
            l::AxisSizingRequestOf::new(spaces.height, operations.height),
        ),
        l::OrthogonalFallbackOf::new(
            icb,
            l::NearestScrollportOf::Absent,
            l::Size::splat(l::OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
}

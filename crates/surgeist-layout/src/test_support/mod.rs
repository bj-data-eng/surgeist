pub(crate) mod grid_layout_comparison;
pub(crate) mod layout_tree;
pub(crate) mod oracle;
pub(crate) mod scroll_geometry;
pub(crate) mod text;
pub(crate) mod text_fragment;

// Existing non-Text diagnostic tests compare the actual typed payload. The
// production error owns arbitrary Text causes and has no diagnostic equality.
pub(crate) fn assert_error_kind<S, M>(
    actual: &crate::LayoutErrorKindOf<S, M>,
    expected: &crate::LayoutErrorKindOf<S, M>,
) where
    S: crate::LayoutScalar,
    M: core::fmt::Debug + PartialEq,
{
    use crate::LayoutErrorKindOf as Kind;
    match (actual, expected) {
        (Kind::InvalidInput(actual), Kind::InvalidInput(expected)) => assert_eq!(actual, expected),
        (Kind::MissingContext(actual), Kind::MissingContext(expected)) => {
            assert_eq!(actual, expected)
        }
        (Kind::UnsupportedCapability(actual), Kind::UnsupportedCapability(expected)) => {
            assert_eq!(actual, expected)
        }
        (Kind::Measurement(actual), Kind::Measurement(expected)) => assert_eq!(actual, expected),
        (Kind::InternalInvariant(actual), Kind::InternalInvariant(expected)) => {
            assert_eq!(actual, expected)
        }
        _ => panic!("unexpected typed error kind: actual={actual:?}, expected={expected:?}"),
    }
}

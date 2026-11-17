//! Display text of the native Paint gradient preparation error and its end-to-end routes.
#![cfg(feature = "render-paint")]
#![cfg(not(target_arch = "wasm32"))]

use core::error::Error as StdError;
use surgeist_paint::radial_gradient::RadialGradientCase;
use surgeist_render::{PaintGradientError, RenderResource};

#[test]
fn the_paint_gradient_error_is_a_standard_error_without_a_cause() {
    fn assert_error<E: StdError + core::fmt::Debug + Send + Sync + 'static>() {}
    assert_error::<PaintGradientError>();
}

#[test]
fn every_variant_has_its_documented_sentence() {
    use PaintGradientError as E;
    use RenderResource as R;
    let rows: Vec<(E, String)> = vec![
        (
            E::UnsupportedZeroLengthLinear,
            "a linear gradient of zero length cannot be prepared".into(),
        ),
        (
            E::UnsupportedRadialCase {
                case: RadialGradientCase::ZeroCircle,
            },
            "a radial gradient of the zero-circle case cannot be prepared as a native field".into(),
        ),
        (
            E::BasisMismatch {
                expected: 10.0,
                actual: 8.5,
            },
            "the used stops are measured on a line of length 8.5, but the geometry's basis is 10"
                .into(),
        ),
        (
            E::UnrepresentablePosition {
                source_index: 2,
                position_index: 1,
            },
            "position 1 of item 2 cannot be represented as a finite native position that keeps its order"
                .into(),
        ),
        (
            E::UnrepresentableSpan {
                first_source_index: 0,
                last_source_index: 3,
            },
            "the span from item 0 to item 3 of a repeating gradient is negative or not finite"
                .into(),
        ),
        (
            E::Allocation {
                resource: R::ScratchCpuBytes,
            },
            "allocating scratch cpu bytes failed".into(),
        ),
        (
            E::CapacityOverflow {
                resource: R::RetainedCpuBytes,
            },
            "the size of retained cpu bytes cannot be represented".into(),
        ),
    ];
    let mut seen = [false; 7];
    for (error, text) in &rows {
        assert_eq!(&error.to_string(), text);
        assert_ne!(&format!("{error:?}"), text);
        assert!(error.source().is_none());
        // The match has no wildcard: a new variant fails to compile until it is given a row above.
        seen[match error {
            E::UnsupportedZeroLengthLinear => 0,
            E::UnsupportedRadialCase { .. } => 1,
            E::BasisMismatch { .. } => 2,
            E::UnrepresentablePosition { .. } => 3,
            E::UnrepresentableSpan { .. } => 4,
            E::Allocation { .. } => 5,
            E::CapacityOverflow { .. } => 6,
        }] = true;
    }
    assert!(seen.iter().all(|covered| *covered), "a variant has no row");
    for (case, label) in [
        (RadialGradientCase::ZeroWidth, "zero-width"),
        (RadialGradientCase::ZeroHeight, "zero-height"),
    ] {
        assert_eq!(
            PaintGradientError::UnsupportedRadialCase { case }.to_string(),
            format!("a radial gradient of the {label} case cannot be prepared as a native field")
        );
    }
}

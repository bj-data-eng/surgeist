//! Qualifies the locked Kurbo dash initializer before entering its opaque work.

use crate::accounting::{Account, AccountedVec, RenderWork};
use crate::{GeometryRealizationError, Result};

use super::bounded;

fn action<T>(account: &Account, run: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(run())
}

fn capacity() -> crate::Error {
    GeometryRealizationError::InsufficientNativeCapacity.into()
}

/// Construction proves initialization for this exact immutable native input.
/// Only consuming this value can enter the dependency initializer.
pub(super) struct QualifiedDash<'a>(DashKind<'a>);

enum DashKind<'a> {
    Solid,
    Cycle { offset: f64, intervals: &'a [f64] },
}

impl<'a> QualifiedDash<'a> {
    pub(super) fn try_new(style: &'a kurbo::Stroke, account: &Account) -> Result<Self> {
        let intervals = style.dash_pattern.as_slice();
        let count = action(account, || intervals.len())?;
        if action(account, || count == 0)? {
            return Ok(Self(DashKind::Solid));
        }

        // Match kurbo 0.13.1's sequential sum and odd-array doubling exactly.
        let mut period = 0.0;
        for interval in intervals {
            let interval = action(account, || *interval)?;
            if !action(account, || interval.is_finite())? || action(account, || interval < 0.0)? {
                return Err(capacity());
            }
            period = action(account, || period + interval)?;
            if !action(account, || period.is_finite())? {
                return Err(capacity());
            }
        }
        if action(account, || period == 0.0)? {
            return Ok(Self(DashKind::Solid));
        }
        let offset = action(account, || style.dash_offset)?;
        if !action(account, || offset.is_finite())? {
            return Err(capacity());
        }
        let parity = action(account, || count % 2)?;
        if action(account, || parity == 1)? {
            period = action(account, || 2.0 * period)?;
        }
        if !action(account, || period.is_finite())? {
            return Err(capacity());
        }
        let phase = action(account, || offset.rem_euclid(period))?;
        if !action(account, || phase.is_finite())? {
            return Err(capacity());
        }
        let Some(first) = intervals.first() else {
            return Err(capacity());
        };
        let first = action(account, || *first)?;
        let mut remaining = action(account, || first - phase)?;
        if !action(account, || remaining.is_finite())? {
            return Err(capacity());
        }

        // One effective cycle contains at most two authored traversals. Reproduce
        // the locked initializer's additions without unchecked indexing, an
        // overflowing doubled count, or an unbounded floating-point progress loop.
        for interval in intervals.iter().chain(intervals.iter()).skip(1) {
            if action(account, || remaining >= 0.0)? {
                return Ok(Self(DashKind::Cycle { offset, intervals }));
            }
            let interval = action(account, || *interval)?;
            remaining = action(account, || remaining + interval)?;
            if !action(account, || remaining.is_finite())? {
                return Err(capacity());
            }
        }
        if action(account, || remaining >= 0.0)? {
            Ok(Self(DashKind::Cycle { offset, intervals }))
        } else {
            Err(capacity())
        }
    }

    pub(super) fn materialize(
        self,
        source: AccountedVec<kurbo::PathEl>,
        account: &Account,
    ) -> Result<AccountedVec<kurbo::PathEl>> {
        match self.0 {
            DashKind::Solid => Ok(source),
            DashKind::Cycle { offset, intervals } => {
                // The opaque initializer repeats its own sum and phase search.
                // Admit that bounded call separately from our proof and copies.
                account.work(RenderWork::DashInitializations, 1)?;
                bounded::materialize(
                    kurbo::dash(source.iter().copied(), offset, intervals),
                    account,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounting::{RenderAttemptKind, RenderPhase};
    use crate::{ErrorCode, RenderAccountingContext, RenderResource};
    use kurbo::PathEl::{LineTo, MoveTo};

    fn style(offset: f64, intervals: &[f64]) -> kurbo::Stroke {
        kurbo::Stroke::new(1.0).with_dashes(offset, intervals.iter().copied())
    }

    #[test]
    fn overflowing_even_sum_and_odd_period_refuse_native_capacity() {
        for intervals in [&[f64::MAX, f64::MAX][..], &[f64::MAX][..]] {
            for offset in [-1.0, 0.0, 1.0] {
                let style = style(offset, intervals);
                let error = QualifiedDash::try_new(&style, &Account::for_test())
                    .err()
                    .expect("overflowed native cycle must refuse");
                assert_eq!(error.code(), ErrorCode::GeometryRealization);
                assert_eq!(
                    error.geometry_realization_diagnostic(),
                    Some(&GeometryRealizationError::InsufficientNativeCapacity)
                );
            }
        }
    }

    #[test]
    fn ordinary_offsets_zero_intervals_and_tiny_positive_cycles_qualify() {
        for intervals in [
            &[2.0, 1.0][..],
            &[2.0, 1.0, 3.0][..],
            &[0.0, 1.0][..],
            &[1.0, 0.0][..],
            &[f64::from_bits(1)][..],
            &[f64::MAX / 4.0][..],
        ] {
            for offset in [-17.0, -1.0, 0.0, 1.0, 17.0] {
                let style = style(offset, intervals);
                let qualified = QualifiedDash::try_new(&style, &Account::for_test()).unwrap();
                assert!(matches!(qualified.0, DashKind::Cycle { .. }));
            }
        }
        for intervals in [&[][..], &[0.0][..], &[0.0, 0.0][..]] {
            let style = style(-1.0, intervals);
            let qualified = QualifiedDash::try_new(&style, &Account::for_test()).unwrap();
            assert!(matches!(qualified.0, DashKind::Solid));
        }
    }

    #[test]
    fn native_output_preserves_authored_phase_and_lengths() {
        for (offset, expected) in [
            (0.0, vec![0.0, 2.0, 3.0, 5.0, 6.0, 8.0]),
            (1.0, vec![0.0, 1.0, 2.0, 4.0, 5.0, 7.0]),
            (-2.0, vec![0.0, 1.0, 2.0, 4.0, 5.0, 7.0]),
        ] {
            let account = Account::for_test();
            let style = style(offset, &[2.0, 1.0]);
            let qualified = QualifiedDash::try_new(&style, &account).unwrap();
            let source = bounded::materialize(
                [MoveTo((0.0, 0.0).into()), LineTo((8.0, 0.0).into())],
                &account,
            )
            .unwrap();
            let output = qualified.materialize(source, &account).unwrap();
            assert_eq!(output.len(), expected.len());
            let mut coordinates = Vec::new();
            for (index, element) in output.iter().enumerate() {
                let point = match (index % 2, element) {
                    (0, MoveTo(point)) | (1, LineTo(point)) => point,
                    _ => panic!("dash output must alternate moves and lines"),
                };
                coordinates.push(point.x);
                assert_eq!(point.y, 0.0);
            }
            // The locked iterator stashes the first mark for seam handling.
            // Compare actual intervals; their emission order is not placement.
            let (segments, remainder) = coordinates.as_chunks::<2>();
            assert!(remainder.is_empty());
            let mut segments = segments.to_vec();
            segments.sort_by(|left, right| left[0].total_cmp(&right[0]));
            let (expected, remainder) = expected.as_chunks::<2>();
            assert!(remainder.is_empty());
            for (segment, expected) in segments.iter().zip(expected) {
                for (actual, expected) in segment.iter().zip(expected) {
                    // The pinned dash algorithm uses 1e-6 arc-length accuracy.
                    assert!((actual - expected).abs() <= 1e-6);
                }
            }
            assert_eq!(
                account.work_count(RenderWork::DashInitializations).unwrap(),
                1
            );
        }
    }

    #[test]
    fn solid_patterns_reuse_source_without_native_initialization() {
        for intervals in [&[][..], &[0.0][..], &[0.0, 0.0][..]] {
            let account = Account::for_test();
            let style = style(-1.0, intervals);
            let qualified = QualifiedDash::try_new(&style, &account).unwrap();
            let expected = [MoveTo((0.0, 0.0).into()), LineTo((8.0, 0.0).into())];
            let source = bounded::materialize(expected, &account).unwrap();
            let output = qualified.materialize(source, &account).unwrap();
            assert_eq!(output.as_slice(), expected);
            assert_eq!(
                account.work_count(RenderWork::DashInitializations).unwrap(),
                0
            );
        }
    }

    #[test]
    fn point_only_source_does_not_require_native_cycle_capacity() {
        let account = Account::for_test();
        let style = style(-1.0, &[f64::MAX]);
        let mut source = kurbo::BezPath::new();
        source.move_to((0.0, 0.0));
        let mut scene = super::super::scene::VelloScene::default();
        scene
            .stroke(
                &style,
                kurbo::Affine::IDENTITY,
                peniko::Color::BLACK,
                None,
                &source,
                &account,
            )
            .unwrap();
        assert_eq!(
            account.work_count(RenderWork::DashInitializations).unwrap(),
            0
        );
    }

    #[test]
    fn short_cycle_proof_admits_exact_prefix_and_clean_retry() {
        // offset capture/check + length capture/check = 4; two interval
        // capture/finite/nonnegative/add/finite groups = 10; zero check,
        // parity arithmetic/check, period finite check = 4; remainder/check = 2;
        // first capture/subtract/check = 3; phase completion check = 1.
        // Total: 24 scalar actions.
        const ACTIONS: u64 = 24;
        let style = style(0.0, &[2.0, 1.0]);
        let mut context = RenderAccountingContext::new(
            crate::RenderAccountingOptions::new().with_limits(
                crate::RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, ACTIONS - 1)
                    .unwrap(),
            ),
        );
        let error = context
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                QualifiedDash::try_new(&style, account).map(|_| ())
            })
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::Accounting);
        let Some(crate::RenderAccountingError::Refused(refusal)) = error.accounting_diagnostic()
        else {
            panic!("expected the actual scalar admission refusal");
        };
        assert_eq!(
            refusal.quantity(),
            crate::RenderQuantity::Work(RenderWork::CoordinateVisits)
        );
        assert_eq!(
            refusal.reason(),
            crate::RenderRefusalReason::Limit { limit: ACTIONS - 1 }
        );
        assert_eq!(refusal.current(), ACTIONS - 1);
        assert_eq!(refusal.amount(), 1);
        let usage = context.last_attempt().unwrap().usage();
        assert_eq!(usage.work(RenderWork::CoordinateVisits), ACTIONS - 1);
        assert_eq!(usage.work(RenderWork::DashInitializations), 0);
        assert_eq!(usage.live(RenderResource::ScratchCpuBytes), 0);
        assert_eq!(usage.live(RenderResource::RetainedCpuBytes), 0);

        let mut retry = RenderAccountingContext::default();
        retry
            .run(RenderAttemptKind::Frame, RenderPhase::Encode, |account| {
                QualifiedDash::try_new(&style, account).map(|_| ())
            })
            .unwrap();
        assert_eq!(
            retry
                .last_attempt()
                .unwrap()
                .usage()
                .work(RenderWork::CoordinateVisits),
            ACTIONS
        );
    }
}

//! Accounted premultiplied numeric-sRGBA reduction for authenticated color measures.

use crate::{
    Color, Result,
    accounting::{Account, RenderWork},
};

fn scalar<T>(account: &Account, action: impl FnOnce() -> T) -> Result<T> {
    account.work(RenderWork::CoordinateVisits, 1)?;
    Ok(action())
}

pub(crate) fn premultiplied(color: Color, account: &Account) -> Result<[f64; 4]> {
    account.work(RenderWork::CoordinateVisits, 4)?;
    let (r, g, b, a) = (color.r(), color.g(), color.b(), color.a());
    let alpha = scalar(account, || f64::from(a))?;
    let r = scalar(account, || f64::from(r))?;
    let g = scalar(account, || f64::from(g))?;
    let b = scalar(account, || f64::from(b))?;
    Ok([
        scalar(account, || r * alpha)?,
        scalar(account, || g * alpha)?,
        scalar(account, || b * alpha)?,
        alpha,
    ])
}

/// Stack-only sum. Callers supply authenticated finite nonnegative weights,
/// bounded by their normalized color/segment measure. Color proves its channels.
/// Identical-order nonnegative arithmetic preserves component<=alpha and finite
/// totals; no intrinsic source-domain checks or semantic weight clamps occur here.
pub(crate) struct PremultipliedSum {
    total: [f64; 4],
}

impl PremultipliedSum {
    pub(crate) const fn new() -> Self {
        Self { total: [0.0; 4] }
    }

    pub(crate) fn add_weighted(
        &mut self,
        color: [f64; 4],
        weight: f64,
        account: &Account,
    ) -> Result<()> {
        for (total, component) in self.total.iter_mut().zip(color) {
            let weighted = scalar(account, || component * weight)?;
            *total = scalar(account, || *total + weighted)?;
        }
        Ok(())
    }

    /// Preserve the existing Repeat trapezoid association: add adjacent channels
    /// before multiplying by half the segment mass. Replacing this with separate
    /// endpoint-weighted adds would change the selected floating-point rounding.
    pub(crate) fn add_segment(
        &mut self,
        previous: [f64; 4],
        current: [f64; 4],
        half_mass: f64,
        account: &Account,
    ) -> Result<()> {
        for ((total, previous), current) in self.total.iter_mut().zip(previous).zip(current) {
            let sum = scalar(account, || previous + current)?;
            let weighted = scalar(account, || sum * half_mass)?;
            *total = scalar(account, || *total + weighted)?;
        }
        Ok(())
    }

    pub(crate) fn finish(self, account: &Account) -> Result<Color> {
        let [r, g, b, alpha] = self.total;
        if scalar(account, || alpha == 0.0)? {
            return Ok(Color::TRANSPARENT);
        }
        let channel = |component: f64| -> Result<f32> {
            let straight = scalar(account, || component / alpha)?;
            let bounded = scalar(account, || straight.clamp(0.0, 1.0))?;
            scalar(account, || bounded as f32)
        };
        let r = channel(r)?;
        let g = channel(g)?;
        let b = channel(b)?;
        let alpha = scalar(account, || alpha.clamp(0.0, 1.0))?;
        let alpha = scalar(account, || alpha as f32)?;
        // The authenticated nonnegative measure and identical-order sums prove
        // component<=alpha and finite totals. Nonzero-alpha division followed by
        // the accounted clamp/narrowing proves four finite normalized channels.
        Ok(Color::rgba(r, g, b, alpha))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        RenderAccountingContext, RenderAccountingOptions, RenderAttemptKind, RenderLimits,
        RenderPhase, RenderResource,
    };

    fn colors() -> [Color; 2] {
        [
            Color::try_rgba(1.0, 0.0, 0.0, 0.5).unwrap(),
            Color::try_rgba(0.0, 0.0, 1.0, 1.0).unwrap(),
        ]
    }

    fn reduce(account: &Account) -> Result<Color> {
        let mut total = PremultipliedSum::new();
        for color in colors() {
            let color = premultiplied(color, account)?;
            total.add_weighted(color, 0.5, account)?;
        }
        total.finish(account)
    }

    #[test]
    fn half_red_alpha_and_opaque_blue_have_the_selected_straight_output() {
        let account = Account::for_test();
        let weighted = reduce(&account).unwrap();
        let [left, right] = colors();
        let left = premultiplied(left, &account).unwrap();
        let right = premultiplied(right, &account).unwrap();
        let mut total = PremultipliedSum::new();
        total.add_segment(left, right, 0.5, &account).unwrap();
        let segment = total.finish(&account).unwrap();
        // Half red alpha .5 plus half opaque blue is premul(.25,0,.5,.75).
        let expected = Color::try_rgba(1.0 / 3.0, 0.0, 2.0 / 3.0, 0.75).unwrap();
        assert_eq!(weighted, expected);
        assert_eq!(segment, expected);
    }

    #[test]
    fn reducer_charges_every_channel_action_and_preserves_refusal_prefix() {
        let mut context = RenderAccountingContext::new(RenderAccountingOptions::new());
        let expected = context
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Coordinates,
                reduce,
            )
            .unwrap();
        // Each input: 4 channel captures + 4 widening conversions + 3 alpha
        // products + 4 weighted products + 4 sums = 19. Finish: alpha check,
        // 3*(divide,clamp,narrow), and alpha clamp/narrow = 12. No allocation.
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 50);
        assert_eq!(report.usage().peak(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().peak(RenderResource::ScratchCpuBytes), 0);
        context.set_options(
            RenderAccountingOptions::new().with_limits(
                RenderLimits::new()
                    .with_work_limit(RenderWork::CoordinateVisits, 49)
                    .unwrap(),
            ),
        );
        let error = context
            .run(
                RenderAttemptKind::GlyphRun,
                RenderPhase::Coordinates,
                reduce,
            )
            .unwrap_err();
        assert!(error.accounting_diagnostic().is_some());
        let report = context.last_attempt().unwrap().attempt().unwrap();
        assert_eq!(report.usage().work(RenderWork::CoordinateVisits), 49);
        assert_eq!(report.usage().live(RenderResource::RetainedCpuBytes), 0);
        assert_eq!(report.usage().live(RenderResource::ScratchCpuBytes), 0);
        context.set_options(RenderAccountingOptions::new());
        assert_eq!(
            context
                .run(
                    RenderAttemptKind::GlyphRun,
                    RenderPhase::Coordinates,
                    reduce
                )
                .unwrap(),
            expected
        );
    }

    #[test]
    fn all_zero_alpha_discards_hidden_straight_channels_after_the_real_sum() {
        let account = Account::for_test();
        let hidden = Color::try_rgba(1.0, 0.5, 0.25, 0.0).unwrap();
        let mut total = PremultipliedSum::new();
        let color = premultiplied(hidden, &account).unwrap();
        total.add_weighted(color, 1.0, &account).unwrap();
        assert_eq!(total.finish(&account).unwrap(), Color::TRANSPARENT);
    }
}

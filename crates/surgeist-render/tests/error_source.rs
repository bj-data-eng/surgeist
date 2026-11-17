//! `source()` of the accounting errors that wrap another error, and the standard-error bounds.
//!
//! `RenderAccountingError::Setup` returns its `RenderAccountingSetupError`; `Refused` returns the
//! `RenderRefusal`; every other variant has no cause. The refusal cannot be built outside the crate,
//! so it is obtained from a real refusal.

use core::error::Error as StdError;
use surgeist_render::{
    RenderAccountingContext, RenderAccountingError, RenderAccountingOptions,
    RenderAccountingSetupError, RenderLimits, RenderRefusal, RenderWork,
};

#[test]
fn setup_wraps_the_setup_error_and_returns_it_as_the_cause() {
    for (setup, text) in [
        (
            RenderAccountingSetupError::Allocation,
            "accounting could not allocate its setup or report storage",
        ),
        (
            RenderAccountingSetupError::TimingUnavailable,
            "timed profiling is unavailable on this target",
        ),
        (
            RenderAccountingSetupError::IdentityExhausted,
            "accounting identities are exhausted",
        ),
    ] {
        let error = RenderAccountingError::Setup(setup);
        assert_eq!(error.to_string(), "accounting setup failed");
        let cause = error.source().expect("the setup error is the cause");
        assert_eq!(cause.to_string(), text);
        assert_eq!(
            cause.downcast_ref::<RenderAccountingSetupError>(),
            Some(&setup)
        );
    }
}

#[test]
fn a_refusal_is_a_standard_error_and_the_cause_of_the_refused_variant() {
    fn assert_error<T: StdError + Send + Sync + 'static>() {}
    assert_error::<RenderRefusal>();
    // The smallest real refusal: a zero cap on the first unit a standalone operation charges.
    let limits = RenderLimits::new()
        .with_work_limit(RenderWork::GlyphVisits, 0)
        .unwrap();
    let mut context =
        RenderAccountingContext::new(RenderAccountingOptions::new().with_limits(limits));
    let font = surgeist_render::FontData::try_from_bytes(
        include_bytes!("fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        0,
        surgeist_test::fixtures::FONT_LIMITS,
    )
    .unwrap();
    let instance = surgeist_render::TextInstance::try_new(
        1000.0,
        vec![],
        surgeist_render::GlyphSynthesis::none(),
    )
    .unwrap();
    let glyph = surgeist_render::TextGlyph::try_new(58, 0.0, 0.0, 1000.0).unwrap();
    let error = font
        .prepare_glyph_run(
            &instance,
            &[glyph],
            surgeist_render::GlyphRunLimits::new(1, 5, 1 << 20, 1 << 20),
            &mut context,
        )
        .unwrap_err();
    let diagnostic = error.accounting_diagnostic().unwrap();
    let refusal = context
        .last_attempt()
        .unwrap()
        .attempt()
        .unwrap()
        .refusal()
        .unwrap();
    assert_eq!(diagnostic, RenderAccountingError::Refused(refusal));
    let cause = diagnostic.source().expect("the refusal is the cause");
    assert_eq!(cause.downcast_ref::<RenderRefusal>(), Some(&refusal));
    assert!(refusal.source().is_none());
}

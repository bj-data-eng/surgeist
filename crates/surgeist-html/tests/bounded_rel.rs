//! Retained HTML877d local rel interpretation and checked refinement.
#![forbid(unsafe_code)]
use surgeist_html::{bounded::*, *};
#[derive(Default)]
struct Meter {
    work: Vec<usize>,
    fail_at: Option<usize>,
    reservations: Vec<(usize, usize, usize)>,
    fail_reserve: bool,
}
impl HtmlRelMeter for Meter {
    type Error = &'static str;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.fail_at == Some(self.work.len()) {
            return Err("original stop");
        }
        self.work.push(n);
        Ok(())
    }
    fn reserve(&mut self, guard: &mut HtmlRelReservation<'_>) -> Result<(), Self::Error> {
        let len = guard.len();
        guard.try_reserve().unwrap();
        self.reservations
            .push((len, guard.capacity(), guard.record_size()));
        if self.fail_reserve {
            return Err("original reserve stop");
        }
        Ok(())
    }
}
fn interpreted<'a>(inputs: RelInputs<'a>) -> HtmlRelInterpretation<'a> {
    let mut owner = HtmlRelInterpretation::new(inputs);
    owner.interpret(&mut Meter::default()).unwrap();
    owner
}
#[test]
fn initial_only_finish_preserves_original_ranges_aliases_and_alternate_stylesheet_rules() {
    let rel = DocumentString::from("ALTERNATE stylesheet alternate");
    let href = DocumentString::from("sheet.css");
    let title = DocumentString::from("Night");
    let owner = interpreted(
        RelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&rel))
            .with_href(Some(&href))
            .with_title(Some(&title)),
    );
    assert_eq!(owner.occurrences().unwrap()[2].duplicate_of(), Some(0));
    let mut slot = Some(owner);
    let report = HtmlRelInterpretation::try_finish(&mut slot).unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::StylesheetModifier);
    assert_eq!(
        report.alternate_stylesheet_title_author(),
        HtmlRelTitleAuthor::Satisfied
    );
    assert_eq!(report.links(), HtmlRelLinks::Declared);
    assert_eq!(
        report.occurrence_units(0),
        Some(&[65, 76, 84, 69, 82, 78, 65, 84, 69][..])
    );
    assert!(std::sync::Arc::ptr_eq(
        report.rel().unwrap().shared_code_units(),
        rel.shared_code_units()
    ));
    assert_eq!(
        HtmlRelInterpretation::try_finish(&mut slot).unwrap_err(),
        HtmlRelInterpretationError::Consumed
    );
}
#[test]
fn checked_refinement_rejects_wrong_source_spelling_order_and_extension_effect_without_admission() {
    let raw = DocumentString::from("vendor vendor2 alternate");
    let rev = DocumentString::from("made");
    let mut owner = interpreted(
        RelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&raw))
            .with_rev(Some(&rev)),
    );
    owner.begin_refinement().unwrap();
    let mut meter = Meter::default();
    for (index, value, expected) in [
        (
            9,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::UnknownOccurrence,
        ),
        (
            3,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::WrongSource,
        ),
        (
            2,
            HtmlRelRefinement::NoEffect,
            HtmlRelInterpretationError::WrongSpelling,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::InternalResource),
            HtmlRelInterpretationError::InvalidExtensionEffect,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::StylesheetModifier),
            HtmlRelInterpretationError::InvalidExtensionEffect,
        ),
        (
            0,
            HtmlRelRefinement::Extension(HtmlRelEffect::Annotation),
            HtmlRelInterpretationError::InvalidExtensionEffect,
        ),
    ] {
        assert_eq!(
            owner.refine(index, value, &mut meter),
            Err(HtmlRelActionError::Protocol(expected))
        );
    }
    assert!(meter.work.is_empty());
    owner
        .refine(
            1,
            HtmlRelRefinement::Standard(HtmlRelKeyword::Stylesheet),
            &mut meter,
        )
        .unwrap();
    assert_eq!(meter.work, [4]);
    assert_eq!(
        owner.refine(1, HtmlRelRefinement::NoEffect, &mut meter),
        Err(HtmlRelActionError::Protocol(
            HtmlRelInterpretationError::RepeatedRefinement
        ))
    );
    assert_eq!(
        owner.refine(0, HtmlRelRefinement::NoEffect, &mut meter),
        Err(HtmlRelActionError::Protocol(
            HtmlRelInterpretationError::OutOfOrderRefinement
        ))
    );
    let mut slot = Some(owner);
    assert_eq!(
        HtmlRelInterpretation::try_finish(&mut slot).unwrap_err(),
        HtmlRelInterpretationError::NotReady
    );
    slot.as_mut().unwrap().refresh(&mut meter).unwrap();
    let report = HtmlRelInterpretation::try_finish(&mut slot).unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::StylesheetModifier);
    assert!(report.occurrences()[1].registry_resolved());
    assert_eq!(
        report.occurrences()[1].effect(),
        Some(HtmlRelEffect::ExternalResource)
    );
}
#[test]
fn refresh_uses_original_type_title_and_image_preload_operands() {
    let rel = DocumentString::from("vendor alternate");
    let href = DocumentString::from("u");
    let hint = DocumentString::from("application/rss+xml");
    let images = DocumentString::from("x 1x");
    let mut owner = interpreted(
        RelInputs::new(Some(HtmlRelRole::Link))
            .with_rel(Some(&rel))
            .with_href(Some(&href))
            .with_type_hint(Some(&hint))
            .with_imagesrcset(Some(&images))
            .with_as_destination(Some(HtmlLinkDestination::Image)),
    );
    assert!(std::ptr::eq(
        owner.inputs().unwrap().type_hint().unwrap(),
        &hint
    ));
    owner.begin_refinement().unwrap();
    owner
        .refine(
            0,
            HtmlRelRefinement::Standard(HtmlRelKeyword::Preload),
            &mut Meter::default(),
        )
        .unwrap();
    owner.refresh(&mut Meter::default()).unwrap();
    let report = HtmlRelInterpretation::try_finish(&mut Some(owner)).unwrap();
    assert_eq!(report.alternate(), HtmlRelAlternate::Feed);
    assert!(report.image_source_preload_context());
}
#[test]
fn every_refresh_admission_failure_hides_report_and_occurrences_including_after_aggregate_writes() {
    let raw = DocumentString::from("vendor alternate stylesheet");
    let href = DocumentString::from("x");
    let title = DocumentString::from("t");
    let make = || {
        let mut o = interpreted(
            RelInputs::new(Some(HtmlRelRole::Link))
                .with_rel(Some(&raw))
                .with_href(Some(&href))
                .with_title(Some(&title)),
        );
        o.begin_refinement().unwrap();
        o
    };
    let mut complete = make();
    let mut count = Meter::default();
    complete.refresh(&mut count).unwrap();
    // Three occurrence visits, fixed feed comparison, three effect updates,
    // aggregate write admission, then the image-preload context admission.
    assert_eq!(count.work, [12, 12, 12, 44, 5, 5, 5, 5, 3]);
    for i in 0..count.work.len() {
        let mut owner = Some(make());
        let mut meter = Meter {
            fail_at: Some(i),
            ..Meter::default()
        };
        assert_eq!(
            owner.as_mut().unwrap().refresh(&mut meter),
            Err(HtmlRelActionError::Admission("original stop"))
        );
        assert_eq!(
            HtmlRelInterpretation::try_finish(&mut owner).unwrap_err(),
            HtmlRelInterpretationError::Stopped
        );
        assert!(matches!(
            owner.as_ref().unwrap().occurrences(),
            Err(HtmlRelInterpretationError::Stopped)
        ));
        let inputs = owner.take().unwrap().abort();
        assert!(std::ptr::eq(inputs.rel().unwrap(), &raw));
    }
}
#[test]
fn initial_error_retains_raw_and_actual_reservation_and_between_action_stop_is_terminal() {
    let raw = DocumentString::from_utf16([120, 0xd800]);
    let mut slot = Some(HtmlRelInterpretation::new(
        RelInputs::new(Some(HtmlRelRole::Anchor)).with_rel(Some(&raw)),
    ));
    let mut meter = Meter {
        fail_reserve: true,
        ..Meter::default()
    };
    assert_eq!(
        slot.as_mut().unwrap().interpret(&mut meter),
        Err(HtmlRelActionError::Admission("original reserve stop"))
    );
    assert!(meter.reservations[0].1 >= 1);
    assert_eq!(
        HtmlRelInterpretation::try_finish(&mut slot).unwrap_err(),
        HtmlRelInterpretationError::Stopped
    );
    assert!(std::ptr::eq(
        slot.take().unwrap().abort().rel().unwrap(),
        &raw
    ));
    let mut owner = Some(interpreted(
        RelInputs::new(Some(HtmlRelRole::Anchor)).with_rel(Some(&raw)),
    ));
    owner.as_mut().unwrap().stop();
    assert_eq!(
        HtmlRelInterpretation::try_finish(&mut owner).unwrap_err(),
        HtmlRelInterpretationError::Stopped
    );
}

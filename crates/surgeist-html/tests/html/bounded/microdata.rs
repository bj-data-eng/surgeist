//! Exact public microdata ownership, admission and publication contracts.
#![forbid(unsafe_code)]
use std::sync::Arc;
use surgeist_html::{DocumentString, HtmlMicrodataTokenKind, bounded::*};

type Owner<'a> = HtmlMicrodataTokenInterpretation<'a>;
type Error = HtmlMicrodataTokenInterpretationError;
#[derive(Default)]
struct Meter {
    work: Vec<usize>,
    reservations: Vec<(usize, usize, usize)>,
    fail_after: Option<usize>,
    skip_reserve: bool,
    fail_after_reserve: bool,
}
impl HtmlMicrodataTokenMeter for Meter {
    type Error = &'static str;
    fn work(&mut self, n: usize) -> Result<(), Self::Error> {
        if self.fail_after == Some(self.work.len()) {
            return Err("caller stopped");
        }
        self.work.push(n);
        Ok(())
    }
    fn reserve(
        &mut self,
        guard: &mut HtmlMicrodataTokenReservation<'_>,
    ) -> Result<(), Self::Error> {
        if self.skip_reserve {
            return Ok(());
        }
        let len = guard.len();
        guard.try_reserve().unwrap();
        self.reservations
            .push((len, guard.capacity(), guard.record_size()));
        assert_eq!(guard.try_reserve(), Err(Error::ReservationAlreadyAttempted));
        if self.fail_after_reserve {
            return Err("post reserve cancelled");
        }
        Ok(())
    }
}
fn drive(owner: &mut Owner<'_>, meter: &mut Meter) {
    for _ in 0..10_000 {
        let Some(action) = owner.next_action().unwrap() else {
            return;
        };
        action.advance(meter).unwrap();
    }
    panic!("fixture action bound exhausted");
}
#[test]
fn original_utf16_ranges_first_duplicates_and_itemprop_suppression_are_retained() {
    // Infra ASCII whitespace only; non-ASCII space and isolated surrogates stay exact.
    let raw = DocumentString::from_utf16([32, 97, 9, 65, 10, 97, 12, 120, 46, 58, 0xd800, 0xa0]);
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::NotReady);
    let mut meter = Meter::default();
    drive(owner.as_mut().unwrap(), &mut meter);
    let report = Owner::try_finish(&mut owner).unwrap();
    let actual: Vec<_> = report
        .tokens()
        .iter()
        .map(|t| {
            (
                t.start(),
                t.end(),
                t.duplicate_of(),
                t.contains_dot_or_colon(),
            )
        })
        .collect();
    assert_eq!(
        actual,
        [
            (1, 2, None, false),
            (3, 4, None, false),
            (5, 6, Some(0), false),
            (7, 12, None, true)
        ]
    );
    assert_eq!(
        report.processing_tokens().collect::<Vec<_>>(),
        vec![&[97][..], &[65][..], &[120, 46, 58, 0xd800, 0xa0][..]]
    );
    assert!(Arc::ptr_eq(
        report.raw().unwrap().shared_code_units(),
        raw.shared_code_units()
    ));
    assert!(!report.local_author_conforming());
    assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::Consumed);
}
#[test]
fn single_token_includes_separate_loop_header_eof_done_and_report_admissions() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    let mut meter = Meter::default();
    drive(owner.as_mut().unwrap(), &mut meter);
    assert_eq!(meter.work, [1, 1, 4, 1, 2, 1, 1, 1, 2, 1]);
    assert_eq!(meter.reservations.len(), 1);
    assert_eq!(meter.reservations[0].0, 0);
    assert!(meter.reservations[0].1 >= 1);
    assert!(meter.reservations[0].2 > 0);
    assert_eq!(
        Owner::try_finish(&mut owner).unwrap().token_units(0),
        Some(&[112][..])
    );
}
#[test]
fn absent_and_empty_are_distinct_and_assembly_is_the_last_required_action() {
    let empty = DocumentString::from("");
    for raw in [None, Some(&empty)] {
        let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemType, raw));
        let mut meter = Meter::default();
        drive(owner.as_mut().unwrap(), &mut meter);
        assert_eq!(
            meter.work,
            if raw.is_some() {
                vec![1, 1, 2, 1]
            } else {
                vec![1]
            }
        );
        assert!(meter.reservations.is_empty());
        let report = Owner::try_finish(&mut owner).unwrap();
        assert_eq!(report.raw().is_some(), raw.is_some());
        assert_eq!(report.local_author_conforming(), raw.is_none());
    }
}
#[test]
fn callback_error_at_every_work_checkpoint_stops_without_report_or_repeated_work() {
    let raw = DocumentString::from("x.y x.y x:z");
    let mut complete = Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw));
    let mut count = Meter::default();
    drive(&mut complete, &mut count);
    for stop in 0..count.work.len() {
        let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
        let mut meter = Meter {
            fail_after: Some(stop),
            ..Meter::default()
        };
        loop {
            let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
            if let Err(error) = action.advance(&mut meter) {
                assert_eq!(
                    error,
                    HtmlMicrodataTokenActionError::Admission("caller stopped")
                );
                break;
            }
        }
        assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::Stopped);
        assert!(matches!(
            owner.as_mut().unwrap().next_action(),
            Err(Error::Stopped)
        ));
        assert_eq!(meter.work.len(), stop);
        assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
    }
}
#[test]
fn unexecuted_reservation_refuses_and_can_be_retried_but_post_reserve_error_stops() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemProp, Some(&raw)));
    let mut meter = Meter {
        skip_reserve: true,
        ..Meter::default()
    };
    loop {
        let action = owner.as_mut().unwrap().next_action().unwrap().unwrap();
        if action.kind() == HtmlMicrodataTokenActionKind::Reserve {
            assert_eq!(
                action.advance(&mut meter),
                Err(HtmlMicrodataTokenActionError::Protocol(
                    Error::ReservationNotPerformed
                ))
            );
            break;
        }
        action.advance(&mut meter).unwrap();
    }
    assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::NotReady);
    meter.skip_reserve = false;
    meter.fail_after_reserve = true;
    assert_eq!(
        owner
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .advance(&mut meter),
        Err(HtmlMicrodataTokenActionError::Admission(
            "post reserve cancelled"
        ))
    );
    assert!(meter.reservations[0].1 >= 1);
    assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::Stopped);
    assert_eq!(Arc::strong_count(raw.shared_code_units()), 1);
    assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
}
#[test]
fn explicit_between_action_stop_prevents_publication_even_after_assembly() {
    let raw = DocumentString::from("p");
    let mut owner = Some(Owner::new(HtmlMicrodataTokenKind::ItemRef, Some(&raw)));
    drive(owner.as_mut().unwrap(), &mut Meter::default());
    owner.as_mut().unwrap().stop();
    assert_eq!(Owner::try_finish(&mut owner).unwrap_err(), Error::Stopped);
    assert!(std::ptr::eq(owner.take().unwrap().abort().1.unwrap(), &raw));
}

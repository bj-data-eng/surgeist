//! The one native monotonic epoch owned by a hosted session.
//!
//! The selected non-isolated profile floors observations to 100 microseconds,
//! without jitter. These are elapsed native-domain observations, not Unix time
//! or a claim about an ECMAScript performance clock.
use std::time::{Duration, Instant};
use surgeist_dom::DomHostedSourceId;

/// A real observation of the epoch bound to the original hosted source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Sample {
    source: DomHostedSourceId,
    elapsed: Duration,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CoarsenedSample {
    pub source: DomHostedSourceId,
    pub microseconds: u64,
}
impl Sample {
    /// Derive source timing from the retained original sample at its actual
    /// coarsening phase; neither resample nor expose the finer private moment.
    pub fn coarsen(&self) -> Result<CoarsenedSample, Error> {
        Ok(CoarsenedSample {
            source: self.source,
            microseconds: coarsen(self.elapsed)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Error {
    Unavailable,
    Regressed,
    Overflow,
}

pub(super) struct Clock {
    source: DomHostedSourceId,
    epoch: Result<Instant, Error>,
    latest: Duration,
    // Per-owner test input only; production always uses native_now(). Scripted
    // fixtures exercise role limits, not production allocation-byte pricing.
    #[cfg(test)]
    observations: Option<std::collections::VecDeque<Result<Instant, Error>>>,
}
impl Clock {
    pub fn new(source: DomHostedSourceId) -> Self {
        Self {
            source,
            epoch: native_now(),
            latest: Duration::ZERO,
            #[cfg(test)]
            observations: None,
        }
    }

    /// Samples the actual clock; the source identity fixes its epoch/domain.
    pub fn sample(&mut self) -> Result<Sample, Error> {
        #[cfg(test)]
        if let Some(observations) = self.observations.as_mut() {
            let observation = observations
                .pop_front()
                .expect("the finite native clock observation script was exhausted");
            return self.observe(observation);
        }
        self.observe(native_now())
    }

    fn observe(&mut self, now: Result<Instant, Error>) -> Result<Sample, Error> {
        // An unavailable owner epoch stays unavailable. A later Document sample
        // cannot establish a replacement epoch or silently rebase the domain.
        let epoch = self.epoch?;
        let now = now?;
        let elapsed = now.checked_duration_since(epoch).ok_or(Error::Regressed)?;
        if elapsed < self.latest {
            return Err(Error::Regressed);
        }
        self.latest = elapsed;
        Ok(Sample {
            source: self.source,
            elapsed,
        })
    }
}

fn coarsen(elapsed: Duration) -> Result<u64, Error> {
    let whole_buckets = elapsed.as_micros() / 100;
    let micros = whole_buckets.checked_mul(100).ok_or(Error::Overflow)?;
    u64::try_from(micros).map_err(|_| Error::Overflow)
}

fn native_now() -> Result<Instant, Error> {
    if cfg!(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    )) {
        Ok(Instant::now())
    } else {
        Err(Error::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    ))]
    #[test]
    fn first_document_creation_preserves_the_native_owners_epoch() {
        use crate::{
            DomHostedRuntimeLimits, DomManipulationRuntimeLimits, DomSession, DomSessionLimits,
            DomStyleRuntimeLimits,
        };
        use std::sync::atomic::AtomicBool;
        use surgeist_dom::{Dom, DomHostedLimits, DomHostedRecordLimits, DomLimits};

        let dom = Dom::new(DomLimits::new(512, 200_000, 1024, 1024, 1024, 64, 4_000_000).unwrap())
            .unwrap();
        let mut session = DomSession::new(
            dom,
            DomSessionLimits {
                source_events: crate::DomSourceEventRuntimeLimits {
                    jobs: 0,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                style: DomStyleRuntimeLimits {
                    roots: 4,
                    completions: 32,
                    reports: 32,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                dom_manipulation: DomManipulationRuntimeLimits {
                    tasks: 32,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
                hosted: DomHostedRuntimeLimits {
                    canonical: DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                        16_000_000,
                        16_000_000,
                        1_000_000,
                    ),
                    tasks: 32,
                    storage_bytes: 16_000_000,
                    scratch_bytes: 16_000_000,
                    work: 1_000_000,
                },
            },
        )
        .unwrap();
        assert_eq!(session.snapshot().nodes().count(), 0);
        let cancel = AtomicBool::new(false);
        let mut hosted = session.hosted_execution();
        let owner = hosted.state().unwrap();
        let before = {
            let state = owner.borrow();
            (state.source.id(), state.clock.epoch)
        };

        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel).unwrap();
        let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
        let settings = hosted.settings(&dom, top.global(), &cancel).unwrap();
        assert!(settings.environment_ready());
        assert_eq!(settings.clock_source(), before.0);

        // Document creation samples the established owner domain; it cannot
        // establish or rebase the epoch shared by the owner's operations.
        let after = {
            let state = owner.borrow();
            (state.source.id(), state.clock.epoch)
        };
        assert_eq!(after, before);
    }

    #[test]
    fn native_nonisolated_clock_floors_to_literal_hundred_microsecond_boundaries() {
        for (nanoseconds, expected_microseconds) in [
            (0, 0),
            (1, 0),
            (99_999, 0),
            (100_000, 100),
            (100_001, 100),
            (199_999, 100),
            (200_000, 200),
            (1_999_999, 1_900),
        ] {
            assert_eq!(
                coarsen(Duration::from_nanos(nanoseconds)),
                Ok(expected_microseconds)
            );
        }
    }

    #[test]
    fn unrepresentable_native_elapsed_time_is_rejected_without_wrapping() {
        assert_eq!(coarsen(Duration::MAX), Err(Error::Overflow));
        let final_bucket = u64::MAX / 100 * 100;
        assert_eq!(
            coarsen(Duration::from_micros(final_bucket)),
            Ok(final_bucket)
        );
        assert_eq!(coarsen(Duration::from_secs(u64::MAX)), Err(Error::Overflow));
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    ))]
    mod observations {
        use super::*;
        use std::sync::atomic::AtomicBool;
        use surgeist_dom::{
            Dom, DomHostedLimits, DomHostedRecordLimits, DomHostedSource, DomLimits,
        };

        // Controlled native observations exercise the production checked
        // observation/coarsening path. They do not certify production sampling.
        fn clock(epoch: Result<Instant, Error>) -> (Dom, DomHostedSource, Clock) {
            let mut dom =
                Dom::new(DomLimits::new(512, 200_000, 1024, 1024, 1024, 64, 4_000_000).unwrap())
                    .unwrap();
            let source = dom
                .register_hosted_source(
                    DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                        16_000_000,
                        16_000_000,
                        1_000_000,
                    ),
                    &AtomicBool::new(false),
                )
                .unwrap();
            let clock = Clock {
                source: source.id(),
                epoch,
                latest: Duration::ZERO,
                observations: None,
            };
            (dom, source, clock)
        }

        #[test]
        fn repeated_zero_and_within_bucket_observations_remain_valid() {
            let epoch = Instant::now();
            let (_dom, source, mut clock) = clock(Ok(epoch));
            for (nanoseconds, microseconds) in [
                (0, 0),
                (0, 0),
                (99_999, 0),
                (99_999, 0),
                (100_000, 100),
                (100_001, 100),
                (100_001, 100),
            ] {
                let sample = clock
                    .observe(Ok(epoch
                        .checked_add(Duration::from_nanos(nanoseconds))
                        .unwrap()))
                    .unwrap();
                assert_eq!(sample.elapsed, Duration::from_nanos(nanoseconds));
                assert_eq!(
                    sample.coarsen(),
                    Ok(CoarsenedSample {
                        source: source.id(),
                        microseconds,
                    })
                );
            }
        }

        #[test]
        fn later_observation_uses_absolute_epoch_and_preserves_retained_creation_sample() {
            let epoch = Instant::now();
            let (_dom, source, mut clock) = clock(Ok(epoch));
            let creation = clock
                .observe(Ok(epoch
                    .checked_add(Duration::from_nanos(199_999))
                    .unwrap()))
                .unwrap();
            let loading = clock
                .observe(Ok(epoch
                    .checked_add(Duration::from_nanos(350_001))
                    .unwrap()))
                .unwrap();
            assert_eq!(
                creation.coarsen(),
                Ok(CoarsenedSample {
                    source: source.id(),
                    microseconds: 100,
                })
            );
            assert_eq!(
                loading.coarsen(),
                Ok(CoarsenedSample {
                    source: source.id(),
                    microseconds: 300,
                })
            );
        }

        #[test]
        fn observations_before_epoch_or_last_success_are_rejected_without_clamping() {
            let epoch = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
            let (_dom, _source, mut clock) = clock(Ok(epoch));
            assert_eq!(
                clock.observe(Ok(epoch.checked_sub(Duration::from_nanos(1)).unwrap())),
                Err(Error::Regressed)
            );
            let current = epoch.checked_add(Duration::from_nanos(200_001)).unwrap();
            let accepted = clock.observe(Ok(current)).unwrap();
            assert_eq!(
                clock.observe(Ok(epoch
                    .checked_add(Duration::from_nanos(199_999))
                    .unwrap())),
                Err(Error::Regressed)
            );
            assert_eq!(clock.observe(Ok(current)), Ok(accepted));
        }

        #[test]
        fn unavailable_observation_retains_previous_monotonic_observation() {
            let epoch = Instant::now();
            let (_dom, _source, mut clock) = clock(Ok(epoch));
            let current = epoch.checked_add(Duration::from_nanos(200_001)).unwrap();
            let accepted = clock.observe(Ok(current)).unwrap();
            assert_eq!(
                clock.observe(Err(Error::Unavailable)),
                Err(Error::Unavailable)
            );
            assert_eq!(
                clock.observe(Ok(epoch
                    .checked_add(Duration::from_nanos(199_999))
                    .unwrap())),
                Err(Error::Regressed)
            );
            assert_eq!(clock.observe(Ok(current)), Ok(accepted));
        }

        #[test]
        fn unavailable_owner_epoch_cannot_be_replaced_by_a_later_observation() {
            let (_dom, _source, mut clock) = clock(Err(Error::Unavailable));
            assert_eq!(clock.observe(Ok(Instant::now())), Err(Error::Unavailable));
            assert_eq!(
                clock.observe(Err(Error::Unavailable)),
                Err(Error::Unavailable)
            );
        }

        #[test]
        fn equal_elapsed_observations_retain_their_distinct_source_domains() {
            let epoch = Instant::now();
            let (_first_dom, first_source, mut first) = clock(Ok(epoch));
            let (_second_dom, second_source, mut second) = clock(Ok(epoch));
            let now = epoch.checked_add(Duration::from_nanos(100_001)).unwrap();
            let first_sample = first.observe(Ok(now)).unwrap().coarsen().unwrap();
            let second_sample = second.observe(Ok(now)).unwrap().coarsen().unwrap();
            assert_ne!(first_source.id(), second_source.id());
            assert_eq!(first_sample.source, first_source.id());
            assert_eq!(second_sample.source, second_source.id());
            assert_eq!(first_sample.microseconds, 100);
            assert_eq!(second_sample.microseconds, 100);
            assert_ne!(first_sample, second_sample);
        }

        #[test]
        fn retained_sample_coarsening_propagates_range_overflow() {
            let (_dom, source, _clock) = clock(Ok(Instant::now()));
            let sample = Sample {
                source: source.id(),
                elapsed: Duration::MAX,
            };
            assert_eq!(sample.coarsen(), Err(Error::Overflow));
        }

        mod phases {
            use super::*;
            use crate::{
                DomHostedRuntimeError, DomHostedRuntimeLimits, DomHostedTopLevel,
                DomManipulationRuntimeLimits, DomSession, DomSessionLimits, DomSourceDriveProgress,
                DomSourceRuntimeError, DomSourceServices, DomStyleRuntimeLimits, HostedRuntime,
            };
            use surgeist_dom::{
                DomBatch, DomHostedChildStep, DomHostedCreationStep, DomHostedError,
                DomHostedOperationId, DomHostedResource, DomSourceEdit, DomSourceLimits,
                DomSourceOperationId, DomSourceProgress, DomSourceRequest, NodeId,
            };

            fn limits(entries: usize, globals: usize) -> DomHostedRuntimeLimits {
                DomHostedRuntimeLimits {
                    canonical: DomHostedLimits::new(
                        DomHostedRecordLimits::new(32, 32, 32, entries, globals, 32, 32),
                        32_000_000,
                        32_000_000,
                        4_000_000,
                    ),
                    tasks: 32,
                    storage_bytes: 32_000_000,
                    scratch_bytes: 32_000_000,
                    work: 4_000_000,
                }
            }

            fn session() -> DomSession {
                DomSession::new(
                    Dom::new(
                        DomLimits::new(512, 1_000_000, 1024, 1024, 1024, 64, 4_000_000).unwrap(),
                    )
                    .unwrap(),
                    DomSessionLimits {
                        source_events: crate::DomSourceEventRuntimeLimits {
                            jobs: 0,
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
                        style: DomStyleRuntimeLimits {
                            roots: 4,
                            completions: 32,
                            reports: 32,
                            storage_bytes: 32_000_000,
                            work: 4_000_000,
                            css_input: surgeist_css::CssUtf16InputLimits::default(),
                        },
                        dom_manipulation: DomManipulationRuntimeLimits {
                            tasks: 32,
                            storage_bytes: 32_000_000,
                            work: 4_000_000,
                        },
                        hosted: limits(32, 32),
                    },
                )
                .unwrap()
            }

            fn script(hosted: &HostedRuntime, observations: &[Result<u64, Error>]) {
                let owner = hosted.state().unwrap();
                let mut state = owner.borrow_mut();
                let epoch = state.clock.epoch.unwrap();
                assert!(state.clock.observations.is_none());
                state.clock.observations = Some(
                    observations
                        .iter()
                        .map(|observation| {
                            observation.map(|nanoseconds| {
                                epoch
                                    .checked_add(Duration::from_nanos(nanoseconds))
                                    .unwrap()
                            })
                        })
                        .collect(),
                );
            }

            fn foreign_sample() -> (DomSession, Sample) {
                let session = session();
                let owner = session.hosted_execution().state().unwrap();
                let sample = owner.borrow_mut().clock.sample().unwrap();
                (session, sample)
            }

            fn top_phase(
                hosted: &HostedRuntime,
                dom: &crate::dom_event::DomEventDom<'_>,
            ) -> (DomHostedOperationId, &'static str) {
                let owner = hosted.state().unwrap();
                let state = owner.borrow();
                let lease = state.creation.as_ref().unwrap().lease.as_ref().unwrap();
                match lease.step(dom.raw()).unwrap() {
                    DomHostedCreationStep::NativeExecution(request) => {
                        (request.operation(), "execution")
                    }
                    DomHostedCreationStep::FinishLoading(request) => {
                        (request.operation(), "loading")
                    }
                    DomHostedCreationStep::InitialEntry(request) => (request.operation(), "entry"),
                    _ => panic!("unexpected actual top-level timing phase"),
                }
            }

            fn source_limits() -> DomSourceLimits {
                DomSourceLimits::new(32_000_000, 1_000_000, 20_000_000, 256, 4096, 4096, 1)
            }

            fn child_phase(
                session: &mut DomSession,
                operation: DomSourceOperationId,
            ) -> (u64, &'static str) {
                let cancel = AtomicBool::new(false);
                let mut dom = session.dom();
                let mut lease = dom
                    .raw_mut()
                    .resume_source_edit(operation, source_limits(), &cancel)
                    .unwrap();
                let DomSourceProgress::Request(DomSourceRequest::HostedChild(child)) =
                    lease.advance(&cancel).unwrap()
                else {
                    panic!("the actual Source cursor must retain its child request")
                };
                assert_eq!(child.operation(), operation);
                let phase = match lease.hosted_child_step(&child).unwrap() {
                    DomHostedChildStep::NativeExecution(_) => "execution",
                    DomHostedChildStep::FinishLoading(_) => "loading",
                    DomHostedChildStep::InitialEntry(_) => "entry",
                    _ => panic!("unexpected actual child timing phase"),
                };
                (child.occurrence(), phase)
            }

            fn create_top(session: &mut DomSession) -> DomHostedTopLevel {
                let cancel = AtomicBool::new(false);
                let mut hosted = session.hosted_execution();
                let mut dom = session.dom();
                let host = hosted.create_host(&mut dom, &cancel).unwrap();
                hosted.create_top_level(&mut dom, host, &cancel).unwrap()
            }

            fn child_container(
                session: &mut DomSession,
                top: DomHostedTopLevel,
            ) -> (NodeId, NodeId) {
                let snapshot = session.snapshot();
                let html = snapshot.node(top.document().node()).unwrap().children()[0];
                let body = snapshot.node(html).unwrap().children()[1];
                let mut batch = DomBatch::new();
                let closed = batch.create_html_document(top.document(), None);
                let iframe =
                    batch.create_element_ns(&closed, surgeist_html::Namespace::html(), "iframe");
                let commit = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
                (body, commit.created(&iframe).unwrap().unwrap())
            }

            #[test]
            fn top_level_timing_survives_real_admission_domain_and_loading_refusals() {
                let cancel = AtomicBool::new(false);
                let mut session = session();
                let mut hosted = session.hosted_execution();
                // The real owner/epoch and Source remain intact. Only individual
                // native observations are controlled; script exhaustion fails.
                script(
                    &hosted,
                    &[
                        Ok(199_999),
                        Err(Error::Unavailable),
                        Ok(350_001),
                        Ok(700_001),
                        Ok(900_001),
                    ],
                );
                let owner = hosted.state().unwrap();
                let mut dom = session.dom();
                let host = hosted.create_host(&mut dom, &cancel).unwrap();
                hosted.set_limits(&mut dom, limits(0, 0), &cancel).unwrap();
                assert!(matches!(
                    hosted.create_top_level(&mut dom, host, &cancel),
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Limit {
                        resource: DomHostedResource::Globals,
                        ..
                    }))
                ));
                let cursor = top_phase(&hosted, &dom);
                assert_eq!(cursor.1, "execution");
                let creation = owner.borrow().creation.as_ref().unwrap().sample.unwrap();
                assert_eq!(creation.elapsed, Duration::from_nanos(199_999));
                let (_foreign_session, foreign) = foreign_sample();
                assert_ne!(foreign.source, creation.source);
                // Malformed private raw input exercises the real checked
                // handoff. No public caller can replace this retained sample.
                owner.borrow_mut().creation.as_mut().unwrap().sample = Some(foreign);
                let before = dom.snapshot().stamp();
                assert!(matches!(
                    hosted.resume_top_level_creation(&mut dom, &cancel),
                    Err(DomHostedRuntimeError::Binding)
                ));
                assert_eq!(dom.snapshot().stamp(), before);
                assert_eq!(top_phase(&hosted, &dom), cursor);
                assert_eq!(
                    owner.borrow().creation.as_ref().unwrap().sample,
                    Some(foreign)
                );
                owner.borrow_mut().creation.as_mut().unwrap().sample = Some(creation);

                hosted.set_limits(&mut dom, limits(0, 32), &cancel).unwrap();
                assert!(matches!(
                    hosted.resume_top_level_creation(&mut dom, &cancel),
                    Err(DomHostedRuntimeError::ClockUnavailable)
                ));
                assert_eq!(top_phase(&hosted, &dom), (cursor.0, "loading"));
                {
                    let state = owner.borrow();
                    let retained = state.creation.as_ref().unwrap();
                    assert_eq!(retained.sample, Some(creation));
                    assert_eq!(retained.finished_sample, None);
                    let native = state.hosts[0].top_levels[0]
                        .group
                        .as_ref()
                        .unwrap()
                        .global()
                        .unwrap();
                    assert!(native.settings.ready);
                    assert_eq!(native.settings.navigation_start.microseconds, 100);
                    assert_eq!(native.settings.navigation_start.source, creation.source);
                }
                assert!(matches!(
                    hosted.resume_top_level_creation(&mut dom, &cancel),
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Limit {
                        resource: DomHostedResource::Entries,
                        ..
                    }))
                ));
                assert_eq!(top_phase(&hosted, &dom), (cursor.0, "entry"));
                let finished = owner.borrow().creation.as_ref().unwrap().finished_sample;
                assert_eq!(
                    finished,
                    Some(CoarsenedSample {
                        source: creation.source,
                        microseconds: 300
                    })
                );
                assert!(matches!(
                    hosted.resume_top_level_creation(&mut dom, &cancel),
                    Err(DomHostedRuntimeError::Dom(DomHostedError::Limit {
                        resource: DomHostedResource::Entries,
                        ..
                    }))
                ));
                assert_eq!(
                    owner.borrow().creation.as_ref().unwrap().finished_sample,
                    finished
                );
                hosted
                    .set_limits(&mut dom, limits(32, 32), &cancel)
                    .unwrap();
                let top = hosted.resume_top_level_creation(&mut dom, &cancel).unwrap();
                let settings = hosted.settings(&dom, top.global(), &cancel).unwrap();
                assert_eq!(settings.time_origin_microseconds(), 100);
                assert_eq!(settings.clock_source(), creation.source);
                assert!(settings.document().is_fully_active().unwrap());
                let later = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
                let settings = hosted.settings(&dom, later.global(), &cancel).unwrap();
                assert_ne!(later.document(), top.document());
                assert_eq!(settings.time_origin_microseconds(), 700);
                assert_eq!(settings.clock_source(), creation.source);
                assert!(
                    owner
                        .borrow()
                        .clock
                        .observations
                        .as_ref()
                        .unwrap()
                        .is_empty()
                );
            }

            #[test]
            fn child_timing_survives_real_admission_domain_and_loading_refusals() {
                let cancel = AtomicBool::new(false);
                let mut session = session();
                let mut hosted = session.hosted_execution();
                script(
                    &hosted,
                    &[
                        Ok(100_001),
                        Ok(200_001),
                        Ok(999_999),
                        Err(Error::Unavailable),
                        Ok(1_200_001),
                    ],
                );
                let top = create_top(&mut session);
                let (body, iframe) = child_container(&mut session, top);
                let owner = hosted.state().unwrap();
                hosted
                    .set_limits(&mut session.dom(), limits(1, 1), &cancel)
                    .unwrap();
                let mut source = session.source_execution();
                let mut policy = |_: &surgeist_dom::DomStyleSource| None;
                let mut services = DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None,
                };
                assert!(matches!(
                    source.apply(
                        &mut session.dom(),
                        DomSourceEdit::insert_before(iframe, body, None),
                        source_limits(),
                        &mut services,
                        &cancel
                    ),
                    Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::Limit {
                            resource: DomHostedResource::Globals,
                            ..
                        }
                    )))
                ));
                let operation = session.snapshot().incomplete_source_operation().unwrap();
                let cursor = child_phase(&mut session, operation);
                assert_eq!(cursor.1, "execution");
                let creation = {
                    let state = owner.borrow();
                    let row = state
                        .children
                        .iter()
                        .find(|row| row.operation == operation && row.occurrence == cursor.0)
                        .unwrap();
                    row.sample.unwrap()
                };
                assert_eq!(creation.elapsed, Duration::from_nanos(999_999));
                let (_foreign_session, foreign) = foreign_sample();
                assert_ne!(foreign.source, creation.source);
                // Save/restore the genuine candidate around private malformed
                // raw-input injection; the Source cursor is never replaced.
                owner
                    .borrow_mut()
                    .children
                    .iter_mut()
                    .find(|row| row.operation == operation && row.occurrence == cursor.0)
                    .unwrap()
                    .sample = Some(foreign);
                let before = session.snapshot().stamp();
                assert!(matches!(
                    source.resume(
                        &mut session.dom(),
                        operation,
                        source_limits(),
                        &mut services,
                        &cancel
                    ),
                    Err(DomSourceRuntimeError::Hosted(
                        DomHostedRuntimeError::Binding
                    ))
                ));
                assert_eq!(session.snapshot().stamp(), before);
                assert_eq!(child_phase(&mut session, operation), cursor);
                {
                    let mut state = owner.borrow_mut();
                    let row = state
                        .children
                        .iter_mut()
                        .find(|row| row.operation == operation && row.occurrence == cursor.0)
                        .unwrap();
                    assert_eq!(row.sample, Some(foreign));
                    row.sample = Some(creation);
                }
                hosted
                    .set_limits(&mut session.dom(), limits(1, 32), &cancel)
                    .unwrap();
                assert!(matches!(
                    source.resume(
                        &mut session.dom(),
                        operation,
                        source_limits(),
                        &mut services,
                        &cancel
                    ),
                    Err(DomSourceRuntimeError::Hosted(
                        DomHostedRuntimeError::ClockUnavailable
                    ))
                ));
                assert_eq!(child_phase(&mut session, operation), (cursor.0, "loading"));
                {
                    let state = owner.borrow();
                    let row = state
                        .children
                        .iter()
                        .find(|row| row.operation == operation && row.occurrence == cursor.0)
                        .unwrap();
                    assert_eq!(row.sample, Some(creation));
                    assert_eq!(row.finished_sample, None);
                    let global = row.global.unwrap();
                    let native = state.hosts[0].top_levels[0]
                        .group
                        .as_ref()
                        .unwrap()
                        .clusters
                        .iter()
                        .flat_map(|cluster| &cluster.agent.globals)
                        .find(|native| native.id == global)
                        .unwrap();
                    assert!(native.settings.ready);
                    assert_eq!(
                        native.settings.navigation_start,
                        CoarsenedSample {
                            source: creation.source,
                            microseconds: 900
                        }
                    );
                }
                assert!(matches!(
                    source.resume(
                        &mut session.dom(),
                        operation,
                        source_limits(),
                        &mut services,
                        &cancel
                    ),
                    Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::Limit {
                            resource: DomHostedResource::Entries,
                            ..
                        }
                    )))
                ));
                assert_eq!(child_phase(&mut session, operation), (cursor.0, "entry"));
                let finished = {
                    let state = owner.borrow();
                    state
                        .children
                        .iter()
                        .find(|row| row.operation == operation && row.occurrence == cursor.0)
                        .unwrap()
                        .finished_sample
                };
                assert_eq!(
                    finished,
                    Some(CoarsenedSample {
                        source: creation.source,
                        microseconds: 1_200
                    })
                );
                assert!(matches!(
                    source.resume(
                        &mut session.dom(),
                        operation,
                        source_limits(),
                        &mut services,
                        &cancel
                    ),
                    Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::Limit {
                            resource: DomHostedResource::Entries,
                            ..
                        }
                    )))
                ));
                {
                    let state = owner.borrow();
                    let row = state
                        .children
                        .iter()
                        .find(|row| row.operation == operation && row.occurrence == cursor.0)
                        .unwrap();
                    assert_eq!(row.sample, Some(creation));
                    assert_eq!(row.finished_sample, finished);
                }
                hosted
                    .set_limits(&mut session.dom(), limits(32, 32), &cancel)
                    .unwrap();
                assert!(matches!(
                    source
                        .resume(
                            &mut session.dom(),
                            operation,
                            source_limits(),
                            &mut services,
                            &cancel
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                let snapshot = session.snapshot();
                let child = snapshot
                    .node(iframe)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .child_navigable()
                    .unwrap()
                    .unwrap();
                let global = snapshot.hosted_navigable(child).unwrap().global();
                let settings = hosted.settings(&session.dom(), global, &cancel).unwrap();
                assert_eq!(settings.time_origin_microseconds(), 900);
                assert_eq!(settings.clock_source(), creation.source);
                assert!(settings.document().is_fully_active().unwrap());
                assert!(
                    owner
                        .borrow()
                        .clock
                        .observations
                        .as_ref()
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }
}

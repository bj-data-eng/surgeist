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
            DomElementTaskRuntimeLimits, DomHostedRuntimeLimits, DomSession, DomSessionLimits,
            DomStyleRuntimeLimits,
        };
        use std::sync::atomic::AtomicBool;
        use surgeist_dom::{Dom, DomHostedLimits, DomHostedRecordLimits, DomLimits};

        let dom = Dom::new(
            DomLimits::new(
                512,
                200_000,
                1024,
                1024,
                1024,
                64,
                4_000_000,
                surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut session = DomSession::new(
            dom,
            DomSessionLimits {
                authored_events: crate::DomEventStorageLimits {
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                },
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
                    css_input: surgeist_css::source::CssUtf16InputLimits::default(),
                },
                element_tasks: DomElementTaskRuntimeLimits {
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
            let mut dom = Dom::new(
                DomLimits::new(
                    512,
                    200_000,
                    1024,
                    1024,
                    1024,
                    64,
                    4_000_000,
                    surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                        .unwrap(),
                )
                .unwrap(),
            )
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
                DomElementTaskRuntimeLimits, DomHostedRuntimeError, DomHostedRuntimeLimits,
                DomHostedTopLevel, DomSession, DomSessionLimits, DomSourceDriveProgress,
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
                session_with_dom_limits(std::convert::identity)
            }

            fn session_with_dom_limits(
                configure: impl FnOnce(DomLimits) -> DomLimits,
            ) -> DomSession {
                DomSession::new(
                    Dom::new(configure(
                        DomLimits::new(
                            512,
                            1_000_000,
                            1024,
                            1024,
                            1024,
                            64,
                            4_000_000,
                            surgeist_dom::DomNameLimits::new(
                                16_777_216, 16_384, 4_000_000, 1_048_576,
                            )
                            .unwrap(),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
                    DomSessionLimits {
                        authored_events: crate::DomEventStorageLimits {
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
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
                            css_input: surgeist_css::source::CssUtf16InputLimits::default(),
                        },
                        element_tasks: DomElementTaskRuntimeLimits {
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
                let iframe = batch.create_element_ns(
                    &closed,
                    surgeist_html::values::Namespace::html(),
                    "iframe",
                );
                let commit = session.dom().apply(batch, &AtomicBool::new(false)).unwrap();
                (body, commit.created(&iframe).unwrap().unwrap())
            }

            #[test]
            fn event_projection_preserves_an_earlier_occurrence_in_the_actual_global_domain() {
                let cancel = AtomicBool::new(false);
                let mut session = session();
                let mut hosted = session.hosted_execution();
                script(
                    &hosted,
                    &[Ok(199_999), Ok(350_001), Ok(700_001), Ok(900_001)],
                );
                let owner = hosted.state().unwrap();
                let earlier = owner.borrow_mut().clock.sample().unwrap();
                let mut dom = session.dom();
                let host = hosted.create_host(&mut dom, &cancel).unwrap();
                let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
                let settings = hosted.settings(&dom, top.global(), &cancel).unwrap();
                let origin = owner.borrow().hosts[0].top_levels[0]
                    .group
                    .as_ref()
                    .unwrap()
                    .global()
                    .unwrap()
                    .settings
                    .navigation_start;
                let observation = crate::dom_lifecycle::event::creation_observation(
                    top.global(),
                    origin,
                    earlier,
                )
                .unwrap();
                assert_eq!(observation.global(), top.global());
                assert_eq!(observation.clock_source(), settings.clock_source());
                assert_eq!(observation.time_origin_microseconds(), 300);
                assert_eq!(observation.occurrence_microseconds(), 100);
                assert_eq!(observation.relative_microseconds(), -200);
                assert_eq!(observation.milliseconds(), -0.2);
                let (_foreign_session, foreign) = foreign_sample();
                assert!(matches!(
                    crate::dom_lifecycle::event::creation_observation(
                        top.global(),
                        origin,
                        foreign
                    ),
                    Err(DomHostedRuntimeError::Protocol)
                ));
                let event = hosted
                    .create_event(
                        &mut dom,
                        top.global(),
                        &[],
                        crate::DomEventInit::default(),
                        &cancel,
                    )
                    .unwrap();
                assert_eq!(event.time_stamp().unwrap(), 0.6);
            }

            #[test]
            fn hosted_details_samples_once_when_its_funded_event_is_formed() {
                use crate::{
                    DomElementTaskEventContext, DomElementTaskProgress, DomElementTaskRunLimits,
                    DomElementTaskRuntime, DomElementTaskRuntimeError, DomElementTaskSourceKind,
                    DomEventCallback, DomEventCreationObservation, DomEventDispatchLimits,
                    DomEventDispatchOutcome, DomEventDispatcher, DomEventInit,
                    DomEventListenerOptions, DomEventRegistryLimits, DomEventResource,
                    DomEventType,
                };

                struct Context {
                    tasks: DomElementTaskRuntime,
                    samples: Vec<Option<DomEventCreationObservation>>,
                }
                impl DomElementTaskEventContext for Context {
                    fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
                        &mut self.tasks
                    }
                }

                let cancel = AtomicBool::new(false);
                let mut session = session_with_dom_limits(|limits| {
                    limits
                        .with_details_toggle_intentions(4)
                        .unwrap()
                        .with_dispatch_custody(
                            surgeist_dom::DomDispatchCustodyLimits::new(
                                8, 64, 1_000_000, 1_000_000, 100_000,
                            )
                            .unwrap(),
                        )
                });
                let mut hosted = session.hosted_execution();
                script(
                    &hosted,
                    &[
                        Ok(150_001),
                        Ok(260_001),
                        Ok(700_001),
                        Ok(1_245_001),
                        Ok(2_360_001),
                    ],
                );
                let host = hosted.create_host(&mut session.dom(), &cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, &cancel)
                    .unwrap();
                let snapshot = session.snapshot();
                let html = snapshot
                    .document(top.document())
                    .unwrap()
                    .document_element()
                    .unwrap();
                let body = snapshot.node(html).unwrap().children()[1];
                let mut batch = DomBatch::new();
                let target = batch.create_element_ns(
                    top.document(),
                    surgeist_html::values::Namespace::html(),
                    "details",
                );
                batch.insert_before(&target, body, None);
                batch.set_attribute(&target, "open", "");
                let target = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&target)
                    .unwrap()
                    .unwrap();
                let mut tasks = session.element_task_execution();
                assert_eq!(tasks.ingest(&mut session.dom(), &cancel).unwrap(), 1);
                let source = DomElementTaskSourceKind::DomManipulation;
                let queued = tasks.pending_task(source, 0).unwrap().unwrap();
                let mut context = Context {
                    tasks,
                    samples: Vec::new(),
                };
                let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 2,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 1_000_000,
                    work: 100_000,
                })
                .unwrap();
                for _ in 0..2 {
                    registry
                        .add_listener(
                            &mut session.dom(),
                            target,
                            DomEventType::TOGGLE,
                            DomEventCallback::new(|_, _, context: &mut Context, event| {
                                context.samples.push(event.creation().copied());
                                Ok(())
                            }),
                            DomEventListenerOptions::default(),
                        )
                        .unwrap();
                }
                let mut observe_clock = |session: &mut DomSession| {
                    hosted
                        .create_event(
                            &mut session.dom(),
                            top.global(),
                            &[],
                            DomEventInit::default(),
                            &cancel,
                        )
                        .unwrap()
                        .creation()
                        .unwrap()
                        .occurrence_microseconds()
                };
                assert_eq!(observe_clock(&mut session), 700);

                let dispatch = DomEventDispatchLimits {
                    path_items: 16,
                    snapshot_listeners: 2,
                    failure_records: 2,
                    callbacks: 2,
                    work: 10_000,
                };
                let mut run = |session: &mut DomSession, dispatch| {
                    DomElementTaskRuntime::run_next(
                        &mut session.dom(),
                        source,
                        &mut registry,
                        &mut context,
                        DomElementTaskRunLimits {
                            dispatch,
                            source: source_limits(),
                        },
                        &cancel,
                    )
                };
                assert!(matches!(
                    run(
                        &mut session,
                        DomEventDispatchLimits {
                            path_items: 1_000_000,
                            ..dispatch
                        },
                    ),
                    Err(DomElementTaskRuntimeError::Admission)
                ));
                assert!(matches!(
                    run(
                        &mut session,
                        DomEventDispatchLimits {
                            callbacks: 1,
                            ..dispatch
                        },
                    ).unwrap(),
                    DomElementTaskProgress::Retained {
                        task,
                        dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
                        ..
                    } if task == queued
                ));
                let DomElementTaskProgress::Complete { task, receipt, .. } =
                    run(&mut session, dispatch).unwrap()
                else {
                    panic!("the original Details task must complete");
                };
                assert_eq!(task, queued);
                assert_eq!(receipt.callbacks(), 2);
                assert!(receipt.failures().is_empty());
                let [Some(first), Some(second)] = context.samples.as_slice() else {
                    panic!("both callbacks must observe the funded Event's creation");
                };
                assert_eq!(first, second);
                assert_eq!(first.global(), top.global());
                assert_eq!(first.time_origin_microseconds(), 100);
                assert_eq!(first.occurrence_microseconds(), 1_200);
                assert_eq!(first.relative_microseconds(), 1_100);
                assert_eq!(observe_clock(&mut session), 2_300);
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
                    meta: None,
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
                    Err(crate::DomSourceDriveError::Source(
                        DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                            DomHostedError::Limit {
                                resource: DomHostedResource::Globals,
                                ..
                            }
                        ))
                    ))
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
                // Save/restore the candidate around private malformed
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
                    Err(crate::DomSourceDriveError::Source(
                        DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Binding)
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
                    Err(crate::DomSourceDriveError::Source(
                        DomSourceRuntimeError::Hosted(DomHostedRuntimeError::ClockUnavailable)
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
                    Err(crate::DomSourceDriveError::Source(
                        DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                            DomHostedError::Limit {
                                resource: DomHostedResource::Entries,
                                ..
                            }
                        ))
                    ))
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
                    Err(crate::DomSourceDriveError::Source(
                        DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                            DomHostedError::Limit {
                                resource: DomHostedResource::Entries,
                                ..
                            }
                        ))
                    ))
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

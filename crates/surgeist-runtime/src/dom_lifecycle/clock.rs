//! The one native monotonic epoch owned by a hosted session.
//!
//! The selected non-isolated profile floors exposed timestamps to 100
//! microseconds without jitter. Scheduling uses native elapsed time.
use std::time::{Duration, Instant};
use surgeist_dom::DomHostedSourceId;

/// A real observation of the epoch bound to the original hosted source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Sample {
    source: DomHostedSourceId,
    elapsed: Duration,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CoarsenedSample {
    pub source: DomHostedSourceId,
    pub microseconds: u64,
}
impl Sample {
    #[cfg(feature = "document-scripting")]
    pub(crate) const fn source(self) -> DomHostedSourceId {
        self.source
    }

    #[cfg(feature = "document-scripting")]
    pub(crate) const fn elapsed(self) -> Duration {
        self.elapsed
    }

    /// Derive source timing from the retained original sample at its actual
    /// coarsening phase; neither resample nor expose the finer private moment.
    pub(super) fn coarsen(&self) -> Result<CoarsenedSample, Error> {
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

    pub const fn source(&self) -> DomHostedSourceId {
        self.source
    }

    #[cfg(all(test, feature = "document-scripting"))]
    pub(super) fn script_offsets(&mut self, offsets: impl IntoIterator<Item = Duration>) {
        assert!(self.observations.is_none());
        let latest = self.epoch.unwrap().checked_add(self.latest).unwrap();
        self.observations = Some(
            offsets
                .into_iter()
                .map(|offset| Ok(latest.checked_add(offset).unwrap()))
                .collect(),
        );
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

            #[cfg(feature = "document-scripting")]
            #[test]
            fn window_timer_samples_preserve_native_resolution_and_reject_regression() {
                let mut session = session();
                let mut hosted = session.hosted_execution();
                let mut dom = session.dom();
                let cancel = AtomicBool::new(false);
                let host = hosted.create_host(&mut dom, &cancel).unwrap();
                let top = hosted.create_top_level(&mut dom, host, &cancel).unwrap();
                let settings = hosted.settings(&dom, top.global(), &cancel).unwrap();
                hosted.script_window_timer_clock([0, 51, 50, 52].map(Duration::from_nanos));

                let sample = || hosted.sample_window_timer(&dom, top.global(), &cancel);
                let first = sample().unwrap();
                assert_eq!(first.source(), settings.clock_source());
                let second = sample().unwrap();
                assert_eq!(second.elapsed() - first.elapsed(), Duration::from_nanos(51));
                assert_eq!(sample(), Err(DomHostedRuntimeError::ClockRegressed));
                let fourth = sample().unwrap();
                assert_eq!(fourth.elapsed() - first.elapsed(), Duration::from_nanos(52));
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

            fn constructor_lookup_steps(
                dom: &crate::DomEventDom<'_>,
                top: DomHostedTopLevel,
                legacy: bool,
                cancel: &AtomicBool,
            ) -> u64 {
                let mut steps = 0;
                let mut step = || {
                    steps += 1;
                    Ok::<(), DomHostedError>(())
                };
                if legacy {
                    dom.raw()
                        .qualify_native_dispatch_document(top.document(), cancel, &mut step)
                        .unwrap();
                } else {
                    let document = dom
                        .raw()
                        .dispatch_global_document_with(top.global(), cancel, &mut step)
                        .unwrap();
                    assert_eq!(document, top.document());
                }
                assert!(steps > 0);
                steps
            }

            fn constructor_host(
                cancel: &AtomicBool,
            ) -> (DomSession, HostedRuntime, DomHostedTopLevel) {
                let mut session = session();
                let mut hosted = session.hosted_execution();
                script(&hosted, &[Ok(150_001), Ok(260_001), Ok(430_001)]);
                let host = hosted.create_host(&mut session.dom(), cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, cancel)
                    .unwrap();
                (session, hosted, top)
            }

            fn check_event_constructor_lookup(legacy: bool) {
                use crate::{DomEventConstructionError, DomEventInit};

                let cancel = AtomicBool::new(false);
                let (mut session, mut hosted, top) = constructor_host(&cancel);
                let canonical_steps =
                    constructor_lookup_steps(&session.dom(), top, legacy, &cancel);
                let owner = hosted.state().unwrap();
                let before = session.snapshot().stamp();
                let clock_samples_left =
                    || owner.borrow().clock.observations.as_ref().unwrap().len();
                assert_eq!(clock_samples_left(), 1);
                let construct = |hosted: &mut HostedRuntime, session: &mut DomSession| {
                    if legacy {
                        hosted.create_legacy_event(
                            &mut session.dom(),
                            top.document(),
                            &b"Event".map(u16::from),
                            &cancel,
                        )
                    } else {
                        hosted.create_event(
                            &mut session.dom(),
                            top.global(),
                            &[u16::from(b'x')],
                            DomEventInit::default(),
                            &cancel,
                        )
                    }
                };
                // The first admitted top-level uses one Host, top-level, cluster,
                // and Global read, followed by one clock sample.
                let required = canonical_steps + 4 + 1;
                let short_cap = owner.borrow().work + required - 1;
                owner.borrow_mut().limits.work = short_cap;
                let result = construct(&mut hosted, &mut session);
                assert!(
                    matches!(
                        result,
                        Err(DomEventConstructionError::Hosted(
                            DomHostedRuntimeError::Admission
                        ))
                    ),
                    "one short of {canonical_steps} canonical steps, four Settings reads and one clock read: {result:?}"
                );
                assert_eq!(owner.borrow().work, short_cap);
                assert_eq!(clock_samples_left(), 1);
                assert_eq!(session.snapshot().stamp(), before);

                let exact_cap = short_cap + required;
                owner.borrow_mut().limits.work = exact_cap;
                let event = construct(&mut hosted, &mut session).unwrap();
                assert_eq!(event.creation().unwrap().global(), top.global());
                assert_eq!(event.creation().unwrap().occurrence_microseconds(), 400);
                if legacy {
                    assert!(event.event_type().code_units().is_empty());
                } else {
                    assert_eq!(event.event_type().code_units(), [u16::from(b'x')]);
                }
                assert_eq!(owner.borrow().work, exact_cap);
                assert_eq!(clock_samples_left(), 0);
                assert_eq!(session.snapshot().stamp(), before);
            }

            #[test]
            fn authored_event_requires_work_for_the_actual_creation_global_lookup() {
                check_event_constructor_lookup(false);
            }

            #[test]
            fn legacy_event_requires_work_for_the_receiving_documents_global_lookup() {
                check_event_constructor_lookup(true);
            }

            #[test]
            fn standalone_target_requires_global_lookup_work_before_spending_registry_work() {
                use crate::{
                    DomEventConstructionError, DomEventDispatcher, DomEventRegistryLimits,
                };

                let cancel = AtomicBool::new(false);
                let (mut session, mut hosted, top) = constructor_host(&cancel);
                let canonical_steps = constructor_lookup_steps(&session.dom(), top, false, &cancel);
                let owner = hosted.state().unwrap();
                let registry = DomEventDispatcher::<()>::new(DomEventRegistryLimits {
                    registrations: 1,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 1_000_000,
                    work: 1,
                })
                .unwrap();
                let before = session.snapshot().stamp();
                let short_cap = {
                    let mut state = owner.borrow_mut();
                    let cap = state.work + canonical_steps + 4 - 1;
                    state.limits.work = cap;
                    cap
                };
                let short = hosted.create_event_target(
                    &mut session.dom(),
                    &registry,
                    top.global(),
                    &cancel,
                );
                assert!(
                    matches!(
                        short,
                        Err(DomEventConstructionError::Hosted(
                            DomHostedRuntimeError::Admission
                        ))
                    ),
                    "one short of {canonical_steps} canonical steps and four Settings reads: {short:?}"
                );
                assert_eq!(owner.borrow().work, short_cap);
                let exact_cap = short_cap + canonical_steps + 4;
                owner.borrow_mut().limits.work = exact_cap;

                // Exact Hosted work and the same registry's only step must suffice.
                let standalone = hosted
                    .create_event_target(&mut session.dom(), &registry, top.global(), &cancel)
                    .unwrap();
                assert_eq!(standalone.relevant_global(), top.global());
                assert_eq!(owner.borrow().work, exact_cap);
                assert_eq!(owner.borrow().clock.observations.as_ref().unwrap().len(), 1);
                assert_eq!(session.snapshot().stamp(), before);
            }

            #[test]
            fn hosted_constructors_charge_the_active_source_for_each_canonical_lookup_step() {
                use crate::{
                    DomEventCallback, DomEventConstructionError, DomEventDispatchError,
                    DomEventDispatchLimits, DomEventDispatcher, DomEventInit,
                    DomEventListenerOptions, DomEventRegistryLimits, DomEventResource,
                    DomEventType, DomSourceEventRuntimeLimits, DomSourceEventServiceLimits,
                    DomSourceEventServiceProgress, DomSourceUnavailable,
                };

                let cancel = AtomicBool::new(false);
                let (mut session, hosted, top, dialog) = hosted_dialog_for_clock(
                    &[
                        Ok(150_001),
                        Ok(260_001),
                        Ok(430_001),
                        Err(Error::Unavailable),
                        Err(Error::Unavailable),
                    ],
                    &cancel,
                );
                let mut source = session.source_execution();
                let mut services = DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                };
                let mut listeners = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 1,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 1_000_000,
                    work: 100_000,
                })
                .unwrap();
                listeners
                    .add_listener(
                        &mut session.dom(),
                        dialog,
                        DomEventType::BEFORE_TOGGLE,
                        DomEventCallback::new(move |dom, _, observations: &mut Vec<_>, event| {
                            let mut hosted = hosted.clone();
                            let cancel = AtomicBool::new(false);
                            let spent = |dom: &crate::DomEventDom<'_>| {
                                dom.pending_source_edit().unwrap().spent_work()
                            };
                            let before_control = spent(dom);
                            let canonical_steps =
                                constructor_lookup_steps(dom, top, false, &cancel);
                            let global_work = spent(dom) - before_control;
                            assert_eq!(global_work, 1 + canonical_steps);
                            let before_control = spent(dom);
                            let canonical_steps = constructor_lookup_steps(dom, top, true, &cancel);
                            let legacy_work = spent(dom) - before_control;
                            assert_eq!(legacy_work, 1 + canonical_steps);

                            // Refuse after the complete lookup so Event and registry allocation
                            // have no effect on the Source work measured here.
                            let before = spent(dom);
                            let event_result = hosted.create_event(
                                dom,
                                top.global(),
                                &[],
                                DomEventInit::default(),
                                &cancel,
                            );
                            observations.push(("Event", spent(dom) - before, global_work + 4 + 1));
                            assert!(matches!(
                                event_result,
                                Err(DomEventConstructionError::Hosted(
                                    DomHostedRuntimeError::ClockUnavailable
                                ))
                            ));

                            let before = spent(dom);
                            let legacy_result = hosted.create_legacy_event(
                                dom,
                                top.document(),
                                &b"Event".map(u16::from),
                                &cancel,
                            );
                            observations.push((
                                "legacy Event",
                                spent(dom) - before,
                                legacy_work + 4 + 1,
                            ));
                            assert!(matches!(
                                legacy_result,
                                Err(DomEventConstructionError::Hosted(
                                    DomHostedRuntimeError::ClockUnavailable
                                ))
                            ));

                            let no_registry_work =
                                DomEventDispatcher::<()>::new(DomEventRegistryLimits {
                                    registrations: 0,
                                    pending_events: 0,
                                    nested_dispatches: 0,
                                    storage_bytes: 1_000_000,
                                    work: 0,
                                })
                                .unwrap();
                            let before = spent(dom);
                            let target_result = hosted.create_event_target(
                                dom,
                                &no_registry_work,
                                top.global(),
                                &cancel,
                            );
                            observations.push((
                                "EventTarget",
                                spent(dom) - before,
                                global_work + 4,
                            ));
                            assert!(matches!(
                                target_result,
                                Err(DomEventConstructionError::Event(
                                    DomEventDispatchError::Admission(DomEventResource::Work)
                                ))
                            ));
                            event.prevent_default();
                            Ok(())
                        }),
                        DomEventListenerOptions {
                            passive: Some(false),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: DomSourceUnavailable::Event,
                } = source
                    .apply(
                        &mut session.dom(),
                        DomSourceEdit::show_dialog(dialog),
                        source_limits(),
                        &mut services,
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("real Show must reach its beforetoggle request");
                };
                source
                    .raise_event_runtime_limits(
                        &session.dom(),
                        DomSourceEventRuntimeLimits {
                            jobs: 1,
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
                        &cancel,
                    )
                    .unwrap();
                let mut observations = Vec::new();
                let DomSourceEventServiceProgress::Serviced { receipt } = source
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut listeners,
                        &mut observations,
                        DomSourceEventServiceLimits {
                            source: source_limits(),
                            dispatch: DomEventDispatchLimits {
                                path_items: 16,
                                snapshot_listeners: 2,
                                failure_records: 2,
                                callbacks: 2,
                                work: 100_000,
                            },
                        },
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("the original real beforetoggle must complete");
                };
                assert_eq!(receipt.callbacks(), 1);
                assert!(receipt.failures().is_empty());
                assert!(receipt.default_prevented());
                assert!(matches!(
                    source
                        .resume(
                            &mut session.dom(),
                            request.operation(),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                assert_eq!(observations.len(), 3);
                let mismatches = observations
                    .iter()
                    .filter(|(_, actual, expected)| actual != expected)
                    .collect::<Vec<_>>();
                assert!(
                    mismatches.is_empty(),
                    "Source work (front, actual, expected): {mismatches:?}"
                );
            }

            #[test]
            fn direct_global_error_registration_charges_both_reads_to_active_source() {
                use crate::{
                    DomEventCallback, DomEventDispatchLimits, DomEventDispatcher,
                    DomEventListenerOptions, DomEventRegistryLimits, DomEventType,
                    DomSourceEventRuntimeLimits, DomSourceEventServiceLimits,
                    DomSourceEventServiceProgress, DomSourceUnavailable,
                };

                let cancel = AtomicBool::new(false);
                let (mut session, _, top, dialog) =
                    hosted_dialog_for_clock(&[Ok(150_001), Ok(260_001), Ok(430_001)], &cancel);
                let limits = session.dom().limits().with_event_recipients(
                    surgeist_dom::DomEventRecipientLimits::new(4, 2_000_000, 2_000_000, 100_000)
                        .unwrap(),
                );
                session.dom().raw_mut().set_limits(limits).unwrap();
                let retain = |dom: &mut crate::DomEventDom<'_>, target| {
                    dom.raw_mut()
                        .retain_event_recipient(
                            target,
                            DomEventType::LOAD.code_units(),
                            true,
                            &cancel,
                        )
                        .unwrap()
                        .unwrap()
                };
                let node_seed = retain(&mut session.dom(), dialog.into());
                let global_seed = retain(&mut session.dom(), top.global().into());

                let mut listeners = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 2,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 2_000_000,
                    work: 100_000,
                })
                .unwrap();
                let options = DomEventListenerOptions {
                    capture: true,
                    passive: Some(false),
                    ..Default::default()
                };
                listeners
                    .add_listener(
                        &mut session.dom(),
                        dialog,
                        DomEventType::BEFORE_TOGGLE,
                        DomEventCallback::new(
                            move |dom, listeners, observed: &mut Option<(u64, u64)>, event| {
                                let cancel = AtomicBool::new(false);
                                let spent = |dom: &crate::DomEventDom<'_>| {
                                    dom.pending_source_edit().unwrap().spent_work()
                                };
                                let lookup_steps =
                                    constructor_lookup_steps(dom, top, false, &cancel);
                                let measure =
                                    |dom: &mut crate::DomEventDom<'_>,
                                     listeners: &mut DomEventDispatcher<Option<(u64, u64)>>,
                                     target| {
                                        let before = spent(dom);
                                        let id = listeners
                                            .add_listener(
                                                dom,
                                                target,
                                                DomEventType::ERROR,
                                                DomEventCallback::new(|_, _, _, _| Ok(())),
                                                options,
                                            )
                                            .unwrap();
                                        assert!(listeners.remove_listener(id));
                                        spent(dom) - before
                                    };
                                let node =
                                    measure(dom, listeners, crate::DomEventTarget::Node(dialog));
                                let global = measure(
                                    dom,
                                    listeners,
                                    crate::DomEventTarget::Global(top.global()),
                                );
                                // Existing Global and Node recipients differ by the u64
                                // source-serial components, each with an extent and a
                                // leaf read. The same active Source also pays for both
                                // Global associations and their scope qualification.
                                let extra_index = 2 * 64 / u64::from(usize::BITS / 2);
                                *observed =
                                    Some((global - node, extra_index + 2 * (1 + lookup_steps)));
                                event.prevent_default();
                                Ok(())
                            },
                        ),
                        options,
                    )
                    .unwrap();
                // Leave one reusable slot so the two registrations have the same
                // Runtime bank shape and neither needs registry replacement storage.
                let slot = listeners
                    .add_listener(
                        &mut session.dom(),
                        dialog,
                        [u16::from(b'x')],
                        DomEventCallback::new(|_, _, _, _| Ok(())),
                        options,
                    )
                    .unwrap();
                assert!(listeners.remove_listener(slot));

                let mut source = session.source_execution();
                let mut services = DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                };
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: DomSourceUnavailable::Event,
                } = source
                    .apply(
                        &mut session.dom(),
                        DomSourceEdit::show_dialog(dialog),
                        source_limits(),
                        &mut services,
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("actual Show must expose beforetoggle");
                };
                source
                    .raise_event_runtime_limits(
                        &session.dom(),
                        DomSourceEventRuntimeLimits {
                            jobs: 1,
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
                        &cancel,
                    )
                    .unwrap();
                let mut observed = None;
                let DomSourceEventServiceProgress::Serviced { receipt } = source
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut listeners,
                        &mut observed,
                        DomSourceEventServiceLimits {
                            source: source_limits(),
                            dispatch: DomEventDispatchLimits {
                                path_items: 16,
                                snapshot_listeners: 2,
                                failure_records: 2,
                                callbacks: 2,
                                work: 100_000,
                            },
                        },
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("original beforetoggle must complete");
                };
                assert!(receipt.failures().is_empty());
                assert!(receipt.default_prevented());
                assert!(matches!(
                    source
                        .resume(
                            &mut session.dom(),
                            request.operation(),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                drop((node_seed, global_seed));
                let (actual, expected) = observed.unwrap();
                assert_eq!(
                    actual, expected,
                    "additional Source work for Global recipient"
                );
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
            fn queued_details_retains_the_reached_unavailable_clock_observation() {
                queued_element_task_retains_clock_failure(false);
            }

            #[test]
            fn queued_dialog_close_retains_the_reached_regressed_clock_observation() {
                queued_element_task_retains_clock_failure(true);
            }

            fn queued_element_task_retains_clock_failure(dialog: bool) {
                use crate::{
                    DomElementTask, DomElementTaskEventContext, DomElementTaskProgress,
                    DomElementTaskRunLimits, DomElementTaskRuntime, DomElementTaskRuntimeError,
                    DomElementTaskSourceKind, DomEventCallback, DomEventDispatchLimits,
                    DomEventDispatchOutcome, DomEventDispatcher, DomEventInit,
                    DomEventListenerOptions, DomEventRegistryLimits, DomEventResource,
                    DomEventTimestampUnavailable, DomEventType, DomSourceEventRuntimeLimits,
                    DomSourceEventServiceLimits, DomSourceEventServiceProgress,
                    DomSourceUnavailable,
                };
                use surgeist_dom::{
                    DomError, DomSourceEventClockFailure, DomSourceEventKind, Namespace,
                };

                struct Context {
                    tasks: DomElementTaskRuntime,
                    seen: Vec<(bool, Result<f64, DomEventTimestampUnavailable>)>,
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
                                8, 64, 2_000_000, 2_000_000, 100_000,
                            )
                            .unwrap(),
                        )
                });
                let mut hosted = session.hosted_execution();
                let mut observations = vec![Ok(150_001), Ok(260_001)];
                let expected_failure = if dialog {
                    observations.extend([Ok(430_001), Ok(300_001)]);
                    DomSourceEventClockFailure::Regressed
                } else {
                    observations.push(Err(Error::Unavailable));
                    DomSourceEventClockFailure::Unavailable
                };
                observations.push(Ok(845_001));
                script(&hosted, &observations);
                let host = hosted.create_host(&mut session.dom(), &cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, &cancel)
                    .unwrap();
                let owner = hosted.state().unwrap();
                let samples_left = || owner.borrow().clock.observations.as_ref().unwrap().len();

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
                    Namespace::html(),
                    if dialog { "dialog" } else { "details" },
                );
                batch.insert_before(&target, body, None);
                if !dialog {
                    batch.set_attribute(&target, "open", "");
                }
                let target = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&target)
                    .unwrap()
                    .unwrap();
                let dispatch = DomEventDispatchLimits {
                    path_items: 16,
                    snapshot_listeners: 2,
                    failure_records: 2,
                    callbacks: 2,
                    work: 100_000,
                };
                let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 2,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 2_000_000,
                    work: 100_000,
                })
                .unwrap();
                let mut context = Context {
                    tasks: session.element_task_execution(),
                    seen: Vec::new(),
                };
                let (task_source, event_type) = if dialog {
                    let mut source = session.source_execution();
                    let mut services = DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    };
                    assert!(matches!(
                        source
                            .apply(
                                &mut session.dom(),
                                DomSourceEdit::set_attribute(target, "open", ""),
                                source_limits(),
                                &mut services,
                                &cancel,
                            )
                            .unwrap(),
                        DomSourceDriveProgress::Complete(_)
                    ));
                    source
                        .raise_event_runtime_limits(
                            &session.dom(),
                            DomSourceEventRuntimeLimits {
                                jobs: 1,
                                storage_bytes: 16_000_000,
                                work: 1_000_000,
                            },
                            &cancel,
                        )
                        .unwrap();
                    let DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Event(request),
                        reason: DomSourceUnavailable::Event,
                    } = source
                        .apply(
                            &mut session.dom(),
                            DomSourceEdit::close_dialog(target, None),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap()
                    else {
                        panic!("actual Close must first expose its beforetoggle");
                    };
                    assert_eq!(request.kind(), DomSourceEventKind::BeforeToggle);
                    let DomSourceEventServiceProgress::Serviced { receipt } = source
                        .service_event(
                            &mut session.dom(),
                            &request,
                            &mut dispatcher,
                            &mut context,
                            DomSourceEventServiceLimits {
                                dispatch,
                                source: source_limits(),
                            },
                            &cancel,
                        )
                        .unwrap()
                    else {
                        panic!("actual Close beforetoggle must deliver");
                    };
                    assert_eq!(receipt.callbacks(), 0);
                    assert!(matches!(
                        source
                            .resume(
                                &mut session.dom(),
                                request.operation(),
                                source_limits(),
                                &mut services,
                                &cancel,
                            )
                            .unwrap(),
                        DomSourceDriveProgress::Complete(_)
                    ));
                    (
                        DomElementTaskSourceKind::UserInteraction,
                        DomEventType::CLOSE,
                    )
                } else {
                    assert_eq!(
                        context.tasks.ingest(&mut session.dom(), &cancel).unwrap(),
                        1
                    );
                    (
                        DomElementTaskSourceKind::DomManipulation,
                        DomEventType::TOGGLE,
                    )
                };
                let queued = context.tasks.pending_task(task_source, 0).unwrap().unwrap();
                assert_eq!(queued.id().target(), target);
                assert_eq!(queued.queue_association().unwrap().global(), top.global());
                assert!(matches!(
                    (dialog, queued),
                    (true, DomElementTask::DialogClose(_))
                        | (false, DomElementTask::DetailsToggle(_))
                ));
                assert!(session.dom().pending_source_edit().is_none());
                assert_eq!(samples_left(), 2);
                for _ in 0..2 {
                    dispatcher
                        .add_listener(
                            &mut session.dom(),
                            target,
                            event_type.clone(),
                            DomEventCallback::new(|_, _, context: &mut Context, event| {
                                context
                                    .seen
                                    .push((event.creation().is_some(), event.time_stamp()));
                                Ok(())
                            }),
                            DomEventListenerOptions::default(),
                        )
                        .unwrap();
                }

                let mut run = |session: &mut DomSession,
                               context: &mut Context,
                               dispatch,
                               cancel: &AtomicBool| {
                    DomElementTaskRuntime::run_next(
                        &mut session.dom(),
                        task_source,
                        &mut dispatcher,
                        context,
                        DomElementTaskRunLimits {
                            dispatch,
                            source: source_limits(),
                        },
                        cancel,
                    )
                };
                assert!(matches!(
                    run(&mut session, &mut context, dispatch, &AtomicBool::new(true)),
                    Err(DomElementTaskRuntimeError::Dom(DomError::Cancelled))
                ));
                assert_eq!(samples_left(), 2);
                assert_eq!(
                    context.tasks.pending_task(task_source, 0).unwrap(),
                    Some(queued)
                );

                // A single Host/top-level/cluster/Global route takes four reads and
                // first entry takes one step. Leave the sample's own admission one short.
                let (short_cap, original_cap) = {
                    let mut state = owner.borrow_mut();
                    let original = state.limits.work;
                    let short = state.work + 5;
                    state.limits.work = short;
                    (short, original)
                };
                let refused = run(&mut session, &mut context, dispatch, &cancel);
                assert!(
                    matches!(
                        refused,
                        Err(DomElementTaskRuntimeError::Hosted(
                            DomHostedRuntimeError::Admission
                        ))
                    ),
                    "the sample work gate must refuse before provider access: {refused:?}"
                );
                assert_eq!(owner.borrow().work, short_cap);
                owner.borrow_mut().limits.work = original_cap;
                assert_eq!(samples_left(), 2);
                assert!(context.seen.is_empty());
                assert_eq!(
                    context.tasks.pending_task(task_source, 0).unwrap(),
                    Some(queued)
                );

                let reached = run(
                    &mut session,
                    &mut context,
                    DomEventDispatchLimits {
                        callbacks: 1,
                        ..dispatch
                    },
                    &cancel,
                );
                assert_eq!(
                    samples_left(),
                    1,
                    "the reached provider observation is consumed"
                );
                assert!(
                    matches!(
                        reached,
                        Ok(DomElementTaskProgress::Retained {
                            task,
                            dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
                            ..
                        }) if task == queued
                    ),
                    "the reached provider failure must retain the original firing for delivery: {reached:?}"
                );
                let expected = (
                    false,
                    Err(DomEventTimestampUnavailable::Provider(expected_failure)),
                );
                assert_eq!(context.seen, [expected]);
                let DomElementTaskProgress::Complete { task, receipt, .. } =
                    run(&mut session, &mut context, dispatch, &cancel).unwrap()
                else {
                    panic!("the same queued task must deliver its original Event receipt");
                };
                assert_eq!(task, queued);
                assert_eq!(receipt.original_target(), target.into());
                assert_eq!(receipt.event_type(), event_type);
                assert_eq!(receipt.callbacks(), 2);
                assert!(receipt.failures().is_empty());
                assert_eq!(context.seen, [expected, expected]);
                assert!(session.dom().pending_source_edit().is_none());
                assert!(matches!(
                    run(&mut session, &mut context, dispatch, &cancel).unwrap(),
                    DomElementTaskProgress::Empty
                ));
                assert_eq!(samples_left(), 1);
                let next = hosted
                    .create_event(
                        &mut session.dom(),
                        top.global(),
                        &[],
                        DomEventInit::default(),
                        &cancel,
                    )
                    .unwrap();
                assert_eq!(next.creation().unwrap().occurrence_microseconds(), 800);
                assert_eq!(samples_left(), 0);
            }

            #[test]
            fn hosted_style_load_keeps_its_first_networking_firing_time_across_resume() {
                use crate::{
                    DomEventCallback, DomEventCreationObservation, DomEventDispatchLimits,
                    DomEventDispatchOutcome, DomEventDispatcher, DomEventListenerOptions,
                    DomEventPhase, DomEventRegistryLimits, DomEventResource, DomEventTarget,
                    DomEventType, DomStyleEventContext, DomStyleNetworkingProgress,
                    DomStyleRuntime,
                };
                use surgeist_dom::{DomStyleCssomRoot, DomStyleInlineCspObservation, Namespace};

                struct Context {
                    styles: DomStyleRuntime,
                    seen: Vec<(
                        Option<DomEventTarget>,
                        DomEventPhase,
                        Option<DomEventCreationObservation>,
                    )>,
                }
                impl DomStyleEventContext for Context {
                    fn styles(&mut self) -> &mut DomStyleRuntime {
                        &mut self.styles
                    }
                }

                let cancel = AtomicBool::new(false);
                let mut session = session_with_dom_limits(|limits| {
                    limits
                        .with_dispatch_custody(
                            surgeist_dom::DomDispatchCustodyLimits::new(
                                8, 64, 2_000_000, 2_000_000, 100_000,
                            )
                            .unwrap(),
                        )
                        .with_event_recipients(
                            surgeist_dom::DomEventRecipientLimits::new(
                                16, 2_000_000, 2_000_000, 100_000,
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
                        Ok(430_001),
                        Ok(545_001),
                        Ok(845_001),
                        Ok(1_125_001),
                    ],
                );
                let host = hosted.create_host(&mut session.dom(), &cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, &cancel)
                    .unwrap();
                let document = top.document();
                let settings = hosted
                    .settings(&session.dom(), top.global(), &cancel)
                    .unwrap();
                let snapshot = session.snapshot();
                let html = snapshot
                    .document(document)
                    .unwrap()
                    .document_element()
                    .unwrap();
                let body = snapshot.node(html).unwrap().children()[1];
                let mut context = Context {
                    styles: session.style_execution(),
                    seen: Vec::new(),
                };
                let mut root = Some(DomStyleCssomRoot::new(
                    document.node(),
                    surgeist_cssom::CssomStore::new(Default::default(), Default::default())
                        .unwrap(),
                ));
                context
                    .styles
                    .bind_root(&session.dom(), "hosted-clock".into(), &mut root, &cancel)
                    .unwrap();
                let mut batch = DomBatch::new();
                let style = batch.create_element_ns(document, Namespace::html(), "style");
                let text = batch.create_text(document, "p { color: red }");
                batch.insert_before(&text, &style, None);
                let target = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&style)
                    .unwrap()
                    .unwrap();
                let result = session
                    .source_execution()
                    .apply(
                        &mut session.dom(),
                        DomSourceEdit::insert_before(target, body, None),
                        source_limits(),
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |source| {
                                Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                            },
                            url: None,
                        },
                        &cancel,
                    )
                    .unwrap();
                assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
                let published = session.snapshot();
                assert_eq!(
                    published.relevant_global(target).unwrap(),
                    Some(top.global())
                );
                let association = published
                    .node(target)
                    .unwrap()
                    .as_style()
                    .unwrap()
                    .sheet()
                    .unwrap()
                    .unwrap()
                    .clone();
                let cssom = context.styles.snapshot(document.node()).unwrap().unwrap();
                let sheet = cssom.sheet(association.id()).unwrap();
                assert_eq!(association.root(), document.node());
                assert_eq!(sheet.authored().rules().len(), 1);
                assert!(sheet.diagnostics().is_empty());
                assert_eq!(context.styles.waiting_resources().unwrap(), 0);

                let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 2,
                    pending_events: 1,
                    nested_dispatches: 1,
                    storage_bytes: 2_000_000,
                    work: 100_000,
                })
                .unwrap();
                for _ in 0..2 {
                    dispatcher
                        .add_listener(
                            &mut session.dom(),
                            target,
                            DomEventType::LOAD,
                            DomEventCallback::new(|_, _, context: &mut Context, event| {
                                context.seen.push((
                                    event.current_target(),
                                    event.phase(),
                                    event.creation().copied(),
                                ));
                                Ok(())
                            }),
                            DomEventListenerOptions::default(),
                        )
                        .unwrap();
                }
                let mut run = |session: &mut DomSession, context: &mut Context, callbacks| {
                    DomStyleRuntime::run_networking(
                        &mut session.dom(),
                        &mut dispatcher,
                        context,
                        DomEventDispatchLimits {
                            path_items: 16,
                            snapshot_listeners: 2,
                            failure_records: 2,
                            callbacks,
                            work: 100_000,
                        },
                        &cancel,
                    )
                    .unwrap()
                };
                let clock_control = || {
                    hosted
                        .state()
                        .unwrap()
                        .borrow_mut()
                        .clock
                        .sample()
                        .unwrap()
                        .coarsen()
                        .unwrap()
                        .microseconds
                };
                let after_queue = clock_control();
                let DomStyleNetworkingProgress::Retained {
                    task: original,
                    dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
                } = run(&mut session, &mut context, 0)
                else {
                    panic!(
                        "the original networking task must reach dispatch before its first callback"
                    );
                };
                assert!(context.seen.is_empty());
                let after_firing = clock_control();
                let DomStyleNetworkingProgress::Retained {
                    task: resumed,
                    dispatch: DomEventDispatchOutcome::Retained(DomEventResource::Callbacks),
                } = run(&mut session, &mut context, 1)
                else {
                    panic!("the original networking task must stop after one actual callback");
                };
                assert_eq!(resumed, original);
                assert_eq!(context.seen.len(), 1);
                let after_first_callback = clock_control();
                let DomStyleNetworkingProgress::Complete { task, receipt } =
                    run(&mut session, &mut context, 2)
                else {
                    panic!("the same networking task must deliver and complete its cleanup");
                };
                assert_eq!(task, original);
                assert_eq!(receipt.original_target(), target.into());
                assert_eq!(receipt.event_type(), DomEventType::LOAD);
                assert_eq!(receipt.callbacks(), 2);
                assert!(receipt.failures().is_empty());
                assert!(matches!(
                    run(&mut session, &mut context, 2),
                    DomStyleNetworkingProgress::Empty
                ));

                let [first, second] = context.seen.as_slice() else {
                    panic!("two actual Load listeners");
                };
                for observation in [first, second] {
                    assert_eq!(observation.0, Some(target.into()));
                    assert_eq!(observation.1, DomEventPhase::AtTarget);
                }
                let [Some(first), Some(second)] = [first.2, second.2] else {
                    panic!(
                        "actual hosted Load callbacks must see original Event creation: {:?}",
                        [first.2, second.2]
                    );
                };
                assert_eq!(first, second);
                assert_eq!(first.global(), top.global());
                assert_eq!(first.clock_source(), settings.clock_source());
                assert_eq!(first.time_origin_microseconds(), 100);
                assert_eq!(first.occurrence_microseconds(), 500);
                assert_eq!(first.relative_microseconds(), 400);
                assert_eq!(
                    [after_queue, after_firing, after_first_callback],
                    [400, 800, 1_100]
                );
            }

            #[test]
            fn hosted_style_firing_charges_active_source_before_nested_dispatch_refuses() {
                use crate::{
                    DomEventCallback, DomEventCreationObservation, DomEventDispatchError,
                    DomEventDispatchLimits, DomEventDispatcher, DomEventListenerOptions,
                    DomEventRegistryLimits, DomEventResource, DomEventType,
                    DomSourceEventRuntimeLimits, DomSourceEventServiceLimits,
                    DomSourceEventServiceProgress, DomSourceUnavailable, DomStyleEventContext,
                    DomStyleNetworkingProgress, DomStyleRuntime, DomStyleRuntimeError,
                };
                use surgeist_dom::{DomStyleCssomRoot, DomStyleInlineCspObservation, Namespace};

                struct Context {
                    styles: DomStyleRuntime,
                    work: Option<(u64, u64, u64)>,
                    seen: Vec<DomEventCreationObservation>,
                }

                impl DomStyleEventContext for Context {
                    fn styles(&mut self) -> &mut DomStyleRuntime {
                        &mut self.styles
                    }
                }

                let cancel = AtomicBool::new(false);
                let (mut session, hosted, top, dialog) = hosted_dialog_for_clock(
                    &[
                        Ok(150_001),
                        Ok(260_001),
                        Ok(430_001),
                        Ok(545_001),
                        Ok(845_001),
                    ],
                    &cancel,
                );
                let recipient_limits = session.dom().limits().with_event_recipients(
                    surgeist_dom::DomEventRecipientLimits::new(16, 2_000_000, 2_000_000, 100_000)
                        .unwrap(),
                );
                session
                    .dom()
                    .raw_mut()
                    .set_limits(recipient_limits)
                    .unwrap();
                let document = top.document();
                let snapshot = session.snapshot();
                let html = snapshot
                    .document(document)
                    .unwrap()
                    .document_element()
                    .unwrap();
                let body = snapshot.node(html).unwrap().children()[1];
                let mut context = Context {
                    styles: session.style_execution(),
                    work: None,
                    seen: Vec::new(),
                };
                let mut root = Some(DomStyleCssomRoot::new(
                    document.node(),
                    surgeist_cssom::CssomStore::new(Default::default(), Default::default())
                        .unwrap(),
                ));
                context
                    .styles
                    .bind_root(&session.dom(), "hosted-clock".into(), &mut root, &cancel)
                    .unwrap();

                let mut batch = DomBatch::new();
                let style = batch.create_element_ns(document, Namespace::html(), "style");
                let text = batch.create_text(document, "p { color: red }");
                batch.insert_before(&text, &style, None);
                let target = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&style)
                    .unwrap()
                    .unwrap();
                let mut source = session.source_execution();
                let mut services = DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |source| {
                        Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                    },
                    url: None,
                };
                assert!(matches!(
                    source
                        .apply(
                            &mut session.dom(),
                            DomSourceEdit::insert_before(target, body, None),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));

                let dispatch = DomEventDispatchLimits {
                    path_items: 16,
                    snapshot_listeners: 2,
                    failure_records: 2,
                    callbacks: 2,
                    work: 100_000,
                };
                let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 2,
                    pending_events: 2,
                    nested_dispatches: 1,
                    storage_bytes: 2_000_000,
                    work: 100_000,
                })
                .unwrap();
                dispatcher
                    .add_listener(
                        &mut session.dom(),
                        dialog,
                        DomEventType::BEFORE_TOGGLE,
                        DomEventCallback::new(
                            move |dom, dispatcher, context: &mut Context, event| {
                                let cancel = AtomicBool::new(false);
                                let spent = |dom: &crate::DomEventDom<'_>| {
                                    dom.pending_source_edit().unwrap().spent_work()
                                };
                                let minimum =
                                    constructor_lookup_steps(dom, top, false, &cancel) + 2;
                                let before = spent(dom);
                                let stamp = dom.snapshot().stamp();
                                for _ in 0..2 {
                                    assert!(matches!(
                                        DomStyleRuntime::run_networking(
                                            dom, dispatcher, context, dispatch, &cancel,
                                        ),
                                        Err(DomStyleRuntimeError::Dispatch(
                                            DomEventDispatchError::Admission(
                                                DomEventResource::NestedDispatches
                                            )
                                        ))
                                    ));
                                    assert_eq!(dom.snapshot().stamp(), stamp);
                                    assert_eq!(event.current_target(), Some(dialog.into()));
                                    if let Some((_, first, _)) = context.work {
                                        context.work =
                                            Some((minimum, first, spent(dom) - before - first));
                                    } else {
                                        context.work = Some((minimum, spent(dom) - before, 0));
                                    }
                                }
                                event.prevent_default();
                                Ok(())
                            },
                        ),
                        DomEventListenerOptions::default(),
                    )
                    .unwrap();
                dispatcher
                    .add_listener(
                        &mut session.dom(),
                        target,
                        DomEventType::LOAD,
                        DomEventCallback::new(|_, _, context: &mut Context, event| {
                            context
                                .seen
                                .push(*event.creation().expect("original hosted firing sample"));
                            Ok(())
                        }),
                        DomEventListenerOptions::default(),
                    )
                    .unwrap();

                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: DomSourceUnavailable::Event,
                } = source
                    .apply(
                        &mut session.dom(),
                        DomSourceEdit::show_dialog(dialog),
                        source_limits(),
                        &mut services,
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("real Show must reach its beforetoggle request");
                };
                source
                    .raise_event_runtime_limits(
                        &session.dom(),
                        DomSourceEventRuntimeLimits {
                            jobs: 1,
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
                        &cancel,
                    )
                    .unwrap();
                let DomSourceEventServiceProgress::Serviced { receipt } = source
                    .service_event(
                        &mut session.dom(),
                        &request,
                        &mut dispatcher,
                        &mut context,
                        DomSourceEventServiceLimits {
                            source: source_limits(),
                            dispatch,
                        },
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("the original beforetoggle must complete");
                };
                assert!(receipt.failures().is_empty());
                assert!(receipt.default_prevented());
                assert!(context.seen.is_empty());
                assert!(matches!(
                    source
                        .resume(
                            &mut session.dom(),
                            request.operation(),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                let DomStyleNetworkingProgress::Complete { receipt, .. } =
                    DomStyleRuntime::run_networking(
                        &mut session.dom(),
                        &mut dispatcher,
                        &mut context,
                        dispatch,
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("retry must finish the original Style Load");
                };
                assert_eq!(receipt.original_target(), target.into());
                assert_eq!(receipt.callbacks(), 1);
                assert!(receipt.failures().is_empty());
                let [creation] = context.seen.as_slice() else {
                    panic!("one actual Style Load callback");
                };
                assert_eq!(creation.global(), top.global());
                assert_eq!(creation.time_origin_microseconds(), 100);
                assert_eq!(creation.occurrence_microseconds(), 500);
                let owner = hosted.state().unwrap();
                assert_eq!(owner.borrow().clock.observations.as_ref().unwrap().len(), 1);
                assert!(matches!(
                    DomStyleRuntime::run_networking(
                        &mut session.dom(),
                        &mut dispatcher,
                        &mut context,
                        dispatch,
                        &cancel,
                    )
                    .unwrap(),
                    DomStyleNetworkingProgress::Empty
                ));

                // The subject provenance read and clock sample each add one unit to
                // the separately observed canonical Global/Document association work.
                let (minimum, first, retry) = context.work.unwrap();
                assert_eq!(
                    retry, 0,
                    "the retained body has already paid for its sample"
                );
                assert!(
                    first >= minimum,
                    "active Source charged {first}, expected at least {minimum} for original origin and clock"
                );
            }

            #[test]
            fn synchronous_dialog_events_keep_their_distinct_reached_clock_samples() {
                use crate::{
                    DomEventCallback, DomEventCreationObservation, DomEventDispatchLimits,
                    DomEventDispatchOutcome, DomEventDispatcher, DomEventListenerOptions,
                    DomEventPhase, DomEventRegistryLimits, DomEventResource, DomEventTarget,
                    DomEventType, DomSourceEventRuntimeLimits, DomSourceEventServiceLimits,
                    DomSourceEventServiceProgress, DomSourceUnavailable,
                };
                use surgeist_dom::{DomError, DomSourceEventKind, DomSourceEventRequest};

                struct Observation {
                    kind: DomEventType,
                    phase: DomEventPhase,
                    current: Option<DomEventTarget>,
                    operation: Option<DomSourceOperationId>,
                    creation: Option<DomEventCreationObservation>,
                    edit: Result<(), DomError>,
                }
                let cancel = AtomicBool::new(false);
                let mut session = session_with_dom_limits(|limits| {
                    limits.with_dispatch_custody(
                        surgeist_dom::DomDispatchCustodyLimits::new(
                            16, 128, 2_000_000, 2_000_000, 100_000,
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
                        Ok(545_001),
                        Ok(845_001),
                        Ok(1_125_001),
                        Ok(1_605_001),
                        Ok(2_075_001),
                        Ok(2_605_001),
                    ],
                );
                let host = hosted.create_host(&mut session.dom(), &cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, &cancel)
                    .unwrap();
                let settings = hosted
                    .settings(&session.dom(), top.global(), &cancel)
                    .unwrap();
                let native_clock_control = || {
                    hosted
                        .state()
                        .unwrap()
                        .borrow_mut()
                        .clock
                        .sample()
                        .unwrap()
                        .coarsen()
                        .unwrap()
                        .microseconds
                };
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
                    "dialog",
                );
                batch.insert_before(&target, body, None);
                let target = session
                    .dom()
                    .apply(batch, &cancel)
                    .unwrap()
                    .created(&target)
                    .unwrap()
                    .unwrap();
                let mut source = session.source_execution();
                let mut services = DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                };
                assert!(matches!(
                    source
                        .apply(
                            &mut session.dom(),
                            DomSourceEdit::set_attribute(target, "open", ""),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceDriveProgress::Complete(_)
                ));
                let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
                    registrations: 4,
                    pending_events: 2,
                    nested_dispatches: 2,
                    storage_bytes: 2_000_000,
                    work: 100_000,
                })
                .unwrap();
                for kind in [DomEventType::CANCEL, DomEventType::BEFORE_TOGGLE] {
                    for _ in 0..2 {
                        registry
                            .add_listener(
                                &mut session.dom(),
                                target,
                                kind.clone(),
                                DomEventCallback::new(
                                    move |dom, _, seen: &mut Vec<Observation>, event| {
                                        let operation = dom
                                            .pending_source_edit()
                                            .map(|status| status.operation());
                                        let mut edit = DomBatch::new();
                                        edit.set_attribute(body, "data-clock-probe", "");
                                        seen.push(Observation {
                                            kind: event.event_type(),
                                            phase: event.phase(),
                                            current: event.current_target(),
                                            operation,
                                            creation: event.creation().copied(),
                                            edit: dom
                                                .apply(edit, &AtomicBool::new(false))
                                                .map(drop),
                                        });
                                        Ok(())
                                    },
                                ),
                                DomEventListenerOptions::default(),
                            )
                            .unwrap();
                    }
                }
                let pending = |progress, kind| {
                    let DomSourceDriveProgress::Pending {
                        request: DomSourceRequest::Event(request),
                        reason: DomSourceUnavailable::Event,
                    } = progress
                    else {
                        panic!("expected an actual synchronous Dialog event request");
                    };
                    assert_eq!(request.kind(), kind);
                    assert_eq!(request.subject(), target);
                    request
                };
                // The original reached occurrence precedes Event Job admission.
                // This fixture starts with zero Jobs; raise only after exposure.
                let cancel_request = pending(
                    source
                        .apply(
                            &mut session.dom(),
                            DomSourceEdit::request_close_dialog(target, None),
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceEventKind::Cancel,
                );
                let operation = cancel_request.operation();
                let after_cancel_exposure = native_clock_control();
                let repeated = pending(
                    source
                        .resume(
                            &mut session.dom(),
                            operation,
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceEventKind::Cancel,
                );
                assert_eq!(repeated.identity(), cancel_request.identity());
                let after_repeated_exposure = native_clock_control();
                source
                    .raise_event_runtime_limits(
                        &session.dom(),
                        DomSourceEventRuntimeLimits {
                            jobs: 2,
                            storage_bytes: 16_000_000,
                            work: 1_000_000,
                        },
                        &cancel,
                    )
                    .unwrap();

                let mut seen = Vec::new();
                let mut deliver = |source: &crate::DomSourceRuntime,
                                   session: &mut DomSession,
                                   request: &DomSourceEventRequest,
                                   kind| {
                    let mut service = |callbacks| {
                        source
                            .service_event(
                                &mut session.dom(),
                                request,
                                &mut registry,
                                &mut seen,
                                DomSourceEventServiceLimits {
                                    source: source_limits(),
                                    dispatch: DomEventDispatchLimits {
                                        path_items: 16,
                                        snapshot_listeners: 4,
                                        failure_records: 4,
                                        callbacks,
                                        work: 100_000,
                                    },
                                },
                                &cancel,
                            )
                            .unwrap()
                    };
                    assert!(matches!(
                        service(1),
                        DomSourceEventServiceProgress::Retained {
                            dispatch: DomEventDispatchOutcome::Retained(
                                DomEventResource::Callbacks
                            ),
                        }
                    ));
                    let DomSourceEventServiceProgress::Serviced { receipt } = service(2) else {
                        panic!("the same original synchronous Event must finish dispatch");
                    };
                    assert_eq!(receipt.event_type(), kind);
                    assert_eq!(receipt.original_target(), target.into());
                    assert_eq!(receipt.callbacks(), 2);
                    assert!(receipt.failures().is_empty());
                    assert!(!receipt.default_prevented());
                };
                deliver(&source, &mut session, &cancel_request, DomEventType::CANCEL);
                let before_request = pending(
                    source
                        .resume(
                            &mut session.dom(),
                            operation,
                            source_limits(),
                            &mut services,
                            &cancel,
                        )
                        .unwrap(),
                    DomSourceEventKind::BeforeToggle,
                );
                assert_eq!(before_request.operation(), operation);
                assert_ne!(before_request.identity(), cancel_request.identity());
                let after_beforetoggle_exposure = native_clock_control();
                deliver(
                    &source,
                    &mut session,
                    &before_request,
                    DomEventType::BEFORE_TOGGLE,
                );
                assert!(matches!(
                    source.resume(&mut session.dom(), operation, source_limits(), &mut services, &cancel)
                        .unwrap(),
                    DomSourceDriveProgress::Complete(done) if done.operation() == operation
                ));
                let after_close = native_clock_control();
                assert!(session.dom().pending_source_edit().is_none());
                assert!(
                    session
                        .snapshot()
                        .node(target)
                        .unwrap()
                        .attribute(&"open".into())
                        .is_none()
                );

                let [first_cancel, resumed_cancel, first_before, resumed_before] = seen.as_slice()
                else {
                    panic!("expected two actual callbacks for each original Event");
                };
                for (observations, kind) in [
                    ([first_cancel, resumed_cancel], DomEventType::CANCEL),
                    ([first_before, resumed_before], DomEventType::BEFORE_TOGGLE),
                ] {
                    for observation in observations {
                        assert_eq!(observation.kind, kind);
                        assert_eq!(observation.phase, DomEventPhase::AtTarget);
                        assert_eq!(observation.current, Some(target.into()));
                        assert_eq!(observation.operation, Some(operation));
                        assert_eq!(observation.edit, Ok(()));
                    }
                }
                let creations = [first_cancel, resumed_cancel, first_before, resumed_before]
                    .map(|observed| observed.creation);
                let [
                    Some(first_cancel),
                    Some(resumed_cancel),
                    Some(first_before),
                    Some(resumed_before),
                ] = creations
                else {
                    panic!(
                        "both original synchronous Dialog occurrences need creation timing: {creations:?}"
                    );
                };
                assert_eq!(first_cancel, resumed_cancel);
                assert_eq!(first_before, resumed_before);
                for (creation, occurrence) in [(first_cancel, 500), (first_before, 1600)] {
                    assert_eq!(creation.global(), top.global());
                    assert_eq!(creation.clock_source(), settings.clock_source());
                    assert_eq!(creation.time_origin_microseconds(), 100);
                    assert_eq!(creation.occurrence_microseconds(), occurrence);
                }
                assert_eq!(
                    [
                        after_cancel_exposure,
                        after_repeated_exposure,
                        after_beforetoggle_exposure,
                        after_close
                    ],
                    [800, 1100, 2000, 2600]
                );
            }

            fn hosted_dialog_for_clock(
                observations: &[Result<u64, Error>],
                cancel: &AtomicBool,
            ) -> (
                DomSession,
                HostedRuntime,
                crate::DomHostedTopLevel,
                surgeist_dom::NodeId,
            ) {
                let mut session = session_with_dom_limits(|limits| {
                    limits.with_dispatch_custody(
                        surgeist_dom::DomDispatchCustodyLimits::new(
                            8, 64, 2_000_000, 2_000_000, 100_000,
                        )
                        .unwrap(),
                    )
                });
                let mut hosted = session.hosted_execution();
                script(&hosted, observations);
                let host = hosted.create_host(&mut session.dom(), cancel).unwrap();
                let top = hosted
                    .create_top_level(&mut session.dom(), host, cancel)
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
                    surgeist_dom::Namespace::html(),
                    "dialog",
                );
                batch.insert_before(&target, body, None);
                let target = session
                    .dom()
                    .apply(batch, cancel)
                    .unwrap()
                    .created(&target)
                    .unwrap()
                    .unwrap();
                (session, hosted, top, target)
            }

            #[test]
            fn canceled_clock_preflight_keeps_the_private_dialog_firing_for_resume() {
                use surgeist_dom::{DomSourceAdvance, DomSourcePhase};

                let cancel = AtomicBool::new(false);
                let (mut session, hosted, top, target) =
                    hosted_dialog_for_clock(&[Ok(150_001), Ok(260_001), Ok(430_001)], &cancel);
                let before_firing = session.snapshot().stamp();
                let operation = {
                    let mut dom = session.dom();
                    let mut lease = dom
                        .raw_mut()
                        .begin_source_edit(
                            DomSourceEdit::show_dialog(target),
                            source_limits(),
                            &cancel,
                        )
                        .unwrap();
                    let DomSourceAdvance::EventFiring(firing) =
                        lease.advance_for_runtime(&cancel).unwrap()
                    else {
                        panic!("the exact Show firing remains private until clock acknowledgment");
                    };
                    let prepared = lease.status();
                    assert_eq!(prepared.phase(), DomSourcePhase::EventFiring);
                    assert!(!prepared.has_published());
                    assert!(matches!(
                        lease.advance(&cancel),
                        Err(surgeist_dom::DomError::HostedSourceUnavailable),
                    ));
                    assert!(matches!(
                        hosted.finish_source_event_firing(
                            &mut lease,
                            firing,
                            &AtomicBool::new(true)
                        ),
                        Err(DomHostedRuntimeError::Cancelled),
                    ));
                    assert_eq!(lease.status().phase(), DomSourcePhase::EventFiring);
                    assert_eq!(lease.snapshot().stamp(), before_firing);
                    prepared.operation()
                };
                let mut source = session.source_execution();
                let mut services = DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                };
                let DomSourceDriveProgress::Pending {
                    request: DomSourceRequest::Event(request),
                    reason: crate::DomSourceUnavailable::Event,
                } = source
                    .resume(
                        &mut session.dom(),
                        operation,
                        source_limits(),
                        &mut services,
                        &cancel,
                    )
                    .unwrap()
                else {
                    panic!("the same private firing must complete through the original runtime");
                };
                assert_eq!(request.operation(), operation);
                assert_eq!(request.subject(), target);
                let mut dom = session.dom();
                let observed = dom
                    .raw()
                    .source_event_time(&request)
                    .unwrap()
                    .unwrap()
                    .unwrap();
                assert_eq!(observed.global(), top.global());
                assert_eq!(observed.occurrence().microseconds(), 400);
                let mut lease = dom
                    .raw_mut()
                    .resume_source_edit(operation, source_limits(), &cancel)
                    .unwrap();
                lease.request_cancel();
                assert!(
                    matches!(lease.advance(&cancel).unwrap(), DomSourceProgress::Complete(done)
                    if done.operation() == operation && done.cancellation_requested())
                );
            }

            #[test]
            fn dialog_clock_failure_survives_inline_and_meta_service_and_watcher_settlement() {
                use crate::{
                    DomEventCallback, DomEventDispatchLimits, DomEventDispatcher,
                    DomEventListenerOptions, DomEventRegistryLimits, DomEventTimestampUnavailable,
                    DomEventType, DomSourceEventRuntimeLimits, DomSourceEventServiceLimits,
                    DomSourceEventServiceProgress, DomSourceInlineStyleEdit, DomSourceUnavailable,
                };
                use surgeist_dom::{
                    DomInlineStyleRequestLimits, DomInlineStyleSourceContext, DomMetaAtomicLimits,
                    DomSourceEventClockFailure, DomSourceEventKind,
                };

                let meta = DomMetaAtomicLimits::new(16_000_000, 1_000_000, 1_000_000, 128).unwrap();
                for (meta, reading, failure) in [
                    (
                        None,
                        Err(Error::Unavailable),
                        DomSourceEventClockFailure::Unavailable,
                    ),
                    (
                        Some(meta),
                        Ok(100_001),
                        DomSourceEventClockFailure::Regressed,
                    ),
                ] {
                    let cancel = AtomicBool::new(false);
                    let (mut session, _, _, target) = hosted_dialog_for_clock(
                        &[Ok(150_001), Ok(260_001), reading, Ok(560_001)],
                        &cancel,
                    );
                    session
                        .dom()
                        .raw_mut()
                        .configure_inline_styles(
                            surgeist_dom::DomInlineStyleLimits::new(
                                surgeist_cssom::CssomLimits::default(),
                                8,
                                64_000_000,
                                8_000_000,
                                16_000_000,
                            )
                            .unwrap(),
                            &cancel,
                        )
                        .unwrap();
                    let mut source = session.source_execution();
                    let mut services = DomSourceServices {
                        meta,
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    };
                    assert!(matches!(
                        source
                            .apply(
                                &mut session.dom(),
                                DomSourceEdit::set_attribute(target, "open", ""),
                                source_limits(),
                                &mut services,
                                &cancel,
                            )
                            .unwrap(),
                        DomSourceDriveProgress::Complete(_)
                    ));
                    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
                        registrations: 2,
                        pending_events: 1,
                        nested_dispatches: 1,
                        storage_bytes: 2_000_000,
                        work: 100_000,
                    })
                    .unwrap();
                    for event_type in [DomEventType::CANCEL, DomEventType::BEFORE_TOGGLE] {
                        dispatcher
                            .add_listener(
                                &mut session.dom(),
                                target,
                                event_type,
                                DomEventCallback::new(|_, _, seen: &mut Vec<_>, event| {
                                    seen.push((event.event_type(), event.time_stamp()));
                                    Ok(())
                                }),
                                DomEventListenerOptions::default(),
                            )
                            .unwrap();
                    }
                    source
                        .raise_event_runtime_limits(
                            &session.dom(),
                            DomSourceEventRuntimeLimits {
                                jobs: 1,
                                storage_bytes: 16_000_000,
                                work: 1_000_000,
                            },
                            &cancel,
                        )
                        .unwrap();

                    let context =
                        DomInlineStyleSourceContext::new(session.snapshot().stamp(), Vec::new())
                            .unwrap();
                    let resolver =
                        |_: &[_], _: &_| panic!("Dialog firing has no inline declarations");
                    let mut progress = source
                        .apply_with_inline_styles(
                            &mut session.dom(),
                            DomSourceInlineStyleEdit::new(
                                DomSourceEdit::request_close_dialog(target, None),
                                context,
                                DomInlineStyleRequestLimits {
                                    input: surgeist_css::source::CssUtf16InputLimits::default(),
                                    declaration:
                                        surgeist_cssom::CssomDeclarationRequestLimits::default(),
                                },
                            ),
                            source_limits(),
                            &mut services,
                            resolver,
                            &cancel,
                        )
                        .unwrap();
                    let mut seen = Vec::new();
                    for kind in [DomSourceEventKind::Cancel, DomSourceEventKind::BeforeToggle] {
                        let DomSourceDriveProgress::Pending {
                            request: DomSourceRequest::Event(request),
                            reason: DomSourceUnavailable::Event,
                        } = progress
                        else {
                            panic!("the reached Dialog firing must expose its real request");
                        };
                        assert_eq!(request.kind(), kind);
                        assert_eq!(request.subject(), target);
                        let delivered = source
                            .service_event(
                                &mut session.dom(),
                                &request,
                                &mut dispatcher,
                                &mut seen,
                                DomSourceEventServiceLimits {
                                    source: source_limits(),
                                    dispatch: DomEventDispatchLimits {
                                        path_items: 16,
                                        snapshot_listeners: 2,
                                        failure_records: 2,
                                        callbacks: 2,
                                        work: 100_000,
                                    },
                                },
                                &cancel,
                            )
                            .unwrap();
                        assert!(
                            matches!(delivered, DomSourceEventServiceProgress::Serviced { ref receipt }
                            if receipt.callbacks() == 1 && !receipt.default_prevented())
                        );
                        progress = source
                            .resume_with_inline_styles(
                                &mut session.dom(),
                                request.operation(),
                                source_limits(),
                                &mut services,
                                resolver,
                                &cancel,
                            )
                            .unwrap();
                    }
                    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
                    assert!(session.dom().pending_source_edit().is_none());
                    assert!(
                        session
                            .snapshot()
                            .node(target)
                            .unwrap()
                            .attribute(&"open".into())
                            .is_none()
                    );
                    assert_eq!(
                        seen,
                        [
                            (
                                DomEventType::CANCEL,
                                Err(DomEventTimestampUnavailable::Provider(failure))
                            ),
                            (DomEventType::BEFORE_TOGGLE, Ok(0.4)),
                        ]
                    );
                }
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

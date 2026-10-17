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
    epoch: Option<Instant>,
    latest: Duration,
}
impl Clock {
    pub fn new(source: DomHostedSourceId) -> Self {
        Self {
            source,
            epoch: None,
            latest: Duration::ZERO,
        }
    }

    /// Samples the actual clock; the source identity fixes its epoch/domain.
    pub fn sample(&mut self) -> Result<Sample, Error> {
        let now = native_now()?;
        // The same real observation both establishes the domain's epoch and
        // supplies its first sample. A zero elapsed value is therefore actual.
        let epoch = *self.epoch.get_or_insert(now);
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
            DomDetailsRuntimeLimits, DomHostedRuntimeLimits, DomSession, DomSessionLimits,
            DomStyleRuntimeLimits,
        };
        use std::sync::atomic::AtomicBool;
        use surgeist_dom::{Dom, DomHostedLimits, DomHostedRecordLimits, DomLimits};

        let dom = Dom::new(DomLimits::new(512, 200_000, 1024, 1024, 1024, 64, 4_000_000).unwrap())
            .unwrap();
        let mut session = DomSession::new(
            dom,
            DomSessionLimits {
                style: DomStyleRuntimeLimits {
                    roots: 4,
                    completions: 32,
                    reports: 32,
                    storage_bytes: 16_000_000,
                    work: 1_000_000,
                    css_input: surgeist_css::CssUtf16InputLimits::default(),
                },
                details: DomDetailsRuntimeLimits {
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
}

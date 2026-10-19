//! One finite registry-only capture, using the original capture owner.
use super::*;
use std::mem::size_of;

/// A genuine complete-empty Global registry and its owning immutable evidence.
/// The local registry identity is distinct from a consumer's canonical identity.
#[derive(Debug)]
pub struct HtmlEmptyGlobalRegistryCapture {
    capture: HtmlCapture,
    registry: HtmlCaptureRegistryId,
    retained: HtmlCaptureRetainedStorageUsage,
    peak: usize,
}

impl HtmlEmptyGlobalRegistryCapture {
    pub fn capture(&self) -> &HtmlCapture {
        &self.capture
    }
    pub fn registry(&self) -> &HtmlCaptureRegistryId {
        &self.registry
    }
    /// Actual final capture payload; excludes this wrapper and its ID handle.
    pub fn retained_storage_usage(&self) -> HtmlCaptureRetainedStorageUsage {
        self.retained
    }
    /// Largest checked formation extent, including the fixed workspace and
    /// this returned wrapper, plus simultaneously live requested/actual backing.
    pub fn formation_peak_bytes(&self) -> usize {
        self.peak
    }
    /// Moves the actual evidence and local identity without cloning their graph.
    pub fn into_parts(self) -> (HtmlCapture, HtmlCaptureRegistryId) {
        (self.capture, self.registry)
    }
}

/// First refusal of the fixed empty-Global formation operation.
#[derive(Debug, Eq, PartialEq)]
pub enum HtmlEmptyGlobalRegistryError<E> {
    FormationBytes {
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    /// A fallible backing reservation reported an allocation failure. Infallible
    /// standard-library Arc allocation retains its ordinary allocation contract.
    AllocationFailed,
    Capture(HtmlCaptureError),
    Interrupted(E),
}

impl<E: fmt::Display> fmt::Display for HtmlEmptyGlobalRegistryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FormationBytes { limit, required } => write!(
                f,
                "empty Global registry requires {required} formation bytes, limit {limit}"
            ),
            Self::RepresentationExhausted => {
                f.write_str("empty Global registry representation exhausted")
            }
            Self::AllocationFailed => {
                f.write_str("empty Global registry backing allocation failed")
            }
            Self::Capture(error) => error.fmt(f),
            Self::Interrupted(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlEmptyGlobalRegistryError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Capture(error) => Some(error),
            Self::Interrupted(error) => Some(error),
            _ => None,
        }
    }
}

// Real inline owners, conservatively live for the complete synchronous call.
// The builder includes CaptureData/identity/Vec headers; scratch includes all
// four empty finish banks. The optional handle is the original builder handle.
struct Workspace {
    builder: Option<HtmlCaptureBuilder>,
    scratch: preflight::DepthScratch,
    registry: Option<HtmlCaptureBuilderRegistry>,
}

struct Control {
    limit: usize,
    fixed: usize,
    backing: usize,
    peak: usize,
}
impl Control {
    fn admit<E>(&mut self, required: usize) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        self.peak = self.peak.max(required);
        if required > self.limit {
            return Err(HtmlEmptyGlobalRegistryError::FormationBytes {
                limit: self.limit,
                required,
            });
        }
        Ok(())
    }
    fn plus<E>(a: usize, b: usize) -> Result<usize, HtmlEmptyGlobalRegistryError<E>> {
        a.checked_add(b)
            .ok_or(HtmlEmptyGlobalRegistryError::RepresentationExhausted)
    }
    fn step<E>(
        step: &mut impl FnMut(usize) -> Result<(), E>,
        units: usize,
    ) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        step(units).map_err(HtmlEmptyGlobalRegistryError::Interrupted)
    }
    fn action<E>(
        &mut self,
        action: preflight::Action,
        step: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), HtmlEmptyGlobalRegistryError<E>> {
        match action {
            preflight::Action::Work(units) => Self::step(step, units),
            preflight::Action::Reserve { .. } => Self::step(step, 0),
            preflight::Action::StorageBefore { requested } => {
                Self::step(step, 0)?;
                self.admit(Self::plus(
                    Self::plus(self.fixed, self.backing)?,
                    requested,
                )?)
            }
            preflight::Action::StorageAfter { old, actual, .. } => {
                // The request was admitted before allocation. Record/check any
                // extra exposed capacity before proceeding or publishing.
                let replacement_peak = Self::plus(Self::plus(self.fixed, self.backing)?, actual)?;
                self.backing = self
                    .backing
                    .checked_sub(old)
                    .and_then(|n| n.checked_add(actual))
                    .ok_or(HtmlEmptyGlobalRegistryError::RepresentationExhausted)?;
                self.admit(replacement_peak)
            }
            preflight::Action::StorageTransfer { .. } | preflight::Action::ElementValues { .. } => {
                Ok(())
            }
        }
    }
}

fn preflight_error<E>(
    error: preflight::Failure<HtmlEmptyGlobalRegistryError<E>>,
) -> HtmlEmptyGlobalRegistryError<E> {
    match error {
        preflight::Failure::Admission(error) => error,
        preflight::Failure::Capture(error) => HtmlEmptyGlobalRegistryError::Capture(error),
        preflight::Failure::AllocationFailed => HtmlEmptyGlobalRegistryError::AllocationFailed,
    }
}

impl HtmlCapture {
    /// Forms exactly one complete-empty Global registry, with no captured nodes.
    ///
    /// `formation_bytes` is the caller's current total allowance for this entire
    /// synchronous producer call. `step` admits actual work and polls cancellation
    /// before source observations; it must not allocate other retained graphs or
    /// change that allowance. Used work is not refunded after a refusal.
    ///
    /// Before the first heap allocation, the allowance covers the real builder,
    /// finish scratch and builder handle workspace, plus the returned wrapper.
    /// The original registry call's empty definition Vec argument and empty
    /// validation HashSet header are also admitted while live. Scalar numerical
    /// control bookkeeping is excluded, as in the controlled OptionValue owner.
    /// Registry backing is admitted before its requested-capacity reservation;
    /// its actual exposed capacity is checked immediately afterward. The final
    /// CaptureData header is admitted before Arc publication, while the workspace
    /// remains priced. Public result/ID headers are included during formation;
    /// consumers price their own retained inline occurrences after transfer.
    ///
    /// The receipt uses the original capture storage observer and excludes opaque
    /// Arc control blocks/allocator bookkeeping, including the zero-payload capture
    /// identity. Requested capacity admission is not a strict allocator/RSS bound:
    /// safe Vec reservation may expose extra capacity, which is checked before
    /// proceeding. Every refusal drops all partial product owners. All fallible
    /// validation, admission and receipt observations precede publication.
    pub fn complete_empty_global_registry_with<E>(
        host_revision: HtmlHostFactsRevision,
        formation_bytes: usize,
        mut step: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<HtmlEmptyGlobalRegistryCapture, HtmlEmptyGlobalRegistryError<E>> {
        Control::step(&mut step, 1)?;
        let fixed = Control::plus(
            size_of::<Workspace>(),
            size_of::<HtmlEmptyGlobalRegistryCapture>(),
        )?;
        let mut control = Control {
            limit: formation_bytes,
            fixed,
            backing: 0,
            peak: 0,
        };
        control.admit(fixed)?;
        let mut workspace = Workspace {
            builder: Some(HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(0, 0, 0, 0, 0, 1, 0, 16 * 1024 * 1024),
                host_revision,
            )),
            scratch: preflight::DepthScratch::new(),
            registry: None,
        };
        let builder = workspace.builder.as_mut().expect("actual builder");
        builder.enable_controlled_storage();
        // Pre-reserve the exact existing registry bank. The original add_registry
        // call then retains its complete validation path without another growth.
        preflight::reserve_storage(&mut builder.data.registries, 1, true, true, &mut |action| {
            control.action(action, &mut step)
        })
        .map_err(preflight_error)?;
        Control::step(&mut step, 1)?;
        let registry_call_headers = Control::plus(
            size_of::<Option<Vec<HtmlCustomElementDefinition>>>(),
            size_of::<HashSet<&DocumentString>>(),
        )?;
        control.admit(Control::plus(
            Control::plus(fixed, control.backing)?,
            registry_call_headers,
        )?)?;
        workspace.registry = Some(
            builder
                .add_registry(Some(Vec::new()))
                .map_err(HtmlEmptyGlobalRegistryError::Capture)?,
        );
        Control::step(&mut step, 1)?;
        builder
            .establish_registry_scope(
                workspace.registry.as_ref().expect("actual registry"),
                HtmlRegistryScope::Global,
            )
            .map_err(HtmlEmptyGlobalRegistryError::Capture)?;
        builder
            .preflight_finish(&mut workspace.scratch, &mut |action| {
                control.action(action, &mut step)
            })
            .map_err(preflight_error)?;
        let retained =
            storage::retained_storage_usage_with(&builder.data, || Control::step(&mut step, 1))
                .map_err(|error| match error {
                    HtmlCaptureStorageError::Interrupted(error) => error,
                    HtmlCaptureStorageError::RepresentationExhausted => {
                        HtmlEmptyGlobalRegistryError::RepresentationExhausted
                    }
                })?;
        Control::step(&mut step, 1)?;
        let required = Control::plus(
            Control::plus(fixed, control.backing)?,
            size_of::<CaptureData>(),
        )?;
        control.admit(required)?;
        let registry = HtmlCaptureRegistryId {
            capture: builder.data.identity.clone(),
            index: workspace.registry.as_ref().expect("actual registry").index,
        };
        let capture = workspace.builder.take().expect("actual builder").publish();
        Ok(HtmlEmptyGlobalRegistryCapture {
            capture,
            registry,
            retained,
            peak: control.peak,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_backing_is_admitted_before_growth_and_extra_capacity_is_checked() {
        let mut control = Control {
            limit: 119,
            fixed: 100,
            backing: 0,
            peak: 0,
        };
        assert_eq!(
            control.action(
                preflight::Action::StorageBefore { requested: 20 },
                &mut |_| Ok::<_, ()>(())
            ),
            Err(HtmlEmptyGlobalRegistryError::FormationBytes {
                limit: 119,
                required: 120
            })
        );
        assert_eq!(control.backing, 0);
        control.limit = 125;
        assert_eq!(
            control.action(
                preflight::Action::StorageBefore { requested: 20 },
                &mut |_| Ok::<_, ()>(())
            ),
            Ok(())
        );
        assert_eq!(
            control.action(
                preflight::Action::StorageAfter {
                    old: 0,
                    actual: 30,
                    retained: true,
                },
                &mut |_| Ok::<_, ()>(())
            ),
            Err(HtmlEmptyGlobalRegistryError::FormationBytes {
                limit: 125,
                required: 130
            })
        );
        assert_eq!(control.backing, 30);
        assert_eq!(control.peak, 130);
        assert_eq!(
            Control::plus::<()>(usize::MAX, 1),
            Err(HtmlEmptyGlobalRegistryError::RepresentationExhausted)
        );
    }

    #[test]
    fn every_interruption_drops_all_partial_capture_owners() {
        let revision = HtmlHostFactsRevision::fresh();
        let mut calls = 0;
        HtmlCapture::complete_empty_global_registry_with(revision.clone(), usize::MAX, |_| {
            calls += 1;
            Ok::<_, ()>(())
        })
        .unwrap();
        for stop in 0..calls {
            let mut seen = 0;
            let result = HtmlCapture::complete_empty_global_registry_with(
                revision.clone(),
                usize::MAX,
                |units| {
                    let current = seen;
                    seen += 1;
                    if current == stop {
                        Err((current, units))
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(
                matches!(result, Err(HtmlEmptyGlobalRegistryError::Interrupted((at, _)))
                if at == stop)
            );
            assert_eq!(seen, stop + 1);
            // A retained partial CaptureData would still own this revision.
            assert_eq!(Arc::strong_count(&revision.0), 1);
        }
        let result = HtmlCapture::complete_empty_global_registry_with(revision.clone(), 0, |_| {
            Ok::<_, ()>(())
        });
        assert!(matches!(
            result,
            Err(HtmlEmptyGlobalRegistryError::FormationBytes { .. })
        ));
        assert_eq!(Arc::strong_count(&revision.0), 1);
    }

    #[test]
    fn receipt_counts_each_real_header_once_and_only_actual_registry_backing() {
        let formed = HtmlCapture::complete_empty_global_registry_with(
            HtmlHostFactsRevision::fresh(),
            usize::MAX,
            |_| Ok::<_, ()>(()),
        )
        .unwrap();
        let data = &formed.capture.0;
        let actual_bank = data.registries.capacity() * size_of::<RegistryRecord>();
        assert!(data.registries.capacity() >= 1);
        assert_eq!(data.records.capacity(), 0);
        assert_eq!(data.registries[0].definitions.capacity(), 0);
        let retained = size_of::<CaptureData>() + actual_bank;
        assert_eq!(formed.retained.retained_bytes(), retained);
        assert_eq!(formed.retained.utf16_extent(), 0);
        // Both inline owners remain conservatively priced through publication.
        let registry_call_headers = size_of::<Option<Vec<HtmlCustomElementDefinition>>>()
            + size_of::<HashSet<&DocumentString>>();
        assert_eq!(
            formed.peak,
            size_of::<Workspace>()
                + size_of::<HtmlEmptyGlobalRegistryCapture>()
                + actual_bank
                + size_of::<CaptureData>().max(registry_call_headers)
        );
    }

    #[test]
    fn existing_safe_allocation_capture_and_admission_causes_are_distinct() {
        assert_eq!(
            preflight_error::<()>(preflight::Failure::AllocationFailed),
            HtmlEmptyGlobalRegistryError::AllocationFailed
        );
        assert_eq!(
            preflight_error::<()>(preflight::Failure::Capture(
                HtmlCaptureError::ForeignBuilder
            )),
            HtmlEmptyGlobalRegistryError::Capture(HtmlCaptureError::ForeignBuilder)
        );
        assert_eq!(
            preflight_error(preflight::Failure::Admission(
                HtmlEmptyGlobalRegistryError::Interrupted("cancelled")
            )),
            HtmlEmptyGlobalRegistryError::Interrupted("cancelled")
        );
    }
}

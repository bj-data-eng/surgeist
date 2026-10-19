//! Borrowed capture validation. The eventual import owner supplies admission;
//! ordinary capture assembly uses the same algorithm without a work controller.

use super::{HtmlCaptureBuilder, HtmlCaptureError, HtmlCapturedNodeKind, TemplateAssociation, add};
use std::convert::Infallible;

/// Actual finite work and logical scratch reservation observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Work(usize),
    Reserve {
        entries: usize,
        record_bytes: usize,
    },
    StorageBefore {
        requested: usize,
    },
    StorageAfter {
        old: usize,
        actual: usize,
        retained: bool,
    },
    StorageTransfer {
        bytes: usize,
    },
    /// Complete prospective element-value payload, including its destination
    /// bank. This is an admission observation, not another retained receipt.
    ElementValues {
        bytes: usize,
        units: usize,
        observed: bool,
    },
}

#[derive(Debug)]
pub(crate) enum Failure<E> {
    Admission(E),
    Capture(HtmlCaptureError),
    AllocationFailed,
}

impl<E> From<HtmlCaptureError> for Failure<E> {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}

pub(super) fn ordinary(action: Action) -> Result<(), Infallible> {
    // Ordinary assembly has no work or scratch controller. Consume the actual
    // observation without introducing a second accounting policy.
    match action {
        Action::Work(_units) => {}
        Action::Reserve {
            entries: _entries,
            record_bytes: _record_bytes,
        } => {}
        Action::StorageBefore { .. }
        | Action::StorageAfter { .. }
        | Action::StorageTransfer { .. }
        | Action::ElementValues { .. } => {}
    }
    Ok(())
}

pub(super) fn ordinary_error(error: Failure<Infallible>) -> HtmlCaptureError {
    match error {
        Failure::Admission(never) => match never {},
        Failure::Capture(error) => error,
        // Preserve the existing ordinary allocation-error spelling.
        Failure::AllocationFailed => HtmlCaptureError::RepresentationExhausted,
    }
}

/// Caller-owned actual temporary allocations survive a refused borrowed pass.
pub(crate) struct DepthScratch {
    incoming: Vec<Option<usize>>,
    state: Vec<u8>,
    depths: Vec<usize>,
    path: Vec<usize>,
}

impl DepthScratch {
    pub(crate) const fn new() -> Self {
        Self {
            incoming: Vec::new(),
            state: Vec::new(),
            depths: Vec::new(),
            path: Vec::new(),
        }
    }

    pub(super) fn into_depths(self) -> Vec<usize> {
        self.depths
    }
}

pub(super) fn work<E>(
    admit: &mut impl FnMut(Action) -> Result<(), E>,
    units: usize,
) -> Result<(), Failure<E>> {
    admit(Action::Work(units)).map_err(Failure::Admission)
}

pub(super) fn reserve<T, E>(
    values: &mut Vec<T>,
    entries: usize,
    admit: &mut impl FnMut(Action) -> Result<(), E>,
) -> Result<(), Failure<E>> {
    reserve_storage(values, entries, false, false, admit)
}

pub(super) fn reserve_storage<T, E>(
    values: &mut Vec<T>,
    entries: usize,
    retained: bool,
    controlled: bool,
    admit: &mut impl FnMut(Action) -> Result<(), E>,
) -> Result<(), Failure<E>> {
    admit(Action::Reserve {
        entries,
        record_bytes: std::mem::size_of::<T>(),
    })
    .map_err(Failure::Admission)?;
    work(admit, values.len())?;
    let old = values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(super::HtmlCaptureError::RepresentationExhausted)?;
    let needed = values
        .len()
        .checked_add(entries)
        .ok_or(super::HtmlCaptureError::RepresentationExhausted)?;
    if needed > values.capacity() {
        let requested = needed
            .checked_mul(std::mem::size_of::<T>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(super::HtmlCaptureError::RepresentationExhausted)?;
        admit(Action::StorageBefore { requested }).map_err(Failure::Admission)?;
        if controlled {
            values
                .try_reserve_exact(entries)
                .map_err(|_| Failure::AllocationFailed)?;
        } else {
            values
                .try_reserve(entries)
                .map_err(|_| Failure::AllocationFailed)?;
        }
        let actual = values
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(super::HtmlCaptureError::RepresentationExhausted)?;
        admit(Action::StorageAfter {
            old,
            actual,
            retained,
        })
        .map_err(Failure::Admission)?;
    }
    work(admit, 0)
}

/// Reference validation is independent of owning reciprocity and forest depth.
pub(super) fn references<E>(
    builder: &HtmlCaptureBuilder,
    admit: &mut impl FnMut(Action) -> Result<(), E>,
) -> Result<(), Failure<E>> {
    for record in &builder.data.records {
        work(admit, 4)?;
        let HtmlCapturedNodeKind::Element(element) = &record.kind else {
            continue;
        };
        let TemplateAssociation::ShadowReference(target) = element.template else {
            continue;
        };
        work(admit, 2)?;
        if !matches!(
            builder.data.records.get(target).map(|record| &record.kind),
            Some(HtmlCapturedNodeKind::Fragment(fragment))
                if matches!(fragment.kind, super::HtmlCapturedFragmentKind::ShadowRoot(_))
        ) {
            return Err(Failure::Capture(
                HtmlCaptureError::InvalidTemplateContentsReference,
            ));
        }
    }
    Ok(())
}

/// The existing iterative full-forest pass, with admitted reads/writes and
/// all four real reservations. References never become incoming owning edges.
pub(super) fn depths<E>(
    builder: &HtmlCaptureBuilder,
    edges: &[(usize, usize)],
    scratch: &mut DepthScratch,
    admit: &mut impl FnMut(Action) -> Result<(), E>,
) -> Result<(), Failure<E>> {
    work(admit, 5)?;
    let n = builder.data.records.len();
    scratch.incoming.clear();
    scratch.state.clear();
    scratch.depths.clear();
    scratch.path.clear();
    reserve_storage(&mut scratch.incoming, n, false, builder.controlled, admit)?;
    for record in &builder.data.records {
        work(admit, 4)?;
        scratch.incoming.push(record.incoming());
    }
    for &(child, parent) in edges {
        work(admit, 3)?;
        scratch.incoming[child] = Some(parent);
    }
    reserve_storage(&mut scratch.state, n, false, builder.controlled, admit)?;
    for _ in 0..n {
        work(admit, 1)?;
        scratch.state.push(0);
    }
    reserve_storage(&mut scratch.depths, n, false, builder.controlled, admit)?;
    for _ in 0..n {
        work(admit, 1)?;
        scratch.depths.push(0);
    }
    reserve_storage(&mut scratch.path, n, false, builder.controlled, admit)?;
    for start in 0..n {
        work(admit, 2)?;
        if scratch.state[start] != 0 {
            continue;
        }
        let mut cursor = Some(start);
        let mut depth = 0;
        while let Some(i) = cursor {
            work(admit, 5)?;
            match scratch.state[i] {
                1 => return Err(Failure::Capture(HtmlCaptureError::Cycle)),
                2 => {
                    depth = add(scratch.depths[i], 1).map_err(Failure::Capture)?;
                    break;
                }
                _ => {
                    scratch.state[i] = 1;
                    scratch.path.push(i);
                    cursor = scratch.incoming[i];
                }
            }
        }
        loop {
            work(admit, 2)?;
            let Some(i) = scratch.path.pop() else { break };
            builder.depth(depth).map_err(Failure::Capture)?;
            scratch.depths[i] = depth;
            scratch.state[i] = 2;
            if !scratch.path.is_empty() {
                depth = add(depth, 1).map_err(Failure::Capture)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn controlled_record_growth_joins_external_mapping_and_the_full_old_bank() {
        use std::sync::atomic::AtomicBool;
        let cancel = AtomicBool::new(false);
        let generous = HtmlOptionValueEnvelope::new(1_000_000, 0, 100_000);
        let mut owner = HtmlControlledOptionValueCapture::new(
            HtmlCaptureLimits::new(4, 0, 0, 0, 0, 0, 0, 16 * 1024 * 1024),
            HtmlHostFactsRevision::fresh(),
            generous,
            &cancel,
        )
        .unwrap();
        let doc = owner
            .add_document(HtmlDocumentMode::NoQuirks, generous, &cancel)
            .unwrap();
        let before = owner.storage_usage().retained_bytes();
        let mut external_mapping = Vec::with_capacity(32);
        external_mapping.push(doc);
        let external_bytes =
            external_mapping.capacity() * std::mem::size_of::<HtmlCaptureBuilderSubject>();
        // First reserve_exact formed one Record. The second needs a complete
        // two-Record new bank while the old one and caller mapping still live.
        let new_bank = 2 * std::mem::size_of::<super::super::Record>();
        let total = external_bytes + before + new_bank;
        let error = owner
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlOptionValueEnvelope::new(total - external_bytes - 1, 0, 100_000),
                &cancel,
            )
            .unwrap_err();
        assert!(
            matches!(error.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
            resource: HtmlOptionValueResource::LiveBytes, required, .. }) if *required == u64::try_from(before + new_bank).unwrap())
        );
        assert_eq!(owner.storage_usage().retained_bytes(), before);
        owner
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlOptionValueEnvelope::new(total - external_bytes, 0, 100_000),
                &cancel,
            )
            .unwrap();
        assert_eq!(owner.capture_usage().subjects(), 2);
        assert_eq!(external_mapping.len(), 1);
    }

    #[test]
    fn every_borrowed_finish_refusal_retains_original_builder_and_facts() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(4, 8, 0, 10, 4, 0, 0, 16 * 1024 * 1024),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::Quirks,
                HtmlScriptingStatus::Enabled,
                HtmlCaptureRegistryInput::Unavailable,
            )
            .unwrap();
        let text = builder
            .add_text(&document, DocumentString::from("kept"))
            .unwrap();
        builder
            .establish_children(&document, std::slice::from_ref(&text))
            .unwrap();
        let usage = builder.usage();
        let mut scratch = DepthScratch::new();
        let mut actions = 0;
        builder
            .preflight_finish(&mut scratch, &mut |_| {
                actions += 1;
                Ok::<_, ()>(())
            })
            .unwrap();
        for stop in 0..actions {
            let mut at = 0;
            let mut refused_scratch = DepthScratch::new();
            let result = builder.preflight_finish(&mut refused_scratch, &mut |_| {
                let action = at;
                at += 1;
                if action == stop {
                    return Err(stop);
                }
                Ok(())
            });
            assert!(matches!(result, Err(Failure::Admission(value)) if value == stop));
            assert_eq!(at, stop + 1, "no callback after the first refusal");
            assert_eq!(builder.usage(), usage);
        }
        let capture = builder.finish().unwrap();
        let doc = capture
            .node(&capture.subject_id(&document).unwrap())
            .unwrap();
        let HtmlCapturedNodeKind::Document(facts) = doc.kind() else {
            panic!("document")
        };
        assert_eq!(facts.mode(), HtmlDocumentMode::Quirks);
        assert_eq!(facts.scripting(), HtmlScriptingStatus::Enabled);
        assert_eq!(doc.lookup_registry(), HtmlCapturedRegistry::Unavailable);
        let text_id = capture.subject_id(&text).unwrap();
        let HtmlCapturedNodeKind::Text(value) = capture.node(&text_id).unwrap().kind() else {
            panic!("text")
        };
        assert_eq!(value.code_units(), [107, 101, 112, 116]);
    }
}

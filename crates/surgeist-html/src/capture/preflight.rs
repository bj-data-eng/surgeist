//! Borrowed capture validation. The eventual import owner supplies admission;
//! ordinary capture assembly uses the same algorithm without a work controller.

use super::{HtmlCaptureBuilder, HtmlCaptureError, HtmlCapturedNodeKind, TemplateAssociation, add};
use std::convert::Infallible;

/// Actual finite work and logical scratch reservation observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    Work(usize),
    Reserve { entries: usize, record_bytes: usize },
}

#[derive(Debug)]
pub(super) enum Failure<E> {
    Admission(E),
    Capture(HtmlCaptureError),
    AllocationFailed,
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
pub(super) struct DepthScratch {
    incoming: Vec<Option<usize>>,
    state: Vec<u8>,
    depths: Vec<usize>,
    path: Vec<usize>,
}

impl DepthScratch {
    pub(super) const fn new() -> Self {
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

fn work<E>(
    admit: &mut impl FnMut(Action) -> Result<(), E>,
    units: usize,
) -> Result<(), Failure<E>> {
    admit(Action::Work(units)).map_err(Failure::Admission)
}

fn reserve<T, E>(
    values: &mut Vec<T>,
    entries: usize,
    admit: &mut impl FnMut(Action) -> Result<(), E>,
) -> Result<(), Failure<E>> {
    admit(Action::Reserve {
        entries,
        record_bytes: std::mem::size_of::<T>(),
    })
    .map_err(Failure::Admission)?;
    values
        .try_reserve(entries)
        .map_err(|_| Failure::AllocationFailed)
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
    reserve(&mut scratch.incoming, n, admit)?;
    for record in &builder.data.records {
        work(admit, 4)?;
        scratch.incoming.push(record.incoming());
    }
    for &(child, parent) in edges {
        work(admit, 3)?;
        scratch.incoming[child] = Some(parent);
    }
    reserve(&mut scratch.state, n, admit)?;
    for _ in 0..n {
        work(admit, 1)?;
        scratch.state.push(0);
    }
    reserve(&mut scratch.depths, n, admit)?;
    for _ in 0..n {
        work(admit, 1)?;
        scratch.depths.push(0);
    }
    reserve(&mut scratch.path, n, admit)?;
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
    fn every_borrowed_finish_refusal_retains_original_builder_and_facts() {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(4, 8, 0, 10, 4, 0, 0),
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

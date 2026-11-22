//! Canonical readiness and timing for the bounded live local HTML document.

use super::*;
use crate::document::DomDocumentReadiness;

/// One publication in the original live HTML end-of-parsing sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomHostedHtmlLoadStep {
    /// The parser reached its own EOF and readiness becomes interactive.
    Interactive,
    /// The DOMContentLoaded task is about to fire its event.
    ContentLoadedStart,
    /// The DOMContentLoaded event completed.
    ContentLoadedEnd,
    /// The load task changes readiness to complete.
    Complete,
    /// The Window load event is about to fire.
    LoadStart,
    /// The Window load event completed.
    LoadEnd,
    /// The page has become showing, before dispatching its initial pageshow event.
    PageShowing,
    /// The original local Document completely finished loading.
    Finished,
}

impl DomHostedHtmlLoadStep {
    const fn previous(self) -> Option<Self> {
        Some(match self {
            Self::Interactive => return None,
            Self::ContentLoadedStart => Self::Interactive,
            Self::ContentLoadedEnd => Self::ContentLoadedStart,
            Self::Complete => Self::ContentLoadedEnd,
            Self::LoadStart => Self::Complete,
            Self::LoadEnd => Self::LoadStart,
            Self::PageShowing => Self::LoadEnd,
            Self::Finished => Self::PageShowing,
        })
    }
}

/// Load timing captured with a local HTML Document's canonical metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomHostedHtmlLoadTiming {
    step: DomHostedHtmlLoadStep,
    // Each value is obtained once, in order, on the Document's native epoch.
    times: [u64; DomHostedHtmlLoadStep::Finished as usize + 1],
}

impl DomHostedHtmlLoadTiming {
    /// The most recent committed step.
    pub const fn step(self) -> DomHostedHtmlLoadStep {
        self.step
    }

    /// Time of a reached step in microseconds on the hosted source's epoch.
    pub const fn microseconds(self, step: DomHostedHtmlLoadStep) -> Option<u64> {
        if step as usize <= self.step as usize {
            Some(self.times[step as usize])
        } else {
            None
        }
    }

    pub(crate) const fn readiness(self) -> DomDocumentReadiness {
        match self.step {
            DomHostedHtmlLoadStep::Interactive
            | DomHostedHtmlLoadStep::ContentLoadedStart
            | DomHostedHtmlLoadStep::ContentLoadedEnd => DomDocumentReadiness::Interactive,
            _ => DomDocumentReadiness::Complete,
        }
    }
}

impl DomHostedSource {
    /// Publish the next local HTML load step for the original active association.
    /// The native lifecycle owner supplies its admitted monotonic clock sample.
    pub fn publish_local_html_load(
        &self,
        dom: &mut Dom,
        document: DocumentId,
        global: DomHostedGlobalId,
        step: DomHostedHtmlLoadStep,
        microseconds: u64,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let (metadata, entry, navigable) =
            self.local_html_current_document(dom, document, global, cancel)?;
        let environment = metadata.document_environment();
        let old = environment
            .hosted
            .as_ref()
            .ok_or(DomHostedError::StaleIdentity)?;
        if !matches!(
            old.phase,
            DocumentPhase::InitialEntry {
                load: HostedDocumentLoad::Loading,
                ..
            }
        ) {
            return Err(DomHostedError::WrongPhase);
        }
        let timing = metadata.local_html_load_timing();
        if timing.map(DomHostedHtmlLoadTiming::step) != step.previous() {
            return Err(DomHostedError::WrongPhase);
        }
        let previous = timing.map_or(old.navigation_start.microseconds, |timing| {
            timing.times[timing.step as usize]
        });
        if microseconds < previous || !microseconds.is_multiple_of(100) {
            return Err(DomHostedError::InvalidClockSample);
        }
        let mut times = timing.map_or(
            [0; DomHostedHtmlLoadStep::Finished as usize + 1],
            |timing| timing.times,
        );
        times[step as usize] = microseconds;
        let association = if step == DomHostedHtmlLoadStep::Finished {
            let retained = self.retain(size_of::<DocumentAssociation>(), cancel)?;
            let moment = DomHostedMoment {
                source: self.id(),
                microseconds,
            };
            Some(Arc::new(DocumentAssociation {
                source: old.source.clone(),
                host: old.host,
                context: old.context,
                global: old.global,
                navigation_start: old.navigation_start,
                policy: old.policy,
                phase: DocumentPhase::InitialEntry {
                    entry,
                    navigable,
                    load: HostedDocumentLoad::Loaded(moment),
                },
                _retained: retained,
            }))
        } else {
            None
        };
        let mut draft = Draft::new(dom, DomBatch::new().owner, cancel)?;
        let mut metadata = draft.document_metadata_mut(Key::Existing(document.node()), cancel)?;
        metadata.set_local_html_load_timing(DomHostedHtmlLoadTiming { step, times });
        if let Some(association) = association {
            metadata.set_hosted_association(association);
        }
        metadata.commit(cancel)?;
        let prepared = draft.finish(dom, cancel)?;
        dom.publish(prepared, cancel)?;
        Ok(())
    }
}

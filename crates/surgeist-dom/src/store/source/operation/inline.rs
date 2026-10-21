//! Per-invocation inline capability and per-advance borrowed execution.
use super::*;
use crate::store::inline_style::recording::{Profiles, Recording, Resolver};
use crate::{
    DomInlineStyleBatchContext, DomInlineStyleError, DomInlineStyleRequestLimits,
    DomInlineStyleSourceContext, DomInlineStyleSourceProgress, DomInlineStyleSourcePublication,
};
use surgeist_cssom::{CssomAdmission, CssomContext, CssomDeclarationCandidate};

pub(super) struct SourceInline {
    profiles: Arc<Profiles>,
    limits: DomInlineStyleRequestLimits,
}

pub(super) enum AdvanceError {
    Dom(DomError),
    Inline(DomInlineStyleError),
    Unwind(Box<dyn std::any::Any + Send>),
    NativePrepared,
}
impl From<DomError> for AdvanceError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}
impl From<DomInlineStyleError> for AdvanceError {
    fn from(error: DomInlineStyleError) -> Self {
        Self::Inline(error)
    }
}

pub(super) struct Execution<'a> {
    pub(super) resolver: Option<&'a mut Resolver<'a>>,
    pub(super) publication: Option<DomInlineStyleSourcePublication>,
    pub(super) native: Option<meta::Execution>,
}
impl Execution<'_> {
    pub(super) fn disabled() -> Self {
        Self {
            resolver: None,
            publication: None,
            native: None,
        }
    }
}

impl Dom {
    /// Begin one authentic Source root with immutable inline profiles. The
    /// configured finite inline policy is required even with no current bindings:
    /// an authenticated event callback may create the first binding later.
    /// Resumption retains these original profiles and cumulative work counters.
    pub fn begin_source_edit_with_inline_styles(
        &mut self,
        edit: DomSourceEdit,
        context: DomInlineStyleSourceContext,
        limits: DomSourceLimits,
        request_limits: DomInlineStyleRequestLimits,
        cancel: &AtomicBool,
    ) -> Result<DomSourceLease<'_>, DomInlineStyleError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if context.basis.owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        if context.basis != self.snapshot().stamp() {
            return Err(DomInlineStyleError::StaleContext);
        }
        if self.inline_styles.is_none() {
            return Err(DomInlineStyleError::Unconfigured);
        }
        let lease = self.begin_source_edit(edit, limits, cancel)?;
        let operation = lease
            .dom
            .source_operation
            .as_ref()
            .expect("new original root");
        let formation = formation::Formation::new(&operation.draft, cancel)?;
        let context = DomInlineStyleBatchContext {
            basis: context.basis,
            batch: operation.draft.owner.clone(),
            profiles: context.profiles,
        };
        let recording = lease
            .dom
            .form_inline_recording(context, formation.as_ref(), true, cancel)?
            .expect("configured Source capability");
        let operation = lease.dom.source_operation.as_mut().expect("original root");
        operation.inline_source = Some(SourceInline {
            profiles: recording.profiles(),
            limits: request_limits,
        });
        operation.refresh_admission(cancel)?;
        Ok(lease)
    }
}

impl DomSourceLease<'_> {
    /// Advance the same retained Source invocation with a pure borrowed resolver.
    /// No callback is retained. Speculative command admission records inputs but
    /// never invokes this resolver. Producer evidence is returned once alongside
    /// the actual canonical segment; repeated requests do not repeat it.
    pub fn advance_with_inline_styles(
        &mut self,
        mut resolver: impl FnMut(
            &[CssomDeclarationCandidate],
            &CssomContext,
        ) -> Result<CssomAdmission, surgeist_cssom::CssomError>,
        cancel: &AtomicBool,
    ) -> Result<DomInlineStyleSourceProgress, DomInlineStyleError> {
        let mut execution = Execution {
            resolver: Some(&mut resolver),
            publication: None,
            native: None,
        };
        match self.advance_execution(&mut execution, cancel) {
            Ok(source) => Ok(DomInlineStyleSourceProgress {
                source,
                inline: execution.publication,
            }),
            Err(AdvanceError::Dom(error)) => Err(error.into()),
            Err(AdvanceError::Inline(error)) => Err(error),
            Err(AdvanceError::Unwind(payload)) => std::panic::resume_unwind(payload),
            Err(AdvanceError::NativePrepared) => unreachable!("no native preparation capability"),
        }
    }
}

impl Operation {
    pub(super) fn inline_request_limits(&self) -> Option<DomInlineStyleRequestLimits> {
        self.inline_source.as_ref().map(|source| source.limits)
    }

    pub(super) fn prepare_rebased_inline(&self, dom: &Dom, draft: &mut Draft) {
        // Authenticated callback reentry discarded the previous Draft. The new
        // canonical basis has no speculative log, so current membership/order
        // can be installed without truncating an uncommitted same-lease retry.
        draft.inline_membership = dom.inline_membership();
        draft.inline_recording = dom.blocked_inline_recording().or_else(|| {
            self.inline_source.as_ref().map(|source| {
                Recording::from_profiles(
                    source.profiles.clone(),
                    &dom.inline_styles.as_ref().expect("configured owner").index,
                )
            })
        });
    }
    pub(super) fn refresh_inline_execution(
        &mut self,
        dom: &Dom,
        enabled: bool,
    ) -> Result<(), DomError> {
        self.draft.inline_membership = dom.inline_membership();
        if let Some(blocked) = dom.blocked_inline_recording() {
            if self
                .draft
                .inline_recording
                .as_ref()
                .is_some_and(|r| r.has_inputs())
            {
                blocked.require_unblocked()?;
            }
            if !self
                .draft
                .inline_recording
                .as_ref()
                .is_some_and(|r| r.is_blocked())
            {
                self.draft.inline_recording = Some(blocked);
            }
            return Ok(());
        }
        if self
            .draft
            .inline_recording
            .as_ref()
            .is_some_and(|r| r.is_blocked())
        {
            self.draft.inline_recording = None;
        }
        if !enabled {
            self.draft.inline_recording = None;
        } else if self.draft.inline_recording.is_none() {
            self.draft.inline_recording = self.inline_source.as_ref().map(|source| {
                Recording::from_profiles(
                    source.profiles.clone(),
                    &dom.inline_styles.as_ref().expect("configured owner").index,
                )
            });
        }
        Ok(())
    }

    pub(super) fn publication_inline(
        &mut self,
        dom: &mut Dom,
        target: PublicationTarget,
        disposition: Option<meta::Disposition>,
        checkpoint: Option<&checkpoint::Checkpoint>,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<PublishedSegment, AdvanceError> {
        let incomplete = !matches!(disposition, Some(meta::Disposition::Complete { .. }));
        let mut segment = self
            .prepare_segment(dom, incomplete, target, None, checkpoint, cancel)
            .map_err(|error| match error {
                crate::DomHostedError::Dom(error) => AdvanceError::Dom(error),
                _ => unreachable!("no hosted permit"),
            })?;
        if execution.native.is_some() && self.meta_reached() {
            let disposition = disposition.ok_or(DomError::InvalidSourceOperation)?;
            self.pending_meta = Some(meta::Pending::new(segment, disposition));
            return Err(AdvanceError::NativePrepared);
        }
        let has_inputs = segment
            .prepared
            .inline_recording
            .as_ref()
            .is_some_and(Recording::has_inputs);
        let inline = if has_inputs {
            // Current inline-bearing public roots return immediately after this
            // publication. No lifetime history or overwrite/recovery API exists.
            assert!(
                execution.publication.is_none(),
                "one inline boundary per advance"
            );
            let source = self
                .inline_source
                .as_ref()
                .expect("recorded Source capability");
            let formation = segment
                .prepared
                ._formation
                .as_ref()
                .expect("Source formation");
            formation.retain_bytes(size_of::<DomInlineStyleSourcePublication>(), cancel)?;
            let required = segment
                .outputs
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            if required
                .checked_add(usize::from(incomplete))
                .is_none_or(|n| n > self.draft.source_limits.expect("Source limits").outputs)
            {
                return Err(DomError::SourceOutputLimit.into());
            }
            segment.outputs = required;
            let resolver = execution
                .resolver
                .as_deref_mut()
                .expect("enabled execution");
            // Only precommit work can call the pure resolver. Transport its exact
            // panic through the normal checkpoint/receipt rollback and original
            // Operation restoration before resuming unwind at the public front.
            let prepared = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                dom.prepare_inline_batch(&segment.prepared, source.limits, resolver, cancel)
            }))
            .map_err(AdvanceError::Unwind)??;
            prepared
                .map(|batch| dom.commit_inline_batch(batch))
                .transpose()?
        } else {
            None
        };
        // CSSOM commit was the last fallible operation. Everything below is an
        // infallible transfer of preadmitted original publication evidence.
        let published = segment.publish(self, dom, checkpoint);
        if let Some(inline) = inline {
            execution.publication = Some(DomInlineStyleSourcePublication {
                dom: published.commit.changes.clone(),
                inline,
            });
        }
        Ok(published)
    }
}

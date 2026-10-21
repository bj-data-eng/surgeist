//! Inverses for the original Source interpreter's control state.
//!
//! The transaction journal owns these values and their retained custody. No
//! inverse consults cancellation, spends work, issues an identity or calls an
//! external service. Frame traversal storage keeps its own sequence inverses;
//! these entries move the owning frame header rather than copying a worklist.
use super::*;

pub(in crate::store) enum Undo {
    Method(Option<method::Frame>),
    Registries {
        semantic: Option<Arc<crate::store::registry::Bank>>,
        custody: Option<transaction::auxiliary::RegistryCustody>,
    },
    HostedHistory {
        semantic: Option<Arc<crate::store::hosted::history::Graph>>,
        custody: Option<transaction::auxiliary::HistoryCustody>,
    },

    DialogReply {
        cursor: surgeist_html::HtmlDialogCursor,
        invocation: Invocation,
    },
    FocusingReply {
        cursor: surgeist_html::HtmlDialogFocusingCursor,
        invocation: Invocation,
    },
    EffectPushed(Arc<super::effects::FrameLife>),
    EffectsCapacity(super::effects::EffectsCapacity),
    EffectPopped(select::EffectFrame),
    StyleEffect {
        index: usize,
        previous: style::Frame,
    },
    AutofocusEffect {
        index: usize,
        previous: autofocus::Frame,
    },
    DialogHookEffect {
        index: usize,
        previous: dialog::HookFrame,
    },
}

impl Undo {
    #[cfg(test)]
    pub(in crate::store) fn journal_bytes(&self) -> usize {
        match self {
            Self::EffectsCapacity(previous) => previous.debit.usage().bytes(),
            Self::EffectPopped(previous) => previous.journal_bytes(),
            // The journal's admitted row capacity contains these inline
            // headers. Shared frontier aliases add no materialized backing.
            _ => 0,
        }
    }
    pub(in crate::store) fn restore(self, draft: &mut Draft) {
        match self {
            Self::Method(previous) => draft.method = previous,
            Self::Registries { semantic, custody } => {
                draft.registries = semantic;
                draft.auxiliary.registries = custody;
            }
            Self::HostedHistory { semantic, custody } => {
                draft.hosted_history = semantic;
                draft.auxiliary.history = custody;
            }

            Self::DialogReply { cursor, invocation } => {
                let Some(method::Frame::Dialog(frame)) = draft.method.as_mut() else {
                    unreachable!("journaled Dialog reply retains its original method")
                };
                frame.cursor = cursor;
                frame.invocation = invocation;
            }
            Self::FocusingReply { cursor, invocation } => {
                let Some(method::Frame::Dialog(frame)) = draft.method.as_mut() else {
                    unreachable!("journaled focusing reply retains its original method")
                };
                let called = frame
                    .focusing
                    .as_mut()
                    .expect("journaled original focusing call");
                called.cursor = cursor;
                called.invocation = invocation;
            }
            Self::EffectsCapacity(mut previous) => {
                #[cfg(test)]
                let moved = draft.effects.len();
                previous.old.append(&mut draft.effects);
                #[cfg(test)]
                super::effects::record_frame_writes(moved);
                draft.effects = previous.old;
                draft.effects_charge = previous.debit;
                draft.effect_bytes = draft.effects.capacity() * size_of::<select::EffectFrame>();
                drop(previous.scratch);
            }
            Self::EffectPushed(life) => {
                if life.is_live() {
                    let frame = draft.effects.pop().expect("journaled Source effect push");
                    assert!(
                        frame
                            .life
                            .as_ref()
                            .is_some_and(|actual| Arc::ptr_eq(actual, &life)),
                        "inverse removes its original live frame"
                    );
                    if let Some(actual) = &frame.life {
                        actual.consume();
                    }
                    drop(frame);
                }
            }
            Self::StyleEffect { index, previous } => {
                draft.effects[index].effect = select::Effect::Style(previous);
            }
            Self::AutofocusEffect { index, previous } => {
                draft.effects[index].effect = select::Effect::AutofocusInsertion(previous);
            }
            Self::DialogHookEffect { index, previous } => {
                draft.effects[index].effect = select::Effect::DialogHook(previous);
            }
            Self::EffectPopped(previous) => {
                // Capacity replacement is a separate earlier inverse. A pop
                // never releases its original backing before this mark seals.
                assert!(draft.effects.len() < draft.effects.capacity());
                if let Some(life) = &previous.life {
                    life.restore();
                }
                draft.effects.push(previous);
                #[cfg(test)]
                super::effects::record_frame_writes(1);
            }
        }
    }
}

impl Draft {
    pub(in crate::store) fn set_registries(
        &mut self,
        next: Option<Arc<crate::store::registry::Bank>>,
        incoming: Option<formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // A refusal reserving the inverse must destroy the consumed semantic
        // output before its incoming producer credit, including on this path.
        let staged = (next, incoming);
        let retain = self.prepare_undo(cancel)?;
        let registry = self.custody_registry().clone();
        let (next, incoming) = staged;
        let incoming = incoming.and_then(formation::Formation::into_debit);
        let (next, custody) = match next {
            Some(next) => {
                let prepared = transaction::auxiliary::prepare_registry(
                    next,
                    incoming,
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    &mut || self.source_step(cancel),
                )?;
                let (semantic, custody) = prepared.into_parts();
                (Some(semantic), Some(custody))
            }
            None => {
                drop(incoming);
                (None, None)
            }
        };
        let semantic = std::mem::replace(&mut self.registries, next);
        let custody = std::mem::replace(&mut self.auxiliary.registries, custody);
        if retain {
            self.record_source_undo_prepared(Undo::Registries { semantic, custody });
        } else {
            drop(semantic);
            drop(custody);
        }
        Ok(())
    }
    pub(in crate::store) fn set_hosted_history(
        &mut self,
        next: Option<Arc<crate::store::hosted::history::Graph>>,
        incoming: Option<formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // A refusal reserving the inverse must destroy the consumed semantic
        // output before its incoming producer credit, including on this path.
        let staged = (next, incoming);
        let retain = self.prepare_undo(cancel)?;
        let registry = self.custody_registry().clone();
        let (next, incoming) = staged;
        let incoming = incoming.and_then(formation::Formation::into_debit);
        let (next, custody) = match next {
            Some(next) => {
                let prepared = transaction::auxiliary::prepare_history(
                    next,
                    incoming,
                    &registry,
                    &mut |usage| self.admit_custody_usage(usage, cancel),
                    &mut || self.source_step(cancel),
                )?;
                let (semantic, custody) = prepared.into_parts();
                (Some(semantic), Some(custody))
            }
            None => {
                drop(incoming);
                (None, None)
            }
        };
        let semantic = std::mem::replace(&mut self.hosted_history, next);
        let custody = std::mem::replace(&mut self.auxiliary.history, custody);
        if retain {
            self.record_source_undo_prepared(Undo::HostedHistory { semantic, custody });
        } else {
            drop(semantic);
            drop(custody);
        }
        Ok(())
    }
    /// Install an already admitted continuation header. If a savepoint needs
    /// the old header, journal reservation precedes the ownership move.
    pub(in crate::store) fn prepare_method_edit(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.prepare_undo(cancel)? {
            // Method cursors are fixed bodies; a called traversal's paired
            // persistent frontier shares its roots and preserves exact custody.
            self.record_source_undo_prepared(Undo::Method(self.method.clone()));
        }
        Ok(())
    }

    pub(in crate::store) fn source_set_method(
        &mut self,
        next: Option<method::Frame>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let retain = self.prepare_undo(cancel)?;
        let previous = std::mem::replace(&mut self.method, next);
        if retain {
            self.record_source_undo_prepared(Undo::Method(previous));
        }
        Ok(())
    }

    pub(in crate::store) fn source_prepare_dialog_reply(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(method::Frame::Dialog(frame)) = self.method.as_ref() else {
            return Err(DomError::InvalidSourceOperation);
        };
        let cursor = frame.cursor;
        let invocation = frame.invocation;
        if self.prepare_undo(cancel)? {
            self.record_source_undo_prepared(Undo::DialogReply { cursor, invocation });
        }
        Ok(())
    }
    pub(in crate::store) fn source_prepare_focusing_reply(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(method::Frame::Dialog(frame)) = self.method.as_ref() else {
            return Err(DomError::InvalidSourceOperation);
        };
        let called = frame
            .focusing
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let cursor = called.cursor;
        let invocation = called.invocation;
        if self.prepare_undo(cancel)? {
            self.record_source_undo_prepared(Undo::FocusingReply { cursor, invocation });
        }
        Ok(())
    }

    pub(in crate::store) fn source_set_style_frame(
        &mut self,
        next: style::Frame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let index = self
            .effects
            .len()
            .checked_sub(1)
            .ok_or(DomError::StyleProtocol)?;
        let select::Effect::Style(previous) = &self.effects[index].effect else {
            return Err(DomError::StyleProtocol);
        };
        let previous = *previous;
        let retain = self.prepare_undo(cancel)?;
        if retain {
            self.record_source_undo_prepared(Undo::StyleEffect { index, previous });
        }
        self.effects[index].effect = select::Effect::Style(next);
        Ok(())
    }

    pub(in crate::store) fn source_set_autofocus_frame(
        &mut self,
        next: autofocus::Frame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let index = self
            .effects
            .len()
            .checked_sub(1)
            .ok_or(DomError::InvalidSourceOperation)?;
        let select::Effect::AutofocusInsertion(previous) = &self.effects[index].effect else {
            return Err(DomError::InvalidSourceOperation);
        };
        let previous = *previous;
        let retain = self.prepare_undo(cancel)?;
        if retain {
            self.record_source_undo_prepared(Undo::AutofocusEffect { index, previous });
        }
        self.effects[index].effect = select::Effect::AutofocusInsertion(next);
        Ok(())
    }

    pub(in crate::store) fn source_set_dialog_hook_frame(
        &mut self,
        next: dialog::HookFrame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let index = self
            .effects
            .len()
            .checked_sub(1)
            .ok_or(DomError::InvalidSourceOperation)?;
        let select::Effect::DialogHook(previous) = &self.effects[index].effect else {
            return Err(DomError::InvalidSourceOperation);
        };
        let previous = *previous;
        if self.prepare_undo(cancel)? {
            self.record_source_undo_prepared(Undo::DialogHookEffect { index, previous });
        }
        self.effects[index].effect = select::Effect::DialogHook(next);
        Ok(())
    }
}

/// Prepared fixed-body acknowledgment of the original fullscreen handoff. The
/// containing loan's real layout admission owns this token's inline storage.
pub(in crate::store) struct PreparedFullscreenControl {
    id: crate::DomFullscreenExitId,
    finish_method: bool,
}

impl Draft {
    pub(in crate::store) fn prepare_fullscreen_handoff_control(
        &self,
        id: crate::DomFullscreenExitId,
        cancel: &AtomicBool,
    ) -> Result<PreparedFullscreenControl, DomError> {
        self.source_step(cancel)?;
        self.fullscreen_exit
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        Ok(PreparedFullscreenControl {
            id,
            finish_method: matches!(self.method, Some(method::Frame::FullscreenExit(_))),
        })
    }

    /// Called inside the actual publication fence after the prepared sparse
    /// rebase seals the old epoch. No private inverse may undo this issued job.
    pub(in crate::store) fn commit_fullscreen_handoff(
        &mut self,
        prepared: PreparedFullscreenControl,
        _committed: &transaction::PublicationCommitted,
    ) {
        self.fullscreen_exit = None;
        self.fullscreen_work_custody = Some(prepared.id);
        if prepared.finish_method {
            self.method = Some(method::Frame::FullscreenExit(Some(
                crate::DomFullscreenExitReturn::Pending(prepared.id),
            )));
        }
    }
}

/// Fixed interpreter headers at a rollback boundary. Record/sequence values
/// belong to the inverse journal; numerical work/peaks are deliberately absent.
pub(in crate::store) struct Snapshot {
    fullscreen_exit: Option<fullscreen::exit::Frame>,
    hosted_pending: Option<operation::hosted::ChildHook>,
    notification_queued: bool,
    effects_running: bool,
    effect_floor: usize,
    effect_bytes: usize,
    effect_scoped: bool,
    input: input_processing::InputProcessing,
    meta_len: Option<usize>,
    meta_scope: Option<(
        crate::DomSourceOperationId,
        crate::store::source::operation::meta::SubjectScope,
    )>,
}
impl Snapshot {
    pub(in crate::store) fn capture(draft: &Draft) -> Self {
        Self {
            fullscreen_exit: draft.fullscreen_exit,
            hosted_pending: draft.hosted_pending,
            notification_queued: draft.notification_queued,
            effects_running: draft.effects_running,
            effect_floor: draft.effect_floor,
            effect_bytes: draft.effect_bytes,
            effect_scoped: draft.effect_scoped,
            input: draft.input_processing.checkpoint(),
            meta_len: draft.meta_recording.as_ref().map(|r| r.entries.len()),
            meta_scope: draft.meta_scope.clone(),
        }
    }
    pub(in crate::store) fn restore(mut self, draft: &mut Draft) {
        self.input.preserve_spent(&draft.input_processing);
        draft.input_processing = self.input;
        if let (Some(len), Some(recording)) = (self.meta_len, draft.meta_recording.as_mut()) {
            recording.rollback(len);
        }
        draft.meta_scope = self.meta_scope;
        draft.fullscreen_exit = self.fullscreen_exit;
        draft.hosted_pending = self.hosted_pending;
        draft.notification_queued = self.notification_queued;
        draft.effects_running = self.effects_running;
        draft.effect_floor = self.effect_floor;
        draft.effect_bytes = self.effect_bytes;
        draft.effect_scoped = self.effect_scoped;
        // Preflight creates its scan only after taking this boundary. Its
        // reached counts are moved out before restoration on the normal path.
        draft.source_headroom = None;
    }
}

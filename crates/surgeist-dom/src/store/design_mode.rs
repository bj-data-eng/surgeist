//! designMode's original setter and called focusing continuation.
use super::*;
use crate::{DomDesignModeRangeDisposition, DomSourceDesignModeStep};
use surgeist_html::{
    HtmlDesignModeAction as Action, HtmlDesignModeCursor, HtmlDesignModeInput,
    HtmlDesignModeReply as Reply, HtmlFocusingCursor, HtmlFocusingFallback,
};

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub document: Key,
    pub cursor: HtmlDesignModeCursor,
    pub range: Option<DomDesignModeRangeDisposition>,
    pub target: Option<Key>,
    pub focusing: Option<HtmlFocusingCursor>,
    pub invocation: source::Invocation,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct Pending {
    pub document: Key,
    pub step: DomSourceDesignModeStep,
    pub target: Option<Key>,
    pub range: Option<DomDesignModeRangeDisposition>,
    pub invocation: source::Invocation,
}
impl Frame {
    fn pending(&self) -> Option<Pending> {
        let step = match self.cursor.action() {
            Action::ReadActiveRange => DomSourceDesignModeStep::ReadActiveRange,
            Action::RunFocusingSteps => {
                DomSourceDesignModeStep::Focusing(self.focusing.as_ref()?.action())
            }
            _ => return None,
        };
        Some(Pending {
            document: self.document,
            step,
            target: self.target,
            range: self.range,
            invocation: self.invocation,
        })
    }
}
impl Draft {
    pub(in crate::store) fn pending_design_mode(&self) -> Option<Pending> {
        let method::Frame::DesignMode(frame) = self.method.as_ref()? else {
            return None;
        };
        frame.pending()
    }
    pub(super) fn design_mode_boundary(
        &mut self,
        document: Key,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        self.prepare_method_edit(cancel)?;
        let mut frame = match self.method.take() {
            Some(method::Frame::DesignMode(frame)) if frame.document == document => frame,
            Some(other) => {
                self.method = Some(other);
                return Err(DomError::InvalidSourceOperation);
            }
            None => {
                // The producer reads at most three UTF-16 units without trimming.
                for _ in 0..value.len().min(3) {
                    self.source_step(cancel)?;
                }
                Frame {
                    document,
                    cursor: HtmlDesignModeCursor::new(HtmlDesignModeInput::from_utf16(
                        value.code_units(),
                    )),
                    range: None,
                    target: None,
                    focusing: None,
                    invocation: source::Invocation::Unissued,
                }
            }
        };
        let result = self.drive_design_mode(&mut frame, cancel);
        // Source needs even a completed frame until final publication succeeds.
        // An atomic batch has no parked/result owner and drains this local call.
        if self.source_limits.is_some() || result.is_err() {
            self.method = Some(method::Frame::DesignMode(frame));
        }
        result
    }
    fn drive_design_mode(
        &mut self,
        frame: &mut Frame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        loop {
            self.source_step(cancel)?;
            let reply = match frame.cursor.action() {
                Action::ReadEnabled => {
                    let DomNodeKind::Document(metadata) = &self.node(frame.document)?.kind else {
                        return Err(DomError::WrongNodeKind);
                    };
                    Reply::Enabled(metadata.design_mode_enabled())
                }
                Action::SetEnabled { enabled } => {
                    self.with_node_fields(frame.document, cancel, |fields| {
                        let DomNodeKind::Document(metadata) = &mut fields.kind else {
                            return Err(DomError::WrongNodeKind);
                        };
                        metadata.set_design_mode_enabled(enabled);
                        Ok(())
                    })??;
                    Reply::Applied
                }
                Action::ReadActiveRange => match self.node(frame.document)?.active_range {
                    crate::DomActiveRangeState::Unavailable => {
                        return Err(DomError::DesignModeProcessingUnavailable {
                            step: DomSourceDesignModeStep::ReadActiveRange,
                        });
                    }
                    crate::DomActiveRangeState::Known(None) => {
                        frame.range = Some(DomDesignModeRangeDisposition::NoActiveRange);
                        Reply::NoActiveRange
                    }
                    crate::DomActiveRangeState::Known(Some(range)) => {
                        self.range_record(range)?;
                        // This identity is retained only until the same-owner reset.
                        // No event/service boundary exists between these actions.
                        frame.range = Some(DomDesignModeRangeDisposition::RangeReset(range));
                        Reply::ActiveRange
                    }
                },
                Action::ResetActiveRange => {
                    let Some(DomDesignModeRangeDisposition::RangeReset(range)) = frame.range else {
                        return Err(DomError::InvalidSourceOperation);
                    };
                    self.reset_design_mode_range(frame.document, range, cancel)?;
                    Reply::Applied
                }
                Action::ReadDocumentElement => {
                    frame.target = None;
                    for child in self.node(frame.document)?.children.iter() {
                        self.source_step(cancel)?;
                        if matches!(self.node(child)?.kind, DomNodeKind::Element(_)) {
                            frame.target = Some(child);
                            break;
                        }
                    }
                    Reply::DocumentElementPresent(frame.target.is_some())
                }
                Action::RunFocusingSteps => {
                    let target = frame.target.ok_or(DomError::InvalidSourceOperation)?;
                    self.node(target)?;
                    let called = frame.focusing.get_or_insert_with(|| {
                        HtmlFocusingCursor::new(HtmlFocusingFallback::Omitted)
                    });
                    // The actual document element is the original target. Both
                    // fallback and trigger are omitted. No native eligibility
                    // producer exists yet (#433); no Dialog predicate applies.
                    if called.action() != surgeist_html::HtmlFocusingAction::Complete {
                        return Err(DomError::DesignModeProcessingUnavailable {
                            step: DomSourceDesignModeStep::Focusing(called.action()),
                        });
                    }
                    Reply::FocusingStepsCompleted
                }
                Action::Complete => return Ok(()),
            };
            frame
                .cursor
                .reply(reply)
                .map_err(DomError::DesignModeProtocol)?;
        }
    }
    fn reset_design_mode_range(
        &mut self,
        document: Key,
        range: crate::DomRangeId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !matches!(self.node(document)?.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if self.node(document)?.active_range != crate::DomActiveRangeState::Known(Some(range)) {
            return Err(DomError::InvalidSourceOperation);
        }
        self.range_record(range)?;
        self.source_step(cancel)?;
        let boundary = crate::range::Boundary {
            container: document,
            offset: 0,
        };
        // All fallible validation precedes the single whole-record assignment.
        self.replace_range_record(
            range,
            crate::range::Record {
                start: boundary,
                end: boundary,
            },
            cancel,
        )?;
        Ok(())
    }
}

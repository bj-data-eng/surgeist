//! Consume one actual original processing receipt into networking custody.
use super::admission::{Custody, Steps, current_storage};
use super::*;
use surgeist_dom::{
    DomParsedHtmlStyleNotificationDisposition as Disposition, DomParsedHtmlStyleNotificationError,
    DomStyleAssociationId,
};

/// Actual queue formation or the precise no-task outcome for this occurrence.
#[derive(Debug)]
pub enum DomStyleParsedNotificationProgress {
    /// Genuine original NoImports completion now owns one networking task.
    Queued {
        /// Exact task issued for this original occurrence.
        task: DomStyleNetworkingTaskId,
    },
    /// The actual original sheet's owner was cleared; no task was issued.
    OwnerAssociationRemoved {
        /// Actual original association whose sheet owner was cleared.
        association: DomStyleAssociationId,
    },
    /// Real imports still require critical-resource outcomes.
    PendingImports {
        /// Original association still awaiting critical resources.
        association: DomStyleAssociationId,
    },
}
/// Queue formation refused without consuming the retained original receipt.
#[derive(Debug)]
pub enum DomStyleParsedNotificationError {
    /// Runtime ownership, admission or Session state refused.
    Runtime(DomStyleRuntimeError),
    /// Canonical original qualification or producer owner refused.
    Notification(DomParsedHtmlStyleNotificationError),
    /// No retained processing receipt exists for the exact qualification.
    ProcessingUnavailable {
        /// Original contribution requested.
        contribution: DomParsedHtmlAssociationId,
    },
    /// Rendering has not handed off the exact notification authority.
    RenderingIncomplete {
        /// Original association awaiting its rendering handoff.
        association: DomStyleAssociationId,
    },
    /// The exact receipt already owns a networking task.
    AlreadyQueued {
        /// Previously issued task; no new task was formed.
        task: DomStyleNetworkingTaskId,
    },
}
impl From<DomStyleRuntimeError> for DomStyleParsedNotificationError {
    fn from(value: DomStyleRuntimeError) -> Self {
        Self::Runtime(value)
    }
}
impl From<DomParsedHtmlStyleNotificationError> for DomStyleParsedNotificationError {
    fn from(value: DomParsedHtmlStyleNotificationError) -> Self {
        Self::Notification(value)
    }
}
impl From<DomParsedHtmlStyleError> for DomStyleParsedNotificationError {
    fn from(value: DomParsedHtmlStyleError) -> Self {
        Self::Notification(value.into())
    }
}
impl std::fmt::Display for DomStyleParsedNotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(e) => e.fmt(f),
            Self::Notification(e) => e.fmt(f),
            Self::ProcessingUnavailable { .. } => {
                f.write_str("original processing receipt unavailable")
            }
            Self::RenderingIncomplete { .. } => {
                f.write_str("original rendering continuation unavailable")
            }
            Self::AlreadyQueued { .. } => f.write_str("original notification already queued"),
        }
    }
}
impl std::error::Error for DomStyleParsedNotificationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(e) => Some(e),
            Self::Notification(e) => Some(e),
            _ => None,
        }
    }
}
impl DomStyleRuntime {
    /// Transfer this original NoImports receipt into the existing networking
    /// queue. Queueing supplies no synchronous frontier or dispatch permission.
    pub fn queue_parsed_style_notification(
        &self,
        dom: &mut DomEventDom<'_>,
        qualification: &DomParsedHtmlStyleQualification,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleParsedNotificationProgress, DomStyleParsedNotificationError> {
        let owner = self.original()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        if state.dom != dom.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        drop(state.require_sheet_set_idle(dom)?);
        let mut hold =
            dom.raw_mut()
                .begin_parsed_html_style(qualification.contribution(), limits, cancel)?;
        let steps = Steps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut custody = Custody {
            bytes: size_of::<DomStyleParsedNotificationProgress>(),
            shared: 0,
            units: 0,
            current: 0,
            maximum: state.limits.storage_bytes,
            outputs: 1,
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        custody.current = current_storage(&state, &mut hold, &custody, &steps)?;
        custody.check()?;
        let mut selected = None;
        for (index, row) in state.completions.iter().enumerate() {
            steps.prefix(&hold)?;
            if row
                .processing()
                .is_some_and(|record| record.qualification == *qualification)
            {
                if let Some(task) = row.as_task() {
                    return Err(DomStyleParsedNotificationError::AlreadyQueued { task: task.id() });
                }
                selected = Some(index);
                break;
            }
        }
        let index = selected.ok_or(DomStyleParsedNotificationError::ProcessingUnavailable {
            contribution: qualification.contribution(),
        })?;
        let record = state.completions[index]
            .processing()
            .expect("selected actual receipt");
        let sheet = &record.sheet;
        if record.dependencies == ImportDependencies::Imports {
            return Ok(DomStyleParsedNotificationProgress::PendingImports {
                association: sheet.association(),
            });
        }
        if record.continuation.is_none() {
            return Err(DomStyleParsedNotificationError::RenderingIncomplete {
                association: sheet.association(),
            });
        }
        custody.add(
            size_of::<Task>()
                + size_of::<std::rc::Rc<DomStyleCssomRoot>>()
                + size_of::<Option<Disposition>>(),
        )?;
        custody.outputs = 2;
        steps.prefix(&hold)?;
        hold.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        let cap = record.owner.clone();
        let task_owner = cap.clone();
        let task_sheet = sheet.id().clone();
        let cursor = HtmlStyleNotificationCursor::new(record.profile, true);
        let StyleState {
            completions,
            work,
            limits: runtime_limits,
            next_task,
            ..
        } = &mut *state;
        let steps = Steps {
            work,
            maximum: runtime_limits.work,
            cancel,
        };
        let (_, tail) = completions.split_at_mut(index);
        let (row, suffix) = tail.split_first_mut().expect("qualified receiving row");
        let Completion::ParsedProcessing(record) = row else {
            unreachable!("qualified actual processing")
        };
        let mut prep =
            hold.prepare_notification(&mut record.continuation, &record.owner_label, &cap, cancel)?;
        prep.step(cancel).map_err(DomStyleRuntimeError::from)?;
        steps.runtime()?;
        let removed = prep.owner_association_removed();
        let next = if removed {
            None
        } else {
            Some(
                next_task
                    .checked_add(1)
                    .ok_or(DomStyleRuntimeError::Admission)?,
            )
        };
        if removed {
            for _ in suffix {
                prep.step(cancel).map_err(DomStyleRuntimeError::from)?;
                steps.runtime()?;
            }
        }
        prep.admit_runtime(custody.usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        steps.runtime()?;
        let serial = *next_task;
        let mut published = prep.commit(cancel)?;
        let disposition = published
            .take_disposition()
            .expect("fresh consuming publication");
        let mut retired = None;
        let progress = match disposition {
            Disposition::Token(token) => {
                let task = Task {
                    serial,
                    root: cap.root(),
                    owner: task_owner,
                    sheet: task_sheet,
                    parsed: None,
                    cursor,
                    delivery: Delivery::NotStarted,
                    running: false,
                    dispatch_reserved: 0,
                    token,
                };
                let id = task.id();
                let old = std::mem::replace(&mut completions[index], Completion::Queued(task));
                let Completion::ParsedProcessing(record) = old else {
                    unreachable!("qualified original row")
                };
                completions[index].task_mut().parsed = Some(record);
                *next_task = next.expect("qualified token branch");
                DomStyleParsedNotificationProgress::Queued { task: id }
            }
            Disposition::OwnerAssociationRemoved { association } => {
                retired = Some(completions.remove(index));
                DomStyleParsedNotificationProgress::OwnerAssociationRemoved { association }
            }
        };
        drop(published);
        drop(retired);
        Ok(progress)
    }
}

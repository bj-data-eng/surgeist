//! Dialog state, the dialog task queue, close watchers and the dialog Source requests.
//!
//! `DomDialog` reads the retained return value and the stored close-watcher
//! (`DomCloseWatcherId`) and task associations; none grants current execution authority.
//! The task queue types (`DomDialogTask`, `DomDialogTaskQueuePlan`,
//! `DomDialogTaskQueueCommit`, `DomDialogTaskQueuePublication` and the `*TaskInfo`
//! values) describe the notification work a Dialog queues. `DomSourceDialogRequest`,
//! `DomSourceCloseWatcherRequest` and `DomSourceCloseWatcherStep` are the Source
//! continuation for the methods that reach event dispatch, focus and watcher services.
//!
//! The runtime owns the actual Window manager and its watcher objects; an ordinary atomic
//! edit that reaches a watcher service refuses with `NeedsEventfulOperation` before
//! publication. `docs/dialogs-and-focus.md` owns the contract.

pub use crate::internal::dialog::{DomCloseWatcherId, DomDialog, DomDialogToggleTracker};
pub use crate::internal::source::{
    DomCloseWatcherHistoryWindows, DomCloseWatcherRequestPreparation,
    DomCloseWatcherRequestPublication, DomDialogToggleTaskDisposition, DomDialogWatcherPreparation,
    DomDialogWatcherPublication, DomSourceCloseWatcherInvocationId, DomSourceCloseWatcherRequest,
    DomSourceCloseWatcherStep, DomSourceDialogRequest,
};
pub use crate::internal::store::source::operation::dialog_queue::{
    DomDialogCloseTaskInfo, DomDialogTask, DomDialogTaskInfo, DomDialogTaskQueueCommit,
    DomDialogTaskQueuePlan, DomDialogTaskQueuePublication, DomDialogToggleTaskInfo,
};

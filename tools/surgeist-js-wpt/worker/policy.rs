use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomError,
    edit::DomNodeRetirementLimits,
    event::DomDispatchCustodyLimits,
    hosted::{DomHostedLimits, DomHostedRecordLimits},
    limits::{DomLimits, DomNameLimits},
    source::DomSourceLimits,
};
use surgeist_runtime::{
    document_scripting::DocumentGuestEventLimits,
    element_task::DomElementTaskRuntimeLimits,
    event::{DomEventDispatchLimits, DomEventRegistryLimits, DomEventStorageLimits},
    hosted::DomHostedRuntimeLimits,
    session::DomSessionLimits,
    source::DomSourceEventRuntimeLimits,
    style::DomStyleRuntimeLimits,
};

use super::fixtures::Admission;

// These are fixed qualification limits for the worker's own native document.
// Engine heap configuration and process limits are independent.
const NATIVE_BYTES: usize = 16_000_000;
const NATIVE_WORK: u64 = 1_000_000;
pub(super) const SCRIPT_STORAGE_BYTES: usize = 1_000_000;

pub(super) fn dom() -> Result<Dom, DomError> {
    let names = DomNameLimits::new(NATIVE_BYTES, 16_384, 4_000_000, 1_048_576)?;
    let dispatch = DomDispatchCustodyLimits::new(64, 4096, NATIVE_BYTES, NATIVE_BYTES, 4_000_000)?;
    let limits = DomLimits::new(256, 200_000, 1024, 1024, 1024, 32, 4_000_000, names)?
        .with_ranges(8)?
        .with_dispatch_custody(dispatch);
    Dom::new(limits)
}

pub(super) fn session() -> DomSessionLimits {
    DomSessionLimits {
        authored_events: DomEventStorageLimits {
            storage_bytes: NATIVE_BYTES,
            work: NATIVE_WORK,
        },
        source_events: DomSourceEventRuntimeLimits {
            jobs: 0,
            storage_bytes: NATIVE_BYTES,
            work: NATIVE_WORK,
        },
        style: DomStyleRuntimeLimits {
            roots: 4,
            completions: 32,
            reports: 32,
            storage_bytes: NATIVE_BYTES,
            work: NATIVE_WORK,
            css_input: CssUtf16InputLimits::default(),
        },
        element_tasks: DomElementTaskRuntimeLimits {
            tasks: 32,
            storage_bytes: NATIVE_BYTES,
            work: NATIVE_WORK,
        },
        hosted: DomHostedRuntimeLimits {
            canonical: DomHostedLimits::new(
                DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                NATIVE_BYTES,
                NATIVE_BYTES,
                NATIVE_WORK,
            ),
            tasks: 32,
            storage_bytes: NATIVE_BYTES,
            scratch_bytes: NATIVE_BYTES,
            work: NATIVE_WORK,
        },
    }
}

pub(super) const REGISTRY: DomEventRegistryLimits = DomEventRegistryLimits {
    registrations: 16,
    pending_events: 4,
    nested_dispatches: 4,
    storage_bytes: 128_000,
    work: 100_000,
};

pub(super) fn guest(admission: Admission) -> DocumentGuestEventLimits {
    let source_work = match admission {
        Admission::NoSourceWork => 0,
        Admission::Standard | Admission::ScriptingDisabled => 20_000,
    };
    DocumentGuestEventLimits {
        storage_bytes: 512 * 1024,
        work: 10_000,
        pending_exception_reports: 16,
        node_retirement: DomNodeRetirementLimits::new(NATIVE_BYTES, 1_000),
        source_edits: DomSourceLimits::new(NATIVE_BYTES, 200_000, source_work, 64, 64, 64, 2),
        dispatch: DomEventDispatchLimits {
            path_items: 4,
            snapshot_listeners: 16,
            failure_records: 8,
            callbacks: 16,
            work: 10_000,
        },
    }
}

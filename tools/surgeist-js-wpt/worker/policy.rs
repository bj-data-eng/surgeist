use surgeist_css::source::CssUtf16InputLimits;
use surgeist_dom::{
    Dom, DomError,
    edit::DomNodeRetirementLimits,
    event::{DomDispatchCustodyLimits, DomEventRecipientLimits},
    hosted::{DomHostedLimits, DomHostedRecordLimits},
    limits::{DomLimits, DomNameLimits},
    source::DomSourceLimits,
};
use surgeist_html_parser::{HtmlParseLimits, HtmlParseLimitsError};
use surgeist_runtime::{
    document_scripting::{DocumentGuestEventLimits, WindowTimerLimits},
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
pub(super) const WINDOW_TIMERS: WindowTimerLimits = WindowTimerLimits::new(8, 4, 4096);
pub(super) const WINDOW_DISPATCH: DomEventDispatchLimits = DomEventDispatchLimits {
    path_items: 4,
    snapshot_listeners: 16,
    failure_records: 8,
    callbacks: 16,
    work: 10_000,
};

pub(super) fn html_parser() -> Result<HtmlParseLimits, HtmlParseLimitsError> {
    HtmlParseLimits::builder()
        .source_units(2048)
        .buffered_utf16_units(4096)
        .token_attributes(8)
        .auxiliary_entries(256)
        .managed_nodes(64)
        .managed_edges(64)
        .managed_attributes(16)
        .managed_utf16_units(4096)
        .source_map_entries(0)
        .source_map_payload_units(0)
        .diagnostics(0)
        .diagnostic_payload_units(0)
        .context_effects(0)
        .open_elements(16)
        .active_formatting_entries(0)
        .tree_depth(16)
        .managed_capacity_bytes(1_048_576)
        .peak_transient_bytes(1_048_576)
        .atomic_layout_bytes(65_536)
        // Scale the live parser's 128-unit qualification budget to this
        // worker's 2,048-unit page limit, including its inline script text.
        .work_credits(16 * 1_048_576)
        .finish()
}

pub(super) fn dom(local_html: bool) -> Result<Dom, DomError> {
    let names = DomNameLimits::new(NATIVE_BYTES, 16_384, 4_000_000, 1_048_576)?;
    let dispatch = DomDispatchCustodyLimits::new(64, 4096, NATIVE_BYTES, NATIVE_BYTES, 4_000_000)?;
    let limits = DomLimits::new(256, 200_000, 1024, 1024, 1024, 32, 4_000_000, names)?
        .with_ranges(8)?
        .with_dispatch_custody(dispatch);
    let limits = if local_html {
        // The page includes a charset meta element. The harness has two load
        // listeners and one error listener that retain native recipient leases.
        let recipients = DomEventRecipientLimits::new(3, NATIVE_BYTES, NATIVE_BYTES, 100_000)?;
        limits
            .with_meta_processing_work(100_000)
            .with_event_recipients(recipients)
    } else {
        limits
    };
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
        // Include all currently installed Window interfaces, as in the
        // attached Runtime behavior fixtures.
        storage_bytes: 640 * 1024,
        work: 10_000,
        pending_exception_reports: 16,
        node_retirement: DomNodeRetirementLimits::new(NATIVE_BYTES, 1_000),
        source_edits: DomSourceLimits::new(NATIVE_BYTES, 200_000, source_work, 64, 64, 64, 2),
        dispatch: WINDOW_DISPATCH,
    }
}

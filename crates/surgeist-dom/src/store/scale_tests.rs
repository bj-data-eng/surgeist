//! Explicit #1263 qualification fixtures with publicly populated canonical stores.
//! The exhausted-generation frontier uses a disclosed vacant-counter seed only;
//! actual MAX-generation creation and retirement still execute public edits.
//!
//! The owner declares this module under `cfg(test)` after Source integration.
//! Select ONE ignored test in a release build with `--test-threads=1 --nocapture`.
//! Never run these fixtures against the old global Draft implementation.
//! Set SURGEIST_SCALE_REVISION, SURGEIST_SCALE_TOOLCHAIN and
//! SURGEIST_SCALE_MACHINE from the qualified runner; unknown values are printed
//! explicitly. Timing has no pass/fail threshold. Private execution counters
//! must supplement these public behavior/work-limit measurements before final
//! scale acceptance; this harness does not manufacture allocator observations.
//! Sequence bytes cover actual initialized, moved or copied sequence values.
//! Fixed in-place scalar/field overwrites are outside that measurement contract.

use std::hint::black_box;
use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use super::scale_metrics::{self, Domain, Hooks, Measurement};
use crate::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomDispatchCustody,
    DomDispatchCustodyLimits, DomError, DomLimits, DomSnapshot, DomSourceEdit, DomSourceLimits,
    DomSourceOutput, DomSourceProgress, DomSourceRequest, DomStyleCssomOwnerBinding,
    DomStyleCssomRoot, DomStyleInlineCspObservation, Namespace, NodeId,
};

mod memory;

const SIZES: [usize; 3] = [10_000, 100_000, 1_000_000];
const CHUNK: usize = 8192;
const WARMUPS: usize = 1;
const SAMPLES: usize = 3;
const LARGE_CHILDREN: usize = 65_536;
const SOURCE_WORK: u64 = 20_000;

#[expect(
    clippy::assertions_on_constants,
    reason = "Keep ignored qualification cases compilable in debug while preserving their runtime release-only guard"
)]
fn require_release() {
    // These ignored cases must compile in normal debug test binaries and
    // refuse execution only when the caller explicitly selects them there.
    assert!(
        !cfg!(debug_assertions),
        "select this fixture in a release build"
    );
}

/// A finite pilot selection is explicit in the receipt; the default retains the
/// complete geometric qualification through one million nodes. A 1k pilot
/// measures actual host resource use and never establishes scale acceptance.
fn selected_sizes() -> Vec<usize> {
    let Some(value) = std::env::var_os("SURGEIST_SCALE_SIZES") else {
        return SIZES.to_vec();
    };
    let mut sizes = Vec::new();
    for item in value.to_str().expect("UTF-8 size selection").split(',') {
        let size: usize = item.parse().expect("comma-separated managed-node counts");
        assert!(
            size == 1000 || SIZES.contains(&size),
            "unsupported size {size}"
        );
        assert!(!sizes.contains(&size), "duplicate size {size}");
        sizes.push(size);
    }
    assert!(!sizes.is_empty());
    eprintln!("scale explicit_selected_sizes={sizes:?} full_default={SIZES:?}");
    sizes
}

// Default finite fixture allowance for the one-million-node stress ceiling plus
// selected-operation headroom; it is no measured-work oracle or admission-threshold
// assertion. The local SourceAttribute case also allows the existing owner's
// cached usage. Actual usage and refusal remain production-owned.
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(
        usize::try_from(32_000_000_000u64).expect("scale qualification requires a 64-bit host"),
        512_000_000,
        SOURCE_WORK,
        8,
        16,
        16,
        1,
    )
}

#[derive(Clone, Copy, Debug)]
enum Context {
    Detached,
    Attached,
    Vacant,
    Attributes,
    LargeChildren,
    RangeBoundary,
    Ranges,
    DetailsTasks,
    SelectTasks,
    DispatchCustody,
    FullscreenModel,
    FullscreenInvocations,
    StyleRoots,
}

struct Fixture {
    dom: Dom,
    target: NodeId,
    text: NodeId,
    other_document: DocumentId,
    children: Vec<NodeId>,
    managed: usize,
    retired: usize,
    peak_managed: usize,
    ranges: usize,
    details_intentions: usize,
    select_tasks: usize,
    dispatch: Option<DomDispatchCustody>,
    fullscreen_members: Vec<NodeId>,
    fullscreen_jobs: Vec<crate::DomFullscreenExitJob>,
    fullscreen_source: Option<crate::DomHostedSource>,
    style_documents: Vec<DocumentId>,
    setup: Duration,
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

/// Install a real native Style and its actual CSSOM producer result. The test
/// service's complete applicable CSP policy list is explicitly empty. Parsing
/// and CSSOM creation execute; no snapshot is presented as a commit receipt.
fn create_style_root(dom: &mut Dom, ordinal: usize) -> DocumentId {
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{
        CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs, CssomStore,
    };
    let cancel = AtomicBool::new(false);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let style = batch.create_element_ns(&document, Namespace::html(), "style");
    let text = batch.create_text(&document, "p {}");
    batch.insert_before(&text, &style, None);
    let commit = apply(dom, batch);
    let document = commit.document(&document).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let allowances = source_limits();
    let setup_limits = DomSourceLimits::new(
        allowances.retained_bytes(),
        allowances.retained_string_code_units(),
        100_000_000,
        64,
        4096,
        4096,
        1,
    );
    let mut lease = dom
        .begin_source_edit(
            DomSourceEdit::insert_before(style, document.node(), None),
            setup_limits,
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual Style policy observation");
    };
    lease
        .accept_inline_csp(
            &policy,
            &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            &cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual CSSOM creation request");
    };
    let root = DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    lease.bind_cssom_root(&create, &root, &cancel).unwrap();
    let create = lease
        .qualify_cssom(&create, document.node(), &root.snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: format!("scale-style-{ordinal}"),
        revision: 1,
    };
    lease
        .bind_cssom_owner(
            &create,
            DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline".into()).unwrap(),
            &cancel,
        )
        .unwrap();
    let parser = CssParserContext::default();
    let report = parser
        .parse_sheet_utf16_with_limits(
            create.source().text().clone(),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            Arc::new(Vec::new()),
            CssUtf16InputLimits::default(),
            || Ok::<_, ()>(()),
        )
        .unwrap();
    let mut inputs = CssomSheetInputs::external(version, true);
    inputs.owner = Some("inline".into());
    let mut batch = root
        .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
        .unwrap();
    let construction = batch
        .create_parsed_sheet_with_media_reports(report, media, inputs)
        .unwrap();
    let commit = root.commit(&mut Some(batch)).unwrap();
    let _notification = lease
        .accept_cssom_publication(
            &create,
            commit,
            root.snapshot().unwrap(),
            Some(construction.ticket()),
        )
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
    drop(lease);
    assert!(dom.style_cssom_root(document.node()).unwrap().is_some());
    document
}

fn fixture(managed: usize, context: Context) -> Fixture {
    fixture_with_capacity(managed, context, 0)
}

fn fixture_with_capacity(managed: usize, context: Context, extra: usize) -> Fixture {
    let started = Instant::now();
    let retired = if matches!(context, Context::Vacant) {
        managed / 4
    } else {
        0
    };
    let node_limit = managed.checked_add(retired).unwrap();
    let capacity = node_limit.checked_add(extra).unwrap();
    let unit_limit = capacity.checked_mul(128).unwrap();
    let ranges = if matches!(context, Context::Ranges) {
        (managed / 8).min(65_536)
    } else {
        0
    };
    let details = if matches!(context, Context::DetailsTasks) {
        (managed / 16).min(32_768)
    } else {
        0
    };
    let select_tasks = if matches!(context, Context::SelectTasks) {
        (managed / 32).min(32_768)
    } else {
        0
    };
    let dispatch_targets = if matches!(context, Context::DispatchCustody) {
        managed / 8
    } else {
        0
    };
    let fullscreen_count = if matches!(context, Context::FullscreenModel) {
        (managed / 64).min(4096)
    } else {
        0
    };
    let style_count = if matches!(context, Context::StyleRoots) {
        (managed / 2048).clamp(8, 1024)
    } else {
        0
    };
    let invocation_count = if matches!(context, Context::FullscreenInvocations) {
        (managed / 2048).clamp(4, 1024)
    } else {
        0
    };
    let limits = DomLimits::new(
        capacity,
        unit_limit,
        CHUNK * 4,
        0,
        0,
        32,
        4_000_000,
        crate::DomNameLimits::new(1_073_741_824, 2_000_000, 64_000_000, 268_435_456).unwrap(),
    )
    .unwrap()
    .with_ranges(ranges)
    .unwrap()
    .with_details_toggle_intentions(details * 2)
    .unwrap()
    .with_select_processing(select_tasks, 16_000_000, 10_000_000)
    .unwrap()
    .with_dispatch_custody(if dispatch_targets == 0 {
        DomDispatchCustodyLimits::default()
    } else {
        super::dispatch::scale_fresh_limits(dispatch_targets)
    });
    let (mut dom, fullscreen_source, fullscreen_document) = if invocation_count > 0 {
        // Use the original DOM issuer boundary, including native hosted
        // permissions/history. This supplies no Runtime or orientation service.
        let (dom, source, document, _) = super::source::operation::fullscreen::native_fixture(
            limits
                .with_fullscreen_exits(invocation_count)
                .unwrap()
                .with_fullscreen_events(invocation_count + 4)
                .unwrap(),
        );
        (dom, Some(source), Some(document))
    } else {
        (Dom::new(limits).unwrap(), None, None)
    };
    let hosted_nodes = dom.usage().managed_nodes();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let target = batch.create_element(&document, "target");
    let text = batch.create_text(&document, "base");
    batch.set_attribute(&target, "data-local", "base");
    batch.insert_before(&target, &document, None);
    batch.insert_before(&text, &target, None);
    let other_document = batch.create_document();
    let other_parent = batch.create_element(&other_document, "other");
    batch.insert_before(&other_parent, &other_document, None);
    let commit = apply(&mut dom, batch);
    let document = commit.document(&document).unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let text = commit.created(&text).unwrap().unwrap();
    let other_document = commit.document(&other_document).unwrap();
    let other_parent = commit.created(&other_parent).unwrap().unwrap();
    let mut children = vec![text];
    let style_documents: Vec<_> = (0..style_count)
        .map(|ordinal| create_style_root(&mut dom, ordinal))
        .collect();
    let mut fullscreen_jobs = Vec::new();
    if let (Some(source), Some(document)) = (&fullscreen_source, fullscreen_document) {
        let cancel = AtomicBool::new(false);
        let policy = source_limits();
        let setup_limits = DomSourceLimits::new(
            policy.retained_bytes(),
            policy.retained_string_code_units(),
            100_000_000,
            64,
            4096,
            4096,
            4,
        );
        for _ in 0..invocation_count {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::exit_fullscreen(document),
                    setup_limits,
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::FullscreenExitHandoff(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("original native Fullscreen handoff");
            };
            let job = lease
                .prepare_fullscreen_exit_handoff(&request, source, &cancel)
                .unwrap()
                .publish();
            let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                panic!("original synchronous exit tail completes");
            };
            assert_eq!(
                done.fullscreen_exit_return(),
                Some(crate::DomFullscreenExitReturn::Pending(job.id()))
            );
            fullscreen_jobs.push(job);
        }
    }
    let large_children = match context {
        Context::LargeChildren => LARGE_CHILDREN,
        // The boundary companion also admits the 10k selection. Its two
        // endpoint Comments occupy the tail of this genuine public sequence.
        Context::RangeBoundary => LARGE_CHILDREN.min(managed.checked_sub(5).unwrap()),
        _ => 0,
    };
    assert!(managed >= 5 + large_children);

    // Attached fixtures use bounded groups. Only the explicitly selected wide
    // Children and Range-boundary fixtures grow a large sequence during setup.
    let mut remaining = managed - 5 - style_count * 3 - hosted_nodes;
    let mut target_remaining = large_children;
    let mut details_remaining = details;
    let mut select_remaining = select_tasks;
    let mut select_ids = Vec::new();
    let mut dispatch_ids = Vec::new();
    let mut fullscreen_remaining = fullscreen_count;
    let mut fullscreen_members = Vec::new();
    while remaining > 0 {
        let count = remaining.min(CHUNK);
        let mut batch = DomBatch::new();
        let mut target_tickets = Vec::new();
        let mut select_tickets = Vec::new();
        let mut dispatch_tickets = Vec::new();
        let mut fullscreen_tickets = Vec::new();
        let mut left = count;
        while left > 0 {
            if matches!(context, Context::Attached) {
                let group = batch.create_element(other_document, "group");
                batch.insert_before(&group, other_parent, None);
                left -= 1;
                let leaves = left.min(63);
                for _ in 0..leaves {
                    let leaf = batch.create_comment(other_document, "u");
                    batch.insert_before(&leaf, &group, None);
                }
                left -= leaves;
            } else if target_remaining > 0 {
                let child = batch.create_comment(document, "child");
                batch.insert_before(&child, target, None);
                target_tickets.push(child);
                target_remaining -= 1;
                left -= 1;
            } else if details_remaining > 0 {
                let details = batch.create_element_ns(other_document, Namespace::html(), "details");
                // Two genuine source transitions leave equal authored state
                // and retain both ordered native task intentions.
                batch.set_attribute(&details, "open", "");
                batch.remove_attribute(&details, "open");
                details_remaining -= 1;
                left -= 1;
            } else if select_remaining > 0 && left >= 2 {
                let select = batch.create_element_ns(other_document, Namespace::html(), "select");
                let option = batch.create_element_ns(other_document, Namespace::html(), "option");
                batch.set_attribute(&option, "selected", "");
                batch.insert_before(&option, &select, None);
                // Removing an actual selected Option queues the original
                // Select update; retain it without checkpoint completion.
                batch.detach(&option);
                select_tickets.push(select);
                select_remaining -= 1;
                left -= 2;
            } else if fullscreen_remaining > 0 {
                let element = batch.create_element(other_document, "fullscreen-member");
                fullscreen_tickets.push(element);
                fullscreen_remaining -= 1;
                left -= 1;
            } else if matches!(context, Context::Attributes) {
                let extra = batch.create_element(other_document, "extra");
                batch.set_attribute(&extra, "data-unrelated", "retained-value");
                batch.set_attribute(&extra, "other", "independent-value");
                left -= 1;
            } else {
                let ticket = batch.create_comment(other_document, "u");
                if dispatch_ids.len() + dispatch_tickets.len() < dispatch_targets {
                    dispatch_tickets.push(ticket);
                }
                left -= 1;
            }
        }
        let commit = apply(&mut dom, batch);
        children.extend(
            target_tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap()),
        );
        select_ids.extend(
            select_tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap()),
        );
        dispatch_ids.extend(
            dispatch_tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap()),
        );
        fullscreen_members.extend(
            fullscreen_tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap()),
        );
        remaining -= count;
    }

    assert_eq!(select_ids.len(), select_tasks);
    assert_eq!(dom.state.counts.select_tasks, select_tasks);
    // Independently enumerate actual retained queue targets during setup, so a
    // cached count cannot stand in for a genuinely populated task bank.
    let queued: Vec<_> = dom
        .state
        .native_tasks
        .iter()
        .filter_map(|task| match task {
            super::select::Task::Update { target, .. } => Some(*target),
            super::select::Task::Slots => None,
        })
        .collect();
    assert_eq!(queued, select_ids);
    drop(queued);
    if let Some(first) = select_ids.first() {
        let checkpoint = crate::DomNativeCheckpointLimits::new(
            16_000_000,
            10_000_000,
            crate::DomSlotQueryLimits::new(0, 0).unwrap(),
        )
        .unwrap();
        let crate::DomNativeCheckpoint::Select(task) = dom
            .native_checkpoint_step(checkpoint, &AtomicBool::new(false))
            .unwrap()
        else {
            panic!("actual queued Select update");
        };
        assert_eq!(task.select(), *first);
        // Permission drop leaves the original queued work parked.
        drop(task);
    }
    drop(select_ids);
    let mut remaining_ranges = ranges;
    while remaining_ranges > 0 {
        let count = remaining_ranges.min(CHUNK);
        let mut batch = DomBatch::new();
        for _ in 0..count {
            batch.create_range(other_document);
        }
        apply(&mut dom, batch);
        remaining_ranges -= count;
    }
    let dispatch = if dispatch_ids.is_empty() {
        None
    } else {
        let verification = super::dispatch::ScaleSetupVerification::begin(&dom, &dispatch_ids);
        let custody = dom
            .register_dispatch_custody(&dispatch_ids, &AtomicBool::new(false))
            .unwrap();
        verification.finish();
        Some(custody)
    };
    drop(dispatch_ids);
    for ids in fullscreen_members.chunks(CHUNK) {
        let mut batch = DomBatch::new();
        for id in ids {
            batch.fullscreen_element_model(*id);
        }
        apply(&mut dom, batch);
    }

    // Publish every identity before retiring any; otherwise each chunk would
    // reuse the previous chunk's vacancies and fail to vary allocated extent.
    let mut retire_ids = Vec::new();
    let mut remaining = retired;
    while remaining > 0 {
        let count = remaining.min(CHUNK);
        let mut batch = DomBatch::new();
        let tickets: Vec<_> = (0..count)
            .map(|_| batch.create_comment(other_document, "retire"))
            .collect();
        let commit = apply(&mut dom, batch);
        retire_ids.extend(
            tickets
                .iter()
                .map(|ticket| commit.created(ticket).unwrap().unwrap()),
        );
        remaining -= count;
    }
    assert_eq!(dom.usage().managed_nodes(), node_limit);
    for ids in retire_ids.chunks(CHUNK) {
        let mut batch = DomBatch::new();
        for id in ids {
            batch.destroy(*id);
        }
        apply(&mut dom, batch);
    }
    drop(retire_ids);
    assert_eq!(dom.usage().managed_nodes(), managed);
    let setup = started.elapsed();
    // Deliberate full enumeration verifies the public fixture outside all edit
    // timing intervals. No private canonical state is manufactured.
    assert_eq!(dom.snapshot().nodes().count(), managed);
    Fixture {
        dom,
        target,
        text,
        other_document,
        children,
        managed,
        retired,
        peak_managed: node_limit,
        ranges,
        details_intentions: details * 2,
        select_tasks,
        dispatch,
        fullscreen_members,
        fullscreen_jobs,
        fullscreen_source,
        style_documents,
        setup,
    }
}

#[derive(Clone, Copy, Debug)]
enum Edit {
    Empty,
    SameAttribute,
    RestoredAttribute,
    ChangedAttribute,
    RestoredText,
    ChangedText,
    SourceAttribute,
}

fn attribute(snapshot: &DomSnapshot, id: NodeId) -> DocumentString {
    snapshot
        .node(id)
        .unwrap()
        .as_element()
        .unwrap()
        .attribute(&"data-local".into())
        .unwrap()
        .clone()
}

fn measure_edit(fixture: &mut Fixture, edit: Edit, sample: usize) -> Duration {
    let before = fixture.dom.snapshot();
    let usage = fixture.dom.usage();
    let old_attribute = attribute(&before, fixture.target);
    let old_text = before.node(fixture.text).unwrap().data().unwrap().clone();
    let next = if sample.is_multiple_of(2) {
        "next"
    } else {
        "base"
    };
    let mut batch = DomBatch::new();
    let (noop, expected) = match edit {
        Edit::Empty => (true, None),
        Edit::SameAttribute => {
            batch.set_attribute(fixture.target, "data-local", old_attribute.clone());
            (true, None)
        }
        Edit::RestoredAttribute => {
            batch.set_attribute(fixture.target, "data-local", "temporary");
            batch.set_attribute(fixture.target, "data-local", old_attribute.clone());
            (true, None)
        }
        Edit::ChangedAttribute => {
            batch.set_attribute(fixture.target, "data-local", next);
            (false, Some(DomChange::Attributes(fixture.target)))
        }
        Edit::RestoredText => {
            batch.replace_data(fixture.text, 0, usize::MAX, "temporary");
            batch.replace_data(fixture.text, 0, usize::MAX, old_text.clone());
            (true, None)
        }
        Edit::ChangedText => {
            batch.replace_data(fixture.text, 0, usize::MAX, next);
            (false, Some(DomChange::Data(fixture.text)))
        }
        Edit::SourceAttribute => (false, Some(DomChange::Attributes(fixture.target))),
    };
    // This local edit has no disjoint input owner. Read the actual already
    // charged baseline in O(1), outside measurement, and retain the unchanged
    // finite 32 GB allowance as additional selected-operation headroom.
    let local_source_limits = matches!(edit, Edit::SourceAttribute).then(|| {
        let baseline = fixture.dom.custody_registry.total().bytes();
        let headroom = source_limits();
        let absolute_bytes = baseline
            .checked_add(headroom.retained_bytes())
            .expect("local Source fixture policy fits the host word");
        let limits = DomSourceLimits::new(
            absolute_bytes,
            headroom.retained_string_code_units(),
            headroom.work(),
            headroom.phase_publications(),
            headroom.output_records(),
            headroom.published_native_identities(),
            headroom.nested_operations(),
        );
        eprintln!(
            "scale local_source_policy baseline_bytes={baseline} additional_headroom_bytes={} absolute_bytes={absolute_bytes} limits={limits:?}",
            headroom.retained_bytes()
        );
        limits
    });
    let measurement = if scale_metrics::ENABLED {
        Some(Measurement::begin().unwrap())
    } else {
        None
    };
    let started = Instant::now();
    let (elapsed, changes) = if matches!(edit, Edit::SourceAttribute) {
        let limits = local_source_limits.expect("selected local Source policy");
        let cancel = AtomicBool::new(false);
        let mut lease = fixture
            .dom
            .begin_source_edit(
                DomSourceEdit::set_attribute(fixture.target, "data-local", next),
                limits,
                &cancel,
            )
            .unwrap();
        let initial_work = lease.status().spent_work();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
            panic!("this plain XML attribute edit requires no external service");
        };
        let elapsed = started.elapsed();
        let changes = done.commit().changes().changes().to_vec();
        // This observable count precedes final advance. Do not report it as
        // total Source work: success additionally proves the finite ceiling.
        eprintln!("scale source_initial_work={initial_work} source_limits={limits:?}");
        (elapsed, changes)
    } else {
        let commit = apply(&mut fixture.dom, batch);
        let elapsed = started.elapsed();
        (elapsed, commit.changes().changes().to_vec())
    };
    if let Some(measurement) = measurement {
        let mut required = Hooks::one(Domain::Transaction);
        if !matches!(edit, Edit::Empty) {
            required = required.union(Hooks::one(Domain::Bank));
        }
        if !noop {
            required = required
                .union(Hooks::one(Domain::Validation))
                .union(Hooks::one(Domain::Accounting));
        }
        if matches!(edit, Edit::SourceAttribute) {
            required = required.union(Hooks::one(Domain::Source));
        }
        let counters = measurement.finish(required).unwrap();
        // Selected qualification allowances for this fixed local payload and
        // neighborhood, independent of implementation output. Index work may
        // grow with depth; reading/rebuilding unrelated records or sequences
        // may not grow with fixture extent. These are structural assertions,
        // never wall-clock thresholds or inferred touched-set counts.
        let binary_depth = usize::BITS as usize - fixture.peak_managed.leading_zeros() as usize;
        assert!(counters.canonical_record_reads <= 512, "{counters:?}");
        assert!(counters.validation_record_visits <= 512, "{counters:?}");
        assert!(counters.accounting_record_visits <= 512, "{counters:?}");
        let queue_admin = counters
            .queue_record_visits
            .checked_add(counters.queue_records_moved)
            .expect("measured queue work");
        assert!(queue_admin <= 512, "{counters:?}");
        assert!(counters.record_materializations <= 64, "{counters:?}");
        // This single XML attribute operation has fixed preflight/rollback
        // journal administration in addition to its selected field/output.
        // Its allowance has no S or k term and still rejects an unrelated
        // 65,536-child NodeId sequence. Direct edits keep the smaller bound.
        let sequence_allowance = if matches!(edit, Edit::SourceAttribute) {
            64 * 1024
        } else {
            8192
        };
        assert!(
            counters.sequence_materialized_bytes <= sequence_allowance,
            "{counters:?}"
        );
        assert!(
            counters.structural_bytes <= 131_072 * binary_depth,
            "{counters:?}"
        );
        assert!(
            counters.bank_page_visits <= 512 * binary_depth,
            "{counters:?}"
        );
        assert_eq!(counters.published_roots, usize::from(!noop), "{counters:?}");
        if !matches!(edit, Edit::Empty) {
            assert!(counters.bank_page_visits > 0, "{counters:?}");
        }
        if matches!(edit, Edit::SourceAttribute) {
            assert!(
                counters.source_steps > 0 && counters.source_steps <= SOURCE_WORK as usize,
                "{counters:?}"
            );
            assert!(
                counters.savepoints > 0 && counters.savepoint_peak_bytes > 0,
                "{counters:?}"
            );
            assert!(
                counters.source_peak_bytes > 0 && counters.source_peak_units > 0,
                "{counters:?}"
            );
        }
        eprintln!("scale execution edit={edit:?} sample={sample} counters={counters:?}");
    }
    let now = fixture.dom.snapshot();
    if noop {
        assert!(changes.is_empty());
        assert_eq!(now.stamp(), before.stamp());
        assert_eq!(fixture.dom.usage(), usage);
        assert_eq!(attribute(&now, fixture.target), old_attribute);
        assert_eq!(now.node(fixture.text).unwrap().data(), Some(&old_text));
    } else {
        assert_eq!(changes, [expected.unwrap()]);
        assert_eq!(now.revision().get(), before.revision().get() + 1);
        if matches!(edit, Edit::ChangedText) {
            assert_eq!(
                now.node(fixture.text).unwrap().data(),
                Some(&DocumentString::from(next))
            );
        } else {
            assert_eq!(attribute(&now, fixture.target), DocumentString::from(next));
        }
    }
    assert_eq!(
        now.node(fixture.target).unwrap().children(),
        fixture.children
    );
    assert_eq!(
        before.node(fixture.target).unwrap().children(),
        fixture.children
    );
    assert_eq!(before.node(fixture.text).unwrap().data(), Some(&old_text));
    assert_eq!(fixture.dom.usage().managed_nodes(), fixture.managed);
    assert_eq!(fixture.dom.usage().ranges(), fixture.ranges);
    assert_eq!(
        fixture.dom.pending_details_toggle_intentions(),
        fixture.details_intentions
    );
    assert_eq!(fixture.dom.state.counts.select_tasks, fixture.select_tasks);
    if let Some(dispatch) = &fixture.dispatch {
        assert!(dispatch.targets() > 0);
    }
    if !fixture.fullscreen_members.is_empty() {
        assert_eq!(
            now.document(fixture.other_document)
                .unwrap()
                .top_layer()
                .unwrap()
                .raw_members(),
            fixture.fullscreen_members
        );
    }
    for document in &fixture.style_documents {
        assert!(
            fixture
                .dom
                .style_cssom_root(document.node())
                .unwrap()
                .is_some()
        );
    }
    assert_eq!(
        now.document(fixture.other_document).unwrap().id(),
        fixture.other_document
    );
    elapsed
}

fn report(label: &str, samples: &[Duration]) {
    assert_eq!(samples.len(), SAMPLES);
    let mut samples = samples.to_vec();
    samples.sort_unstable();
    // Report this finite sample directly; the maximum is not a tail percentile.
    eprintln!(
        "scale measurement={label} samples={} median_ns={} observed_maximum_of_three_ns={}",
        samples.len(),
        samples[samples.len() / 2].as_nanos(),
        samples.last().unwrap().as_nanos()
    );
}

fn run(context: Context) {
    require_release();
    eprintln!(
        "scale environment profile=release debug_assertions={} os={} arch={} revision={} toolchain={} machine={} counters_enabled={} counters_build_value={:?} warmups={WARMUPS} samples={SAMPLES} chunk={CHUNK} source_storage_configuration_bytes=32000000000 storage_configuration_not_oracle=true",
        cfg!(debug_assertions),
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into()),
        scale_metrics::ENABLED,
        option_env!("SURGEIST_DOM_STORAGE_COUNTERS")
    );
    for managed in selected_sizes() {
        if matches!(context, Context::LargeChildren) && managed < LARGE_CHILDREN + 5 {
            continue;
        }
        let mut fixture = fixture(managed, context);
        eprintln!(
            "scale fixture context={context:?} managed={} retired={} peak_managed={} target_children={} setup_ns={} limits={:?} usage={:?}",
            fixture.managed,
            fixture.retired,
            fixture.peak_managed,
            fixture.children.len(),
            fixture.setup.as_nanos(),
            fixture.dom.limits(),
            fixture.dom.usage()
        );
        eprintln!("scale retained_select_tasks={}", fixture.select_tasks);
        let original = fixture.dom.snapshot();
        eprintln!(
            "scale retained ranges={} details_intentions={} dispatch_targets={} dispatch_producer_bytes={} fullscreen_model_members={} active_fullscreen_invocations={}",
            fixture.ranges,
            fixture.details_intentions,
            fixture
                .dispatch
                .as_ref()
                .map_or(0, DomDispatchCustody::targets),
            fixture
                .dispatch
                .as_ref()
                .map_or(0, DomDispatchCustody::retained_bytes),
            fixture.fullscreen_members.len(),
            fixture.fullscreen_jobs.len()
        );
        eprintln!(
            "scale retained_live_style_roots={} retired_style_roots=0",
            fixture.style_documents.len()
        );
        let mut snapshot_times = Vec::new();
        let mut snapshots = Vec::new();
        for sample in 0..WARMUPS + SAMPLES {
            let started = Instant::now();
            let snapshot = black_box(fixture.dom.snapshot());
            let elapsed = started.elapsed();
            if sample >= WARMUPS {
                snapshot_times.push(elapsed);
            }
            snapshots.push(snapshot);
        }
        report("snapshot_acquire", &snapshot_times);
        let mut snapshot_drops = Vec::new();
        for (sample, snapshot) in snapshots.into_iter().enumerate() {
            let started = Instant::now();
            drop(black_box(snapshot));
            let elapsed = started.elapsed();
            if sample >= WARMUPS {
                snapshot_drops.push(elapsed);
            }
        }
        report("snapshot_drop_shared", &snapshot_drops);
        for edit in [
            Edit::Empty,
            Edit::SameAttribute,
            Edit::RestoredAttribute,
            Edit::ChangedAttribute,
            Edit::RestoredText,
            Edit::ChangedText,
            Edit::SourceAttribute,
        ] {
            // Each edit sequence starts with literal base values. This setup
            // is excluded from mutation samples and prevents cross-case state
            // from turning the first changed edit into an accidental no-op.
            let mut reset = DomBatch::new();
            reset.set_attribute(fixture.target, "data-local", "base");
            reset.replace_data(fixture.text, 0, usize::MAX, "base");
            apply(&mut fixture.dom, reset);
            let mut samples = Vec::new();
            for sample in 0..WARMUPS + SAMPLES {
                let elapsed = measure_edit(&mut fixture, edit, sample);
                if sample >= WARMUPS {
                    samples.push(elapsed);
                }
            }
            report(&format!("{edit:?}"), &samples);
        }
        assert_eq!(
            original.node(fixture.target).unwrap().children(),
            fixture.children
        );
        assert_eq!(
            attribute(&original, fixture.target),
            DocumentString::from("base")
        );
        assert_eq!(
            original.node(fixture.text).unwrap().data(),
            Some(&DocumentString::from("base"))
        );
        for job in &fixture.fullscreen_jobs {
            assert_eq!(
                fixture.dom.fullscreen_exit_status(job.id()).unwrap().id(),
                job.id()
            );
        }
        eprintln!("scale final_usage={:?}", fixture.dom.usage());
        let Fixture {
            dom,
            children,
            dispatch,
            fullscreen_members,
            fullscreen_jobs,
            fullscreen_source,
            ..
        } = fixture;
        drop(children);
        drop(fullscreen_members);
        drop(fullscreen_jobs);
        let started = Instant::now();
        drop(dom);
        eprintln!(
            "scale owner_drop_held_snapshot_ns={}",
            started.elapsed().as_nanos()
        );
        drop(fullscreen_source);
        let started = Instant::now();
        drop(dispatch);
        eprintln!(
            "scale dispatch_custody_drop_after_owner_ns={}",
            started.elapsed().as_nanos()
        );
        let started = Instant::now();
        drop(original);
        eprintln!(
            "scale snapshot_drop_last_ns={}",
            started.elapsed().as_nanos()
        );
    }
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn detached_other_document_scale() {
    run(Context::Detached);
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn attached_other_document_scale() {
    run(Context::Attached);
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn historical_vacant_slots_scale() {
    run(Context::Vacant);
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn unrelated_attribute_backing_scale() {
    run(Context::Attributes);
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn large_unchanged_child_list_scale() {
    run(Context::LargeChildren);
}

#[test]
#[ignore = "explicit release retained Range-bank qualification after Source integration"]
fn unrelated_ranges_scale() {
    run(Context::Ranges);
}

fn scale_range_bounds(
    snapshot: &DomSnapshot,
    id: crate::DomRangeId,
) -> ((NodeId, usize), (NodeId, usize)) {
    let range = snapshot.range(id).unwrap();
    (
        (range.start().container(), range.start().offset()),
        (range.end().container(), range.end().offset()),
    )
}

#[derive(Clone, Copy, Debug)]
enum RangeBoundaryEdit {
    StartNoop,
    EndNoop,
    StartChanged,
    EndChanged,
}

#[test]
#[ignore = "explicit release Source Range-boundary qualification beside an unchanged sibling prefix"]
fn source_range_boundaries_beside_unchanged_siblings_scale() {
    require_release();
    // The genuine public regression in tests/range_position_locality.rs uses
    // this same finite work policy. Here even one old tail-prefix enumeration
    // exceeds it: the smallest default fixture has 9,994 preceding siblings.
    // This is a work/admission assertion, never a latency threshold. The fixed
    // endpoints, two ordinary ancestor edges and one Range do not grow with S.
    const WORK: u64 = 4096;
    eprintln!(
        "scale range_boundary environment profile=release os={} arch={} counters_enabled={} warmups={WARMUPS} samples={SAMPLES} work={WORK} revision={} toolchain={} machine={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        scale_metrics::ENABLED,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into()),
    );
    let cancel = AtomicBool::new(false);
    for managed in selected_sizes() {
        // A 1k pilot cannot discriminate the old scan with this unchanged work
        // policy; disclose that ineligibility instead of claiming scale credit.
        let comments = LARGE_CHILDREN.min(managed.checked_sub(5).unwrap());
        if comments < usize::try_from(WORK).unwrap() + 2 {
            eprintln!(
                "scale range_boundary skipped managed={managed} tail_prefix_below_fixed_work=true comments={comments} work={WORK}"
            );
            continue;
        }
        let setup_started = Instant::now();
        let mut fixture = fixture(managed, Context::RangeBoundary);
        let document = fixture
            .dom
            .snapshot()
            .node(fixture.target)
            .unwrap()
            .owner_document()
            .unwrap();
        fixture
            .dom
            .set_limits(fixture.dom.limits().with_ranges(1).unwrap())
            .unwrap();
        let a = fixture.children[fixture.children.len() - 2];
        let b = fixture.children[fixture.children.len() - 1];
        let prefix = fixture.children.len() - 2;
        assert!(prefix > usize::try_from(WORK).unwrap());
        let old_two_prefix_steps = prefix.checked_mul(2).unwrap().checked_add(3).unwrap();
        assert!(old_two_prefix_steps > usize::try_from(WORK).unwrap());
        let mut setup = DomBatch::new();
        let range = setup.create_range(document);
        setup.set_range_start(&range, a, 0);
        setup.set_range_end(&range, b, 1);
        setup.set_active_range(document, Some((&range).into()));
        let commit = apply(&mut fixture.dom, setup);
        let range = commit.created_range(&range).unwrap().unwrap();
        let original = fixture.dom.snapshot();
        assert_eq!(scale_range_bounds(&original, range), ((a, 0), (b, 1)));
        assert_eq!(original.node(a).unwrap().parent(), Some(fixture.target));
        assert_eq!(original.node(b).unwrap().parent(), Some(fixture.target));
        assert_eq!(
            original.node(fixture.target).unwrap().parent(),
            Some(document.node())
        );
        assert_eq!(
            original.document(document).unwrap().active_range(),
            crate::DomActiveRangeState::Known(Some(range))
        );
        assert_eq!(&fixture.children[prefix..], &[a, b]);
        assert_eq!(fixture.dom.usage().managed_nodes(), managed);
        assert_eq!(fixture.dom.usage().ranges(), 1);
        eprintln!(
            "scale range_boundary fixture managed={managed} children={} unchanged_prefix={prefix} old_two_prefix_minimum_charged_steps={old_two_prefix_steps} endpoint_depth=2 endpoints={a:?},{b:?} ranges=1 setup_ns={} canonical_limits={:?}",
            fixture.children.len(),
            setup_started.elapsed().as_nanos(),
            fixture.dom.limits(),
        );
        for edit in [
            RangeBoundaryEdit::StartNoop,
            RangeBoundaryEdit::EndNoop,
            RangeBoundaryEdit::StartChanged,
            RangeBoundaryEdit::EndChanged,
        ] {
            assert_eq!(
                scale_range_bounds(&fixture.dom.snapshot(), range),
                ((a, 0), (b, 1))
            );
            let mut samples = Vec::new();
            for sample in 0..WARMUPS + SAMPLES {
                let before = fixture.dom.snapshot();
                let bounds = scale_range_bounds(&before, range);
                let usage = fixture.dom.usage();
                let (command, expected, changed) = match edit {
                    RangeBoundaryEdit::StartNoop => (
                        DomSourceEdit::set_range_start(range, a, 0),
                        ((a, 0), (b, 1)),
                        false,
                    ),
                    RangeBoundaryEdit::EndNoop => (
                        DomSourceEdit::set_range_end(range, b, 1),
                        ((a, 0), (b, 1)),
                        false,
                    ),
                    RangeBoundaryEdit::StartChanged => {
                        let offset = usize::from(sample.is_multiple_of(2));
                        (
                            DomSourceEdit::set_range_start(range, a, offset),
                            ((a, offset), (b, 1)),
                            true,
                        )
                    }
                    RangeBoundaryEdit::EndChanged => {
                        let offset = usize::from(!sample.is_multiple_of(2));
                        (
                            DomSourceEdit::set_range_end(range, b, offset),
                            ((a, 0), (b, offset)),
                            true,
                        )
                    }
                };
                // The cached existing-owner baseline and finite additional
                // storage are fixture configuration, outside measurement. Only
                // the fixed work limit discriminates sibling-prefix discovery.
                let baseline = fixture.dom.custody_registry.total().bytes();
                let headroom = source_limits();
                let absolute_bytes = baseline.checked_add(headroom.retained_bytes()).unwrap();
                let policy = DomSourceLimits::new(
                    absolute_bytes,
                    headroom.retained_string_code_units(),
                    WORK,
                    headroom.phase_publications(),
                    headroom.output_records(),
                    headroom.published_native_identities(),
                    headroom.nested_operations(),
                );
                eprintln!(
                    "scale range_boundary policy edit={edit:?} sample={sample} baseline_bytes={baseline} additional_headroom_bytes={} absolute_bytes={absolute_bytes} limits={policy:?}",
                    headroom.retained_bytes(),
                );
                let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
                let started = Instant::now();
                let mut lease = fixture
                    .dom
                    .begin_source_edit(command, policy, &cancel)
                    .unwrap();
                let initial_work = lease.status().spent_work();
                let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                    panic!("intrinsic Range boundary edit needs no external service");
                };
                let elapsed = started.elapsed();
                assert_eq!(done.output(), None);
                let changes = done.commit().changes().changes().to_vec();
                drop(lease);
                if let Some(measurement) = measurement {
                    let required = Hooks::one(Domain::Source)
                        .union(Hooks::one(Domain::Transaction))
                        .union(Hooks::one(Domain::Bank));
                    let counters = measurement.finish(required).unwrap();
                    assert!(counters.source_steps > 0);
                    assert!(
                        counters.source_steps <= usize::try_from(WORK).unwrap(),
                        "{counters:?}"
                    );
                    assert_eq!(
                        counters.published_roots,
                        usize::from(changed),
                        "{counters:?}"
                    );
                    // No Children mutation or output projection is requested.
                    assert_eq!(
                        counters.relation_sequence_materializations, 0,
                        "{counters:?}"
                    );
                    assert_eq!(
                        counters.relation_sequence_materialized_bytes, 0,
                        "{counters:?}"
                    );
                    // Generic reads/visits/allocations are reported as actual
                    // observations, not claimed as separately bounded here.
                    // The fixed charged SourceWork limit above detects the
                    // original scan; zero relation copies independently guards
                    // the unchanged wide Children backing.
                    eprintln!(
                        "scale range_boundary execution managed={managed} children={} edit={edit:?} sample={sample} initial_work={initial_work} counters={counters:?}",
                        fixture.children.len()
                    );
                }
                eprintln!(
                    "scale range_boundary completion managed={managed} edit={edit:?} sample={sample} initial_work={initial_work} changed={changed} final_publication_complete=true"
                );
                let after = fixture.dom.snapshot();
                let after_usage = fixture.dom.usage();
                assert_eq!(scale_range_bounds(&after, range), expected);
                assert_eq!(scale_range_bounds(&before, range), bounds);
                assert_eq!(scale_range_bounds(&original, range), ((a, 0), (b, 1)));
                assert_eq!(after_usage.managed_nodes(), usage.managed_nodes());
                assert_eq!(after_usage.string_code_units(), usage.string_code_units());
                assert_eq!(after_usage.ranges(), usage.ranges());
                if changed {
                    assert_ne!(bounds, expected);
                    assert_eq!(changes, [DomChange::RangeBoundary(range)]);
                    assert_eq!(after.revision().get(), before.revision().get() + 1);
                } else {
                    assert_eq!(bounds, expected);
                    assert!(changes.is_empty());
                    assert_eq!(after.stamp(), before.stamp());
                    assert_eq!(after_usage, usage);
                }
                assert_eq!(
                    fixture.dom.changes_since(before.stamp()).unwrap().changes(),
                    changes
                );
                for snapshot in [&original, &before, &after] {
                    assert_eq!(
                        snapshot.node(fixture.target).unwrap().children(),
                        fixture.children
                    );
                    assert_eq!(
                        snapshot.node(a).unwrap().data(),
                        Some(&DocumentString::from("child"))
                    );
                    assert_eq!(
                        snapshot.node(b).unwrap().data(),
                        Some(&DocumentString::from("child"))
                    );
                    assert_eq!(
                        snapshot.document(document).unwrap().active_range(),
                        original.document(document).unwrap().active_range()
                    );
                }
                assert!(fixture.dom.pending_source_edit().is_none());
                if sample >= WARMUPS {
                    samples.push(elapsed);
                }
            }
            report(&format!("RangeBoundary{edit:?}"), &samples);
        }
        let Fixture { dom, children, .. } = fixture;
        drop(children);
        let started = Instant::now();
        drop(dom);
        eprintln!(
            "scale range_boundary owner_drop_held_snapshot_ns={}",
            started.elapsed().as_nanos()
        );
        let started = Instant::now();
        drop(original);
        eprintln!(
            "scale range_boundary snapshot_drop_last_ns={}",
            started.elapsed().as_nanos()
        );
    }
}

#[test]
#[ignore = "explicit release retained native Details intentions after Source integration"]
fn unrelated_native_task_intentions_scale() {
    run(Context::DetailsTasks);
}

#[test]
#[ignore = "explicit release unrelated actual queued Select updates after public population"]
fn unrelated_select_task_bank_scale() {
    run(Context::SelectTasks);
}

#[test]
#[ignore = "explicit release retained public dispatch custody after Source integration"]
fn retained_dispatch_custody_scale() {
    run(Context::DispatchCustody);
}

#[test]
#[ignore = "explicit release native fullscreen model membership; no platform exit claim"]
fn unrelated_fullscreen_model_scale() {
    run(Context::FullscreenModel);
}

#[test]
#[ignore = "explicit release million-node qualification after incremental Source integration"]
fn retained_fullscreen_invocations_scale() {
    run(Context::FullscreenInvocations);
}

#[test]
#[ignore = "explicit release live Style/CSSOM root retention with no retirements"]
fn unrelated_live_style_roots_scale() {
    run(Context::StyleRoots);
}

const BULK_RECORDS: usize = 64;
const BULK_COUNTS: [usize; 5] = [16, 64, 256, 1024, 4096];
const BULK_FIXED_STORE: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bulk {
    Create,
    Import,
    SourceImport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Frontier {
    Occupied,
    Sparse,
    MixedExhausted,
}

// Exact selectors partition only the execution matrix. Operation/frontier
// choices absent from the invoked test fail. Explicit k may select any member
// of the shared geometric ladder, including in a full-size wrapper whose
// default is 64. Invalid choices cannot silently execute zero scenarios; unset
// selectors preserve every original default.
fn select_bulk_values<T: Copy + std::fmt::Debug + PartialEq>(
    variable: &str,
    available: &[T],
    parse: impl FnOnce(&str) -> T,
) -> Vec<T> {
    assert!(!available.is_empty());
    match std::env::var(variable) {
        Ok(value) => {
            let selected = parse(&value);
            assert!(
                available.contains(&selected),
                "{variable}={value} is absent from this test's original cases {available:?}"
            );
            vec![selected]
        }
        Err(std::env::VarError::NotPresent) => available.to_vec(),
        Err(std::env::VarError::NotUnicode(_)) => panic!("{variable} must be UTF-8"),
    }
}

fn run_selected_bulk(
    operations: &[Bulk],
    frontiers: &[Frontier],
    sizes: &[usize],
    counts: &[usize],
) {
    let selected_operations = select_bulk_values(
        "SURGEIST_SCALE_BULK_OPERATION",
        operations,
        |value| match value {
            "Create" => Bulk::Create,
            "Import" => Bulk::Import,
            "SourceImport" => Bulk::SourceImport,
            _ => panic!("exact bulk operation must be Create, Import or SourceImport"),
        },
    );
    let selected_frontiers = select_bulk_values(
        "SURGEIST_SCALE_BULK_FRONTIER",
        frontiers,
        |value| match value {
            "Occupied" => Frontier::Occupied,
            "Sparse" => Frontier::Sparse,
            "MixedExhausted" => Frontier::MixedExhausted,
            _ => panic!("exact bulk frontier must be Occupied, Sparse or MixedExhausted"),
        },
    );
    let count_choices = if std::env::var_os("SURGEIST_SCALE_BULK_K").is_some() {
        BULK_COUNTS.as_slice()
    } else {
        counts
    };
    let selected_counts = select_bulk_values("SURGEIST_SCALE_BULK_K", count_choices, |value| {
        value.parse::<usize>().expect("exact integer bulk k")
    });
    eprintln!(
        "bulk explicit_selection operations={selected_operations:?} frontiers={selected_frontiers:?} counts={selected_counts:?} sizes={sizes:?} default_operations={operations:?} default_frontiers={frontiers:?} default_counts={counts:?} explicit_count_choices={BULK_COUNTS:?} fixed_geometric_store={BULK_FIXED_STORE} warmups={WARMUPS} samples={SAMPLES}"
    );
    for operation in selected_operations {
        for &frontier in &selected_frontiers {
            run_bulk(operation, frontier, sizes, &selected_counts);
        }
    }
}

struct CapturedInput {
    snapshot: DomSnapshot,
    root: NodeId,
    children: Vec<NodeId>,
}

fn captured_input(records: usize) -> CapturedInput {
    let started = Instant::now();
    let mut dom = Dom::new(
        DomLimits::new(
            records + 1,
            records * 128,
            records * 3,
            0,
            0,
            8,
            65_536,
            crate::DomNameLimits::new(1_073_741_824, 2_000_000, 64_000_000, 268_435_456).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element(&document, "import-root");
    batch.set_attribute(&root, "captured", "literal");
    let mut children = Vec::new();
    for index in 0..records - 1 {
        let child = batch.create_comment(&document, format!("child-{index}"));
        batch.insert_before(&child, &root, None);
        children.push(child);
    }
    let commit = apply(&mut dom, batch);
    let root = commit.created(&root).unwrap().unwrap();
    let children = children
        .iter()
        .map(|ticket| commit.created(ticket).unwrap().unwrap())
        .collect();
    let setup = started.elapsed();
    let started = Instant::now();
    let snapshot = dom.snapshot();
    let capture = started.elapsed();
    assert_eq!(dom.usage().managed_nodes(), records + 1);
    let started = Instant::now();
    drop(dom);
    eprintln!(
        "bulk input_setup_ns={} input_capture_ns={} input_owner_drop_held_snapshot_ns={} input_managed={} imported_subtree_records={records}",
        setup.as_nanos(),
        capture.as_nanos(),
        started.elapsed().as_nanos(),
        records + 1
    );
    CapturedInput {
        snapshot,
        root,
        children,
    }
}

/// Populate every actual node through public edits. Only the three selected
/// vacant counter-boundary substitutions in MixedExhausted use an owning test
/// helper: reaching u64::MAX by public retirement requires 2^64 lifecycle edits.
/// MAX-generation creation and exhaustion then run through actual public edits.
/// No live identity, node, topology or allocation result is fabricated.
fn bulk_fixture(
    managed: usize,
    records: usize,
    frontier: Frontier,
) -> (Fixture, Vec<NodeId>, Vec<NodeId>, DomSnapshot) {
    let holes = match frontier {
        Frontier::Occupied => 0,
        Frontier::Sparse => *BULK_COUNTS.last().unwrap(),
        Frontier::MixedExhausted => 8,
    };
    let mut fixture = fixture_with_capacity(managed + holes, Context::Detached, records);
    let started = Instant::now();
    let mut retired = Vec::new();
    let mut exhausted = Vec::new();
    for index in 0..holes {
        // This is selected from public dense append order, independently of
        // the allocator's chosen free slot or cached eligibility summaries.
        let slot = 5 + index * (fixture.peak_managed - 6) / (holes - 1);
        let mut id = NodeId {
            owner: fixture.dom.owner(),
            slot,
            generation: 0,
        };
        assert_eq!(
            fixture.dom.snapshot().node(id).unwrap().data(),
            Some(&"u".into())
        );
        if matches!(frontier, Frontier::MixedExhausted) && [0, 3, 6].contains(&index) {
            // Only this slot is vacant. Seed its otherwise unreachable empty
            // counter, then require real public creation to issue MAX and real
            // public retirement below to make the slot permanently ineligible.
            let mut remove = DomBatch::new();
            remove.destroy(id);
            apply(&mut fixture.dom, remove);
            fixture
                .dom
                .scale_seed_vacant_generation(id.slot, u64::MAX - 1);
            // An empty slot stores its NEXT issuance generation. Exercise
            // both final public lifetimes, rather than treating that counter
            // as the previously retired generation.
            for generation in [u64::MAX - 1, u64::MAX] {
                let mut create = DomBatch::new();
                let ticket = create.create_comment(fixture.other_document, "u");
                let commit = apply(&mut fixture.dom, create);
                let issued = commit.created(&ticket).unwrap().unwrap();
                assert_eq!(issued.slot, id.slot);
                assert_eq!(issued.generation, generation);
                id = issued;
                if generation < u64::MAX {
                    let mut remove = DomBatch::new();
                    remove.destroy(issued);
                    apply(&mut fixture.dom, remove);
                }
            }
            exhausted.push(id);
        }
        retired.push(id);
    }
    let witness = fixture.dom.snapshot();
    if !retired.is_empty() {
        let mut batch = DomBatch::new();
        for id in &retired {
            batch.destroy(*id);
        }
        let retirement = Instant::now();
        let commit = apply(&mut fixture.dom, batch);
        assert_eq!(
            commit.changes().changes(),
            retired
                .iter()
                .copied()
                .map(DomChange::Destroyed)
                .collect::<Vec<_>>()
        );
        eprintln!(
            "bulk setup_retired_records={} exhausted_records={} retirement_apply_ns={}",
            retired.len(),
            exhausted.len(),
            retirement.elapsed().as_nanos()
        );
    }
    fixture.managed = managed;
    fixture.retired = holes;
    fixture.setup += started.elapsed();
    assert_eq!(fixture.dom.usage().managed_nodes(), managed);
    assert_eq!(fixture.dom.snapshot().nodes().count(), managed);
    for id in &retired {
        assert_eq!(
            fixture.dom.snapshot().node(*id).unwrap_err(),
            DomError::StaleNode
        );
        assert_eq!(witness.node(*id).unwrap().data(), Some(&"u".into()));
    }
    (fixture, retired, exhausted, witness)
}

fn expected_bulk_ids(fixture: &Fixture, retired: &[NodeId], records: usize) -> Vec<NodeId> {
    let eligible = retired.iter().filter(|old| old.generation != u64::MAX);
    let reused = eligible.clone().count().min(records);
    eligible
        .take(records)
        .map(|old| NodeId {
            generation: old.generation.checked_add(1).unwrap(),
            ..*old
        })
        .chain((0..records - reused).map(|offset| NodeId {
            owner: fixture.dom.owner(),
            slot: fixture.peak_managed + offset,
            generation: 0,
        }))
        .collect()
}

fn bulk_add(left: usize, right: usize) -> usize {
    left.checked_add(right).expect("bounded bulk addition")
}
fn bulk_mul(left: usize, right: usize) -> usize {
    left.checked_mul(right)
        .expect("bounded bulk multiplication")
}

#[derive(Debug)]
struct BulkEnvelope {
    reads_and_lookups: usize,
    materializations: usize,
    bank_pages: usize,
    vacancy_pages: usize,
    sequence_bytes: usize,
    relation_bytes: usize,
    structural_bytes: usize,
    source_work: usize,
    slot_discovery: usize,
}
impl BulkEnvelope {
    fn new(extent: usize, records: usize, operation: Bulk) -> Self {
        // This model is specific to detached Comments or one flat XML root
        // with one fixed attribute and Comment children, not arbitrary native
        // subtrees. Its fixed source frontiers are detailed below.
        let p = bulk_add(records, 8);
        let r = bulk_mul(
            p,
            if matches!(operation, Bulk::Create) {
                1
            } else {
                3
            },
        );
        let binary_depth = usize::BITS as usize - extent.leading_zeros() as usize;
        let d = binary_depth.div_ceil(4);
        let registry_depth = u64::BITS.div_ceil(4) as usize;
        let h = bulk_add(d, registry_depth);
        // counted_relations::{adjust,replace}: five qualified parts per key,
        // two key paths per change. inbound::{adjust_pair,finish}: at most
        // five Map changes/reference, including both old-Created removal maps.
        // nodes/identities/publication/slot_index add at most 16p births.
        let u = bulk_add(bulk_mul(50, r), bulk_mul(16, p));
        let a = if matches!(operation, Bulk::Create) {
            0
        } else {
            bulk_add(bulk_mul(10, r), bulk_mul(4, p))
        };
        // Complete generation/global-ID words retain R=16, even when current
        // values happen to use shallow paths. Native Bank radix is sixteen.
        let frontier = bulk_add(
            bulk_mul(bulk_mul(22, bulk_add(h, 3)), r),
            bulk_mul(bulk_mul(36, d), p),
        );
        let n = bulk_add(bulk_add(u, a), frontier);
        // <=63 generic calls/Map change, <=20r publication visitor reads,
        // plus separate full-key aliases and four 32p callgraph allowances.
        let q = bulk_add(bulk_add(bulk_mul(367, r), bulk_mul(128, p)), a);
        // bank::{register_page,register_record,reconcile_releases}: <=8R
        // registry passes per new/adopted record/page; native paths and fixed
        // selected traversal remain separate. No term contains k*k or S.
        let bank_pages = bulk_add(
            bulk_add(bulk_mul(bulk_mul(8, registry_depth), n), bulk_mul(h, q)),
            bulk_mul(32, p),
        );
        // <=36 custody gates per frontier allocation, one nested dependency
        // per new/adopted record, and bounded interpreter/string/frame work.
        let source_work = bulk_add(
            usize::try_from(SOURCE_WORK).unwrap(),
            bulk_add(
                bulk_mul(37, bulk_add(u, a)),
                bulk_add(bulk_mul(36, frontier), bulk_mul(512, p)),
            ),
        );
        // import::Frame/select::EffectFrame have <=25 stages/output here,
        // rounded to 32p. Push/pop plus other mutation inverses fit 96p.
        // journal::{reserve_deque,reserve_vec} doubles capacity: initialized
        // writes plus copied prefixes are bounded by 3J+12 entries.
        let f = bulk_mul(32, p);
        let j = bulk_mul(96, p);
        let journal = bulk_mul(
            bulk_add(bulk_mul(3, j), 12),
            size_of::<super::transaction::Undo>(),
        );
        // Both records/created arenas initialize <=p total entries and move
        // <2p initialized prefixes across geometric capacity growth.
        let arena = bulk_mul(bulk_mul(3, p), size_of::<Option<super::DraftNode>>());
        // Effects initialize <=F frames and copy <2F growth prefixes.
        let effects = bulk_mul(bulk_mul(3, f), size_of::<super::select::EffectFrame>());
        let keys = bulk_mul(
            bulk_mul(4, p),
            bulk_add(size_of::<super::Key>(), size_of::<NodeId>()),
        );
        let outputs = bulk_mul(
            bulk_mul(8, p),
            bulk_add(size_of::<super::Key>(), size_of::<DomChange>()),
        );
        // Actual local tickets, receipt fields, mapped IDs, selected sparse
        // slots and Created changes have their own initialized-write terms;
        // final change-Arc copying remains in the separate output allowance.
        let initialized_per_record = bulk_add(
            bulk_add(
                size_of::<super::Key>(),
                bulk_add(size_of::<Option<NodeId>>(), size_of::<bool>()),
            ),
            bulk_add(
                bulk_add(size_of::<NodeId>(), size_of::<usize>()),
                size_of::<DomChange>(),
            ),
        );
        let initialized = bulk_mul(p, initialized_per_record);
        let markers = bulk_mul(bulk_mul(8, p), size_of::<(u64, usize)>());
        let sequence_bytes = bulk_add(
            bulk_add(bulk_add(journal, arena), bulk_add(effects, keys)),
            bulk_add(
                bulk_add(bulk_add(outputs, initialized), markers),
                size_of::<super::history::HistoryEntry>(),
            ),
        );
        let envelope = Self {
            reads_and_lookups: q,
            materializations: u,
            bank_pages,
            vacancy_pages: bulk_mul(bulk_mul(2, p), binary_depth),
            sequence_bytes,
            // The real relation/shared-sequence subcategory excludes Journal,
            // effects and arenas. Appends initialize n keys, doubling moves
            // <2n old keys, and publication projects n canonical NodeIds.
            relation_bytes: if matches!(operation, Bulk::Create) {
                0
            } else {
                bulk_add(
                    bulk_mul(bulk_mul(3, p), size_of::<super::Key>()),
                    bulk_mul(p, size_of::<NodeId>()),
                )
            },
            // Keep the existing allocated-byte guard independently unchanged.
            structural_bytes: bulk_mul(bulk_mul(p, 131_072), binary_depth),
            source_work,
            slot_discovery: bulk_mul(32, p),
        };
        eprintln!(
            "bulk source_derived_envelope operation={operation:?} k={records} extent={extent} p={p} r={r} U={u} A={a} P={frontier} N={n} D={binary_depth} d={d} R={registry_depth} h={h} F={f} J={j} limits={envelope:?} finite_storage_configuration_bytes=32000000000 storage_configuration_not_oracle=true"
        );
        envelope
    }
}

fn bulk_counters(
    measurement: Option<Measurement>,
    records: usize,
    operation: Bulk,
    bounds: &BulkEnvelope,
) {
    let Some(measurement) = measurement else {
        return;
    };
    let mut required = Hooks::one(Domain::Bank)
        .union(Hooks::one(Domain::Transaction))
        .union(Hooks::one(Domain::Validation))
        .union(Hooks::one(Domain::Accounting));
    if matches!(operation, Bulk::SourceImport) {
        required = required.union(Hooks::one(Domain::Source));
    }
    let counters = measurement.finish(required).unwrap();
    if records == *BULK_COUNTS.last().unwrap() {
        // At 4096, even one counted repeated-prefix scan cannot fit the
        // complete qualified-admin bounds: pairs=8,386,560 > worst Q=5,183,352.
        let pairs = bulk_mul(records, records.checked_sub(1).unwrap()) / 2;
        assert!(
            pairs > bounds.reads_and_lookups && pairs > bounds.vacancy_pages,
            "independent prefix discriminator: pairs={pairs} bounds={bounds:?}"
        );
        let prefix_bytes = bulk_mul(pairs, size_of::<super::Key>().min(size_of::<NodeId>()));
        assert!(
            prefix_bytes > bounds.relation_bytes,
            "independent relation-prefix discriminator: bytes={prefix_bytes} bounds={bounds:?}"
        );
    }
    assert!(
        counters.canonical_record_reads <= bounds.reads_and_lookups,
        "{counters:?}"
    );
    assert!(
        counters.validation_record_visits <= bounds.reads_and_lookups,
        "{counters:?}"
    );
    assert!(
        counters.accounting_record_visits <= bounds.reads_and_lookups,
        "{counters:?}"
    );
    let queue_admin = counters
        .queue_record_visits
        .checked_add(counters.queue_records_moved)
        .expect("measured queue work");
    assert!(queue_admin <= bounds.reads_and_lookups, "{counters:?}");
    assert!(
        counters.bank_lookups <= bounds.reads_and_lookups,
        "{counters:?}"
    );
    assert!(
        counters.record_materializations <= bounds.materializations,
        "{counters:?}"
    );
    assert!(
        counters.sequence_materialized_bytes <= bounds.sequence_bytes,
        "{counters:?}"
    );
    if matches!(operation, Bulk::Create) {
        assert_eq!(
            counters.relation_sequence_materializations, 0,
            "{counters:?}"
        );
        assert_eq!(
            counters.relation_sequence_materialized_bytes, 0,
            "{counters:?}"
        );
    } else {
        assert!(
            counters.relation_sequence_materializations > 0,
            "{counters:?}"
        );
        assert!(
            counters.relation_sequence_materialized_bytes > 0
                && counters.relation_sequence_materialized_bytes <= bounds.relation_bytes,
            "{counters:?}"
        );
    }
    assert!(
        counters.structural_bytes <= bounds.structural_bytes,
        "{counters:?}"
    );
    assert!(
        counters.bank_page_visits > 0 && counters.bank_page_visits <= bounds.bank_pages,
        "{counters:?}"
    );
    assert!(
        counters.vacancy_page_visits <= bounds.vacancy_pages,
        "{counters:?}"
    );
    // This exact flat XML/Comment fixture has no Slot/shadow/control
    // participants. The owning fixed/growing-P/ordinal tests cover real slots.
    assert_eq!(counters.slot_assignments, 0, "{counters:?}");
    assert_eq!(counters.slot_ordinal_successor_reads, 0, "{counters:?}");
    assert!(
        counters.slot_discovery_steps <= bounds.slot_discovery,
        "{counters:?}"
    );
    assert_eq!(counters.published_roots, 1, "{counters:?}");
    if matches!(operation, Bulk::SourceImport) {
        // Plain XML import has no reached external wait requiring headroom.
        // The Style completion companion requires its real preflight instead.
        assert!(
            counters.source_steps > 0 && counters.source_steps <= bounds.source_work,
            "{counters:?}"
        );
        assert!(
            counters.savepoints > 0 && counters.savepoint_peak_bytes > 0,
            "{counters:?}"
        );
        assert!(
            counters.source_peak_bytes > 0 && counters.source_peak_units > 0,
            "{counters:?}"
        );
    }
    eprintln!(
        "bulk execution operation={operation:?} k={records} bounds={bounds:?} counters={counters:?}"
    );
}

fn check_import(
    snapshot: &DomSnapshot,
    document: DocumentId,
    ids: &[NodeId],
    input: &CapturedInput,
) {
    let root = ids[0];
    let children = &ids[1..];
    assert_eq!(snapshot.node(root).unwrap().parent(), None);
    assert_eq!(
        snapshot.node(root).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(snapshot.node(root).unwrap().children(), children);
    assert_eq!(
        snapshot
            .node(root)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"captured".into()),
        Some(&DocumentString::from("literal"))
    );
    for (index, child) in children.iter().enumerate() {
        let node = snapshot.node(*child).unwrap();
        assert_eq!(node.parent(), Some(root));
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(
            node.data(),
            Some(&DocumentString::from(format!("child-{index}")))
        );
    }
    assert_eq!(
        input.snapshot.node(input.root).unwrap().children(),
        input.children
    );
    assert_eq!(
        snapshot.node(input.root).unwrap_err(),
        DomError::ForeignOwner
    );
    assert_eq!(
        input.snapshot.node(root).unwrap_err(),
        DomError::ForeignOwner
    );
}

fn run_bulk(operation: Bulk, frontier: Frontier, sizes: &[usize], counts: &[usize]) {
    require_release();
    eprintln!(
        "bulk environment profile=release os={} arch={} revision={} toolchain={} machine={} counters_enabled={} operation={operation:?} frontier={frontier:?} warmups={WARMUPS} samples={SAMPLES}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into()),
        scale_metrics::ENABLED,
    );
    for &managed in sizes {
        for &records in counts {
            let input = (!matches!(operation, Bulk::Create)).then(|| captured_input(records));
            let mut timings = Vec::new();
            let mut begin_timings = Vec::new();
            let mut advance_timings = Vec::new();
            // Rebuild outside measurement for every sample. Both S and its
            // vacancy/exhaustion frontier are identical across geometric k and
            // repeats: an earlier occupied sample cannot create a sparse bank.
            for sample in 0..WARMUPS + SAMPLES {
                let (mut fixture, retired, exhausted, witness) =
                    bulk_fixture(managed, records, frontier);
                let before = fixture.dom.snapshot();
                let extent = bulk_add(fixture.peak_managed, records);
                let bounds = BulkEnvelope::new(extent, records, operation);
                let expected = expected_bulk_ids(&fixture, &retired, records);
                let expected_changes: Vec<_> =
                    expected.iter().copied().map(DomChange::Created).collect();
                eprintln!(
                    "bulk fixture operation={operation:?} frontier={frontier:?} k={records} sample={sample} managed={managed} allocated_extent={} holes={} first_retired={:?} last_retired={:?} setup_ns={} limits={:?} usage={:?} exhausted={exhausted:?}",
                    fixture.peak_managed,
                    retired.len(),
                    retired.first(),
                    retired.last(),
                    fixture.setup.as_nanos(),
                    fixture.dom.limits(),
                    fixture.dom.usage()
                );
                let mut batch = DomBatch::new();
                let mut tickets = Vec::new();
                let private = if matches!(operation, Bulk::Create) {
                    let private = batch.create_comment(fixture.other_document, "never published");
                    batch.destroy(&private);
                    Some(private)
                } else {
                    None
                };
                match operation {
                    Bulk::Create => {
                        for index in 0..records {
                            tickets.push(
                                batch.create_comment(
                                    fixture.other_document,
                                    format!("bulk-{index}"),
                                ),
                            );
                        }
                    }
                    Bulk::Import => {
                        let input = input.as_ref().unwrap();
                        tickets.push(batch.import_node(
                            &input.snapshot,
                            input.root,
                            fixture.other_document,
                            true,
                        ));
                    }
                    Bulk::SourceImport => {}
                }
                let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
                let started = Instant::now();
                let elapsed = if matches!(operation, Bulk::SourceImport) {
                    let input = input.as_ref().unwrap();
                    let cancel = AtomicBool::new(false);
                    let allowances = source_limits();
                    let limits = DomSourceLimits::new(
                        allowances.retained_bytes(),
                        allowances.retained_string_code_units(),
                        u64::try_from(bounds.source_work).expect("finite Source bulk work"),
                        8,
                        records * 4,
                        u64::try_from(records).unwrap().checked_mul(2).unwrap(),
                        1,
                    );
                    let edit = DomSourceEdit::import_node(
                        &input.snapshot,
                        input.root,
                        fixture.other_document.node(),
                        true,
                    );
                    let begin = Instant::now();
                    let mut lease = fixture
                        .dom
                        .begin_source_edit(edit, limits, &cancel)
                        .unwrap();
                    let begin_elapsed = begin.elapsed();
                    let initial_work = lease.status().spent_work();
                    let advance = Instant::now();
                    let done = match lease.advance(&cancel) {
                        Ok(DomSourceProgress::Complete(done)) => done,
                        Ok(_) => panic!("captured generic XML subtree has no external service"),
                        Err(error) => {
                            eprintln!(
                                "bulk source_refusal operation={operation:?} frontier={frontier:?} k={records} sample={sample} canonical_managed={managed} input_managed={} input_root_children={} input_root_attributes=1 initial_work={initial_work} status={:?} limits={limits:?} error={error:?}",
                                records + 1,
                                input.children.len(),
                                lease.status(),
                            );
                            if let Some(measurement) = measurement {
                                let counters = measurement
                                    .finish(
                                        Hooks::one(Domain::Bank)
                                            .union(Hooks::one(Domain::Transaction))
                                            .union(Hooks::one(Domain::Accounting))
                                            .union(Hooks::one(Domain::Source)),
                                    )
                                    .unwrap();
                                eprintln!("bulk source_refusal actual_counters={counters:?}");
                            }
                            panic!("actual Source import refused: {error:?}");
                        }
                    };
                    let advance_elapsed = advance.elapsed();
                    let elapsed = started.elapsed();
                    assert_eq!(done.output(), Some(DomSourceOutput::Node(expected[0])));
                    assert_eq!(done.published_node(), Some(expected[0]));
                    assert_eq!(done.commit().changes().changes(), expected_changes);
                    eprintln!(
                        "bulk source k={records} sample={sample} new_owner_capture_admission=true input_managed={} begin_including_input_admission_and_preflight_ns={} advance_ns={} initial_work={initial_work} limits={limits:?}",
                        records + 1,
                        begin_elapsed.as_nanos(),
                        advance_elapsed.as_nanos()
                    );
                    if sample >= WARMUPS {
                        begin_timings.push(begin_elapsed);
                        advance_timings.push(advance_elapsed);
                    }
                    elapsed
                } else {
                    let commit = apply(&mut fixture.dom, batch);
                    let elapsed = started.elapsed();
                    assert_eq!(commit.changes().changes(), expected_changes);
                    if let Some(private) = &private {
                        assert_eq!(commit.created(private).unwrap(), None);
                    }
                    if matches!(operation, Bulk::Create) {
                        let actual: Vec<_> = tickets
                            .iter()
                            .map(|ticket| commit.created(ticket).unwrap().unwrap())
                            .collect();
                        assert_eq!(actual, expected);
                    } else {
                        assert_eq!(commit.created(&tickets[0]).unwrap(), Some(expected[0]));
                    }
                    elapsed
                };
                bulk_counters(measurement, records, operation, &bounds);
                let now = fixture.dom.snapshot();
                assert_eq!(now.revision().get(), before.revision().get() + 1);
                assert_eq!(fixture.dom.usage().managed_nodes(), managed + records);
                for id in &expected {
                    assert_eq!(before.node(*id).unwrap_err(), DomError::StaleNode);
                }
                for id in &retired {
                    assert_eq!(now.node(*id).unwrap_err(), DomError::StaleNode);
                }
                for id in &exhausted {
                    assert!(!expected.iter().any(|fresh| fresh.slot == id.slot));
                    assert!(fixture.dom.state.slots[id.slot].retired);
                }
                if matches!(operation, Bulk::Create) {
                    for (index, id) in expected.iter().enumerate() {
                        let node = now.node(*id).unwrap();
                        assert_eq!(node.parent(), None);
                        assert_eq!(node.owner_document(), Some(fixture.other_document));
                        assert_eq!(
                            node.data(),
                            Some(&DocumentString::from(format!("bulk-{index}")))
                        );
                    }
                } else {
                    check_import(
                        &now,
                        fixture.other_document,
                        &expected,
                        input.as_ref().unwrap(),
                    );
                }
                assert_eq!(before.nodes().count(), managed);
                assert_eq!(witness.nodes().count(), managed + retired.len());
                assert_eq!(
                    now.node(fixture.target).unwrap().children(),
                    fixture.children
                );
                assert_eq!(
                    before.node(fixture.target).unwrap().children(),
                    fixture.children
                );
                eprintln!(
                    "bulk sample operation={operation:?} frontier={frontier:?} k={records} sample={sample} before_managed={managed} after_managed={} elapsed_ns={}",
                    fixture.dom.usage().managed_nodes(),
                    elapsed.as_nanos()
                );
                if sample >= WARMUPS {
                    timings.push(elapsed);
                }
                let started = Instant::now();
                drop(fixture);
                eprintln!(
                    "bulk owner_drop_held_snapshots_ns={}",
                    started.elapsed().as_nanos()
                );
                let started = Instant::now();
                drop(before);
                drop(now);
                drop(witness);
                eprintln!(
                    "bulk target_snapshot_teardown_ns={}",
                    started.elapsed().as_nanos()
                );
            }
            report(
                &format!("bulk_{operation:?}_{frontier:?}_S{managed}_k{records}"),
                &timings,
            );
            if matches!(operation, Bulk::SourceImport) {
                report(
                    "bulk_source_begin_including_input_admission_and_preflight",
                    &begin_timings,
                );
                report("bulk_source_advance", &advance_timings);
            }
            let started = Instant::now();
            drop(input);
            eprintln!(
                "bulk input_snapshot_teardown_ns={}",
                started.elapsed().as_nanos()
            );
        }
    }
}

#[test]
#[ignore = "explicit release public bulk creation after million-node occupied setup"]
fn postsetup_bulk_creation_occupied_scale() {
    run_selected_bulk(
        &[Bulk::Create],
        &[Frontier::Occupied],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release public bulk creation into sparse retired slots"]
fn postsetup_bulk_creation_sparse_vacancies_scale() {
    run_selected_bulk(
        &[Bulk::Create],
        &[Frontier::Sparse],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release public import after million-node occupied setup"]
fn postsetup_captured_import_occupied_scale() {
    run_selected_bulk(
        &[Bulk::Import],
        &[Frontier::Occupied],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release public import into sparse retired slots"]
fn postsetup_captured_import_sparse_vacancies_scale() {
    run_selected_bulk(
        &[Bulk::Import],
        &[Frontier::Sparse],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release original Source import after million-node occupied setup"]
fn postsetup_source_captured_import_occupied_scale() {
    run_selected_bulk(
        &[Bulk::SourceImport],
        &[Frontier::Occupied],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release original Source import into sparse retired slots"]
fn postsetup_source_captured_import_sparse_vacancies_scale() {
    run_selected_bulk(
        &[Bulk::SourceImport],
        &[Frontier::Sparse],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release mixed exhausted generations after public million-node setup"]
fn postsetup_bulk_exhausted_generation_frontier_scale() {
    run_selected_bulk(
        &[Bulk::Create, Bulk::Import, Bulk::SourceImport],
        &[Frontier::MixedExhausted],
        &selected_sizes(),
        &[BULK_RECORDS],
    );
}
#[test]
#[ignore = "explicit release geometric k with identical populated/sparse/exhausted S"]
fn postsetup_geometric_bulk_frontiers_scale() {
    run_selected_bulk(
        &[Bulk::Create, Bulk::Import, Bulk::SourceImport],
        &[
            Frontier::Occupied,
            Frontier::Sparse,
            Frontier::MixedExhausted,
        ],
        &[BULK_FIXED_STORE],
        &BULK_COUNTS,
    );
}

fn continuation_limits() -> DomSourceLimits {
    let limits = source_limits();
    DomSourceLimits::new(
        limits.retained_bytes(),
        limits.retained_string_code_units(),
        100_000,
        64,
        4096,
        4096,
        4,
    )
}

fn callback_source(dom: &mut Dom, edit: DomSourceEdit) -> crate::DomSourceCompletion {
    let cancel = AtomicBool::new(false);
    let mut lease = dom
        .begin_source_edit(edit, continuation_limits(), &cancel)
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
        panic!("the selected callback primitive has no external service");
    };
    done
}

// Four fixed Source participants: the original Show plus the three callback
// primitives. Each has at most 32 journal slots, four effect frames and one
// selected arena record. journal::{reserve_vec,reserve_deque,push_prepared},
// source::effects and publication output initialization/movement are counted.
// This is a callgraph allowance, independent of the observed pilot byte count.
fn show_sequence_bound() -> usize {
    let participants = 4;
    let journal = bulk_mul(32, participants);
    let frames = bulk_mul(4, participants);
    let arenas = participants;
    let markers = bulk_mul(4, participants);
    let terms = [
        bulk_mul(bulk_mul(3, journal), size_of::<super::transaction::Undo>()),
        bulk_mul(bulk_mul(3, frames), size_of::<super::select::EffectFrame>()),
        bulk_mul(bulk_mul(3, arenas), size_of::<Option<super::DraftNode>>()),
        bulk_mul(bulk_mul(4, markers), size_of::<[u64; 2]>()),
        // Three actual History pushes; shared survivors are not copied.
        bulk_mul(3, size_of::<super::history::HistoryEntry>()),
        bulk_mul(32, size_of::<DomChange>()),
        bulk_mul(32, size_of::<super::Key>()),
        bulk_mul(32, size_of::<NodeId>()),
        bulk_mul(32, size_of::<usize>()),
        bulk_mul(16, size_of::<crate::Attribute>()),
        bulk_mul(512, size_of::<u16>()),
    ];
    let bound = terms.into_iter().fold(0, bulk_add);
    eprintln!(
        "scale show_sequence_model participants={participants} journal={journal} frames={frames} arenas={arenas} markers={markers} formula=3J_Undo+3F_EffectFrame+3A_OptionDraftNode+4M_Marker+3_HistoryEntry+32_DomChange+32_Key+32_NodeId+32_usize+16_Attribute+512_u16 bound_bytes={bound} relation_events_bound=0 relation_bytes_bound=0"
    );
    bound
}

/// Native DOM executor qualification, not Runtime listener-delivery evidence.
/// The Runtime companion uses its real dispatcher/receipt for the same callback
/// sequence. Here the actual affine permit scopes actual callback edits and the
/// issuing executor seals their truthful uncanceled outcome through its public
/// capability; no receipt fields, request stages or private work atoms are set.
#[test]
#[ignore = "explicit release actual parked Source/savepoint/reentry instrumentation after public million-node setup"]
fn postsetup_native_source_park_resume_reentry_scale() {
    require_release();
    let sequence_bound = show_sequence_bound();
    let cancel = AtomicBool::new(false);
    for managed in selected_sizes() {
        let mut fixture = fixture_with_capacity(managed - 1, Context::Detached, 2);
        fixture
            .dom
            .set_limits(fixture.dom.limits().with_dispatch_custody(
                DomDispatchCustodyLimits::new(8, 128, 1_000_000, 1_000_000, 100_000).unwrap(),
            ))
            .unwrap();
        let setup = Instant::now();
        let document = fixture
            .dom
            .snapshot()
            .node(fixture.target)
            .unwrap()
            .owner_document()
            .unwrap();
        let mut batch = DomBatch::new();
        let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
        batch.insert_before(&dialog, fixture.target, None);
        let commit = apply(&mut fixture.dom, batch);
        let dialog = commit.created(&dialog).unwrap().unwrap();
        fixture.children.push(dialog);
        fixture.managed = managed;
        fixture.setup += setup.elapsed();
        let executor = fixture.dom.bind_source_event_executor(&cancel).unwrap();
        assert_eq!(fixture.dom.usage().managed_nodes(), managed);
        assert_eq!(fixture.dom.snapshot().nodes().count(), managed);
        let original = fixture.dom.snapshot();
        let mut begin_samples = Vec::new();
        let mut callback_samples = Vec::new();
        let mut resume_samples = Vec::new();
        eprintln!(
            "scale continuation profile=release os={} arch={} warmups={WARMUPS} samples={SAMPLES} managed={managed} setup_ns={} source_limits={:?} counters_enabled={} executor=native_DOM_capability runtime_listener_evidence=companion revision={} toolchain={} machine={}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            fixture.setup.as_nanos(),
            continuation_limits(),
            scale_metrics::ENABLED,
            std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
            std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
            std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into())
        );
        for sample in 0..WARMUPS + SAMPLES {
            let next = if sample.is_multiple_of(2) {
                "next"
            } else {
                "base"
            };
            let before = fixture.dom.snapshot();
            let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
            let start = Instant::now();
            let (event, parked_work) = {
                let mut lease = fixture
                    .dom
                    .begin_source_edit(
                        DomSourceEdit::show_dialog(dialog),
                        continuation_limits(),
                        &cancel,
                    )
                    .unwrap();
                let DomSourceProgress::Request(DomSourceRequest::Event(event)) =
                    lease.advance(&cancel).unwrap()
                else {
                    panic!("actual native Show must reach beforetoggle");
                };
                assert_eq!(lease.status().operation(), event.operation());
                assert!(!lease.status().has_published());
                (event, lease.status().spent_work())
                // Lease loss deliberately parks the original continuation.
            };
            let begin = start.elapsed();
            let operation = event.operation();
            assert_eq!(fixture.dom.snapshot().stamp(), before.stamp());
            assert_eq!(event.kind(), crate::DomSourceEventKind::BeforeToggle);
            let toggle = event.toggle_event().unwrap();
            assert_eq!(toggle.old_state(), surgeist_html::HtmlDialogState::Closed);
            assert_eq!(toggle.new_state(), surgeist_html::HtmlDialogState::Open);
            let start = Instant::now();
            let permit = executor
                .begin_dispatch(&fixture.dom, &event, &cancel)
                .unwrap();
            let scope = executor
                .enter_dispatch(&fixture.dom, &permit, &cancel)
                .unwrap();
            let changed = callback_source(
                &mut fixture.dom,
                DomSourceEdit::set_attribute(fixture.target, "data-local", next),
            );
            assert_eq!(
                changed.commit().changes().changes(),
                &[DomChange::Attributes(fixture.target)]
            );
            let created = callback_source(
                &mut fixture.dom,
                DomSourceEdit::create_comment(fixture.other_document.node(), "callback-created"),
            );
            let Some(DomSourceOutput::Node(id)) = created.output() else {
                panic!("actual nested constructor output");
            };
            assert_eq!(created.published_node(), Some(id));
            assert_eq!(
                created.commit().changes().changes(),
                &[DomChange::Created(id)]
            );
            let opened = callback_source(
                &mut fixture.dom,
                DomSourceEdit::set_attribute(dialog, "open", ""),
            );
            assert_eq!(
                opened.commit().changes().changes(),
                &[DomChange::Attributes(dialog)]
            );
            drop(scope);
            let mut permit = Some(permit);
            let mut publication = Some(
                executor
                    .seal_completed(
                        &fixture.dom,
                        &mut permit,
                        crate::DomSourceEventOutcome::NotCanceled,
                        &cancel,
                    )
                    .unwrap(),
            );
            assert!(permit.is_none());
            let callback_elapsed = start.elapsed();
            let published = fixture.dom.snapshot();
            assert_eq!(published.revision().get(), before.revision().get() + 3);
            assert_eq!(
                published.node(id).unwrap().data(),
                Some(&"callback-created".into())
            );
            assert_eq!(
                published.node(id).unwrap().owner_document(),
                Some(fixture.other_document)
            );
            assert_eq!(before.node(id).unwrap_err(), DomError::StaleNode);
            assert_eq!(
                fixture.dom.changes_since(before.stamp()).unwrap().changes(),
                &[
                    DomChange::Attributes(fixture.target),
                    DomChange::Created(id),
                    DomChange::Attributes(dialog)
                ]
            );
            assert!(fixture.dom.pending_source_edit().unwrap().spent_work() > parked_work);
            let start = Instant::now();
            {
                let mut lease = fixture
                    .dom
                    .resume_source_edit(operation, continuation_limits(), &cancel)
                    .unwrap();
                lease
                    .accept_source_event(&event, &mut publication, &cancel)
                    .unwrap();
                assert!(publication.is_none());
                let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                    panic!("same original Show sees callback-open state and completes");
                };
                assert_eq!(done.operation(), operation);
                assert!(done.commit().changes().changes().is_empty());
            }
            let resumed = start.elapsed();
            assert_eq!(fixture.dom.snapshot().stamp(), published.stamp());
            assert!(fixture.dom.pending_source_edit().is_none());
            assert_eq!(
                fixture.dom.snapshot().node(id).unwrap().data(),
                Some(&"callback-created".into())
            );
            assert_eq!(
                attribute(&fixture.dom.snapshot(), fixture.target),
                DocumentString::from(next)
            );
            assert_eq!(
                before
                    .node(dialog)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"open".into()),
                None
            );
            assert_eq!(
                before.node(fixture.target).unwrap().children(),
                fixture.children
            );
            if let Some(measurement) = measurement {
                let counters = measurement
                    .finish(
                        Hooks::one(Domain::Source)
                            .union(Hooks::one(Domain::Bank))
                            .union(Hooks::one(Domain::Transaction))
                            .union(Hooks::one(Domain::Accounting))
                            .union(Hooks::one(Domain::Validation)),
                    )
                    .unwrap();
                // Require actual observer values; missing hooks/zero fallback
                // values cannot qualify the real parked/savepoint/custody path.
                // This genuine beforetoggle path reserves its fixed tail and
                // the callback makes resume a no-op. It does not reach the
                // execution-and-restore preflight; the Style companion does.

                assert!(
                    counters.savepoints >= 2 && counters.savepoint_peak_bytes > 0,
                    "{counters:?}"
                );
                assert!(
                    counters.source_peak_bytes > 0 && counters.source_peak_units > 0,
                    "{counters:?}"
                );
                assert!(
                    counters.source_peak_bytes >= counters.source_live_bytes,
                    "{counters:?}"
                );
                assert!(
                    counters.source_steps > 0 && counters.source_steps <= 100_000,
                    "{counters:?}"
                );
                assert!(counters.canonical_record_reads <= 2048, "{counters:?}");
                assert!(counters.accounting_record_visits <= 2048, "{counters:?}");
                let queue_admin = counters
                    .queue_record_visits
                    .checked_add(counters.queue_records_moved)
                    .expect("measured queue work");
                assert!(queue_admin <= 2048, "{counters:?}");
                assert!(counters.validation_record_visits <= 2048, "{counters:?}");
                assert!(counters.record_materializations <= 256, "{counters:?}");
                assert!(
                    counters.sequence_materialized_bytes <= sequence_bound,
                    "bound={sequence_bound} {counters:?}"
                );
                // No callback changes Children/events/shared relation backing.
                assert_eq!(
                    counters.relation_sequence_materializations, 0,
                    "{counters:?}"
                );
                assert_eq!(
                    counters.relation_sequence_materialized_bytes, 0,
                    "{counters:?}"
                );
                let depth = usize::BITS as usize - managed.leading_zeros() as usize;
                assert!(counters.bank_page_visits <= 4096 * depth, "{counters:?}");
                assert_eq!(counters.published_roots, 3, "{counters:?}");
                eprintln!(
                    "scale continuation actual_park_reentry_savepoint_and_custody_path sample={sample} managed={managed} counters={counters:?}"
                );
            }
            eprintln!(
                "scale continuation sample={sample} managed={managed} operation={operation:?} parked_work={parked_work} created={id:?} begin_ns={} scoped_callback_and_seal_ns={} original_resume_ns={}",
                begin.as_nanos(),
                callback_elapsed.as_nanos(),
                resumed.as_nanos()
            );
            if sample >= WARMUPS {
                begin_samples.push(begin);
                callback_samples.push(callback_elapsed);
                resume_samples.push(resumed);
            }
            let mut cleanup = DomBatch::new();
            cleanup.remove_attribute(dialog, "open");
            cleanup.destroy(id);
            apply(&mut fixture.dom, cleanup);
            assert_eq!(fixture.dom.usage().managed_nodes(), managed);
        }
        report("parked_source_begin", &begin_samples);
        report("native_scoped_callback_and_seal", &callback_samples);
        report("same_original_source_resume", &resume_samples);
        assert_eq!(original.nodes().count(), managed);
        assert_eq!(
            attribute(&original, fixture.target),
            DocumentString::from("base")
        );
        let start = Instant::now();
        drop(executor);
        drop(fixture);
        eprintln!(
            "scale continuation_owner_drop_held_snapshot_ns={}",
            start.elapsed().as_nanos()
        );
        let start = Instant::now();
        drop(original);
        eprintln!(
            "scale continuation_snapshot_last_drop_ns={}",
            start.elapsed().as_nanos()
        );
    }
}

/// Fixed public topology/row count, one unrelated UTF-16 field with equal
/// one-code-unit semantics and independently varied actual spare backing.
#[test]
#[ignore = "explicit release fixed unrelated UTF16 backing-capacity sweep"]
fn fixed_unrelated_spare_utf16_backing_scale() {
    require_release();
    for managed in selected_sizes() {
        let mut fixture = fixture(managed, Context::Detached);
        let unrelated = fixture
            .dom
            .snapshot()
            .node(fixture.other_document.node())
            .unwrap()
            .children()[0];
        for capacity in [32, 4096, 65_536] {
            let mut backing = Vec::with_capacity(capacity);
            backing.push(0xd800);
            let value = DocumentString::from_utf16(backing);
            assert_eq!(value.shared_code_units().capacity(), capacity);
            // Equal authored values deliberately preserve old backing. End
            // that field occurrence before installing the next actual extent;
            // both public setup operations stay outside measurement.
            let mut remove = DomBatch::new();
            remove.remove_attribute(unrelated, "data-spare");
            apply(&mut fixture.dom, remove);
            let mut batch = DomBatch::new();
            batch.set_attribute(unrelated, "data-spare", value);
            apply(&mut fixture.dom, batch);
            let original = fixture.dom.snapshot();
            let retained = original
                .node(unrelated)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-spare".into())
                .unwrap();
            assert_eq!(retained.code_units(), &[0xd800]);
            assert_eq!(retained.shared_code_units().capacity(), capacity);
            eprintln!(
                "scale spare_utf16 managed={managed} capacity_code_units={capacity} logical_code_units=1 unrelated_rows=1 usage={:?} limits={:?}",
                fixture.dom.usage(),
                fixture.dom.limits()
            );
            for edit in [
                Edit::ChangedAttribute,
                Edit::ChangedText,
                Edit::SourceAttribute,
            ] {
                let mut reset = DomBatch::new();
                reset.set_attribute(fixture.target, "data-local", "base");
                reset.replace_data(fixture.text, 0, usize::MAX, "base");
                apply(&mut fixture.dom, reset);
                let mut samples = Vec::new();
                for sample in 0..WARMUPS + SAMPLES {
                    let elapsed = measure_edit(&mut fixture, edit, sample);
                    if sample >= WARMUPS {
                        samples.push(elapsed);
                    }
                }
                report(
                    &format!("spare_utf16_{edit:?}_S{managed}_capacity{capacity}"),
                    &samples,
                );
            }
            let current = fixture.dom.snapshot();
            for snapshot in [&original, &current] {
                let retained = snapshot
                    .node(unrelated)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attribute(&"data-spare".into())
                    .unwrap();
                assert_eq!(retained.code_units(), &[0xd800]);
                assert_eq!(retained.shared_code_units().capacity(), capacity);
            }
        }
    }
}

/// Observe the real existing Draft, full input admission, and exact same-owner
/// checkpoint/reentry reuse. Extents are fixture inputs, never measured bounds.
fn measure_input_capture(
    draft: &mut super::Draft,
    snapshot: &DomSnapshot,
    label: &str,
    destination_managed: usize,
    input_managed: usize,
    missing: bool,
    cancel: &AtomicBool,
) {
    let extent = destination_managed.max(input_managed);
    let depth = usize::BITS as usize - extent.leading_zeros() as usize;
    let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
    if measurement.is_some() {
        // This interval begins after the real Draft already exists.
        // Seed the selected observer with its actual numerical union
        // and original external envelope; retain no additional lease.
        let ledger = draft.custody_ledger();
        scale_metrics::select_source_ledger(
            ledger.measurement_identity(),
            ledger.total(),
            super::custody::Usage::new(draft.source_other_bytes, draft.source_other_units),
        );
    }
    let before = draft.source_work.load(std::sync::atomic::Ordering::Relaxed);
    let started = Instant::now();
    let capsule = draft.retain_input_capture(&snapshot.state, cancel).unwrap();
    let admission = started.elapsed();
    let spent = draft.source_work.load(std::sync::atomic::Ordering::Relaxed) - before;
    if missing {
        assert!(
            spent >= input_managed as u64,
            "whole disjoint input must qualify its records"
        );
    } else {
        assert!(
            spent <= (128 * depth) as u64,
            "{label}: spent={spent} depth={depth}"
        );
    }
    if let Some(measurement) = measurement {
        let actual = draft
            .custody_ledger()
            .total()
            .checked_add(super::custody::Usage::new(
                draft.source_other_bytes,
                draft.source_other_units,
            ))
            .unwrap();
        let counters = measurement
            .finish(
                Hooks::one(Domain::Bank)
                    .union(Hooks::one(Domain::Accounting))
                    .union(Hooks::one(Domain::Source)),
            )
            .unwrap();
        assert!(
            counters.source_live_bytes > 0 && counters.source_live_units > 0,
            "{counters:?}"
        );
        assert!(
            counters.source_peak_bytes > 0 && counters.source_peak_units > 0,
            "{counters:?}"
        );
        assert_eq!(counters.source_live_bytes, actual.bytes(), "{counters:?}");
        assert_eq!(counters.source_live_units, actual.units(), "{counters:?}");
        assert_eq!(
            counters.source_steps,
            usize::try_from(spent).unwrap(),
            "{counters:?}"
        );
        let queue_admin = counters
            .queue_record_visits
            .checked_add(counters.queue_records_moved)
            .expect("measured queue work");
        assert!(queue_admin <= 128 * depth, "{counters:?}");
        if !missing {
            assert!(
                counters.accounting_record_visits <= 128 * depth,
                "{counters:?}"
            );
        }
        eprintln!(
            "scale input_capture label={label} destination_managed={destination_managed} input_managed={input_managed} admission_ns={} spent={spent} counters={counters:?}",
            admission.as_nanos()
        );
    } else {
        eprintln!(
            "scale input_capture label={label} destination_managed={destination_managed} input_managed={input_managed} admission_ns={} spent={spent}",
            admission.as_nanos()
        );
    }
    // Real nested savepoint operations must not evict or reacquire the
    // original admitted input. Their own headers are outside this
    // repeat's before/after custody and work observations.
    let checkpoint = draft.source_checkpoint(0, 0, cancel).unwrap();
    let usage = draft.custody_ledger().total();
    let work = draft.source_work.load(std::sync::atomic::Ordering::Relaxed);
    let repeated = draft.retain_input_capture(&snapshot.state, cancel).unwrap();
    assert_eq!(draft.custody_ledger().total(), usage);
    assert_eq!(
        draft.source_work.load(std::sync::atomic::Ordering::Relaxed),
        work
    );
    checkpoint.restore(draft);
    let usage = draft.custody_ledger().total();
    let work = draft.source_work.load(std::sync::atomic::Ordering::Relaxed);
    let resumed = draft.retain_input_capture(&snapshot.state, cancel).unwrap();
    assert_eq!(draft.custody_ledger().total(), usage);
    assert_eq!(
        draft.source_work.load(std::sync::atomic::Ordering::Relaxed),
        work
    );
    drop((capsule, repeated, resumed));
}

/// The production InputCapture entry point is private ownership machinery.
/// These real public stores retain the selected full canonical extent while
/// measuring current-root and small historical-frontier admission/reuse.
#[test]
#[ignore = "explicit release actual large InputCapture historical frontier and reuse"]
fn historical_input_capture_frontier_and_reuse_scale() {
    require_release();
    let cancel = AtomicBool::new(false);
    for managed in selected_sizes() {
        let mut fixture = fixture(managed, Context::Detached);
        let historical = fixture.dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(fixture.target, "data-local", "after");
        apply(&mut fixture.dom, batch);
        let policy = source_limits();
        eprintln!(
            "scale input_capture fixture destination_managed={managed} input_managed={managed} shared_historical_owner=true source_limits={policy:?} canonical_limits={:?}",
            fixture.dom.limits()
        );
        let empty = DomBatch::new();
        let mut draft = super::Draft::new_source(
            &fixture.dom,
            empty.owner.clone(),
            &empty.commands,
            policy,
            &cancel,
        )
        .unwrap()
        .0;
        for (label, snapshot) in [
            ("current_same_root", fixture.dom.snapshot()),
            ("historical_small_frontier", historical.clone()),
        ] {
            measure_input_capture(
                &mut draft, &snapshot, label, managed, managed, false, &cancel,
            );
        }
        assert_eq!(
            attribute(&historical, fixture.target),
            DocumentString::from("base")
        );
        assert_eq!(
            attribute(&fixture.dom.snapshot(), fixture.target),
            DocumentString::from("after")
        );
        drop(draft);
        assert_eq!(historical.nodes().count(), managed);
    }
}

/// A real million-node external owner supplies the complete input snapshot.
/// That complete snapshot is admitted into the genuine small destination;
/// shallow import size cannot replace it.
/// This fixed-size companion runs once per profile, independently of SIZES.
#[test]
#[ignore = "explicit release actual disjoint million-node InputCapture and reuse"]
fn large_disjoint_input_capture_and_reuse_scale() {
    require_release();
    const DESTINATION_MANAGED: usize = 1000;
    let input_managed = std::env::var("SURGEIST_SCALE_EXTERNAL_SIZE")
        .map(|value| value.parse::<usize>().expect("external input node count"))
        .unwrap_or(1_000_000);
    assert!(
        [1000, 1_000_000].contains(&input_managed),
        "explicit pilot1000 or required1000000 external nodes"
    );
    let cancel = AtomicBool::new(false);
    // Public admitted construction creates every external canonical record.
    // Ending the owner leaves an ordinary immutable snapshot with no live
    // accounting lease; its missing frontier must genuinely be qualified.
    let disjoint = fixture(input_managed, Context::Detached);
    let external = disjoint.dom.snapshot();
    drop(disjoint);
    let fixture = fixture(DESTINATION_MANAGED, Context::Detached);
    let destination = fixture.dom.snapshot();
    let allowances = source_limits();
    let policy = DomSourceLimits::new(
        allowances.retained_bytes(),
        allowances.retained_string_code_units(),
        input_managed as u64 * 128 + 100_000,
        64,
        4096,
        4096,
        1,
    );
    eprintln!(
        "scale input_capture fixture destination_managed={DESTINATION_MANAGED} input_managed={input_managed} disjoint_owners=2 external_full_default=1000000 source_limits={policy:?} canonical_limits={:?}",
        fixture.dom.limits()
    );
    let empty = DomBatch::new();
    let mut draft = super::Draft::new_source(
        &fixture.dom,
        empty.owner.clone(),
        &empty.commands,
        policy,
        &cancel,
    )
    .unwrap()
    .0;
    measure_input_capture(
        &mut draft,
        &external,
        "large_disjoint",
        DESTINATION_MANAGED,
        input_managed,
        true,
        &cancel,
    );
    assert_eq!(external.nodes().count(), input_managed);
    assert_eq!(destination.nodes().count(), DESTINATION_MANAGED);
    assert_eq!(fixture.dom.snapshot().stamp(), destination.stamp());
    assert_eq!(
        attribute(&destination, fixture.target),
        DocumentString::from("base")
    );
}

const OUTER_ROWS: [usize; 3] = [64, 1024, 16_384];

fn selected_outer_rows() -> Vec<usize> {
    let Some(value) = std::env::var_os("SURGEIST_SCALE_OUTER_ROWS") else {
        return OUTER_ROWS.to_vec();
    };
    let mut rows = Vec::new();
    for item in value
        .to_str()
        .expect("UTF-8 outer row selection")
        .split(',')
    {
        let count: usize = item.parse().expect("comma-separated outer row counts");
        assert!(
            OUTER_ROWS.contains(&count),
            "unsupported outer row count {count}"
        );
        assert!(!rows.contains(&count), "duplicate outer row count {count}");
        rows.push(count);
    }
    assert!(!rows.is_empty());
    rows
}

/// Measure one real private producer call. Output predicates and teardown are
/// outside the mutation measurement. These inputs have at most one selected
/// semantic child item; unrelated outer rows are never a semantic participant.
fn measure_outer<T>(
    label: &str,
    rows: usize,
    semantic_items: usize,
    work: &crate::hosted::Work,
    mut mutate: impl FnMut() -> T,
    mut verify: impl FnMut(&T),
) {
    let mut samples = Vec::new();
    let depth = usize::BITS as usize - (rows * 2 + 32).leading_zeros() as usize;
    let admin = 512 * depth;
    if rows == *OUTER_ROWS.last().unwrap() {
        assert!(
            rows > admin,
            "a full unrelated-row scan must exceed the bound"
        );
    }
    for sample in 0..WARMUPS + SAMPLES {
        let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
        let before = work.spent.load(std::sync::atomic::Ordering::Relaxed);
        let started = Instant::now();
        let candidate = mutate();
        let elapsed = started.elapsed();
        let spent = work.spent.load(std::sync::atomic::Ordering::Relaxed) - before;
        assert!(
            spent > 0 && spent <= admin as u64,
            "{label}: spent={spent}, limit={admin}"
        );
        if let Some(measurement) = measurement {
            // These are producer-only calls, not transaction validation or a
            // canonical Source publication. Require the actually owning Bank.
            let counters = measurement.finish(Hooks::one(Domain::Bank)).unwrap();
            // Inserting into an empty sparse index can allocate the missing
            // path without visiting an existing page; a dense append can
            // visit its tail directly without a lookup. Require one actual
            // Bank path observation and retain both independent ceilings.
            assert!(counters.bank_page_visits <= admin, "{counters:?}");
            assert!(counters.bank_lookups <= admin, "{counters:?}");
            assert!(
                counters.bank_page_visits > 0 || counters.bank_lookups > 0,
                "{counters:?}"
            );
            assert!(counters.canonical_record_reads <= admin, "{counters:?}");
            assert!(counters.accounting_record_visits <= admin, "{counters:?}");
            assert!(counters.record_materializations <= admin, "{counters:?}");
            assert!(
                counters.structural_bytes > 0 && counters.structural_bytes <= 131_072 * depth,
                "{counters:?}"
            );
            assert!(counters.sequence_materialized_bytes <= 4096, "{counters:?}");
            let queue_admin = counters
                .queue_record_visits
                .checked_add(counters.queue_records_moved)
                .expect("measured queue work");
            assert!(queue_admin <= admin, "{counters:?}");
            assert_eq!(
                counters.hosted_semantic_visits, semantic_items,
                "{label}: {counters:?}"
            );
            assert_eq!(
                counters.published_roots, 0,
                "private producer output is not canonical publication"
            );
            eprintln!(
                "scale outer_execution label={label} unrelated_rows={rows} sample={sample} actual_hosted_work={spent} selected_semantic_items={semantic_items} admin_limit={admin} counters={counters:?}"
            );
        } else {
            eprintln!(
                "scale outer_execution label={label} unrelated_rows={rows} sample={sample} actual_hosted_work={spent} selected_semantic_items={semantic_items} counters=disabled"
            );
        }
        verify(&candidate);
        let started = Instant::now();
        drop(candidate);
        eprintln!(
            "scale outer_candidate_drop label={label} sample={sample} elapsed_ns={}",
            started.elapsed().as_nanos()
        );
        if sample >= WARMUPS {
            samples.push(elapsed);
        }
    }
    report(&format!("outer_{label}_unrelated{rows}"), &samples);
}

/// Actual DOM consuming boundary for a historical HOST-only input. The old
/// semantic/capsule pair is controlled before measurement; a genuine producer
/// has formed the one changed frontier outside this DOM-admission interval.
/// None is the real historical-input protocol, never a fabricated fresh receipt.
fn measure_dom_aux<T, C>(
    label: &str,
    rows: usize,
    dom: &Dom,
    old: &Arc<T>,
    candidate: &Arc<T>,
    mut prepare: impl FnMut(Arc<T>, &super::Draft) -> (Arc<T>, C),
    mut verify: impl FnMut(&Arc<T>),
) {
    let cancel = AtomicBool::new(false);
    let limits = source_limits();
    let limits = DomSourceLimits::new(
        limits.retained_bytes(),
        limits.retained_string_code_units(),
        u64::try_from(rows)
            .unwrap()
            .checked_mul(4096)
            .unwrap()
            .checked_add(1_000_000)
            .unwrap(),
        64,
        4096,
        4096,
        1,
    );
    let commands = DomBatch::new();
    let started = Instant::now();
    let draft = super::Draft::new_source(
        dom,
        commands.owner.clone(),
        &commands.commands,
        limits,
        &cancel,
    )
    .unwrap()
    .0;
    let old_pair = prepare(old.clone(), &draft);
    assert!(Arc::ptr_eq(&old_pair.0, old));
    draft.custody_registry().reconcile_releases();
    eprintln!(
        "scale dom_aux_setup label={label} unrelated_rows={rows} input_protocol=historical_HOST_None source_phase_publication=false prepare_old_ns={} source_limits={limits:?} actual_controlled_usage={:?}",
        started.elapsed().as_nanos(),
        draft.custody_ledger().total()
    );
    let depth = usize::BITS as usize - (rows * 2 + 32).leading_zeros() as usize;
    let admin = 512 * depth;
    if rows == *OUTER_ROWS.last().unwrap() {
        assert!(rows > admin);
    }
    let mut samples = Vec::new();
    for sample in 0..WARMUPS + SAMPLES {
        draft.custody_registry().reconcile_releases();
        let baseline = draft.custody_ledger().total();
        let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
        if measurement.is_some() {
            scale_metrics::select_source_ledger(
                draft.custody_ledger().measurement_identity(),
                baseline,
                super::custody::Usage::new(draft.source_other_bytes, draft.source_other_units),
            );
        }
        let before = draft.source_work.load(std::sync::atomic::Ordering::Relaxed);
        let started = Instant::now();
        let prepared = prepare(candidate.clone(), &draft);
        let elapsed = started.elapsed();
        let spent = draft.source_work.load(std::sync::atomic::Ordering::Relaxed) - before;
        let controlled = draft.custody_ledger().total();
        let actual = controlled
            .checked_add(super::custody::Usage::new(
                draft.source_other_bytes,
                draft.source_other_units,
            ))
            .unwrap();
        assert!(
            spent > 0 && spent <= admin as u64,
            "{label}: spent={spent}, limit={admin}"
        );
        assert!(
            controlled.bytes() > baseline.bytes(),
            "a genuinely missing changed frontier joins the retained old control"
        );
        if let Some(measurement) = measurement {
            let counters = measurement
                .finish(Hooks::one(Domain::Accounting).union(Hooks::one(Domain::Source)))
                .unwrap();
            assert!(
                counters.bank_page_visits > 0 && counters.bank_page_visits <= admin,
                "{counters:?}"
            );
            assert!(counters.bank_lookups <= admin, "{counters:?}");
            assert!(counters.canonical_record_reads <= admin, "{counters:?}");
            assert!(
                counters.accounting_record_visits > 0 && counters.accounting_record_visits <= admin,
                "{counters:?}"
            );
            assert_eq!(
                counters.record_materializations, 0,
                "admission aliases the original semantic materializations"
            );
            assert!(
                counters.structural_bytes > 0 && counters.structural_bytes <= 131_072 * depth,
                "{counters:?}"
            );
            assert_eq!(
                counters.source_steps,
                usize::try_from(spent).unwrap(),
                "{counters:?}"
            );
            assert_eq!(counters.source_live_bytes, actual.bytes(), "{counters:?}");
            assert_eq!(counters.source_live_units, actual.units(), "{counters:?}");
            assert!(
                counters.source_peak_bytes >= actual.bytes() && counters.source_peak_bytes > 0,
                "{counters:?}"
            );
            assert_eq!(
                counters.hosted_semantic_visits, 0,
                "DOM adoption has no Graph semantic traversal"
            );
            let queue_admin = counters
                .queue_record_visits
                .checked_add(counters.queue_records_moved)
                .expect("measured queue work");
            assert!(queue_admin <= admin, "{counters:?}");
            assert_eq!(counters.published_roots, 0);
            eprintln!(
                "scale dom_aux_execution label={label} unrelated_rows={rows} sample={sample} actual_dom_source_work={spent} admin_limit={admin} counters={counters:?}"
            );
        } else {
            eprintln!(
                "scale dom_aux_execution label={label} unrelated_rows={rows} sample={sample} actual_dom_source_work={spent} retained_old_and_new_usage={controlled:?} counters=disabled"
            );
        }
        assert!(Arc::ptr_eq(&prepared.0, candidate));
        verify(&prepared.0);
        let started = Instant::now();
        drop(prepared);
        draft.custody_registry().reconcile_releases();
        assert_eq!(
            draft.custody_ledger().total(),
            baseline,
            "only the changed frontier's custody ends"
        );
        eprintln!(
            "scale dom_aux_candidate_release label={label} sample={sample} elapsed_ns={}",
            started.elapsed().as_nanos()
        );
        if sample >= WARMUPS {
            samples.push(elapsed);
        }
    }
    report(&format!("dom_aux_{label}_unrelated{rows}"), &samples);
    let started = Instant::now();
    drop(old_pair);
    drop(draft);
    dom.custody_registry.reconcile_releases();
    eprintln!(
        "scale dom_aux_old_scope_teardown label={label} unrelated_rows={rows} elapsed_ns={}",
        started.elapsed().as_nanos()
    );
}

/// Separate HOST mutation and DOM historical auxiliary-admission intervals,
/// with real issuer capabilities and native Document payload IDs from the
/// existing owning fixtures. These calls do not publish
/// these graphs as hosted Documents, attest host execution, or claim a million
/// outer rows. The public canonical million fixtures remain separate evidence.
/// SURGEIST_SCALE_SIZES does not select this fixed outer-row ladder.
#[test]
#[ignore = "explicit release real outer History/Global/registry mutation at selected row counts"]
fn hosted_outer_bank_local_mutation_scale() {
    use super::hosted::GlobalDocumentIndex;
    use super::hosted::history::{Context as HistoryContext, Graph};
    let cancel = AtomicBool::new(false);
    require_release();
    eprintln!(
        "scale outer_environment profile=release rows={:?} default_rows={OUTER_ROWS:?} counters_enabled={} warmups={WARMUPS} samples={SAMPLES} canonical_million_fixture=false issuer=genuine_native_payload_fixture os={} arch={} revision={} toolchain={} machine={}",
        selected_outer_rows(),
        scale_metrics::ENABLED,
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
        std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into())
    );
    for rows in selected_outer_rows() {
        // History stores insertion ordinals densely. Genuine unused issued IDs
        // leave holes in the qualified identity index, not fabricated ordinals.
        let started = Instant::now();
        let mut fixture = super::hosted::history::tests::Fixture::new_for_outer_rows(rows * 2 + 32);
        let (top, _, top_work) = fixture.issue(0);
        let mut graph = Graph::initial(
            None,
            fixture.input(top, None),
            &HistoryContext::new(&fixture.source, &top_work, &cancel, 0),
        )
        .unwrap();
        drop(top_work);
        for _ in 0..rows {
            let (_, _, skipped_work) = fixture.issue(2);
            drop(skipped_work);
            let (creation, _, work) = fixture.issue(2);
            graph = Graph::initial(
                Some(&graph),
                fixture.input(creation, None),
                &HistoryContext::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap();
        }
        let (child, operation, work) = fixture.issue(1);
        graph = Graph::initial(
            Some(&graph),
            fixture.input(child, Some(top)),
            &HistoryContext::new(&fixture.source, &work, &cancel, 0),
        )
        .unwrap();
        graph = graph
            .mark_creation_queued(
                child.navigable,
                operation,
                &HistoryContext::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap();
        graph = graph
            .append_creation(
                child.navigable,
                operation,
                &HistoryContext::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap();
        assert_eq!(graph.navigable_bank().len(), rows + 2);
        assert_eq!(graph.entry_bank().len(), rows + 2);
        assert_eq!(
            graph.entry(top.entry).unwrap().nested.values,
            [child.navigable]
        );
        let captured = graph.clone();
        let canonical = fixture.dom.snapshot();
        eprintln!(
            "scale outer_setup owner=History unrelated_rows={rows} navigables={} entries={} qualified_identity_holes_per_role={rows} selected_parent_nested=1 setup_ns={} hosted_limits={:?} actual_producer_bytes={} native_documents={} canonical_stamp={:?}",
            graph.navigable_bank().len(),
            graph.entry_bank().len(),
            started.elapsed().as_nanos(),
            fixture.source.limits(),
            fixture.source.limits().retained_bytes() - fixture.source.remaining_bytes().unwrap(),
            canonical.nodes().count(),
            canonical.stamp()
        );
        measure_outer(
            "history_nav_retiring",
            rows,
            0,
            &work,
            || {
                graph
                    .mark_retiring(
                        child.navigable,
                        &HistoryContext::new(&fixture.source, &work, &cancel, 0),
                    )
                    .unwrap()
            },
            |candidate| {
                assert!(candidate.navigable(child.navigable).unwrap().retiring);
                assert!(!captured.navigable(child.navigable).unwrap().retiring);
                assert_eq!(
                    candidate.navigable(child.navigable).unwrap().id,
                    child.navigable
                );
                assert_eq!(
                    candidate.entry(child.entry).unwrap().document,
                    Some(child.document)
                );
            },
        );
        // Follow the production removal order before measuring unlink: the
        // original graph owns the retired-in-progress child and one nested ID.
        let unlink_base = graph
            .mark_retiring(
                child.navigable,
                &HistoryContext::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap();
        measure_outer(
            "history_entry_unlink",
            rows,
            1,
            &work,
            || {
                unlink_base
                    .unlink_child(
                        child.navigable,
                        top.navigable,
                        &HistoryContext::new(&fixture.source, &work, &cancel, 0),
                    )
                    .unwrap()
            },
            |candidate| {
                let changed = candidate.navigable(child.navigable).unwrap();
                assert!(changed.retiring);
                assert_eq!(changed.parent, None);
                assert_eq!(changed.former_parent, Some(top.navigable));
                assert!(candidate.entry(top.entry).unwrap().nested.values.is_empty());
                assert_eq!(
                    captured.navigable(child.navigable).unwrap().parent,
                    Some(top.navigable)
                );
                assert_eq!(
                    captured.entry(top.entry).unwrap().nested.values,
                    [child.navigable]
                );
                assert_eq!(candidate.entry(child.entry).unwrap().id, child.entry);
                assert_eq!(
                    candidate.entry(child.entry).unwrap().document,
                    Some(child.document)
                );
            },
        );
        measure_dom_aux(
            "history_nav_frontier",
            rows,
            &fixture.dom,
            &graph,
            &unlink_base,
            |semantic, draft| {
                super::transaction::auxiliary::prepare_history(
                    semantic,
                    None,
                    draft.custody_registry(),
                    &mut |usage| draft.admit_custody_usage(usage, &cancel),
                    &mut || draft.source_step(&cancel),
                )
                .unwrap()
                .into_parts()
            },
            |candidate| {
                assert!(candidate.navigable(child.navigable).unwrap().retiring);
                assert!(!captured.navigable(child.navigable).unwrap().retiring);
                assert_eq!(
                    candidate.entry(child.entry).unwrap().document,
                    Some(child.document)
                );
            },
        );
        let unlinked = unlink_base
            .unlink_child(
                child.navigable,
                top.navigable,
                &HistoryContext::new(&fixture.source, &work, &cancel, 0),
            )
            .unwrap();
        measure_dom_aux(
            "history_entry_frontier",
            rows,
            &fixture.dom,
            &unlink_base,
            &unlinked,
            |semantic, draft| {
                super::transaction::auxiliary::prepare_history(
                    semantic,
                    None,
                    draft.custody_registry(),
                    &mut |usage| draft.admit_custody_usage(usage, &cancel),
                    &mut || draft.source_step(&cancel),
                )
                .unwrap()
                .into_parts()
            },
            |candidate| {
                assert!(candidate.navigable(child.navigable).unwrap().retiring);
                assert_eq!(candidate.navigable(child.navigable).unwrap().parent, None);
                assert!(candidate.entry(top.entry).unwrap().nested.values.is_empty());
                assert_eq!(
                    unlink_base.entry(top.entry).unwrap().nested.values,
                    [child.navigable]
                );
                assert_eq!(candidate.entry(child.entry).unwrap().id, child.entry);
            },
        );
        assert_eq!(fixture.dom.snapshot().stamp(), canonical.stamp());
        drop((graph, unlink_base, unlinked, captured, work, fixture));

        let started = Instant::now();
        let mut fixture = super::hosted::index_tests::Fixture::new_for_outer_rows(rows * 2 + 32);
        let (first, first_work) = fixture.global();
        let mut dense = GlobalDocumentIndex::establish(
            None,
            first,
            fixture.documents[0],
            &fixture.source,
            &first_work,
            None,
            &cancel,
        )
        .unwrap();
        drop(first_work);
        for _ in 1..rows {
            let (_, skipped_work) = fixture.global();
            drop(skipped_work);
            let (global, work) = fixture.global();
            dense = GlobalDocumentIndex::establish(
                Some(&dense),
                global,
                fixture.documents[0],
                &fixture.source,
                &work,
                None,
                &cancel,
            )
            .unwrap();
        }
        let (last, work) = fixture.global();
        assert_eq!(last.slot, rows * 2 - 1);
        let captured = dense.clone();
        let canonical = fixture.dom.snapshot();
        eprintln!(
            "scale outer_setup owner=Global unrelated_rows={rows} retained_associations={rows} issued_global_extent={} sparse_candidate_missing_keys={} setup_ns={} hosted_limits={:?} actual_producer_bytes={} native_documents={} canonical_stamp={:?}",
            last.slot + 1,
            last.slot,
            started.elapsed().as_nanos(),
            fixture.source.limits(),
            fixture.source.limits().retained_bytes() - fixture.source.remaining_bytes().unwrap(),
            canonical.nodes().count(),
            canonical.stamp()
        );
        measure_outer(
            "global_dense_append",
            rows,
            0,
            &work,
            || {
                GlobalDocumentIndex::establish(
                    Some(&dense),
                    last,
                    fixture.documents[1],
                    &fixture.source,
                    &work,
                    None,
                    &cancel,
                )
                .unwrap()
            },
            |candidate| {
                assert_eq!(candidate.document(first).unwrap(), fixture.documents[0]);
                assert_eq!(candidate.document(last).unwrap(), fixture.documents[1]);
                assert!(captured.document(last).is_err());
                assert_eq!(captured.document(first).unwrap(), fixture.documents[0]);
            },
        );
        measure_outer(
            "global_sparse_issued_extent",
            rows,
            0,
            &work,
            || {
                GlobalDocumentIndex::establish(
                    None,
                    last,
                    fixture.documents[1],
                    &fixture.source,
                    &work,
                    None,
                    &cancel,
                )
                .unwrap()
            },
            |candidate| {
                assert_eq!(candidate.document(last).unwrap(), fixture.documents[1]);
                assert!(candidate.document(first).is_err());
                assert_eq!(captured.document(first).unwrap(), fixture.documents[0]);
            },
        );
        let extended = GlobalDocumentIndex::establish(
            Some(&dense),
            last,
            fixture.documents[1],
            &fixture.source,
            &work,
            None,
            &cancel,
        )
        .unwrap();
        measure_dom_aux(
            "global_association_frontier",
            rows,
            &fixture.dom,
            &dense,
            &extended,
            |semantic, draft| {
                super::transaction::auxiliary::prepare_globals(
                    semantic,
                    None,
                    draft.custody_registry(),
                    &mut |usage| draft.admit_custody_usage(usage, &cancel),
                    &mut || draft.source_step(&cancel),
                )
                .unwrap()
                .into_parts()
            },
            |candidate| {
                assert_eq!(candidate.document(first).unwrap(), fixture.documents[0]);
                assert_eq!(candidate.document(last).unwrap(), fixture.documents[1]);
                assert!(captured.document(last).is_err());
            },
        );
        assert_eq!(fixture.dom.snapshot().stamp(), canonical.stamp());
        drop((dense, extended, captured, work, fixture));

        let started = Instant::now();
        let fixture = super::hosted::index_tests::Fixture::new_for_outer_rows(rows + 32);
        let work = fixture.source.work(&cancel).unwrap();
        let (mut bank, first) = super::registry::Bank::form(
            None,
            fixture.dom.owner,
            &fixture.source,
            &work,
            None,
            &cancel,
        )
        .unwrap();
        for _ in 1..rows {
            bank = super::registry::Bank::form(
                Some(&bank),
                fixture.dom.owner,
                &fixture.source,
                &work,
                None,
                &cancel,
            )
            .unwrap()
            .0;
        }
        let first_capture = bank.get(first).unwrap().capture.identity().clone();
        let captured = bank.clone();
        let canonical = fixture.dom.snapshot();
        assert_eq!(bank.custody_bank().len(), rows);
        eprintln!(
            "scale outer_setup owner=Registry unrelated_rows={rows} retained_registries={rows} selected_registry=complete_empty_global setup_ns={} hosted_limits={:?} actual_producer_bytes={} native_documents={} canonical_stamp={:?}",
            started.elapsed().as_nanos(),
            fixture.source.limits(),
            fixture.source.limits().retained_bytes() - fixture.source.remaining_bytes().unwrap(),
            canonical.nodes().count(),
            canonical.stamp()
        );
        measure_outer(
            "registry_append",
            rows,
            0,
            &work,
            || {
                super::registry::Bank::form(
                    Some(&bank),
                    fixture.dom.owner,
                    &fixture.source,
                    &work,
                    None,
                    &cancel,
                )
                .unwrap()
            },
            |(candidate, id)| {
                assert_eq!(id.slot, rows);
                assert_eq!(id.generation, 1);
                assert_eq!(id.owner, fixture.dom.owner);
                assert_eq!(
                    candidate.get(first).unwrap().capture.identity(),
                    &first_capture
                );
                assert!(candidate.get(*id).is_ok());
                assert!(captured.get(*id).is_err());
                assert_eq!(
                    captured.get(first).unwrap().capture.identity(),
                    &first_capture
                );
            },
        );
        let (extended, id) = super::registry::Bank::form(
            Some(&bank),
            fixture.dom.owner,
            &fixture.source,
            &work,
            None,
            &cancel,
        )
        .unwrap();
        measure_dom_aux(
            "registry_frontier",
            rows,
            &fixture.dom,
            &bank,
            &extended,
            |semantic, draft| {
                super::transaction::auxiliary::prepare_registry(
                    semantic,
                    None,
                    draft.custody_registry(),
                    &mut |usage| draft.admit_custody_usage(usage, &cancel),
                    &mut || draft.source_step(&cancel),
                )
                .unwrap()
                .into_parts()
            },
            |candidate| {
                assert_eq!(id.slot, rows);
                assert_eq!(
                    candidate.get(first).unwrap().capture.identity(),
                    &first_capture
                );
                assert!(candidate.get(id).is_ok());
                assert!(captured.get(id).is_err());
            },
        );
        assert_eq!(fixture.dom.snapshot().stamp(), canonical.stamp());
        drop((bank, extended, captured, work, fixture));
    }
}

// Fixed Style path: three native creations, seven tree executions across the
// first CSP wait, execution-and-restore replay and admitted continuation, plus
// at most four acyclic Style cursor invocations. This bounds real forward and
// inverse journal/effect/arena/output writes, not only backing growth. The
// source/operation.rs text/sheet-order captures and one headroom row are fixed.
fn style_sequence_bounds() -> (usize, usize) {
    let nodes = 3;
    let trees = 7;
    let styles = 4;
    let participants = bulk_add(bulk_add(nodes, trees), styles);
    let frames = bulk_add(bulk_mul(32, trees), bulk_mul(16, styles));
    let journal = bulk_add(bulk_mul(2, frames), bulk_mul(32, participants));
    let native_units = bulk_add(
        super::NATIVE_DOCUMENT_STRING_UNITS,
        bulk_add(28, bulk_add(5, 4)),
    );
    let terms = [
        bulk_mul(
            bulk_add(bulk_mul(3, journal), 24),
            size_of::<super::transaction::Undo>(),
        ),
        bulk_mul(
            bulk_mul(3, participants),
            size_of::<Option<super::DraftNode>>(),
        ),
        bulk_mul(bulk_mul(3, frames), size_of::<super::select::EffectFrame>()),
        bulk_mul(
            bulk_mul(4, participants),
            bulk_add(size_of::<super::Key>(), size_of::<NodeId>()),
        ),
        bulk_mul(
            bulk_mul(8, participants),
            bulk_add(size_of::<super::Key>(), size_of::<DomChange>()),
        ),
        bulk_mul(
            participants,
            [
                size_of::<super::Key>(),
                size_of::<Option<NodeId>>(),
                size_of::<bool>(),
                size_of::<NodeId>(),
                size_of::<usize>(),
                size_of::<DomChange>(),
            ]
            .into_iter()
            .fold(0, bulk_add),
        ),
        bulk_mul(4, size_of::<super::history::HistoryEntry>()),
        bulk_mul(24, size_of::<(u64, usize)>()),
        bulk_mul(bulk_mul(4, native_units), size_of::<u16>()),
        bulk_mul(2, size_of::<crate::DomStyleSheetPlacement>()),
        size_of::<(super::Key, u64, u64)>(),
    ];
    let bound = terms.into_iter().fold(0, bulk_add);
    // Four empty Children appends and at most four singleton projections.
    // No shared-prefix/tail or inverse relation movement is required here.
    let relation = bulk_mul(4, bulk_add(size_of::<super::Key>(), size_of::<NodeId>()));
    eprintln!(
        "scale style_sequence_model nodes={nodes} trees={trees} styles={styles} participants={participants} frames={frames} journal={journal} native_units={native_units} formula=(3J+24)_Undo+3p_OptionDraftNode+3F_EffectFrame+4p_(Key+NodeId)+8p_(Key+DomChange)+p_(Key+OptionNodeId+bool+NodeId+usize+DomChange)+4_HistoryEntry+24_Marker+4_native_u16+2_StyleSheetPlacement+1_HeadroomRow bound_bytes={bound} positive_relation_bytes_bound={relation}"
    );
    (bound, relation)
}

/// Execute a genuine reached headroom pass after public population, and finish
/// with actual CSP and CSSOM producer observations. This complements the Show
/// callback/no-op continuation; it does not invent a wait for that method.
#[test]
#[ignore = "explicit release real Source Style headroom and completion after public population"]
fn postsetup_source_style_headroom_completion_scale() {
    require_release();
    let (sequence_bound, relation_bound) = style_sequence_bounds();
    for managed in selected_sizes() {
        let mut samples = Vec::new();
        for sample in 0..WARMUPS + SAMPLES {
            let mut fixture = fixture_with_capacity(managed, Context::Detached, 3);
            let before = fixture.dom.snapshot();
            eprintln!(
                "scale style_headroom_setup managed={managed} sample={sample} setup_ns={} canonical_limits={:?} source_work_limit=100000000 revision={} toolchain={} machine={} counters_enabled={}",
                fixture.setup.as_nanos(),
                fixture.dom.limits(),
                std::env::var("SURGEIST_SCALE_REVISION").unwrap_or_else(|_| "unknown".into()),
                std::env::var("SURGEIST_SCALE_TOOLCHAIN").unwrap_or_else(|_| "unknown".into()),
                std::env::var("SURGEIST_SCALE_MACHINE").unwrap_or_else(|_| "unknown".into()),
                scale_metrics::ENABLED
            );
            let measurement = scale_metrics::ENABLED.then(|| Measurement::begin().unwrap());
            let started = Instant::now();
            let document = create_style_root(&mut fixture.dom, sample);
            let elapsed = started.elapsed();
            let now = fixture.dom.snapshot();
            let revisions = now.revision().get() - before.revision().get();
            if let Some(measurement) = measurement {
                let counters = measurement
                    .finish(
                        Hooks::one(Domain::Bank)
                            .union(Hooks::one(Domain::Transaction))
                            .union(Hooks::one(Domain::Validation))
                            .union(Hooks::one(Domain::Accounting))
                            .union(Hooks::one(Domain::Source)),
                    )
                    .unwrap();
                let depth = usize::BITS as usize - (managed + 3).leading_zeros() as usize;
                assert!(counters.source_preflights > 0, "{counters:?}");
                assert!(
                    counters.savepoints > 0 && counters.savepoint_peak_bytes > 0,
                    "{counters:?}"
                );
                assert!(
                    counters.source_peak_bytes > 0 && counters.source_peak_units > 0,
                    "{counters:?}"
                );
                assert!(
                    counters.source_steps > 0 && counters.source_steps <= 100_000,
                    "{counters:?}"
                );
                assert!(counters.canonical_record_reads <= 4096, "{counters:?}");
                assert!(counters.validation_record_visits <= 4096, "{counters:?}");
                assert!(counters.accounting_record_visits <= 4096, "{counters:?}");
                assert!(counters.record_materializations <= 512, "{counters:?}");
                assert!(
                    counters.sequence_materialized_bytes <= sequence_bound,
                    "bound={sequence_bound} {counters:?}"
                );
                assert!(
                    counters.relation_sequence_materializations > 0,
                    "{counters:?}"
                );
                assert!(
                    counters.relation_sequence_materialized_bytes > 0
                        && counters.relation_sequence_materialized_bytes <= relation_bound,
                    "relation_bound={relation_bound} {counters:?}"
                );
                assert!(
                    counters.structural_bytes <= 1_048_576 * depth,
                    "{counters:?}"
                );
                assert!(counters.bank_page_visits <= 16_384 * depth, "{counters:?}");
                let queue_admin = counters
                    .queue_record_visits
                    .checked_add(counters.queue_records_moved)
                    .expect("measured queue work");
                assert!(queue_admin <= 4096, "{counters:?}");
                assert_eq!(
                    counters.published_roots,
                    usize::try_from(revisions).unwrap(),
                    "each changed public revision is one actual root swap"
                );
                eprintln!(
                    "scale style_headroom_execution managed={managed} sample={sample} counters={counters:?}"
                );
            }
            // Literal construction and the accepted real CSSOM owner survive
            // the original Source completion; the old capture sees no new ID.
            assert!(before.node(document.node()).is_err());
            assert_eq!(fixture.dom.usage().managed_nodes(), managed + 3);
            let children = now.node(document.node()).unwrap().children();
            assert_eq!(children.len(), 1);
            let style = now.node(children[0]).unwrap();
            assert_eq!(style.owner_document(), Some(document));
            assert_eq!(
                style.as_element().unwrap().name().local_name(),
                &DocumentString::from("style")
            );
            let text = style.children();
            assert_eq!(text.len(), 1);
            assert_eq!(
                now.node(text[0]).unwrap().data(),
                Some(&DocumentString::from("p {}"))
            );
            assert!(
                fixture
                    .dom
                    .style_cssom_root(document.node())
                    .unwrap()
                    .is_some()
            );
            assert!(
                revisions >= 2,
                "ordinary construction and genuine Source publication both finish"
            );
            if sample >= WARMUPS {
                samples.push(elapsed);
            }
            let started = Instant::now();
            drop(fixture);
            eprintln!(
                "scale style_headroom_owner_drop_held_snapshots_ns={}",
                started.elapsed().as_nanos()
            );
            let started = Instant::now();
            drop((now, before));
            eprintln!(
                "scale style_headroom_final_snapshots_drop_ns={}",
                started.elapsed().as_nanos()
            );
        }
        report(&format!("source_style_headroom_S{managed}"), &samples);
    }
}

/// Diagnostic only: complete the same real Source import at fixed public S
/// under a separately disclosed generous finite policy. This never selects or
/// changes the qualification caps, ceilings, sample count or acceptance result.
/// Actual step events are the original standalone operation's work increments;
/// public status is observed before advance, without inventing a final receipt.
#[test]
#[ignore = "diagnostic completed Source import geometry; not scale qualification"]
#[expect(
    clippy::assertions_on_constants,
    reason = "This ignored diagnostic must compile without counters and refuse execution unless its owning instrumentation is enabled"
)]
fn diagnostic_source_import_completed_geometry() {
    const MANAGED: usize = 1000;
    const DIAGNOSTIC_WORK: u64 = 10_000_000;
    require_release();
    assert!(
        scale_metrics::ENABLED,
        "actual owning instrumentation required"
    );
    let cancel = AtomicBool::new(false);
    for records in [8, 16, 32, 64, 128] {
        let input = captured_input(records);
        let (mut fixture, retired, exhausted, witness) =
            bulk_fixture(MANAGED, records, Frontier::Occupied);
        assert!(retired.is_empty() && exhausted.is_empty());
        let before = fixture.dom.snapshot();
        let expected = expected_bulk_ids(&fixture, &retired, records);
        let expected_changes: Vec<_> = expected.iter().copied().map(DomChange::Created).collect();
        let allowances = source_limits();
        let limits = DomSourceLimits::new(
            allowances.retained_bytes(),
            allowances.retained_string_code_units(),
            DIAGNOSTIC_WORK,
            8,
            records * 4,
            u64::try_from(records).unwrap().checked_mul(2).unwrap(),
            1,
        );
        let measurement = Measurement::begin().unwrap();
        let started = Instant::now();
        let mut lease = fixture
            .dom
            .begin_source_edit(
                DomSourceEdit::import_node(
                    &input.snapshot,
                    input.root,
                    fixture.other_document.node(),
                    true,
                ),
                limits,
                &cancel,
            )
            .unwrap();
        let begin = started.elapsed();
        let initial = lease.status();
        let advance = Instant::now();
        let progress = lease.advance(&cancel);
        let elapsed = advance.elapsed();
        let done = match progress {
            Ok(DomSourceProgress::Complete(done)) => done,
            Ok(_) => panic!("generic XML clone has no external service"),
            Err(error) => {
                eprintln!(
                    "diagnostic bulk Source refused k={records} initial={initial:?} status={:?} error={error:?}",
                    lease.status()
                );
                eprintln!(
                    "diagnostic bulk refusal counters={:?}",
                    measurement.finish(Hooks::one(Domain::Source)).unwrap()
                );
                panic!("diagnostic finite policy refused: {error:?}");
            }
        };
        let counters = measurement
            .finish(
                Hooks::one(Domain::Bank)
                    .union(Hooks::one(Domain::Transaction))
                    .union(Hooks::one(Domain::Accounting))
                    .union(Hooks::one(Domain::Source)),
            )
            .unwrap();
        assert!(counters.source_steps >= usize::try_from(initial.spent_work()).unwrap());
        assert!(counters.source_steps <= usize::try_from(DIAGNOSTIC_WORK).unwrap());
        assert_eq!(counters.published_roots, 1);
        assert_eq!(done.output(), Some(DomSourceOutput::Node(expected[0])));
        assert_eq!(done.published_node(), Some(expected[0]));
        assert_eq!(done.commit().changes().changes(), expected_changes);
        drop(lease);
        let now = fixture.dom.snapshot();
        assert_eq!(now.revision().get(), before.revision().get() + 1);
        assert_eq!(fixture.dom.usage().managed_nodes(), MANAGED + records);
        assert_eq!(before.nodes().count(), MANAGED);
        assert_eq!(witness.nodes().count(), MANAGED);
        for id in &expected {
            assert_eq!(before.node(*id).unwrap_err(), DomError::StaleNode);
        }
        check_import(&now, fixture.other_document, &expected, &input);
        assert_eq!(
            now.node(fixture.target).unwrap().children(),
            fixture.children
        );
        eprintln!(
            "diagnostic bulk completed qualification=false operation=SourceImport frontier=Occupied managed={MANAGED} k={records} input_managed={} input_root_children={} input_root_attributes=1 initial_status={initial:?} diagnostic_limits={limits:?} begin_ns={} advance_ns={} actual_counters={counters:?}",
            records + 1,
            input.children.len(),
            begin.as_nanos(),
            elapsed.as_nanos(),
        );
    }
}

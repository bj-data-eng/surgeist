//! Small populated native Input/Form/Style and cross-owner Source-import
//! lifecycle. CSSOM association is produced by its real public protocol.
//! This excludes populated Form past-name history and hosted browser groups.

use super::scale_metrics;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::{Duration, Instant};

use crate::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomLimits, DomNameLimits,
    DomNativeInputPlatform, DomSnapshot, DomSourceEdit, DomSourceLimits, DomSourceOutput,
    DomSourceProgress, DomSourceRequest, DomStyleAction, DomStyleCssomOwnerBinding,
    DomStyleCssomRoot, DomStyleInlineCspObservation, Namespace, NodeId,
};
use surgeist_css::{CssMediaQueryList, CssParser, CssParserContext, CssUtf16InputLimits};
use surgeist_cssom::{
    CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs, CssomStore,
};
use surgeist_html::{
    HtmlInputEmptySelectionSupport, HtmlInputNoneDirectionSupport, HtmlInputSelectionDirection,
    HtmlStyleUpdateAction,
};

mod selected;

const SOURCE_NODES: usize = 10;
const TARGET_NODES: usize = 3;
const COMBINED_NODES: usize = SOURCE_NODES + TARGET_NODES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    HarnessOnly,
    FeaturesPopulated,
    SourceEdited,
    SourceOwnerReleased,
    ImportBegun,
    ImportComplete,
    ImportEdited,
    SourceSnapshotsReleased,
    TargetOwnerReleased,
    TargetOldOnly,
    Released,
}
impl Stage {
    const ALL: [Self; 11] = [
        Self::HarnessOnly,
        Self::FeaturesPopulated,
        Self::SourceEdited,
        Self::SourceOwnerReleased,
        Self::ImportBegun,
        Self::ImportComplete,
        Self::ImportEdited,
        Self::SourceSnapshotsReleased,
        Self::TargetOwnerReleased,
        Self::TargetOldOnly,
        Self::Released,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::HarnessOnly => "harness-only",
            Self::FeaturesPopulated => "features-populated",
            Self::SourceEdited => "source-edited",
            Self::SourceOwnerReleased => "source-owner-released",
            Self::ImportBegun => "import-begun",
            Self::ImportComplete => "import-complete",
            Self::ImportEdited => "import-edited",
            Self::SourceSnapshotsReleased => "source-snapshots-released",
            Self::TargetOwnerReleased => "target-owner-released",
            Self::TargetOldOnly => "target-old-only",
            Self::Released => "released",
        }
    }
    fn holders(self) -> Holders {
        Holders {
            source_owner: matches!(self, Self::FeaturesPopulated | Self::SourceEdited),
            source_snapshots: match self {
                Self::SourceEdited
                | Self::SourceOwnerReleased
                | Self::ImportBegun
                | Self::ImportComplete
                | Self::ImportEdited => 2,
                _ => 0,
            },
            target_owner: matches!(
                self,
                Self::ImportBegun
                    | Self::ImportComplete
                    | Self::ImportEdited
                    | Self::SourceSnapshotsReleased
            ),
            target_old: matches!(
                self,
                Self::ImportComplete
                    | Self::ImportEdited
                    | Self::SourceSnapshotsReleased
                    | Self::TargetOwnerReleased
                    | Self::TargetOldOnly
            ),
            target_current: matches!(
                self,
                Self::ImportEdited | Self::SourceSnapshotsReleased | Self::TargetOwnerReleased
            ),
            import_lease: self == Self::ImportBegun,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Holders {
    source_owner: bool,
    source_snapshots: usize,
    target_owner: bool,
    target_old: bool,
    target_current: bool,
    import_lease: bool,
}
fn observe(observer: &mut Option<&mut dyn FnMut(Stage)>, at: Stage) {
    if let Some(observer) = observer {
        observer(at);
    }
}

#[derive(Debug, Default)]
struct Times {
    owners: Duration,
    ordinary_setup: Duration,
    style_begin: Duration,
    style_advance: Duration,
    style_protocol_calls: Duration,
    css_parsing: Duration,
    cssom_calls: Duration,
    retained_snapshot_acquire: Duration,
    traversal: Duration,
    native_source_edit: Duration,
    import_begin: Duration,
    import_advance: Duration,
    import_close: Duration,
    native_target_edit: Duration,
    source_owner_drop: Duration,
    source_snapshots_drop: Duration,
    target_owner_drop: Duration,
    target_current_drop: Duration,
    target_old_drop: Duration,
}
fn timed<T>(elapsed: &mut Duration, action: impl FnOnce() -> T) -> T {
    if scale_metrics::ENABLED {
        return action();
    }
    let start = Instant::now();
    let result = action();
    *elapsed += start.elapsed();
    result
}
fn limits() -> DomLimits {
    DomLimits::new(
        64,
        131_072,
        2048,
        512,
        512,
        64,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_input_processing(1_000_000, 4_000_000)
    .unwrap()
}
fn source_limits() -> DomSourceLimits {
    // The existing Style fixture's finite policy, not a measured tariff/SLO.
    DomSourceLimits::new(16_000_000, 100_000, 10_000_000, 64, 4096, 4096, 1)
}
fn owner(times: &mut Times) -> Dom {
    let limits = limits();
    let platform = DomNativeInputPlatform::new(
        HtmlInputEmptySelectionSupport::Unsupported,
        HtmlInputNoneDirectionSupport::Unsupported,
    );
    timed(&mut times.owners, || {
        Dom::with_native_input_platform(limits, platform)
    })
    .unwrap()
}
#[derive(Clone, Copy)]
struct SourceIds {
    associated: DocumentId,
    document: DocumentId,
    doctype: NodeId,
    html: NodeId,
    head: NodeId,
    body: NodeId,
    style: NodeId,
    text: NodeId,
    form: NodeId,
    input: NodeId,
}

fn populate(dom: &mut Dom, times: &mut Times, cancel: &AtomicBool) -> SourceIds {
    let (associated, document, doctype, html, head, body) = {
        let mut batch = DomBatch::new();
        let associated = batch.create_document();
        let document = batch.create_html_document(&associated, None);
        let commit = timed(&mut times.ordinary_setup, || dom.apply(batch, cancel)).unwrap();
        let associated = commit.document(&associated).unwrap();
        let document = commit.document(&document).unwrap();
        let snapshot = dom.snapshot();
        let root = snapshot.node(document.node()).unwrap();
        assert_eq!(root.children().len(), 2);
        let doctype = root.children()[0];
        let html = root.children()[1];
        let children = snapshot.node(html).unwrap().children();
        assert_eq!(children.len(), 2);
        (
            associated,
            document,
            doctype,
            html,
            children[0],
            children[1],
        )
    };
    let (style, text, form, input) = {
        let mut batch = DomBatch::new();
        let style = batch.create_element_ns(document, Namespace::html(), "style");
        batch.set_attribute(&style, "blocking", "other\trender\nrender");
        let text = batch.create_text(document, "a {}");
        batch.insert_before(&text, &style, None);
        let form = batch.create_element_ns(document, Namespace::html(), "form");
        batch.set_attribute(&form, "id", "f");
        let input = batch.create_element_ns(document, Namespace::html(), "input");
        batch.set_attribute(&input, "type", "text");
        batch.set_attribute(&input, "value", "seed");
        batch.set_custom_validity(&input, "custom\r\nmessage");
        batch.insert_before(&form, body, None);
        batch.insert_before(&input, &form, None);
        let commit = timed(&mut times.ordinary_setup, || dom.apply(batch, cancel)).unwrap();
        (
            commit.created(&style).unwrap().unwrap(),
            commit.created(&text).unwrap().unwrap(),
            commit.created(&form).unwrap().unwrap(),
            commit.created(&input).unwrap().unwrap(),
        )
    };

    // This is the existing public Style/CSSOM creation sequence. No local
    // browser state, sheet ID, policy decision or publication is fabricated.
    {
        let edit = DomSourceEdit::insert_before(style, head, None);
        let limits = source_limits();
        let mut lease = timed(&mut times.style_begin, || {
            dom.begin_source_edit(edit, limits, cancel)
        })
        .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
            timed(&mut times.style_advance, || lease.advance(cancel)).unwrap()
        else {
            panic!("actual inline CSP request");
        };
        assert_eq!(
            policy.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::CheckInlineCsp)
        );
        let observation = DomStyleInlineCspObservation::complete_empty(policy.source(), 1);
        timed(&mut times.style_protocol_calls, || {
            lease.accept_inline_csp(&policy, &observation, cancel)
        })
        .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
            timed(&mut times.style_advance, || lease.advance(cancel)).unwrap()
        else {
            panic!("actual CSSOM creation request");
        };
        assert_eq!(
            create.action(),
            DomStyleAction::Update(HtmlStyleUpdateAction::CreateSheet)
        );
        assert_eq!(create.source().text().as_ref(), &[97, 32, 123, 125]);
        let store_limits = Default::default();
        let style_policy = Default::default();
        let store = timed(&mut times.cssom_calls, || {
            CssomStore::new(store_limits, style_policy)
        })
        .unwrap();
        let root_id = document.node();
        let root = timed(&mut times.cssom_calls, || {
            DomStyleCssomRoot::new(root_id, store)
        });
        timed(&mut times.style_protocol_calls, || {
            lease.bind_cssom_root(&create, &root, cancel)
        })
        .unwrap();
        let snapshot = root.snapshot().unwrap();
        let create = timed(&mut times.style_protocol_calls, || {
            lease.qualify_cssom(&create, root_id, &snapshot)
        })
        .unwrap();
        drop(snapshot);
        let version = CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "blocking-style".into(),
            revision: 1,
        };
        let binding =
            DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline".into()).unwrap();
        timed(&mut times.style_protocol_calls, || {
            lease.bind_cssom_owner(&create, binding, cancel)
        })
        .unwrap();
        let context = CssParserContext::default();
        let sheet_text = create.source().text().clone();
        let sheet_limits = CssUtf16InputLimits::default();
        let sheet_poll = || Ok::<_, ()>(());
        let report = timed(&mut times.css_parsing, || {
            CssParser::new(context, Default::default()).parse_sheet_utf16(
                sheet_text,
                sheet_limits,
                sheet_poll,
            )
        })
        .unwrap();
        let media_text = Arc::new(Vec::new());
        let media_limits = CssUtf16InputLimits::default();
        let media_poll = || Ok::<_, ()>(());
        let media = timed(&mut times.css_parsing, || {
            CssMediaQueryList::parse_utf16(
                &CssParser::new(context, Default::default()),
                media_text,
                media_limits,
                media_poll,
            )
        })
        .unwrap();
        let mut inputs = CssomSheetInputs::external(version, true);
        inputs.owner = Some("inline".into());
        let snapshot = root.snapshot().unwrap();
        let guard = CssomEditGuard::from_snapshot(&snapshot);
        let mut edit = timed(&mut times.cssom_calls, || root.batch(guard)).unwrap();
        drop(snapshot);
        let construction = timed(&mut times.cssom_calls, || {
            edit.create_parsed_sheet_with_media_reports(report, media, inputs)
        })
        .unwrap();
        let mut pending = Some(edit);
        let commit = timed(&mut times.cssom_calls, || root.commit(&mut pending)).unwrap();
        let snapshot = root.snapshot().unwrap();
        let ticket = Some(construction.ticket());
        // Keep the large error on this stack. End timing before result handling,
        // and read no clock at all in a storage-counter build.
        let started = (!scale_metrics::ENABLED).then(Instant::now);
        let outcome = lease.accept_cssom_publication(&create, commit, snapshot, ticket);
        if let Some(started) = started {
            times.style_protocol_calls += started.elapsed();
        }
        let _notification = outcome.unwrap();
        let DomSourceProgress::Complete(done) =
            timed(&mut times.style_advance, || lease.advance(cancel)).unwrap()
        else {
            panic!("completed native Style Source");
        };
        assert_eq!(done.output(), None);
        drop(lease);
        // The canonical DOM now owns the bound CSSOM root. All local requests,
        // parse reports, creation tickets, snapshots and root handles end here.
    }
    assert!(dom.pending_source_edit().is_none());
    SourceIds {
        associated,
        document,
        doctype,
        html,
        head,
        body,
        style,
        text,
        form,
        input,
    }
}

fn check_source(snapshot: &DomSnapshot, ids: SourceIds, value: &[u16], dirty: bool) {
    assert_eq!(snapshot.nodes().count(), SOURCE_NODES);
    assert!(
        snapshot
            .node(ids.associated.node())
            .unwrap()
            .children()
            .is_empty()
    );
    assert_eq!(
        snapshot.node(ids.document.node()).unwrap().children(),
        &[ids.doctype, ids.html]
    );
    assert_eq!(
        snapshot.node(ids.html).unwrap().children(),
        &[ids.head, ids.body]
    );
    assert_eq!(snapshot.node(ids.head).unwrap().children(), &[ids.style]);
    assert_eq!(snapshot.node(ids.body).unwrap().children(), &[ids.form]);
    assert_eq!(snapshot.node(ids.style).unwrap().children(), &[ids.text]);
    assert_eq!(snapshot.node(ids.form).unwrap().children(), &[ids.input]);
    assert_eq!(snapshot.node(ids.input).unwrap().parent(), Some(ids.form));
    assert_eq!(
        snapshot
            .node(ids.text)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        &[97, 32, 123, 125]
    );
    for id in [ids.style, ids.text, ids.form, ids.input] {
        assert_eq!(
            snapshot.node(id).unwrap().owner_document(),
            Some(ids.document)
        );
    }
    let input_node = snapshot.node(ids.input).unwrap();
    let input = input_node.as_input().unwrap();
    assert_eq!(input.value(), value);
    assert_eq!(input.state().flags().dirty_value(), dirty);
    if dirty {
        assert_eq!(
            input.state().selection().unwrap().cursor_position(),
            Some(7)
        );
    }
    assert_eq!(
        input.state().custom_validity_message().code_units(),
        &[
            99, 117, 115, 116, 111, 109, 10, 109, 101, 115, 115, 97, 103, 101
        ]
    );
    assert_eq!(input.form_owner().unwrap(), Some(ids.form));
    let attributes = input_node.as_element().unwrap().attributes();
    assert_eq!(attributes.len(), 2);
    assert_eq!(
        attributes[0].name().local_name().code_units(),
        &[116, 121, 112, 101]
    );
    assert_eq!(attributes[0].value().code_units(), &[116, 101, 120, 116]);
    assert_eq!(
        attributes[1].name().local_name().code_units(),
        &[118, 97, 108, 117, 101]
    );
    assert_eq!(attributes[1].value().code_units(), &[115, 101, 101, 100]);
    let form = snapshot.node(ids.form).unwrap().as_form().unwrap();
    assert_eq!(form.past_names().len(), 0);
    let style = snapshot.node(ids.style).unwrap().as_style().unwrap();
    let sheet = style
        .sheet()
        .unwrap()
        .expect("actual associated CSSOM sheet");
    assert_eq!(sheet.root(), ids.document.node());
    assert_eq!(sheet.association().element(), ids.style);
    assert!(
        !style.is_render_blocking().unwrap(),
        "the literal HTML body is present"
    );
}

fn check_import(
    snapshot: &DomSnapshot,
    document: DocumentId,
    form: NodeId,
    input: NodeId,
    value: &[u16],
    cursor: usize,
) {
    assert_eq!(snapshot.nodes().count(), TARGET_NODES);
    assert_eq!(snapshot.node(document.node()).unwrap().children(), &[]);
    let root = snapshot.node(form).unwrap();
    assert_eq!(root.parent(), None);
    assert_eq!(root.owner_document(), Some(document));
    assert_eq!(root.children(), &[input]);
    assert_eq!(root.node_name(), DocumentString::from("form"));
    assert_eq!(root.as_form().unwrap().past_names().len(), 0);
    let form_attributes = root.as_element().unwrap().attributes();
    assert_eq!(form_attributes.len(), 1);
    assert_eq!(
        form_attributes[0].name().local_name().code_units(),
        &[105, 100]
    );
    assert_eq!(form_attributes[0].value().code_units(), &[102]);
    let node = snapshot.node(input).unwrap();
    assert_eq!(node.parent(), Some(form));
    assert_eq!(node.owner_document(), Some(document));
    let native = node.as_input().unwrap();
    assert_eq!(native.form_owner().unwrap(), Some(form));
    assert_eq!(native.value(), value);
    assert!(native.state().flags().dirty_value());
    assert!(!native.state().flags().dirty_checkedness());
    assert!(!native.state().flags().checkedness());
    assert!(!native.state().flags().indeterminate());
    assert!(native.state().custom_validity_message().is_empty());
    assert!(!native.state().parser_inserted());
    let selection = native.state().selection().unwrap();
    assert_eq!(selection.cursor_position(), Some(cursor));
    assert_eq!(selection.direction(), HtmlInputSelectionDirection::Forward);
    assert_eq!(native.explicit_popover_target(), None);
    let attributes = node.as_element().unwrap().attributes();
    assert_eq!(attributes.len(), 2);
    assert_eq!(
        attributes[0].name().local_name().code_units(),
        &[116, 121, 112, 101]
    );
    assert_eq!(attributes[0].value().code_units(), &[116, 101, 120, 116]);
    assert_eq!(
        attributes[1].name().local_name().code_units(),
        &[118, 97, 108, 117, 101]
    );
    assert_eq!(attributes[1].value().code_units(), &[115, 101, 101, 100]);
}

fn run_sample(mut observer: Option<&mut dyn FnMut(Stage)>) -> Times {
    let mut times = Times::default();
    let cancel = AtomicBool::new(false);
    observe(&mut observer, Stage::HarnessOnly);
    let mut source = owner(&mut times);
    let ids = populate(&mut source, &mut times, &cancel);
    {
        let snapshot = source.snapshot();
        check_source(&snapshot, ids, &[115, 101, 101, 100], false);
        assert_eq!(timed(&mut times.traversal, || snapshot.nodes().count()), 10);
    }
    observe(&mut observer, Stage::FeaturesPopulated);
    let source_old = timed(&mut times.retained_snapshot_acquire, || source.snapshot());
    let mut batch = DomBatch::new();
    batch.set_input_value(ids.input, "private");
    let commit = timed(&mut times.native_source_edit, || {
        source.apply(batch, &cancel)
    })
    .unwrap();
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::ElementState(ids.input)]
    );
    assert_eq!(
        source.snapshot().revision().get(),
        source_old.revision().get() + 1
    );
    drop(commit);
    let source_current = timed(&mut times.retained_snapshot_acquire, || source.snapshot());
    check_source(&source_old, ids, &[115, 101, 101, 100], false);
    check_source(
        &source_current,
        ids,
        &[112, 114, 105, 118, 97, 116, 101],
        true,
    );
    assert!(std::ptr::eq(
        source_old
            .node(ids.input)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
        source_current
            .node(ids.input)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
    ));
    assert_eq!(
        source_old
            .node(ids.style)
            .unwrap()
            .as_style()
            .unwrap()
            .state(),
        source_current
            .node(ids.style)
            .unwrap()
            .as_style()
            .unwrap()
            .state()
    );
    observe(&mut observer, Stage::SourceEdited);
    timed(&mut times.source_owner_drop, || drop(source));
    check_source(
        &source_current,
        ids,
        &[112, 114, 105, 118, 97, 116, 101],
        true,
    );
    observe(&mut observer, Stage::SourceOwnerReleased);

    let mut target = owner(&mut times);
    let document = {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = timed(&mut times.ordinary_setup, || target.apply(batch, &cancel)).unwrap();
        commit.document(&document).unwrap()
    };
    let target_before = target.snapshot().stamp();
    let imported = {
        let edit = DomSourceEdit::import_node(&source_current, ids.form, document.node(), true);
        let limits = source_limits();
        let mut lease = timed(&mut times.import_begin, || {
            target.begin_source_edit(edit, limits, &cancel)
        })
        .unwrap();
        assert_eq!(lease.snapshot().nodes().count(), 1);
        observe(&mut observer, Stage::ImportBegun);
        let DomSourceProgress::Complete(done) =
            timed(&mut times.import_advance, || lease.advance(&cancel)).unwrap()
        else {
            panic!("Form/Text-Input import needs no external Style/host service");
        };
        let form = done.published_node().expect("real imported Form identity");
        assert_eq!(done.output(), Some(DomSourceOutput::Node(form)));
        drop(done);
        timed(&mut times.import_close, || drop(lease));
        form
    };
    assert!(target.pending_source_edit().is_none());
    assert_ne!(imported.owner(), ids.form.owner());
    assert_ne!(imported, ids.form);
    assert!(target.snapshot().revision().get() > target_before.revision().get());
    let target_old = timed(&mut times.retained_snapshot_acquire, || target.snapshot());
    let input = target_old.node(imported).unwrap().children()[0];
    assert_ne!(input, ids.input);
    assert_eq!(input.owner(), imported.owner());
    check_import(
        &target_old,
        document,
        imported,
        input,
        &[112, 114, 105, 118, 97, 116, 101],
        0,
    );
    assert_eq!(
        timed(&mut times.traversal, || target_old.nodes().count()),
        3
    );
    observe(&mut observer, Stage::ImportComplete);

    let mut batch = DomBatch::new();
    batch.set_input_value(input, "target");
    let commit = timed(&mut times.native_target_edit, || {
        target.apply(batch, &cancel)
    })
    .unwrap();
    assert_eq!(
        commit.changes().changes(),
        &[DomChange::ElementState(input)]
    );
    assert_eq!(
        target.snapshot().revision().get(),
        target_old.revision().get() + 1
    );
    drop(commit);
    let target_current = timed(&mut times.retained_snapshot_acquire, || target.snapshot());
    check_import(
        &target_current,
        document,
        imported,
        input,
        &[116, 97, 114, 103, 101, 116],
        6,
    );
    check_import(
        &target_old,
        document,
        imported,
        input,
        &[112, 114, 105, 118, 97, 116, 101],
        0,
    );
    assert!(std::ptr::eq(
        target_old
            .node(input)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
        target_current
            .node(input)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
    ));
    check_source(
        &source_current,
        ids,
        &[112, 114, 105, 118, 97, 116, 101],
        true,
    );
    check_source(&source_old, ids, &[115, 101, 101, 100], false);
    observe(&mut observer, Stage::ImportEdited);
    timed(&mut times.source_snapshots_drop, || {
        drop((source_old, source_current))
    });
    observe(&mut observer, Stage::SourceSnapshotsReleased);
    timed(&mut times.target_owner_drop, || drop(target));
    check_import(
        &target_current,
        document,
        imported,
        input,
        &[116, 97, 114, 103, 101, 116],
        6,
    );
    observe(&mut observer, Stage::TargetOwnerReleased);
    timed(&mut times.target_current_drop, || drop(target_current));
    check_import(
        &target_old,
        document,
        imported,
        input,
        &[112, 114, 105, 118, 97, 116, 101],
        0,
    );
    observe(&mut observer, Stage::TargetOldOnly);
    timed(&mut times.target_old_drop, || drop(target_old));
    observe(&mut observer, Stage::Released);
    times
}

#[test]
fn populated_native_features_and_source_import_preserve_snapshots_and_clone_boundaries() {
    let mut stage = 0;
    let times = run_sample(Some(&mut |at| {
        assert_eq!(at, Stage::ALL[stage]);
        let holders = at.holders();
        assert_eq!(holders.import_lease, at == Stage::ImportBegun);
        stage += 1;
    }));
    assert_eq!(stage, 11);
    eprintln!(
        "native_features source_nodes=10 target_nodes=3 declared_combined_nodes=13 clock_enabled={} timings={times:?} timings_are_individual_calls_oracles_and_observers_excluded=true batch_authoring_and_protocol_input_preparation_outside_clocks=true parser_interrupt_poll_inside_parse_call=true no_timing_slo_or_physical_claim=true",
        !scale_metrics::ENABLED
    );
}

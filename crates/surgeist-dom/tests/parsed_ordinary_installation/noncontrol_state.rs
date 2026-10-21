//! Completed non-control history and later canonical operations stay distinct.
use super::*;
use surgeist_html::{
    HtmlElementClass, HtmlNodeId, HtmlNodeKind, HtmlScriptResult, KnownHtmlElement,
};
use surgeist_html_parser::HtmlFragmentParseRequest;
use surgeist_html_parser::HtmlParseMetadataStep;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlFragmentContext};

#[test]
fn contextual_installation_preserves_replacement_split_shadow_and_host_documents() {
    let cancel = AtomicBool::new(false);
    let (mut dom, _) = setup();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let host = batch.create_element_ns(&a, Namespace::html(), "div");
    let root = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let child = batch.create_comment(&a, "transferred");
    batch.insert_before(&host, &a, None);
    batch.insert_before(&child, &root, None);
    let b = batch.create_document();
    let target = batch.create_element_ns(&b, Namespace::html(), "div");
    let old = batch.create_comment(&b, "old");
    batch.insert_before(&target, &b, None);
    batch.insert_before(&old, &target, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let a = commit.document(&a).unwrap();
    let b = commit.document(&b).unwrap();
    let host = commit.created(&host).unwrap().unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let target = commit.created(&target).unwrap().unwrap();
    let old = commit.created(&old).unwrap().unwrap();
    let initial = dom.snapshot();
    // The actual replacement algorithm adopts its supplied fragment before
    // moving children; direct public ShadowRoot adoption remains forbidden.
    let mut batch = DomBatch::new();
    batch.replace_child(root, target, old);
    dom.apply(batch, &cancel).unwrap();
    let before = dom.snapshot();
    assert_eq!(before.node(root).unwrap().owner_document(), Some(b));
    assert_eq!(before.node(host).unwrap().owner_document(), Some(a));
    assert_eq!(before.shadow_root(root).unwrap().host(), host);
    assert_eq!(before.node(target).unwrap().children(), &[child]);
    assert!(before.node(root).unwrap().children().is_empty());
    let capture = before
        .html_capture(
            b,
            None,
            HtmlCaptureLimits::new(1024, 4096, 4096, 1_000_000, 1024, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(1024, 1_000_000),
            &cancel,
        )
        .unwrap();
    let context =
        HtmlFragmentContext::hosted_fragment(capture.capture(), &capture.subject(root).unwrap())
            .unwrap();
    assert_eq!(
        capture.node(context.target_document().subject()).unwrap(),
        b.node()
    );
    assert_eq!(
        capture.node(context.context_document().subject()).unwrap(),
        a.node()
    );
    let report = Arc::new(
        HtmlFragmentParseRequest::new(HtmlSource::utf8("<p>X</p>"), context, parse_limits())
            .unwrap()
            .parse()
            .unwrap(),
    );
    let source = report.source().identity().clone();
    let installed = dom
        .install_parsed_html_fragment(report.clone(), &capture, limits(), &cancel)
        .unwrap();
    assert!(std::ptr::eq(installed.report(), report.as_ref()));
    assert_eq!(installed.document(), b);
    assert_eq!(installed.context_basis(), Some(capture.basis()));
    for id in [root, host, a.node(), b.node()] {
        assert_eq!(
            installed.captured_node_for(&capture.subject(id).unwrap()),
            Some(id)
        );
    }
    let p = element(&installed, "p");
    let snapshot = installed.snapshot();
    assert_eq!(snapshot.revision().get(), before.revision().get() + 1);
    assert_eq!(
        snapshot.node(installed.root()).unwrap().owner_document(),
        Some(b)
    );
    assert_eq!(snapshot.node(p).unwrap().owner_document(), Some(b));
    assert_eq!(snapshot.node(installed.root()).unwrap().children(), &[p]);
    assert!(snapshot.node(installed.root()).unwrap().parent().is_none());
    assert!(snapshot.node(root).unwrap().children().is_empty());
    assert_eq!(
        snapshot.document(a).unwrap().metadata(),
        before.document(a).unwrap().metadata()
    );
    assert_eq!(
        snapshot.document(b).unwrap().metadata(),
        before.document(b).unwrap().metadata()
    );
    let text = snapshot.node(p).unwrap().children()[0];
    assert_eq!(
        snapshot.node(text).unwrap().data().unwrap().code_units(),
        [88]
    );
    let mut batch = DomBatch::new();
    batch.insert_before(installed.root(), root, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let current = dom.snapshot();
    assert_eq!(commit.changes().from(), snapshot.stamp());
    assert_eq!(current.revision().get(), snapshot.revision().get() + 1);
    assert_eq!(current.node(root).unwrap().children(), &[p]);
    assert_eq!(current.node(p).unwrap().parent(), Some(root));
    assert_eq!(current.node(p).unwrap().owner_document(), Some(b));
    assert_eq!(current.node(root).unwrap().owner_document(), Some(b));
    assert_eq!(current.node(host).unwrap().owner_document(), Some(a));
    assert_eq!(current.shadow_root(root).unwrap().host(), host);
    assert_eq!(current.node(host).unwrap().hosted_shadow_root(), Some(root));
    assert_eq!(current.node(target).unwrap().children(), &[child]);
    assert_eq!(current.node(old).unwrap().parent(), None);
    assert!(snapshot.node(root).unwrap().children().is_empty());
    assert_eq!(initial.node(root).unwrap().owner_document(), Some(a));
    assert_eq!(initial.node(root).unwrap().children(), &[child]);
    assert_eq!(report.source().identity(), &source);
    assert_eq!(report.source().as_utf8(), Some("<p>X</p>"));
}

fn assert_original_patch(
    report: &HtmlParseReport,
    installed: &DomParsedHtmlInstallation,
    template: NodeId,
    target: NodeId,
    marker: NodeId,
) {
    let mut records = report.metadata();
    let mut templates = 0;
    loop {
        match records.advance() {
            HtmlParseMetadataStep::Record(row) => {
                if let Some(patch) = row.template() {
                    assert_eq!(installed.node_for(&row.node().id()), Some(template));
                    assert_eq!(
                        installed.node_for(&patch.target().unwrap().id()),
                        Some(target)
                    );
                    assert_eq!(
                        installed.node_for(&patch.start().unwrap().id()),
                        Some(marker)
                    );
                    assert_eq!(installed.node_for(&patch.end().unwrap().id()), Some(marker));
                    templates += 1;
                }
            }
            HtmlParseMetadataStep::Skipped => {}
            HtmlParseMetadataStep::Complete => break,
        }
    }
    assert_eq!(templates, 1);
}

#[test]
fn retiring_only_the_template_target_preserves_live_markers_and_never_retargets_a_replacement() {
    for native_context in [false, true] {
        let markup = "<body><div><?marker name='x'?></div><template for=x><p>N";
        let (mut dom, report, installed) = if native_context {
            let (mut dom, capture, _, host) = fragment_fixture("div");
            let report = fragment_report(&capture, host, HtmlSource::utf8(markup), false);
            let installed = dom
                .install_parsed_html_fragment(
                    report.clone(),
                    &capture,
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap();
            (dom, report, installed)
        } else {
            let report = parse(HtmlSource::utf8(markup));
            let (mut dom, environment) = setup();
            let installed = install(&mut dom, report.clone(), environment);
            (dom, report, installed)
        };
        assert_eq!(
            installed
                .snapshot()
                .document(installed.document())
                .unwrap()
                .focused_area_designation(),
            DomFocusedAreaDesignation::KnownNull
        );
        let template = element(&installed, "template");
        let target = element(&installed, "div");
        let marker = installed
            .bindings()
            .iter()
            .find_map(|binding| {
                matches!(
                    installed.snapshot().node(binding.node()).unwrap().kind(),
                    DomNodeKind::ProcessingInstruction(_)
                )
                .then_some(binding.node())
            })
            .unwrap();
        let original = installed.snapshot();
        let patch = original
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .insertion_state();
        assert_eq!(
            (patch.target(), patch.start(), patch.end()),
            (Some(target), Some(marker), Some(marker))
        );
        assert_original_patch(&report, &installed, template, target, marker);
        let mut batch = DomBatch::new();
        let other = batch.create_document();
        batch.adopt(marker, &other);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let other = commit.document(&other).unwrap();
        let moved = dom.snapshot();
        assert_eq!(moved.node(marker).unwrap().parent(), None);
        assert_eq!(moved.node(marker).unwrap().owner_document(), Some(other));
        assert_eq!(
            moved
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .insertion_state(),
            patch
        );
        let mut batch = DomBatch::new();
        batch.detach(target);
        batch.destroy(target);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let retired = dom.snapshot();
        let current_patch = retired
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .insertion_state();
        assert_eq!(
            (
                current_patch.target(),
                current_patch.start(),
                current_patch.end()
            ),
            (None, Some(marker), Some(marker))
        );
        assert!(matches!(retired.node(target), Err(DomError::StaleNode)));
        assert!(retired.node(marker).is_ok());
        assert_eq!(retired.revision().get(), moved.revision().get() + 1);
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::Destroyed(target))
        );
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::TemplateRelations)
        );
        let mut batch = DomBatch::new();
        let replacement = batch.create_element_ns(installed.document(), Namespace::html(), "div");
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let replacement = commit.created(&replacement).unwrap().unwrap();
        assert_ne!(replacement, target);
        let replaced = dom.snapshot();
        assert!(matches!(replaced.node(target), Err(DomError::StaleNode)));
        assert!(replaced.node(replacement).is_ok());
        assert_eq!(
            replaced
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .insertion_state(),
            current_patch
        );
        let mut batch = DomBatch::new();
        batch.destroy(marker);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let current = dom.snapshot();
        let final_patch = current
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .insertion_state();
        assert_eq!(
            (final_patch.target(), final_patch.start(), final_patch.end()),
            (None, None, None)
        );
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::Destroyed(marker))
        );
        assert!(
            commit
                .changes()
                .changes()
                .contains(&DomChange::TemplateRelations)
        );
        assert!(current.node(replacement).is_ok());
        assert_eq!(
            original
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .insertion_state(),
            patch
        );
        assert!(original.node(target).is_ok());
        assert!(original.node(marker).is_ok());
        assert_eq!(
            retired
                .node(template)
                .unwrap()
                .as_template()
                .unwrap()
                .insertion_state(),
            current_patch
        );
        assert!(retired.node(marker).is_ok());
        assert!(std::ptr::eq(installed.report(), report.as_ref()));
        assert_eq!(report.source().as_utf8(), Some(markup));
        assert_original_patch(&report, &installed, template, target, marker);
    }
}

fn script_flags(script: DomHtmlScript<'_>) -> [bool; 5] {
    let flags = script.flags();
    [
        flags.force_async(),
        flags.already_started(),
        flags.external_file(),
        flags.ready_to_be_parser_executed(),
        flags.delaying_load(),
    ]
}

fn assert_script_copy(snapshot: &DomSnapshot, id: NodeId, started: bool) {
    let script = snapshot.node(id).unwrap().as_html_script().unwrap();
    // The HTML Script clone extension copies already-started, not the original
    // parser/preparation identities or a pending execution/result capability.
    assert_eq!(script_flags(script), [true, started, false, false, false]);
    assert_eq!(script.script_type(), None);
    assert_eq!(script.parser_document(), None);
    assert_eq!(script.preparation_document(), None);
    assert_eq!(script.result(), HtmlScriptResult::Uninitialized);
    assert_eq!(script.steps_when_result_ready(), None);
    assert_eq!(snapshot.node(id).unwrap().parent(), None);
    let text = snapshot.node(id).unwrap().children();
    assert_eq!(text.len(), 1);
    assert_eq!(
        snapshot.node(text[0]).unwrap().data().unwrap().code_units(),
        [105, 110, 101, 114, 116]
    );
}

#[test]
fn installed_true_and_false_html_script_flags_feed_the_ordinary_clone_rule() {
    for started in [true, false] {
        let (mut dom, report, installed) = if started {
            let report = parse(HtmlSource::utf8("<!doctype html><script>inert</script>"));
            let (mut dom, environment) = setup();
            let installed = install(&mut dom, report.clone(), environment);
            (dom, report, installed)
        } else {
            let (mut dom, capture, _, host) = fragment_fixture("div");
            let report = fragment_report(
                &capture,
                host,
                HtmlSource::utf8("<script>inert</script>"),
                false,
            );
            let installed = dom
                .install_parsed_html_fragment(
                    report.clone(),
                    &capture,
                    limits(),
                    &AtomicBool::new(false),
                )
                .unwrap();
            (dom, report, installed)
        };
        let original = installed.snapshot();
        let script = element(&installed, "script");
        let original_script = original.node(script).unwrap().as_html_script().unwrap();
        assert_eq!(
            script_flags(original_script),
            [!started, started, false, false, false]
        );
        assert_eq!(
            original_script.parser_document(),
            started.then_some(installed.document())
        );
        assert_eq!(
            original_script.preparation_document(),
            started.then_some(installed.document())
        );
        assert_eq!(
            original_script.script_type(),
            started.then_some(surgeist_html::HtmlScriptKind::Classic)
        );
        let mut batch = DomBatch::new();
        let clone = batch.import_node(original, script, installed.document(), true);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let clone = commit.created(&clone).unwrap().unwrap();
        assert_ne!(clone, script);
        let current = dom.snapshot();
        assert_script_copy(&current, clone, started);
        assert_eq!(
            current.node(clone).unwrap().owner_document(),
            Some(installed.document())
        );
        let (mut target, _) = setup();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let imported = batch.import_node(original, script, &document, true);
        let commit = target.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.document(&document).unwrap();
        let imported = commit.created(&imported).unwrap().unwrap();
        assert_ne!(imported.owner(), script.owner());
        assert_script_copy(&target.snapshot(), imported, started);
        assert_eq!(
            target.snapshot().node(imported).unwrap().owner_document(),
            Some(document)
        );
        assert_eq!(
            script_flags(original_script),
            [!started, started, false, false, false]
        );
        assert_eq!(
            original_script.parser_document(),
            started.then_some(installed.document())
        );
        assert_eq!(
            original_script.preparation_document(),
            started.then_some(installed.document())
        );
        assert!(std::ptr::eq(installed.report(), report.as_ref()));
        assert_eq!(
            report.source().as_utf8(),
            Some(if started {
                "<!doctype html><script>inert</script>"
            } else {
                "<script>inert</script>"
            })
        );
    }
}

fn source_element(report: &HtmlParseReport, local: &str) -> HtmlNodeId {
    let document = report.document().unwrap();
    let mut pending = vec![document.root().id()];
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind()
            && element
                .name()
                .local_name()
                .code_units()
                .iter()
                .copied()
                .eq(local.encode_utf16())
        {
            return id;
        }
        pending.extend(node.children().rev());
    }
    panic!("literal source element {local}")
}

#[test]
fn unsupported_audio_refuses_after_an_admitted_prefix_without_publishing_any_state() {
    let markup = "<!doctype html><div>private prefix</div><audio></audio>";
    let report = parse(HtmlSource::utf8(markup));
    let source = report.source().identity().clone();
    let audio = source_element(&report, "audio");
    let prefix = source_element(&report, "div");
    let (mut dom, environment) = setup();
    let mut batch = DomBatch::new();
    let existing = batch.create_document();
    let sentinel = batch.create_comment(&existing, "unchanged");
    batch.insert_before(&sentinel, &existing, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    let existing = commit.document(&existing).unwrap();
    let sentinel = commit.created(&sentinel).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    for _ in 0..2 {
        let error = dom
            .install_parsed_html_document(
                before.stamp(),
                report.clone(),
                environment.clone(),
                limits(),
                &AtomicBool::new(false),
            )
            .unwrap_err();
        assert!(
            matches!(error, DomParsedHtmlInstallError::FamilyStateUnavailable { subject, class: HtmlElementClass::KnownHtml(KnownHtmlElement::Audio) } if subject == audio)
        );
        let current = dom.snapshot();
        assert_eq!(current.stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        assert_eq!(
            current.node(existing.node()).unwrap().children(),
            &[sentinel]
        );
        assert_eq!(
            current.node(sentinel).unwrap().parent(),
            Some(existing.node())
        );
        assert_eq!(
            current.node(sentinel).unwrap().data().unwrap().code_units(),
            [117, 110, 99, 104, 97, 110, 103, 101, 100]
        );
        assert_eq!(
            current.document(existing).unwrap().metadata(),
            before.document(existing).unwrap().metadata()
        );
        assert_eq!(report.source().identity(), &source);
        assert_eq!(report.source().as_utf8(), Some(markup));
        assert_ne!(audio, prefix);
        let document = report.document().unwrap();
        let text = document.node(&prefix).unwrap().children().next().unwrap();
        let HtmlNodeKind::Text(value) = document.node(&text).unwrap().kind() else {
            panic!("the actual admitted prefix remains report Text")
        };
        assert_eq!(
            value.code_units(),
            "private prefix".encode_utf16().collect::<Vec<_>>()
        );
    }
}

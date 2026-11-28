//! Canonical Document design-mode facts in HTML editability queries.
//! HTML877d html-editability lines121,151: fresh Document design mode is false;
//! this direct HTML child has no explicit contenteditable host state.
use std::sync::atomic::AtomicBool;

use surgeist_dom::{Dom, DomBatch, capture::DomHtmlCaptureLimits, limits::DomLimits};
use surgeist_html::values::HtmlValueInterpreter;
use surgeist_html::values::Namespace;
use surgeist_html_semantics::{
    HtmlCaptureLimits, HtmlSemanticCancellation, HtmlSemanticFact, HtmlSemanticLimits,
    HtmlSemanticQuery, HtmlSemanticRequest, HtmlSemanticValue,
};

#[test]
fn native_fresh_document_design_mode_disables_implicit_editing_host() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            8,
            4096,
            16,
            0,
            0,
            8,
            65536,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(root).unwrap().parent(), Some(document.node()));
    assert!(
        snapshot
            .node(root)
            .unwrap()
            .attributes()
            .unwrap()
            .is_empty()
    );
    let capture = snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(8, 64, 16, 4096, 8, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(8, 10000),
            &cancel,
        )
        .unwrap();
    let report = capture
        .query(
            &[HtmlSemanticRequest::new(
                capture.subject(root).unwrap(),
                HtmlSemanticQuery::EditingHost,
            )],
            HtmlValueInterpreter::new(1024),
            HtmlSemanticLimits::new(100000, 16, 1024, 64, 1000000, 1024),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
    assert!(
        matches!(
            report.results()[0].fact(),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
        ),
        "fresh native Document design mode is disabled; got {:?}",
        report.results()[0].fact()
    );
}

use surgeist_dom::{
    DomError, DomSnapshot,
    capture::DomHtmlCapture,
    identity::{DocumentId, NodeId},
};
use surgeist_dom::{
    capture::{
        DomHostChange, DomHostChangeCoverage, DomHostDomains, DomHostError, DomHostPublisher,
        DomHostSpellcheckError, DomHostSpellcheckLimits,
    },
    edit::DomChange,
};
use surgeist_html_semantics::HtmlSpellcheckDefaultBehavior::{
    FalseByDefault, InheritByDefault, TrueByDefault,
};
use surgeist_html_semantics::{HtmlSemanticMissing, HtmlTranslationMode};

fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            256,
            100_000,
            1024,
            1024,
            1024,
            16,
            1_000_000,
            surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn spellcheck_capture(
    snapshot: &DomSnapshot,
    document: DocumentId,
    publisher: &DomHostPublisher,
) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            publisher.latest(),
            HtmlCaptureLimits::new(32, 128, 32, 8192, 32, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(32, 10_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}

#[test]
fn spellcheck_policy_and_canonical_design_mode_advance_only_their_own_revisions() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.set_design_mode(&document, "on");
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let inherited = batch.create_element_ns(&document, Namespace::html(), "span");
    let authored = batch.create_element_ns(&document, Namespace::html(), "span");
    let omitted = batch.create_element_ns(&document, Namespace::html(), "p");
    batch.set_attribute(&authored, "spellcheck", "");
    batch.insert_before(&root, &document, None);
    for child in [&inherited, &authored, &omitted] {
        batch.insert_before(child, &root, None);
    }
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let [root, inherited, authored, omitted] =
        [root, inherited, authored, omitted].map(|node| created.created(&node).unwrap().unwrap());
    let before = dom.snapshot();
    let view = before.document(document).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::SpellcheckDefaults);
    let limits = DomHostSpellcheckLimits::new(32, 256);
    let first = publisher
        .publish_spellcheck_defaults(
            view,
            view.stamp(),
            &[
                (root, TrueByDefault),
                (inherited, InheritByDefault),
                (authored, FalseByDefault),
            ],
            limits,
            &cancel,
        )
        .unwrap();
    let old = spellcheck_capture(&before, document, &publisher);
    let changed = [
        (authored, FalseByDefault),
        (root, FalseByDefault),
        (inherited, InheritByDefault),
    ];
    let second = publisher
        .publish_spellcheck_defaults(view, view.stamp(), &changed, limits, &cancel)
        .unwrap();
    assert_eq!(before.stamp(), dom.snapshot().stamp());
    assert_eq!(
        first.capture().stamp().document(),
        second.capture().stamp().document()
    );
    assert_eq!(
        first.capture().stamp().host_revision().get() + 1,
        second.capture().stamp().host_revision().get()
    );
    let DomHostChangeCoverage::Complete(delta) = second.changes() else {
        panic!("the single unchanged canonical basis permits complete policy invalidation");
    };
    assert_eq!(
        delta.changes().copied().collect::<Vec<_>>(),
        [DomHostChange::SpellcheckDefaults]
    );
    let current = spellcheck_capture(&before, document, &publisher);
    let rebuilt = with_slots(spellcheck_capture(&before, document, &publisher));
    assert_eq!(
        rebuilt.host().unwrap().stamp(),
        current.host().unwrap().stamp()
    );
    for (capture, expected) in [(&old, true), (&current, false)] {
        assert_eq!(
            fact(capture, inherited, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(expected))
        );
        assert_eq!(
            fact(capture, root, HtmlSemanticQuery::EditingHost),
            HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(capture, authored, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(capture, omitted, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
                HtmlSemanticMissing::SpellcheckDefault(capture.subject(omitted).unwrap())
            ))
        );
    }
    assert_eq!(
        fact(&rebuilt, inherited, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(false))
    );
    let reordered = [changed[2], changed[0], changed[1]];
    let equal = publisher
        .publish_spellcheck_defaults(view, view.stamp(), &reordered, limits, &cancel)
        .unwrap();
    assert_eq!(equal.capture(), second.capture());
    let mut batch = DomBatch::new();
    batch.set_design_mode(document, "off");
    assert_eq!(
        dom.apply(batch, &cancel).unwrap().changes().changes(),
        [DomChange::DocumentDesignMode(document)]
    );
    let after = dom.snapshot();
    let after_view = after.document(document).unwrap();
    assert!(matches!(
        publisher.publish_spellcheck_defaults(after_view, view.stamp(), &changed, limits, &cancel),
        Err(DomHostError::DomRevisionMismatch { .. })
    ));
    let rebased = publisher
        .publish_spellcheck_defaults(after_view, after_view.stamp(), &changed, limits, &cancel)
        .unwrap();
    assert_eq!(
        rebased.capture().stamp().host_revision(),
        second.capture().stamp().host_revision()
    );
    assert_eq!(rebased.capture().stamp().document(), after_view.stamp());
    let after = spellcheck_capture(&after, document, &publisher);
    assert_eq!(
        fact(&after, root, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        fact(&after, inherited, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        fact(&old, inherited, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true))
    );
}

#[test]
fn invalid_or_unadmitted_spellcheck_policy_keeps_the_previous_capture() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    batch.insert_before(&root, &document, None);
    let created = dom.apply(batch, &cancel).unwrap();
    let snapshot = dom.snapshot();
    let document = snapshot
        .document(created.document(&document).unwrap())
        .unwrap();
    let root = created.created(&root).unwrap().unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::SpellcheckDefaults);
    let limits = DomHostSpellcheckLimits::new(8, 128);
    let original = publisher
        .publish_spellcheck_defaults(
            document,
            document.stamp(),
            &[(root, TrueByDefault)],
            DomHostSpellcheckLimits::new(1, 128),
            &cancel,
        )
        .unwrap();
    for (entries, limits, cancel, expected) in [
        (
            vec![(root, InheritByDefault)],
            limits,
            false,
            DomHostSpellcheckError::InheritWithoutElementParent(root),
        ),
        (
            vec![(root, FalseByDefault), (root, TrueByDefault)],
            limits,
            false,
            DomHostSpellcheckError::DuplicateSubject(root),
        ),
        (
            vec![(root, FalseByDefault)],
            DomHostSpellcheckLimits::new(8, 2),
            false,
            DomHostSpellcheckError::Work { limit: 2 },
        ),
        (
            vec![(root, FalseByDefault)],
            DomHostSpellcheckLimits::new(0, 16),
            false,
            DomHostSpellcheckError::Capacity { limit: 0 },
        ),
        (
            vec![(root, FalseByDefault)],
            DomHostSpellcheckLimits::new(1, 128),
            false,
            DomHostSpellcheckError::Capacity { limit: 1 },
        ),
        (
            vec![(root, FalseByDefault)],
            limits,
            true,
            DomHostSpellcheckError::Cancelled,
        ),
    ] {
        assert_eq!(
            publisher.publish_spellcheck_defaults(
                document,
                document.stamp(),
                &entries,
                limits,
                &AtomicBool::new(cancel)
            ),
            Err(DomHostError::Spellcheck(expected))
        );
        assert_eq!(publisher.latest(), Some(original.capture()));
    }
    let mut foreign_dom = owner();
    let mut foreign = DomBatch::new();
    let foreign_document = foreign.create_document();
    let foreign_element = foreign.create_element_ns(&foreign_document, Namespace::html(), "div");
    let foreign = foreign_dom
        .apply(foreign, &cancel)
        .unwrap()
        .created(&foreign_element)
        .unwrap()
        .unwrap();
    assert_eq!(
        publisher.publish_spellcheck_defaults(
            document,
            document.stamp(),
            &[(foreign, FalseByDefault)],
            limits,
            &cancel
        ),
        Err(DomHostError::Subject {
            node: foreign,
            error: DomError::ForeignOwner
        })
    );
    assert_eq!(publisher.latest(), Some(original.capture()));
}

#[test]
fn design_mode_and_policy_rows_fit_the_shared_exact_subject_limit() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let elements = ["div", "p", "span"]
        .map(|name| batch.create_element_ns(&document, Namespace::html(), name));
    let created = dom.apply(batch, &cancel).unwrap();
    let document = created.document(&document).unwrap();
    let elements = elements.map(|element| created.created(&element).unwrap().unwrap());
    let snapshot = dom.snapshot();
    let view = snapshot.document(document).unwrap();
    let mut publisher = DomHostPublisher::new(DomHostDomains::SpellcheckDefaults);
    let defaults = elements.map(|element| (element, TrueByDefault));
    publisher
        .publish_spellcheck_defaults(
            view,
            view.stamp(),
            &defaults,
            DomHostSpellcheckLimits::new(3, 128),
            &cancel,
        )
        .unwrap();
    let capture = snapshot
        .html_capture(
            document,
            publisher.latest(),
            HtmlCaptureLimits::new(8, 64, 32, 8192, 8, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(4, 10_000),
            &cancel,
        )
        .unwrap();
    for element in elements {
        assert_eq!(
            fact(&capture, element, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(true))
        );
    }
}

#[test]
fn native_translation_mode_follows_ordinary_elements_and_reopens_current_state() {
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, Namespace::html(), "section");
    let foreign = batch.create_element_ns(&document, Namespace::svg(), "g");
    let child = batch.create_element_ns(&document, Namespace::html(), "span");
    batch.set_attribute(&root, "translate", "NO");
    batch.set_attribute(&foreign, "translate", "yes");
    batch.set_attribute(&child, "translate", " no");
    batch.insert_before(&root, &document, None);
    batch.insert_before(&foreign, &root, None);
    batch.insert_before(&child, &foreign, None);
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let child = commit.created(&child).unwrap().unwrap();
    let foreign = commit.created(&foreign).unwrap().unwrap();
    let old = dom.snapshot();
    let mode = |snapshot: &DomSnapshot, node| {
        snapshot.html_translation_mode_with(node, || Ok::<_, DomError>(()))
    };
    assert_eq!(mode(&old, child), Ok(HtmlTranslationMode::Disabled));
    assert_eq!(mode(&old, foreign), Ok(HtmlTranslationMode::Disabled));
    assert_eq!(mode(&old, document.node()), Err(DomError::WrongNodeKind));

    let mut steps = 0;
    let refused = old.html_translation_mode_with(child, || {
        steps += 1;
        if steps == 3 {
            Err(DomError::Cancelled)
        } else {
            Ok(())
        }
    });
    assert_eq!(refused, Err(DomError::Cancelled));
    assert_eq!(steps, 3);

    let mut batch = DomBatch::new();
    batch.set_attribute(child, "translate", "");
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        mode(&dom.snapshot(), child),
        Ok(HtmlTranslationMode::Enabled)
    );
    assert_eq!(mode(&old, child), Ok(HtmlTranslationMode::Disabled));

    let mut batch = DomBatch::new();
    batch.remove_attribute(child, "translate");
    batch.detach(child);
    dom.apply(batch, &cancel).unwrap();
    assert_eq!(
        mode(&dom.snapshot(), child),
        Ok(HtmlTranslationMode::Enabled)
    );
}

fn capture(snapshot: &DomSnapshot, document: DocumentId) -> DomHtmlCapture {
    snapshot
        .html_capture(
            document,
            None,
            HtmlCaptureLimits::new(512, 8192, 1024, 200_000, 512, 0, 0, 16_777_216),
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &AtomicBool::new(false),
        )
        .unwrap()
}
fn fact(capture: &DomHtmlCapture, node: NodeId, query: HtmlSemanticQuery) -> HtmlSemanticFact {
    let subject = capture.subject(node).unwrap();
    assert_eq!(capture.node(&subject).unwrap(), node);
    capture
        .query(
            &[HtmlSemanticRequest::new(subject, query)],
            HtmlValueInterpreter::new(8192),
            HtmlSemanticLimits::new(1_000_000, 16, 8192, 1024, 1_000_000, 8192),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .results()[0]
        .fact()
        .clone()
}
fn with_slots(capture: DomHtmlCapture) -> DomHtmlCapture {
    capture
        .with_slot_semantics(
            surgeist_dom::capture::DomSlotSemanticLimits::new(
                surgeist_dom::shadow::DomSlotQueryLimits::new(512, 4096).unwrap(),
                DomHtmlCaptureLimits::new(512, 1_000_000),
                surgeist_html_semantics::HtmlLanguageDirectionLimits::new(
                    512, 512, 200_000, 1_000_000,
                ),
                surgeist_html_semantics::HtmlInertFactLimits::new(512, 1_000_000),
            ),
            &AtomicBool::new(false),
        )
        .unwrap()
}

#[test]
fn every_native_document_creator_establishes_its_own_disabled_state_and_snapshots_retain_it() {
    let mut dom = owner();
    let cancel = AtomicBool::new(false);
    let mut b = DomBatch::new();
    let native = b.create_document();
    let html = b.create_html_document(&native, None);
    let xml = b.create_xml_document(&html, Namespace::html(), "root", None);
    let template = b.create_element_ns(&native, Namespace::html(), "template");
    let done = dom.apply(b, &cancel).unwrap();
    let native = done.document(&native).unwrap();
    let html = done.document(&html).unwrap();
    let xml = done.document(&xml).unwrap();
    let old = dom.snapshot();
    let contents = old
        .node(done.created(&template).unwrap().unwrap())
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = old.node(contents).unwrap().owner_document().unwrap();
    assert_ne!(inert, native);
    for document in [native, html, xml, inert] {
        assert!(
            !old.document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
    let mut b = DomBatch::new();
    b.create_comment(native, "unrelated revision");
    dom.apply(b, &cancel).unwrap();
    assert_ne!(old.stamp(), dom.snapshot().stamp());
    for document in [native, html, xml, inert] {
        assert!(
            !dom.snapshot()
                .document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
        assert!(
            !old.document(document)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
}

#[test]
fn ordinary_editability_does_not_cross_shadow_or_template_boundaries_and_survives_slot_rebuild() {
    use surgeist_html::construction::{
        HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
    };
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let host = b.create_element_ns(&d, Namespace::html(), "div");
    b.set_attribute(&host, "contenteditable", "TrUe");
    b.insert_before(&host, &d, None);
    let child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&child, &host, None);
    let blocked = b.create_element_ns(&d, Namespace::html(), "span");
    b.set_attribute(&blocked, "contenteditable", "false");
    b.insert_before(&blocked, &host, None);
    let foreign = b.create_element_ns(&d, Namespace::none(), "div");
    b.set_attribute(&foreign, "contenteditable", "true");
    b.insert_before(&foreign, &host, None);
    let svg = b.create_element_ns(&d, Namespace::svg(), "svg");
    b.insert_before(&svg, &host, None);
    let detached = b.create_element_ns(&d, Namespace::html(), "div");
    b.set_attribute(&detached, "contenteditable", "plaintext-only");
    let shadow = b.attach_shadow(
        &host,
        HtmlShadowRootProperties::new(
            HtmlShadowRootMode::Closed,
            HtmlSlotAssignment::Named,
            false,
            false,
            false,
        ),
    );
    let shadow_child = b.create_element_ns(&d, Namespace::html(), "span");
    b.insert_before(&shadow_child, &shadow, None);
    let template = b.create_element_ns(&d, Namespace::html(), "template");
    b.insert_before(&template, &host, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let id = |ticket: &surgeist_dom::edit::CreatedNode| done.created(ticket).unwrap().unwrap();
    let (host, child, blocked, foreign, svg, detached, shadow_child, template) = (
        id(&host),
        id(&child),
        id(&blocked),
        id(&foreign),
        id(&svg),
        id(&detached),
        id(&shadow_child),
        id(&template),
    );
    let snapshot = dom.snapshot();
    let contents = snapshot
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let inert = snapshot.node(contents).unwrap().owner_document().unwrap();
    let mut b = DomBatch::new();
    let template_child = b.create_element_ns(inert, Namespace::html(), "span");
    b.insert_before(&template_child, contents, None);
    let done = dom.apply(b, &cancel).unwrap();
    let template_child = done.created(&template_child).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let stamp = snapshot.stamp();
    for (node, expected) in [
        (host, true),
        (child, true),
        (blocked, false),
        (detached, true),
        (shadow_child, false),
        (template_child, false),
    ] {
        assert_eq!(
            snapshot.html_is_content_editable_with(node, || Ok::<_, DomError>(())),
            Ok(expected)
        );
    }
    assert_eq!(
        snapshot.html_is_content_editable_with(foreign, || Ok::<_, DomError>(())),
        Err(DomError::WrongNodeKind)
    );
    let mut steps = 0;
    let refused = snapshot.html_is_content_editable_with(child, || {
        steps += 1;
        if steps == 3 {
            Err(DomError::Cancelled)
        } else {
            Ok(())
        }
    });
    assert_eq!(refused, Err(DomError::Cancelled));
    assert_eq!(steps, 3);
    for c in [capture(&snapshot, d), with_slots(capture(&snapshot, d))] {
        for node in [host, detached] {
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::EditingHost),
                HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(true))
            );
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::IsContentEditable),
                HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
            );
        }
        assert_eq!(
            fact(&c, child, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(true))
        );
        assert_eq!(
            fact(&c, svg, HtmlSemanticQuery::Editable),
            HtmlSemanticFact::Editable(HtmlSemanticValue::Known(true))
        );
        for node in [blocked, foreign, shadow_child, template_child] {
            assert_eq!(
                fact(&c, node, HtmlSemanticQuery::Editable),
                HtmlSemanticFact::Editable(HtmlSemanticValue::Known(false))
            );
        }
        assert!(
            !c.source()
                .document(inert)
                .unwrap()
                .metadata()
                .design_mode_enabled()
        );
    }
    assert_eq!(dom.snapshot().stamp(), stamp);
}

#[test]
fn explicit_spellcheck_and_contenteditable_mutation_preserve_old_captures_without_policy_defaults()
{
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let e = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&e, &d, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let e = done.created(&e).unwrap().unwrap();
    let old_snapshot = dom.snapshot();
    let old = capture(&old_snapshot, d);
    assert!(matches!(
        fact(&old, e, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
    for (value, expected) in [("", true), ("TrUe", true), ("FALSE", false)] {
        let mut b = DomBatch::new();
        b.set_attribute(e, "spellcheck", value);
        b.set_attribute(e, "contenteditable", "plaintext-only");
        dom.apply(b, &cancel).unwrap();
        assert_eq!(
            dom.snapshot()
                .html_is_content_editable_with(e, || Ok::<_, DomError>(())),
            Ok(true)
        );
        let current = capture(&dom.snapshot(), d);
        assert_eq!(
            fact(&current, e, HtmlSemanticQuery::Spellcheck),
            HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Known(expected))
        );
        assert_eq!(
            fact(
                &with_slots(current),
                e,
                HtmlSemanticQuery::IsContentEditable
            ),
            HtmlSemanticFact::IsContentEditable(HtmlSemanticValue::Known(true))
        );
    }
    let mut b = DomBatch::new();
    b.set_attribute(e, "spellcheck", "unknown");
    dom.apply(b, &cancel).unwrap();
    assert!(matches!(
        fact(
            &capture(&dom.snapshot(), d),
            e,
            HtmlSemanticQuery::Spellcheck
        ),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
    assert_eq!(
        fact(&old, e, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert_eq!(
        old_snapshot.html_is_content_editable_with(e, || Ok::<_, DomError>(())),
        Ok(false)
    );
    assert!(matches!(
        fact(&old, e, HtmlSemanticQuery::Spellcheck),
        HtmlSemanticFact::Spellcheck(HtmlSemanticValue::Missing(
            HtmlSemanticMissing::SpellcheckDefault(_)
        ))
    ));
}

fn parse_limits() -> surgeist_html_parser::HtmlParseLimits {
    surgeist_html_parser::HtmlParseLimits::builder()
        .source_units(4096)
        .buffered_utf16_units(4096)
        .token_attributes(1024)
        .auxiliary_entries(100_000)
        .active_formatting_entries(100_000)
        .managed_nodes(1024)
        .managed_edges(1024)
        .managed_attributes(1024)
        .managed_utf16_units(65536)
        .source_map_entries(65536)
        .source_map_payload_units(65536)
        .diagnostics(1024)
        .diagnostic_payload_units(65536)
        .context_effects(1024)
        .open_elements(64)
        .tree_depth(128)
        .managed_capacity_bytes(16_000_000)
        .peak_transient_bytes(16_000_000)
        .atomic_layout_bytes(16_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap()
}

#[test]
fn actual_reports_transfer_separate_owned_document_states_and_preserve_captured_target() {
    use std::sync::Arc;
    use surgeist_dom::{
        document::DomNativeInputPlatform,
        installation::{DomParsedDocumentEnvironment, DomParsedHtmlInstallLimits},
    };
    use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlSource};
    use surgeist_html_semantics::{
        HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
        HtmlBaseUrlScheme, HtmlFragmentContext,
    };
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let native = b.create_document();
    let target = b.create_element_ns(&native, Namespace::html(), "div");
    b.insert_before(&target, &native, None);
    let done = dom.apply(b, &cancel).unwrap();
    let native = done.document(&native).unwrap();
    let target = done.created(&target).unwrap().unwrap();
    let before = dom.snapshot();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 1, 1),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        before.document(native).unwrap().metadata().origin().clone(),
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    let report = Arc::new(
        HtmlDocumentParseRequest::offline(
            HtmlSource::utf8("<!doctype html><template><span></span></template>"),
            parse_limits(),
        )
        .parse()
        .unwrap(),
    );
    let independent =
        HtmlDocumentParseRequest::offline(HtmlSource::utf8("<div></div>"), parse_limits())
            .parse()
            .unwrap();
    assert_eq!(
        report
            .document()
            .unwrap()
            .root()
            .document_design_mode_enabled(),
        Some(false)
    );
    let installation = dom
        .install_parsed_html_document(
            before.stamp(),
            report.clone(),
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    assert!(
        installation
            .node_for(&independent.document().unwrap().root().id())
            .is_none()
    );
    let original = installation.document();
    let snapshot = dom.snapshot();
    let installed_documents: Vec<_> = snapshot
        .nodes()
        .filter_map(|n| snapshot.node(n).unwrap().as_document())
        .filter(|d| d.id() != native)
        .collect();
    assert_eq!(installed_documents.len(), 2); // ordinary + distinct inert Template owner
    for document in &installed_documents {
        assert!(!document.metadata().design_mode_enabled());
    }
    let fresh_capture = capture(&snapshot, native);
    let context = HtmlFragmentContext::element(
        fresh_capture.capture(),
        &fresh_capture.subject(target).unwrap(),
    )
    .unwrap();
    let fragment = Arc::new(
        HtmlFragmentParseRequest::new(
            HtmlSource::utf8("<script>inert</script><style>a{}</style>"),
            context,
            parse_limits(),
        )
        .unwrap()
        .parse()
        .unwrap(),
    );
    let descriptor = fragment.fragment_parser_document().unwrap().unwrap();
    assert_eq!(
        descriptor.document().document_design_mode_enabled(),
        Some(false)
    );
    let auxiliary_source = descriptor.document().id();
    let installed = dom
        .install_parsed_html_fragment(
            fragment,
            &fresh_capture,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &cancel,
        )
        .unwrap();
    let auxiliary = installed
        .snapshot()
        .node(installed.node_for(&auxiliary_source).unwrap())
        .unwrap()
        .as_document()
        .unwrap()
        .id();
    assert_ne!(auxiliary, native);
    assert_ne!(auxiliary, original);
    assert!(
        !installed
            .snapshot()
            .document(auxiliary)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert_eq!(
        installed.captured_node_for(&fresh_capture.subject(native.node()).unwrap()),
        Some(native.node())
    );
    assert!(
        !installed
            .snapshot()
            .document(native)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
    assert!(
        !before
            .document(native)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
}

#[test]
fn failed_or_cancelled_capture_does_not_mutate_canonical_state_and_good_limits_recover() {
    use surgeist_dom::capture::{DomHtmlCaptureError, DomHtmlCaptureResource};
    let cancel = AtomicBool::new(false);
    let mut dom = owner();
    let mut b = DomBatch::new();
    let d = b.create_document();
    let root = b.create_element_ns(&d, Namespace::html(), "div");
    b.insert_before(&root, &d, None);
    let done = dom.apply(b, &cancel).unwrap();
    let d = done.document(&d).unwrap();
    let root = done.created(&root).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let stamp = snapshot.stamp();
    let usage = dom.usage();
    let limits = HtmlCaptureLimits::new(512, 8192, 1024, 200_000, 512, 0, 0, 16_777_216);
    assert!(matches!(
        snapshot.html_capture(
            d,
            None,
            limits,
            DomHtmlCaptureLimits::new(1, 1_000_000),
            &cancel
        ),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Subjects,
            ..
        })
    ));
    assert!(matches!(
        snapshot.html_capture(d, None, limits, DomHtmlCaptureLimits::new(512, 0), &cancel),
        Err(DomHtmlCaptureError::ResourceLimit {
            resource: DomHtmlCaptureResource::Work,
            ..
        })
    ));
    assert!(matches!(
        snapshot.html_capture(
            d,
            None,
            limits,
            DomHtmlCaptureLimits::new(512, 1_000_000),
            &AtomicBool::new(true)
        ),
        Err(DomHtmlCaptureError::Cancelled)
    ));
    assert_eq!(
        fact(&capture(&snapshot, d), root, HtmlSemanticQuery::EditingHost),
        HtmlSemanticFact::EditingHost(HtmlSemanticValue::Known(false))
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(dom.usage(), usage);
    assert!(
        !dom.snapshot()
            .document(d)
            .unwrap()
            .metadata()
            .design_mode_enabled()
    );
}

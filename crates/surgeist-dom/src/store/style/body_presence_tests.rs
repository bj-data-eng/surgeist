//! Actual body discovery after a genuine acknowledged CSSOM publication.
use super::*;
use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest};
use std::cell::Cell;

#[derive(Clone, Copy, Default, Debug)]
struct Reads {
    calls: usize,
    work: u64,
}
thread_local! {
    static READS: Cell<Option<Reads>> = const { Cell::new(None) };
}
pub(super) struct BodyRead<'a> {
    draft: &'a Draft,
    start: u64,
}
impl<'a> BodyRead<'a> {
    pub(super) fn begin(draft: &'a Draft) -> Self {
        Self {
            draft,
            start: draft.source_work.load(Ordering::Relaxed),
        }
    }
}
impl Drop for BodyRead<'_> {
    fn drop(&mut self) {
        if let Some(mut reads) = READS.get() {
            reads.calls += 1;
            reads.work += self.draft.source_work.load(Ordering::Relaxed) - self.start;
            READS.set(Some(reads));
        }
    }
}
struct Observation;
impl Observation {
    fn begin() -> Self {
        assert!(READS.get().is_none());
        READS.set(Some(Reads::default()));
        Self
    }
    fn finish(self) -> Reads {
        READS.take().unwrap()
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        READS.set(None);
    }
}
fn policy() -> DomSourceLimits {
    DomSourceLimits::new(256_000_000, 4_000_000, 100_000_000, 64, 4096, 4096, 1)
}

// Adequate fixture configuration, not an operation-cost oracle: existing owner
// storage is already controlled, and the original fixed 256 MB is additional
// preparation/old-new overlap headroom. Work and discovery assertions remain
// independent of these capacity-based storage inputs.
fn owner_preparation_policy(dom: &Dom) -> crate::DomSourceLimits {
    let baseline = dom.custody_registry.ledger().total().bytes();
    let mut configured = policy();
    let additional = configured.retained_bytes;
    configured.retained_bytes = baseline
        .checked_add(additional)
        .expect("fixture owner baseline and preparation allowance");
    eprintln!(
        "discovery fixture storage: preoperation_owner={baseline}, additional={additional}, total={}",
        configured.retained_bytes
    );
    configured
}

#[test]
#[ignore = "requires an instrumented DOM build; isolates post-CSSOM body discovery"]
fn acknowledged_style_creation_discovers_body_without_scanning_unrelated_children() {
    use crate::{DomStyleCssomOwnerBinding, DomStyleCssomRoot, DomStyleInlineCspObservation};
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};
    // The fixed html/head/body classifiers read at most three namespace
    // headers plus their literal 28-unit HTML URI, and three local-name tests
    // bounded by headers + html(4), body(4), frameset(8). Together with four
    // fixed child visits this is 3*(1+28+5+5+9)+4 = 148 actual steps. A fixed
    // 256 envelope admits those participants without pricing a scan from its
    // observation. At unrelated=256, the two comment lists alone add 512 reads.
    const FIXED_BODY_WORK: u64 = 256;
    scale_metrics::require_enabled().expect("compile with SURGEIST_DOM_STORAGE_COUNTERS=1");
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256, 4096] {
        for body_present in [false, true] {
            let mut dom = Dom::new(
                DomLimits::new(
                    20_000,
                    4_000_000,
                    32_768,
                    0,
                    0,
                    16,
                    4_000_000,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let mut batch = DomBatch::new();
            let source = batch.create_document();
            let ticket = batch.create_html_document(&source, None);
            let setup = dom.apply(batch, &cancel).unwrap();
            let document = setup.document(&ticket).unwrap();
            let snapshot = dom.snapshot();
            let html = snapshot.node(document.node()).unwrap().children()[1];
            let children = snapshot.node(html).unwrap().children();
            let (head, body) = (children[0], children[1]);
            let mut batch = DomBatch::new();
            if !body_present {
                batch.detach(body);
            }
            for _ in 0..unrelated {
                let before_root = batch.create_comment(document, "outside");
                batch.insert_before(&before_root, document.node(), Some(html.into()));
                let before_body = batch.create_comment(document, "inside");
                batch.insert_before(&before_body, html, body_present.then_some(body.into()));
            }
            let style = batch.create_element_ns(document, crate::Namespace::html(), "style");
            batch.set_attribute(&style, "blocking", "render");
            let text = batch.create_text(document, "a {}");
            batch.insert_before(&text, &style, None);
            let setup = dom.apply(batch, &cancel).unwrap();
            let style = setup.created(&style).unwrap().unwrap();
            let old = dom.snapshot();
            let command_policy = owner_preparation_policy(&dom);
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::insert_before(style, head, None),
                    command_policy,
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(csp)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("real CSP request")
            };
            lease
                .accept_inline_csp(
                    &csp,
                    &DomStyleInlineCspObservation::complete_empty(csp.source(), 1),
                    &cancel,
                )
                .unwrap();
            let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("real CSSOM creation request")
            };
            assert_eq!(
                create.action(),
                crate::DomStyleAction::Update(Action::CreateSheet)
            );
            let root = DomStyleCssomRoot::new(
                document.node(),
                surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
            );
            lease.bind_cssom_root(&create, &root, &cancel).unwrap();
            let create = lease
                .qualify_cssom(&create, root.root(), &root.snapshot().unwrap())
                .unwrap();
            let version = CssomInputVersion {
                role: CssomInputRole::Owner,
                identity: "body-presence".into(),
                revision: 1,
            };
            lease
                .bind_cssom_owner(
                    &create,
                    DomStyleCssomOwnerBinding::new(&create, version.clone(), "inline".into())
                        .unwrap(),
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
            let mut edit = root
                .batch(CssomEditGuard::from_snapshot(&root.snapshot().unwrap()))
                .unwrap();
            let construction = edit
                .create_parsed_sheet_with_media_reports(report, media, inputs)
                .unwrap();
            let commit = root.commit(&mut Some(edit)).unwrap();
            let _notification = lease
                .accept_cssom_publication(
                    &create,
                    commit,
                    root.snapshot().unwrap(),
                    Some(construction.ticket()),
                )
                .unwrap();
            // Capture, parsing, receipt admission, and required earlier native
            // hooks have finished. Observe only the actual resumed body query.
            let observation = Observation::begin();
            let result = lease.advance(&cancel).unwrap();
            let reads = observation.finish();
            assert!(matches!(result, DomSourceProgress::Complete(_)));
            let completed = lease.snapshot();
            assert_eq!(
                completed
                    .node(style)
                    .unwrap()
                    .as_style()
                    .unwrap()
                    .is_render_blocking()
                    .unwrap(),
                !body_present
            );
            assert_eq!(old.node(style).unwrap().parent(), None);
            assert_eq!(completed.node(style).unwrap().parent(), Some(head));
            assert!(
                completed
                    .node(style)
                    .unwrap()
                    .as_style()
                    .unwrap()
                    .sheet()
                    .unwrap()
                    .is_some()
            );
            assert!(
                reads.calls > 0,
                "actual body discovery must execute: {reads:?}"
            );
            assert!(
                reads.work <= FIXED_BODY_WORK,
                "fixed body participants, unrelated={unrelated}, body={body_present}: {reads:?}"
            );
        }
    }
}

#[test]
fn absent_render_blocking_removal_ignores_other_roles_and_preserves_duplicate_occurrences() {
    let cancel = AtomicBool::new(false);
    for unrelated in [0, 256] {
        let mut dom = Dom::new(
            DomLimits::new(
                1024,
                1_000_000,
                2048,
                0,
                0,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap()
            .with_autofocus_candidates(16)
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let absent = batch.create_element_ns(&document, crate::Namespace::html(), "style");
        let anchor = batch.create_element_ns(&document, crate::Namespace::html(), "style");
        let tickets: Vec<_> = (0..unrelated)
            .map(|_| batch.create_element_ns(&document, crate::Namespace::html(), "style"))
            .collect();
        let setup = dom.apply(batch, &cancel).unwrap();
        let document = setup.document(&document).unwrap();
        let absent = setup.created(&absent).unwrap().unwrap();
        let anchor = setup.created(&anchor).unwrap().unwrap();
        let mut members = vec![anchor, anchor];
        members.extend(
            tickets
                .iter()
                .map(|ticket| setup.created(ticket).unwrap().unwrap()),
        );
        let batch = DomBatch::new();
        let (mut staged, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            policy(),
            &cancel,
        )
        .unwrap();
        let owner = staged.resolve(document.into()).unwrap();
        let keys: Vec<_> = members
            .iter()
            .map(|id| staged.resolve((*id).into()).unwrap())
            .collect();
        // Genuine typed relation-owner fixture. This does not assert these
        // detached Styles completed CSP/CSSOM or became rendering resources.
        staged
            .relations_insert(
                owner,
                transaction::RelationField::RenderBlocking,
                0,
                &keys,
                &cancel,
            )
            .unwrap();
        let absent_key = staged.resolve(absent.into()).unwrap();
        staged
            .autofocus_append(owner, absent_key, dom.limits, &cancel)
            .unwrap();
        dom.publish(staged.finish(&dom, &cancel).unwrap(), &cancel)
            .unwrap();
        let old = dom.snapshot();
        let batch = DomBatch::new();
        let (mut staged, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            policy(),
            &cancel,
        )
        .unwrap();
        let owner = staged.resolve(document.into()).unwrap();
        let subject = staged.resolve(absent.into()).unwrap();
        let before: Vec<_> = staged
            .node(owner)
            .unwrap()
            .base_document()
            .as_ref()
            .unwrap()
            .render_blocking
            .iter()
            .collect();
        let spent = staged.source_work.load(Ordering::Relaxed);
        let mut tight = policy();
        tight.work = spent + 64;
        staged.source_limits = Some(tight);
        let result = staged.style_unblock_rendering(subject, &cancel);
        assert_eq!(
            staged
                .node(owner)
                .unwrap()
                .base_document()
                .as_ref()
                .unwrap()
                .render_blocking
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        staged.source_limits = Some(policy());
        staged.style_unblock_rendering(subject, &cancel).unwrap();
        assert_eq!(
            staged
                .node(owner)
                .unwrap()
                .base_document()
                .as_ref()
                .unwrap()
                .render_blocking
                .iter()
                .collect::<Vec<_>>(),
            before
        );
        assert!(
            old.node(anchor)
                .unwrap()
                .as_style()
                .unwrap()
                .is_render_blocking()
                .unwrap()
        );
        assert!(
            !old.node(absent)
                .unwrap()
                .as_style()
                .unwrap()
                .is_render_blocking()
                .unwrap()
        );
        assert_eq!(
            old.document(document)
                .unwrap()
                .autofocus()
                .unwrap()
                .candidates(),
            &[absent]
        );
        assert_eq!(dom.snapshot().stamp(), old.stamp());
        assert_eq!(
            result,
            Ok(()),
            "absent rendering participant, unrelated={unrelated}"
        );
    }
}

#[test]
fn body_presence_uses_the_first_document_element_and_only_direct_html_children() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(
            128,
            100_000,
            256,
            0,
            0,
            16,
            1_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let source = batch.create_document();
    let ticket = batch.create_html_document(&source, None);
    let setup = dom.apply(batch, &cancel).unwrap();
    let document = setup.document(&ticket).unwrap();
    let old = dom.snapshot();
    let html = old.node(document.node()).unwrap().children()[1];
    let body = old.node(html).unwrap().children()[1];
    let check = |dom: &Dom, expected| {
        let batch = DomBatch::new();
        let (staged, _) =
            Draft::new_source(dom, batch.owner.clone(), &batch.commands, policy(), &cancel)
                .unwrap();
        assert_eq!(
            staged
                .style_document_has_body(staged.resolve(document.into()).unwrap(), &cancel)
                .unwrap(),
            expected
        );
    };
    check(&dom, true);
    let mut batch = DomBatch::new();
    let container = batch.create_element_ns(document, crate::Namespace::html(), "div");
    batch.insert_before(&container, html, None);
    batch.insert_before(body, &container, None);
    dom.apply(batch, &cancel).unwrap();
    check(&dom, false); // A nested body is not the Document's body.
    let mut batch = DomBatch::new();
    batch.insert_before(body, html, None);
    dom.apply(batch, &cancel).unwrap();
    check(&dom, true);
    let mut batch = DomBatch::new();
    batch.detach(html);
    let xml = batch.create_element_ns(document, crate::Namespace::none(), "html");
    batch.insert_before(&xml, document.node(), None);
    batch.insert_before(body, &xml, None);
    dom.apply(batch, &cancel).unwrap();
    check(&dom, false); // Same spelling with the wrong namespace is not HTML.
    assert_eq!(old.node(body).unwrap().parent(), Some(html));
    assert_eq!(old.node(html).unwrap().parent(), Some(document.node()));
}

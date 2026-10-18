//! Actual owner-custody visits, separate from canonical node/Source work.
//! These counters observe the reached owner path, never a substitute bank.

use std::cell::Cell;

use super::*;
use crate::style::{DomStyleCssomRoot, DomStyleInlineCspObservation, DomStyleRequest};
use crate::{DomSourceEdit, DomSourceLimits, DomSourceProgress, DomSourceRequest, DomStyleAction};

#[derive(Clone, Copy)]
pub(in crate::store) enum Site {
    Publication,
    Retention,
    Lookup,
    BindingLookup,
    BindingCopy,
    LeaseLookup,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Visits {
    publication: usize,
    retention: usize,
    lookup: usize,
    binding_lookup: usize,
    binding_copy: usize,
    lease_lookup: usize,
}

thread_local! {
    static VISITS: Cell<Option<Visits>> = const { Cell::new(None) };
}

pub(in crate::store) fn observe(site: Site) {
    let Some(mut visits) = VISITS.get() else {
        return;
    };
    let count = match site {
        Site::Publication => &mut visits.publication,
        Site::Retention => &mut visits.retention,
        Site::Lookup => &mut visits.lookup,
        Site::BindingLookup => &mut visits.binding_lookup,
        Site::BindingCopy => &mut visits.binding_copy,
        Site::LeaseLookup => &mut visits.lease_lookup,
    };
    *count += 1;
    VISITS.set(Some(visits));
}

struct Observation;
impl Observation {
    fn start() -> Self {
        assert!(VISITS.replace(Some(Visits::default())).is_none());
        Self
    }

    fn finish(self) -> Visits {
        VISITS.take().expect("active custody observation")
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        VISITS.set(None);
    }
}

struct Fixture {
    dom: Dom,
    document: DocumentId,
    element: NodeId,
    style: NodeId,
    owners: Vec<DomStyleCssomRoot>,
    target: Option<DomStyleCssomRoot>,
}

fn cssom_root(root: NodeId) -> DomStyleCssomRoot {
    DomStyleCssomRoot::new(
        root,
        surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
    )
}

fn fixture(unrelated: usize, with_target: bool) -> Fixture {
    let cancel = AtomicBool::new(false);
    let mut dom =
        Dom::new(DomLimits::new(1024, 1_000_000, 4096, 0, 0, 8, 1_000_000).unwrap()).unwrap();
    let mut batch = DomBatch::new();
    // Target identity and topology stay fixed when unrelated roots increase.
    let document = batch.create_document();
    let element = batch.create_element_ns(&document, crate::Namespace::html(), "div");
    let style = batch.create_element_ns(&document, crate::Namespace::html(), "style");
    let others = (0..unrelated)
        .map(|_| batch.create_document())
        .collect::<Vec<_>>();
    let commit = dom.apply(batch, &cancel).unwrap();
    let document = commit.document(&document).unwrap();
    let element = commit.created(&element).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let mut owners = Vec::new();
    // This private fixture supplies real mutable CSSOM owners of actual live
    // canonical Documents. No CSSOM operation or association is fabricated.
    for ticket in others {
        let owner = cssom_root(commit.document(&ticket).unwrap().node());
        dom.style_cssom_roots.push(owner.duplicate());
        owners.push(owner);
    }
    let target = with_target.then(|| {
        let owner = cssom_root(document.node());
        dom.style_cssom_roots.push(owner.duplicate());
        owner
    });
    Fixture {
        dom,
        document,
        element,
        style,
        owners,
        target,
    }
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 50_000_000, 64, 4096, 4096, 1)
}

fn creation_request(
    lease: &mut source::operation::DomSourceLease<'_>,
    cancel: &AtomicBool,
) -> DomStyleRequest {
    let DomSourceProgress::Request(DomSourceRequest::Style(policy)) =
        lease.advance(cancel).unwrap()
    else {
        panic!("actual inline policy boundary")
    };
    lease
        .accept_inline_csp(
            &policy,
            &DomStyleInlineCspObservation::complete_empty(policy.source(), 1),
            cancel,
        )
        .unwrap();
    let DomSourceProgress::Request(DomSourceRequest::Style(create)) =
        lease.advance(cancel).unwrap()
    else {
        panic!("actual sheet creation boundary")
    };
    assert_eq!(
        create.action(),
        DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CreateSheet)
    );
    create
}

fn complete_sheet(
    lease: &mut source::operation::DomSourceLease<'_>,
    request: &DomStyleRequest,
    owner: &DomStyleCssomRoot,
    cancel: &AtomicBool,
) {
    use surgeist_css::{CssParserContext, CssUtf16InputLimits};
    use surgeist_cssom::{CssomEditGuard, CssomInputRole, CssomInputVersion, CssomSheetInputs};

    let request = lease
        .qualify_cssom(request, owner.root(), &owner.snapshot().unwrap())
        .unwrap();
    let version = CssomInputVersion {
        role: CssomInputRole::Owner,
        identity: "custody-scale-style".into(),
        revision: 1,
    };
    lease
        .bind_cssom_owner(
            &request,
            crate::DomStyleCssomOwnerBinding::new(&request, version.clone(), "actual-style".into())
                .unwrap(),
            cancel,
        )
        .unwrap();
    let parser = CssParserContext::default();
    let body = parser
        .parse_sheet_utf16_with_limits(
            request.source().text().clone(),
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
    inputs.owner = Some("actual-style".into());
    let mut edit = owner
        .batch(CssomEditGuard::from_snapshot(&owner.snapshot().unwrap()))
        .unwrap();
    let sheet = edit
        .create_parsed_sheet_with_media_reports(body, media, inputs)
        .unwrap();
    let publication = owner.commit(&mut Some(edit)).unwrap();
    let actual = publication.sheet(sheet.ticket()).unwrap().clone();
    let notification = lease
        .accept_cssom_publication(
            &request,
            publication,
            owner.snapshot().unwrap(),
            Some(sheet.ticket()),
        )
        .unwrap();
    assert!(notification.is_some());
    assert_eq!(owner.snapshot().unwrap().sheets(), &[actual]);
    assert!(matches!(
        lease.advance(cancel).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

#[test]
fn local_attribute_publication_does_not_visit_unrelated_style_owners() {
    let cancel = AtomicBool::new(false);
    let mut samples = Vec::new();
    for unrelated in [0, 128] {
        let mut fixture = fixture(unrelated, true);
        let original = fixture.dom.snapshot();
        let owner = fixture.target.as_ref().unwrap();
        let before = owner.snapshot().unwrap();
        let mut batch = DomBatch::new();
        batch.set_attribute(fixture.element, "data-local", "changed");
        let observation = Observation::start();
        let commit = fixture.dom.apply(batch, &cancel).unwrap();
        let visits = observation.finish();
        assert_eq!(
            commit.changes().changes(),
            &[DomChange::Attributes(fixture.element)]
        );
        assert!(
            original
                .node(fixture.element)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into())
                .is_none()
        );
        assert_eq!(
            fixture
                .dom
                .snapshot()
                .node(fixture.element)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"data-local".into()),
            Some(&DocumentString::from("changed"))
        );
        let retained = fixture
            .dom
            .style_cssom_root(fixture.document.node())
            .unwrap()
            .unwrap();
        assert!(retained.same_owner(owner));
        assert_eq!(retained.snapshot().unwrap().revision(), before.revision());
        for other in &fixture.owners {
            assert!(
                fixture
                    .dom
                    .style_cssom_root(other.root())
                    .unwrap()
                    .unwrap()
                    .same_owner(other)
            );
        }
        samples.push((visits.publication, visits.retention));
    }
    assert_eq!(
        samples,
        [(0, 0), (0, 0)],
        "an attribute publication has no Style retirement dependency"
    );
}

#[test]
fn exact_style_owner_lookup_visits_only_the_requested_owner() {
    let mut samples = Vec::new();
    for unrelated in [0, 128] {
        let fixture = fixture(unrelated, true);
        let observation = Observation::start();
        let retained = fixture
            .dom
            .style_cssom_root(fixture.document.node())
            .unwrap()
            .unwrap();
        let visits = observation.finish();
        assert!(retained.same_owner(fixture.target.as_ref().unwrap()));
        samples.push(visits.lookup);
    }
    assert_eq!(
        samples,
        [1, 1],
        "lookup must select the exact root instead of inspecting unrelated owners"
    );
}

#[test]
fn real_style_binding_and_lease_lookup_do_not_copy_or_visit_unrelated_owners() {
    let cancel = AtomicBool::new(false);
    let mut samples = Vec::new();
    for unrelated in [0, 128] {
        let Fixture {
            mut dom,
            document,
            style,
            owners,
            ..
        } = fixture(unrelated, false);
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, document.node(), None),
                source_limits(),
                &cancel,
            )
            .unwrap();
        let request = creation_request(&mut lease, &cancel);
        assert_eq!(request.cssom_root(), Some(document.node()));
        let owner = cssom_root(document.node());
        let observation = Observation::start();
        lease.bind_cssom_root(&request, &owner, &cancel).unwrap();
        let binding = observation.finish();
        let observation = Observation::start();
        let retained = lease.cssom_root(&request).unwrap();
        let lookup = observation.finish();
        assert!(retained.same_owner(&owner));
        // Exact owner identity remains authoritative even at equal revisions.
        lease.bind_cssom_root(&request, &owner, &cancel).unwrap();
        let foreign = cssom_root(document.node());
        assert_eq!(
            lease
                .bind_cssom_root(&request, &foreign, &cancel)
                .unwrap_err(),
            DomError::StyleReceiptMismatch
        );
        complete_sheet(&mut lease, &request, &retained, &cancel);
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert!(
            dom.style_cssom_root(document.node())
                .unwrap()
                .unwrap()
                .same_owner(&owner)
        );
        for other in owners {
            assert!(
                dom.style_cssom_root(other.root())
                    .unwrap()
                    .unwrap()
                    .same_owner(&other)
            );
        }
        samples.push((
            binding.binding_lookup,
            binding.binding_copy,
            lookup.lease_lookup,
        ));
    }
    assert_eq!(
        samples,
        [(0, 0, 1), (0, 0, 1)],
        "binding and reissuing one mutable owner must not inspect or copy unrelated owner records"
    );
}

#[test]
fn retirement_releases_only_the_exact_generation_and_preserves_external_owner_custody() {
    use surgeist_cssom::CssomEditGuard;

    let cancel = AtomicBool::new(false);
    let Fixture {
        mut dom,
        document,
        element,
        owners,
        ..
    } = fixture(8, false);
    let mut batch = DomBatch::new();
    let shadow = batch.attach_shadow(
        element,
        crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
    );
    let commit = dom.apply(batch, &cancel).unwrap();
    let shadow = commit.created(&shadow).unwrap().unwrap();
    let original = cssom_root(shadow);
    dom.style_cssom_roots.push(original.duplicate());
    let historical = dom.snapshot();
    let retained = dom.style_cssom_root(shadow).unwrap().unwrap();
    assert!(retained.same_owner(&original));
    drop(original); // Both canonical custody and the reissued external owner exist.
    let cssom_before = retained.snapshot().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(element);
    dom.apply(batch, &cancel).unwrap();
    assert!(matches!(
        dom.style_cssom_root(shadow),
        Err(DomError::StaleNode)
    ));
    assert!(historical.node(shadow).unwrap().as_shadow_root().is_some());
    assert_eq!(
        retained.snapshot().unwrap().revision(),
        cssom_before.revision()
    );
    for other in &owners {
        assert!(
            dom.style_cssom_root(other.root())
                .unwrap()
                .unwrap()
                .same_owner(other)
        );
    }
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document.node(), crate::Namespace::html(), "div");
    let replacement = batch.attach_shadow(
        &host,
        crate::DomShadowRootInit::new(crate::HtmlShadowRootMode::Open),
    );
    let commit = dom.apply(batch, &cancel).unwrap();
    let replacement = commit.created(&replacement).unwrap().unwrap();
    assert_eq!(replacement.slot, shadow.slot);
    assert_ne!(replacement.generation, shadow.generation);
    assert!(dom.style_cssom_root(replacement).unwrap().is_none());
    drop(dom);
    // Historical snapshots grant no writer; this actual retained capability
    // still owns the original resource after canonical and owner destruction.
    let edit = retained
        .batch(CssomEditGuard::from_snapshot(&retained.snapshot().unwrap()))
        .unwrap();
    retained.commit(&mut Some(edit)).unwrap();
    assert_eq!(
        retained.snapshot().unwrap().revision(),
        cssom_before.revision()
    );
}

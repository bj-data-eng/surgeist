//! Negative protocol cases over the actual Session-owned child creation.
//! No request, identity, policy phase or positive policy value is fabricated.
use super::*;
use surgeist_dom::{
    DomFullscreenInherited, DomHostedPolicyPublication, DomResource, DomSourceUrlObservation,
    DomSourceUrlOrigin, DomSourceUrlRequest,
};
use surgeist_html::DocumentString;
use surgeist_html_semantics::{HtmlBaseAllowed, HtmlBaseUrlRecord, HtmlBaseUrlScheme};

struct Pending {
    session: DomSession,
    hosted: HostedRuntime,
    child: DomHostedChildRequest,
    iframe: surgeist_dom::NodeId,
    revision: HtmlBaseFactsRevision,
}

impl Pending {
    fn new(raw: &str) -> Self {
        assert!(matches!(raw, "" | "about:blank"));
        let cancel = AtomicBool::new(false);
        let mut session = fixture();
        let mut hosted = session.hosted_execution();
        let top = {
            let mut dom = session.dom();
            let host = hosted.create_host(&mut dom, &cancel).unwrap();
            hosted.create_top_level(&mut dom, host, &cancel).unwrap()
        };
        let snapshot = session.snapshot();
        let html = snapshot.node(top.document().node()).unwrap().children()[0];
        let body = snapshot.node(html).unwrap().children()[1];
        let iframe = {
            let mut batch = DomBatch::new();
            let closed = batch.create_html_document(top.document(), None);
            let iframe = batch.create_element_ns(&closed, Namespace::html(), "iframe");
            batch.set_attribute(&iframe, "src", raw);
            batch.set_attribute(&iframe, "allow", "fullscreen 'src'");
            session
                .dom()
                .apply(batch, &cancel)
                .unwrap()
                .created(&iframe)
                .unwrap()
                .unwrap()
        };
        let progress = session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(iframe, body, None),
                source_limits(20_000_000),
                &mut DomSourceServices {
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel,
            )
            .unwrap();
        let DomSourceDriveProgress::Pending {
            request: DomSourceRequest::HostedChild(child),
            reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
        } = progress
        else {
            panic!("actual present src must reach the policy URL phase");
        };
        Self {
            session,
            hosted,
            child,
            iframe,
            revision: HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 21, 22, 23),
        }
    }

    fn issue(&mut self, stage: DomSourceUrlStage) -> DomSourceUrlRequest {
        self.session
            .dom()
            .resume_source_edit(
                self.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap()
            .source_url_request(&self.child, stage, &self.revision, &AtomicBool::new(false))
            .unwrap()
    }

    fn accept_url(
        &mut self,
        issued: &DomSourceUrlRequest,
        value: &mut Option<DomSourceUrlObservation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.session
            .dom()
            .resume_source_edit(
                self.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap()
            .accept_source_url_observation(&self.child, issued, value, cancel)
    }

    fn accepted_revision(&mut self) -> Option<HtmlBaseFactsRevision> {
        let mut dom = self.session.dom();
        let lease = dom
            .resume_source_edit(
                self.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        lease
            .source_url_observation(&self.child, DomSourceUrlStage::IframeDeclaredOrigin)
            .unwrap()
            .map(|value| value.request().revision().clone())
    }

    fn assert_pending_policy(&mut self) {
        let mut dom = self.session.dom();
        let lease = dom
            .resume_source_edit(
                self.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(matches!(
            lease.hosted_child_step(&self.child).unwrap(),
            DomHostedChildStep::PermissionsPolicy(_)
        ));
        assert_eq!(
            lease
                .snapshot()
                .node(self.iframe)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }

    fn sealed_policy(&mut self) -> Option<DomHostedPolicyPublication> {
        let issued = self.issue(DomSourceUrlStage::IframeDeclaredOrigin);
        let mut response = Some(failure(&issued));
        self.accept_url(&issued, &mut response, &AtomicBool::new(false))
            .unwrap();
        assert!(response.is_none());
        assert_eq!(self.accepted_revision(), Some(self.revision.clone()));
        let cancel = AtomicBool::new(false);
        {
            let mut dom = self.session.dom();
            let mut lease = dom
                .resume_source_edit(self.child.operation(), source_limits(20_000_000), &cancel)
                .unwrap();
            let _arm = CancelPolicyAccept::arm();
            assert!(matches!(
                self.hosted.service_source_child(
                    &mut lease,
                    &self.child,
                    Some(&self.revision),
                    &cancel
                ),
                Err(DomHostedRuntimeError::Dom(DomHostedError::Dom(
                    DomError::Cancelled
                )))
            ));
            assert!(cancel.load(Ordering::Acquire));
        }
        self.assert_pending_policy();
        let owner = self.hosted.state().unwrap();
        let mut state = owner.borrow_mut();
        state
            .children
            .iter_mut()
            .find(|row| row.operation == self.child.operation())
            .unwrap()
            .policy
            .take()
    }

    fn accept_policy(
        &mut self,
        publication: &mut Option<DomHostedPolicyPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedError> {
        let owner = self.hosted.state().unwrap();
        let mut state = owner.borrow_mut();
        self.session
            .dom()
            .resume_source_edit(
                self.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap()
            .accept_hosted_child_policy(&mut state.source, &self.child, publication, cancel)
    }

    fn finish(&mut self, expected: DomFullscreenInherited) {
        let mut observe = |_: &DomSourceUrlRequest, _: &AtomicBool| {
            panic!("a still-qualified accepted reply must not be requested again")
        };
        let progress = self
            .session
            .source_execution()
            .resume(
                &mut self.session.dom(),
                self.child.operation(),
                source_limits(20_000_000),
                &mut DomSourceServices {
                    style_policy: &mut |_| None,
                    url: Some(DomSourceUrlService {
                        revision: &self.revision,
                        observe: &mut observe,
                    }),
                },
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
        let snapshot = self.session.snapshot();
        let nav = snapshot
            .node(self.iframe)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable()
            .unwrap()
            .unwrap();
        let document = snapshot
            .hosted_navigable(nav)
            .unwrap()
            .active_document()
            .unwrap();
        assert_eq!(
            snapshot
                .document(document)
                .unwrap()
                .metadata()
                .initial_permissions_policy()
                .unwrap()
                .inherited_fullscreen(),
            expected
        );
        assert_eq!(snapshot.incomplete_source_operation(), None);
    }
}

fn failure(issued: &DomSourceUrlRequest) -> DomSourceUrlObservation {
    assert!(issued.raw().is_empty());
    assert_eq!(issued.base_url().serialization(), &"about:blank".into());
    issued
        .observe(
            HtmlBaseParseObservation::new(
                issued.raw().clone(),
                issued.base_url().clone(),
                issued.encoding().clone(),
                issued.revision().clone(),
                HtmlBaseParseOutcome::Failure,
            )
            .unwrap(),
            None,
        )
        .unwrap()
}

fn parsed_blank(
    issued: &DomSourceUrlRequest,
    origin: Option<DomSourceUrlOrigin>,
) -> Result<DomSourceUrlObservation, DomError> {
    assert_eq!(issued.raw(), &"about:blank".into());
    let record = HtmlBaseUrlRecord::observed(
        issued.revision().clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    issued.observe(
        HtmlBaseParseObservation::new(
            issued.raw().clone(),
            issued.base_url().clone(),
            issued.encoding().clone(),
            issued.revision().clone(),
            HtmlBaseParseOutcome::Parsed {
                record,
                base_allowed: HtmlBaseAllowed::Unavailable,
            },
        )
        .unwrap(),
        origin,
    )
}

fn address<T>(candidate: &Option<T>) -> *const T {
    std::ptr::from_ref(candidate.as_ref().expect("original unconsumed candidate"))
}

#[test]
fn foreign_policy_request_issuer_and_publication_preserve_both_real_candidates() {
    let mut a = Pending::new("");
    let mut b = Pending::new("");
    let mut publication_a = a.sealed_policy();
    let mut publication_b = b.sealed_policy();
    let address_a = address(&publication_a);
    let address_b = address(&publication_b);
    let cancel = AtomicBool::new(false);
    {
        let owner_a = a.hosted.state().unwrap();
        let mut state_a = owner_a.borrow_mut();
        let mut dom_b = b.session.dom();
        let lease_b = dom_b
            .resume_source_edit(b.child.operation(), source_limits(20_000_000), &cancel)
            .unwrap();
        let DomHostedChildStep::PermissionsPolicy(request_b) =
            lease_b.hosted_child_step(&b.child).unwrap()
        else {
            panic!("actual B policy phase");
        };
        // The value is deliberately irrelevant at this invalid issuer boundary;
        // positive publications above came from Runtime's real evaluator.
        assert!(matches!(
            state_a.source.seal_permissions_policy(
                request_b,
                DomFullscreenInherited::Enabled,
                &cancel
            ),
            Err(DomHostedError::ForeignSource)
        ));
        let mut dom_a = a.session.dom();
        let mut lease_a = dom_a
            .resume_source_edit(a.child.operation(), source_limits(20_000_000), &cancel)
            .unwrap();
        assert_eq!(
            lease_a.accept_hosted_child_policy(
                &mut state_a.source,
                &b.child,
                &mut publication_a,
                &cancel
            ),
            Err(DomHostedError::WrongPhase)
        );
        assert_eq!(
            lease_a.accept_hosted_child_policy(
                &mut state_a.source,
                &a.child,
                &mut publication_b,
                &cancel
            ),
            Err(DomHostedError::WrongPhase)
        );
        let owner_b = b.hosted.state().unwrap();
        let mut state_b = owner_b.borrow_mut();
        assert_eq!(
            lease_a.accept_hosted_child_policy(
                &mut state_b.source,
                &a.child,
                &mut publication_a,
                &cancel
            ),
            Err(DomHostedError::ForeignSource)
        );
    }
    assert_eq!(address(&publication_a), address_a);
    assert_eq!(address(&publication_b), address_b);
    a.assert_pending_policy();
    b.assert_pending_policy();
    a.accept_policy(&mut publication_a, &cancel).unwrap();
    b.accept_policy(&mut publication_b, &cancel).unwrap();
    assert!(publication_a.is_none());
    assert!(publication_b.is_none());
    a.finish(DomFullscreenInherited::Enabled);
    b.finish(DomFullscreenInherited::Enabled);
}

#[test]
fn cancelled_policy_accept_keeps_candidate_then_success_is_single_use() {
    let mut pending = Pending::new("");
    let mut publication = pending.sealed_policy();
    let original = address(&publication);
    assert_eq!(
        pending.accept_policy(&mut publication, &AtomicBool::new(true)),
        Err(DomHostedError::Dom(DomError::Cancelled))
    );
    assert_eq!(address(&publication), original);
    pending.assert_pending_policy();
    pending
        .accept_policy(&mut publication, &AtomicBool::new(false))
        .unwrap();
    assert!(publication.is_none());
    assert_eq!(
        pending.accept_policy(&mut publication, &AtomicBool::new(false)),
        Err(DomHostedError::WrongPhase)
    );
    assert!(publication.is_none());
    pending.finish(DomFullscreenInherited::Enabled);
}

#[test]
fn real_issuer_revision_change_rejects_policy_candidate_until_actual_reseal() {
    let mut pending = Pending::new("");
    let mut publication = pending.sealed_policy();
    let original = address(&publication);
    let old_revision = pending
        .hosted
        .state()
        .unwrap()
        .borrow()
        .source
        .revision()
        .unwrap();
    pending
        .hosted
        .create_host(&mut pending.session.dom(), &AtomicBool::new(false))
        .unwrap();
    let revision = pending
        .hosted
        .state()
        .unwrap()
        .borrow()
        .source
        .revision()
        .unwrap();
    assert_ne!(old_revision, revision);
    let bytes = pending
        .hosted
        .state()
        .unwrap()
        .borrow()
        .source
        .retained_storage_bytes();
    assert_eq!(
        pending.accept_policy(&mut publication, &AtomicBool::new(false)),
        Err(DomHostedError::StaleBasis)
    );
    assert_eq!(address(&publication), original);
    assert_eq!(
        pending
            .hosted
            .state()
            .unwrap()
            .borrow()
            .source
            .revision()
            .unwrap(),
        revision
    );
    assert_eq!(
        pending
            .hosted
            .state()
            .unwrap()
            .borrow()
            .source
            .retained_storage_bytes(),
        bytes
    );
    pending.assert_pending_policy();
    // The Runtime row no longer owns the deliberately retained old candidate;
    // the original driver evaluates and seals a fresh qualified publication.
    pending.finish(DomFullscreenInherited::Enabled);
    assert_eq!(address(&publication), original);
}

#[test]
fn real_url_stage_outcome_origin_pairings_reject_before_policy_and_then_complete() {
    let mut empty = Pending::new("");
    let issued = empty.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    let actual_parent = issued.target_origin().unwrap().clone();
    for origin in [
        DomSourceUrlOrigin::Opaque(actual_parent),
        DomSourceUrlOrigin::UnequalToTarget,
    ] {
        let observation = HtmlBaseParseObservation::new(
            issued.raw().clone(),
            issued.base_url().clone(),
            issued.encoding().clone(),
            issued.revision().clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap();
        assert!(matches!(
            issued.observe(observation, Some(origin)),
            Err(DomError::SourceUrlInputsMismatch)
        ));
    }
    assert_eq!(empty.accepted_revision(), None);
    empty.assert_pending_policy();
    let mut value = Some(failure(&issued));
    empty
        .accept_url(&issued, &mut value, &AtomicBool::new(false))
        .unwrap();
    assert!(value.is_none());
    empty.finish(DomFullscreenInherited::Enabled);

    let mut parsed = Pending::new("about:blank");
    let pp = parsed.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    assert!(matches!(
        parsed_blank(&pp, None),
        Err(DomError::SourceUrlInputsMismatch)
    ));
    assert_eq!(parsed.accepted_revision(), None);
    parsed.assert_pending_policy();
    // URLfde3 origin Otherwise returns a NEW opaque identity; its inequality
    // to the preexisting constructor target is not inferred from serialization.
    let mut value = Some(parsed_blank(&pp, Some(DomSourceUrlOrigin::UnequalToTarget)).unwrap());
    parsed
        .accept_url(&pp, &mut value, &AtomicBool::new(false))
        .unwrap();
    assert!(value.is_none());
    {
        let mut dom = parsed.session.dom();
        let mut lease = dom
            .resume_source_edit(
                parsed.child.operation(),
                source_limits(20_000_000),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert!(matches!(
            parsed.hosted.service_source_child(
                &mut lease,
                &parsed.child,
                Some(&parsed.revision),
                &AtomicBool::new(false)
            ),
            Ok(SourceProgress::Url(DomSourceUrlStage::FrameIframeSrc))
        ));
    }
    let src = parsed.issue(DomSourceUrlStage::FrameIframeSrc);
    assert_eq!(src.target_origin(), None);
    assert!(matches!(
        parsed_blank(&src, Some(DomSourceUrlOrigin::UnequalToTarget)),
        Err(DomError::SourceUrlInputsMismatch)
    ));
    let mut value = Some(parsed_blank(&src, None).unwrap());
    parsed
        .accept_url(&src, &mut value, &AtomicBool::new(false))
        .unwrap();
    assert!(value.is_none());
    parsed.finish(DomFullscreenInherited::Disabled);
}

#[test]
fn foreign_superseded_and_duplicate_url_responses_do_not_replace_original_slot() {
    let mut a = Pending::new("");
    let mut b = Pending::new("");
    let issued_a = a.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    let issued_b = b.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    let mut a_value = Some(failure(&issued_a));
    let mut b_value = Some(failure(&issued_b));
    let a_address = address(&a_value);
    let b_address = address(&b_value);
    assert_eq!(
        a.accept_url(&issued_a, &mut b_value, &AtomicBool::new(false)),
        Err(DomError::SourceUrlInputsMismatch)
    );
    assert_eq!(
        a.accept_url(&issued_b, &mut a_value, &AtomicBool::new(false)),
        Err(DomError::SourceUrlInputsMismatch)
    );
    assert_eq!(address(&a_value), a_address);
    assert_eq!(address(&b_value), b_address);
    assert_eq!(a.accepted_revision(), None);
    assert_eq!(b.accepted_revision(), None);
    a.revision = HtmlBaseFactsRevision::new(a.revision.producer().clone(), 22, 22, 23);
    let current = a.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    assert!(current.occurrence() > issued_a.occurrence());
    assert_eq!(
        a.accept_url(&current, &mut a_value, &AtomicBool::new(false)),
        Err(DomError::SourceUrlInputsMismatch)
    );
    assert_eq!(address(&a_value), a_address);
    assert_eq!(a.accepted_revision(), None);
    let mut valid = Some(failure(&current));
    a.accept_url(&current, &mut valid, &AtomicBool::new(false))
        .unwrap();
    assert!(valid.is_none());
    let mut duplicate = Some(failure(&current));
    let duplicate_address = address(&duplicate);
    assert_eq!(
        a.accept_url(&current, &mut duplicate, &AtomicBool::new(false)),
        Err(DomError::SourceUrlInputsMismatch)
    );
    assert_eq!(address(&duplicate), duplicate_address);
    assert_eq!(a.accepted_revision(), Some(a.revision.clone()));
    b.accept_url(&issued_b, &mut b_value, &AtomicBool::new(false))
        .unwrap();
    assert!(b_value.is_none());
    a.finish(DomFullscreenInherited::Enabled);
    b.finish(DomFullscreenInherited::Enabled);
}

#[test]
fn url_cancel_work_and_bytes_refusals_keep_phase_candidate_and_retry_custody() {
    let mut pending = Pending::new("");
    let issued = pending.issue(DomSourceUrlStage::IframeDeclaredOrigin);
    let mut value = Some(failure(&issued));
    let original = address(&value);
    assert_eq!(
        pending.accept_url(&issued, &mut value, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    assert_eq!(address(&value), original);
    assert_eq!(pending.accepted_revision(), None);
    pending.assert_pending_policy();
    let cancel = AtomicBool::new(false);
    // Measure only the real unchanged lease-entry walk for quota SETUP. The
    // oracle is actual accept-call refusal and preserved custody, not a claim
    // that this measured number independently establishes a tariff boundary.
    let before = pending
        .session
        .dom()
        .pending_source_edit()
        .unwrap()
        .spent_work();
    let entry_end = {
        let mut dom = pending.session.dom();
        dom.resume_source_edit(
            pending.child.operation(),
            source_limits(20_000_000),
            &cancel,
        )
        .unwrap()
        .status()
        .spent_work()
    };
    let entry_work = entry_end.checked_sub(before).unwrap();
    assert!(entry_work > 0);
    let limit = entry_end.checked_add(entry_work).unwrap();
    {
        let mut dom = pending.session.dom();
        let mut lease = dom
            .resume_source_edit(pending.child.operation(), source_limits(limit), &cancel)
            .unwrap();
        assert_eq!(lease.status().spent_work(), limit);
        assert_eq!(
            lease.accept_source_url_observation(&pending.child, &issued, &mut value, &cancel),
            Err(DomError::SourceWorkLimit { limit })
        );
    }
    assert_eq!(address(&value), original);
    assert_eq!(pending.accepted_revision(), None);
    pending.assert_pending_policy();
    // Raw is still the exact empty input, with a real spare-capacity backing.
    // Its backing alone exceeds this finite byte envelope; no private header
    // estimate or graph/layout mirror is used as the expected refusal oracle.
    let raw = DocumentString::from_utf16(Vec::with_capacity(4_000_000));
    assert!(raw.is_empty());
    let observation = HtmlBaseParseObservation::new(
        raw,
        issued.base_url().clone(),
        issued.encoding().clone(),
        issued.revision().clone(),
        HtmlBaseParseOutcome::Failure,
    )
    .unwrap();
    let mut large = Some(issued.observe(observation, None).unwrap());
    let large_address = address(&large);
    {
        let mut dom = pending.session.dom();
        let mut lease = dom
            .resume_source_edit(
                pending.child.operation(),
                DomSourceLimits::new(4_000_000, 16_000_000, 20_000_000, 256, 4096, 4096, 1),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            lease.accept_source_url_observation(&pending.child, &issued, &mut large, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: 4_000_000
            })
        );
        assert!(
            lease
                .source_url_observation(&pending.child, DomSourceUrlStage::IframeDeclaredOrigin)
                .unwrap()
                .is_none()
        );
        // The large rejected candidate is still caller-owned. A valid small
        // candidate must fit the SAME envelope, proving no failed Source debit
        // survives the actual candidate formation's unwind.
        lease
            .accept_source_url_observation(&pending.child, &issued, &mut value, &cancel)
            .unwrap();
        assert!(value.is_none());
    }
    assert_eq!(address(&large), large_address);
    assert_eq!(pending.accepted_revision(), Some(pending.revision.clone()));
    pending.finish(DomFullscreenInherited::Enabled);
}

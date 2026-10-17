//! Independent literal outcomes from PP29398046 §§9.4–9.7 and HTML877d
//! child creation. The fixtures supply raw attributes and actual session
//! identities; Runtime computes the policy and canonical DOM retains it.
//! New child fronts use functional evidence, not an artificial API RED.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
#![forbid(unsafe_code)]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_html_semantics::{
    HtmlBaseAllowed, HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseParseObservation,
    HtmlBaseParseOutcome, HtmlBaseProducerId, HtmlBaseUrlRecord, HtmlBaseUrlScheme,
};
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}

fn session() -> DomSession {
    DomSession::new(
        Dom::new(DomLimits::new(512, 1_000_000, 1024, 1024, 1024, 64, 4_000_000).unwrap()).unwrap(),
        DomSessionLimits {
            source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 64,
                reports: 64,
                storage_bytes: 32_000_000,
                work: 4_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            dom_manipulation: DomManipulationRuntimeLimits {
                tasks: 64,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(64, 64, 64, 64, 64, 64, 64),
                    32_000_000,
                    32_000_000,
                    4_000_000,
                ),
                tasks: 64,
                storage_bytes: 32_000_000,
                scratch_bytes: 32_000_000,
                work: 4_000_000,
            },
        },
    )
    .unwrap()
}

fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(32_000_000, 1_000_000, 20_000_000, 256, 4096, 4096, 1)
}

fn top_level(session: &mut DomSession) -> DomHostedTopLevel {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
}

fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let html = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(html).unwrap().children()[1]
}

fn detached_container(
    session: &mut DomSession,
    associated: DocumentId,
    local: &str,
    attributes: &[(&str, &str)],
) -> NodeId {
    let mut dom = session.dom();
    let mut setup = DomBatch::new();
    let closed = setup.create_html_document(associated, None);
    let container = setup.create_element_ns(&closed, Namespace::html(), local);
    for &(name, value) in attributes {
        setup.set_attribute(&container, name, value);
    }
    dom.apply(setup, &cancel())
        .unwrap()
        .created(&container)
        .unwrap()
        .unwrap()
}

fn insert_without_url(
    session: &mut DomSession,
    container: NodeId,
    parent: NodeId,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    source.apply(
        &mut session.dom(),
        DomSourceEdit::insert_before(container, parent, None),
        source_limits(),
        &mut DomSourceServices {
            style_policy: &mut policy,
            url: None,
        },
        &cancel(),
    )
}

fn completed_insert(session: &mut DomSession, container: NodeId, parent: NodeId) {
    let result = insert_without_url(session, container, parent).unwrap();
    assert!(
        matches!(result, DomSourceDriveProgress::Complete(_)),
        "actual Source tail: {result:?}"
    );
    assert_eq!(session.snapshot().incomplete_source_operation(), None);
    assert_eq!(session.snapshot().incomplete_hosted_operation(), None);
}

fn child_document(snapshot: &DomSnapshot, container: NodeId) -> DocumentId {
    let child = snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .expect("actual child published before readback");
    // Structural snapshots remain readable during an incomplete Source tail.
    // Locate the Document by its actual captured association, without asking
    // the complete-history/fully-active view to certify unfinished navigation.
    let mut associated = snapshot.nodes().filter_map(|id| {
        let document = snapshot.node(id).unwrap().as_document()?;
        (document.navigable() == Some(child)).then_some(document.id())
    });
    let document = associated.next().expect("actual associated child Document");
    assert_eq!(
        associated.next(),
        None,
        "unique initial Document association"
    );
    document
}

fn fullscreen(snapshot: &DomSnapshot, document: DocumentId) -> DomFullscreenInherited {
    let policy = snapshot
        .document(document)
        .unwrap()
        .metadata()
        .initial_permissions_policy()
        .expect("actual constructor-computed policy");
    assert_eq!(
        policy.profile(),
        DomInitialPermissionsPolicyProfile::FullscreenSelfOpaqueInitial
    );
    policy.inherited_fullscreen()
}

#[test]
fn no_opener_top_and_same_opaque_default_iframe_have_actual_enabled_policy() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    assert_eq!(
        fullscreen(&before, top.document()),
        DomFullscreenInherited::Enabled
    );
    let container = detached_container(&mut session, top.document(), "iframe", &[]);
    let detached = session.snapshot();
    completed_insert(&mut session, container, body(&before, top.document()));
    let current = session.snapshot();
    let child = child_document(&current, container);
    assert_ne!(child, top.document());
    assert_eq!(
        current.document(child).unwrap().metadata().origin(),
        before.document(top.document()).unwrap().metadata().origin()
    );
    assert_eq!(fullscreen(&current, child), DomFullscreenInherited::Enabled);
    assert_eq!(
        fullscreen(&before, top.document()),
        DomFullscreenInherited::Enabled
    );
    assert_eq!(
        detached
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

#[test]
fn raw_directives_follow_literal_last_entry_ascii_and_allowfullscreen_outcomes() {
    use DomFullscreenInherited::{Disabled, Enabled};
    // Expected values are literal PP source outcomes. Feature recognition is
    // exact; only quoted self/src keywords use ASCII-insensitive comparison.
    for (allow, allow_fullscreen, expected) in [
        (None, false, Enabled),
        (Some(""), false, Enabled),
        (Some(" ; \t;"), false, Enabled),
        (Some("geolocation *"), false, Enabled),
        (Some("FULLSCREEN 'none'"), false, Enabled),
        (Some("fullscreen"), false, Enabled),
        (Some("fullscreen 'src'"), false, Enabled),
        (Some("fullscreen 'SeLf'"), false, Enabled),
        (Some("\tfullscreen\n'SeLf'\r"), false, Enabled),
        (Some("fullscreen 'none' 'self'"), false, Enabled),
        (Some("fullscreen 'none' *"), false, Enabled),
        (
            Some("fullscreen https://example.test/ * 'self'"),
            false,
            Enabled,
        ),
        (Some("fullscreen *; fullscreen 'none'"), false, Disabled),
        (Some("fullscreen 'none'; fullscreen *"), false, Enabled),
        (
            Some("fullscreen 'self'; fullscreen 'none'"),
            false,
            Disabled,
        ),
        (Some("fullscreen 'none'"), true, Disabled),
        (Some("geolocation *"), true, Enabled),
        (None, true, Enabled),
        (Some("fullscreen https://example.test/"), false, Disabled),
        (Some("fullscreen 'ſelf'"), false, Disabled),
        (Some("fullscreen\u{a0}'none'"), false, Enabled),
    ] {
        let mut session = session();
        let top = top_level(&mut session);
        let before = session.snapshot();
        let mut attributes = Vec::new();
        if let Some(raw) = allow {
            attributes.push(("allow", raw));
        }
        if allow_fullscreen {
            attributes.push(("allowfullscreen", ""));
        }
        let container = detached_container(&mut session, top.document(), "iframe", &attributes);
        completed_insert(&mut session, container, body(&before, top.document()));
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_eq!(
            fullscreen(&current, child),
            expected,
            "allow={allow:?}, allowfullscreen={allow_fullscreen}"
        );
        assert_eq!(
            current.node(container).unwrap().owner_document(),
            Some(top.document())
        );
        assert_eq!(
            current
                .node(container)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"allow".into()),
            allow.map(DocumentString::from).as_ref()
        );
        assert_eq!(fullscreen(&before, top.document()), Enabled);
    }
}

#[test]
fn sandbox_origin_is_fresh_and_wildcard_does_not_merge_declared_with_child_origin() {
    use DomFullscreenInherited::{Disabled, Enabled};
    for (sandbox, allow, allow_fullscreen, same_origin, expected) in [
        (None, None, false, true, Enabled),
        (Some(""), None, false, false, Disabled),
        (Some("allow-scripts"), None, false, false, Disabled),
        (Some("allow-same-origin"), None, false, true, Enabled),
        (
            Some("ALLOW-SAME-ORIGIN"),
            Some("fullscreen 'src'"),
            false,
            true,
            Enabled,
        ),
        (Some(""), Some("fullscreen 'self'"), false, false, Disabled),
        (Some(""), Some("fullscreen 'src'"), false, false, Disabled),
        (Some(""), Some("fullscreen"), false, false, Disabled),
        (Some(""), Some("fullscreen *"), false, false, Enabled),
        (Some(""), None, true, false, Enabled),
        (Some(""), Some("fullscreen 'none'"), true, false, Disabled),
        (Some(""), Some("geolocation *"), false, false, Disabled),
        (Some(""), Some("FULLSCREEN *"), false, false, Disabled),
    ] {
        let mut session = session();
        let top = top_level(&mut session);
        let before = session.snapshot();
        let mut attributes = Vec::new();
        if let Some(raw) = sandbox {
            attributes.push(("sandbox", raw));
        }
        if let Some(raw) = allow {
            attributes.push(("allow", raw));
        }
        if allow_fullscreen {
            attributes.push(("allowfullscreen", ""));
        }
        let container = detached_container(&mut session, top.document(), "iframe", &attributes);
        completed_insert(&mut session, container, body(&before, top.document()));
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_eq!(
            current.document(child).unwrap().metadata().origin()
                == before.document(top.document()).unwrap().metadata().origin(),
            same_origin
        );
        assert_eq!(
            fullscreen(&current, child),
            expected,
            "sandbox={sandbox:?}, allow={allow:?}, allowfullscreen={allow_fullscreen}"
        );
    }
}

#[test]
fn actual_frame_ignores_iframe_allow_sandbox_and_srcdoc_attributes() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let container = detached_container(
        &mut session,
        top.document(),
        "frame",
        &[
            ("sandbox", ""),
            ("allow", "fullscreen 'none'"),
            ("allowfullscreen", ""),
            ("srcdoc", "ignored on Frame"),
        ],
    );
    completed_insert(&mut session, container, body(&before, top.document()));
    let current = session.snapshot();
    let child = child_document(&current, container);
    assert_eq!(
        current.document(child).unwrap().metadata().origin(),
        before.document(top.document()).unwrap().metadata().origin()
    );
    assert_eq!(fullscreen(&current, child), DomFullscreenInherited::Enabled);
    assert_eq!(
        current
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"sandbox".into()),
        Some(&"".into())
    );
}

#[test]
fn disabled_parent_blocks_wildcards_before_the_separate_src_processing_request() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent_container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("allow", "fullscreen 'none'")],
    );
    let parent_node = body(&session.snapshot(), top.document());
    completed_insert(&mut session, parent_container, parent_node);
    let before = session.snapshot();
    let parent_document = child_document(&before, parent_container);
    assert_eq!(
        fullscreen(&before, parent_document),
        DomFullscreenInherited::Disabled
    );
    let container = detached_container(
        &mut session,
        parent_document,
        "iframe",
        &[
            ("allow", "fullscreen *"),
            ("allowfullscreen", ""),
            ("src", "unobserved-target"),
        ],
    );
    let result =
        insert_without_url(&mut session, container, body(&before, parent_document)).unwrap();
    assert!(
        matches!(
            result,
            DomSourceDriveProgress::Pending {
                reason: DomSourceUnavailable::Url(DomSourceUrlStage::FrameIframeSrc),
                ..
            }
        ),
        "PP's declared-origin URL branch must be skipped, but the later src processing remains reached: {result:?}"
    );
    let current = session.snapshot();
    let child = child_document(&current, container);
    assert_eq!(
        fullscreen(&current, child),
        DomFullscreenInherited::Disabled
    );
    assert!(current.incomplete_source_operation().is_some());
    assert_eq!(
        fullscreen(&before, parent_document),
        DomFullscreenInherited::Disabled
    );
}

#[test]
fn present_srcdoc_uses_parent_origin_before_truthful_unsupported_navigation_tail() {
    for srcdoc in ["", "<p>content"] {
        let mut session = session();
        let top = top_level(&mut session);
        let before = session.snapshot();
        let container = detached_container(
            &mut session,
            top.document(),
            "iframe",
            &[
                ("allow", "fullscreen 'src'"),
                ("srcdoc", srcdoc),
                ("src", "http://["),
            ],
        );
        let result = insert_without_url(&mut session, container, body(&before, top.document()));
        assert!(
            matches!(
                result,
                Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                    DomHostedError::UnsupportedProfile
                )))
            ),
            "srcdoc navigation remains outside the admitted initial tail: {result:?}"
        );
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_eq!(fullscreen(&current, child), DomFullscreenInherited::Enabled);
        assert_eq!(
            current.document(child).unwrap().metadata().origin(),
            before.document(top.document()).unwrap().metadata().origin()
        );
        assert!(current.incomplete_source_operation().is_some());
    }
}

#[test]
fn present_empty_src_is_not_missing_policy_evidence_even_with_wildcard() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("allow", "fullscreen *"), ("src", "")],
    );
    let result =
        insert_without_url(&mut session, container, body(&before, top.document())).unwrap();
    assert!(
        matches!(
            result,
            DomSourceDriveProgress::Pending {
                reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
                ..
            }
        ),
        "a later wildcard cannot short-circuit the reached declared-origin parse: {result:?}"
    );
    let current = session.snapshot();
    assert!(current.incomplete_source_operation().is_some());
    assert_eq!(
        current
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        fullscreen(&current, top.document()),
        DomFullscreenInherited::Enabled
    );
}

// A finite external URL producer fixture. Its inputs and outcomes are selected
// before Source issues a request. These cases do not implement a URL parser or
// derive an origin from request.target_origin(). The fresh native top has no
// Base elements, so its actual fallback is its constructor Document URL.
struct FixedUrlInputs {
    document: DocumentId,
    raw: DocumentString,
    base: HtmlBaseUrlRecord,
    encoding: HtmlBaseEncoding,
    revision: HtmlBaseFactsRevision,
    target: DomOpaqueOrigin,
}

impl FixedUrlInputs {
    fn new(session: &DomSession, document: DocumentId, raw: &str) -> Self {
        let snapshot = session.snapshot();
        let document_view = snapshot.document(document).unwrap();
        let facts = document_view.base_document_facts().unwrap();
        assert_eq!(
            facts.document_url().unwrap().serialization(),
            &DocumentString::from("about:blank")
        );
        Self {
            document,
            raw: raw.into(),
            base: facts.document_url().unwrap().clone(),
            encoding: facts.encoding().unwrap().clone(),
            revision: HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 11, 12, 13),
            target: document_view.metadata().origin().clone(),
        }
    }

    fn check(&self, request: &DomSourceUrlRequest) {
        assert_eq!(request.document().document(), self.document);
        assert_eq!(request.raw(), &self.raw);
        assert_eq!(request.base_url(), &self.base);
        assert_eq!(request.encoding(), &self.encoding);
        assert_eq!(request.revision(), &self.revision);
        match request.stage() {
            DomSourceUrlStage::IframeDeclaredOrigin => {
                assert_eq!(request.target_origin(), Some(&self.target));
            }
            DomSourceUrlStage::FrameIframeSrc => assert_eq!(request.target_origin(), None),
        }
    }

    fn observation(&self, outcome: HtmlBaseParseOutcome) -> HtmlBaseParseObservation {
        HtmlBaseParseObservation::new(
            self.raw.clone(),
            self.base.clone(),
            self.encoding.clone(),
            self.revision.clone(),
            outcome,
        )
        .unwrap()
    }
}

fn insert_with_url(
    session: &mut DomSession,
    container: NodeId,
    parent: NodeId,
    revision: &HtmlBaseFactsRevision,
    observe: &mut dyn FnMut(
        &DomSourceUrlRequest,
        &AtomicBool,
    ) -> surgeist_runtime::DomSourceUrlObservationResult,
) -> Result<DomSourceDriveProgress, DomSourceRuntimeError> {
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    source.apply(
        &mut session.dom(),
        DomSourceEdit::insert_before(container, parent, None),
        source_limits(),
        &mut DomSourceServices {
            style_policy: &mut policy,
            url: Some(DomSourceUrlService { revision, observe }),
        },
        &cancel(),
    )
}

#[test]
fn qualified_fixed_parse_failure_falls_back_to_parent_origin_and_completes_blank_tail() {
    for raw in ["", "http://["] {
        let mut session = session();
        let top = top_level(&mut session);
        let before = session.snapshot();
        let inputs = FixedUrlInputs::new(&session, top.document(), raw);
        let container = detached_container(
            &mut session,
            top.document(),
            "iframe",
            &[("src", raw), ("allow", "fullscreen 'src'")],
        );
        // URLfde3 no-scheme state: empty input against an opaque about:blank
        // base fails at EOF; the other fixed input has an unclosed IPv6 host.
        let mut stages = Vec::new();
        let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
            inputs.check(request);
            stages.push(request.stage());
            Ok(Some(
                request
                    .observe(inputs.observation(HtmlBaseParseOutcome::Failure), None)
                    .unwrap(),
            ))
        };
        let progress = insert_with_url(
            &mut session,
            container,
            body(&before, top.document()),
            &inputs.revision,
            &mut observe,
        )
        .unwrap();
        assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
        let expected = if raw.is_empty() {
            vec![DomSourceUrlStage::IframeDeclaredOrigin]
        } else {
            vec![
                DomSourceUrlStage::IframeDeclaredOrigin,
                DomSourceUrlStage::FrameIframeSrc,
            ]
        };
        assert_eq!(stages, expected);
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_eq!(fullscreen(&current, child), DomFullscreenInherited::Enabled);
        assert_eq!(
            current.document(child).unwrap().metadata().origin(),
            before.document(top.document()).unwrap().metadata().origin()
        );
        assert_eq!(current.incomplete_source_operation(), None);
    }
}

#[test]
fn parsed_absolute_about_blank_has_new_opaque_declared_origin_not_parent_origin() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let inputs = FixedUrlInputs::new(&session, top.document(), "about:blank");
    let record = HtmlBaseUrlRecord::observed(
        inputs.revision.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("src", "about:blank"), ("allow", "fullscreen 'src'")],
    );
    let mut stages = Vec::new();
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        inputs.check(request);
        stages.push(request.stage());
        // URLfde3 origin's Otherwise branch returns a NEW opaque origin.
        // The target above preexists this parse, so inequality is proved by
        // freshness; an initial about:blank Document's inherited origin is a
        // different source fact from this URL record's origin.
        let origin = (request.stage() == DomSourceUrlStage::IframeDeclaredOrigin)
            .then_some(DomSourceUrlOrigin::UnequalToTarget);
        Ok(Some(
            request
                .observe(
                    inputs.observation(HtmlBaseParseOutcome::Parsed {
                        record: record.clone(),
                        base_allowed: HtmlBaseAllowed::Unavailable,
                    }),
                    origin,
                )
                .unwrap(),
        ))
    };
    let progress = insert_with_url(
        &mut session,
        container,
        body(&before, top.document()),
        &inputs.revision,
        &mut observe,
    )
    .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    assert_eq!(
        stages,
        [
            DomSourceUrlStage::IframeDeclaredOrigin,
            DomSourceUrlStage::FrameIframeSrc
        ]
    );
    let current = session.snapshot();
    let child = child_document(&current, container);
    assert_eq!(
        fullscreen(&current, child),
        DomFullscreenInherited::Disabled
    );
    assert_eq!(
        current.document(child).unwrap().metadata().origin(),
        before.document(top.document()).unwrap().metadata().origin()
    );
    assert_eq!(current.incomplete_source_operation(), None);
}

#[test]
fn prebound_blob_entry_environment_supplies_shared_or_unequal_actual_opaque_origin() {
    struct FixedBlobEntry {
        record: HtmlBaseUrlRecord,
        environment: DomHostedSettings,
    }
    const BLOB_URL: &str = "blob:null/00000000-0000-4000-8000-000000000001";
    for shared_environment in [true, false] {
        let mut session = session();
        let top = top_level(&mut session);
        let entry_top = if shared_environment {
            top
        } else {
            top_level(&mut session)
        };
        // This finite dependency fixture has ONE blob URL entry whose actual
        // environment is captured before any request. URLfde3's origin branch
        // for a non-null blob URL entry returns that environment's origin.
        // It supplies neither a general blob store nor a URL implementation.
        let environment = session
            .hosted_execution()
            .settings(&session.dom(), entry_top.global(), &cancel())
            .unwrap();
        let before = session.snapshot();
        assert_eq!(
            environment.origin() == before.document(top.document()).unwrap().metadata().origin(),
            shared_environment
        );
        let inputs = FixedUrlInputs::new(&session, top.document(), BLOB_URL);
        let entry = FixedBlobEntry {
            record: HtmlBaseUrlRecord::observed(
                inputs.revision.clone(),
                BLOB_URL.into(),
                HtmlBaseUrlScheme::Other,
                false,
                None,
            )
            .unwrap(),
            environment,
        };
        let container = detached_container(
            &mut session,
            top.document(),
            "iframe",
            &[("src", BLOB_URL), ("allow", "fullscreen 'src'")],
        );
        let mut stages = Vec::new();
        let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
            inputs.check(request);
            stages.push(request.stage());
            let origin = (request.stage() == DomSourceUrlStage::IframeDeclaredOrigin)
                .then(|| DomSourceUrlOrigin::Opaque(entry.environment.origin().clone()));
            Ok(Some(
                request
                    .observe(
                        inputs.observation(HtmlBaseParseOutcome::Parsed {
                            record: entry.record.clone(),
                            base_allowed: HtmlBaseAllowed::Unavailable,
                        }),
                        origin,
                    )
                    .unwrap(),
            ))
        };
        let result = insert_with_url(
            &mut session,
            container,
            body(&before, top.document()),
            &inputs.revision,
            &mut observe,
        );
        assert!(
            matches!(
                result,
                Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                    DomHostedError::UnsupportedProfile
                )))
            ),
            "a non-blank blob navigation is not implemented by this fixture: {result:?}"
        );
        assert_eq!(
            stages,
            [
                DomSourceUrlStage::IframeDeclaredOrigin,
                DomSourceUrlStage::FrameIframeSrc
            ]
        );
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_eq!(
            fullscreen(&current, child),
            if shared_environment {
                DomFullscreenInherited::Enabled
            } else {
                DomFullscreenInherited::Disabled
            }
        );
        assert!(current.incomplete_source_operation().is_some());
        assert_eq!(
            fullscreen(&before, top.document()),
            DomFullscreenInherited::Enabled
        );
    }
}

#[test]
fn unavailable_qualified_producer_preserves_request_until_actual_failure_observation() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let inputs = FixedUrlInputs::new(&session, top.document(), "");
    let container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("src", ""), ("allow", "fullscreen 'src'")],
    );
    let mut issued_operation = None;
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        inputs.check(request);
        assert_eq!(request.stage(), DomSourceUrlStage::IframeDeclaredOrigin);
        issued_operation = Some(request.operation());
        Ok(None)
    };
    let result = insert_with_url(
        &mut session,
        container,
        body(&before, top.document()),
        &inputs.revision,
        &mut observe,
    )
    .unwrap();
    assert!(matches!(
        result,
        DomSourceDriveProgress::Pending {
            reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
            ..
        }
    ));
    let operation = issued_operation.unwrap();
    let pending = session.snapshot();
    assert_eq!(
        pending
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        inputs.check(request);
        assert_eq!(request.operation(), operation);
        Ok(Some(
            request
                .observe(inputs.observation(HtmlBaseParseOutcome::Failure), None)
                .unwrap(),
        ))
    };
    let mut policy = |_: &DomStyleSource| None;
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: Some(DomSourceUrlService {
                    revision: &inputs.revision,
                    observe: &mut observe,
                }),
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    let current = session.snapshot();
    assert_eq!(
        fullscreen(&current, child_document(&current, container)),
        DomFullscreenInherited::Enabled
    );
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(
        pending
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

#[test]
fn stale_external_revision_refuses_without_default_policy_then_same_source_resumes() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let inputs = FixedUrlInputs::new(&session, top.document(), "");
    let old_revision = HtmlBaseFactsRevision::new(inputs.revision.producer().clone(), 11, 11, 13);
    let container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("src", ""), ("allow", "fullscreen 'src'")],
    );
    let mut issued_operation = None;
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        inputs.check(request);
        issued_operation = Some(request.operation());
        let stale = HtmlBaseParseObservation::new(
            inputs.raw.clone(),
            inputs.base.clone(),
            inputs.encoding.clone(),
            old_revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap();
        Ok(Some(request.observe(stale, None).unwrap()))
    };
    let result = insert_with_url(
        &mut session,
        container,
        body(&before, top.document()),
        &inputs.revision,
        &mut observe,
    );
    assert!(matches!(
        result,
        Err(DomSourceRuntimeError::Dom(
            DomError::SourceUrlInputsMismatch
        ))
    ));
    let operation = issued_operation.unwrap();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    let refused = session.snapshot();
    assert_eq!(
        refused
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        inputs.check(request);
        assert_eq!(request.operation(), operation);
        Ok(Some(
            request
                .observe(inputs.observation(HtmlBaseParseOutcome::Failure), None)
                .unwrap(),
        ))
    };
    let mut policy = |_: &DomStyleSource| None;
    let result = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: Some(DomSourceUrlService {
                    revision: &inputs.revision,
                    observe: &mut observe,
                }),
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
    let current = session.snapshot();
    assert_eq!(
        fullscreen(&current, child_document(&current, container)),
        DomFullscreenInherited::Enabled
    );
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(
        refused
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

#[test]
fn parent_active_origin_restriction_precedes_srcdoc_and_is_inherited_by_frame() {
    for local in ["iframe", "frame"] {
        let mut session = session();
        let top = top_level(&mut session);
        let top_snapshot = session.snapshot();
        let parent_container = detached_container(
            &mut session,
            top.document(),
            "iframe",
            &[("sandbox", ""), ("allow", "fullscreen *")],
        );
        completed_insert(
            &mut session,
            parent_container,
            body(&top_snapshot, top.document()),
        );
        let before = session.snapshot();
        let parent = child_document(&before, parent_container);
        assert_eq!(fullscreen(&before, parent), DomFullscreenInherited::Enabled);
        let container = detached_container(
            &mut session,
            parent,
            local,
            &[
                ("sandbox", "allow-same-origin"),
                ("allow", "fullscreen 'src'"),
                ("srcdoc", ""),
            ],
        );
        let result = insert_without_url(&mut session, container, body(&before, parent));
        if local == "iframe" {
            assert!(
                matches!(
                    result,
                    Err(DomSourceRuntimeError::Hosted(DomHostedRuntimeError::Dom(
                        DomHostedError::UnsupportedProfile
                    )))
                ),
                "srcdoc navigation remains unsupported after policy creation: {result:?}"
            );
        } else {
            assert!(
                matches!(result, Ok(DomSourceDriveProgress::Complete(_))),
                "Frame's absent-src initial tail must complete: {result:?}"
            );
        }
        let current = session.snapshot();
        let child = child_document(&current, container);
        assert_ne!(
            current.document(child).unwrap().metadata().origin(),
            before.document(parent).unwrap().metadata().origin()
        );
        assert_eq!(
            fullscreen(&current, child),
            DomFullscreenInherited::Disabled
        );
        assert_eq!(fullscreen(&before, parent), DomFullscreenInherited::Enabled);
    }
}

#[test]
fn completed_child_policy_and_origin_do_not_change_after_allow_and_sandbox_edits() {
    for initially_enabled in [true, false] {
        let mut session = session();
        let top = top_level(&mut session);
        let initial_top = session.snapshot();
        let initial_attributes = if initially_enabled {
            Vec::new()
        } else {
            vec![("sandbox", ""), ("allow", "fullscreen 'self'")]
        };
        let container =
            detached_container(&mut session, top.document(), "iframe", &initial_attributes);
        completed_insert(&mut session, container, body(&initial_top, top.document()));
        let before = session.snapshot();
        let child = child_document(&before, container);
        let expected = if initially_enabled {
            DomFullscreenInherited::Enabled
        } else {
            DomFullscreenInherited::Disabled
        };
        assert_eq!(fullscreen(&before, child), expected);
        let original_origin = before.document(child).unwrap().metadata().origin().clone();
        let original_navigable = before
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable();
        // These are real attribute changes on the already-connected container,
        // each through one Source operation. They neither create a new child
        // Document nor retroactively replace its initial inherited policy.
        let mutations = if initially_enabled {
            [("allow", "fullscreen 'none'"), ("sandbox", "")]
        } else {
            [("allow", "fullscreen *"), ("sandbox", "allow-same-origin")]
        };
        for (name, value) in mutations {
            let mut policy = |_: &DomStyleSource| None;
            let result = session.source_execution().apply(
                &mut session.dom(),
                DomSourceEdit::set_attribute(container, name, value),
                source_limits(),
                &mut DomSourceServices {
                    style_policy: &mut policy,
                    url: None,
                },
                &cancel(),
            );
            assert!(
                matches!(result, Ok(DomSourceDriveProgress::Complete(_))),
                "actual {name} mutation must complete before readback: {result:?}"
            );
            let current = session.snapshot();
            let element = current.node(container).unwrap().as_element().unwrap();
            assert_eq!(element.attribute(&name.into()), Some(&value.into()));
            assert_eq!(element.child_navigable(), original_navigable);
            assert_eq!(child_document(&current, container), child);
            assert_eq!(fullscreen(&current, child), expected);
            assert_eq!(
                current.document(child).unwrap().metadata().origin(),
                &original_origin
            );
            assert_eq!(current.incomplete_source_operation(), None);
        }
        let old_element = before.node(container).unwrap().as_element().unwrap();
        assert_eq!(
            old_element.attribute(&"allow".into()),
            (!initially_enabled)
                .then(|| DocumentString::from("fullscreen 'self'"))
                .as_ref()
        );
        assert_eq!(
            old_element.attribute(&"sandbox".into()),
            (!initially_enabled)
                .then(|| DocumentString::from(""))
                .as_ref()
        );
        assert_eq!(fullscreen(&before, child), expected);
        assert_eq!(
            before.document(child).unwrap().metadata().origin(),
            &original_origin
        );
    }
}

#[test]
fn adopted_container_uses_new_document_url_basis_and_rejects_old_document_observation() {
    let mut session = session();
    let a = top_level(&mut session);
    let b = top_level(&mut session);
    let old_inputs = FixedUrlInputs::new(&session, a.document(), "");
    let current_inputs = FixedUrlInputs::new(&session, b.document(), "");
    assert_ne!(old_inputs.target, current_inputs.target);
    // The two actual native constructors own distinct qualified record and
    // encoding identities, despite both spelling about:blank and UTF-8.
    assert_ne!(old_inputs.base, current_inputs.base);
    assert_ne!(old_inputs.encoding, current_inputs.encoding);
    // The supported native initializer creates this disconnected iframe in a
    // closed Document. Real Source adoption then establishes actual hosted A
    // as its node Document before the A-to-B insertion under test.
    let container = detached_container(
        &mut session,
        a.document(),
        "iframe",
        &[("allow", "fullscreen 'src'"), ("src", "")],
    );
    let mut policy = |_: &DomStyleSource| None;
    let adoption = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::adopt(container, a.document().node()),
        source_limits(),
        &mut DomSourceServices {
            style_policy: &mut policy,
            url: None,
        },
        &cancel(),
    );
    assert!(
        matches!(adoption, Ok(DomSourceDriveProgress::Complete(_))),
        "disconnected adoption into actual A must complete: {adoption:?}"
    );
    let before = session.snapshot();
    assert_eq!(
        before.node(container).unwrap().owner_document(),
        Some(a.document())
    );
    assert_eq!(before.node(container).unwrap().parent(), None);
    assert_eq!(
        before
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    let mut operation = None;
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        current_inputs.check(request);
        assert_eq!(request.stage(), DomSourceUrlStage::IframeDeclaredOrigin);
        assert_eq!(request.embedder(), container);
        assert_ne!(request.target_origin(), Some(&old_inputs.target));
        operation = Some(request.operation());
        // This is the same current external producer configuration, not a
        // stale revision test. Its fixed Failure is relative to actual A's
        // about:blank record/encoding and cannot establish the requested B
        // computation after insertion's real adoption.
        let old_basis = HtmlBaseParseObservation::new(
            old_inputs.raw.clone(),
            old_inputs.base.clone(),
            old_inputs.encoding.clone(),
            current_inputs.revision.clone(),
            HtmlBaseParseOutcome::Failure,
        )
        .unwrap();
        Ok(Some(request.observe(old_basis, None).unwrap()))
    };
    let result = insert_with_url(
        &mut session,
        container,
        body(&before, b.document()),
        &current_inputs.revision,
        &mut observe,
    );
    assert!(
        matches!(
            result,
            Err(DomSourceRuntimeError::Dom(
                DomError::SourceUrlInputsMismatch
            ))
        ),
        "old node-Document facts must not establish policy: {result:?}"
    );
    let operation = operation.unwrap();
    let refused = session.snapshot();
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    assert_eq!(
        refused.node(container).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(
        refused
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        before.node(container).unwrap().owner_document(),
        Some(a.document())
    );
    let mut observe = |request: &DomSourceUrlRequest, _: &AtomicBool| {
        current_inputs.check(request);
        assert_eq!(request.operation(), operation);
        assert_eq!(request.embedder(), container);
        Ok(Some(
            request
                .observe(
                    current_inputs.observation(HtmlBaseParseOutcome::Failure),
                    None,
                )
                .unwrap(),
        ))
    };
    let mut policy = |_: &DomStyleSource| None;
    let progress = session
        .source_execution()
        .resume(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                style_policy: &mut policy,
                url: Some(DomSourceUrlService {
                    revision: &current_inputs.revision,
                    observe: &mut observe,
                }),
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(progress, DomSourceDriveProgress::Complete(_)));
    let current = session.snapshot();
    let child = child_document(&current, container);
    assert_eq!(
        current.node(container).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(fullscreen(&current, child), DomFullscreenInherited::Enabled);
    assert_eq!(
        current.document(child).unwrap().metadata().origin(),
        &current_inputs.target
    );
    assert_ne!(
        current.document(child).unwrap().metadata().origin(),
        &old_inputs.target
    );
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(
        refused
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        before.node(container).unwrap().owner_document(),
        Some(a.document())
    );
}

#[test]
fn absent_raw_sandbox_after_detach_still_reaches_present_empty_declared_origin_url() {
    let mut session = session();
    let top = top_level(&mut session);
    let before = session.snapshot();
    let parent = body(&before, top.document());
    let container = detached_container(
        &mut session,
        top.document(),
        "iframe",
        &[("sandbox", ""), ("allow", "fullscreen *")],
    );
    completed_insert(&mut session, container, parent);
    let first = session.snapshot();
    let first_child = child_document(&first, container);
    assert_ne!(
        first.document(first_child).unwrap().metadata().origin(),
        before.document(top.document()).unwrap().metadata().origin()
    );
    assert_eq!(
        fullscreen(&first, first_child),
        DomFullscreenInherited::Enabled
    );
    let mut policy = |_: &DomStyleSource| None;
    let detached = session.source_execution().apply(
        &mut session.dom(),
        DomSourceEdit::detach(container),
        source_limits(),
        &mut DomSourceServices {
            style_policy: &mut policy,
            url: None,
        },
        &cancel(),
    );
    assert!(
        matches!(detached, Ok(DomSourceDriveProgress::Complete(_))),
        "actual detach must clear the content link before raw edits: {detached:?}"
    );
    let detached_snapshot = session.snapshot();
    assert_eq!(detached_snapshot.node(container).unwrap().parent(), None);
    assert_eq!(
        detached_snapshot
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    // HTML's stored Iframe sandbox flag slot is distinct from this raw
    // attribute. While content navigable is null, removing the attribute does
    // not reset that slot. The slot has no public getter; this test observes
    // the independently required RAW-attribute PP branch, not private bits.
    let mut raw_edits = DomBatch::new();
    raw_edits.remove_attribute(container, "sandbox");
    raw_edits.set_attribute(container, "src", "");
    session.dom().apply(raw_edits, &cancel()).unwrap();
    let edited = session.snapshot();
    let element = edited.node(container).unwrap().as_element().unwrap();
    assert_eq!(element.attribute(&"sandbox".into()), None);
    assert_eq!(element.attribute(&"src".into()), Some(&"".into()));
    assert_eq!(
        element.attribute(&"allow".into()),
        Some(&"fullscreen *".into())
    );
    assert_eq!(element.child_navigable(), Some(None));
    let result = insert_without_url(&mut session, container, parent).unwrap();
    assert!(
        matches!(
            result,
            DomSourceDriveProgress::Pending {
                reason: DomSourceUnavailable::Url(DomSourceUrlStage::IframeDeclaredOrigin),
                ..
            }
        ),
        "raw sandbox is absent, so present src must be parsed before wildcard: {result:?}"
    );
    let pending = session.snapshot();
    assert!(pending.incomplete_source_operation().is_some());
    assert_eq!(
        pending
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(
        first
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .attribute(&"sandbox".into()),
        Some(&"".into())
    );
    assert_eq!(
        fullscreen(&first, first_child),
        DomFullscreenInherited::Enabled
    );
    assert_eq!(edited.node(container).unwrap().parent(), None);
    assert_eq!(
        edited
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

use super::*;
use crate::{DomHostedRuntimeError, DomHostedTopLevel, HostedRuntime};
use surgeist_cssom::{CssomLimits, CssomSheetInit};

fn fresh(label: &str) -> DomConstructedSheetOwnerInit<'_> {
    DomConstructedSheetOwnerInit {
        label,
        limits: CssomLimits::default(),
    }
}

fn fixture() -> (DomSession, HostedRuntime, DomHostedTopLevel) {
    let mut dom = dispatch_dom();
    let base = dom.limits().with_base_processing(8, 10_000).unwrap();
    dom.set_limits(base).unwrap();
    let mut session = DomSession::new(dom, session_limits(limits())).unwrap();
    let mut hosted = session.hosted_execution();
    let host = hosted
        .create_host(&mut session.dom(), &AtomicBool::new(false))
        .unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &AtomicBool::new(false))
        .unwrap();
    (session, hosted, top)
}

#[test]
fn fresh_and_joined_windows_use_one_real_store_and_the_current_native_fallback() {
    let (mut session, hosted, top) = fixture();
    let styles = session.style_execution();
    let root = top.document().node();
    let before = session.snapshot();
    assert!(session.dom().style_cssom_root(root).unwrap().is_none());
    assert!(styles.snapshot(root).unwrap().is_none());
    let original = Arc::new(vec![0xd800]);
    let cancel = AtomicBool::new(false);
    let first = styles
        .construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            CssomSheetInit {
                base_url: Some(original.clone()),
                ..Default::default()
            },
            fresh("fresh"),
            &cancel,
        )
        .unwrap();
    let state = first.owner().snapshot().unwrap();
    assert!(state.context().inputs.iter().any(|input| {
        matches!(input.data, CssomInputData::ParserMode(parser) if parser.mode() == CssParserMode::Quirks)
    }));
    let sheet = state.sheet(first.sheet()).unwrap();
    assert_eq!(sheet.inputs().location.as_deref(), Some("about:blank"));
    assert!(Arc::ptr_eq(
        sheet.inputs().base_url.as_ref().unwrap(),
        &original
    ));
    let constructor = sheet.inputs().constructor_document.as_deref().unwrap();
    assert_eq!(first.document(), top.document());
    assert_eq!(first.revision(), state.revision());
    assert_eq!(first.revision().value(), 1);
    assert!(state.sheets().is_empty());
    let (mut foreign, other_host, other_top) = fixture();
    let foreign_stamp = foreign.snapshot().stamp();
    assert!(matches!(
        styles.construct_document_sheet(
            &mut session.dom(),
            &other_host,
            top,
            Default::default(),
            fresh("foreign"),
            &cancel
        ),
        Err(DomConstructedSheetError::Hosted(_))
    ));
    assert!(matches!(
        styles.construct_document_sheet(
            &mut foreign.dom(),
            &other_host,
            other_top,
            Default::default(),
            fresh("foreign"),
            &cancel
        ),
        Err(DomConstructedSheetError::Style(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert_eq!(foreign.snapshot().stamp(), foreign_stamp);

    let second = styles
        .construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            fresh(""),
            &cancel,
        )
        .unwrap();
    let canonical = session
        .dom()
        .style_cssom_root(root)
        .unwrap()
        .unwrap()
        .snapshot()
        .unwrap();
    assert_ne!(first.sheet(), second.sheet());
    assert_eq!(state.owner(), canonical.owner());
    assert_eq!(canonical.revision(), second.revision());
    assert_eq!(canonical.revision().value(), 2);
    assert_eq!(
        styles.snapshot(root).unwrap().unwrap().revision(),
        canonical.revision()
    );
    assert_eq!(
        canonical
            .sheet(second.sheet())
            .unwrap()
            .inputs()
            .constructor_document
            .as_deref(),
        Some(constructor)
    );
    assert!(canonical.sheets().is_empty());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    let substituted = DomStyleCssomRoot::new(
        root,
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let refused = hosted.with_current_top_level(&mut session.dom(), top, &cancel, |source, dom| {
        let prepared = source.prepare_document_sheet(
            dom.raw_mut(),
            top.global(),
            top.document(),
            &substituted,
            Default::default(),
            &cancel,
        )?;
        drop(prepared);
        Ok::<_, DomConstructedSheetError>(())
    });
    assert!(matches!(
        refused,
        Err(DomConstructedSheetError::Construction(
            surgeist_dom::DomHostedSheetConstructionError::Hosted(
                surgeist_dom::DomHostedError::Dom(DomError::StyleReceiptMismatch)
            )
        ))
    ));
    assert_eq!(
        first.owner().snapshot().unwrap().revision(),
        canonical.revision()
    );
}

#[test]
fn runtime_only_owner_is_joined_and_unavailable_hosted_base_change_preserves_the_first_refusal() {
    let (mut session, hosted, top) = fixture();
    let mut styles = session.style_execution();
    let document = top.document();
    let cancel = AtomicBool::new(false);
    let mut input = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    let old = input.as_ref().unwrap().snapshot().unwrap();
    styles
        .bind_root(&session.dom(), "prebound".into(), &mut input, &cancel)
        .unwrap();
    assert!(
        session
            .dom()
            .style_cssom_root(document.node())
            .unwrap()
            .is_none()
    );
    let snap = session.snapshot();
    let html = snap.node(document.node()).unwrap().children()[0];
    let head = snap.node(html).unwrap().children()[0];
    let mut edit = DomBatch::new();
    let base = edit.create_element_ns(document, Namespace::html(), "base");
    edit.set_attribute(&base, "href", "https://example.invalid/changed/");
    edit.insert_before(&base, head, None);
    let refusal = session.dom().apply(edit, &cancel).unwrap_err();
    assert!(
        matches!(
            refusal,
            DomError::BaseFacts(surgeist_dom::DomBaseFactsError::Missing(
                surgeist_html_semantics::HtmlBaseMissing::DocumentFacts
            ))
        ),
        "{refusal:?}"
    );
    assert_eq!(session.snapshot().stamp(), snap.stamp());
    let sheet = styles
        .construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            fresh(""),
            &cancel,
        )
        .unwrap();
    let after = sheet.owner().snapshot().unwrap();
    assert_eq!(after.owner(), old.owner());
    assert_eq!(after.context(), old.context());
    assert_eq!(
        after
            .sheet(sheet.sheet())
            .unwrap()
            .inputs()
            .location
            .as_deref(),
        Some("about:blank")
    );
    assert_eq!(after.revision().value(), old.revision().value() + 1);
}

#[test]
fn exact_style_root_bytes_and_late_cssom_refusal_leave_no_partial_owner() {
    let label = "funded-owner";
    let (mut session, mut hosted, top) = fixture();
    let mut styles = session.style_execution();
    let cancel = AtomicBool::new(false);
    let state = styles.original().unwrap();
    let before = state
        .borrow()
        .storage(None, &cancel, limits().work)
        .unwrap();
    let exact = before
        + size_of::<Root>()
        + label.len()
        + size_of::<Vec<u16>>()
        + size_of::<DomStyleCssomRoot>();
    let root = top.document().node();
    let stamp = session.snapshot().stamp();
    let native_limits = session_limits(limits()).hosted;
    let mut native_short = native_limits;
    native_short.canonical = surgeist_dom::DomHostedLimits::new(
        native_limits.canonical.records(),
        native_limits.canonical.retained_bytes(),
        2048,
        native_limits.canonical.work(),
    );
    hosted
        .set_limits(&mut session.dom(), native_short, &cancel)
        .unwrap();
    let refused = styles.construct_document_sheet(
        &mut session.dom(),
        &hosted,
        top,
        Default::default(),
        fresh(label),
        &cancel,
    );
    assert!(
        matches!(
            refused,
            Err(DomConstructedSheetError::Construction(
                surgeist_dom::DomHostedSheetConstructionError::Hosted(
                    surgeist_dom::DomHostedError::Dom(DomError::ResourceLimit {
                        resource: surgeist_dom::DomResource::SourceRetainedBytes,
                        ..
                    })
                )
            ))
        ),
        "{refused:?}"
    );
    assert!(session.dom().style_cssom_root(root).unwrap().is_none());
    assert!(styles.snapshot(root).unwrap().is_none());
    hosted
        .set_limits(&mut session.dom(), native_limits, &cancel)
        .unwrap();
    let unpublished = DomStyleCssomRoot::new(
        root,
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let unpublished_revision = unpublished.snapshot().unwrap().revision().clone();
    let stopped = hosted.with_current_top_level(&mut session.dom(), top, &cancel, |source, dom| {
        let staged = source.prepare_document_sheet(
            dom.raw_mut(),
            top.global(),
            top.document(),
            &unpublished,
            Default::default(),
            &cancel,
        )?;
        cancel.store(true, Ordering::Relaxed);
        match staged.publish() {
            Ok(_) => Ok(()),
            Err(error) => Err(DomConstructedSheetError::from(error)),
        }
    });
    assert!(matches!(
        stopped,
        Err(DomConstructedSheetError::Construction(
            surgeist_dom::DomHostedSheetConstructionError::Hosted(
                surgeist_dom::DomHostedError::Dom(DomError::Cancelled)
            )
        ))
    ));
    assert_eq!(
        unpublished.snapshot().unwrap().revision(),
        &unpublished_revision
    );
    assert!(session.dom().style_cssom_root(root).unwrap().is_none());
    cancel.store(false, Ordering::Relaxed);
    let mut tight = limits();
    tight.storage_bytes = exact - 1;
    styles.set_limits(&session.dom(), tight, &cancel).unwrap();
    assert!(matches!(
        styles.construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            fresh(label),
            &cancel
        ),
        Err(DomConstructedSheetError::Style(
            DomStyleRuntimeError::Admission
        ))
    ));
    tight.storage_bytes = exact;
    styles.set_limits(&session.dom(), tight, &cancel).unwrap();
    assert!(matches!(
        styles.construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            DomConstructedSheetOwnerInit {
                label,
                limits: CssomLimits {
                    max_revision: 0,
                    ..Default::default()
                }
            },
            &cancel
        ),
        Err(DomConstructedSheetError::Construction(
            surgeist_dom::DomHostedSheetConstructionError::Cssom(
                surgeist_cssom::CssomError::RevisionExhausted
            )
        ))
    ));
    assert!(matches!(
        styles.construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            fresh(label),
            &AtomicBool::new(true)
        ),
        Err(DomConstructedSheetError::Hosted(
            DomHostedRuntimeError::Cancelled
        ))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    assert!(session.dom().style_cssom_root(root).unwrap().is_none());
    assert!(styles.snapshot(root).unwrap().is_none());
    assert_eq!(state.borrow().roots.capacity(), 0);
    let sheet = styles
        .construct_document_sheet(
            &mut session.dom(),
            &hosted,
            top,
            Default::default(),
            fresh(label),
            &cancel,
        )
        .unwrap();
    assert_eq!(sheet.revision().value(), 1);
    assert_eq!(
        state
            .borrow()
            .storage(None, &cancel, limits().work)
            .unwrap(),
        exact
    );
}

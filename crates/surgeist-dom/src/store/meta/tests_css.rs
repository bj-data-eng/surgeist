//! Owning CSS admission/custody controls over actual canonical Meta nodes.
use super::*;
use std::sync::atomic::Ordering;

fn policy() -> DomLimits {
    super::tests::policy()
        .with_meta_processing_work(100_000_000)
        .with_meta_theme_color_scratch_bytes(1_000_000)
        .unwrap()
        .with_meta_color_scheme_limits(surgeist_css::CssUtf16PropertyValueLimits {
            input: surgeist_css::CssUtf16InputLimits::default(),
            max_work: 1_000_000,
            max_live_bytes: 1_000_000,
            max_retained_bytes: 1_000_000,
        })
        .with_meta_color_scheme_retained_bytes(1_000_000)
        .unwrap()
}
fn fixture(name: Option<&str>, content: &str) -> (Dom, DocumentId, NodeId) {
    let mut dom = Dom::new(policy()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let root = batch.create_element_ns(&document, crate::Namespace::html(), "head");
    let meta = batch.create_element_ns(&document, crate::Namespace::html(), "meta");
    batch.set_attribute(&meta, "content", content);
    if let Some(name) = name {
        batch.set_attribute(&meta, "name", name);
    }
    batch.insert_before(&root, &document, None);
    batch.insert_before(&meta, &root, None);
    let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
    (
        dom,
        commit.document(&document).unwrap(),
        commit.created(&meta).unwrap().unwrap(),
    )
}
fn draft(dom: &Dom) -> Draft {
    Draft::new(dom, Arc::new(BatchIdentity), &AtomicBool::new(false)).unwrap()
}
fn released(draft: &Draft) {
    assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
}

#[test]
fn theme_whole_color_preserves_literal_and_refuses_missing_facts_without_fallback() {
    let cancel = AtomicBool::new(false);
    // Actual canonical content; this called-operation unit does not supply UI,
    // media, palette, scheme, device, or renderer facts.
    let (dom, document, meta) = fixture(None, "  #123abc  ");
    let mut draft = draft(&dom);
    super::tests::source_budget(&mut draft, 1_000_000, 1_000_000);
    let content = draft
        .meta_attribute(Key::Existing(meta), Attr::Content, &cancel)
        .unwrap()
        .unwrap();
    let color = color::parse(&draft, content, policy(), &cancel).unwrap();
    assert_eq!(
        color,
        Some(surgeist_color::Color::from_srgba8([0x12, 0x3a, 0xbc, 255]))
    );
    released(&draft);
    assert_eq!(
        color::parse(&draft, &"not-a-color".into(), policy(), &cancel).unwrap(),
        None
    );
    released(&draft);
    assert!(matches!(
        color::parse(&draft, &"currentColor".into(), policy(), &cancel),
        Err(DomError::Meta(Failure::ThemeColorUnavailable(_)))
    ));
    released(&draft);
    assert_eq!(
        draft
            .document_metadata(Key::Existing(document.node()))
            .unwrap()
            .meta_theme_color(),
        &crate::DomMetaThemeColorState::Known(None)
    );
}

#[test]
fn theme_scratch_source_and_cancellation_refusals_reclaim_actual_banks() {
    let cancel = AtomicBool::new(false);
    let (dom, _, _) = fixture(None, "#123abc");
    let mut draft = draft(&dom);
    let input: DocumentString = "#123abc".into();
    super::tests::source_budget(&mut draft, 1_000_000, 1_000_000);
    assert!(matches!(
        color::parse(
            &draft,
            &input,
            policy().with_meta_theme_color_scratch_bytes(0).unwrap(),
            &cancel
        ),
        Err(DomError::Meta(Failure::ThemeColorScratchLimit { limit: 0 }))
    ));
    released(&draft);
    super::tests::source_budget(&mut draft, 0, 1_000_000);
    assert!(matches!(
        color::parse(&draft, &input, policy(), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    released(&draft);
    super::tests::source_budget(&mut draft, 1_000_000, 1_000_000);
    let spent = draft.meta_work.get();
    draft.meta_work_limit = spent + 1;
    assert!(matches!(
        color::parse(&draft, &input, policy(), &cancel),
        Err(DomError::Meta(Failure::WorkLimit { .. }))
    ));
    assert!(draft.meta_work.get() > spent);
    released(&draft);
    draft.meta_work_limit = 100_000_000;
    cancel.store(true, Ordering::Release);
    assert_eq!(
        color::parse(&draft, &input, policy(), &cancel),
        Err(DomError::Cancelled)
    );
    released(&draft);
    cancel.store(false, Ordering::Release);
    assert!(
        color::parse(&draft, &input, policy(), &cancel)
            .unwrap()
            .is_some()
    );
    released(&draft);
}

#[test]
fn scheme_equal_recomputation_preserves_source_and_refusal_preserves_old_custody() {
    let cancel = AtomicBool::new(false);
    let (dom, document, _) = fixture(Some("color-scheme"), "light dark");
    let mut draft = draft(&dom);
    let document = Key::Existing(document.node());
    let old = draft
        .document_metadata(document)
        .unwrap()
        .meta_color_scheme
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .report
        .clone();
    let old_units = draft.units;
    super::tests::source_budget(&mut draft, 0, 1_000_000);
    assert!(matches!(
        draft.meta_candidates(document, false, true, policy(), &cancel),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            ..
        })
    ));
    released(&draft);
    assert!(Arc::ptr_eq(
        &old,
        &draft
            .document_metadata(document)
            .unwrap()
            .meta_color_scheme
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .report
    ));
    assert_eq!(draft.units, old_units);
    super::tests::source_budget(&mut draft, 1_000_000, 1_000_000);
    draft
        .meta_candidates(document, false, true, policy(), &cancel)
        .unwrap();
    released(&draft);
    assert!(Arc::ptr_eq(
        &old,
        &draft
            .document_metadata(document)
            .unwrap()
            .meta_color_scheme
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .report
    ));
    assert_eq!(draft.units, old_units);
    cancel.store(true, Ordering::Release);
    assert_eq!(
        draft.meta_candidates(document, false, true, policy(), &cancel),
        Err(DomError::Cancelled)
    );
    released(&draft);
    assert!(Arc::ptr_eq(
        &old,
        &draft
            .document_metadata(document)
            .unwrap()
            .meta_color_scheme
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .report
    ));
    cancel.store(false, Ordering::Release);
    draft
        .meta_candidates(document, false, true, policy(), &cancel)
        .unwrap();
    released(&draft);
}

#[test]
fn theme_actual_source_work_one_short_refuses_and_same_owner_retry_keeps_spent_work() {
    let cancel = AtomicBool::new(false);
    let (dom, _, _) = fixture(None, "#123abc");
    let input: DocumentString = "#123abc".into();
    let mut baseline = draft(&dom);
    super::tests::source_budget(&mut baseline, 1_000_000, 1_000_000);
    let started = baseline.source_work.load(Ordering::Relaxed);
    let expected = color::parse(&baseline, &input, policy(), &cancel).unwrap();
    let cost = baseline.source_work.load(Ordering::Relaxed) - started;
    assert!(cost > 0);
    // This calibrates the real caller admission boundary, not an independent
    // numeric tariff. The independent outcome is a complete literal or a
    // precise refusal without retained scratch; retry never refunds spent work.
    for missing in [0, 1] {
        let mut current = draft(&dom);
        super::tests::source_budget(&mut current, 1_000_000, 1_000_000);
        let limits = current.source_limits.unwrap();
        current.source_limits = Some(crate::DomSourceLimits::new(
            limits.retained_bytes(),
            limits.retained_string_code_units(),
            current.source_work.load(Ordering::Relaxed) + cost - missing,
            limits.phase_publications(),
            limits.output_records(),
            limits.published_native_identities(),
            limits.nested_operations(),
        ));
        let actual = color::parse(&current, &input, policy(), &cancel);
        if missing == 0 {
            assert_eq!(actual.unwrap(), expected);
        } else {
            assert!(matches!(actual, Err(DomError::SourceWorkLimit { .. })));
            let spent = current.source_work.load(Ordering::Relaxed);
            current.source_limits = Some(limits);
            assert_eq!(
                color::parse(&current, &input, policy(), &cancel).unwrap(),
                expected
            );
            assert!(current.source_work.load(Ordering::Relaxed) > spent);
        }
        released(&current);
    }
}

#[test]
fn incomplete_style_prefix_keeps_last_completed_meta_result_until_real_tail_recomputes() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, original) = fixture(Some("theme-color"), "#0000ff");
    let head = dom.snapshot().node(original).unwrap().parent().unwrap();
    let mut setup = DomBatch::new();
    let container = setup.create_element_ns(document, crate::Namespace::html(), "div");
    let style = setup.create_element_ns(document, crate::Namespace::html(), "style");
    setup.set_attribute(&style, "type", "text/plain");
    setup.insert_before(&style, &container, None);
    let new_meta = setup.create_element_ns(document, crate::Namespace::html(), "meta");
    setup.set_attribute(&new_meta, "name", "theme-color");
    setup.set_attribute(&new_meta, "content", "#ff0000");
    setup.insert_before(&new_meta, &container, None);
    let commit = dom.apply(setup, &cancel).unwrap();
    let container = commit.created(&container).unwrap().unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let new_meta = commit.created(&new_meta).unwrap().unwrap();
    let mut update = DomBatch::new();
    update.set_attribute(style, "type", "text/css");
    dom.apply(update, &cancel).unwrap();
    let before = dom.snapshot();
    let limits = crate::DomSourceLimits::new(16_000_000, 1_000_000, 100_000_000, 64, 4096, 4096, 1);
    let mut lease = dom
        .begin_source_edit(
            crate::DomSourceEdit::insert_before(container, head, Some(original)),
            limits,
            &cancel,
        )
        .unwrap();
    let crate::DomSourceProgress::Request(crate::DomSourceRequest::Style(request)) =
        lease.advance(&cancel).unwrap()
    else {
        panic!("actual first connected Style requires its CSP observation");
    };
    assert_eq!(request.source().element(), style);
    assert_eq!(
        request.action(),
        crate::DomStyleAction::Update(surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp)
    );
    let partial = lease.snapshot();
    assert!(partial.incomplete_source_operation().is_some());
    assert_eq!(partial.node(new_meta).unwrap().parent(), Some(container));
    assert_eq!(
        partial
            .document(document)
            .unwrap()
            .metadata()
            .meta_theme_color(),
        &crate::DomMetaThemeColorState::Known(Some(surgeist_color::Color::from_srgba8([
            0, 0, 255, 255
        ])))
    );
    let operation = lease.operation();
    drop(lease);
    let mut lease = dom.resume_source_edit(operation, limits, &cancel).unwrap();
    // A controlled external evaluation of this literal complete CSP list.
    // This owning fixture claims no native CSP parser, Window, or CSSOM sheet.
    let block = crate::DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    );
    lease.accept_inline_csp(&request, &block, &cancel).unwrap();
    assert!(matches!(
        lease.advance(&cancel).unwrap(),
        crate::DomSourceProgress::Complete(_)
    ));
    let complete = lease.snapshot();
    assert!(complete.incomplete_source_operation().is_none());
    assert_eq!(
        complete
            .document(document)
            .unwrap()
            .metadata()
            .meta_theme_color(),
        &crate::DomMetaThemeColorState::Known(Some(surgeist_color::Color::from_srgba8([
            255, 0, 0, 255
        ])))
    );
    assert_eq!(
        before
            .document(document)
            .unwrap()
            .metadata()
            .meta_theme_color(),
        partial
            .document(document)
            .unwrap()
            .metadata()
            .meta_theme_color()
    );
}

#[test]
fn theme_empty_media_guard_preserves_non_ascii_and_nul_unavailability() {
    let cancel = AtomicBool::new(false);
    for media in ["\u{a0}", " \t\0\r\n", "\u{2003}"] {
        let (mut dom, _, meta) = fixture(None, "#123abc");
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut edit = DomBatch::new();
        edit.set_attribute(meta, "media", media);
        edit.set_attribute(meta, "name", "theme-color");
        assert_eq!(
            dom.apply(edit, &cancel).unwrap_err(),
            DomError::Meta(Failure::ThemeColorMediaUnavailable)
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.snapshot()
                .node(meta)
                .unwrap()
                .as_element()
                .unwrap()
                .attribute(&"name".into())
                .is_none()
        );
    }
}

#[test]
fn theme_ascii_empty_media_scan_is_bounded_and_cancelled_without_selection_changes() {
    let cancel = AtomicBool::new(false);
    let (mut dom, document, meta) = fixture(None, "#123abc");
    let mut edit = DomBatch::new();
    edit.set_attribute(meta, "media", " ".repeat(65_536));
    edit.set_attribute(meta, "name", "theme-color");
    dom.apply(edit, &cancel).unwrap();
    let mut draft = draft(&dom);
    let document = Key::Existing(document.node());
    let selected = *draft
        .document_metadata(document)
        .unwrap()
        .meta_theme_color();
    super::tests::source_budget(&mut draft, 1_000_000, 1_000_000);
    draft.meta_work_limit = draft.meta_work.get() + 1_000;
    assert!(matches!(
        draft.meta_candidates(document, true, false, policy(), &cancel),
        Err(DomError::Meta(Failure::WorkLimit { .. }))
    ));
    assert_eq!(
        draft
            .document_metadata(document)
            .unwrap()
            .meta_theme_color(),
        &selected
    );
    released(&draft);
    draft.meta_work_limit = 100_000_000;
    cancel.store(true, Ordering::Release);
    assert_eq!(
        draft.meta_candidates(document, true, false, policy(), &cancel),
        Err(DomError::Cancelled)
    );
    assert_eq!(
        draft
            .document_metadata(document)
            .unwrap()
            .meta_theme_color(),
        &selected
    );
    released(&draft);
    cancel.store(false, Ordering::Release);
    draft
        .meta_candidates(document, true, false, policy(), &cancel)
        .unwrap();
    assert_eq!(
        draft
            .document_metadata(document)
            .unwrap()
            .meta_theme_color(),
        &selected
    );
    released(&draft);
}

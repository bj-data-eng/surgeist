//! Ordinary DOM notifications preserve their actual ingestion context and order.
use std::{cell::RefCell, sync::atomic::AtomicBool};
use surgeist_css::*;
use surgeist_cssom::*;
use surgeist_dom::*;
fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomLimits {
    DomLimits::new(
        256,
        100_000,
        512,
        256,
        256,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_base_processing(64, 1_000_000)
    .unwrap()
}
fn requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}
fn support(enabled: bool, decide: bool) -> CssomDeclarationSupport {
    CssomDeclarationSupport::try_new(if enabled {
        vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            if decide {
                CssomUsabilityRequirement::WholeDeclarationDecision
            } else {
                CssomUsabilityRequirement::AllCheckedValues
            },
        )]
    } else {
        vec![]
    })
    .unwrap()
}
fn version(enabled: bool, decide: bool) -> CssomInputVersion {
    CssomInputVersion {
        role: CssomInputRole::Support,
        identity: format!("width:{enabled}:{decide}"),
        revision: 1,
    }
}
fn row(
    element: NodeId,
    document: impl Into<NodeOperand>,
    enabled: bool,
    decide: bool,
) -> DomInlineStyleProfile {
    DomInlineStyleProfile::new(
        element,
        document,
        CssParserContext::default(),
        version(enabled, decide),
        support(enabled, decide),
    )
    .unwrap()
}
fn context(
    dom: &Dom,
    batch: &DomBatch,
    rows: Vec<DomInlineStyleProfile>,
) -> DomInlineStyleBatchContext {
    DomInlineStyleBatchContext::new(batch, dom.snapshot().stamp(), rows).unwrap()
}
fn no_decisions(
    _: &[CssomDeclarationCandidate],
    _: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    panic!("no whole declaration decision")
}
fn admission(
    candidates: &[CssomDeclarationCandidate],
    context: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    let version = context
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Support)
        .unwrap()
        .version
        .clone();
    Ok(CssomAdmission::new(
        candidates
            .iter()
            .map(|c| {
                c.decide(version.clone(), CssomDeclarationUsability::Usable)
                    .unwrap()
            })
            .collect(),
        context.inputs.iter().map(|i| i.version.clone()).collect(),
    ))
}
struct Fixture {
    dom: Dom,
    a: DocumentId,
    b: DocumentId,
    root: NodeId,
    left: NodeId,
    right: NodeId,
    element: NodeId,
    other: NodeId,
}
fn fixture() -> Fixture {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let a = batch.create_document();
    let b = batch.create_document();
    let root = batch.create_element_ns(&a, Namespace::html(), "main");
    let left = batch.create_element_ns(&a, Namespace::html(), "section");
    let right = batch.create_element_ns(&a, Namespace::html(), "aside");
    let element = batch.create_element_ns(&a, Namespace::html(), "div");
    let other = batch.create_element_ns(&a, Namespace::html(), "span");
    batch.insert_before(&root, &a, None);
    batch.insert_before(&left, &root, None);
    batch.insert_before(&right, &root, None);
    batch.insert_before(&element, &left, None);
    batch.insert_before(&other, &right, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    let f = Fixture {
        a: commit.document(&a).unwrap(),
        b: commit.document(&b).unwrap(),
        root: commit.created(&root).unwrap().unwrap(),
        left: commit.created(&left).unwrap().unwrap(),
        right: commit.created(&right).unwrap().unwrap(),
        element: commit.created(&element).unwrap().unwrap(),
        other: commit.created(&other).unwrap().unwrap(),
        dom,
    };
    let mut f = f;
    f.dom
        .configure_inline_styles(
            DomInlineStyleLimits::new(CssomLimits::default(), 32, 64_000_000, 8_000_000, 4_000_000)
                .unwrap(),
            &cancel(),
        )
        .unwrap();
    f
}
fn bind(f: &mut Fixture, element: NodeId) -> CssStyleProperties {
    let input = DomInlineStyleInputs::new(
        f.dom.snapshot().document(f.a).unwrap().stamp(),
        CssParserContext::default(),
        version(true, false),
        support(true, false),
    )
    .unwrap();
    f.dom
        .inline_style(element, &input, requests(), no_decisions, &cancel())
        .unwrap()
}
fn capture(dom: &Dom, element: NodeId) -> DomInlineStyleCapture {
    dom.capture_inline_style(element).unwrap().unwrap()
}
fn width(c: &DomInlineStyleCapture) -> String {
    c.properties()
        .get_property_value("width", requests().declaration)
        .unwrap()
}
fn packet(c: &DomInlineStyleCapture) -> &CssomContext {
    c.properties()
        .snapshot()
        .block(c.properties().handle().id())
        .unwrap()
        .inline_context()
        .unwrap()
        .context()
}

#[test]
fn setters_same_value_and_removal_keep_real_block_and_return_both_publications() {
    let mut f = fixture();
    let element = f.element;
    let handle = bind(&mut f, element);
    let old = capture(&f.dom, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    let result = f
        .dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert!(result.inline().unwrap().owner_effects().is_empty());
    assert_eq!(capture(&f.dom, element).properties().handle(), &handle);
    assert_eq!(width(&capture(&f.dom, element)), "3px");
    assert_eq!(width(&old), "");
    let before = capture(&f.dom, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, true)]);
    let calls = RefCell::new(0);
    f.dom
        .apply_with_inline_styles(
            batch,
            inputs,
            requests(),
            |c, ctx| {
                *calls.borrow_mut() += 1;
                admission(c, ctx)
            },
            &cancel(),
        )
        .unwrap();
    assert_eq!(*calls.borrow(), 1);
    assert_ne!(
        packet(&capture(&f.dom, element)).inputs[0].version,
        packet(&before).inputs[0].version
    );
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "style");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, false, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let after = capture(&f.dom, element);
    assert_eq!(width(&after), "");
    assert!(
        after
            .dom()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .iter()
            .all(|a| a.name().local_name().to_utf8().unwrap() != "style")
    );
    assert_eq!(width(&before), "3px");
}

#[test]
fn set_in_a_then_adopt_b_preserves_parse_facts_until_actual_set_in_b() {
    let mut f = fixture();
    let element = f.element;
    let handle = bind(&mut f, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    batch.adopt(element, f.b);
    let inputs = context(
        &f.dom,
        &batch,
        vec![
            row(element, f.a, true, false),
            row(element, f.b, false, false),
        ],
    );
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let moved = capture(&f.dom, element);
    assert_eq!(width(&moved), "3px");
    assert_eq!(moved.properties().handle(), &handle);
    assert_eq!(
        moved.dom().node(element).unwrap().owner_document(),
        Some(f.b)
    );
    assert!(
        packet(&moved)
            .inputs
            .iter()
            .any(|i| matches!(&i.data,CssomInputData::Support(s) if s.properties().is_empty()))
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.b, false, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(width(&capture(&f.dom, element)), "");
    assert_eq!(width(&moved), "3px");
}

#[test]
fn a_b_a_same_final_text_keeps_intermediate_ingestion_and_current_versions() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    batch.adopt(element, f.b);
    batch.set_attribute(element, "style", "width: 3px");
    batch.adopt(element, f.a);
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![row(element, f.a, true, true), row(element, f.b, true, true)],
    );
    let observed = RefCell::new(Vec::new());
    f.dom
        .apply_with_inline_styles(
            batch,
            inputs,
            requests(),
            |c, ctx| {
                observed.borrow_mut().push(
                    ctx.inputs
                        .iter()
                        .find(|i| i.version.role == CssomInputRole::Document)
                        .unwrap()
                        .version
                        .clone(),
                );
                admission(c, ctx)
            },
            &cancel(),
        )
        .unwrap();
    let observed = observed.borrow();
    assert_eq!(observed.len(), 3);
    assert_eq!(observed[0].identity, observed[2].identity);
    assert_ne!(observed[0].identity, observed[1].identity);
    assert!(
        observed[0].revision < observed[1].revision && observed[1].revision < observed[2].revision
    );
    assert_eq!(width(&capture(&f.dom, element)), "3px");
}

#[test]
fn moved_descendant_is_discovered_by_later_ancestor_adoption() {
    let mut f = fixture();
    let element = f.element;
    let other = f.other;
    bind(&mut f, element);
    bind(&mut f, other);
    let mut batch = DomBatch::new();
    batch.insert_before(element, f.right, None);
    batch.adopt(f.right, f.b);
    batch.set_attribute(element, "style", "width: 8px");
    batch.set_attribute(other, "style", "width: 9px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![
            row(element, f.a, true, false),
            row(element, f.b, true, false),
            row(other, f.a, true, false),
            row(other, f.b, true, false),
        ],
    );
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(width(&capture(&f.dom, element)), "8px");
    assert_eq!(width(&capture(&f.dom, other)), "9px");
    assert_eq!(
        f.dom.snapshot().node(element).unwrap().owner_document(),
        Some(f.b)
    );
}

#[test]
fn created_document_ticket_qualifies_real_staged_context_and_actual_creation_output() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    batch.adopt(element, &document);
    batch.set_attribute(element, "style", "width: 5px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![
            row(element, f.a, true, false),
            row(element, &document, true, false),
        ],
    );
    let result = f
        .dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let doc = result.dom().document(&document).unwrap();
    let captured = capture(&f.dom, element);
    assert_eq!(
        captured.dom().node(element).unwrap().owner_document(),
        Some(doc)
    );
    assert_eq!(width(&captured), "5px");
    let first = packet(&captured)
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Document)
        .unwrap()
        .version
        .clone();
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 6px");
    let inputs = context(&f.dom, &batch, vec![row(element, doc, true, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let later = capture(&f.dom, element);
    let next = &packet(&later)
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Document)
        .unwrap()
        .version;
    assert_eq!(next.identity, first.identity);
    assert!(next.revision > first.revision);
}

#[test]
fn missing_duplicate_stale_and_late_dom_refusal_publish_neither_owner() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let old = capture(&f.dom, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![]);
    assert!(
        matches!(f.dom.apply_with_inline_styles(batch,inputs,requests(),no_decisions,&cancel()),Err(DomInlineStyleError::Dom(DomError::InlineStyleProfileMissing{element:e})) if e==element)
    );
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![
            row(element, f.a, true, false),
            row(element, f.a, true, false),
        ],
    );
    assert!(matches!(
        f.dom
            .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::Dom(
            DomError::InlineStyleProfileConflict { .. }
        ))
    ));
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    batch.insert_before(f.root, element, None);
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, true)]);
    assert!(matches!(
        f.dom
            .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::Dom(DomError::HierarchyRequest))
    ));
    let current = capture(&f.dom, element);
    assert_eq!(current.dom().stamp(), old.dom().stamp());
    assert_eq!(
        current.properties().snapshot().revision(),
        old.properties().snapshot().revision()
    );
    assert_eq!(width(&current), "");
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    let mut edit = DomBatch::new();
    edit.set_attribute(f.other, "id", "new");
    f.dom.apply(edit, &cancel()).unwrap();
    assert!(matches!(
        f.dom
            .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::StaleContext)
    ));
}

#[test]
fn resolver_refusal_consumes_observed_versions_but_preserves_both_captures() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let old = capture(&f.dom, element);
    let observed = RefCell::new(Vec::new());
    for fail in [true, false] {
        let mut batch = DomBatch::new();
        batch.set_attribute(element, "style", "width: 7px");
        let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, true)]);
        let result = f.dom.apply_with_inline_styles(
            batch,
            inputs,
            requests(),
            |c, ctx| {
                observed.borrow_mut().push(
                    ctx.inputs
                        .iter()
                        .find(|i| i.version.role == CssomInputRole::Owner)
                        .unwrap()
                        .version
                        .clone(),
                );
                if fail {
                    Err(CssomError::InvalidInput("owner refuses"))
                } else {
                    admission(c, ctx)
                }
            },
            &cancel(),
        );
        if fail {
            assert!(matches!(
                result,
                Err(DomInlineStyleError::Cssom(CssomError::InvalidInput(
                    "owner refuses"
                )))
            ));
            let current = capture(&f.dom, element);
            assert_eq!(current.dom().stamp(), old.dom().stamp());
            assert_eq!(
                current.properties().snapshot().revision(),
                old.properties().snapshot().revision()
            );
        } else {
            result.unwrap();
        }
    }
    let values = observed.borrow();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].identity, values[1].identity);
    assert!(values[0].revision < values[1].revision);
    assert_eq!(width(&capture(&f.dom, element)), "7px");
    assert_eq!(width(&old), "");
}

#[test]
fn no_binding_unrelated_names_and_absent_removal_need_no_profiles_or_inline_commit() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let mut batch = DomBatch::new();
    batch.remove_attribute(element, "style");
    batch.set_attribute(f.other, "style", "width: 4px");
    batch.set_attribute_ns(
        element,
        Namespace::from_uri("urn:test"),
        "style",
        "unrelated",
    );
    let inputs = context(&f.dom, &batch, vec![]);
    let result = f
        .dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert!(result.inline().is_none());
    assert_eq!(width(&capture(&f.dom, element)), "");
    assert!(f.dom.capture_inline_style(f.other).unwrap().is_none());
}

fn assert_unchanged(dom: &Dom, old: &DomInlineStyleCapture) {
    let current = capture(dom, old.element());
    assert_eq!(current.dom().stamp(), old.dom().stamp());
    assert_eq!(
        current.properties().snapshot().revision(),
        old.properties().snapshot().revision()
    );
    assert_eq!(current.properties().handle(), old.properties().handle());
    assert_eq!(width(&current), width(old));
    let attributes = |c: &DomInlineStyleCapture| {
        c.dom()
            .node(c.element())
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .to_vec()
    };
    assert_eq!(attributes(&current), attributes(old));
}
#[test]
fn second_element_resolver_refusal_after_first_staged_update_publishes_neither_owner() {
    let mut f = fixture();
    let a = f.element;
    let b = f.other;
    bind(&mut f, a);
    bind(&mut f, b);
    let old_a = capture(&f.dom, a);
    let old_b = capture(&f.dom, b);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "style", "width: 3px");
    batch.set_attribute(b, "style", "width: 4px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![row(a, f.a, true, true), row(b, f.a, true, true)],
    );
    let mut calls = 0;
    let error = f.dom.apply_with_inline_styles(
        batch,
        inputs,
        requests(),
        |c, ctx| {
            calls += 1;
            if calls == 2 {
                Err(CssomError::InvalidInput("second declaration refused"))
            } else {
                admission(c, ctx)
            }
        },
        &cancel(),
    );
    assert!(matches!(
        error,
        Err(DomInlineStyleError::Cssom(CssomError::InvalidInput(
            "second declaration refused"
        )))
    ));
    assert_eq!(calls, 2);
    assert_unchanged(&f.dom, &old_a);
    assert_unchanged(&f.dom, &old_b);
}
#[test]
fn cancellation_after_both_resolver_calls_keeps_both_original_owners() {
    let mut f = fixture();
    let a = f.element;
    let b = f.other;
    bind(&mut f, a);
    bind(&mut f, b);
    let old_a = capture(&f.dom, a);
    let old_b = capture(&f.dom, b);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "style", "width: 3px");
    batch.set_attribute(b, "style", "width: 4px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![row(a, f.a, true, true), row(b, f.a, true, true)],
    );
    let cancelled = cancel();
    let mut calls = 0;
    let result = f.dom.apply_with_inline_styles(
        batch,
        inputs,
        requests(),
        |c, ctx| {
            calls += 1;
            if calls == 2 {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            admission(c, ctx)
        },
        &cancelled,
    );
    assert!(matches!(
        result,
        Err(DomInlineStyleError::Dom(DomError::Cancelled))
    ));
    assert_eq!(calls, 2);
    assert_unchanged(&f.dom, &old_a);
    assert_unchanged(&f.dom, &old_b);
}
#[test]
fn resolver_unwind_drops_private_replay_and_next_real_operation_succeeds() {
    let mut f = fixture();
    let a = f.element;
    let b = f.other;
    bind(&mut f, a);
    bind(&mut f, b);
    let old_a = capture(&f.dom, a);
    let old_b = capture(&f.dom, b);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "style", "width: 3px");
    batch.set_attribute(b, "style", "width: 4px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![row(a, f.a, true, true), row(b, f.a, true, true)],
    );
    let mut calls = 0;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        f.dom.apply_with_inline_styles(
            batch,
            inputs,
            requests(),
            |c, ctx| {
                calls += 1;
                assert!(calls != 2, "second resolver unwinds");
                admission(c, ctx)
            },
            &cancel(),
        )
    }));
    assert!(result.is_err());
    assert_eq!(calls, 2);
    assert_unchanged(&f.dom, &old_a);
    assert_unchanged(&f.dom, &old_b);
    let mut batch = DomBatch::new();
    batch.set_attribute(a, "style", "width: 6px");
    batch.set_attribute(b, "style", "width: 7px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![row(a, f.a, true, false), row(b, f.a, true, false)],
    );
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(width(&capture(&f.dom, a)), "6px");
    assert_eq!(width(&capture(&f.dom, b)), "7px");
}
#[test]
fn notification_uses_exact_utf16_and_css_preprocessing_without_rewriting_dom() {
    let mut f = fixture();
    let element = f.element;
    let handle = bind(&mut f, element);
    let old = capture(&f.dom, element);
    let mut units: Vec<u16> = "--x:".encode_utf16().collect();
    units.extend([13, 10, 13, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00]);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", DocumentString::from_utf16(units.clone()));
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let current = capture(&f.dom, element);
    let block = current.properties().snapshot().block(handle.id()).unwrap();
    let source = block
        .replacement_input()
        .unwrap()
        .parse_source()
        .unwrap()
        .source();
    assert_eq!(source.as_str(), "--x:\n\n\n�😀�a�");
    assert_eq!(source.original_utf16().unwrap().as_slice(), units);
    assert_eq!(source.original_utf16_range(4..5), Some(4..6));
    assert_eq!(source.original_utf16_range(10..14), Some(9..11));
    assert_eq!(
        current
            .dom()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()[0]
            .value()
            .code_units(),
        units
    );
    assert_eq!(width(&old), "");
}
#[test]
fn selected_grammar_preserves_profile_but_actual_document_supplies_mode() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let selected =
        CssParserContext::new(CssParserMode::Quirks).with_svg_glyph_orientation_vertical();
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 6px");
    let profile = DomInlineStyleProfile::new(
        element,
        f.a,
        selected,
        version(true, false),
        support(true, false),
    )
    .unwrap();
    let inputs = context(&f.dom, &batch, vec![profile]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let current = capture(&f.dom, element);
    assert!(packet(&current).inputs.iter().any(
        |i| i.data == CssomInputData::ParserMode(selected.with_mode(CssParserMode::Standards))
    ));
    assert_eq!(width(&current), "6px");
}
#[test]
fn producer_input_limit_refusal_is_atomic_and_does_not_reparse_old_capture() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let old = capture(&f.dom, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    let mut limit = requests();
    limit.input.max_input_units = 1;
    assert!(matches!(
        f.dom
            .apply_with_inline_styles(batch, inputs, limit, no_decisions, &cancel()),
        Err(DomInlineStyleError::Input(_))
    ));
    assert_unchanged(&f.dom, &old);
}

#[test]
fn actual_child_replacement_tracks_inserted_and_detached_bound_subtrees_separately() {
    let mut f = fixture();
    let element = f.element;
    let other = f.other;
    bind(&mut f, element);
    bind(&mut f, other);
    let mut batch = DomBatch::new();
    batch.replace_child(element, f.right, other);
    batch.adopt(f.right, f.b);
    batch.set_attribute(element, "style", "width: 8px");
    batch.set_attribute(other, "style", "width: 9px");
    let inputs = context(
        &f.dom,
        &batch,
        vec![
            row(element, f.a, true, false),
            row(element, f.b, true, false),
            row(other, f.a, true, false),
        ],
    );
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(
        f.dom.snapshot().node(element).unwrap().owner_document(),
        Some(f.b)
    );
    assert_eq!(
        f.dom.snapshot().node(other).unwrap().owner_document(),
        Some(f.a)
    );
    assert!(f.dom.snapshot().node(f.left).unwrap().children().is_empty());
    assert_eq!(width(&capture(&f.dom, element)), "8px");
    assert_eq!(width(&capture(&f.dom, other)), "9px");
}

#[test]
fn failed_observation_then_first_binding_uses_a_fresh_same_document_qualification() {
    let mut f = fixture();
    let element = f.element;
    bind(&mut f, element);
    let observed = RefCell::new(None);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 7px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, true)]);
    assert!(
        f.dom
            .apply_with_inline_styles(
                batch,
                inputs,
                requests(),
                |_, ctx| {
                    *observed.borrow_mut() = Some(
                        ctx.inputs
                            .iter()
                            .find(|i| i.version.role == CssomInputRole::Document)
                            .unwrap()
                            .version
                            .clone(),
                    );
                    Err(CssomError::InvalidInput("refused"))
                },
                &cancel()
            )
            .is_err()
    );
    let other = f.other;
    bind(&mut f, other);
    let current = capture(&f.dom, other);
    let version = &packet(&current)
        .inputs
        .iter()
        .find(|i| i.version.role == CssomInputRole::Document)
        .unwrap()
        .version;
    let old = observed.borrow();
    let old = old.as_ref().unwrap();
    assert_eq!(version.identity, old.identity);
    assert!(version.revision > old.revision);
}
#[test]
fn shadow_scope_and_detach_keep_block_identity_and_canonical_scope_current() {
    let mut f = fixture();
    let element = f.element;
    let handle = bind(&mut f, element);
    let old = capture(&f.dom, element);
    let mut batch = DomBatch::new();
    let shadow = batch.attach_shadow(f.left, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(element, &shadow, None);
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    let result = f
        .dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let shadow = result.dom().created(&shadow).unwrap().unwrap();
    assert_eq!(capture(&f.dom, element).tree_scope().root(), shadow);
    assert_eq!(capture(&f.dom, element).properties().handle(), &handle);
    assert_eq!(old.tree_scope().root(), f.a.node());
    let mut batch = DomBatch::new();
    batch.detach(element);
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(capture(&f.dom, element).tree_scope().root(), element);
    assert_eq!(width(&capture(&f.dom, element)), "");
}
#[test]
fn base_refreeze_between_true_setters_qualifies_each_ingestion_and_context_only_keeps_checked_values()
 {
    use surgeist_html_semantics::*;
    let mut f = fixture();
    let r = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let encoding = HtmlBaseEncoding::fresh();
    let fallback = HtmlBaseUrlRecord::observed(
        r.clone(),
        "about:blank".into(),
        HtmlBaseUrlScheme::Other,
        true,
        None,
    )
    .unwrap();
    let observations = [
        ("first", "https://first.test/"),
        ("second", "https://second.test/"),
    ]
    .map(|(raw, url)| {
        HtmlBaseParseObservation::new(
            raw.into(),
            fallback.clone(),
            encoding.clone(),
            r.clone(),
            HtmlBaseParseOutcome::Parsed {
                record: HtmlBaseUrlRecord::observed(
                    r.clone(),
                    url.into(),
                    HtmlBaseUrlScheme::Other,
                    false,
                    None,
                )
                .unwrap(),
                base_allowed: HtmlBaseAllowed::Allowed,
            },
        )
        .unwrap()
    });
    let facts = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            r,
            Some(fallback),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        observations.into(),
        HtmlBaseFactsLimits::new(64, 65536, 1_000_000),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    batch.bind_base_facts(f.dom.snapshot().document(f.a).unwrap().stamp(), &facts);
    let base = batch.create_element_ns(f.a, Namespace::html(), "base");
    batch.set_attribute(&base, "href", "first");
    batch.insert_before(&base, f.root, Some(f.left.into()));
    let result = f.dom.apply(batch, &cancel()).unwrap();
    let base = result.created(&base).unwrap().unwrap();
    let element = f.element;
    bind(&mut f, element);
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "style", "width: 3px");
    batch.set_attribute(base, "href", "second");
    batch.set_attribute(element, "style", "width: 4px");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, true, true)]);
    let observed = RefCell::new(Vec::new());
    f.dom
        .apply_with_inline_styles(
            batch,
            inputs,
            requests(),
            |c, ctx| {
                observed.borrow_mut().push(
                    ctx.inputs
                        .iter()
                        .find_map(|i| match &i.data {
                            CssomInputData::Base { url, .. } => url.clone(),
                            _ => None,
                        })
                        .unwrap(),
                );
                admission(c, ctx)
            },
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        *observed.borrow(),
        ["https://first.test/", "https://second.test/"]
    );
    let before = capture(&f.dom, element);
    assert_eq!(width(&before), "4px");
    let parse_versions = before
        .properties()
        .snapshot()
        .block(before.properties().handle().id())
        .unwrap()
        .admission_inputs()
        .to_vec();
    let mut batch = DomBatch::new();
    batch.set_attribute(base, "href", "first");
    let inputs = context(&f.dom, &batch, vec![row(element, f.a, false, false)]);
    f.dom
        .apply_with_inline_styles(batch, inputs, requests(), no_decisions, &cancel())
        .unwrap();
    let after = capture(&f.dom, element);
    assert_eq!(width(&after), "4px");
    assert_eq!(
        after
            .properties()
            .snapshot()
            .block(after.properties().handle().id())
            .unwrap()
            .admission_inputs(),
        parse_versions
    );
    assert!(packet(&after).inputs.iter().any(
        |i| matches!(&i.data,CssomInputData::Base{url:Some(url),..} if url=="https://first.test/")
    ));
}

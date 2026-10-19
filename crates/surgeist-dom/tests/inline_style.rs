//! Same-object inline foundation and precise context-free publication barriers.
use std::{cell::Cell, sync::atomic::AtomicBool};
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
}
fn policy() -> DomInlineStyleLimits {
    DomInlineStyleLimits::new(CssomLimits::default(), 64, 64_000_000, 8_000_000, 4_000_000).unwrap()
}
fn requests() -> DomInlineStyleRequestLimits {
    DomInlineStyleRequestLimits {
        input: CssUtf16InputLimits::default(),
        declaration: CssomDeclarationRequestLimits::default(),
    }
}
fn inputs(dom: &Dom, document: DocumentId) -> DomInlineStyleInputs {
    DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "inline-test-profile".into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            CssomUsabilityRequirement::AllCheckedValues,
        )])
        .unwrap(),
    )
    .unwrap()
}
fn no_decisions(
    _: &[CssomDeclarationCandidate],
    _: &CssomContext,
) -> Result<CssomAdmission, CssomError> {
    panic!("no whole-declaration candidate")
}
fn bind(dom: &mut Dom, document: DocumentId, element: NodeId) -> CssStyleProperties {
    let inputs = inputs(dom, document);
    dom.inline_style(element, &inputs, requests(), no_decisions, &cancel())
        .unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, NodeId, NodeId) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let root = b.create_element_ns(&document, Namespace::html(), "main");
    let a = b.create_element_ns(&document, Namespace::html(), "div");
    let sibling = b.create_element_ns(&document, Namespace::html(), "aside");
    b.insert_before(&root, &document, None);
    b.insert_before(&a, &root, None);
    b.insert_before(&sibling, &root, None);
    b.set_attribute(&a, "style", "width: 3px;");
    let c = dom.apply(b, &cancel()).unwrap();
    let ids = (
        c.document(&document).unwrap(),
        c.created(&root).unwrap().unwrap(),
        c.created(&a).unwrap().unwrap(),
        c.created(&sibling).unwrap().unwrap(),
    );
    dom.configure_inline_styles(policy(), &cancel()).unwrap();
    (dom, ids.0, ids.1, ids.2, ids.3)
}

#[test]
fn binding_is_lazy_same_object_and_capture_keeps_actual_owner_flags_and_initial_text() {
    let (mut dom, document, _, element, _) = fixture();
    let before = dom.snapshot();
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    let first = bind(&mut dom, document, element);
    let second = bind(&mut dom, document, element);
    assert_eq!(first, second);
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let capture = dom.capture_inline_style(element).unwrap().unwrap();
    assert_eq!(capture.properties().handle(), &first);
    assert_eq!(
        capture
            .properties()
            .get_property_value("width", requests().declaration)
            .unwrap(),
        "3px"
    );
    let block = capture.properties().snapshot().block(first.id()).unwrap();
    assert_eq!(block.flags(), CssomBlockFlags::default());
    assert!(block.parent().is_none());
    assert_eq!(block.owner(), Some(block.inline_context().unwrap().owner()));
    assert_eq!(capture.element(), element);
    assert_eq!(
        capture.dom().node(element).unwrap().owner_document(),
        Some(document)
    );
    assert_eq!(capture.tree_scope().root(), document.node());
}

#[test]
fn equal_text_elements_and_clone_import_have_independent_mutable_identity() {
    let (mut dom, document, _, element, sibling) = fixture();
    let mut b = DomBatch::new();
    b.set_attribute(sibling, "style", "width: 3px;");
    dom.apply(b, &cancel()).unwrap();
    let a = bind(&mut dom, document, element);
    let b = bind(&mut dom, document, sibling);
    assert_ne!(a, b);
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    let clone = edit.import_node(&old, element, document, false);
    let c = dom.apply(edit, &cancel()).unwrap();
    let clone = c.created(&clone).unwrap().unwrap();
    let third = bind(&mut dom, document, clone);
    assert_ne!(a, third);
    assert_ne!(b, third);
    assert_eq!(
        dom.capture_inline_style(clone)
            .unwrap()
            .unwrap()
            .properties()
            .get_property_value("width", requests().declaration)
            .unwrap(),
        "3px"
    );
}

#[test]
fn html_svg_mathml_are_included_other_namespaces_are_precisely_inapplicable() {
    let (mut dom, document, _, _, _) = fixture();
    let mut b = DomBatch::new();
    let html = b.create_element_ns(document, Namespace::html(), "span");
    let svg = b.create_element_ns(document, Namespace::svg(), "rect");
    let math = b.create_element_ns(document, Namespace::mathml(), "mi");
    let other = b.create_element_ns(document, Namespace::from_uri("urn:test"), "e");
    let c = dom.apply(b, &cancel()).unwrap();
    for t in [&html, &svg, &math] {
        let id = c.created(t).unwrap().unwrap();
        let handle = bind(&mut dom, document, id);
        assert_eq!(
            dom.capture_inline_style(id)
                .unwrap()
                .unwrap()
                .properties()
                .handle(),
            &handle
        );
    }
    let other = c.created(&other).unwrap().unwrap();
    let i = inputs(&dom, document);
    assert!(matches!(
        dom.inline_style(other, &i, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::Inapplicable)
    ));
    assert!(dom.capture_inline_style(other).unwrap().is_none());
}

#[test]
fn exact_canonical_utf16_is_untouched_while_css_retains_its_preprocessing_map() {
    let (mut dom, document, _, element, _) = fixture();
    let mut units: Vec<u16> = "--x:".encode_utf16().collect();
    units.extend([13, 10, 13, 12, 0, 0xd83d, 0xde00, 0xd800, 97, 0xdc00]);
    let text = DocumentString::from_utf16(units.clone());
    let mut b = DomBatch::new();
    b.set_attribute(element, "style", text);
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    let handle = bind(&mut dom, document, element);
    let capture = dom.capture_inline_style(element).unwrap().unwrap();
    let block = capture.properties().snapshot().block(handle.id()).unwrap();
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
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()[0]
            .value()
            .code_units(),
        units
    );
}

#[test]
fn bound_same_value_set_remove_and_retirement_refuse_atomically() {
    let (mut dom, document, _, element, _) = fixture();
    let handle = bind(&mut dom, document, element);
    let capture = dom.capture_inline_style(element).unwrap().unwrap();
    for remove in [false, true] {
        let before = dom.snapshot();
        let mut b = DomBatch::new();
        if remove {
            b.remove_attribute(element, "style");
        } else {
            b.set_attribute(element, "style", "width: 3px;");
        }
        assert_eq!(
            dom.apply(b, &cancel()).unwrap_err(),
            DomError::InlineStyleContextRequired {
                element,
                change: DomInlineStyleChange::Attribute
            }
        );
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.capture_inline_style(element)
                .unwrap()
                .unwrap()
                .properties()
                .handle(),
            &handle
        );
    }
    let mut b = DomBatch::new();
    b.detach(element);
    b.destroy(element);
    assert!(
        matches!(dom.apply(b, &cancel()), Err(DomError::InlineStyleContextRequired { element: id, .. }) if id == element)
    );
    assert_eq!(
        capture
            .properties()
            .get_property_value("width", requests().declaration)
            .unwrap(),
        "3px"
    );
}

#[test]
fn unrelated_attributes_unbound_edits_and_missing_removal_keep_bindings_current() {
    let (mut dom, document, _, element, sibling) = fixture();
    let handle = bind(&mut dom, document, element);
    let css = dom
        .capture_inline_style(element)
        .unwrap()
        .unwrap()
        .properties()
        .snapshot()
        .revision()
        .clone();
    let mut b = DomBatch::new();
    b.set_attribute(element, "title", "ok");
    b.set_attribute_ns(
        element,
        Namespace::from_uri("urn:test"),
        "p:style",
        "unrelated",
    );
    b.remove_attribute(element, "absent");
    b.set_attribute(sibling, "style", "width: 8px;");
    dom.apply(b, &cancel()).unwrap();
    let c = dom.capture_inline_style(element).unwrap().unwrap();
    assert_eq!(c.properties().handle(), &handle);
    assert_eq!(c.properties().snapshot().revision(), &css);
    assert!(dom.capture_inline_style(sibling).unwrap().is_none());
}

#[test]
fn same_scope_moves_reform_membership_before_later_subtree_scope_refusal() {
    let (mut dom, document, root, element, sibling) = fixture();
    let handle = bind(&mut dom, document, element);
    let mut b = DomBatch::new();
    b.insert_before(element, sibling, None);
    dom.apply(b, &cancel()).unwrap();
    assert_eq!(bind(&mut dom, document, element), handle);
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(sibling);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::InlineStyleContextRequired {
            element,
            change: DomInlineStyleChange::Context
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.snapshot().node(sibling).unwrap().parent(), Some(root));
    let mut b = DomBatch::new();
    let unrelated = b.create_element_ns(document, Namespace::html(), "section");
    b.insert_before(&unrelated, root, None);
    dom.apply(b, &cancel()).unwrap();
}

#[test]
fn adoption_and_new_tree_scope_require_context_without_recreating_the_binding() {
    let (mut dom, document, _, element, _) = fixture();
    let handle = bind(&mut dom, document, element);
    let mut b = DomBatch::new();
    let other = b.create_document();
    let c = dom.apply(b, &cancel()).unwrap();
    let other = c.document(&other).unwrap();
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.adopt(element, other);
    assert!(
        matches!(dom.apply(b, &cancel()), Err(DomError::InlineStyleContextRequired { element: id, change: DomInlineStyleChange::Context }) if id == element)
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(bind(&mut dom, document, element), handle);
}

#[test]
fn stale_foreign_wrong_document_and_mode_mismatch_publish_neither_owner() {
    let (mut dom, document, _, element, sibling) = fixture();
    let stale = inputs(&dom, document);
    let mut b = DomBatch::new();
    b.set_attribute(sibling, "title", "new");
    dom.apply(b, &cancel()).unwrap();
    let before = dom.snapshot();
    assert!(matches!(
        dom.inline_style(element, &stale, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::StaleContext)
    ));
    let wrong_mode = DomInlineStyleInputs::new(
        before.document(document).unwrap().stamp(),
        CssParserContext::new(CssParserMode::Quirks),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "s".into(),
            revision: 1,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    assert!(matches!(
        dom.inline_style(element, &wrong_mode, requests(), no_decisions, &cancel()),
        Err(DomInlineStyleError::ParserModeMismatch)
    ));
    let (foreign, foreign_document, _, _, _) = fixture();
    assert!(matches!(
        dom.inline_style(
            element,
            &inputs(&foreign, foreign_document),
            requests(),
            no_decisions,
            &cancel()
        ),
        Err(DomInlineStyleError::Dom(DomError::ForeignOwner))
    ));
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut b = DomBatch::new();
    let other = b.create_document();
    let c = dom.apply(b, &cancel()).unwrap();
    let other = c.document(&other).unwrap();
    let before = dom.snapshot();
    assert!(matches!(
        dom.inline_style(
            element,
            &inputs(&dom, other),
            requests(),
            no_decisions,
            &cancel()
        ),
        Err(DomInlineStyleError::StaleContext)
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.capture_inline_style(element).unwrap().is_none());
}

#[test]
fn producer_refusal_and_cancellation_leave_first_binding_unpublished() {
    let (mut dom, document, _, element, _) = fixture();
    let i = inputs(&dom, document);
    let before = dom.snapshot();
    let mut small = requests();
    small.input.max_input_units = 1;
    assert!(
        dom.inline_style(element, &i, small, no_decisions, &cancel())
            .is_err()
    );
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    assert!(
        dom.inline_style(
            element,
            &i,
            requests(),
            no_decisions,
            &AtomicBool::new(true)
        )
        .is_err()
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    bind(&mut dom, document, element);
}

#[test]
fn historical_capture_survives_owner_drop_without_retaining_an_executable_owner() {
    let (mut dom, document, _, element, _) = fixture();
    let handle = bind(&mut dom, document, element);
    let capture = dom.capture_inline_style(element).unwrap().unwrap();
    drop(dom);
    assert_eq!(capture.properties().handle(), &handle);
    assert_eq!(
        capture
            .properties()
            .get_property_value("width", requests().declaration)
            .unwrap(),
        "3px"
    );
    assert_eq!(
        capture.dom().node(element).unwrap().owner_document(),
        Some(document)
    );
}

#[test]
fn explicit_empty_support_and_absent_attribute_do_not_call_the_resolver() {
    let (mut dom, document, _, _, sibling) = fixture();
    let i = DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "empty".into(),
            revision: 1,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let calls = Cell::new(0);
    let handle = dom
        .inline_style(
            sibling,
            &i,
            requests(),
            |_, _| {
                calls.set(calls.get() + 1);
                panic!("absent input")
            },
            &cancel(),
        )
        .unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(
        dom.capture_inline_style(sibling)
            .unwrap()
            .unwrap()
            .properties()
            .length(requests().declaration)
            .unwrap(),
        0
    );
    assert_eq!(
        dom.inline_style(sibling, &i, requests(), no_decisions, &cancel())
            .unwrap(),
        handle
    );
    let mut b = DomBatch::new();
    b.remove_attribute(sibling, "style");
    let before = dom.snapshot();
    dom.apply(b, &cancel()).unwrap();
    assert_eq!(dom.snapshot().stamp(), before.stamp());
}

#[test]
fn independently_selected_profiles_at_one_dom_stamp_have_distinct_parser_qualifications() {
    let (mut dom, document, _, a, b) = fixture();
    let basis = dom.snapshot().document(document).unwrap().stamp();
    let ordinary = inputs(&dom, document);
    let svg = DomInlineStyleInputs::new(
        basis,
        CssParserContext::default().with_svg_glyph_orientation_vertical(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "svg-profile".into(),
            revision: 1,
        },
        CssomDeclarationSupport::default(),
    )
    .unwrap();
    let first = dom
        .inline_style(a, &ordinary, requests(), no_decisions, &cancel())
        .unwrap();
    let second = dom
        .inline_style(b, &svg, requests(), no_decisions, &cancel())
        .unwrap();
    assert_eq!(dom.snapshot().document(document).unwrap().stamp(), basis);
    let ca = dom.capture_inline_style(a).unwrap().unwrap();
    let cb = dom.capture_inline_style(b).unwrap().unwrap();
    let parser = |block: &CssomBlock| {
        block
            .inline_context()
            .unwrap()
            .context()
            .inputs
            .iter()
            .find(|i| i.version.role == CssomInputRole::ParserMode)
            .unwrap()
            .clone()
    };
    let pa = parser(ca.properties().snapshot().block(first.id()).unwrap());
    let pb = parser(cb.properties().snapshot().block(second.id()).unwrap());
    assert_ne!(pa.data, pb.data);
    assert_ne!(pa.version.identity, pb.version.identity);
    assert_eq!(pa.version.revision, pb.version.revision);
}

#[test]
fn whole_declaration_admission_runs_once_and_error_or_unwind_publishes_no_binding() {
    let (mut dom, document, _, element, _) = fixture();
    let i = DomInlineStyleInputs::new(
        dom.snapshot().document(document).unwrap().stamp(),
        CssParserContext::default(),
        CssomInputVersion {
            role: CssomInputRole::Support,
            identity: "whole-width".into(),
            revision: 1,
        },
        CssomDeclarationSupport::try_new(vec![(
            CssPropertyGrammar::from_name("width").unwrap(),
            CssomUsabilityRequirement::WholeDeclarationDecision,
        )])
        .unwrap(),
    )
    .unwrap();
    let before = dom.snapshot();
    let calls = Cell::new(0);
    assert!(
        dom.inline_style(
            element,
            &i,
            requests(),
            |candidates, context| {
                calls.set(calls.get() + 1);
                assert_eq!(candidates.len(), 1);
                assert!(
                    context
                        .inputs
                        .iter()
                        .any(|i| i.version.role == CssomInputRole::Document)
                );
                Err(CssomError::InvalidInput("caller refusal"))
            },
            &cancel()
        )
        .is_err()
    );
    assert_eq!(calls.get(), 1);
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        dom.inline_style(
            element,
            &i,
            requests(),
            |_, _| panic!("resolver unwind"),
            &cancel(),
        )
    }));
    assert!(panic.is_err());
    assert!(dom.capture_inline_style(element).unwrap().is_none());
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let calls = Cell::new(0);
    dom.inline_style(
        element,
        &i,
        requests(),
        |candidates, context| {
            calls.set(calls.get() + 1);
            let version = CssomInputVersion {
                role: CssomInputRole::Support,
                identity: "decision-owner".into(),
                revision: 4,
            };
            let decisions = candidates
                .iter()
                .map(|candidate| {
                    candidate
                        .decide(version.clone(), CssomDeclarationUsability::Usable)
                        .unwrap()
                })
                .collect();
            let mut current: Vec<_> = context.inputs.iter().map(|i| i.version.clone()).collect();
            current.push(version);
            Ok(CssomAdmission::new(decisions, current))
        },
        &cancel(),
    )
    .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        dom.capture_inline_style(element)
            .unwrap()
            .unwrap()
            .properties()
            .get_property_value("width", requests().declaration)
            .unwrap(),
        "3px"
    );
}

#[test]
fn identical_slot_coordinates_in_different_dom_owners_export_distinct_context_identities() {
    let (mut a, da, _, ea, _) = fixture();
    let (mut b, db, _, eb, _) = fixture();
    let pa = bind(&mut a, da, ea);
    let pb = bind(&mut b, db, eb);
    let ca = a.capture_inline_style(ea).unwrap().unwrap();
    let cb = b.capture_inline_style(eb).unwrap().unwrap();
    let inputs = |capture: &DomInlineStyleCapture, handle: &CssStyleProperties| {
        capture
            .properties()
            .snapshot()
            .block(handle.id())
            .unwrap()
            .inline_context()
            .unwrap()
            .context()
            .inputs
            .clone()
    };
    let ia = inputs(&ca, &pa);
    let ib = inputs(&cb, &pb);
    for role in [
        CssomInputRole::Owner,
        CssomInputRole::Document,
        CssomInputRole::ParserMode,
        CssomInputRole::Base,
    ] {
        let a = ia.iter().find(|i| i.version.role == role).unwrap();
        let b = ib.iter().find(|i| i.version.role == role).unwrap();
        assert_ne!(a.version.identity, b.version.identity);
    }
}

#[test]
fn same_parent_reorder_preserves_sparse_subtree_scope_refusal() {
    let (mut dom, document, root, first, second) = fixture();
    let mut b = DomBatch::new();
    let inside_second = b.create_element_ns(document, Namespace::html(), "span");
    let third = b.create_element_ns(document, Namespace::html(), "section");
    b.insert_before(&inside_second, second, None);
    b.insert_before(&third, root, None);
    let c = dom.apply(b, &cancel()).unwrap();
    let inside_second = c.created(&inside_second).unwrap().unwrap();
    let third = c.created(&third).unwrap().unwrap();
    let first_handle = bind(&mut dom, document, first);
    let second_handle = bind(&mut dom, document, inside_second);
    let third_handle = bind(&mut dom, document, third);
    let mut b = DomBatch::new();
    b.insert_before(second, root, Some(first.into()));
    let c = dom.apply(b, &cancel()).unwrap();
    assert!(c.changes().changes().contains(&DomChange::Children(root)));
    assert!(!c.changes().changes().contains(&DomChange::Parent(second)));
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.detach(second);
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::InlineStyleContextRequired {
            element: inside_second,
            change: DomInlineStyleChange::Context,
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    for (id, expected) in [
        (first, first_handle),
        (inside_second, second_handle),
        (third, third_handle),
    ] {
        assert_eq!(bind(&mut dom, document, id), expected);
    }
}

#[test]
fn html_attribute_case_folding_reaches_the_barrier_but_svg_uppercase_stays_unrelated() {
    let (mut dom, associated, _, _, _) = fixture();
    let mut b = DomBatch::new();
    let document = b.create_html_document(associated, None);
    let html = b.create_element_ns(&document, Namespace::html(), "div");
    let svg = b.create_element_ns(&document, Namespace::svg(), "rect");
    b.set_attribute(&html, "style", "width: 3px;");
    let c = dom.apply(b, &cancel()).unwrap();
    let document = c.document(&document).unwrap();
    let html = c.created(&html).unwrap().unwrap();
    let svg = c.created(&svg).unwrap().unwrap();
    bind(&mut dom, document, html);
    let svg_handle = bind(&mut dom, document, svg);
    let before = dom.snapshot();
    let mut b = DomBatch::new();
    b.set_attribute(html, "STYLE", "width: 3px;");
    assert_eq!(
        dom.apply(b, &cancel()).unwrap_err(),
        DomError::InlineStyleContextRequired {
            element: html,
            change: DomInlineStyleChange::Attribute
        }
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let mut b = DomBatch::new();
    b.set_attribute(svg, "STYLE", "width: 9px;");
    dom.apply(b, &cancel()).unwrap();
    let captured = dom.capture_inline_style(svg).unwrap().unwrap();
    assert_eq!(captured.properties().handle(), &svg_handle);
    assert_eq!(
        captured
            .properties()
            .length(requests().declaration)
            .unwrap(),
        0
    );
}

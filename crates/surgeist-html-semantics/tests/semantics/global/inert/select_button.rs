//! HTML 877d6146487f #the-button-element (forms capture line 7399): a `button`
//! that is the first child which is an element of a `select` is inert. "Child"
//! is the DOM ordinary-tree child, so text and comments never precede the first
//! element child and flat-tree edges never decide it. The sentence marks the
//! button itself; unlike the `inert` attribute (host-facts capture line 284) it
//! names no flat-tree descendants.
//!
//! Connected nodes that the document flat tree excludes are no node's flat-tree
//! descendant, so an ancestor's `inert` attribute cannot make them inert
//! (host-facts capture line 284; CSS Shadow §4.1, selectors-execution capture
//! lines 393-417). Modal blocking still reaches every connected node outside
//! the blocking subject's flat tree (host-facts capture line 256).
use super::*;

type Subject = HtmlCaptureBuilderSubject;

/// Unblocked document in which every `(node, parent)` edge is a flat edge and
/// `html` is the flat root.
fn flat_context(
    c: &HtmlCapture,
    d: &Subject,
    html: &Subject,
    edges: &[(&Subject, &Subject)],
) -> HtmlSemanticContext {
    let mut nodes = vec![root(c, html)];
    nodes.extend(
        edges
            .iter()
            .map(|(node, flat_parent)| parent(c, node, flat_parent)),
    );
    context(
        c,
        Some(supplement(c, key(), nodes, vec![modal(c, d, None)])),
    )
}

fn text(b: &mut HtmlCaptureBuilder, d: &Subject) -> Subject {
    b.add_text(d, "text".into()).unwrap()
}

fn comment(b: &mut HtmlCaptureBuilder, d: &Subject) -> Subject {
    b.add_comment(d, "comment".into()).unwrap()
}

#[test]
fn first_element_child_button_of_a_select_is_inert() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    let option = element(&mut b, &d, "option", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[button.clone(), option.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[(&select, &html), (&button, &select), (&option, &select)],
    );
    let report = query(&c, &cx, &[&button, &option, &select]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 2), &HtmlSemanticValue::Known(false));
}

#[test]
fn only_the_first_of_several_buttons_in_a_select_is_inert() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let first = element(&mut b, &d, "button", &[]);
    let second = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[first.clone(), second.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[(&select, &html), (&first, &select), (&second, &select)],
    );
    let report = query(&c, &cx, &[&first, &second]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
}

#[test]
fn a_button_after_an_element_sibling_is_not_the_first_element_child() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let option = element(&mut b, &d, "option", &[]);
    let later = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[option.clone(), later.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[(&select, &html), (&option, &select), (&later, &select)],
    );
    assert_eq!(
        value(&query(&c, &cx, &[&later]), 0),
        &HtmlSemanticValue::Known(false)
    );
}

#[test]
fn text_and_comment_nodes_before_a_button_do_not_precede_the_first_element_child() {
    // The sentence says "first child which is an element": earlier text and
    // comment children are skipped, so this button is still that child.
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let leading_text = text(&mut b, &d);
    let leading_comment = comment(&mut b, &d);
    let button = element(&mut b, &d, "button", &[]);
    let option = element(&mut b, &d, "option", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(
        &select,
        &[
            leading_text.clone(),
            leading_comment.clone(),
            button.clone(),
            option.clone(),
        ],
    )
    .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[(&select, &html), (&button, &select), (&option, &select)],
    );
    let report = query(&c, &cx, &[&button, &option]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
}

#[test]
fn a_button_whose_parent_is_not_a_select_is_not_inert() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let div = element(&mut b, &d, "div", &[]);
    let in_div = element(&mut b, &d, "button", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let optgroup = element(&mut b, &d, "optgroup", &[]);
    let in_optgroup = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[div.clone(), select.clone()])
        .unwrap();
    b.establish_children(&div, std::slice::from_ref(&in_div))
        .unwrap();
    b.establish_children(&select, std::slice::from_ref(&optgroup))
        .unwrap();
    b.establish_children(&optgroup, std::slice::from_ref(&in_optgroup))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[
            (&div, &html),
            (&in_div, &div),
            (&select, &html),
            (&optgroup, &select),
            (&in_optgroup, &optgroup),
        ],
    );
    let report = query(&c, &cx, &[&in_div, &in_optgroup]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
}

#[test]
fn only_html_buttons_and_html_selects_take_part_in_the_rule() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    // An HTML button inside a select element of another namespace.
    let svg_select = expanded(&mut b, &d, Namespace::svg(), "select", &[]);
    let in_svg_select = element(&mut b, &d, "button", &[]);
    // A button element of another namespace inside an HTML select.
    let select = element(&mut b, &d, "select", &[]);
    let svg_button = expanded(&mut b, &d, Namespace::svg(), "button", &[]);
    // A non-HTML element is the first element child, so the HTML button is later.
    let other_select = element(&mut b, &d, "select", &[]);
    let svg_first = expanded(&mut b, &d, Namespace::svg(), "g", &[]);
    let after_svg = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(
        &html,
        &[svg_select.clone(), select.clone(), other_select.clone()],
    )
    .unwrap();
    b.establish_children(&svg_select, std::slice::from_ref(&in_svg_select))
        .unwrap();
    b.establish_children(&select, std::slice::from_ref(&svg_button))
        .unwrap();
    b.establish_children(&other_select, &[svg_first.clone(), after_svg.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[
            (&svg_select, &html),
            (&in_svg_select, &svg_select),
            (&select, &html),
            (&svg_button, &select),
            (&other_select, &html),
            (&svg_first, &other_select),
            (&after_svg, &other_select),
        ],
    );
    let report = query(&c, &cx, &[&in_svg_select, &svg_button, &after_svg]);
    for index in 0..3 {
        assert_eq!(value(&report, index), &HtmlSemanticValue::Known(false));
    }
}

#[test]
fn light_children_decide_the_rule_by_ordinary_parent_not_flat_parent() {
    // The select is a shadow host. Its light button is the first element child
    // of the select in the ordinary tree even though the flat tree places it
    // under the slot. The button in the shadow root has the shadow root as its
    // ordinary parent even though the flat tree places it under the select.
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let light = element(&mut b, &d, "button", &[]);
    let shadow = b
        .add_shadow_root(
            &d,
            &select,
            HtmlCapturedShadowProperties::new(
                HtmlShadowRootProperties::new(
                    HtmlShadowRootMode::Closed,
                    HtmlSlotAssignment::Named,
                    false,
                    false,
                    false,
                ),
                false,
                false,
                false,
            ),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let in_shadow = element(&mut b, &d, "button", &[]);
    let slot = element(&mut b, &d, "slot", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, std::slice::from_ref(&light))
        .unwrap();
    b.establish_children(&shadow, &[in_shadow.clone(), slot.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[
            (&select, &html),
            (&in_shadow, &select),
            (&slot, &select),
            (&light, &slot),
        ],
    );
    let report = query(&c, &cx, &[&light, &in_shadow]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
}

#[test]
fn the_rule_marks_the_button_and_not_its_descendants() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    let inside = element(&mut b, &d, "span", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, std::slice::from_ref(&button))
        .unwrap();
    b.establish_children(&button, std::slice::from_ref(&inside))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[(&select, &html), (&button, &select), (&inside, &button)],
    );
    let report = query(&c, &cx, &[&button, &inside]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
}

#[test]
fn a_select_button_with_unavailable_ordered_children_is_missing_evidence() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_parent(&button, &select).unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(&c, &d, &html, &[(&select, &html), (&button, &select)]);
    assert_eq!(
        value(&query(&c, &cx, &[&button]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::OrderedChildren(
            c.subject_id(&select).unwrap()
        ))
    );
}

#[test]
fn a_select_button_is_inert_with_no_flat_tree_or_modal_supplement() {
    let (mut b, d) = builder();
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_children(&select, std::slice::from_ref(&button))
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(&c, None);
    assert_eq!(
        value(&query(&c, &cx, &[&button]), 0),
        &HtmlSemanticValue::Known(true)
    );
}

#[test]
fn connected_node_outside_the_flat_tree_is_not_made_inert_by_an_ancestor_attribute() {
    // `light` is connected, a light child of the inert host, and excluded from
    // the flat tree (unassigned): it is no node's flat-tree descendant.
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let host = element(&mut b, &d, "div", &[("inert", "")]);
    let light = element(&mut b, &d, "span", &[]);
    let unknown_connection = element(&mut b, &d, "span", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&host))
        .unwrap();
    b.establish_children(&host, &[light.clone(), unknown_connection.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &host, &html),
                fact(
                    &c,
                    &light,
                    Some(true),
                    HtmlDocumentFlatTreeRelation::Excluded,
                ),
                fact(
                    &c,
                    &unknown_connection,
                    None,
                    HtmlDocumentFlatTreeRelation::Excluded,
                ),
            ],
            vec![modal(&c, &d, None)],
        )),
    );
    let report = query(&c, &cx, &[&host, &light, &unknown_connection]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(false));
    // Without a connectedness fact the exclusion may be detachment, for which
    // no flat tree is defined: the result stays missing evidence.
    assert_eq!(
        value(&report, 2),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::InertAncestry(
            c.subject_id(&unknown_connection).unwrap()
        ))
    );
}

#[test]
fn a_select_button_with_an_unavailable_parent_is_missing_evidence_naming_the_parent_link() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let button = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[]).unwrap();
    let c = b.finish().unwrap();
    // No flat record for the button either, so only the rule's own gap is named.
    let cx = flat_context(&c, &d, &html, &[]);
    assert_eq!(
        value(&query(&c, &cx, &[&button]), 0),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::Parent(c.subject_id(&button).unwrap()))
    );
}

#[test]
fn inherited_inertness_proves_true_where_the_button_rule_lacks_evidence() {
    // `lost_parent` has no ordinary parent link and `lost_children` sits in a
    // select whose ordered children are unavailable, so the rule alone is
    // missing evidence. Their flat ancestor carries the inert attribute.
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let inert = element(&mut b, &d, "div", &[("inert", "")]);
    let lost_parent = element(&mut b, &d, "button", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let lost_children = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&inert))
        .unwrap();
    b.establish_children(&inert, std::slice::from_ref(&select))
        .unwrap();
    b.establish_parent(&lost_children, &select).unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[
            (&inert, &html),
            (&lost_parent, &inert),
            (&select, &inert),
            (&lost_children, &select),
        ],
    );
    let report = query(&c, &cx, &[&lost_parent, &lost_children]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
}

#[test]
fn a_select_button_stays_inert_inside_a_blocking_dialog_that_escapes_ancestor_inertness() {
    // The blocking dialog and its flat descendants escape modal inertness, but
    // the button rule is a separate source that still applies to the button.
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let outside = element(&mut b, &d, "div", &[]);
    let dialog = element(&mut b, &d, "dialog", &[("open", "")]);
    let select = element(&mut b, &d, "select", &[]);
    let first = element(&mut b, &d, "button", &[]);
    let option = element(&mut b, &d, "option", &[]);
    let later = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, &[outside.clone(), dialog.clone()])
        .unwrap();
    b.establish_children(&dialog, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[first.clone(), option.clone(), later.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &outside, &html),
                parent(&c, &dialog, &html),
                parent(&c, &select, &dialog),
                parent(&c, &first, &select),
                parent(&c, &option, &select),
                parent(&c, &later, &select),
            ],
            vec![modal(&c, &d, Some(&dialog))],
        )),
    );
    let report = query(&c, &cx, &[&outside, &first, &option, &later]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 2), &HtmlSemanticValue::Known(false));
    assert_eq!(value(&report, 3), &HtmlSemanticValue::Known(false));
}

#[test]
fn unavailable_modal_blocking_leaves_a_later_button_missing_but_not_the_select_button() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let first = element(&mut b, &d, "button", &[]);
    let later = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[first.clone(), later.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = context(
        &c,
        Some(supplement(
            &c,
            key(),
            vec![
                root(&c, &html),
                parent(&c, &select, &html),
                parent(&c, &first, &select),
                parent(&c, &later, &select),
            ],
            vec![HtmlDocumentModalFact::new(
                c.subject_id(&d).unwrap(),
                HtmlDocumentModalBlocking::Unavailable,
            )],
        )),
    );
    let report = query(&c, &cx, &[&first, &later]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(
        value(&report, 1),
        &HtmlSemanticValue::Missing(HtmlSemanticMissing::ModalBlocking(
            c.subject_id(&d).unwrap()
        ))
    );
}

#[test]
fn a_select_button_combines_with_an_inert_ancestor_and_a_later_button_still_inherits_it() {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let inert = element(&mut b, &d, "div", &[("inert", "")]);
    let select = element(&mut b, &d, "select", &[]);
    let first = element(&mut b, &d, "button", &[]);
    let later = element(&mut b, &d, "button", &[]);
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&inert))
        .unwrap();
    b.establish_children(&inert, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &[first.clone(), later.clone()])
        .unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(
        &c,
        &d,
        &html,
        &[
            (&inert, &html),
            (&select, &inert),
            (&first, &select),
            (&later, &select),
        ],
    );
    let report = query(&c, &cx, &[&first, &later]);
    assert_eq!(value(&report, 0), &HtmlSemanticValue::Known(true));
    assert_eq!(value(&report, 1), &HtmlSemanticValue::Known(true));
}

/// A select holding `leading` text children before its button, in an unblocked
/// document. Returns the capture, context and the button's request.
fn text_before_button(leading: usize) -> (HtmlCapture, HtmlSemanticContext, HtmlSemanticRequest) {
    let (mut b, d) = builder();
    let html = element(&mut b, &d, "html", &[]);
    let select = element(&mut b, &d, "select", &[]);
    let button = element(&mut b, &d, "button", &[]);
    let mut children = (0..leading).map(|_| text(&mut b, &d)).collect::<Vec<_>>();
    children.push(button.clone());
    b.establish_parent(&html, &d).unwrap();
    b.establish_children(&html, std::slice::from_ref(&select))
        .unwrap();
    b.establish_children(&select, &children).unwrap();
    let c = b.finish().unwrap();
    let cx = flat_context(&c, &d, &html, &[(&select, &html), (&button, &select)]);
    let request =
        HtmlSemanticRequest::new(c.subject_id(&button).unwrap(), HtmlSemanticQuery::Inertness);
    (c, cx, request)
}

#[test]
fn scanning_leading_non_element_children_is_charged_and_bounded_by_the_work_limit() {
    let work = |leading| {
        let (c, cx, request) = text_before_button(leading);
        let report = HtmlSemanticSession::new(
            &c,
            &cx,
            HtmlValueInterpreter::new(10000),
            limits(),
            HtmlSemanticCancellation::new(),
        )
        .unwrap()
        .query(&[request])
        .unwrap();
        assert_eq!(
            value(&report, 0),
            &HtmlSemanticValue::Known(true),
            "{leading} leading children"
        );
        report.usage().work()
    };
    // Each scanned child costs one step to advance and one to read its node.
    assert_eq!(work(40) - work(10), 2 * 30);

    let (c, cx, request) = text_before_button(40);
    let report = assert_exact_and_one_short_caps(
        "button after leading text",
        &c,
        &cx,
        10000,
        limits(),
        std::slice::from_ref(&request),
        ExpectedUsage {
            work: Charge::AtLeast(2 * 40),
            output_records: Charge::Exactly(1),
            allocated_bytes: Charge::Exactly(std::mem::size_of::<HtmlSemanticResult>()),
            ..ExpectedUsage::none()
        },
    );
    let too_small =
        HtmlSemanticLimits::new(report.usage().work() - 1, 100, 1000, 1000, 1000000, 1000);
    let failure = HtmlSemanticSession::new(
        &c,
        &cx,
        HtmlValueInterpreter::new(10000),
        too_small,
        HtmlSemanticCancellation::new(),
    )
    .unwrap()
    .query(&[request])
    .unwrap_err();
    assert!(matches!(
        failure.kind(),
        HtmlSemanticFailureKind::LimitExceeded { resource, .. } if resource == &HtmlSemanticResource::Work
    ));
}

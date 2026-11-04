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

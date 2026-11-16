use crate::parser::support::{fragment_context, limits};
use surgeist_html::{
    construction::HtmlNodeKind, elements::native::HtmlCommonScope, parsing::HtmlParserScriptingMode,
};
use surgeist_html_parser::*;

#[test]
fn completed_null_registry_fragment_lends_actual_closed_common_state() {
    // HTML877d element creation sets the common tuple; this original inert
    // parser executes no script or interaction that can transition that tuple.
    // SVG has the element tuple only; HTML has both. No native initializer is
    // a witness for either source-created subject.
    let report = HtmlFragmentParseRequest::with_options(
        HtmlSource::utf8("<p nonce=retained>x</p><svg><g></g></svg>"),
        fragment_context(),
        limits(),
        HtmlParserScriptingMode::Inert,
        false,
    )
    .unwrap()
    .parse()
    .unwrap();
    assert!(report.document_state().is_none());
    let mut found = [false; 2];
    // The report answers by node: these elements have the creation state of their namespace, so
    // no metadata record holds it.
    let fragment = report.fragment().unwrap();
    let mut pending = vec![fragment.root().id()];
    while let Some(id) = pending.pop() {
        let node = fragment.node(&id).unwrap();
        pending.extend(node.children());
        let HtmlNodeKind::Element(element) = node.kind() else {
            assert!(report.creation(&id).unwrap().value().is_none());
            assert!(report.common_state(&id).unwrap().value().is_none());
            continue;
        };
        assert!(report.creation(&id).unwrap().value().is_some());
        let local = element.name().local_name().code_units();
        let (index, scope) = if local == [112] {
            (0, HtmlCommonScope::Html)
        } else if local == [103] {
            (1, HtmlCommonScope::Element)
        } else {
            continue;
        };
        let common = report
            .common_state(&id)
            .unwrap()
            .value()
            .expect("actual qualified contextual common tuple");
        assert_eq!(common.scope(), scope);
        assert!(common.element().reaction_queue_is_empty());
        assert!(!common.element().click_in_progress());
        assert_eq!(common.html().is_some(), index == 0);
        if let Some(html) = common.html() {
            assert!(!html.has_previously_focused_element());
            assert!(!html.has_attached_internals());
            assert!(html.popover_is_hidden());
            assert!(!html.has_popover_trigger());
            assert!(!html.popover_is_hiding());
            assert!(!html.has_popover_toggle_task_tracker());
            assert!(!html.has_popover_close_watcher());
        }
        found[index] = true;
    }
    assert_eq!(found, [true, true]);
}

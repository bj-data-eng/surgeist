//! The nonce a published report answers by node, for the elements that hold a row for it and the
//! contexts that hold one: a nonce on an ordinary element, on `html` and `body` through the
//! merge of a later token, and on a foreign element, a template's contents, a fragment parse, a
//! shadow root and a clone.
//!
//! The oracle is the pinned nonce algorithms: the nonce slot of an element that implements the
//! nonce mixin holds the value of its `nonce` content attribute when one is installed, and the
//! empty string otherwise.
use super::creation_state::{document_report, elements, fragment_report, template_contents};
use super::*;
use crate::HtmlParseNodeReadError;
use surgeist_html::construction::HtmlNodeId;

/// The nonce a report answers for `id`, as text, or `None` for a node with no nonce slot.
fn nonce(report: &crate::HtmlParseReport, id: &HtmlNodeId) -> Option<String> {
    report
        .cryptographic_nonce(id)
        .unwrap()
        .value()
        .map(|nonce| String::from_utf16(nonce.code_units()).unwrap())
}

fn rows(report: &crate::HtmlParseReport) -> usize {
    report.history().storage().payloads.facts.nonces.len()
}

/// The nonce of an element, whether a row holds it or its namespace gives the empty nonce.
fn held(value: &str) -> Option<String> {
    Some(value.to_string())
}

/// The empty nonce of an element with no row.
fn empty() -> Option<String> {
    Some(String::new())
}

#[test]
fn an_ordinary_element_holds_a_row_only_for_a_nonce_attribute() {
    let report = document_report(
        "<!doctype html><div nonce=a></div><span></span><p nonce=></p><b nonce=\"\"></b>",
    );
    assert_eq!(nonce(&report, &elements(&report, "div")[0]), held("a"));
    assert_eq!(nonce(&report, &elements(&report, "span")[0]), empty());
    // A nonce attribute with no value, or an empty one, installs the empty string in a row.
    assert_eq!(nonce(&report, &elements(&report, "p")[0]), held(""));
    assert_eq!(nonce(&report, &elements(&report, "b")[0]), held(""));
    // The three rows, and those of `html` and `body`.
    assert_eq!(rows(&report), 3 + 2);
}

#[test]
fn html_and_body_hold_rows_that_a_later_token_updates_in_place() {
    let report = document_report("<!doctype html><html><body><html nonce=m><body nonce=n><p>");
    assert_eq!(nonce(&report, &elements(&report, "html")[0]), held("m"));
    assert_eq!(nonce(&report, &elements(&report, "body")[0]), held("n"));
    assert_eq!(nonce(&report, &elements(&report, "p")[0]), empty());
    // The merge writes the rows the creation reserved: no row was added.
    assert_eq!(rows(&report), 2);
    // A merged attribute does not replace one the element has.
    let report = document_report("<!doctype html><html nonce=x><body nonce=y><html nonce=q>");
    assert_eq!(nonce(&report, &elements(&report, "html")[0]), held("x"));
    assert_eq!(nonce(&report, &elements(&report, "body")[0]), held("y"));
    assert_eq!(rows(&report), 2);
    // Neither has an attribute: the rows hold the empty nonce.
    let report = document_report("<!doctype html><p>");
    assert_eq!(nonce(&report, &elements(&report, "html")[0]), held(""));
    assert_eq!(nonce(&report, &elements(&report, "body")[0]), held(""));
    assert_eq!(rows(&report), 2);
}

#[test]
fn a_foreign_element_holds_a_row_for_its_nonce_and_an_unrelated_attribute_holds_none() {
    let report = document_report(
        "<!doctype html><svg nonce=s xlink:nonce=x><g></g></svg><math nonce=u><mi></mi></math>",
    );
    assert_eq!(nonce(&report, &elements(&report, "svg")[0]), held("s"));
    assert_eq!(nonce(&report, &elements(&report, "g")[0]), empty());
    assert_eq!(nonce(&report, &elements(&report, "math")[0]), held("u"));
    assert_eq!(nonce(&report, &elements(&report, "mi")[0]), empty());
    // An attribute of another namespace named `nonce` is not the nonce attribute.
    let report = document_report("<!doctype html><svg xlink:nonce=x></svg>");
    assert_eq!(nonce(&report, &elements(&report, "svg")[0]), empty());
}

#[test]
fn the_elements_of_a_templates_contents_are_answered_like_any_other() {
    let report =
        document_report("<!doctype html><template nonce=t><i nonce=a></i><b></b></template>");
    assert_eq!(nonce(&report, &elements(&report, "template")[0]), held("t"));
    assert_eq!(nonce(&report, &elements(&report, "i")[0]), held("a"));
    assert_eq!(nonce(&report, &elements(&report, "b")[0]), empty());
    // The contents of the template are a fragment, which is not an element.
    assert_eq!(nonce(&report, &template_contents(&report)), None);
}

#[test]
fn a_fragment_parse_answers_by_node_and_its_implicit_root_holds_a_row() {
    let report = fragment_report("<p nonce=a></p><html nonce=r><i></i>", false, false);
    assert_eq!(nonce(&report, &elements(&report, "p")[0]), held("a"));
    assert_eq!(nonce(&report, &elements(&report, "i")[0]), empty());
    // The implicit `html` root of the parse is merged into by a later `html` token.
    assert_eq!(nonce(&report, &elements(&report, "html")[0]), held("r"));
    assert_eq!(rows(&report), 2);
}

#[test]
fn the_contents_of_a_shadow_root_are_a_fragment_and_their_elements_are_answered() {
    let report = fragment_report(
        "<template shadowrootmode=open><x-a nonce=s></x-a><b></b></template>",
        false,
        true,
    );
    assert_eq!(nonce(&report, &elements(&report, "x-a")[0]), held("s"));
    assert_eq!(nonce(&report, &elements(&report, "b")[0]), empty());
    let effects = report.effects().unwrap().unwrap();
    let mut contents = None;
    for index in 0..effects.len() {
        if let Some(crate::HtmlParseEffect::AttachShadow {
            contents: found, ..
        }) = effects.effect(index).unwrap()
        {
            contents = Some(found.id());
        }
    }
    let contents = contents.expect("the template attaches a shadow root");
    assert_eq!(nonce(&report, &contents), None);
}

#[test]
fn a_clone_holds_a_row_exactly_when_its_source_does() {
    // The select clones the content of its selected option into the selectedcontent element.
    let report = document_report(
        "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
         <option selected><span nonce=c>t</span><i></i></option></select>",
    );
    let spans = elements(&report, "span");
    let plain = elements(&report, "i");
    assert_eq!(spans.len(), 2);
    assert_eq!(plain.len(), 2);
    for span in &spans {
        assert_eq!(nonce(&report, span), held("c"));
    }
    for i in &plain {
        assert_eq!(nonce(&report, i), empty());
    }
    // `html`, `body`, the original and its clone.
    assert_eq!(rows(&report), 2 + 2);
}

#[test]
fn a_node_that_is_not_an_element_has_no_nonce_and_a_node_of_another_result_is_foreign() {
    let one = document_report("<!doctype html><!--c-->text<p nonce=a>");
    let other = document_report("<!doctype html><p nonce=a>");
    let document = one.document().unwrap().root().id();
    assert_eq!(nonce(&one, &document), None);
    let arena = &one.history().storage().view;
    for position in 0..arena.len() {
        let record = arena.node_at(position).unwrap();
        if !matches!(record.kind(), ConstructionNodeKind::Element(_)) {
            let id = record.as_node().unwrap().id();
            assert_eq!(nonce(&one, &id), None, "node {position}");
        }
    }
    let foreign = elements(&other, "p")[0];
    assert_eq!(
        one.cryptographic_nonce(&foreign).err(),
        Some(HtmlParseNodeReadError::Foreign)
    );
}

#[test]
fn a_row_the_report_cannot_resolve_is_an_unresolved_reference() {
    let mut report = document_report("<!doctype html><p nonce=a>");
    let p = elements(&report, "p")[0];
    assert_eq!(nonce(&report, &p), held("a"));
    let position = report
        .history()
        .storage()
        .view
        .node_by_result_id(&p)
        .unwrap()
        .position();
    let storage = report.storage_mut().history_mut().storage_mut();
    let index = storage.payloads.facts.nonces.find_published(position).0;
    // The row of the `p` names a string past the registry.
    let past = storage.payloads.values.len();
    let row = storage
        .payloads
        .facts
        .nonces
        .row_mut(index.expect("the nonce row of the p"))
        .unwrap();
    row.raw = past;
    assert!(matches!(
        report.cryptographic_nonce(&p).err(),
        Some(HtmlParseNodeReadError::Unresolved(_))
    ));
}

#[test]
fn an_empty_row_and_a_plain_element_answer_the_same_nonce() {
    let report = document_report("<!doctype html><b nonce=\"\"></b><span></span>");
    let row = elements(&report, "b")[0];
    let plain = elements(&report, "span")[0];
    let with_row = report.cryptographic_nonce(&row).unwrap();
    let without = report.cryptographic_nonce(&plain).unwrap();
    assert_eq!(with_row.value(), without.value());
    assert_eq!(
        *without.value(),
        Some(&surgeist_html::values::DocumentString::default())
    );
    assert_eq!(rows(&report), 1 + 2);
}

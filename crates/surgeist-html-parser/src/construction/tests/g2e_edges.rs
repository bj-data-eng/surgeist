//! The template metadata and shadow state a report answers at the edges of their creation: a
//! template inside a template, a template cloned by a `selectedcontent` element, a second
//! declarative shadow root, and a fragment whose context is the shadow host.
use super::g2e_corpus::{fragment_report, records};
use super::*;
use surgeist_html::construction::HtmlNodeId;

fn document(source: &str, detail: crate::DetailSet) -> crate::HtmlParseReport {
    crate::HtmlDocumentParseRequest::offline(
        crate::HtmlSource::utf8(source),
        g2_d2::corpus_limits(),
    )
    .with_detail(detail)
    .unwrap()
    .parse()
    .unwrap()
}

/// The elements of a report named `local`, as ids with the ids of their template contents.
fn elements(report: &crate::HtmlParseReport, local: &str) -> Vec<(HtmlNodeId, Option<HtmlNodeId>)> {
    let arena = &report.history().storage().view;
    let mut found = Vec::new();
    for position in 0..arena.len() {
        let record = arena.node_at(position).unwrap();
        let surgeist_html::construction::ConstructionNodeKind::Element(element) = record.kind()
        else {
            continue;
        };
        if String::from_utf16_lossy(element.name().local_name().code_units()) != local {
            continue;
        }
        let contents = record.template_contents().map(|contents| {
            arena
                .node_at(contents.position())
                .unwrap()
                .as_node()
                .unwrap()
                .id()
        });
        found.push((record.as_node().unwrap().id(), contents));
    }
    found
}

/// A template inside the contents of a template has the metadata of its own contents, and not those
/// of the template that holds it.
#[test]
fn a_template_inside_a_template_has_its_own_metadata() {
    for detail in g2_d2::details() {
        let report = document(
            "<!doctype html><template><div><template><i></i></template></div></template>",
            detail,
        );
        let found = records(&report);
        let templates = elements(&report, "template");
        assert_eq!(templates.len(), 2, "{detail:?}");
        let (outer, inner) = (&templates[0], &templates[1]);
        assert_ne!(outer.1, inner.1);
        for (id, contents) in [outer, inner] {
            let [answer] = found.templates[id].as_slice() else {
                panic!("one record for each template, {detail:?}");
            };
            assert_eq!(Some(answer.contents), *contents, "{detail:?}");
            assert!(!answer.markers);
        }
    }
}

/// A template cloned into a `selectedcontent` element is a template of its own: its record names
/// the contents of the clone, which hold a copy of what the original's hold.
#[test]
fn a_template_cloned_by_selectedcontent_has_metadata_for_the_clone() {
    for detail in g2_d2::details() {
        let report = document(
            "<!doctype html><select><button><selectedcontent></selectedcontent></button>\
             <option selected><template><b>x</b></template></option></select>",
            detail,
        );
        let found = records(&report);
        let templates = elements(&report, "template");
        assert_eq!(templates.len(), 2, "{detail:?}");
        assert_ne!(templates[0].1, templates[1].1);
        for (id, contents) in &templates {
            let [answer] = found.templates[id].as_slice() else {
                panic!("one record for each template, {detail:?}");
            };
            assert_eq!(Some(answer.contents), *contents, "{detail:?}");
        }
        // The original's contents and the clone's each hold one `b`.
        assert_eq!(elements(&report, "b").len(), 2, "{detail:?}");
    }
}

/// A host that has a shadow root refuses a second declarative one: the second template stays a
/// template, and the report answers one shadow state for the host.
#[test]
fn a_second_declarative_shadow_root_is_refused_and_has_no_state() {
    for detail in g2_d2::details() {
        for source in [
            "<div><template shadowrootmode=open><p>a</p></template>\
             <template shadowrootmode=open><p>b</p></template></div>",
            "<template shadowrootmode=open><p>a</p></template>\
             <template shadowrootmode=closed><p>b</p></template>",
        ] {
            let report = fragment_report(source, "div", true, detail);
            let found = records(&report);
            let shadows: usize = found.shadows.values().map(Vec::len).sum();
            let templates: usize = found.templates.values().map(Vec::len).sum();
            assert_eq!(shadows, 1, "{source} {detail:?}");
            assert_eq!(templates, 2, "{source} {detail:?}");
            // The state is that of a root the parser attached.
            for states in found.shadows.values() {
                assert!(states[0].1 && states[0].2);
            }
        }
    }
}

/// A fragment whose context element is the shadow host: a template at the top of the fragment
/// attaches the root to the captured element, whose contents have a state and no host in the tree.
#[test]
fn a_fragment_whose_context_is_the_shadow_host_answers_the_state_of_the_root() {
    for detail in g2_d2::details() {
        let report = fragment_report(
            "<template shadowrootmode=open shadowrootclonable><p>a</p></template>",
            "div",
            true,
            detail,
        );
        let found = records(&report);
        let arena = &report.history().storage().view;
        let roots: Vec<_> = (0..arena.len())
            .filter_map(|position| arena.node_at(position))
            .filter(|record| record.shadow_properties().is_some())
            .map(|record| record.as_node().unwrap().id())
            .collect();
        assert_eq!(roots.len(), 1, "{detail:?}");
        let [state] = found.shadows[&roots[0]].as_slice() else {
            panic!("one state for the root, {detail:?}");
        };
        let (null, declarative, available) = *state;
        assert!(declarative && available, "{detail:?}");
        assert!(!null, "the host's registry is not kept null");
    }
}

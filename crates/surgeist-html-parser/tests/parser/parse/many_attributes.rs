//! Attribute lists around the length at which the parser starts to look names up in a table
//! (32) keep every name and value, in source order, on every path that installs them.

use crate::parser::support::limits;
use surgeist_html::construction::HtmlNodeKind;
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};

type Attributes = Vec<(String, String)>;

/// Every element named `name` of the parsed document, in document order, with its attributes.
fn elements(raw: &str, name: &str) -> Vec<Attributes> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses: {raw:.80}"));
    let document = report.document().unwrap();
    let mut found = Vec::new();
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind()
            && element.name().local_name().to_utf8().unwrap() == name
        {
            found.push(
                element
                    .attributes()
                    .iter()
                    .map(|attribute| {
                        (
                            attribute.name().local_name().to_utf8().unwrap(),
                            attribute.value().to_utf8().unwrap(),
                        )
                    })
                    .collect(),
            );
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
    }
    found
}

fn source(prefix: &str, count: usize) -> String {
    (0..count)
        .map(|i| format!(" {prefix}{i}={prefix}v{i}"))
        .collect()
}

fn expected(prefix: &str, count: usize) -> Attributes {
    (0..count)
        .map(|i| (format!("{prefix}{i}"), format!("{prefix}v{i}")))
        .collect()
}

const COUNTS: [usize; 9] = [0, 1, 2, 31, 32, 33, 64, 65, 200];

#[test]
fn elements_keep_every_attribute_in_order_at_every_length_around_the_table_threshold() {
    for tag in ["div", "input", "details", "select"] {
        for count in COUNTS {
            let raw = format!("<!doctype html><{tag}{}>", source("a", count));
            assert_eq!(
                elements(&raw, tag),
                [expected("a", count)],
                "{tag} with {count} attributes"
            );
        }
    }
}

#[test]
fn a_repeated_name_keeps_its_first_value_on_a_long_list() {
    for tag in ["div", "input", "details", "select"] {
        for count in [31, 32, 33, 100] {
            let raw = format!(
                "<!doctype html><{tag}{} a3=again a{}=last>",
                source("a", count),
                count - 1
            );
            assert_eq!(
                elements(&raw, tag),
                [expected("a", count)],
                "{tag} with {count} attributes and two repeats"
            );
        }
    }
}

#[test]
fn the_attributes_of_a_later_html_start_tag_merge_after_those_the_element_has() {
    for existing in [0, 1, 31, 32, 33] {
        for merged in [1, 2, 31, 32, 33, 100] {
            // The merged tag repeats the existing names b0 and b1 with other values, and adds
            // names of its own, which are kept in their order.
            let raw = format!(
                "<!doctype html><html{}><body><html b0=other b1=other{}>",
                source("b", existing),
                source("c", merged)
            );
            let mut want = expected("b", existing);
            if existing < 2 {
                for i in existing..2 {
                    want.push((format!("b{i}"), "other".to_owned()));
                }
            }
            want.extend(expected("c", merged));
            assert_eq!(
                elements(&raw, "html"),
                [want],
                "{existing} existing, {merged} merged"
            );
        }
    }
}

#[test]
fn a_replayed_formatting_element_copies_every_attribute() {
    for count in COUNTS {
        // `</p>` closes the b it contains; the x that follows reconstructs it.
        let raw = format!("<!doctype html><p><b{}></p>x", source("a", count));
        assert_eq!(
            elements(&raw, "b"),
            [expected("a", count), expected("a", count)],
            "{count} attributes"
        );
    }
}

//! The recipe model is itself an oracle, so its serialization and expected
//! trees are checked here against handwritten values stated from the HTML
//! Standard, with no parser involved.
use super::model::*;
use super::tree::*;
use super::{Body, Case, Mode, seeds};
use surgeist_html::construction::HtmlDocumentMode;

fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn items(case: &Case) -> &[Item] {
    match &case.body {
        Body::Exact(items) => items,
        Body::Soup(_) => &[],
    }
}
fn element(uri: &'static str, local: &'static str, attrs: Vec<ExpAttr>, children: Vec<Exp>) -> Exp {
    Exp {
        kind: ExpKind::Element { uri, local, attrs },
        children,
        contents: None,
        shadow: None,
    }
}
fn text(s: &str) -> Exp {
    Exp {
        kind: ExpKind::Text(units(s)),
        children: vec![],
        contents: None,
        shadow: None,
    }
}
fn plain(local: &'static str, value: &str) -> ExpAttr {
    ExpAttr {
        uri: "",
        prefix: None,
        local,
        value: units(value),
    }
}

#[test]
fn document_seed_spells_references_newlines_quotes_and_the_shell() {
    let case = seeds::case(0, false).unwrap();
    let Mode::Document { doctype, shell } = case.mode else {
        panic!("document seed")
    };
    assert_eq!(
        serialize_document(doctype, shell, items(&case)),
        units(
            "<!DOCTYPE html><html><head></head><body>\
             <div id=\"a\" class='x y'>A&amp;B\r\nC<!--note--><br></div>\
             </body></html>"
        )
    );
}

#[test]
fn document_seed_expects_references_decoded_and_newlines_normalized() {
    // 13.2.5.78 `&amp;` is `&`; 13.2.3.5 CR LF is one LF; a missing doctype
    // identifier is the empty string; `<!DOCTYPE html>` is no-quirks.
    let case = seeds::case(0, false).unwrap();
    let div = element(
        HTML_NS,
        "div",
        vec![plain("id", "a"), plain("class", "x y")],
        vec![
            text("A&B\nC"),
            Exp {
                kind: ExpKind::Comment(units("note")),
                children: vec![],
                contents: None,
                shadow: None,
            },
            element(HTML_NS, "br", vec![], vec![]),
        ],
    );
    let expected = Exp {
        kind: ExpKind::Document(HtmlDocumentMode::NoQuirks),
        children: vec![
            Exp {
                kind: ExpKind::Doctype {
                    name: "html",
                    public: "",
                    system: "",
                },
                children: vec![],
                contents: None,
                shadow: None,
            },
            element(
                HTML_NS,
                "html",
                vec![],
                vec![
                    element(HTML_NS, "head", vec![], vec![]),
                    element(HTML_NS, "body", vec![], vec![div]),
                ],
            ),
        ],
        contents: None,
        shadow: None,
    };
    let Mode::Document { doctype, .. } = case.mode else {
        panic!("document seed")
    };
    assert_eq!(expect_document(doctype, items(&case)), expected);
}

#[test]
fn svg_seed_spells_mixed_case_and_expects_adjusted_names_and_qualified_attributes() {
    // 13.2.6.5 adjusts the tag name `foreignobject` to `foreignObject`; 13.2.6.1
    // adjusts the attribute `viewbox` to `viewBox` and gives `xlink:href` and
    // `xml:lang` their namespaces and prefixes; the foreignObject is an HTML
    // integration point.
    let case = seeds::case(3, false).unwrap();
    let Mode::Document { doctype, shell } = case.mode else {
        panic!("document seed")
    };
    assert_eq!(
        serialize_document(doctype, shell, items(&case)),
        units(
            "<!DOCTYPE html SYSTEM \"about:legacy-compat\"><html><head></head><body>\
             <SVG VIEWBOX=\"0 0 1 1\" xlink:href='x' xml:lang=\"en\">\
             <g><rect></rect><circle></circle></g>\
             <FoReIgNoBjEcT><div>h</div></FoReIgNoBjEcT></SVG></body></html>"
        )
    );
    let svg = element(
        SVG_NS,
        "svg",
        vec![
            plain("viewBox", "0 0 1 1"),
            ExpAttr {
                uri: "http://www.w3.org/1999/xlink",
                prefix: Some("xlink"),
                local: "href",
                value: units("x"),
            },
            ExpAttr {
                uri: "http://www.w3.org/XML/1998/namespace",
                prefix: Some("xml"),
                local: "lang",
                value: units("en"),
            },
        ],
        vec![
            element(
                SVG_NS,
                "g",
                vec![],
                vec![
                    element(SVG_NS, "rect", vec![], vec![]),
                    element(SVG_NS, "circle", vec![], vec![]),
                ],
            ),
            element(
                SVG_NS,
                "foreignObject",
                vec![],
                vec![element(HTML_NS, "div", vec![], vec![text("h")])],
            ),
        ],
    );
    let body = expect_document(doctype, items(&case));
    assert_eq!(body.children[1].children[1].children, vec![svg]);
}

#[test]
fn declarative_shadow_is_a_root_when_permitted_and_a_first_child_template_when_not() {
    // 13.2.6.4.4 (bits 0b10101: closed, named slots, delegates focus, serializable):
    // with permission the template's contents become the host's
    // shadow root; without it the template is inserted as an ordinary element
    // that keeps its attributes.
    let permitted = seeds::case(8, false).unwrap();
    let refused = seeds::case(9, false).unwrap();
    assert!(permitted.shadow_permitted() && !refused.shadow_permitted());
    let class = Class::Flow;
    let span = element(HTML_NS, "span", vec![], vec![text("s")]);
    let with_root = expect_fragment(items(&permitted), class, true);
    let host = &with_root.children[0];
    assert_eq!(host.children, vec![text("light")]);
    let (properties, contents) = host.shadow.as_ref().unwrap();
    assert_eq!(*properties, shadow_properties(0b10101));
    assert_eq!(contents, &vec![span.clone()]);

    let without = expect_fragment(items(&refused), class, false);
    let host = &without.children[0];
    assert!(host.shadow.is_none());
    let template = &host.children[0];
    assert_eq!(
        template.kind,
        ExpKind::Element {
            uri: HTML_NS,
            local: "template",
            attrs: vec![
                plain("shadowrootmode", "closed"),
                plain("shadowrootdelegatesfocus", ""),
                plain("shadowrootserializable", ""),
            ],
        }
    );
    assert_eq!(template.contents, Some(vec![span]));
    assert_eq!(host.children[1], text("light"));
}

#[test]
fn numeric_references_follow_the_end_state_table() {
    // 13.2.5.84: NUL, out of range and surrogate codes are U+FFFD; C1 codes map
    // through the table where it has an entry and are unchanged where it has none.
    for (code, value) in [
        (0, 0xFFFD),
        (0x11_0000, 0xFFFD),
        (0xD800, 0xFFFD),
        (0xDFFF, 0xFFFD),
        (0x80, 0x20AC),
        (0x81, 0x81),
        (0x9F, 0x0178),
        (0x0D, 0x0D),
        (0x41, 0x41),
        (0x10_FFFF, 0x10_FFFF),
    ] {
        assert_eq!(numeric_value(code), value, "&#{code};");
    }
}

#[test]
fn compatibility_modes_follow_the_prologue_rules() {
    // 13.2.6.4.1: a missing doctype, a name other than html, and the HTML 4.01
    // Transitional public identifier without a system identifier (or with an
    // empty one) are quirks; XHTML 1.0 Transitional and HTML 4.01 Transitional
    // with a system identifier are limited quirks; everything else is no-quirks.
    let modes: Vec<_> = (0..10).map(compatibility_mode).collect();
    use HtmlDocumentMode::{LimitedQuirks as L, NoQuirks as N, Quirks as Q};
    assert_eq!(modes, [Q, N, N, L, Q, N, N, Q, L, Q]);
}

#[test]
fn text_without_a_parsed_value_and_newline_fusion_are_not_valid_recipes() {
    let fragment = |items: Vec<Item>| {
        Case::new(
            Mode::Fragment {
                context: 0,
                shadow: false,
            },
            true,
            Body::Exact(items),
            crate::codec::CAPS,
            false,
            None,
        )
    };
    // A lone NUL is ignored in HTML text, so no text node would exist.
    assert!(fragment(vec![Item::Text(vec![Piece::Nul])]).is_err());
    // CR followed by a spelled LF would be one newline, not two.
    assert!(fragment(vec![Item::Text(vec![Piece::Cr, Piece::Unit(10)])]).is_err());
    // Two adjacent text items would merge into one node.
    assert!(
        fragment(vec![
            Item::Text(vec![Piece::Unit(97)]),
            Item::Text(vec![Piece::Unit(98)])
        ])
        .is_err()
    );
    // A paragraph may not hold a block, nor an anchor another anchor.
    let p = |tag| {
        Item::Elem(Elem {
            tag: P as u8,
            spell: 0,
            attrs: vec![],
            shadow: None,
            children: vec![Item::Elem(Elem {
                tag,
                spell: 0,
                attrs: vec![],
                shadow: None,
                children: vec![],
            })],
        })
    };
    assert!(fragment(vec![p(DIV as u8)]).is_err());
    assert!(fragment(vec![p(SPAN as u8)]).is_ok());
    let a_in_a = Item::Elem(Elem {
        tag: A as u8,
        spell: 0,
        attrs: vec![],
        shadow: None,
        children: vec![Item::Elem(Elem {
            tag: A as u8,
            spell: 0,
            attrs: vec![],
            shadow: None,
            children: vec![],
        })],
    });
    assert!(fragment(vec![a_in_a]).is_err());
}

fn parsed(source: &str) -> Vec<crate::observe::NativeNode> {
    let limits = crate::codec::limits(crate::codec::CAPS).unwrap();
    let report = surgeist_html_parser::HtmlDocumentParseRequest::offline(
        surgeist_html_parser::HtmlSource::utf8(source.to_owned()),
        limits,
    )
    .with_detail(surgeist_html_parser::DetailSet::ALL)
    .unwrap()
    .parse()
    .unwrap();
    crate::observe::native(&report).unwrap()
}

#[test]
fn the_tree_comparison_reports_each_kind_of_disagreement() {
    // `<p id=a>x</p>` after a no-quirks prologue and body start tag.
    let nodes = parsed("<!DOCTYPE html><body><p id=a>x</p>");
    let paragraph =
        |attr: &str, child: &str| element(HTML_NS, "p", vec![plain("id", attr)], vec![text(child)]);
    let document = |body: Vec<Exp>, mode: HtmlDocumentMode| Exp {
        kind: ExpKind::Document(mode),
        children: vec![
            Exp {
                kind: ExpKind::Doctype {
                    name: "html",
                    public: "",
                    system: "",
                },
                children: vec![],
                contents: None,
                shadow: None,
            },
            element(
                HTML_NS,
                "html",
                vec![],
                vec![
                    element(HTML_NS, "head", vec![], vec![]),
                    element(HTML_NS, "body", vec![], body),
                ],
            ),
        ],
        contents: None,
        shadow: None,
    };
    use super::compare::tree;
    let ok = document(vec![paragraph("a", "x")], HtmlDocumentMode::NoQuirks);
    assert_eq!(tree(&nodes, &ok), Ok(()));
    assert_eq!(
        tree(
            &nodes,
            &document(vec![paragraph("a", "y")], HtmlDocumentMode::NoQuirks)
        ),
        Err("tree-text")
    );
    assert_eq!(
        tree(
            &nodes,
            &document(vec![paragraph("b", "x")], HtmlDocumentMode::NoQuirks)
        ),
        Err("tree-attributes")
    );
    assert_eq!(
        tree(
            &nodes,
            &document(vec![paragraph("a", "x")], HtmlDocumentMode::Quirks)
        ),
        Err("tree-mode")
    );
    assert_eq!(
        tree(&nodes, &document(vec![], HtmlDocumentMode::NoQuirks)),
        Err("tree-children")
    );
    let div = element(HTML_NS, "div", vec![plain("id", "a")], vec![text("x")]);
    assert_eq!(
        tree(&nodes, &document(vec![div], HtmlDocumentMode::NoQuirks)),
        Err("tree-name")
    );
}

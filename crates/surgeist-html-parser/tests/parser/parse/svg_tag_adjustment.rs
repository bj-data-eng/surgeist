//! The SVG tag name adjustment of the tree builder's foreign content rules.
//!
//! HTML877d §13.2.6.5 lists, in its table, each lowercase tag name that becomes a
//! mixed-case SVG element name. The expectations below are that table.

use crate::parser::support::limits;
use surgeist_html::{
    construction::{HtmlNode, HtmlNodeKind},
    values::DocumentString,
};
use surgeist_html_parser::{HtmlDocumentParseRequest, HtmlSource};

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

const ADJUSTMENTS: [(&str, &str); 37] = [
    ("altglyph", "altGlyph"),
    ("altglyphdef", "altGlyphDef"),
    ("altglyphitem", "altGlyphItem"),
    ("animatecolor", "animateColor"),
    ("animatemotion", "animateMotion"),
    ("animatetransform", "animateTransform"),
    ("clippath", "clipPath"),
    ("feblend", "feBlend"),
    ("fecolormatrix", "feColorMatrix"),
    ("fecomponenttransfer", "feComponentTransfer"),
    ("fecomposite", "feComposite"),
    ("feconvolvematrix", "feConvolveMatrix"),
    ("fediffuselighting", "feDiffuseLighting"),
    ("fedisplacementmap", "feDisplacementMap"),
    ("fedistantlight", "feDistantLight"),
    ("fedropshadow", "feDropShadow"),
    ("feflood", "feFlood"),
    ("fefunca", "feFuncA"),
    ("fefuncb", "feFuncB"),
    ("fefuncg", "feFuncG"),
    ("fefuncr", "feFuncR"),
    ("fegaussianblur", "feGaussianBlur"),
    ("feimage", "feImage"),
    ("femerge", "feMerge"),
    ("femergenode", "feMergeNode"),
    ("femorphology", "feMorphology"),
    ("feoffset", "feOffset"),
    ("fepointlight", "fePointLight"),
    ("fespecularlighting", "feSpecularLighting"),
    ("fespotlight", "feSpotLight"),
    ("fetile", "feTile"),
    ("feturbulence", "feTurbulence"),
    ("foreignobject", "foreignObject"),
    ("glyphref", "glyphRef"),
    ("lineargradient", "linearGradient"),
    ("radialgradient", "radialGradient"),
    ("textpath", "textPath"),
];

/// The namespace URI and local name of every element of the parsed document, in
/// document order, after the implied `html`, `head` and `body` elements.
fn elements(raw: &str) -> Vec<(DocumentString, DocumentString)> {
    let report = HtmlDocumentParseRequest::offline(HtmlSource::utf8(raw.to_owned()), limits())
        .parse()
        .unwrap_or_else(|_| panic!("the document parses: {raw:.80}"));
    let document = report.document().unwrap();
    let mut found = Vec::new();
    let mut pending: Vec<_> = document.root().children().collect();
    pending.reverse();
    while let Some(id) = pending.pop() {
        let node: HtmlNode<'_> = document.node(&id).unwrap();
        if let HtmlNodeKind::Element(element) = node.kind() {
            found.push((
                element.name().namespace().uri().unwrap().clone(),
                element.name().local_name().clone(),
            ));
        }
        let mut children: Vec<_> = node.children().collect();
        children.reverse();
        pending.extend(children);
    }
    found
}

fn svg_child(raw: &str) -> (DocumentString, DocumentString) {
    let all = elements(raw);
    let svg = all
        .iter()
        .position(|(_, local)| *local == DocumentString::from("svg"))
        .unwrap_or_else(|| panic!("an svg element in {raw:.80}"));
    all.get(svg + 1)
        .unwrap_or_else(|| panic!("an element after svg in {raw:.80}"))
        .clone()
}

#[test]
fn each_lowercase_svg_tag_name_becomes_its_mixed_case_element_name() {
    for (lower, adjusted) in ADJUSTMENTS {
        let (namespace, local) =
            svg_child(&format!("<!doctype html><svg><{lower}></{lower}></svg>"));
        assert_eq!(namespace, DocumentString::from(SVG_NAMESPACE), "{lower}");
        assert_eq!(local, DocumentString::from(adjusted), "{lower}");
        // The tokenizer lowercases tag names, so a source spelling in any case
        // reaches the table as the lowercase name.
        let (_, local) = svg_child(&format!(
            "<!doctype html><svg><{}></svg>",
            lower.to_ascii_uppercase()
        ));
        assert_eq!(local, DocumentString::from(adjusted), "{lower}");
    }
}

#[test]
fn svg_names_outside_the_table_and_html_names_keep_their_lowercase_spelling() {
    for (lower, _) in ADJUSTMENTS {
        for unlisted in [format!("{lower}x"), format!("x{lower}")] {
            let (_, local) = svg_child(&format!("<!doctype html><svg><{unlisted}></svg>"));
            assert_eq!(local, DocumentString::from(unlisted.as_str()), "{unlisted}");
        }
    }
    // An HTML-namespace element is not adjusted, even where the name is listed.
    for (lower, _) in ADJUSTMENTS {
        let all = elements(&format!("<!doctype html><body><{lower}></{lower}>"));
        let found = all
            .iter()
            .find(|(_, local)| *local == DocumentString::from(lower))
            .unwrap_or_else(|| panic!("an element named {lower}"));
        assert_eq!(found.0, DocumentString::from(HTML_NAMESPACE), "{lower}");
    }
}

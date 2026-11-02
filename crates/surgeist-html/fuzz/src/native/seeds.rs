//! Literal construction recipes from HTML877d and public construction tests.
use super::*;
pub(super) const HTML: &[u16] = &[
    104, 116, 116, 112, 58, 47, 47, 119, 119, 119, 46, 119, 51, 46, 111, 114, 103, 47, 49, 57, 57,
    57, 47, 120, 104, 116, 109, 108,
];
pub(super) fn units(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
fn raw(uri: &str, prefix: Option<&str>, local: &str) -> Name {
    Name::Raw {
        uri: units(uri),
        prefix: prefix.map(units),
        local: units(local),
    }
}
fn attr(prefix: Option<&str>, local: &str, value: &str) -> Attr {
    Attr::Raw(raw("", prefix, local), units(value))
}
pub(super) fn case(id: u8) -> Result<NativeCase> {
    let mut root = 0;
    let ops = match id {
        0 => vec![
            Op::Text(0, units("before")),
            Op::Comment(0, units("<!-- raw -->")),
            Op::Element(
                0,
                Name::Known(1),
                vec![attr(None, "href", " relative &amp; ")],
            ),
            Op::Text(3, vec![0, 0xD83D, 0xDE00, 0xD800, 32, 0xDC00, 13, 10]),
            Op::Text(0, units("after")),
            Op::Element(0, Name::Known(0), vec![Attr::Hidden(0)]),
            Op::Element(0, Name::Known(3), vec![Attr::Hidden(1)]),
        ],
        1 => [
            "http://www.w3.org/1999/xhtml",
            "http://www.w3.org/2000/svg",
            "http://www.w3.org/1998/Math/MathML",
            "urn:other",
        ]
        .into_iter()
        .map(|uri| {
            Op::Element(
                0,
                raw(uri, Some(""), "Title"),
                vec![
                    attr(None, "z-last", " invalid &amp; "),
                    Attr::Raw(raw("urn:custom", Some("p"), "id"), vec![0xDFFF]),
                    attr(None, "id", "native"),
                ],
            )
        })
        .collect(),
        2 => {
            root = 2;
            vec![Op::Doctype(
                0,
                units("HTML"),
                Some(vec![0xD800]),
                Some(vec![]),
            )]
        }
        3 => vec![
            Op::Pi(0, units("_Target-9"), vec![32, 0xD800, 63, 97]),
            Op::Pi(0, units("xml"), units("data")),
            Op::Pi(0, units("Xml-StYlEsHeEt"), units("data")),
            Op::Pi(0, units("9start"), units("data")),
            Op::Pi(0, units("x"), units("a>b")),
            Op::Pi(0, units("x"), units("a?")),
            Op::Text(0, units("after")),
        ],
        4 => vec![
            Op::Element(0, Name::Known(2), vec![]),
            Op::Text(1, units("wrong parent")),
            Op::Contents(1),
            Op::Text(3, units("inside")),
            Op::Element(
                0,
                raw("http://www.w3.org/2000/svg", None, "template"),
                vec![],
            ),
            Op::Text(5, units("ordinary")),
        ],
        5 => vec![
            Op::Element(0, Name::Known(0), vec![]),
            Op::Text(1, units("light")),
            Op::Shadow(1, 23),
            Op::Text(3, units("shadow")),
            Op::Shadow(1, 23),
            Op::Text(0, units("after")),
        ],
        6 => vec![
            Op::Element(
                0,
                Name::Known(0),
                vec![attr(Some("p"), "id", "x"), attr(Some("q"), "id", "y")],
            ),
            Op::Text(0, units("after duplicate")),
        ],
        7 => vec![Op::Text(255, units("foreign")), Op::Text(0, units("valid"))],
        _ => return Err(invalid("unknown native literal")),
    };
    Ok(NativeCase {
        root,
        literal: Some(id),
        caps: CAPS,
        ops,
    })
}
pub(super) fn rejection(id: u8, index: usize) -> Option<&'static str> {
    match (id, index) {
        (3, 1 | 2) => Some("pi-reserved"),
        (3, 3) => Some("pi-target"),
        (3, 4) => Some("pi-greater"),
        (3, 5) => Some("pi-question"),
        (4, 1) => Some("template-parent"),
        (5, 4) => Some("shadow-duplicate"),
        (6, 0) => Some("attribute-duplicate"),
        (7, 0) => Some("foreign-builder"),
        _ => None,
    }
}

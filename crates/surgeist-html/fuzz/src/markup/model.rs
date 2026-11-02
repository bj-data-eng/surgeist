//! A closed recipe language for balanced markup with an independent tree oracle.
//!
//! A recipe is a tree of text, comments and elements drawn from closed tables
//! whose parsed shape does not depend on error recovery: well-nested elements in
//! content positions where the HTML Standard's tree construction (13.2.6) only
//! appends, character references from the tokenizer's reference tables
//! (13.2.5.72-84), the newline preprocessing of 13.2.3.5, case folding of tag and
//! attribute names, the SVG/MathML name adjustments of 13.2.6.5 and the
//! compatibility-mode rules of 13.2.6.4.1. The expected tree is computed from the
//! recipe alone; it is never read back from a parse.
use surgeist_html::HtmlDocumentMode;

pub(crate) const HTML_NS: &str = "http://www.w3.org/1999/xhtml";
pub(crate) const SVG_NS: &str = "http://www.w3.org/2000/svg";
pub(crate) const MATH_NS: &str = "http://www.w3.org/1998/Math/MathML";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";

pub(crate) const MAX_ITEMS: usize = 48;
pub(crate) const MAX_DEPTH: usize = 6;
pub(crate) const MAX_ATTRIBUTES: usize = 4;
pub(crate) const MAX_PIECES: usize = 384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Ns {
    Html,
    Svg,
    Math,
}
impl Ns {
    pub(crate) fn uri(self) -> &'static str {
        match self {
            Self::Html => HTML_NS,
            Self::Svg => SVG_NS,
            Self::Math => MATH_NS,
        }
    }
    fn mask(self) -> u8 {
        match self {
            Self::Html => 1,
            Self::Svg => 2,
            Self::Math => 4,
        }
    }
}

/// What a parent may contain without error recovery changing the tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Class {
    /// Flow content: also template contents and HTML integration points.
    Flow,
    /// Phrasing content, which may sit inside an ancestor paragraph.
    Phrasing,
    /// A paragraph's own content: phrasing without foreign roots.
    Paragraph,
    SvgContent,
    SvgLeaf,
    MathContent,
    MathText,
}
impl Class {
    pub(crate) fn accepts_text(self) -> bool {
        !matches!(self, Self::SvgLeaf | Self::MathContent)
    }
    /// Text of an HTML or integration-point parent follows the HTML insertion
    /// modes; text of any other foreign parent follows 13.2.6.5.
    pub(crate) fn foreign_text(self) -> bool {
        matches!(self, Self::SvgContent | Self::SvgLeaf | Self::MathContent)
    }
    pub(crate) fn allows(self, tag: usize, inside_anchor: bool) -> bool {
        let anchor = tag == A && inside_anchor;
        match self {
            Self::Flow => {
                matches!(
                    tag,
                    DIV | SECTION | SPAN | B | EM | A | P | BR | HR | TEMPLATE | SVG | MATH
                ) && !anchor
            }
            Self::Phrasing => matches!(tag, SPAN | B | EM | A | BR | SVG | MATH) && !anchor,
            Self::Paragraph => matches!(tag, SPAN | B | EM | A | BR) && !anchor,
            Self::SvgContent => matches!(tag, G | RECT | CIRCLE | FOREIGN_OBJECT),
            Self::MathContent => matches!(tag, MROW | MI | MN),
            Self::SvgLeaf | Self::MathText => false,
        }
    }
}

pub(crate) const DIV: usize = 0;
pub(crate) const SECTION: usize = 1;
pub(crate) const SPAN: usize = 2;
pub(crate) const B: usize = 3;
pub(crate) const EM: usize = 4;
pub(crate) const A: usize = 5;
pub(crate) const P: usize = 6;
pub(crate) const BR: usize = 7;
pub(crate) const HR: usize = 8;
pub(crate) const TEMPLATE: usize = 9;
pub(crate) const SVG: usize = 10;
pub(crate) const G: usize = 11;
pub(crate) const RECT: usize = 12;
pub(crate) const CIRCLE: usize = 13;
pub(crate) const FOREIGN_OBJECT: usize = 14;
pub(crate) const MATH: usize = 15;
pub(crate) const MROW: usize = 16;
pub(crate) const MI: usize = 17;
pub(crate) const MN: usize = 18;

pub(crate) struct Tag {
    /// Lowercase source spelling, as the tokenizer folds it.
    pub(crate) source: &'static str,
    /// The element's local name after the 13.2.6.5 SVG tag-name adjustment.
    pub(crate) local: &'static str,
    pub(crate) ns: Ns,
    pub(crate) class: Class,
    pub(crate) void: bool,
}
const fn tag(source: &'static str, local: &'static str, ns: Ns, class: Class) -> Tag {
    Tag {
        source,
        local,
        ns,
        class,
        void: false,
    }
}
pub(crate) const TAGS: [Tag; 19] = [
    tag("div", "div", Ns::Html, Class::Flow),
    tag("section", "section", Ns::Html, Class::Flow),
    tag("span", "span", Ns::Html, Class::Phrasing),
    tag("b", "b", Ns::Html, Class::Phrasing),
    tag("em", "em", Ns::Html, Class::Phrasing),
    tag("a", "a", Ns::Html, Class::Phrasing),
    tag("p", "p", Ns::Html, Class::Paragraph),
    Tag {
        void: true,
        ..tag("br", "br", Ns::Html, Class::Phrasing)
    },
    Tag {
        void: true,
        ..tag("hr", "hr", Ns::Html, Class::Flow)
    },
    tag("template", "template", Ns::Html, Class::Flow),
    tag("svg", "svg", Ns::Svg, Class::SvgContent),
    tag("g", "g", Ns::Svg, Class::SvgContent),
    tag("rect", "rect", Ns::Svg, Class::SvgLeaf),
    tag("circle", "circle", Ns::Svg, Class::SvgLeaf),
    tag("foreignobject", "foreignObject", Ns::Svg, Class::Flow),
    tag("math", "math", Ns::Math, Class::MathContent),
    tag("mrow", "mrow", Ns::Math, Class::MathContent),
    tag("mi", "mi", Ns::Math, Class::MathText),
    tag("mn", "mn", Ns::Math, Class::MathText),
];

pub(crate) struct Attr {
    /// Lowercase source spelling.
    pub(crate) source: &'static str,
    uri: &'static str,
    prefix: Option<&'static str>,
    local: &'static str,
    /// Namespaces of elements that may carry it.
    applies: u8,
}
impl Attr {
    pub(crate) fn uri_str(&self) -> &'static str {
        self.uri
    }
    pub(crate) fn prefix_str(&self) -> Option<&'static str> {
        self.prefix
    }
    pub(crate) fn local_str(&self) -> &'static str {
        self.local
    }
}
const fn attr(
    source: &'static str,
    uri: &'static str,
    prefix: Option<&'static str>,
    local: &'static str,
    applies: u8,
) -> Attr {
    Attr {
        source,
        uri,
        prefix,
        local,
        applies,
    }
}
pub(crate) const ATTRS: [Attr; 12] = [
    attr("id", "", None, "id", 7),
    attr("class", "", None, "class", 7),
    attr("title", "", None, "title", 7),
    attr("lang", "", None, "lang", 1),
    attr("data-k", "", None, "data-k", 1),
    attr("hidden", "", None, "hidden", 1),
    attr("viewbox", "", None, "viewBox", 2),
    attr("xlink:href", XLINK_NS, Some("xlink"), "href", 2),
    attr("xml:lang", XML_NS, Some("xml"), "lang", 6),
    attr("fill", "", None, "fill", 2),
    attr("definitionurl", "", None, "definitionURL", 4),
    attr("xmlns:xlink", XMLNS_NS, Some("xmlns"), "xlink", 2),
];
pub(crate) fn attribute_applies(index: usize, ns: Ns) -> bool {
    ATTRS.get(index).is_some_and(|a| a.applies & ns.mask() != 0)
}

/// Named character references spelled with their semicolon.
pub(crate) const NAMED: [(&str, &[u16]); 8] = [
    ("&amp;", &[0x26]),
    ("&lt;", &[0x3C]),
    ("&gt;", &[0x3E]),
    ("&quot;", &[0x22]),
    ("&apos;", &[0x27]),
    ("&nbsp;", &[0xA0]),
    ("&copy;", &[0xA9]),
    ("&hellip;", &[0x2026]),
];
/// 13.2.5.84 numeric character reference end state: the scalar a code yields.
pub(crate) fn numeric_value(code: u32) -> u32 {
    const C1: [(u32, u32); 27] = [
        (0x80, 0x20AC),
        (0x82, 0x201A),
        (0x83, 0x0192),
        (0x84, 0x201E),
        (0x85, 0x2026),
        (0x86, 0x2020),
        (0x87, 0x2021),
        (0x88, 0x02C6),
        (0x89, 0x2030),
        (0x8A, 0x0160),
        (0x8B, 0x2039),
        (0x8C, 0x0152),
        (0x8E, 0x017D),
        (0x91, 0x2018),
        (0x92, 0x2019),
        (0x93, 0x201C),
        (0x94, 0x201D),
        (0x95, 0x2022),
        (0x96, 0x2013),
        (0x97, 0x2014),
        (0x98, 0x02DC),
        (0x99, 0x2122),
        (0x9A, 0x0161),
        (0x9B, 0x203A),
        (0x9C, 0x0153),
        (0x9E, 0x017E),
        (0x9F, 0x0178),
    ];
    if code == 0 || code > 0x10FFFF || (0xD800..=0xDFFF).contains(&code) {
        return 0xFFFD;
    }
    C1.iter()
        .find(|(c, _)| *c == code)
        .map_or(code, |(_, v)| *v)
}

/// Where a piece's value lands, which decides what a NUL becomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Flavor {
    /// Character tokens the HTML insertion modes process: NUL is ignored.
    HtmlText,
    /// Character tokens of foreign content, attribute values and comments:
    /// NUL becomes U+FFFD.
    Replaced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Piece {
    /// Any unit except NUL, CR, `<`, `&`, `"` and `'`.
    Unit(u16),
    Named(u8),
    Numeric {
        code: u32,
        hex: bool,
    },
    Cr,
    CrLf,
    Nul,
}
impl Piece {
    pub(crate) fn unit_allowed(unit: u16) -> bool {
        !matches!(unit, 0 | 13 | 0x3C | 0x26 | 0x22 | 0x27)
    }
    pub(crate) fn spelling(&self, out: &mut Vec<u16>) {
        match self {
            Self::Unit(u) => out.push(*u),
            Self::Named(i) => {
                if let Some((s, _)) = NAMED.get(usize::from(*i)) {
                    out.extend(s.encode_utf16());
                }
            }
            Self::Numeric { code, hex } => {
                let s = if *hex {
                    format!("&#x{code:X};")
                } else {
                    format!("&#{code};")
                };
                out.extend(s.encode_utf16());
            }
            Self::Cr => out.push(13),
            Self::CrLf => out.extend([13, 10]),
            Self::Nul => out.push(0),
        }
    }
    pub(crate) fn value(&self, flavor: Flavor, out: &mut Vec<u16>) {
        match self {
            Self::Unit(u) => out.push(*u),
            Self::Named(i) => {
                if let Some((_, v)) = NAMED.get(usize::from(*i)) {
                    out.extend(*v);
                }
            }
            Self::Numeric { code, .. } => {
                let scalar = char::from_u32(numeric_value(*code)).unwrap_or('\u{FFFD}');
                let mut buffer = [0; 2];
                out.extend(scalar.encode_utf16(&mut buffer).iter());
            }
            // 13.2.3.5: CR and CRLF each become one LF.
            Self::Cr | Self::CrLf => out.push(10),
            Self::Nul => {
                if flavor == Flavor::Replaced {
                    out.push(0xFFFD);
                }
            }
        }
    }
}
pub(crate) fn values(pieces: &[Piece], flavor: Flavor) -> Vec<u16> {
    let mut out = Vec::new();
    for p in pieces {
        p.value(flavor, &mut out);
    }
    out
}
pub(crate) fn spellings(pieces: &[Piece]) -> Vec<u16> {
    let mut out = Vec::new();
    for p in pieces {
        p.spelling(&mut out);
    }
    out
}
/// A CR immediately followed by a spelled LF would fuse into one newline.
pub(crate) fn fuses_newline(pieces: &[Piece]) -> bool {
    pieces
        .windows(2)
        .any(|w| w[0] == Piece::Cr && w[1] == Piece::Unit(10))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Attribute {
    pub(crate) name: u8,
    /// ASCII case pattern of the spelled name.
    pub(crate) case: u8,
    /// 0 double quoted, 1 single quoted, 2 unquoted when the value permits.
    pub(crate) quote: u8,
    pub(crate) value: Vec<Piece>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Shadow {
    /// Bit 0 closed, 1 manual slot assignment, 2 delegates focus, 3 clonable,
    /// 4 serializable.
    pub(crate) bits: u8,
    pub(crate) children: Vec<Item>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Elem {
    pub(crate) tag: u8,
    /// Bits 0-1 start-tag case, bits 2-3 end-tag case, bit 7 self-closing.
    pub(crate) spell: u8,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) shadow: Option<Shadow>,
    pub(crate) children: Vec<Item>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Item {
    Text(Vec<Piece>),
    Comment(Vec<Piece>),
    Elem(Elem),
}
impl Elem {
    /// 13.2.6.4.1 valid shadow host names reachable by this language.
    pub(crate) fn shadow_host(&self) -> bool {
        matches!(usize::from(self.tag), DIV | SECTION | SPAN | P)
    }
}

pub(crate) fn apply_case(name: &str, pattern: u8) -> String {
    name.chars()
        .enumerate()
        .map(|(i, c)| match pattern & 3 {
            1 => c.to_ascii_uppercase(),
            2 if i % 2 == 0 => c.to_ascii_uppercase(),
            3 if i == 0 => c.to_ascii_uppercase(),
            _ => c,
        })
        .collect()
}

/// Structural validity of a recipe, independent of any encoding.
pub(crate) struct Counts {
    pub(crate) items: usize,
    pub(crate) pieces: usize,
    pub(crate) units: usize,
    pub(crate) depth: usize,
}
pub(crate) fn check_items(
    items: &[Item],
    class: Class,
    inside_anchor: bool,
    shadow_ok: bool,
    depth: usize,
    counts: &mut Counts,
) -> Result<(), &'static str> {
    if depth > MAX_DEPTH {
        return Err("recipe depth");
    }
    counts.depth = counts.depth.max(depth);
    let mut previous_text = false;
    for item in items {
        counts.items += 1;
        if counts.items > MAX_ITEMS {
            return Err("recipe item count");
        }
        let is_text = matches!(item, Item::Text(_));
        if is_text && previous_text {
            return Err("adjacent text items");
        }
        previous_text = is_text;
        match item {
            Item::Text(pieces) => {
                if !class.accepts_text() {
                    return Err("text not permitted here");
                }
                check_pieces(pieces, Flavor::HtmlText, class, counts)?;
                let flavor = if class.foreign_text() {
                    Flavor::Replaced
                } else {
                    Flavor::HtmlText
                };
                if values(pieces, flavor).is_empty() {
                    return Err("text without a parsed value");
                }
            }
            Item::Comment(pieces) => {
                for p in pieces {
                    match p {
                        Piece::Unit(u)
                            if !matches!(*u, 0x2D | 0x3E | 0x21) && Piece::unit_allowed(*u) => {}
                        Piece::Cr | Piece::CrLf | Piece::Nul => {}
                        _ => return Err("comment piece"),
                    }
                }
                if pieces.len() > MAX_PIECES || fuses_newline(pieces) {
                    return Err("comment pieces");
                }
                counts.pieces += pieces.len();
                counts.units += spellings(pieces).len();
            }
            Item::Elem(e) => {
                let tag = usize::from(e.tag);
                let Some(info) = TAGS.get(tag) else {
                    return Err("unknown tag");
                };
                if !class.allows(tag, inside_anchor) {
                    return Err("tag not permitted here");
                }
                if e.attrs.len() > MAX_ATTRIBUTES {
                    return Err("attribute count");
                }
                for a in &e.attrs {
                    if !attribute_applies(usize::from(a.name), info.ns) || a.quote > 2 {
                        return Err("attribute not applicable");
                    }
                    check_pieces(&a.value, Flavor::Replaced, Class::Phrasing, counts)?;
                    counts.units += a.value.len();
                }
                let inner = inside_anchor || tag == A;
                let child_class = info.class;
                if info.void && (!e.children.is_empty() || e.shadow.is_some()) {
                    return Err("void element content");
                }
                if e.spell & 0x7F >= 16 {
                    return Err("spelling bits");
                }
                if e.spell & 0x80 != 0
                    && (info.ns == Ns::Html && !info.void
                        || !e.children.is_empty()
                        || e.shadow.is_some())
                {
                    return Err("self-closing form");
                }
                if let Some(shadow) = &e.shadow {
                    if !shadow_ok || !e.shadow_host() || shadow.bits > 31 {
                        return Err("shadow host");
                    }
                    check_items(
                        &shadow.children,
                        child_class,
                        inner,
                        shadow_ok,
                        depth + 1,
                        counts,
                    )?;
                }
                check_items(
                    &e.children,
                    child_class,
                    inner,
                    shadow_ok,
                    depth + 1,
                    counts,
                )?;
            }
        }
    }
    Ok(())
}
fn check_pieces(
    pieces: &[Piece],
    _flavor: Flavor,
    _class: Class,
    counts: &mut Counts,
) -> Result<(), &'static str> {
    if pieces.len() > MAX_PIECES {
        return Err("piece count");
    }
    counts.pieces += pieces.len();
    if counts.pieces > MAX_PIECES * 2 {
        return Err("aggregate pieces");
    }
    for p in pieces {
        match p {
            Piece::Unit(u) if !Piece::unit_allowed(*u) => return Err("forbidden unit"),
            Piece::Named(i) if usize::from(*i) >= NAMED.len() => return Err("named reference"),
            Piece::Numeric { code, .. } if *code > 0x11_0000 + 16 => {
                return Err("numeric reference range");
            }
            _ => {}
        }
    }
    if fuses_newline(pieces) {
        return Err("newline fusion");
    }
    counts.units += spellings(pieces).len();
    Ok(())
}
/// An unquoted attribute value is only spelled for non-empty ASCII alphanumerics.
pub(crate) fn unquotable(pieces: &[Piece]) -> bool {
    !pieces.is_empty()
        && pieces.iter().all(
            |p| matches!(p, Piece::Unit(u) if matches!(u, 0x30..=0x39 | 0x41..=0x5A | 0x61..=0x7A)),
        )
}

/// Document prologue variants of 13.2.6.4.1, with their mode and doctype.
pub(crate) struct Doctype {
    pub(crate) source: &'static str,
    pub(crate) name: &'static str,
    pub(crate) public: Option<&'static str>,
    pub(crate) system: Option<&'static str>,
    pub(crate) mode: HtmlDocumentMode,
}
const fn doctype(
    source: &'static str,
    name: &'static str,
    public: Option<&'static str>,
    system: Option<&'static str>,
    mode: HtmlDocumentMode,
) -> Doctype {
    Doctype {
        source,
        name,
        public,
        system,
        mode,
    }
}
pub(crate) const DOCTYPES: [Option<Doctype>; 10] = [
    None,
    Some(doctype(
        "<!DOCTYPE html>",
        "html",
        None,
        None,
        HtmlDocumentMode::NoQuirks,
    )),
    Some(doctype(
        "<!doctype HTML>",
        "html",
        None,
        None,
        HtmlDocumentMode::NoQuirks,
    )),
    Some(doctype(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">",
        "html",
        Some("-//W3C//DTD XHTML 1.0 Transitional//EN"),
        Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd"),
        HtmlDocumentMode::LimitedQuirks,
    )),
    Some(doctype(
        "<!DOCTYPE html public \"-//W3C//DTD HTML 4.01 Transitional//EN\">",
        "html",
        Some("-//W3C//DTD HTML 4.01 Transitional//EN"),
        None,
        HtmlDocumentMode::Quirks,
    )),
    Some(doctype(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01//EN\" \"http://www.w3.org/TR/html4/strict.dtd\">",
        "html",
        Some("-//W3C//DTD HTML 4.01//EN"),
        Some("http://www.w3.org/TR/html4/strict.dtd"),
        HtmlDocumentMode::NoQuirks,
    )),
    Some(doctype(
        "<!DOCTYPE html SYSTEM \"about:legacy-compat\">",
        "html",
        None,
        Some("about:legacy-compat"),
        HtmlDocumentMode::NoQuirks,
    )),
    Some(doctype(
        "<!DOCTYPE foo>",
        "foo",
        None,
        None,
        HtmlDocumentMode::Quirks,
    )),
    Some(doctype(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01 Transitional//EN\" \"http://www.w3.org/TR/html4/loose.dtd\">",
        "html",
        Some("-//W3C//DTD HTML 4.01 Transitional//EN"),
        Some("http://www.w3.org/TR/html4/loose.dtd"),
        HtmlDocumentMode::LimitedQuirks,
    )),
    Some(doctype(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01 Transitional//EN\" \"\">",
        "html",
        Some("-//W3C//DTD HTML 4.01 Transitional//EN"),
        Some(""),
        HtmlDocumentMode::Quirks,
    )),
];
pub(crate) fn compatibility_mode(index: u8) -> HtmlDocumentMode {
    DOCTYPES
        .get(usize::from(index))
        .and_then(Option::as_ref)
        .map_or(HtmlDocumentMode::Quirks, |d| d.mode)
}

/// Document shells whose explicit and implied head/body tags yield one tree.
pub(crate) const SHELLS: [(&str, &str); 3] = [
    ("<html><head></head><body>", "</body></html>"),
    ("<body>", ""),
    ("<head></head><body>", "</body>"),
];

pub(crate) struct Context {
    pub(crate) ns: Ns,
    pub(crate) local: &'static str,
    /// The class of the context's content when its parse is error-free.
    pub(crate) class: Option<Class>,
}
const fn context(ns: Ns, local: &'static str, class: Option<Class>) -> Context {
    Context { ns, local, class }
}
pub(crate) const CONTEXTS: [Context; 24] = [
    context(Ns::Html, "div", Some(Class::Flow)),
    context(Ns::Html, "section", Some(Class::Flow)),
    context(Ns::Html, "template", Some(Class::Flow)),
    context(Ns::Svg, "g", Some(Class::SvgContent)),
    context(Ns::Svg, "foreignObject", Some(Class::Flow)),
    context(Ns::Html, "table", None),
    context(Ns::Html, "tr", None),
    context(Ns::Html, "td", None),
    context(Ns::Html, "select", None),
    context(Ns::Html, "caption", None),
    context(Ns::Html, "colgroup", None),
    context(Ns::Html, "body", None),
    context(Ns::Html, "html", None),
    context(Ns::Html, "head", None),
    context(Ns::Html, "textarea", None),
    context(Ns::Html, "title", None),
    context(Ns::Html, "style", None),
    context(Ns::Html, "script", None),
    context(Ns::Math, "mi", None),
    context(Ns::Math, "annotation-xml", None),
    context(Ns::Svg, "svg", None),
    context(Ns::Html, "frameset", None),
    context(Ns::Html, "plaintext", None),
    context(Ns::Html, "option", None),
];

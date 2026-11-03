//! The expected tree and the source spelling of a markup recipe.
use super::model::*;
use surgeist_html::construction::{
    HtmlDocumentMode, HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpAttr {
    pub(crate) uri: &'static str,
    pub(crate) prefix: Option<&'static str>,
    pub(crate) local: &'static str,
    pub(crate) value: Vec<u16>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExpKind {
    Document(HtmlDocumentMode),
    Fragment,
    /// A missing public or system identifier is the empty string (13.2.6.4.1).
    Doctype {
        name: &'static str,
        public: &'static str,
        system: &'static str,
    },
    Element {
        uri: &'static str,
        local: &'static str,
        attrs: Vec<ExpAttr>,
    },
    Text(Vec<u16>),
    Comment(Vec<u16>),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Exp {
    pub(crate) kind: ExpKind,
    pub(crate) children: Vec<Exp>,
    /// Template contents of a template element.
    pub(crate) contents: Option<Vec<Exp>>,
    /// Attached declarative shadow root: properties and contents.
    pub(crate) shadow: Option<(HtmlShadowRootProperties, Vec<Exp>)>,
}
impl Exp {
    fn leaf(kind: ExpKind) -> Self {
        Self {
            kind,
            children: Vec::new(),
            contents: None,
            shadow: None,
        }
    }
}

pub(crate) fn shadow_properties(bits: u8) -> HtmlShadowRootProperties {
    HtmlShadowRootProperties::new(
        if bits & 1 == 0 {
            HtmlShadowRootMode::Open
        } else {
            HtmlShadowRootMode::Closed
        },
        if bits & 2 == 0 {
            HtmlSlotAssignment::Named
        } else {
            HtmlSlotAssignment::Manual
        },
        bits & 4 != 0,
        bits & 8 != 0,
        bits & 16 != 0,
    )
}
/// The attributes of the declarative shadow template, in spelled order.
fn shadow_attributes(bits: u8) -> Vec<(&'static str, &'static str)> {
    let mut out = vec![(
        "shadowrootmode",
        if bits & 1 == 0 { "open" } else { "closed" },
    )];
    if bits & 4 != 0 {
        out.push(("shadowrootdelegatesfocus", ""));
    }
    if bits & 8 != 0 {
        out.push(("shadowrootclonable", ""));
    }
    if bits & 16 != 0 {
        out.push(("shadowrootserializable", ""));
    }
    if bits & 2 != 0 {
        out.push(("shadowrootslotassignment", "manual"));
    }
    out
}

pub(crate) fn expect_items(items: &[Item], class: Class, shadow_permitted: bool) -> Vec<Exp> {
    let flavor = if class.foreign_text() {
        Flavor::Replaced
    } else {
        Flavor::HtmlText
    };
    items
        .iter()
        .map(|item| match item {
            Item::Text(p) => Exp::leaf(ExpKind::Text(values(p, flavor))),
            Item::Comment(p) => Exp::leaf(ExpKind::Comment(values(p, Flavor::Replaced))),
            Item::Elem(e) => expect_element(e, shadow_permitted),
        })
        .collect()
}
fn expect_element(e: &Elem, shadow_permitted: bool) -> Exp {
    let tag = usize::from(e.tag);
    let info = &TAGS[tag];
    let mut attrs: Vec<ExpAttr> = Vec::new();
    // The tokenizer drops a later attribute whose spelled name repeats an
    // earlier one (13.2.5.33), before any foreign adjustment.
    for (i, a) in e.attrs.iter().enumerate() {
        if e.attrs[..i].iter().any(|b| b.name == a.name) {
            continue;
        }
        let def = &ATTRS[usize::from(a.name)];
        attrs.push(ExpAttr {
            uri: def.uri_str(),
            prefix: def.prefix_str(),
            local: def.local_str(),
            value: values(&a.value, Flavor::Replaced),
        });
    }
    let mut exp = Exp::leaf(ExpKind::Element {
        uri: info.ns.uri(),
        local: info.local,
        attrs,
    });
    let class = info.class;
    if tag == TEMPLATE {
        exp.contents = Some(expect_items(&e.children, class, shadow_permitted));
        return exp;
    }
    exp.children = expect_items(&e.children, class, shadow_permitted);
    if let Some(shadow) = &e.shadow {
        let inner = expect_items(&shadow.children, class, shadow_permitted);
        if shadow_permitted {
            exp.shadow = Some((shadow_properties(shadow.bits), inner));
        } else {
            // Without permission the template is an ordinary first child.
            let attrs = shadow_attributes(shadow.bits)
                .into_iter()
                .map(|(name, value)| ExpAttr {
                    uri: "",
                    prefix: None,
                    local: name,
                    value: value.encode_utf16().collect(),
                })
                .collect();
            let mut template = Exp::leaf(ExpKind::Element {
                uri: HTML_NS,
                local: "template",
                attrs,
            });
            template.contents = Some(inner);
            exp.children.insert(0, template);
        }
    }
    exp
}
pub(crate) fn expect_document(doctype: u8, items: &[Item]) -> Exp {
    let mut html = Exp::leaf(ExpKind::Element {
        uri: HTML_NS,
        local: "html",
        attrs: Vec::new(),
    });
    let head = Exp::leaf(ExpKind::Element {
        uri: HTML_NS,
        local: "head",
        attrs: Vec::new(),
    });
    let mut body = Exp::leaf(ExpKind::Element {
        uri: HTML_NS,
        local: "body",
        attrs: Vec::new(),
    });
    body.children = expect_items(items, Class::Flow, false);
    html.children = vec![head, body];
    let mut root = Exp::leaf(ExpKind::Document(compatibility_mode(doctype)));
    if let Some(Some(d)) = DOCTYPES.get(usize::from(doctype)) {
        root.children.push(Exp::leaf(ExpKind::Doctype {
            name: d.name,
            public: d.public.unwrap_or(""),
            system: d.system.unwrap_or(""),
        }));
    }
    root.children.push(html);
    root
}
pub(crate) fn expect_fragment(items: &[Item], class: Class, shadow_permitted: bool) -> Exp {
    let mut root = Exp::leaf(ExpKind::Fragment);
    root.children = expect_items(items, class, shadow_permitted);
    root
}

fn push_str(out: &mut Vec<u16>, s: &str) {
    out.extend(s.encode_utf16());
}
fn push_attribute(out: &mut Vec<u16>, name: &str, value: &[u16], quote: u8, bare_ok: bool) {
    out.push(32);
    push_str(out, name);
    if value.is_empty() && bare_ok {
        return;
    }
    out.push(0x3D);
    let quote_unit = match quote {
        1 => 0x27,
        _ => 0x22,
    };
    out.push(quote_unit);
    out.extend(value);
    out.push(quote_unit);
}
pub(crate) fn serialize_items(items: &[Item], out: &mut Vec<u16>) {
    for item in items {
        match item {
            Item::Text(p) => out.extend(spellings(p)),
            Item::Comment(p) => {
                push_str(out, "<!--");
                out.extend(spellings(p));
                push_str(out, "-->");
            }
            Item::Elem(e) => serialize_element(e, out),
        }
    }
}
fn serialize_element(e: &Elem, out: &mut Vec<u16>) {
    let info = &TAGS[usize::from(e.tag)];
    out.push(0x3C);
    push_str(out, &apply_case(info.source, e.spell));
    for a in &e.attrs {
        let name = apply_case(ATTRS[usize::from(a.name)].source, a.case);
        let value = spellings(&a.value);
        if a.quote == 2 && unquotable(&a.value) {
            out.push(32);
            push_str(out, &name);
            out.push(0x3D);
            out.extend(value);
        } else {
            push_attribute(out, &name, &value, a.quote.min(1), a.quote == 2);
        }
    }
    if e.spell & 0x80 != 0 {
        if !e.attrs.is_empty() {
            out.push(32);
        }
        out.push(0x2F);
    }
    out.push(0x3E);
    if e.spell & 0x80 != 0 || info.void {
        return;
    }
    if let Some(shadow) = &e.shadow {
        push_str(out, "<template");
        for (name, value) in shadow_attributes(shadow.bits) {
            push_attribute(
                out,
                name,
                &value.encode_utf16().collect::<Vec<_>>(),
                0,
                true,
            );
        }
        out.push(0x3E);
        serialize_items(&shadow.children, out);
        push_str(out, "</template>");
    }
    serialize_items(&e.children, out);
    push_str(out, "</");
    push_str(out, &apply_case(info.source, e.spell >> 2));
    out.push(0x3E);
}
pub(crate) fn serialize_document(doctype: u8, shell: u8, items: &[Item]) -> Vec<u16> {
    let mut out = Vec::new();
    if let Some(Some(d)) = DOCTYPES.get(usize::from(doctype)) {
        push_str(&mut out, d.source);
    }
    let (open, close) = SHELLS[usize::from(shell).min(SHELLS.len() - 1)];
    push_str(&mut out, open);
    serialize_items(items, &mut out);
    push_str(&mut out, close);
    out
}
pub(crate) fn serialize_fragment(items: &[Item]) -> Vec<u16> {
    let mut out = Vec::new();
    serialize_items(items, &mut out);
    out
}

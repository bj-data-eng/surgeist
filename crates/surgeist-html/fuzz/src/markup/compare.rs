//! Structural comparison of a parsed tree with a recipe's expected tree.
use super::tree::{Exp, ExpKind};
use crate::observe::{NativeKind, NativeNode};
use std::collections::HashMap;
use surgeist_html::{
    construction::HtmlNodeId,
    values::{DocumentString, QualifiedName},
};

struct Tree<'a> {
    nodes: HashMap<&'a HtmlNodeId, &'a NativeNode>,
}
fn text_is(actual: &DocumentString, expected: &[u16]) -> bool {
    actual.code_units() == expected
}
fn name_is(actual: &QualifiedName, uri: &str, prefix: Option<&str>, local: &str) -> bool {
    let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
    actual
        .namespace()
        .uri()
        .map_or(&[][..], DocumentString::code_units)
        == units(uri).as_slice()
        && actual.prefix().map(DocumentString::code_units) == prefix.map(units).as_deref()
        && actual.local_name().code_units() == units(local).as_slice()
}
impl Tree<'_> {
    fn node(&self, id: &HtmlNodeId) -> Result<&NativeNode, &'static str> {
        self.nodes.get(id).copied().ok_or("tree-unreachable")
    }
    fn children(&self, ids: &[HtmlNodeId], expected: &[Exp]) -> Result<(), &'static str> {
        if ids.len() != expected.len() {
            return Err("tree-children");
        }
        for (id, exp) in ids.iter().zip(expected) {
            self.check(self.node(id)?, exp)?;
        }
        Ok(())
    }
    fn check(&self, node: &NativeNode, exp: &Exp) -> Result<(), &'static str> {
        match (&node.kind, &exp.kind) {
            (NativeKind::Document(actual), ExpKind::Document(mode)) => {
                if actual != mode {
                    return Err("tree-mode");
                }
            }
            (NativeKind::Fragment, ExpKind::Fragment) => {}
            (
                NativeKind::Doctype(d),
                ExpKind::Doctype {
                    name,
                    public,
                    system,
                },
            ) => {
                let units = |s: &str| s.encode_utf16().collect::<Vec<_>>();
                if !text_is(d.name(), &units(name))
                    || d.public_id().map(DocumentString::code_units)
                        != Some(units(public).as_slice())
                    || d.system_id().map(DocumentString::code_units)
                        != Some(units(system).as_slice())
                {
                    return Err("tree-doctype");
                }
            }
            (NativeKind::Element(e), ExpKind::Element { uri, local, attrs }) => {
                if !name_is(e.name(), uri, None, local) {
                    return Err("tree-name");
                }
                let actual = e.attributes().as_slice();
                if actual.len() != attrs.len()
                    || !actual.iter().zip(attrs).all(|(a, x)| {
                        name_is(a.name(), x.uri, x.prefix, x.local) && text_is(a.value(), &x.value)
                    })
                {
                    return Err("tree-attributes");
                }
            }
            (NativeKind::Text(t), ExpKind::Text(units))
            | (NativeKind::Comment(t), ExpKind::Comment(units)) => {
                if !text_is(t, units) {
                    return Err("tree-text");
                }
            }
            _ => return Err("tree-kind"),
        }
        self.children(&node.children, &exp.children)?;
        match (&node.template, &exp.contents) {
            (None, None) => {}
            (Some(id), Some(items)) => {
                let contents = self.node(id)?;
                if !matches!(contents.kind, NativeKind::Fragment) {
                    return Err("tree-template");
                }
                self.children(&contents.children, items)?;
            }
            _ => return Err("tree-template"),
        }
        match (&node.shadow, &exp.shadow) {
            (None, None) => {}
            (Some((id, properties)), Some((expected, items))) => {
                if properties != expected {
                    return Err("tree-shadow");
                }
                let contents = self.node(id)?;
                if !matches!(contents.kind, NativeKind::Fragment) {
                    return Err("tree-shadow");
                }
                self.children(&contents.children, items)?;
            }
            _ => return Err("tree-shadow"),
        }
        Ok(())
    }
}
/// Compares every reachable node; `nodes[0]` is the document or fragment root.
pub(crate) fn tree(nodes: &[NativeNode], expected: &Exp) -> Result<(), &'static str> {
    let by_id = Tree {
        nodes: nodes.iter().map(|n| (&n.id, n)).collect(),
    };
    let root = nodes.first().ok_or("tree-empty")?;
    by_id.check(root, expected)?;
    // Every observed node is reachable from the compared root: a parse may not
    // retain stray nodes in the report.
    fn count(exp: &Exp) -> usize {
        1 + exp.children.iter().map(count).sum::<usize>()
            + exp
                .contents
                .as_ref()
                .map_or(0, |c| 1 + c.iter().map(count).sum::<usize>())
            + exp
                .shadow
                .as_ref()
                .map_or(0, |(_, c)| 1 + c.iter().map(count).sum::<usize>())
    }
    if count(expected) != nodes.len() {
        return Err("tree-node-count");
    }
    Ok(())
}

/// Invariants every parsed tree holds whatever the input: HTML names are
/// folded, text is non-empty, attribute expanded names are unique, and every
/// HTML template element has contents.
pub(crate) fn general(nodes: &[NativeNode]) -> Result<(), &'static str> {
    let html = "http://www.w3.org/1999/xhtml"
        .encode_utf16()
        .collect::<Vec<_>>();
    for n in nodes {
        match &n.kind {
            NativeKind::Text(t) if t.code_units().is_empty() => return Err("empty-text-node"),
            NativeKind::Element(e) => {
                let in_html = e
                    .name()
                    .namespace()
                    .uri()
                    .is_some_and(|u| u.code_units() == html.as_slice());
                if in_html {
                    if e.name()
                        .local_name()
                        .code_units()
                        .iter()
                        .any(|u| (0x41..=0x5A).contains(u))
                    {
                        return Err("html-name-case");
                    }
                    if e.name().local_name().code_units() == [116, 101, 109, 112, 108, 97, 116, 101]
                        && n.template.is_none()
                    {
                        return Err("template-without-contents");
                    }
                    if e.attributes().as_slice().iter().any(|a| {
                        a.name().namespace().uri().is_none()
                            && a.name()
                                .local_name()
                                .code_units()
                                .iter()
                                .any(|u| (0x41..=0x5A).contains(u))
                    }) {
                        return Err("attribute-name-case");
                    }
                }
                let attrs = e.attributes().as_slice();
                for (i, a) in attrs.iter().enumerate() {
                    if attrs[..i].iter().any(|b| {
                        b.name().namespace().uri() == a.name().namespace().uri()
                            && b.name().local_name() == a.name().local_name()
                    }) {
                        return Err("duplicate-attribute-name");
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

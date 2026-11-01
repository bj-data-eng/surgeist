// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/tag_sets.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. The selected scope lists are
// governed by HTML 877d6146487f15b15bc8dd0e3c625063a37b456a §13.2.4.2.

use super::super::control::{ParseControl, ParseStop, WorkCategory};
use crate::{DocumentString, QualifiedName};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Scope {
    Default,
    ListItem,
    Button,
    Table,
}

const HTML: &str = "http://www.w3.org/1999/xhtml";
const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
const SVG: &str = "http://www.w3.org/2000/svg";

/// Exact units: names containing non-scalars or non-ASCII units simply do not
/// match a defining ASCII literal. No scalar conversion or allocation occurs.
pub(super) fn ascii_eq(
    value: &DocumentString,
    expected: &'static str,
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    if value.len() != expected.len() {
        return Ok(false);
    }
    // The lengths are equal, so the pairs are exactly the literal's indices.
    for (&unit, byte) in value.code_units().iter().zip(expected.bytes()) {
        // Two stored UTF-16 bytes and one defining literal byte are inspected.
        control.charge(WorkCategory::InspectByte, 3)?;
        if unit != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn namespace(
    name: &QualifiedName,
    expected: &'static str,
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    control.charge(WorkCategory::RecordVisit, 2)?;
    match name.namespace().uri() {
        Some(uri) => ascii_eq(uri, expected, control),
        None => Ok(false),
    }
}

pub(super) fn local_in(
    name: &QualifiedName,
    choices: &[&'static str],
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    control.charge(WorkCategory::RecordVisit, 1)?;
    let local = name.local_name();
    for &choice in choices {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if ascii_eq(local, choice, control)? {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn html_named(
    name: &QualifiedName,
    local: &'static str,
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    if !namespace(name, HTML, control)? {
        return Ok(false);
    }
    local_in(name, &[local], control)
}

pub(super) fn foster_target(
    name: &QualifiedName,
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    if !namespace(name, HTML, control)? {
        return Ok(false);
    }
    local_in(name, &["table", "tbody", "tfoot", "thead", "tr"], control)
}

pub(super) fn scope_boundary(
    name: &QualifiedName,
    scope: Scope,
    control: &mut ParseControl,
) -> Result<bool, ParseStop> {
    if namespace(name, HTML, control)? {
        if matches!(scope, Scope::Table) {
            return local_in(name, &["html", "table", "template"], control);
        }
        if local_in(
            name,
            &[
                "applet", "caption", "html", "table", "td", "th", "marquee", "object", "select",
                "template",
            ],
            control,
        )? {
            return Ok(true);
        }
        return match scope {
            Scope::ListItem => local_in(name, &["ol", "ul"], control),
            Scope::Button => local_in(name, &["button"], control),
            Scope::Default | Scope::Table => Ok(false),
        };
    }
    if matches!(scope, Scope::Table) {
        return Ok(false);
    }
    if namespace(name, MATHML, control)? {
        // annotation-xml is a scope boundary regardless of its encoding. The
        // upstream helper used only text integration points and omitted it.
        return local_in(
            name,
            &["mi", "mo", "mn", "ms", "mtext", "annotation-xml"],
            control,
        );
    }
    if namespace(name, SVG, control)? {
        return local_in(name, &["foreignObject", "desc", "title"], control);
    }
    Ok(false)
}

pub(super) fn equal_units(
    left: &DocumentString,
    right: &DocumentString,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    c.charge(WorkCategory::RecordVisit, 2)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    // The lengths are equal, so the pairs are exactly the shared indices.
    for (left, right) in left.code_units().iter().zip(right.code_units()) {
        c.charge(WorkCategory::InspectByte, 4)?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

#[derive(Clone, Copy)]
pub(super) enum NodeSet {
    Implied,
    Heading,
    BodyEnd,
    Special,
}

/// Namespace-qualified source sets, including rb/rtc in the governing EOF set
/// and annotation-xml in the MathML special category.
pub(super) fn in_set(
    name: &QualifiedName,
    set: NodeSet,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    if namespace(name, HTML, c)? {
        return local_in(
            name,
            match set {
                NodeSet::Implied => &[
                    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
                ],
                NodeSet::Heading => &["h1", "h2", "h3", "h4", "h5", "h6"],
                NodeSet::BodyEnd => &[
                    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody",
                    "td", "tfoot", "th", "thead", "tr", "body", "html",
                ],
                NodeSet::Special => &[
                    "address",
                    "applet",
                    "area",
                    "article",
                    "aside",
                    "base",
                    "basefont",
                    "bgsound",
                    "blockquote",
                    "body",
                    "br",
                    "button",
                    "caption",
                    "center",
                    "col",
                    "colgroup",
                    "dd",
                    "details",
                    "dir",
                    "div",
                    "dl",
                    "dt",
                    "embed",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "form",
                    "frame",
                    "frameset",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "h5",
                    "h6",
                    "head",
                    "header",
                    "hgroup",
                    "hr",
                    "html",
                    "iframe",
                    "img",
                    "input",
                    "keygen",
                    "li",
                    "link",
                    "listing",
                    "main",
                    "marquee",
                    "menu",
                    "meta",
                    "nav",
                    "noembed",
                    "noframes",
                    "noscript",
                    "object",
                    "ol",
                    "p",
                    "param",
                    "plaintext",
                    "pre",
                    "script",
                    "search",
                    "section",
                    "select",
                    "source",
                    "style",
                    "summary",
                    "table",
                    "tbody",
                    "td",
                    "template",
                    "textarea",
                    "tfoot",
                    "th",
                    "thead",
                    "title",
                    "tr",
                    "track",
                    "ul",
                    "wbr",
                    "xmp",
                ],
            },
            c,
        );
    }
    if !matches!(set, NodeSet::Special) {
        return Ok(false);
    }
    if namespace(name, MATHML, c)? {
        return local_in(
            name,
            &["mi", "mo", "mn", "ms", "mtext", "annotation-xml"],
            c,
        );
    }
    if namespace(name, SVG, c)? {
        return local_in(name, &["foreignObject", "desc", "title"], c);
    }
    Ok(false)
}

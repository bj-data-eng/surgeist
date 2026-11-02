// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/data.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf. The governing HTML
// 877d6146487f15b15bc8dd0e3c625063a37b456a supplies the prologue conditions.

use crate::HtmlDocumentMode;
use crate::parser::{
    control::{ParseControl, ParseStop, WorkCategory},
    tokenizer::token::{Doctype, Text},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DoctypeDecision {
    pub(super) error: bool,
    pub(super) mode: HtmlDocumentMode,
}

// These should all be lowercase, for ASCII-case-insensitive matching.
static QUIRKY_PUBLIC_PREFIXES: &[&str] = &[
    "+//silmaril//dtd html pro v0r11 19970101//",
    "-//advasoft ltd//dtd html 3.0 aswedit + extensions//",
    "-//as//dtd html 3.0 aswedit + extensions//",
    "-//ietf//dtd html 2.0 level 1//",
    "-//ietf//dtd html 2.0 level 2//",
    "-//ietf//dtd html 2.0 strict level 1//",
    "-//ietf//dtd html 2.0 strict level 2//",
    "-//ietf//dtd html 2.0 strict//",
    "-//ietf//dtd html 2.0//",
    "-//ietf//dtd html 2.1e//",
    "-//ietf//dtd html 3.0//",
    "-//ietf//dtd html 3.2 final//",
    "-//ietf//dtd html 3.2//",
    "-//ietf//dtd html 3//",
    "-//ietf//dtd html level 0//",
    "-//ietf//dtd html level 1//",
    "-//ietf//dtd html level 2//",
    "-//ietf//dtd html level 3//",
    "-//ietf//dtd html strict level 0//",
    "-//ietf//dtd html strict level 1//",
    "-//ietf//dtd html strict level 2//",
    "-//ietf//dtd html strict level 3//",
    "-//ietf//dtd html strict//",
    "-//ietf//dtd html//",
    "-//metrius//dtd metrius presentational//",
    "-//microsoft//dtd internet explorer 2.0 html strict//",
    "-//microsoft//dtd internet explorer 2.0 html//",
    "-//microsoft//dtd internet explorer 2.0 tables//",
    "-//microsoft//dtd internet explorer 3.0 html strict//",
    "-//microsoft//dtd internet explorer 3.0 html//",
    "-//microsoft//dtd internet explorer 3.0 tables//",
    "-//netscape comm. corp.//dtd html//",
    "-//netscape comm. corp.//dtd strict html//",
    "-//o'reilly and associates//dtd html 2.0//",
    "-//o'reilly and associates//dtd html extended 1.0//",
    "-//o'reilly and associates//dtd html extended relaxed 1.0//",
    "-//softquad software//dtd hotmetal pro 6.0::19990601::extensions to html 4.0//",
    "-//softquad//dtd hotmetal pro 4.0::19971010::extensions to html 4.0//",
    "-//spyglass//dtd html 2.0 extended//",
    "-//sq//dtd html 2.0 hotmetal + extensions//",
    "-//sun microsystems corp.//dtd hotjava html//",
    "-//sun microsystems corp.//dtd hotjava strict html//",
    "-//w3c//dtd html 3 1995-03-24//",
    "-//w3c//dtd html 3.2 draft//",
    "-//w3c//dtd html 3.2 final//",
    "-//w3c//dtd html 3.2//",
    "-//w3c//dtd html 3.2s draft//",
    "-//w3c//dtd html 4.0 frameset//",
    "-//w3c//dtd html 4.0 transitional//",
    "-//w3c//dtd html experimental 19960712//",
    "-//w3c//dtd html experimental 970421//",
    "-//w3c//dtd w3 html//",
    "-//w3o//dtd w3 html 3.0//",
    "-//webtechs//dtd mozilla html 2.0//",
    "-//webtechs//dtd mozilla html//",
];

static QUIRKY_PUBLIC_MATCHES: &[&str] = &[
    "-//w3o//dtd w3 html strict 3.0//en//",
    "-/w3c/dtd html 4.0 transitional/en",
    "html",
];

static QUIRKY_SYSTEM_MATCHES: &[&str] =
    &["http://www.ibm.com/data/dtd/v11/ibmxhtml1-transitional.dtd"];

static LIMITED_QUIRKY_PUBLIC_PREFIXES: &[&str] = &[
    "-//w3c//dtd xhtml 1.0 frameset//",
    "-//w3c//dtd xhtml 1.0 transitional//",
];

static HTML4_PUBLIC_PREFIXES: &[&str] = &[
    "-//w3c//dtd html 4.01 frameset//",
    "-//w3c//dtd html 4.01 transitional//",
];

/// Compares exact retained UTF-16 to ASCII, without scalar conversion or a
/// lowercase allocation. Each inspected provider unit and literal byte is paid.
fn matches(
    text: &Text,
    literal: &str,
    prefix: bool,
    insensitive: bool,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    c.require_document_string(text.value.allocation(), text.value.text())?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    let units = text.value.text().code_units();
    if units.len() < literal.len() || (!prefix && units.len() != literal.len()) {
        return Ok(false);
    }
    // The text is at least as long as the literal, so the pairs are exactly the
    // literal's indices.
    for (&unit, byte) in units.iter().zip(literal.bytes()) {
        c.charge(WorkCategory::StateStep, 1)?;
        c.charge(WorkCategory::InspectByte, 3)?;
        let mut unit = unit;
        if insensitive && (65..=90).contains(&unit) {
            unit |= 0x20;
        }
        let byte = if insensitive {
            byte.to_ascii_lowercase()
        } else {
            byte
        };
        if unit != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn optional_matches(
    text: &Option<Text>,
    literal: &str,
    prefix: bool,
    insensitive: bool,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    match text {
        Some(text) => matches(text, literal, prefix, insensitive, c),
        None => Ok(false),
    }
}

fn any(
    text: &Option<Text>,
    candidates: &[&str],
    prefix: bool,
    c: &mut ParseControl,
) -> Result<bool, ParseStop> {
    for candidate in candidates {
        c.charge(WorkCategory::StateStep, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if optional_matches(text, candidate, prefix, true, c)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Governing §13.2.6.4.1, for a fresh ordinary document whose initial
/// parser-cannot-change-mode flag is false. Hosted/srcdoc admission is separate.
pub(super) fn classify(
    doctype: &Doctype,
    c: &mut ParseControl,
) -> Result<DoctypeDecision, ParseStop> {
    // Validate every retained input owner before reading flags or payloads.
    for text in [&doctype.name, &doctype.public_id, &doctype.system_id] {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(text) = text {
            c.require_document_string(text.value.allocation(), text.value.text())?;
        }
    }
    let html = optional_matches(&doctype.name, "html", false, false, c)?;
    c.charge(WorkCategory::RecordVisit, 2)?;
    let public_missing = doctype.public_id.is_none();
    let system_missing = doctype.system_id.is_none();
    let legacy = optional_matches(&doctype.system_id, "about:legacy-compat", false, false, c)?;
    let error = !html || !public_missing || (!system_missing && !legacy);
    c.charge(WorkCategory::RecordVisit, 1)?;
    c.charge(WorkCategory::InspectByte, 1)?;
    let force = doctype.force_quirks;
    let mode = if force
        || !html
        || any(&doctype.public_id, QUIRKY_PUBLIC_MATCHES, false, c)?
        || any(&doctype.system_id, QUIRKY_SYSTEM_MATCHES, false, c)?
        || any(&doctype.public_id, QUIRKY_PUBLIC_PREFIXES, true, c)?
    {
        HtmlDocumentMode::Quirks
    } else {
        let html4 = any(&doctype.public_id, HTML4_PUBLIC_PREFIXES, true, c)?;
        let empty_system = optional_matches(&doctype.system_id, "", false, false, c)?;
        if html4 && (system_missing || empty_system) {
            HtmlDocumentMode::Quirks
        } else if any(&doctype.public_id, LIMITED_QUIRKY_PUBLIC_PREFIXES, true, c)? || html4 {
            HtmlDocumentMode::LimitedQuirks
        } else {
            HtmlDocumentMode::NoQuirks
        }
    };
    super::write(std::mem::size_of::<DoctypeDecision>(), 2, c)?;
    Ok(DoctypeDecision { error, mode })
}

// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from tree_builder/mod.rs 1605–1692, 1837–1883 and rules.rs
// 1608–1688 at d7232d746d3112f08926d169591576bd12fe2fdf.
// HTML877d controls PI and annotation-xml integration/breakout differences.

use super::{
    KernelError,
    head::{HeadBoundary, HeadCondition, HeadDiagnostic, HeadError, HeadState, HeadStep},
    prologue::Payload,
    tag_sets, write,
};
use crate::{
    DocumentString,
    construction::parser_session::{
        BodyTextKind, BodyTextRuns, ForeignNamespace, ParserNode, partition_body_text,
    },
    parser::{
        control::{ParseControl, StandaloneUnsupported, WorkCategory},
        tokenizer::token::{TagKind, Text},
    },
};
use std::mem::size_of;

const HTML: &str = "http://www.w3.org/1999/xhtml";
const MATHML: &str = "http://www.w3.org/1998/Math/MathML";
const SVG: &str = "http://www.w3.org/2000/svg";

fn insensitive(
    value: &DocumentString,
    literal: &'static str,
    c: &mut ParseControl,
) -> Result<bool, HeadError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if value.len() != literal.len() {
        return Ok(false);
    }
    for (index, byte) in literal.bytes().enumerate() {
        c.charge(WorkCategory::InspectByte, 3)?;
        let unit = value.code_units()[index];
        let lower = if (65..=90).contains(&unit) {
            unit + 32
        } else {
            unit
        };
        if lower != u16::from(byte) {
            return Ok(false);
        }
    }
    Ok(true)
}

impl HeadState<'_> {
    pub(super) fn foreign_namespace(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<ForeignNamespace>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.before.kernel.stack()?.len() == 0 {
            return Ok(None);
        }
        let current = self.before.kernel.current(c)?;
        let name = self
            .before
            .kernel
            .session
            .element_name(c, &current)?
            .ok_or(KernelError::EmptyStack)?;
        if tag_sets::namespace(name, HTML, c)? {
            return Ok(None);
        }
        if tag_sets::namespace(name, MATHML, c)? {
            return Ok(Some(ForeignNamespace::MathMl));
        }
        if tag_sets::namespace(name, SVG, c)? {
            return Ok(Some(ForeignNamespace::Svg));
        }
        Err(c
            .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
            .into())
    }
    fn math_text_point(&self, node: &ParserNode, c: &mut ParseControl) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?;
        Ok(tag_sets::namespace(name, MATHML, c)?
            && tag_sets::local_in(name, &["mi", "mo", "mn", "ms", "mtext"], c)?)
    }
    fn integration_point(
        &self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?;
        if tag_sets::namespace(name, SVG, c)? {
            return Ok(tag_sets::local_in(
                name,
                &["foreignObject", "desc", "title"],
                c,
            )?);
        }
        if !tag_sets::namespace(name, MATHML, c)?
            || !tag_sets::local_in(name, &["annotation-xml"], c)?
        {
            return Ok(false);
        }
        let attrs = self
            .before
            .kernel
            .session
            .element_attributes(c, node)?
            .ok_or(KernelError::EmptyStack)?;
        for attribute in attrs.as_slice() {
            c.charge(WorkCategory::RecordVisit, 3)?;
            if attribute.name().namespace().uri().is_none()
                && tag_sets::ascii_eq(attribute.name().local_name(), "encoding", c)?
            {
                return Ok(insensitive(attribute.value(), "text/html", c)?
                    || insensitive(attribute.value(), "application/xhtml+xml", c)?);
            }
        }
        Ok(false)
    }
    pub(super) fn foreign_dispatch(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        if matches!(payload, Payload::Eof) || self.foreign_namespace(c)?.is_none() {
            return Ok(false);
        }
        let current = self.before.kernel.current(c)?;
        let kind = self.kind(c)?;
        let text = matches!(payload, Payload::Text(_));
        if self.math_text_point(&current, c)?
            && (text
                || (matches!(kind, Some((TagKind::Start, _)))
                    && self.token_in(&["mglyph", "malignmark"], c)?.is_none()))
        {
            return Ok(false);
        }
        let name = self
            .before
            .kernel
            .session
            .element_name(c, &current)?
            .ok_or(KernelError::EmptyStack)?;
        if tag_sets::namespace(name, MATHML, c)?
            && tag_sets::local_in(name, &["annotation-xml"], c)?
            && matches!(kind, Some((TagKind::Start, _)))
            && self.token_in(&["svg"], c)?.is_some()
        {
            return Ok(false);
        }
        if self.integration_point(&current, c)?
            && (text || matches!(kind, Some((TagKind::Start, _))))
        {
            return Ok(false);
        }
        Ok(true)
    }
    pub(super) fn insert_foreign(
        &mut self,
        namespace: ForeignNamespace,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.admit_hidden_capability(c)?;
        // SVG script requires SVG processing facts; HTML Disabled is not its
        // implementation. Retain the untouched token before creation/effects.
        if namespace == ForeignNamespace::Svg && self.token_in(&["script"], c)?.is_some() {
            return self.stop_at(HeadBoundary::Foreign, c);
        }
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::StandaloneCapabilityUnavailable)?
            .capability;
        let node = self
            .before
            .kernel
            .session
            .allocate_foreign_tag_for_creation(
                c,
                input,
                &location.parent,
                namespace,
                &pending.source,
                capability,
            )?;
        self.foreign_xmlns_errors(&node, namespace, c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|s| &s.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        if close {
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|s| &s.capability))?
                .ok_or(KernelError::EmptyStack)?;
        }
        self.finish(c)
    }
    fn foreign_xmlns_errors(
        &mut self,
        node: &ParserNode,
        namespace: ForeignNamespace,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        // At most the two source-defined attributes can produce these errors.
        // Scan separately so each diagnostic precedes the next actual read.
        for (local, expected) in [
            ("xmlns", namespace.uri()),
            ("xlink", "http://www.w3.org/1999/xlink"),
        ] {
            let mut mismatch = false;
            let attrs = self
                .before
                .kernel
                .session
                .element_attributes(c, node)?
                .ok_or(KernelError::EmptyStack)?;
            for attribute in attrs.as_slice() {
                c.charge(WorkCategory::RecordVisit, 2)?;
                if tag_sets::namespace(attribute.name(), "http://www.w3.org/2000/xmlns/", c)?
                    && tag_sets::ascii_eq(attribute.name().local_name(), local, c)?
                    && !tag_sets::ascii_eq(attribute.value(), expected, c)?
                {
                    mismatch = true;
                    break;
                }
            }
            if mismatch {
                self.emit(HeadCondition::ForeignNamespaceMismatch, c)?;
            }
        }
        Ok(())
    }
    fn foreign_characters(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.body_runs.is_none() {
            let Payload::Text(input) = &mut self
                .pending
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .payload
            else {
                return Err(HeadError::MissingInput);
            };
            write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
            self.body_runs = Some(partition_body_text(c, input)?);
        }
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.body_run.is_none() {
                write(
                    size_of::<Option<crate::construction::parser_session::TextRun>>(),
                    1,
                    c,
                )?;
                self.body_run = self
                    .body_runs
                    .as_mut()
                    .expect("retained runs")
                    .take_next(c)?;
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            let Some(run) = self.body_run.as_ref() else {
                break;
            };
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            let kind = run.kind;
            if kind == BodyTextKind::Null {
                let mode = self.mode(c)?;
                for origin in self
                    .body_run
                    .as_ref()
                    .expect("retained run")
                    .text
                    .origins
                    .as_slice()
                {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    write(size_of::<HeadDiagnostic>(), 3, c)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.diagnostics.try_push(
                        c,
                        HeadDiagnostic {
                            condition: HeadCondition::NullInForeign,
                            mode,
                            source: origin.source.clone(),
                        },
                    )?;
                }
            }
            write(
                size_of::<Option<crate::construction::parser_session::TextRun>>()
                    + size_of::<Option<Text>>(),
                2,
                c,
            )?;
            self.whitespace_pending = self.body_run.take().map(|run| run.text);
            if kind == BodyTextKind::Null {
                self.before
                    .kernel
                    .session
                    .replace_foreign_nulls(c, &mut self.whitespace_pending)?;
            }
            self.before.kernel.insert_text(
                &mut self.whitespace_pending,
                None,
                c,
                self.script.as_ref().map(|s| &s.capability),
            )?;
            if kind == BodyTextKind::Other {
                self.frameset(false, c)?;
            }
        }
        self.body_runs.as_mut().expect("retained runs").release(c)?;
        write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
        self.body_runs = None;
        self.finish(c)
    }
    /// None delegates/reprocesses the SAME token in the current HTML mode;
    /// no new token or dispatcher override is fabricated.
    pub(super) fn foreign_rules(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        {
            Payload::Text(_) => return Ok(Some(self.foreign_characters(c)?)),
            Payload::Comment(_) | Payload::Pi(_) => return Ok(Some(self.ordinary_node(c)?)),
            Payload::Doctype(_) => {
                self.emit(HeadCondition::UnexpectedDoctype, c)?;
                return Ok(Some(self.discard(c)?));
            }
            Payload::Eof => return Ok(None),
            Payload::Tag(_) => {}
        }
        let (kind, close) = self.kind(c)?.ok_or(HeadError::MissingInput)?;
        let breakout = if kind == TagKind::Start {
            self.token_in(
                &[
                    "b",
                    "big",
                    "blockquote",
                    "body",
                    "br",
                    "center",
                    "code",
                    "dd",
                    "div",
                    "dl",
                    "dt",
                    "em",
                    "embed",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "h5",
                    "h6",
                    "head",
                    "hr",
                    "i",
                    "img",
                    "li",
                    "listing",
                    "menu",
                    "meta",
                    "nobr",
                    "ol",
                    "p",
                    "pre",
                    "ruby",
                    "s",
                    "small",
                    "span",
                    "strong",
                    "strike",
                    "sub",
                    "sup",
                    "table",
                    "tt",
                    "u",
                    "ul",
                    "var",
                ],
                c,
            )?
            .is_some()
                || (self.token_in(&["font"], c)?.is_some() && self.foreign_font_breakout(c)?)
        } else {
            self.token_in(&["br", "p"], c)?.is_some()
        };
        if breakout {
            self.emit(HeadCondition::UnexpectedForeignToken, c)?;
            loop {
                c.charge(WorkCategory::StateStep, 1)?;
                let current = self.before.kernel.current(c)?;
                if self.foreign_namespace(c)?.is_none()
                    || self.math_text_point(&current, c)?
                    || self.integration_point(&current, c)?
                {
                    break;
                }
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|s| &s.capability))?
                    .ok_or(KernelError::EmptyStack)?;
            }
            return Ok(None);
        }
        if kind == TagKind::Start {
            let namespace = self.foreign_namespace(c)?.ok_or(KernelError::EmptyStack)?;
            return Ok(Some(self.insert_foreign(namespace, close, c)?));
        }
        let current = self.before.kernel.current(c)?;
        if self.foreign_namespace(c)? == Some(ForeignNamespace::Svg)
            && self.token_in(&["script"], c)?.is_some()
        {
            let name = self
                .before
                .kernel
                .session
                .element_name(c, &current)?
                .ok_or(KernelError::EmptyStack)?;
            if tag_sets::local_in(name, &["script"], c)? {
                return Ok(Some(self.stop_at(HeadBoundary::Foreign, c)?));
            }
        }
        if !self.foreign_end_matches(&current, c)? {
            self.emit(HeadCondition::UnexpectedForeignToken, c)?;
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        let mut index = self.before.kernel.stack()?.len() - 1;
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            if index == 0 {
                return Ok(Some(self.discard(c)?));
            }
            let node = self.before.kernel.at(index, c)?;
            if self.foreign_end_matches(&node, c)? {
                loop {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if self.before.kernel.stack()?.len() <= index {
                        break;
                    }
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|s| &s.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                }
                return Ok(Some(self.discard(c)?));
            }
            index -= 1;
            let node = self.before.kernel.at(index, c)?;
            let name = self
                .before
                .kernel
                .session
                .element_name(c, &node)?
                .ok_or(KernelError::EmptyStack)?;
            if tag_sets::namespace(name, HTML, c)? {
                return Ok(None);
            }
        }
    }
    fn foreign_font_breakout(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        for attribute in input
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .attributes
            .as_slice()
        {
            c.charge(WorkCategory::RecordVisit, 1)?;
            for name in ["color", "face", "size"] {
                if tag_sets::ascii_eq(attribute.name.value.text(), name, c)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    fn foreign_end_matches(
        &self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        let name = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::EmptyStack)?
            .local_name();
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let token = input
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .name
            .value
            .text();
        c.charge(WorkCategory::RecordVisit, 2)?;
        if name.len() != token.len() {
            return Ok(false);
        }
        for index in 0..name.len() {
            c.charge(WorkCategory::InspectByte, 4)?;
            let unit = name.code_units()[index];
            let lower = if (65..=90).contains(&unit) {
                unit + 32
            } else {
                unit
            };
            if lower != token.code_units()[index] {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

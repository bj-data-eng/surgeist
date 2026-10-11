// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from selected ordinary body rows in tree_builder/rules.rs:
// 356-541,558-602,616-630,693-731,796-813,842-873,880-892,947-1004,
// 1466-1493,1563-1583; and tree_builder/mod.rs:1043-1223,1555-1585.
// Formatting, forms, table/foreign creation and frameset transitions are
// retained boundaries rather than imported implementations. Origin: html5ever
// d7232d746d3112f08926d169591576bd12fe2fdf. HTML
// 877d6146487f15b15bc8dd0e3c625063a37b456a governs PI, exact units,
// scope/special/implied/EOF sets and the actual Disabled scripting rules.

use super::{
    KernelError, Scope, copy_node,
    head::{
        HeadBoundary, HeadCondition, HeadDiagnostic, HeadError, HeadState, HeadStep, InsertionMode,
    },
    prologue::Payload,
    tag_sets::{self, NodeSet},
    write,
};
use crate::{
    HtmlSource,
    construction::parser_session::{
        BodyTextKind, BodyTextRuns, ParserNode, TextRun, partition_body_text, split_first_lf,
    },
    parser::{
        control::{ParseControl, WorkCategory},
        tokenizer::{
            states::{Raw, State},
            token::{TagKind, Text},
        },
    },
};
use std::mem::size_of;

const BLOCK: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "center",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "figcaption",
    "figure",
    "footer",
    "header",
    "hgroup",
    "main",
    "menu",
    "nav",
    "ol",
    "p",
    "search",
    "section",
    "summary",
    "ul",
];
const BLOCK_END: &[&str] = &[
    "address",
    "article",
    "aside",
    "blockquote",
    "button",
    "center",
    "details",
    "dialog",
    "dir",
    "div",
    "dl",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "header",
    "hgroup",
    "listing",
    "main",
    "menu",
    "nav",
    "ol",
    "pre",
    "search",
    "section",
    "select",
    "summary",
    "ul",
];
const HEAD: &[&str] = &[
    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style", "template",
    "title",
];
const HEADING: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];

impl HeadState<'_> {
    fn token_in(
        &self,
        names: &[&'static str],
        c: &mut ParseControl,
    ) -> Result<Option<&'static str>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(None);
        };
        let tag = input.as_ref().ok_or(HeadError::MissingInput)?;
        c.require_document_string(tag.name.value.allocation(), tag.name.value.text())?;
        for &name in names {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if tag_sets::ascii_eq(tag.name.value.text(), name, c)? {
                return Ok(Some(name));
            }
        }
        Ok(None)
    }
    fn kind(&self, c: &mut ParseControl) -> Result<Option<(TagKind, bool)>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(None);
        };
        let tag = input.as_ref().ok_or(HeadError::MissingInput)?;
        c.charge(
            WorkCategory::InspectByte,
            (size_of::<TagKind>() + size_of::<bool>()) as u64,
        )?;
        Ok(Some((tag.kind, tag.self_closing)))
    }
    fn frameset(&mut self, value: bool, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<bool>(), 1, c)?;
        self.frameset_ok = value;
        Ok(())
    }
    fn ignored_text(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.whitespace_pending.is_none() {
            return Ok(());
        }
        write(size_of::<Option<Text>>(), 1, c)?;
        let text = self
            .whitespace_pending
            .take()
            .expect("retained omitted input");
        if let Err((stop, text)) = self.before.whitespace.try_push_recover(c, text) {
            self.whitespace_pending = Some(text);
            return Err(stop.into());
        }
        Ok(())
    }
    fn first_lf(&mut self, c: &mut ParseControl) -> Result<bool, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
        if !self.ignore_first_lf {
            return Ok(false);
        }
        write(size_of::<bool>(), 1, c)?;
        self.ignore_first_lf = false;
        let Payload::Text(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        write(size_of::<Option<Text>>(), 1, c)?;
        self.whitespace_pending = split_first_lf(c, input)?;
        let empty = input.is_none();
        self.ignored_text(c)?;
        Ok(empty)
    }
    fn body_text(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
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
            write(size_of::<Option<TextRun>>(), 1, c)?;
            self.body_run = self
                .body_runs
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .take_next(c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let mode = self.mode(c)?;
            let Some(run) = self.body_run.as_mut() else {
                break;
            };
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            let kind = run.kind;
            if kind == BodyTextKind::Null {
                // Every omitted NULL retains its real origin; a diagnostic uses
                // that exact range rather than the enclosing text-token span.
                c.charge(WorkCategory::RecordVisit, 1)?;
                for origin in run.text.origins.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    write(size_of::<HeadDiagnostic>(), 3, c)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.diagnostics.try_push(
                        c,
                        HeadDiagnostic {
                            condition: HeadCondition::NullInBody,
                            mode,
                            source: origin.source.clone(),
                        },
                    )?;
                }
                write(
                    size_of::<Option<TextRun>>() + size_of::<Option<Text>>(),
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                self.ignored_text(c)?;
            } else {
                let other = kind == BodyTextKind::Other;
                write(
                    size_of::<Option<TextRun>>() + size_of::<Option<Text>>(),
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                // This fresh family has consumed no formatting opener/marker;
                // classification suspends before all such source rows.
                self.before
                    .kernel
                    .insert_text(&mut self.whitespace_pending, None, c)?;
                if other {
                    self.frameset(false, c)?;
                }
            }
        }
        self.body_runs
            .as_mut()
            .expect("retained partition")
            .release(c)?;
        write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
        self.body_runs = None;
        self.finish(c)
    }
    fn close_p(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.before.kernel.implied_ends(Some("p"), c)?;
        let current = self.before.kernel.current(c)?;
        if !self.before.kernel.named(&current, "p", c)? {
            self.emit(HeadCondition::UnexpectedOpenElement, c)?;
        }
        loop {
            let node = self.before.kernel.pop(c)?.ok_or(KernelError::EmptyStack)?;
            if self.before.kernel.named(&node, "p", c)? {
                return Ok(());
            }
        }
    }
    fn close_p_in_scope(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if self.before.kernel.scoped_name("p", Scope::Button, c)? {
            self.close_p(c)?;
        }
        Ok(())
    }
    fn implicit_body(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        let node = self.before.kernel.session.allocate_implicit_body(c)?;
        self.before.kernel.insert_element_at(&node, &location, c)?;
        self.before.kernel.push(&node, c)?;
        self.set_mode(InsertionMode::InBody, c)
    }
    fn body_end_check(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in 0..self.before.kernel.stack()?.len() {
            let node = self.before.kernel.at(index, c)?;
            if !self.before.kernel.node_set(&node, NodeSet::BodyEnd, c)? {
                self.emit(HeadCondition::UnexpectedOpenElement, c)?;
                break;
            }
        }
        Ok(())
    }
    fn head_delegation(
        &mut self,
        late: bool,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        if self.token_in(&["template"], c)?.is_some() {
            return self.stop_at(HeadBoundary::Template, c);
        }
        if late && self.token_in(&["script"], c)?.is_some() {
            // The sealed owner must still qualify before even the temporary
            // stack entry or delegated-row diagnostic is changed.
            self.validate_script_capability(c)?;
        }
        if late {
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let head = self.head.as_ref().ok_or(HeadError::MissingInput)?;
            let node = copy_node(head, c)?;
            write(size_of::<Option<ParserNode>>(), 1, c)?;
            self.temporary_head = Some(node);
            self.before
                .kernel
                .push(self.temporary_head.as_ref().expect("stored head"), c)?;
        }
        let step = if self.token_in(&["script"], c)?.is_some() {
            self.script_start(close, c)?
        } else if self.token_in(&["title"], c)?.is_some() {
            self.raw_text(Raw::Rcdata, close, c)?
        } else if self.token_in(&["style", "noframes"], c)?.is_some() {
            self.raw_text(Raw::Rawtext, close, c)?
        } else {
            self.insert_tag(c)?;
            self.before.kernel.pop(c)?.ok_or(KernelError::EmptyStack)?;
            self.finish(c)?
        };
        if late {
            self.before.kernel.remove_stack_identity(
                self.temporary_head
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?,
                c,
            )?;
            write(size_of::<Option<ParserNode>>(), 1, c)?;
            self.temporary_head = None;
        }
        Ok(step)
    }
    fn end_template(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        if self.before.kernel.contains_html_template(c)? {
            return self.stop_at(HeadBoundary::Template, c);
        }
        self.emit(HeadCondition::UnexpectedEndTag, c)?;
        self.discard(c)
    }
    fn ordinary_at(
        &mut self,
        parent: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let node = match &mut pending.payload {
            Payload::Comment(input) => self.before.kernel.session.allocate_comment(c, input)?,
            Payload::Pi(input) => self
                .before
                .kernel
                .session
                .allocate_processing_instruction(c, input)?,
            _ => return Err(HeadError::MissingInput),
        };
        self.before.kernel.session.insert(c, parent, &node, None)?;
        self.finish(c)
    }
    fn excluded(
        &self,
        kind: TagKind,
        c: &mut ParseControl,
    ) -> Result<Option<HeadBoundary>, HeadError> {
        if kind == TagKind::End {
            if self
                .token_in(
                    &[
                        "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                        "strong", "tt", "u", "applet", "marquee", "object",
                    ],
                    c,
                )?
                .is_some()
            {
                return Ok(Some(HeadBoundary::Formatting));
            }
            if self.token_in(&["form"], c)?.is_some() {
                return Ok(Some(HeadBoundary::Form));
            }
            return Ok(None);
        }
        for (names, boundary) in [
            (&["template"][..], HeadBoundary::Template),
            (
                &[
                    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                    "strong", "tt", "u", "applet", "marquee", "object",
                ][..],
                HeadBoundary::Formatting,
            ),
            (
                &["form", "fieldset", "button", "input", "textarea", "output"][..],
                HeadBoundary::Form,
            ),
            (&["select", "option", "optgroup"][..], HeadBoundary::Select),
            (
                &[
                    "table", "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead",
                    "tr",
                ][..],
                HeadBoundary::Table,
            ),
            (&["frameset", "frame"][..], HeadBoundary::Frameset),
            (&["math", "svg"][..], HeadBoundary::Foreign),
        ] {
            if self.token_in(names, c)?.is_some() {
                return Ok(Some(boundary));
            }
        }
        Ok(None)
    }
    pub(super) fn body_advance(
        &mut self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        if self.first_lf(c)? {
            return Ok(Some(self.finish(c)?));
        }
        let mode = self.mode(c)?;
        let kind = self.kind(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        if matches!(payload, Payload::Eof) {
            if mode == InsertionMode::AfterHead {
                self.implicit_body(c)?;
                return Ok(None);
            }
            if mode == InsertionMode::InBody {
                self.body_end_check(c)?;
            }
            return Ok(Some(self.stop_at(HeadBoundary::ConstructionEof, c)?));
        }
        if matches!(payload, Payload::Comment(_) | Payload::Pi(_)) {
            return Ok(Some(match mode {
                InsertionMode::AfterBody => {
                    let html = self.before.kernel.first(c)?;
                    self.ordinary_at(&html, c)?
                }
                InsertionMode::AfterAfterBody => {
                    let document = self.before.kernel.session.root(c)?;
                    self.ordinary_at(&document, c)?
                }
                _ => self.ordinary_node(c)?,
            }));
        }
        if matches!(payload, Payload::Doctype(_)) {
            self.emit(HeadCondition::UnexpectedDoctype, c)?;
            return Ok(Some(self.discard(c)?));
        }
        if kind.is_some_and(|(kind, _)| kind == TagKind::Start)
            && self.token_in(&["html"], c)?.is_some()
        {
            return Ok(Some(self.html(kind.expect("tag").1, c)?));
        }
        if mode == InsertionMode::AfterHead {
            if self.whitespace(c)? {
                return Ok(Some(self.finish(c)?));
            }
            if let Some((TagKind::Start, close)) = kind {
                if self.token_in(&["body"], c)?.is_some() {
                    self.insert_tag(c)?;
                    self.frameset(false, c)?;
                    self.set_mode(InsertionMode::InBody, c)?;
                    if close {
                        self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
                    }
                    return Ok(Some(self.finish(c)?));
                }
                if self.token_in(HEAD, c)?.is_some() {
                    return Ok(Some(self.head_delegation(true, close, c)?));
                }
                if self.token_in(&["frameset"], c)?.is_some() {
                    return Ok(Some(self.stop_at(HeadBoundary::Frameset, c)?));
                }
                if self.token_in(&["head"], c)?.is_some() {
                    self.emit(HeadCondition::UnexpectedHead, c)?;
                    return Ok(Some(self.discard(c)?));
                }
            }
            if matches!(kind, Some((TagKind::End, _))) {
                if self.token_in(&["template"], c)?.is_some() {
                    return Ok(Some(self.end_template(c)?));
                }
                if self.token_in(&["body", "html", "br"], c)?.is_none() {
                    self.emit(HeadCondition::UnexpectedEndTag, c)?;
                    return Ok(Some(self.discard(c)?));
                }
            }
            self.implicit_body(c)?;
            return Ok(None);
        }
        if matches!(
            mode,
            InsertionMode::AfterBody | InsertionMode::AfterAfterBody
        ) {
            if self.whitespace(c)? {
                return Ok(Some(self.finish(c)?));
            }
            if mode == InsertionMode::AfterBody
                && matches!(kind, Some((TagKind::End, _)))
                && self.token_in(&["html"], c)?.is_some()
            {
                self.set_mode(InsertionMode::AfterAfterBody, c)?;
                return Ok(Some(self.discard(c)?));
            }
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            self.set_mode(InsertionMode::InBody, c)?;
            return Ok(None);
        }
        // Data character runs are split by the actual owning Text producer.
        c.charge(WorkCategory::RecordVisit, 1)?;
        if matches!(
            self.pending
                .as_ref()
                .ok_or(HeadError::MissingInput)?
                .payload,
            Payload::Text(_)
        ) {
            return Ok(Some(self.body_text(c)?));
        }
        if matches!(kind, Some((TagKind::End, _))) && self.token_in(&["template"], c)?.is_some() {
            return Ok(Some(self.end_template(c)?));
        }
        if matches!(kind, Some((TagKind::Start, _))) && self.token_in(&["frameset"], c)?.is_some() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let body_present = if self.before.kernel.stack()?.len() > 1 {
                let body = self.before.kernel.at(1, c)?;
                self.before.kernel.named(&body, "body", c)?
            } else {
                false
            };
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
            if body_present && self.frameset_ok {
                return Ok(Some(self.stop_at(HeadBoundary::Frameset, c)?));
            }
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            return Ok(Some(self.discard(c)?));
        }
        if matches!(kind, Some((TagKind::Start, _)))
            && self
                .token_in(
                    &[
                        "caption", "col", "colgroup", "frame", "head", "tbody", "td", "tfoot",
                        "th", "thead", "tr",
                    ],
                    c,
                )?
                .is_some()
        {
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            return Ok(Some(self.discard(c)?));
        }
        if let Some(boundary) = self.excluded(kind.ok_or(HeadError::MissingInput)?.0, c)? {
            return Ok(Some(self.stop_at(boundary, c)?));
        }
        let Some((kind, close)) = kind else {
            return Err(HeadError::MissingInput);
        };
        if kind == TagKind::Start {
            if self.token_in(HEAD, c)?.is_some() {
                return Ok(Some(self.head_delegation(false, close, c)?));
            }
            return self.body_start(source, close, c);
        }
        self.body_end(c)
    }
    fn body_start(
        &mut self,
        source: &HtmlSource,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        if self.token_in(&["body"], c)?.is_some() {
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.before.kernel.stack()?.len() > 1
                && !self.before.kernel.contains_html_template(c)?
            {
                let body = self.before.kernel.at(1, c)?;
                if self.before.kernel.named(&body, "body", c)? {
                    self.frameset(false, c)?;
                    let Payload::Tag(input) = &mut self
                        .pending
                        .as_mut()
                        .ok_or(HeadError::MissingInput)?
                        .payload
                    else {
                        unreachable!()
                    };
                    self.before
                        .kernel
                        .session
                        .add_body_attributes_if_missing(c, &body, input)?;
                    return Ok(Some(self.finish(c)?));
                }
            }
            return Ok(Some(self.discard(c)?));
        }
        if self.token_in(&["head"], c)?.is_some() {
            self.emit(HeadCondition::UnexpectedHead, c)?;
            return Ok(Some(self.discard(c)?));
        }
        if self.token_in(BLOCK, c)?.is_some() {
            self.close_p_in_scope(c)?;
        } else if self.token_in(HEADING, c)?.is_some() {
            self.close_p_in_scope(c)?;
            let current = self.before.kernel.current(c)?;
            if self.before.kernel.node_set(&current, NodeSet::Heading, c)? {
                self.emit(HeadCondition::UnexpectedOpenElement, c)?;
                self.before.kernel.pop(c)?;
            }
        } else if self.token_in(&["pre", "listing"], c)?.is_some() {
            self.close_p_in_scope(c)?;
            self.insert_tag(c)?;
            write(size_of::<bool>(), 1, c)?;
            self.ignore_first_lf = true;
            self.frameset(false, c)?;
            if close {
                self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
            }
            return Ok(Some(self.finish(c)?));
        } else if let Some(name) = self.token_in(&["li", "dd", "dt"], c)? {
            self.frameset(false, c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            for index in (0..self.before.kernel.stack()?.len()).rev() {
                let node = self.before.kernel.at(index, c)?;
                let matched = if name == "li" {
                    self.before.kernel.named(&node, "li", c)?
                } else {
                    self.before.kernel.named(&node, "dd", c)?
                        || self.before.kernel.named(&node, "dt", c)?
                };
                if matched {
                    let except = if self.before.kernel.named(&node, "li", c)? {
                        "li"
                    } else if self.before.kernel.named(&node, "dd", c)? {
                        "dd"
                    } else {
                        "dt"
                    };
                    self.before.kernel.implied_ends(Some(except), c)?;
                    let current = self.before.kernel.current(c)?;
                    if !self.before.kernel.session.same_node(c, &current, &node)? {
                        self.emit(HeadCondition::UnexpectedOpenElement, c)?;
                    }
                    self.before.kernel.pop_through(&node, c)?;
                    break;
                }
                if self.before.kernel.node_set(&node, NodeSet::Special, c)?
                    && !self.before.kernel.named(&node, "address", c)?
                    && !self.before.kernel.named(&node, "div", c)?
                    && !self.before.kernel.named(&node, "p", c)?
                {
                    break;
                }
            }
            self.close_p_in_scope(c)?;
        } else if self.token_in(&["plaintext"], c)?.is_some() {
            self.close_p_in_scope(c)?;
            self.insert_tag(c)?;
            self.tokenizer
                .as_mut()
                .ok_or(HeadError::Consumed)?
                .set_state(State::Plaintext, c)?;
            if close {
                self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
            }
            return Ok(Some(self.finish(c)?));
        } else if self.token_in(&["xmp", "iframe", "noembed"], c)?.is_some() {
            if self.token_in(&["xmp"], c)?.is_some() {
                self.close_p_in_scope(c)?;
            }
            if self.token_in(&["xmp", "iframe"], c)?.is_some() {
                self.frameset(false, c)?;
            }
            return Ok(Some(self.raw_text(Raw::Rawtext, close, c)?));
        } else if self.token_in(&["image"], c)?.is_some() {
            self.emit(HeadCondition::UnexpectedBodyToken, c)?;
            let Payload::Tag(input) = &mut self
                .pending
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .payload
            else {
                unreachable!()
            };
            self.before
                .kernel
                .session
                .recover_image_tag(c, source, input)?;
            return Ok(None);
        } else if self
            .token_in(
                &[
                    "area", "br", "embed", "img", "keygen", "wbr", "param", "source", "track", "hr",
                ],
                c,
            )?
            .is_some()
        {
            let affects = self
                .token_in(&["area", "br", "embed", "img", "keygen", "wbr", "hr"], c)?
                .is_some();
            if self.token_in(&["hr"], c)?.is_some() {
                self.close_p_in_scope(c)?;
            }
            self.insert_tag(c)?;
            self.before.kernel.pop(c)?;
            if affects {
                self.frameset(false, c)?;
            }
            return Ok(Some(self.finish(c)?));
        } else if let Some(name) = self.token_in(&["rb", "rtc", "rp", "rt"], c)?
            && self.before.kernel.scoped_name("ruby", Scope::Default, c)?
        {
            self.before.kernel.implied_ends(
                if matches!(name, "rp" | "rt") {
                    Some("rtc")
                } else {
                    None
                },
                c,
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, "ruby", c)?
                && !(matches!(name, "rp" | "rt") && self.before.kernel.named(&current, "rtc", c)?)
            {
                self.emit(HeadCondition::UnexpectedOpenElement, c)?;
            }
        }
        self.insert_tag(c)?;
        if close {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        Ok(Some(self.finish(c)?))
    }
    fn token_matches_node(
        &self,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        let q = self
            .before
            .kernel
            .session
            .element_name(c, node)?
            .ok_or(KernelError::NotElement)?;
        if !tag_sets::namespace(q, "http://www.w3.org/1999/xhtml", c)? {
            return Ok(false);
        }
        c.charge(WorkCategory::RecordVisit, 2)?;
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let tag = input.as_ref().ok_or(HeadError::MissingInput)?;
        Ok(tag_sets::equal_units(
            q.local_name(),
            tag.name.value.text(),
            c,
        )?)
    }
    fn body_end(&mut self, c: &mut ParseControl) -> Result<Option<HeadStep>, HeadError> {
        if let Some(name) = self.token_in(&["body", "html"], c)? {
            if !self.before.kernel.scoped_name("body", Scope::Default, c)? {
                self.emit(HeadCondition::UnexpectedEndTag, c)?;
                return Ok(Some(self.discard(c)?));
            }
            self.body_end_check(c)?;
            self.set_mode(InsertionMode::AfterBody, c)?;
            return if name == "html" {
                Ok(None)
            } else {
                Ok(Some(self.discard(c)?))
            };
        }
        if self.token_in(&["br"], c)?.is_some() {
            self.emit(HeadCondition::UnexpectedEndTag, c)?;
            let Payload::Tag(input) = &mut self
                .pending
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .payload
            else {
                unreachable!()
            };
            self.before.kernel.session.recover_br_end_tag(c, input)?;
            return Ok(None);
        }
        if self.token_in(&["p"], c)?.is_some() {
            if !self.before.kernel.scoped_name("p", Scope::Button, c)? {
                self.emit(HeadCondition::UnexpectedEndTag, c)?;
                let location = self.before.kernel.adjusted_location(None, c)?;
                let node = self.before.kernel.session.allocate_implicit_p(c)?;
                self.before.kernel.insert_element_at(&node, &location, c)?;
                self.before.kernel.push(&node, c)?;
            }
            self.close_p(c)?;
            return Ok(Some(self.discard(c)?));
        }
        if let Some(name) = self
            .token_in(BLOCK_END, c)?
            .or(self.token_in(&["li", "dd", "dt"], c)?)
        {
            if !self.before.kernel.scoped_name(
                name,
                if name == "li" {
                    Scope::ListItem
                } else {
                    Scope::Default
                },
                c,
            )? {
                self.emit(HeadCondition::UnexpectedEndTag, c)?;
                return Ok(Some(self.discard(c)?));
            }
            self.before.kernel.implied_ends(
                if matches!(name, "li" | "dd" | "dt") {
                    Some(name)
                } else {
                    None
                },
                c,
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, name, c)? {
                self.emit(HeadCondition::UnexpectedOpenElement, c)?;
            }
            loop {
                let node = self.before.kernel.pop(c)?.ok_or(KernelError::EmptyStack)?;
                if self.before.kernel.named(&node, name, c)? {
                    break;
                }
            }
            return Ok(Some(self.discard(c)?));
        }
        if self.token_in(HEADING, c)?.is_some() {
            let mut target = None;
            c.charge(WorkCategory::RecordVisit, 1)?;
            for index in (0..self.before.kernel.stack()?.len()).rev() {
                let node = self.before.kernel.at(index, c)?;
                if self.before.kernel.node_set(&node, NodeSet::Heading, c)? {
                    write(size_of::<Option<ParserNode>>(), 1, c)?;
                    target = Some(node);
                    break;
                }
                let q = self
                    .before
                    .kernel
                    .session
                    .element_name(c, &node)?
                    .ok_or(KernelError::NotElement)?;
                if tag_sets::scope_boundary(q, Scope::Default, c)? {
                    break;
                }
            }
            let Some(target) = target else {
                self.emit(HeadCondition::UnexpectedEndTag, c)?;
                return Ok(Some(self.discard(c)?));
            };
            self.before.kernel.implied_ends(None, c)?;
            let current = self.before.kernel.current(c)?;
            if !self.token_matches_node(&current, c)? {
                self.emit(HeadCondition::UnexpectedOpenElement, c)?;
            }
            self.before.kernel.pop_through(&target, c)?;
            return Ok(Some(self.discard(c)?));
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.before.kernel.stack()?.len()).rev() {
            let node = self.before.kernel.at(index, c)?;
            if self.token_matches_node(&node, c)? {
                loop {
                    let current = self.before.kernel.current(c)?;
                    if !self.before.kernel.node_set(&current, NodeSet::Implied, c)?
                        || self.token_matches_node(&current, c)?
                    {
                        break;
                    }
                    self.before.kernel.pop(c)?;
                }
                let current = self.before.kernel.current(c)?;
                if !self.before.kernel.session.same_node(c, &node, &current)? {
                    self.emit(HeadCondition::UnexpectedOpenElement, c)?;
                }
                self.before.kernel.pop_through(&node, c)?;
                return Ok(Some(self.discard(c)?));
            }
            if self.before.kernel.node_set(&node, NodeSet::Special, c)? {
                self.emit(HeadCondition::UnexpectedEndTag, c)?;
                return Ok(Some(self.discard(c)?));
            }
        }
        Err(KernelError::EmptyStack.into())
    }
    pub(super) fn release_body(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if c.stop().is_none() {
            write(
                size_of::<Option<TextRun>>() + size_of::<Option<ParserNode>>(),
                2,
                c,
            )?;
        }
        if let Some(run) = self.body_run.take() {
            run.text.release(c)?;
        }
        if let Some(runs) = &mut self.body_runs {
            runs.release(c)?;
        }
        if c.stop().is_none() {
            write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
        }
        self.body_runs = None;
        self.temporary_head = None;
        Ok(())
    }
}

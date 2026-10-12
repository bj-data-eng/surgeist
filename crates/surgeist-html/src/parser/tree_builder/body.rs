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
// 356-541,558-602,616-630,693-731,786-813,842-873,880-892,947-1004,
// 1466-1493,1563-1583, plus form/control rows508-518,543-558,604-665,765-784,
// and textarea874-878, Hidden input815-840, MathML/SVG entry971-979;
// and tree_builder/mod.rs:1043-1223,1555-1585.
// Other resettable form controls and SVG script creation are
// retained boundaries rather than imported implementations. Origin: html5ever
// d7232d746d3112f08926d169591576bd12fe2fdf. HTML
// 877d6146487f15b15bc8dd0e3c625063a37b456a governs PI, exact units,
// scope/special/implied/EOF sets and the actual Disabled scripting rules.

use super::{
    KernelError, Scope, copy_node,
    formatting::FORMATTING,
    head::{HeadBoundary, HeadCondition, HeadError, HeadState, HeadStep, InsertionMode},
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
    "fieldset",
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
    pub(super) fn hidden_input(&self, c: &mut ParseControl) -> Result<bool, HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        let Payload::Tag(input) = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Ok(false);
        };
        Ok(crate::construction::parser_session::hidden_input_tag(
            c,
            input.as_ref().ok_or(HeadError::MissingInput)?,
        )?)
    }
    pub(super) fn admit_hidden_capability(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        match self.validate_standalone_capability(c) {
            Err(HeadError::StandaloneCapabilityUnavailable) => Err(c
                .stop_unsupported_standalone(
                    crate::parser::control::StandaloneUnsupported::Provenance,
                )
                .into()),
            other => other,
        }
    }
    pub(super) fn token_in(
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
    pub(super) fn kind(&self, c: &mut ParseControl) -> Result<Option<(TagKind, bool)>, HeadError> {
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
    pub(super) fn frameset(&mut self, value: bool, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<bool>(), 1, c)?;
        self.frameset_ok = value;
        Ok(())
    }
    pub(super) fn ignored_text(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
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
    pub(super) fn first_lf(&mut self, c: &mut ParseControl) -> Result<bool, HeadError> {
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

                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.before.kernel.session.evidence(c)?.append(c, crate::construction::parser_session::evidence::DiagnosticKind::Tree(HeadCondition::NullInBody), crate::construction::parser_session::evidence::DiagnosticContext::Tree(mode), crate::construction::parser_session::evidence::DiagnosticRule { anchor: "parsing-main-inbody", branch: "null-character/ignore" }, crate::construction::parser_session::evidence::DiagnosticLocationInput { source: &origin.source, role: crate::construction::parser_session::evidence::DiagnosticLocationRole::TextOrigin, token: Some(crate::construction::parser_session::evidence::DiagnosticToken::Text) })?;
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
                self.body_character(other, c)?;
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
    pub(super) fn body_character(
        &mut self,
        other: bool,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.reconstruct_formatting(c)?;
        self.before.kernel.insert_text(
            &mut self.whitespace_pending,
            None,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        if other {
            self.frameset(false, c)?;
        }
        Ok(())
    }
    fn close_p(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.before.kernel.implied_ends(
            Some("p"),
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        let current = self.before.kernel.current(c)?;
        if !self.before.kernel.named(&current, "p", c)? {
            self.emit(
                HeadCondition::UnexpectedOpenElement,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "p-close/current-node-mismatch",
                },
                c,
            )?;
        }
        loop {
            let node = self
                .before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
            if self.before.kernel.named(&node, "p", c)? {
                return Ok(());
            }
        }
    }
    pub(super) fn close_p_in_scope(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if self.before.kernel.scoped_name("p", Scope::Button, c)? {
            self.close_p(c)?;
        }
        Ok(())
    }
    fn implicit_body(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        let location = self.before.kernel.adjusted_location(None, c)?;
        let node = self.before.kernel.session.allocate_implicit_body(c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        self.set_mode(InsertionMode::InBody, c)
    }
    fn body_end_check(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in 0..self.before.kernel.stack()?.len() {
            let node = self.before.kernel.at(index, c)?;
            if !self.before.kernel.node_set(&node, NodeSet::BodyEnd, c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "body-end-or-eof/disallowed-open-element",
                    },
                    c,
                )?;
                break;
            }
        }
        Ok(())
    }
    pub(super) fn head_delegation(
        &mut self,
        late: bool,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        // The pending token is now processed using the InHead rules, even
        // when its actual insertion mode belongs to the delegating caller.
        self.record_trigger(
            crate::construction::parser_session::provenance::ParserRule::Tree(
                InsertionMode::InHead,
            ),
            c,
        )?;
        if self.token_in(&["template"], c)?.is_some() {
            return self.template_start(c);
        }
        if late && self.token_in(&["script"], c)?.is_some() {
            // The sealed owner must still qualify before even the temporary
            // stack entry or delegated-row diagnostic is changed.
            self.validate_standalone_capability(c)?;
        }
        if late {
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "the-after-head-insertion-mode",
                    branch: "head-start/delegate-using-head-pointer",
                },
                c,
            )?;
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
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
            self.finish(c)?
        };
        if late {
            self.before.kernel.remove_stack_identity(
                self.temporary_head
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?,
                c,
                self.script.as_ref().map(|script| &script.capability),
            )?;
            write(size_of::<Option<ParserNode>>(), 1, c)?;
            self.temporary_head = None;
        }
        Ok(step)
    }
    pub(super) fn end_template(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        if self.before.kernel.contains_html_template(c)? {
            self.close_template(c, true)?;
            return self.discard(c);
        }
        self.emit(
            HeadCondition::UnexpectedEndTag,
            crate::construction::parser_session::evidence::DiagnosticRule {
                anchor: "parsing-main-inhead",
                branch: "template-end/no-template-in-scope",
            },
            c,
        )?;
        self.discard(c)
    }
    pub(super) fn ordinary_at(
        &mut self,
        parent: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let node = match &mut pending.payload {
            Payload::Comment(input) => {
                self.before
                    .kernel
                    .session
                    .allocate_comment(c, input, Some(parent))?
            }
            Payload::Pi(input) => self.before.kernel.session.allocate_processing_instruction(
                c,
                input,
                Some(parent),
            )?,
            _ => return Err(HeadError::MissingInput),
        };
        self.before.kernel.session.insert(
            c,
            parent,
            &node,
            None,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.finish(c)
    }
    fn excluded(
        &self,
        kind: TagKind,
        c: &mut ParseControl,
    ) -> Result<Option<HeadBoundary>, HeadError> {
        if kind == TagKind::End {
            return Ok(None);
        }
        for (names, boundary) in [(&["input"][..], HeadBoundary::Form)] {
            if self.token_in(names, c)?.is_some() {
                if boundary == HeadBoundary::Form && self.hidden_input(c)? {
                    continue;
                }
                return Ok(Some(boundary));
            }
        }
        Ok(None)
    }
    pub(super) fn body_rules(
        &mut self,
        mode: InsertionMode,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        self.record_trigger(
            crate::construction::parser_session::provenance::ParserRule::Tree(mode),
            c,
        )?;
        if self.first_lf(c)? {
            return Ok(Some(self.finish(c)?));
        }
        let kind = self.kind(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        if matches!(payload, Payload::Eof) {
            if self.before.kernel.contains_html_template(c)? {
                return self.template_eof(c);
            }
            if mode == InsertionMode::AfterHead {
                self.implicit_body(c)?;
                return Ok(None);
            }
            if mode == InsertionMode::InBody {
                self.body_end_check(c)?;
            }
            return Ok(Some(self.begin_end(c)?));
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
            self.emit(
                HeadCondition::UnexpectedDoctype,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: mode.rule_anchor(),
                    branch: "doctype/ignore",
                },
                c,
            )?;
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
                        self.emit(
                            HeadCondition::UnacknowledgedSelfClosing,
                            crate::construction::parser_session::evidence::DiagnosticRule {
                                anchor: "the-after-head-insertion-mode",
                                branch: "after-head/body-start/self-closing",
                            },
                            c,
                        )?;
                    }
                    return Ok(Some(self.finish(c)?));
                }
                if self.token_in(HEAD, c)?.is_some() {
                    return Ok(Some(self.head_delegation(true, close, c)?));
                }
                if self.token_in(&["frameset"], c)?.is_some() {
                    return Ok(Some(self.frameset_start(
                        "the-after-head-insertion-mode",
                        close,
                        c,
                    )?));
                }
                if self.token_in(&["head"], c)?.is_some() {
                    self.emit(
                        HeadCondition::UnexpectedHead,
                        crate::construction::parser_session::evidence::DiagnosticRule {
                            anchor: mode.rule_anchor(),
                            branch: "after-head/head-start/ignore",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
            }
            if matches!(kind, Some((TagKind::End, _))) {
                if self.token_in(&["template"], c)?.is_some() {
                    return Ok(Some(self.end_template(c)?));
                }
                if self.token_in(&["body", "html", "br"], c)?.is_none() {
                    self.emit(
                        HeadCondition::UnexpectedEndTag,
                        crate::construction::parser_session::evidence::DiagnosticRule {
                            anchor: mode.rule_anchor(),
                            branch: "after-head/disallowed-end-tag",
                        },
                        c,
                    )?;
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
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: mode.rule_anchor(),
                    branch: "after-body/anything-else",
                },
                c,
            )?;
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
        if let Some((TagKind::Start, close)) = kind
            && self.token_in(&["frameset"], c)?.is_some()
        {
            self.frameset_admission(c)?;
            // The governing source diagnoses before stack/flag checks/removal.
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "frameset-start/in-body",
                },
                c,
            )?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.before.kernel.stack()?.len() < 2 {
                return Ok(Some(self.discard(c)?));
            }
            let body = self.before.kernel.at(1, c)?;
            if !self.before.kernel.named(&body, "body", c)? {
                return Ok(Some(self.discard(c)?));
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
            if self.frameset_ok {
                if self.before.kernel.session.parent(c, &body)?.is_some() {
                    self.before.kernel.session.remove(
                        c,
                        &body,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                }
                loop {
                    let current = self.before.kernel.current(c)?;
                    if self.before.kernel.named(&current, "html", c)? {
                        break;
                    }
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                }
                return Ok(Some(self.frameset_start(
                    "parsing-main-inbody",
                    close,
                    c,
                )?));
            }
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
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "disallowed-table-or-frame-start",
                },
                c,
            )?;
            return Ok(Some(self.discard(c)?));
        }
        if let Some(boundary) = self.excluded(kind.ok_or(HeadError::MissingInput)?.0, c)? {
            return Ok(Some(self.stop_at(boundary, c)?));
        }
        let Some((kind, close)) = kind else {
            return Err(HeadError::MissingInput);
        };
        if kind == TagKind::Start {
            if let Some(name) = self.token_in(&["math", "svg"], c)? {
                self.admit_hidden_capability(c)?;
                self.reconstruct_formatting(c)?;
                let namespace = if name == "math" {
                    crate::construction::parser_session::ForeignNamespace::MathMl
                } else {
                    crate::construction::parser_session::ForeignNamespace::Svg
                };
                return Ok(Some(self.insert_foreign(namespace, close, c)?));
            }
            if self.token_in(&["input"], c)?.is_some() {
                self.admit_hidden_capability(c)?;
                if self
                    .before
                    .kernel
                    .scoped_name("select", Scope::Default, c)?
                {
                    self.emit(
                        HeadCondition::UnexpectedBodyToken,
                        crate::construction::parser_session::evidence::DiagnosticRule {
                            anchor: "parsing-main-inbody",
                            branch: "input-start/select-in-scope",
                        },
                        c,
                    )?;
                    self.pop_select(c)?;
                }
                self.reconstruct_formatting(c)?;
                self.insert_hidden_input(c)?;
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                return Ok(Some(self.finish(c)?));
            }
            if self.token_in(&["textarea"], c)?.is_some() {
                match self.validate_standalone_capability(c) {
                    Err(HeadError::StandaloneCapabilityUnavailable) => {
                        return Err(c
                            .stop_unsupported_standalone(
                                crate::parser::control::StandaloneUnsupported::Provenance,
                            )
                            .into());
                    }
                    other => other?,
                }
                self.insert_tag(c)?;
                write(size_of::<bool>(), 1, c)?;
                self.ignore_first_lf = true;
                self.tokenizer
                    .as_mut()
                    .ok_or(HeadError::Consumed)?
                    .set_state(State::Raw(Raw::Rcdata), c)?;
                let mode = self.mode(c)?;
                write(size_of::<Option<InsertionMode>>(), 1, c)?;
                self.return_mode = Some(mode);
                self.frameset(false, c)?;
                self.set_mode(InsertionMode::Text, c)?;
                if close {
                    self.emit(
                        HeadCondition::UnacknowledgedSelfClosing,
                        crate::construction::parser_session::evidence::DiagnosticRule {
                            anchor: "parsing-main-inbody",
                            branch: "textarea-start/self-closing",
                        },
                        c,
                    )?;
                }
                return Ok(Some(self.finish(c)?));
            }
            if self.token_in(&["form"], c)?.is_some() {
                return Ok(Some(self.form_start(close, c)?));
            }
            if self.token_in(HEAD, c)?.is_some() {
                return Ok(Some(self.head_delegation(false, close, c)?));
            }
            if self.token_in(FORMATTING, c)?.is_some() {
                return Ok(Some(self.formatting_start(close, c)?));
            }
            if self
                .token_in(&["applet", "marquee", "object"], c)?
                .is_some()
            {
                return Ok(Some(self.marker_start(close, c)?));
            }
            return self.body_start(source, close, c);
        }
        if self.token_in(&["form"], c)?.is_some() {
            return Ok(Some(self.form_end(c)?));
        }
        if self.token_in(FORMATTING, c)?.is_some() {
            self.adoption(c)?;
            return Ok(Some(self.discard(c)?));
        }
        if let Some(name) = self.token_in(&["applet", "marquee", "object"], c)? {
            return Ok(Some(self.marker_end(name, c)?));
        }
        self.body_end(c)
    }
    fn form_start(&mut self, close: bool, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        let template = self.before.kernel.contains_html_template(c)?;
        // HTML877d4394: reject the nested opener before close-p or creation.
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.current_form.is_some() && !template {
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "form-start/existing-form-pointer",
                },
                c,
            )?;
            if close {
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "ignored-form-start/self-closing",
                    },
                    c,
                )?;
            }
            return self.discard(c);
        }
        self.close_p_in_scope(c)?;
        let node = self.insert_tag(c)?;
        if !template {
            let node = copy_node(&node, c)?;
            write(size_of::<Option<ParserNode>>(), 1, c)?;
            self.current_form = Some(node);
        }
        if close {
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "form-start/self-closing",
                },
                c,
            )?;
        }
        self.finish(c)
    }
    fn form_end(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        // Template form dispatch is a different source algorithm and remains
        // an actual continuation before effects in this ordinary unit.
        if self.before.kernel.contains_html_template(c)? {
            if !self.before.kernel.scoped_name("form", Scope::Default, c)? {
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "template-form-end/not-in-scope",
                    },
                    c,
                )?;
                return self.discard(c);
            }
            self.before.kernel.implied_ends(
                None,
                c,
                self.script.as_ref().map(|s| &s.capability),
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, "form", c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "template-form-end/current-node-mismatch",
                    },
                    c,
                )?;
            }
            loop {
                let node = self
                    .before
                    .kernel
                    .pop(c, self.script.as_ref().map(|s| &s.capability))?
                    .ok_or(KernelError::EmptyStack)?;
                if self.before.kernel.named(&node, "form", c)? {
                    break;
                }
            }
            return self.discard(c);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        let node = self.current_form.take();
        let Some(node) = node else {
            self.emit(
                HeadCondition::UnexpectedEndTag,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "form-end/null-form-pointer",
                },
                c,
            )?;
            return self.discard(c);
        };
        if !self.before.kernel.in_scope(&node, Scope::Default, c)? {
            self.emit(
                HeadCondition::UnexpectedEndTag,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "form-end/pointed-form-not-in-scope",
                },
                c,
            )?;
            return self.discard(c);
        }
        self.before.kernel.implied_ends(
            None,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        let current = self.before.kernel.current(c)?;
        if !self.before.kernel.session.same_node(c, &current, &node)? {
            // Governing HTML4516 diagnoses BEFORE4518 removes the identity;
            // the acquired engine's reversed ordering is not carried over.
            self.emit(
                HeadCondition::UnexpectedOpenElement,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "form-end/current-node-mismatch",
                },
                c,
            )?;
        }
        self.before.kernel.remove_stack_identity(
            &node,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.discard(c)
    }
    fn body_start(
        &mut self,
        source: &HtmlSource,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        if let Some(name) =
            self.token_in(&["select", "option", "optgroup", "selectedcontent"], c)?
        {
            // The real constructor proof is required before diagnostics, stack
            // mutation, reconstruction, or lexical ownership transfer.
            match self.validate_standalone_capability(c) {
                Err(HeadError::StandaloneCapabilityUnavailable) => {
                    return Err(c
                        .stop_unsupported_standalone(
                            crate::parser::control::StandaloneUnsupported::Provenance,
                        )
                        .into());
                }
                other => other?,
            }
            let select = self
                .before
                .kernel
                .scoped_name("select", Scope::Default, c)?;
            if name == "select" && select {
                self.emit(
                    HeadCondition::UnexpectedBodyToken,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "select-start/select-in-scope",
                    },
                    c,
                )?;
                let step = self.discard(c)?;
                self.pop_select(c)?;
                return Ok(Some(step));
            }
            if name == "option" || name == "optgroup" {
                if select {
                    self.before.kernel.implied_ends(
                        (name == "option").then_some("optgroup"),
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                    if self
                        .before
                        .kernel
                        .scoped_name("option", Scope::Default, c)?
                        || (name == "optgroup"
                            && self
                                .before
                                .kernel
                                .scoped_name("optgroup", Scope::Default, c)?)
                    {
                        self.emit(
                            HeadCondition::UnexpectedOpenElement,
                            crate::construction::parser_session::evidence::DiagnosticRule {
                                anchor: "parsing-main-inbody",
                                branch: "select-start/current-option-or-optgroup-not-in-scope",
                            },
                            c,
                        )?;
                    }
                } else {
                    let current = self.before.kernel.current(c)?;
                    if self.before.kernel.named(&current, "option", c)? {
                        self.before
                            .kernel
                            .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    }
                }
            }
            self.reconstruct_formatting(c)?;
            self.insert_select_family(c)?;
            if name == "select" {
                self.frameset(false, c)?;
            }
            if close {
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "select-start/self-closing",
                    },
                    c,
                )?;
            }
            return Ok(Some(self.finish(c)?));
        }
        if self.token_in(&["button"], c)?.is_some() {
            if self
                .before
                .kernel
                .scoped_name("button", Scope::Default, c)?
            {
                // HTML877d4478-4484 requires the diagnostic before implied
                // ends or stack removal, so rejected diagnostics retain the
                // old button and the actual unconsumed new token.
                self.emit(
                    HeadCondition::UnexpectedBodyToken,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "button-start/button-in-scope",
                    },
                    c,
                )?;
                self.before.kernel.implied_ends(
                    None,
                    c,
                    self.script.as_ref().map(|script| &script.capability),
                )?;
                loop {
                    let node = self
                        .before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                    if self.before.kernel.named(&node, "button", c)? {
                        break;
                    }
                }
            }
            self.reconstruct_formatting(c)?;
            self.insert_tag(c)?;
            self.frameset(false, c)?;
            if close {
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "button-start/self-closing",
                    },
                    c,
                )?;
            }
            return Ok(Some(self.finish(c)?));
        }
        if self.token_in(&["table"], c)?.is_some() {
            if self.before.kernel.session.document_mode(c)? != crate::HtmlDocumentMode::Quirks {
                self.close_p_in_scope(c)?;
            }
            self.insert_tag(c)?;
            self.frameset(false, c)?;
            self.set_mode(InsertionMode::InTable, c)?;
            if close {
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "table-start/self-closing",
                    },
                    c,
                )?;
            }
            return Ok(Some(self.finish(c)?));
        }
        if self.token_in(&["body"], c)?.is_some() {
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "body-start/in-body",
                },
                c,
            )?;
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
            self.emit(
                HeadCondition::UnexpectedHead,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "head-start/in-body",
                },
                c,
            )?;
            return Ok(Some(self.discard(c)?));
        }
        if self.token_in(BLOCK, c)?.is_some() {
            self.close_p_in_scope(c)?;
        } else if self.token_in(HEADING, c)?.is_some() {
            self.close_p_in_scope(c)?;
            let current = self.before.kernel.current(c)?;
            if self.before.kernel.node_set(&current, NodeSet::Heading, c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "heading-start/current-heading",
                    },
                    c,
                )?;
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?;
            }
        } else if self.token_in(&["pre", "listing"], c)?.is_some() {
            self.close_p_in_scope(c)?;
            self.insert_tag(c)?;
            write(size_of::<bool>(), 1, c)?;
            self.ignore_first_lf = true;
            self.frameset(false, c)?;
            if close {
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "pre-or-listing-start/self-closing",
                    },
                    c,
                )?;
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
                    self.before.kernel.implied_ends(
                        Some(except),
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                    let current = self.before.kernel.current(c)?;
                    if !self.before.kernel.session.same_node(c, &current, &node)? {
                        self.emit(
                            HeadCondition::UnexpectedOpenElement,
                            crate::construction::parser_session::evidence::DiagnosticRule {
                                anchor: "parsing-main-inbody",
                                branch: "list-item-start/current-node-mismatch",
                            },
                            c,
                        )?;
                    }
                    self.before.kernel.pop_through(
                        &node,
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
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
                self.emit(
                    HeadCondition::UnacknowledgedSelfClosing,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "plaintext-start/self-closing",
                    },
                    c,
                )?;
            }
            return Ok(Some(self.finish(c)?));
        } else if self.token_in(&["xmp", "iframe", "noembed"], c)?.is_some() {
            if self.token_in(&["xmp"], c)?.is_some() {
                self.close_p_in_scope(c)?;
                self.reconstruct_formatting(c)?;
            }
            if self.token_in(&["xmp", "iframe"], c)?.is_some() {
                self.frameset(false, c)?;
            }
            return Ok(Some(self.raw_text(Raw::Rawtext, close, c)?));
        } else if self.token_in(&["image"], c)?.is_some() {
            self.emit(
                HeadCondition::UnexpectedBodyToken,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "image-start/reprocess-as-img",
                },
                c,
            )?;
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
                if self
                    .before
                    .kernel
                    .scoped_name("select", Scope::Default, c)?
                {
                    self.before.kernel.implied_ends(
                        None,
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                    if self
                        .before
                        .kernel
                        .scoped_name("option", Scope::Default, c)?
                        || self
                            .before
                            .kernel
                            .scoped_name("optgroup", Scope::Default, c)?
                    {
                        self.emit(
                            HeadCondition::UnexpectedOpenElement,
                            crate::construction::parser_session::evidence::DiagnosticRule {
                                anchor: "parsing-main-inbody",
                                branch: "option-start/current-option-or-optgroup-not-in-scope",
                            },
                            c,
                        )?;
                    }
                }
            }
            if self
                .token_in(&["area", "br", "embed", "img", "keygen", "wbr"], c)?
                .is_some()
            {
                self.reconstruct_formatting(c)?;
            }
            self.insert_tag(c)?;
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?;
            if affects {
                self.frameset(false, c)?;
            }
            return Ok(Some(self.finish(c)?));
        } else if let Some(name) = self.token_in(&["rb", "rtc", "rp", "rt"], c)? {
            self.ruby_start(name, c)?;
        } else {
            self.reconstruct_formatting(c)?;
        }
        self.insert_tag(c)?;
        if close {
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "ordinary-start/self-closing",
                },
                c,
            )?;
        }
        Ok(Some(self.finish(c)?))
    }
    fn ruby_start(&mut self, name: &'static str, c: &mut ParseControl) -> Result<(), HeadError> {
        if self.before.kernel.scoped_name("ruby", Scope::Default, c)? {
            self.before.kernel.implied_ends(
                if matches!(name, "rp" | "rt") {
                    Some("rtc")
                } else {
                    None
                },
                c,
                self.script.as_ref().map(|script| &script.capability),
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, "ruby", c)?
                && !(matches!(name, "rp" | "rt") && self.before.kernel.named(&current, "rtc", c)?)
            {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "ruby-start/current-node-mismatch",
                    },
                    c,
                )?;
            }
        }
        Ok(())
    }
    fn pop_select(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        loop {
            let node = self
                .before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
            if self.before.kernel.named(&node, "select", c)? {
                return Ok(());
            }
        }
    }
    pub(super) fn token_matches_node(
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
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "body-or-html-end/body-not-in-scope",
                    },
                    c,
                )?;
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
            self.emit(
                HeadCondition::UnexpectedEndTag,
                crate::construction::parser_session::evidence::DiagnosticRule {
                    anchor: "parsing-main-inbody",
                    branch: "br-end/reprocess-as-start",
                },
                c,
            )?;
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
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "p-end/p-not-in-button-scope",
                    },
                    c,
                )?;
                let location = self.before.kernel.adjusted_location(None, c)?;
                let node = self
                    .before
                    .kernel
                    .session
                    .allocate_implicit_p(c, Some(&location.parent))?;
                self.before.kernel.insert_element_at(
                    &node,
                    &location,
                    c,
                    self.script.as_ref().map(|script| &script.capability),
                )?;
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
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "ordinary-end/no-target-in-required-scope",
                    },
                    c,
                )?;
                return Ok(Some(self.discard(c)?));
            }
            self.before.kernel.implied_ends(
                if matches!(name, "li" | "dd" | "dt") {
                    Some(name)
                } else {
                    None
                },
                c,
                self.script.as_ref().map(|script| &script.capability),
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, name, c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "ordinary-end/current-node-mismatch",
                    },
                    c,
                )?;
            }
            loop {
                let node = self
                    .before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?
                    .ok_or(KernelError::EmptyStack)?;
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
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "heading-end/no-heading-in-scope",
                    },
                    c,
                )?;
                return Ok(Some(self.discard(c)?));
            };
            self.before.kernel.implied_ends(
                None,
                c,
                self.script.as_ref().map(|script| &script.capability),
            )?;
            let current = self.before.kernel.current(c)?;
            if !self.token_matches_node(&current, c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "heading-end/current-node-mismatch",
                    },
                    c,
                )?;
            }
            self.before.kernel.pop_through(
                &target,
                c,
                self.script.as_ref().map(|script| &script.capability),
            )?;
            return Ok(Some(self.discard(c)?));
        }
        self.generic_end(c)?;
        Ok(Some(self.discard(c)?))
    }
    pub(super) fn generic_end(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
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
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                }
                let current = self.before.kernel.current(c)?;
                if !self.before.kernel.session.same_node(c, &node, &current)? {
                    self.emit(
                        HeadCondition::UnexpectedOpenElement,
                        crate::construction::parser_session::evidence::DiagnosticRule {
                            anchor: "parsing-main-inbody",
                            branch: "generic-end/matching-node-not-current",
                        },
                        c,
                    )?;
                }
                self.before.kernel.pop_through(
                    &node,
                    c,
                    self.script.as_ref().map(|script| &script.capability),
                )?;
                return Ok(());
            }
            if self.before.kernel.node_set(&node, NodeSet::Special, c)? {
                self.emit(
                    HeadCondition::UnexpectedEndTag,
                    crate::construction::parser_session::evidence::DiagnosticRule {
                        anchor: "parsing-main-inbody",
                        branch: "generic-end/special-node-before-match",
                    },
                    c,
                )?;
                return Ok(());
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

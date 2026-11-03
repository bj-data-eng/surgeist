// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/rules.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf:378-383,458-475,
// 1494-1563,1584-1608. Governing HTML877d adds PI and exact origin handling.
// Frame postconnection returns at the null browsing-context source guard,
// established by the same sealed fresh standalone owner, before external work.

use super::{
    KernelError,
    head::{HeadError, HeadState, HeadStep, HtmlInsertionMode, HtmlTreeDiagnostic},
    prologue::Payload,
    write,
};
use crate::{
    construction::{BodyTextKind, BodyTextRuns, TextRun, partition_body_text},
    parser::{
        control::{HtmlParseUnsupportedState, HtmlParseWorkCategory, ParseControl},
        tokenizer::token::{TagKind, Text},
    },
};
use std::mem::size_of;

impl HeadState<'_> {
    pub(super) fn frameset_admission(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        // Generic Document or Disabled alone cannot prove absent browsing context.
        // No frame-family effects occur through a proofless/imported head owner.
        match self.validate_standalone_capability(c) {
            Err(HeadError::ConstructionCapabilityUnavailable) => Err(c
                .stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance)
                .into()),
            result => result,
        }
    }

    pub(super) fn frameset_start(
        &mut self,
        anchor: &'static str,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.frameset_admission(c)?;
        self.insert_tag(c)?;
        self.set_mode(HtmlInsertionMode::InFrameset, c)?;
        if close {
            self.emit(
                HtmlTreeDiagnostic::UnacknowledgedSelfClosing,
                crate::construction::evidence::HtmlParseDiagnosticRule {
                    anchor,
                    branch: "frameset-start/self-closing",
                },
                c,
            )?;
        }
        self.finish(c)
    }

    fn frameset_text(
        &mut self,
        mode: HtmlInsertionMode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            // The returned owner is retained before any subsequent admission.
            write(size_of::<Option<TextRun>>(), 1, c)?;
            self.body_run = self
                .body_runs
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .take_next(c)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let Some(run) = &self.body_run else { break };
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                size_of::<BodyTextKind>() as u64,
            )?;
            let kind = run.kind;
            if kind == BodyTextKind::Whitespace {
                write(
                    const { size_of::<Option<TextRun>>() + size_of::<Option<Text>>() },
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                if mode == HtmlInsertionMode::AfterAfterFrameset {
                    // Source delegates whitespace to InBody without changing mode.
                    self.record_trigger(
                        crate::construction::provenance::HtmlParserRule::Tree(
                            HtmlInsertionMode::InBody,
                        ),
                        c,
                    )?;
                    self.body_character(false, c)?;
                } else {
                    self.before.kernel.insert_text(
                        &mut self.whitespace_pending,
                        None,
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                }
            } else {
                let condition = if kind == BodyTextKind::Null {
                    HtmlTreeDiagnostic::NullInFrameset
                } else {
                    HtmlTreeDiagnostic::UnexpectedFramesetToken
                };
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                for origin in run.text.origins.as_slice() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;

                    c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                    self.before.kernel.session.evidence(c)?.append(
                        c,
                        crate::construction::evidence::HtmlParseDiagnosticKind::Tree(condition),
                        crate::construction::evidence::HtmlParseDiagnosticContext::Tree(mode),
                        crate::construction::evidence::HtmlParseDiagnosticRule {
                            anchor: mode.rule_anchor(),
                            branch: "non-whitespace-character/ignore",
                        },
                        crate::construction::evidence::DiagnosticLocationInput {
                            source: &origin.source,
                            role: crate::construction::evidence::HtmlParseDiagnosticLocationRole::TextOrigin,
                            token: Some(crate::construction::evidence::HtmlParseTokenFamily::Text),
                        },
                    )?;
                }
                // Preserve actual omitted text evidence under the existing owner.
                write(
                    const { size_of::<Option<TextRun>>() + size_of::<Option<Text>>() },
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                self.ignored_text(
                    crate::construction::provenance::HtmlParseOmissionKind::FramesetCharacter,
                    c,
                )?;
            }
        }
        self.body_runs
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .release(c)?;
        write(size_of::<Option<BodyTextRuns>>(), 1, c)?;
        self.body_runs = None;
        self.finish(c)
    }

    pub(super) fn frameset_rules(
        &mut self,
        mode: HtmlInsertionMode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.frameset_admission(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        {
            Payload::Text(_) => return self.frameset_text(mode, c),
            Payload::Comment(_) | Payload::Pi(_) => {
                return if mode == HtmlInsertionMode::AfterAfterFrameset {
                    let document = self.before.kernel.session.root(c)?;
                    self.ordinary_at(&document, c)
                } else {
                    self.ordinary_node(c)
                };
            }
            Payload::Doctype(_) => {
                self.emit(
                    HtmlTreeDiagnostic::UnexpectedDoctype,
                    crate::construction::evidence::HtmlParseDiagnosticRule {
                        anchor: mode.rule_anchor(),
                        branch: "frameset-doctype/ignore",
                    },
                    c,
                )?;
                return self.discard(c);
            }
            Payload::Eof => {
                if mode == HtmlInsertionMode::InFrameset {
                    let current = self.before.kernel.current(c)?;
                    let html = self.before.kernel.first(c)?;
                    if !self.before.kernel.session.same_node(c, &current, &html)? {
                        self.emit(
                            HtmlTreeDiagnostic::UnexpectedOpenElement,
                            crate::construction::evidence::HtmlParseDiagnosticRule {
                                anchor: mode.rule_anchor(),
                                branch: "frameset-eof/current-node-not-html",
                            },
                            c,
                        )?;
                    }
                }
                return self.begin_end(c);
            }
            Payload::Tag(_) => {}
        }
        let (kind, close) = self.kind(c)?.ok_or(HeadError::MissingInput)?;
        if kind == TagKind::Start {
            if self.token_in(&["html"], c)?.is_some() {
                return self.html(close, c);
            }
            if self.token_in(&["noframes"], c)?.is_some() {
                return self.head_delegation(false, close, c);
            }
            if mode == HtmlInsertionMode::InFrameset {
                if self.token_in(&["frameset"], c)?.is_some() {
                    return self.frameset_start("parsing-main-inframeset", close, c);
                }
                if self.token_in(&["frame"], c)?.is_some() {
                    self.insert_tag(c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?
                        .ok_or(KernelError::EmptyStack)?;
                    // Actual fresh node-document has null browsing context, so
                    // frame postconnection returned before navigation machinery.
                    return self.finish(c);
                }
            }
        } else {
            if mode == HtmlInsertionMode::InFrameset && self.token_in(&["frameset"], c)?.is_some() {
                let current = self.before.kernel.current(c)?;
                let html = self.before.kernel.first(c)?;
                if self.before.kernel.session.same_node(c, &current, &html)? {
                    self.emit(
                        HtmlTreeDiagnostic::UnexpectedEndTag,
                        crate::construction::evidence::HtmlParseDiagnosticRule {
                            anchor: mode.rule_anchor(),
                            branch: "frameset-end/current-node-html",
                        },
                        c,
                    )?;
                    return self.discard(c);
                }
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?
                    .ok_or(KernelError::EmptyStack)?;
                // HTML877d "in frameset": the switch needs a null fragment
                // context element and a current node that is no longer a
                // frameset.
                let fragment = self.before.kernel.session.fragment_context(c)?.is_some();
                let current = self.before.kernel.current(c)?;
                if !fragment && !self.before.kernel.named(&current, "frameset", c)? {
                    self.set_mode(HtmlInsertionMode::AfterFrameset, c)?;
                }
                return self.discard(c);
            }
            if mode == HtmlInsertionMode::AfterFrameset && self.token_in(&["html"], c)?.is_some() {
                self.set_mode(HtmlInsertionMode::AfterAfterFrameset, c)?;
                return self.discard(c);
            }
        }
        self.emit(
            HtmlTreeDiagnostic::UnexpectedFramesetToken,
            crate::construction::evidence::HtmlParseDiagnosticRule {
                anchor: mode.rule_anchor(),
                branch: "frameset/anything-else",
            },
            c,
        )?;
        self.discard(c)
    }
}

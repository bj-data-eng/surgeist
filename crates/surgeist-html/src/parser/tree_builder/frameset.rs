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
    head::{HeadCondition, HeadDiagnostic, HeadError, HeadState, HeadStep, InsertionMode},
    prologue::Payload,
    write,
};
use crate::{
    construction::parser_session::{BodyTextKind, BodyTextRuns, TextRun, partition_body_text},
    parser::{
        control::{ParseControl, StandaloneUnsupported, WorkCategory},
        tokenizer::token::{TagKind, Text},
    },
};
use std::mem::size_of;

impl HeadState<'_> {
    pub(super) fn frameset_admission(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        // Generic Document or Disabled alone cannot prove absent browsing context.
        // No frame-family effects occur through a proofless/imported head owner.
        match self.validate_standalone_capability(c) {
            Err(HeadError::StandaloneCapabilityUnavailable) => Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into()),
            result => result,
        }
    }

    pub(super) fn frameset_start(
        &mut self,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.frameset_admission(c)?;
        self.insert_tag(c)?;
        self.set_mode(InsertionMode::InFrameset, c)?;
        if close {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }

    fn frameset_text(
        &mut self,
        mode: InsertionMode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
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
            // The returned owner is retained before any subsequent admission.
            write(size_of::<Option<TextRun>>(), 1, c)?;
            self.body_run = self
                .body_runs
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .take_next(c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let Some(run) = &self.body_run else { break };
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            let kind = run.kind;
            if kind == BodyTextKind::Whitespace {
                write(
                    size_of::<Option<TextRun>>() + size_of::<Option<Text>>(),
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                if mode == InsertionMode::AfterAfterFrameset {
                    // Source delegates whitespace to InBody without changing mode.
                    self.body_character(false, c)?;
                } else {
                    self.before
                        .kernel
                        .insert_text(&mut self.whitespace_pending, None, c)?;
                }
            } else {
                let condition = if kind == BodyTextKind::Null {
                    HeadCondition::NullInFrameset
                } else {
                    HeadCondition::UnexpectedFramesetToken
                };
                c.charge(WorkCategory::RecordVisit, 1)?;
                for origin in run.text.origins.as_slice() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    write(size_of::<HeadDiagnostic>(), 3, c)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.diagnostics.try_push(
                        c,
                        HeadDiagnostic {
                            condition,
                            mode,
                            source: origin.source.clone(),
                        },
                    )?;
                }
                // Preserve actual omitted text evidence under the existing owner.
                write(
                    size_of::<Option<TextRun>>() + size_of::<Option<Text>>(),
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                self.ignored_text(c)?;
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

    pub(super) fn frameset_rules(
        &mut self,
        mode: InsertionMode,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.frameset_admission(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        {
            Payload::Text(_) => return self.frameset_text(mode, c),
            Payload::Comment(_) | Payload::Pi(_) => {
                return if mode == InsertionMode::AfterAfterFrameset {
                    let document = self.before.kernel.session.root(c)?;
                    self.ordinary_at(&document, c)
                } else {
                    self.ordinary_node(c)
                };
            }
            Payload::Doctype(_) => {
                self.emit(HeadCondition::UnexpectedDoctype, c)?;
                return self.discard(c);
            }
            Payload::Eof => {
                if mode == InsertionMode::InFrameset {
                    let current = self.before.kernel.current(c)?;
                    let html = self.before.kernel.first(c)?;
                    if !self.before.kernel.session.same_node(c, &current, &html)? {
                        self.emit(HeadCondition::UnexpectedOpenElement, c)?;
                    }
                }
                return self.stop_at(super::head::HeadBoundary::ConstructionEof, c);
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
            if mode == InsertionMode::InFrameset {
                if self.token_in(&["frameset"], c)?.is_some() {
                    return self.frameset_start(close, c);
                }
                if self.token_in(&["frame"], c)?.is_some() {
                    self.insert_tag(c)?;
                    self.before.kernel.pop(c)?.ok_or(KernelError::EmptyStack)?;
                    // Actual fresh node-document has null browsing context, so
                    // frame postconnection returned before navigation machinery.
                    return self.finish(c);
                }
            }
        } else {
            if mode == InsertionMode::InFrameset && self.token_in(&["frameset"], c)?.is_some() {
                let current = self.before.kernel.current(c)?;
                let html = self.before.kernel.first(c)?;
                if self.before.kernel.session.same_node(c, &current, &html)? {
                    self.emit(HeadCondition::UnexpectedEndTag, c)?;
                    return self.discard(c);
                }
                self.before.kernel.pop(c)?.ok_or(KernelError::EmptyStack)?;
                let current = self.before.kernel.current(c)?;
                if !self.before.kernel.named(&current, "frameset", c)? {
                    self.set_mode(InsertionMode::AfterFrameset, c)?;
                }
                return self.discard(c);
            }
            if mode == InsertionMode::AfterFrameset && self.token_in(&["html"], c)?.is_some() {
                self.set_mode(InsertionMode::AfterAfterFrameset, c)?;
                return self.discard(c);
            }
        }
        self.emit(HeadCondition::UnexpectedFramesetToken, c)?;
        self.discard(c)
    }
}

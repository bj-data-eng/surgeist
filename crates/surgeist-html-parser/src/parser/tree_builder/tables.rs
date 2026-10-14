// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever tree_builder/rules.rs:1037-1411 and
// tree_builder/mod.rs:1236-1320 at d7232d746d3112f08926d169591576bd12fe2fdf.
// Governing HTML877d syntax5056-5498 controls PI, predicate differences,
// diagnostic order and source reprocessing. Other input/resettable controls, templates,
// fragments and SVG script remain actual pre-effect continuations.

use super::head::{HeadCondition, HeadError, HeadState, HeadStep, InsertionMode};
use super::prologue::Payload;
use super::{KernelError, Scope, copy_node, write};
use crate::{
    HtmlParseResource, HtmlSource, HtmlSourceRange,
    construction::{BodyTextKind, partition_body_text},
    parser::{
        control::{OwnedVec, ParseControl, WorkCategory},
        tokenizer::token::{TagKind, Text},
    },
};
use std::mem::size_of;

pub(super) struct TableCharacter {
    source: HtmlSourceRange,
    kind: BodyTextKind,
    text: Option<Text>,
    cause: Option<crate::construction::provenance::ConstructionCauseRef>,
}
pub(super) struct TableCharacters {
    pub(super) entries: OwnedVec<Option<TableCharacter>>,
    transfer: Option<TableCharacter>,
    next: usize,
    nonwhite: bool,
}
impl TableCharacters {
    pub(super) fn new(c: &mut ParseControl) -> Result<Self, HeadError> {
        let mut entries = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        entries.require_owner(c)?;
        if let Err(stop) = write(size_of::<Self>(), 4, c) {
            entries.release(c)?;
            return Err(stop.into());
        }
        Ok(Self {
            entries,
            transfer: None,
            next: 0,
            nonwhite: false,
        })
    }
}
const PARTS: &[&str] = &[
    "caption", "col", "colgroup", "tbody", "td", "tfoot", "th", "thead", "tr",
];
const SECTIONS: &[&str] = &["tbody", "thead", "tfoot"];
impl HeadState<'_> {
    fn table_error(
        &mut self,
        rule: crate::construction::evidence::DiagnosticRule,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        self.emit(HeadCondition::UnexpectedTableToken, rule, c)
    }
    fn current_in(&self, names: &[&'static str], c: &mut ParseControl) -> Result<bool, HeadError> {
        let current = self.before.kernel.current(c)?;
        for name in names {
            if self.before.kernel.named(&current, name, c)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn table_scope_any(
        &self,
        names: &[&'static str],
        c: &mut ParseControl,
    ) -> Result<bool, HeadError> {
        for name in names {
            if self.before.kernel.scoped_name(name, Scope::Table, c)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn clear_table_context(
        &mut self,
        names: &[&'static str],
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        loop {
            if self.current_in(names, c)? {
                return Ok(());
            }
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
        }
    }
    fn pop_table_name(
        &mut self,
        name: &'static str,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        loop {
            let node = self
                .before
                .kernel
                .pop(c, self.script.as_ref().map(|script| &script.capability))?
                .ok_or(KernelError::EmptyStack)?;
            if self.before.kernel.named(&node, name, c)? {
                return Ok(());
            }
        }
    }
    pub(super) fn reset_table_mode(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.before.kernel.stack()?.len()).rev() {
            let node = self.before.kernel.at(index, c)?;
            let context = if index == 0 {
                self.before.kernel.session.fragment_context(c)?
            } else {
                None
            };
            let name = if let Some(context) = context {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let node = context
                    .capture()
                    .node(context.context_element().subject())
                    .map_err(|_| crate::construction::SessionError::ContextUnavailable)?;
                let crate::HtmlCapturedNodeKind::Element(element) = node.kind() else {
                    return Err(KernelError::NotElement.into());
                };
                element.name()
            } else {
                self.before
                    .kernel
                    .session
                    .element_name(c, &node)?
                    .ok_or(KernelError::NotElement)?
            };
            if super::tag_sets::html_named(name, "template", c)? {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let mode = *self
                    .template_modes
                    .as_slice()
                    .last()
                    .ok_or(HeadError::MissingInput)?;
                return self.set_mode(mode, c);
            }
            if index != 0
                && (super::tag_sets::html_named(name, "td", c)?
                    || super::tag_sets::html_named(name, "th", c)?)
            {
                return self.set_mode(InsertionMode::InCell, c);
            }
            if let Some(mode) = crate::bounded::stack_reset_insertion_mode(|local| {
                super::tag_sets::html_named(name, local, c)
            })? {
                return self.set_mode(super::context::parser_insertion_mode(mode), c);
            }
            if index != 0 && super::tag_sets::html_named(name, "head", c)? {
                return self.set_mode(InsertionMode::InHead, c);
            }
            if super::tag_sets::html_named(name, "html", c)? {
                c.charge(WorkCategory::RecordVisit, 1)?;
                return self.set_mode(
                    if self.head.is_some() {
                        InsertionMode::AfterHead
                    } else {
                        InsertionMode::BeforeHead
                    },
                    c,
                );
            }
            if index == 0 {
                return self.set_mode(InsertionMode::InBody, c);
            }
        }
        self.set_mode(InsertionMode::InBody, c)
    }
    fn implied_table_element(
        &mut self,
        name: &'static str,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        let place = self.before.kernel.appropriate_place(None, c)?;
        let node = match name {
            "colgroup" => self
                .before
                .kernel
                .session
                .allocate_implicit_colgroup(c, Some(&place.parent))?,
            "tbody" => self
                .before
                .kernel
                .session
                .allocate_implicit_tbody(c, Some(&place.parent))?,
            "tr" => self
                .before
                .kernel
                .session
                .allocate_implicit_tr(c, Some(&place.parent))?,
            _ => return Err(HeadError::InvalidTextToken),
        };
        let location = self.before.kernel.adjusted_location(None, c)?;
        self.before.kernel.insert_element_at(
            &node,
            &location,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        self.before.kernel.push(&node, c)?;
        Ok(())
    }
    fn table_insert(
        &mut self,
        mode: InsertionMode,
        anchor: &'static str,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        self.insert_tag(c)?;
        self.set_mode(mode, c)?;
        if close {
            self.emit(
                HeadCondition::UnacknowledgedSelfClosing,
                crate::construction::evidence::DiagnosticRule {
                    anchor,
                    branch: "table-structure-start/self-closing",
                },
                c,
            )?;
        }
        Ok(Some(self.finish(c)?))
    }
    /// Preserve the actual input-state recognition before delegated body work.
    fn classify_delegated_input(&self, c: &mut ParseControl) -> Result<(), HeadError> {
        if let Some((TagKind::Start, _)) = self.kind(c)?
            && self.token_in(&["input"], c)?.is_some()
        {
            self.input_type(c)?;
        }
        Ok(())
    }
    fn delegate_body(
        &mut self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        self.classify_delegated_input(c)?;
        self.body_rules(InsertionMode::InBody, source, c)
    }
    fn foster_body(
        &mut self,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        self.classify_delegated_input(c)?;
        self.table_error(
            crate::construction::evidence::DiagnosticRule {
                anchor: "parsing-main-intable",
                branch: "anything-else/foster",
            },
            c,
        )?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
        let previous = self.before.kernel.foster_parenting;
        self.before.kernel.set_foster_parenting(true, c)?;
        loop {
            c.charge(WorkCategory::StateStep, 1)?;
            if let Some(step) = self.body_rules(InsertionMode::InBody, source, c)? {
                self.before.kernel.set_foster_parenting(previous, c)?;
                return Ok(Some(step));
            }
        }
    }
    fn collect_table_text(&mut self, c: &mut ParseControl) -> Result<Option<HeadStep>, HeadError> {
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
            write(size_of::<Option<crate::construction::BodyTextRuns>>(), 1, c)?;
            self.body_runs = Some(partition_body_text(c, input)?);
        }
        loop {
            write(size_of::<Option<crate::construction::TextRun>>(), 1, c)?;
            self.body_run = self
                .body_runs
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .take_next(c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let Some(run) = &self.body_run else {
                break;
            };
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            if run.kind == BodyTextKind::Null {
                // Source origin records are real tokenizer-owned ranges.
                c.charge(WorkCategory::RecordVisit, 1)?;
                let mode = self.mode(c)?;
                for origin in self
                    .body_run
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?
                    .text
                    .origins
                    .as_slice()
                {
                    c.charge(WorkCategory::RecordVisit, 1)?;

                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.before.kernel.session.evidence(c)?.append(
                        c,
                        crate::construction::evidence::DiagnosticKind::Tree(
                            HeadCondition::NullInTable,
                        ),
                        crate::construction::evidence::DiagnosticContext::Tree(mode),
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intabletext",
                            branch: "null-character/ignore",
                        },
                        crate::construction::evidence::DiagnosticLocationInput {
                            source: &origin.source,
                            role: crate::construction::evidence::DiagnosticLocationRole::TextOrigin,
                            token: Some(crate::construction::evidence::DiagnosticToken::Text),
                        },
                    )?;
                }
                write(
                    size_of::<Option<crate::construction::TextRun>>() + size_of::<Option<Text>>(),
                    2,
                    c,
                )?;
                self.whitespace_pending = self.body_run.take().map(|run| run.text);
                let text = self
                    .whitespace_pending
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?;
                self.before.kernel.session.evidence(c)?.retain_omission(
                    c,
                    text.origins.as_slice(),
                    crate::construction::provenance::OmissionKind::NullInTable,
                    crate::construction::provenance::ParserRule::Tree(mode),
                )?;
                write(size_of::<Option<Text>>(), 1, c)?;
                self.whitespace_pending
                    .take()
                    .ok_or(HeadError::MissingInput)?
                    .release(c)?;
            } else {
                let kind = run.kind;
                c.charge(WorkCategory::RecordVisit, 1)?;
                write(size_of::<HtmlSourceRange>(), 1, c)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                let source = self
                    .pending
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?
                    .source
                    .clone();
                let cause = self.before.kernel.session.current_parser_cause(c)?;
                write(
                    size_of::<TableCharacter>() + size_of::<Option<crate::construction::TextRun>>(),
                    5,
                    c,
                )?;
                self.table.transfer = Some(TableCharacter {
                    source,
                    kind,
                    cause,
                    text: self.body_run.take().map(|run| run.text),
                });
                if kind == BodyTextKind::Other {
                    write(size_of::<bool>(), 1, c)?;
                    self.table.nonwhite = true;
                }
                write(size_of::<Option<TableCharacter>>(), 1, c)?;
                let entry = self.table.transfer.take();
                if let Err((stop, entry)) = self.table.entries.try_push_recover(c, entry) {
                    self.table.transfer = entry;
                    return Err(stop.into());
                }
            }
        }
        self.body_runs
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .release(c)?;
        write(size_of::<Option<crate::construction::BodyTextRuns>>(), 1, c)?;
        self.body_runs = None;
        Ok(Some(self.finish(c)?))
    }
    fn flush_table_text(&mut self, c: &mut ParseControl) -> Result<Option<HeadStep>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
        let nonwhite = self.table.nonwhite;
        if nonwhite {
            // One list-level error, then each character list member is processed
            // using InTable's anything-else rule (including its own error).
            let mode = self.mode(c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            for entry in self.table.entries.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let entry = entry.as_ref().ok_or(HeadError::MissingInput)?;
                c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
                if entry.kind == BodyTextKind::Other {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    self.before.kernel.session.evidence(c)?.append(c, crate::construction::evidence::DiagnosticKind::Tree(HeadCondition::NonWhitespaceInTable), crate::construction::evidence::DiagnosticContext::Tree(mode), crate::construction::evidence::DiagnosticRule { anchor: "parsing-main-intabletext", branch: "pending-characters/non-whitespace" }, crate::construction::evidence::DiagnosticLocationInput { source: &entry.source, role: crate::construction::evidence::DiagnosticLocationRole::BufferedTableCharacter, token: Some(crate::construction::evidence::DiagnosticToken::Text) })?;
                    break;
                }
            }
        }
        loop {
            c.charge(WorkCategory::RecordVisit, 2)?;
            c.charge(WorkCategory::InspectByte, size_of::<usize>() as u64)?;
            let index = self.table.next;
            if index == self.table.entries.len() {
                break;
            }
            write(
                size_of::<Option<TableCharacter>>() * 2 + size_of::<usize>(),
                3,
                c,
            )?;
            self.table.transfer = self
                .table
                .entries
                .get_mut(index, c)?
                .ok_or(HeadError::MissingInput)?
                .take();
            self.table.next += 1;
            c.charge(WorkCategory::RecordVisit, 1)?;
            let entry = self
                .table
                .transfer
                .as_ref()
                .ok_or(HeadError::MissingInput)?;
            if nonwhite {
                let mode = self.mode(c)?;

                c.charge(WorkCategory::ProviderCall, 1)?;
                self.before.kernel.session.evidence(c)?.append(c, crate::construction::evidence::DiagnosticKind::Tree(HeadCondition::UnexpectedTableToken), crate::construction::evidence::DiagnosticContext::Tree(mode), crate::construction::evidence::DiagnosticRule { anchor: "parsing-main-intable", branch: "anything-else/pending-character" }, crate::construction::evidence::DiagnosticLocationInput { source: &entry.source, role: crate::construction::evidence::DiagnosticLocationRole::BufferedTableCharacter, token: Some(crate::construction::evidence::DiagnosticToken::Text) })?;
            }
            let previous = self
                .before
                .kernel
                .session
                .begin_buffered_replay(c, entry.cause.as_ref())?;
            c.charge(WorkCategory::InspectByte, size_of::<BodyTextKind>() as u64)?;
            let other = entry.kind == BodyTextKind::Other;
            write(size_of::<Option<Text>>() * 2, 2, c)?;
            self.whitespace_pending = self
                .table
                .transfer
                .as_mut()
                .ok_or(HeadError::MissingInput)?
                .text
                .take();
            if nonwhite {
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(WorkCategory::InspectByte, size_of::<bool>() as u64)?;
                let old = self.before.kernel.foster_parenting;
                self.before.kernel.set_foster_parenting(true, c)?;
                self.body_character(other, c)?;
                self.before.kernel.set_foster_parenting(old, c)?;
            } else {
                self.before.kernel.insert_text(
                    &mut self.whitespace_pending,
                    None,
                    c,
                    self.script.as_ref().map(|script| &script.capability),
                )?;
            }
            self.before
                .kernel
                .session
                .finish_buffered_replay(c, previous.as_ref())?;
            write(size_of::<Option<TableCharacter>>(), 1, c)?;
            self.table.transfer = None;
        }
        self.table.entries.clear(c)?;
        write(size_of::<usize>() + size_of::<bool>(), 2, c)?;
        self.table.next = 0;
        self.table.nonwhite = false;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            size_of::<Option<InsertionMode>>() as u64,
        )?;
        let mode = self.return_mode.ok_or(HeadError::MissingReturnMode)?;
        self.set_mode(mode, c)?;
        write(size_of::<Option<InsertionMode>>(), 1, c)?;
        self.return_mode = None;
        Ok(None)
    }
    fn dispose_table_transfer(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(entry) = &mut self.table.transfer {
            if c.stop().is_none() {
                write(size_of::<Option<Text>>(), 1, c)?;
            }
            if let Some(text) = entry.text.take() {
                text.release(c)?;
            }
            if c.stop().is_none() {
                write(size_of::<Option<TableCharacter>>(), 1, c)?;
            }
            self.table.transfer = None;
        }
        Ok(())
    }
    pub(super) fn release_table(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.dispose_table_transfer(c)?;
        loop {
            // Admit the actual owning transfer before removing any vector slot.
            // A later disposal stop leaves this field reachable for terminal retry.
            if c.stop().is_none() {
                write(size_of::<Option<TableCharacter>>(), 1, c)?;
            }
            self.table.transfer = if c.stop().is_some() {
                self.table.entries.cleanup_pop(c)?
            } else {
                self.table.entries.try_pop(c)?
            }
            .flatten();
            if c.stop().is_none() {
                c.charge(WorkCategory::RecordVisit, 1)?;
            }
            if self.table.transfer.is_none() {
                break;
            }
            self.dispose_table_transfer(c)?;
        }
        Ok(())
    }
    fn close_table_cell(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        self.before.kernel.implied_ends(
            None,
            c,
            self.script.as_ref().map(|script| &script.capability),
        )?;
        if !self.current_in(&["td", "th"], c)? {
            self.table_error(
                crate::construction::evidence::DiagnosticRule {
                    anchor: "parsing-main-intd",
                    branch: "cell-close/current-node-mismatch",
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
            if self.before.kernel.named(&node, "td", c)?
                || self.before.kernel.named(&node, "th", c)?
            {
                break;
            }
        }
        self.clear_marker(c)?;
        self.set_mode(InsertionMode::InRow, c)
    }
    pub(super) fn table_rules(
        &mut self,
        mode: InsertionMode,
        source: &HtmlSource,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        let kind = self.kind(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let text = matches!(
            self.pending
                .as_ref()
                .ok_or(HeadError::MissingInput)?
                .payload,
            Payload::Text(_)
        );
        if mode == InsertionMode::InTableText {
            return if text {
                self.collect_table_text(c)
            } else {
                self.flush_table_text(c)
            };
        }
        let start = kind.is_some_and(|(kind, _)| kind == TagKind::Start);
        let end = kind.is_some_and(|(kind, _)| kind == TagKind::End);
        let close = kind.is_some_and(|(_, close)| close);
        match mode {
            InsertionMode::InCaption => {
                let close_caption = end && self.token_in(&["caption"], c)?.is_some();
                let reprocess = (start && self.token_in(PARTS, c)?.is_some())
                    || (end && self.token_in(&["table"], c)?.is_some());
                if close_caption || reprocess {
                    if !self.before.kernel.scoped_name("caption", Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-incaption",
                                branch: "caption-close/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.before.kernel.implied_ends(
                        None,
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                    if !self.current_in(&["caption"], c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-incaption",
                                branch: "caption-close/current-node-mismatch",
                            },
                            c,
                        )?;
                    }
                    self.pop_table_name("caption", c)?;
                    self.clear_marker(c)?;
                    self.set_mode(InsertionMode::InTable, c)?;
                    return if reprocess {
                        Ok(None)
                    } else {
                        Ok(Some(self.discard(c)?))
                    };
                }
                if end
                    && self
                        .token_in(
                            &[
                                "body", "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
                                "thead", "tr",
                            ],
                            c,
                        )?
                        .is_some()
                {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-incaption",
                            branch: "disallowed-end-tag",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
                return self.delegate_body(source, c);
            }
            InsertionMode::InColumnGroup => {
                if text && self.whitespace(c)? {
                    return Ok(Some(self.finish(c)?));
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                match &self
                    .pending
                    .as_ref()
                    .ok_or(HeadError::MissingInput)?
                    .payload
                {
                    Payload::Comment(_) | Payload::Pi(_) => {
                        return Ok(Some(self.ordinary_node(c)?));
                    }
                    Payload::Doctype(_) => {
                        self.emit(
                            HeadCondition::UnexpectedDoctype,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-incolgroup",
                                branch: "doctype/ignore",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    Payload::Eof => return self.delegate_body(source, c),
                    _ => {}
                }
                if start && self.token_in(&["html"], c)?.is_some() {
                    return self.delegate_body(source, c);
                }
                if start && self.token_in(&["col"], c)?.is_some() {
                    self.insert_tag(c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    return Ok(Some(self.finish(c)?));
                }
                if end && self.token_in(&["colgroup"], c)?.is_some() {
                    if !self.current_in(&["colgroup"], c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-incolgroup",
                                branch: "colgroup-end/current-node-mismatch",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    self.set_mode(InsertionMode::InTable, c)?;
                    return Ok(Some(self.discard(c)?));
                }
                if end && self.token_in(&["col"], c)?.is_some() {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-incolgroup",
                            branch: "col-end/ignore",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
                if self.token_in(&["template"], c)?.is_some() {
                    return if start {
                        Ok(Some(self.template_start(c)?))
                    } else {
                        Ok(Some(self.end_template(c)?))
                    };
                }
                if !self.current_in(&["colgroup"], c)? {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-incolgroup",
                            branch: "anything-else/current-node-mismatch",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                self.set_mode(InsertionMode::InTable, c)?;
                return Ok(None);
            }
            InsertionMode::InTableBody => {
                if start && self.token_in(&["tr"], c)?.is_some() {
                    self.clear_table_context(&["tbody", "tfoot", "thead", "template", "html"], c)?;
                    return self.table_insert(
                        InsertionMode::InRow,
                        "parsing-main-intbody",
                        close,
                        c,
                    );
                }
                if start && self.token_in(&["td", "th"], c)?.is_some() {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intbody",
                            branch: "cell-start/implied-row",
                        },
                        c,
                    )?;
                    self.clear_table_context(&["tbody", "tfoot", "thead", "template", "html"], c)?;
                    self.implied_table_element("tr", c)?;
                    self.set_mode(InsertionMode::InRow, c)?;
                    return Ok(None);
                }
                if end && let Some(name) = self.token_in(SECTIONS, c)? {
                    if !self.before.kernel.scoped_name(name, Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intbody",
                                branch: "section-end/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.clear_table_context(&["tbody", "tfoot", "thead", "template", "html"], c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    self.set_mode(InsertionMode::InTable, c)?;
                    return Ok(Some(self.discard(c)?));
                }
                if (start
                    && self
                        .token_in(
                            &["caption", "col", "colgroup", "tbody", "tfoot", "thead"],
                            c,
                        )?
                        .is_some())
                    || (end && self.token_in(&["table"], c)?.is_some())
                {
                    if !self.table_scope_any(SECTIONS, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intbody",
                                branch: "table-structure/no-section-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.clear_table_context(&["tbody", "tfoot", "thead", "template", "html"], c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    self.set_mode(InsertionMode::InTable, c)?;
                    return Ok(None);
                }
                if end
                    && self
                        .token_in(
                            &[
                                "body", "caption", "col", "colgroup", "html", "td", "th", "tr",
                            ],
                            c,
                        )?
                        .is_some()
                {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intbody",
                            branch: "disallowed-end-tag",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
            }
            InsertionMode::InRow => {
                if start && self.token_in(&["td", "th"], c)?.is_some() {
                    self.clear_table_context(&["tr", "template", "html"], c)?;
                    self.insert_tag(c)?;
                    self.set_mode(InsertionMode::InCell, c)?;
                    self.push_marker(c)?;
                    if close {
                        self.emit(
                            HeadCondition::UnacknowledgedSelfClosing,
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intr",
                                branch: "cell-start/self-closing",
                            },
                            c,
                        )?;
                    }
                    return Ok(Some(self.finish(c)?));
                }
                let end_row = end && self.token_in(&["tr"], c)?.is_some();
                let outer = (start
                    && self
                        .token_in(
                            &[
                                "caption", "col", "colgroup", "tbody", "tfoot", "thead", "tr",
                            ],
                            c,
                        )?
                        .is_some())
                    || (end && self.token_in(&["table"], c)?.is_some());
                let section = if end {
                    self.token_in(SECTIONS, c)?
                } else {
                    None
                };
                if let Some(name) = section {
                    if !self.before.kernel.scoped_name(name, Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intr",
                                branch: "section-end/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    if !self.before.kernel.scoped_name("tr", Scope::Table, c)? {
                        return Ok(Some(self.discard(c)?));
                    }
                }
                if end_row || outer || section.is_some() {
                    if !self.before.kernel.scoped_name("tr", Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intr",
                                branch: "row-close/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.clear_table_context(&["tr", "template", "html"], c)?;
                    self.before
                        .kernel
                        .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                    self.set_mode(InsertionMode::InTableBody, c)?;
                    return if end_row {
                        Ok(Some(self.discard(c)?))
                    } else {
                        Ok(None)
                    };
                }
                if end
                    && self
                        .token_in(
                            &["body", "caption", "col", "colgroup", "html", "td", "th"],
                            c,
                        )?
                        .is_some()
                {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intr",
                            branch: "disallowed-end-tag",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
            }
            InsertionMode::InCell => {
                if end && let Some(name) = self.token_in(&["td", "th"], c)? {
                    if !self.before.kernel.scoped_name(name, Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intd",
                                branch: "cell-end/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.before.kernel.implied_ends(
                        None,
                        c,
                        self.script.as_ref().map(|script| &script.capability),
                    )?;
                    if !self.current_in(&[name], c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intd",
                                branch: "cell-end/current-node-mismatch",
                            },
                            c,
                        )?;
                    }
                    self.pop_table_name(name, c)?;
                    self.clear_marker(c)?;
                    self.set_mode(InsertionMode::InRow, c)?;
                    return Ok(Some(self.discard(c)?));
                }
                if start
                    && self
                        .token_in(
                            &[
                                "caption", "col", "colgroup", "tbody", "td", "tfoot", "th",
                                "thead", "tr",
                            ],
                            c,
                        )?
                        .is_some()
                {
                    self.close_table_cell(c)?;
                    return Ok(None);
                }
                if end
                    && self
                        .token_in(&["body", "caption", "col", "colgroup", "html"], c)?
                        .is_some()
                {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intd",
                            branch: "disallowed-end-tag",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
                if end
                    && let Some(name) =
                        self.token_in(&["table", "tbody", "tfoot", "thead", "tr"], c)?
                {
                    if !self.before.kernel.scoped_name(name, Scope::Table, c)? {
                        self.table_error(
                            crate::construction::evidence::DiagnosticRule {
                                anchor: "parsing-main-intd",
                                branch: "outer-end/not-in-scope",
                            },
                            c,
                        )?;
                        return Ok(Some(self.discard(c)?));
                    }
                    self.close_table_cell(c)?;
                    return Ok(None);
                }
                return self.delegate_body(source, c);
            }
            _ => {}
        }
        // Remaining table-body/row rules delegate InTable without changing the
        // actual current mode. Text entry saves that real current mode.
        self.record_trigger(
            crate::construction::provenance::ParserRule::Tree(InsertionMode::InTable),
            c,
        )?;
        if text && self.current_in(&["table", "tbody", "template", "tfoot", "thead", "tr"], c)? {
            write(size_of::<Option<InsertionMode>>(), 1, c)?;
            self.return_mode = Some(mode);
            self.set_mode(InsertionMode::InTableText, c)?;
            return Ok(None);
        }
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload
        {
            Payload::Comment(_) | Payload::Pi(_) => return Ok(Some(self.ordinary_node(c)?)),
            Payload::Doctype(_) => {
                self.emit(
                    HeadCondition::UnexpectedDoctype,
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-intable",
                        branch: "doctype/ignore",
                    },
                    c,
                )?;
                return Ok(Some(self.discard(c)?));
            }
            Payload::Eof => return self.delegate_body(source, c),
            _ => {}
        }
        if start {
            if self.token_in(&["caption"], c)?.is_some() {
                self.clear_table_context(&["table", "template", "html"], c)?;
                self.push_marker(c)?;
                return self.table_insert(
                    InsertionMode::InCaption,
                    "parsing-main-intable",
                    close,
                    c,
                );
            }
            if self.token_in(&["colgroup"], c)?.is_some() {
                self.clear_table_context(&["table", "template", "html"], c)?;
                return self.table_insert(
                    InsertionMode::InColumnGroup,
                    "parsing-main-intable",
                    close,
                    c,
                );
            }
            if self.token_in(&["col"], c)?.is_some() {
                self.clear_table_context(&["table", "template", "html"], c)?;
                self.implied_table_element("colgroup", c)?;
                self.set_mode(InsertionMode::InColumnGroup, c)?;
                return Ok(None);
            }
            if self.token_in(SECTIONS, c)?.is_some() {
                self.clear_table_context(&["table", "template", "html"], c)?;
                return self.table_insert(
                    InsertionMode::InTableBody,
                    "parsing-main-intable",
                    close,
                    c,
                );
            }
            if self.token_in(&["td", "th", "tr"], c)?.is_some() {
                self.clear_table_context(&["table", "template", "html"], c)?;
                self.implied_table_element("tbody", c)?;
                self.set_mode(InsertionMode::InTableBody, c)?;
                return Ok(None);
            }
            if self.token_in(&["table"], c)?.is_some() {
                self.table_error(
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-intable",
                        branch: "table-start/nested",
                    },
                    c,
                )?;
                if !self.before.kernel.scoped_name("table", Scope::Table, c)? {
                    return Ok(Some(self.discard(c)?));
                }
                self.pop_table_name("table", c)?;
                self.reset_table_mode(c)?;
                return Ok(None);
            }
            if self
                .token_in(&["style", "script", "template"], c)?
                .is_some()
            {
                return Ok(Some(self.head_delegation(false, close, c)?));
            }
            if self.token_in(&["input"], c)?.is_some() {
                self.input_type(c)?;
                if !self.hidden_input(c)? {
                    return self.foster_body(source, c);
                }
                self.admit_hidden_capability(c)?;
                self.table_error(
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-intable",
                        branch: "hidden-input-start",
                    },
                    c,
                )?;
                self.insert_input(c)?;
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                return Ok(Some(self.finish(c)?));
            }
            if self.token_in(&["form"], c)?.is_some() {
                self.table_error(
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-intable",
                        branch: "form-start",
                    },
                    c,
                )?;
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.current_form.is_some() || self.before.kernel.contains_html_template(c)? {
                    return Ok(Some(self.discard(c)?));
                }
                let node = self.insert_tag(c)?;
                let node = copy_node(&node, c)?;
                write(size_of::<Option<super::head::FormPointer>>(), 1, c)?;
                self.current_form = Some(super::head::FormPointer::Owned(node));
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|script| &script.capability))?;
                if close {
                    self.emit(
                        HeadCondition::UnacknowledgedSelfClosing,
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intable",
                            branch: "form-start/self-closing",
                        },
                        c,
                    )?;
                }
                return Ok(Some(self.finish(c)?));
            }
        }
        if end {
            if self.token_in(&["table"], c)?.is_some() {
                if !self.before.kernel.scoped_name("table", Scope::Table, c)? {
                    self.table_error(
                        crate::construction::evidence::DiagnosticRule {
                            anchor: "parsing-main-intable",
                            branch: "table-end/not-in-scope",
                        },
                        c,
                    )?;
                    return Ok(Some(self.discard(c)?));
                }
                self.pop_table_name("table", c)?;
                self.reset_table_mode(c)?;
                return Ok(Some(self.discard(c)?));
            }
            if self
                .token_in(
                    &[
                        "body", "caption", "col", "colgroup", "html", "tbody", "td", "tfoot", "th",
                        "thead", "tr",
                    ],
                    c,
                )?
                .is_some()
            {
                self.table_error(
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-intable",
                        branch: "disallowed-end-tag",
                    },
                    c,
                )?;
                return Ok(Some(self.discard(c)?));
            }
            if self.token_in(&["template"], c)?.is_some() {
                return Ok(Some(self.end_template(c)?));
            }
        }
        self.foster_body(source, c)
    }
}

#[cfg(test)]
mod owner_tests {
    use super::*;
    use crate::HtmlParseCancellation;

    #[test]
    fn table_pending_constructor_admits_actual_fixed_layout_and_four_fields() {
        let bytes = size_of::<TableCharacters>() as u64;
        for short in [false, true] {
            let selected = super::super::tests::limits(100)
                .into_builder()
                .work_credits(bytes + 4 - u64::from(short))
                .finish()
                .unwrap();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            let result = TableCharacters::new(&mut c);
            if short {
                assert!(result.is_err());
            } else {
                let table = result.unwrap();
                assert_eq!(c.usage().work(WorkCategory::CopyByte), bytes);
                assert_eq!(c.usage().work(WorkCategory::RecordWrite), 4);
                assert_eq!(table.entries.len(), 0);
                // Exhausted normal budget cannot cause disposal to execute work.
                assert!(c.charge(WorkCategory::StateStep, 1).is_err());
                table.entries.release(&mut c).unwrap();
            }
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::AuxiliaryEntries)
                    .current,
                0
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current,
                0
            );
        }
    }

    #[test]
    fn table_pending_constructor_rejects_one_short_atomic_layout_before_record_work() {
        let selected = super::super::tests::limits(100)
            .into_builder()
            .atomic_layout_bytes(size_of::<TableCharacters>() as u64 - 1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        assert!(TableCharacters::new(&mut c).is_err());
        assert_eq!(c.usage().work(WorkCategory::CopyByte), 0);
        assert_eq!(c.usage().work(WorkCategory::RecordWrite), 0);
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::AuxiliaryEntries)
                .current,
            0
        );
    }
}

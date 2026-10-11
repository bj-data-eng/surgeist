// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from tree_builder/mod.rs:713-921,963-1035,1516-1553,1587-1604
// and rules.rs:730-784 at d7232d746d3112f08926d169591576bd12fe2fdf.
// HTML 877d6146487f15b15bc8dd0e3c625063a37b456a governs the original
// token association, intended parents and conditional adoption insertion.

use super::{
    KernelError, Scope, copy_node,
    head::{HeadCondition, HeadError, HeadState, HeadStep},
    prologue::Payload,
    tag_sets::NodeSet,
    write,
};
use crate::{
    construction::parser_session::{FormattingCreation, ParserNode},
    parser::control::{ParseControl, WorkCategory},
};
use std::mem::size_of;

pub(super) const FORMATTING: &[&str] = &[
    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt", "u",
];

/// Only an in-flight creation may have no node. Every element admitted to the
/// active list has its actual arena-qualified node and its original owned tag.
pub(super) struct FormattingEntry {
    /// None is the actual marker, with no fabricated element or creation tag.
    element: Option<FormattingElement>,
}
struct FormattingElement {
    node: Option<ParserNode>,
    creation: FormattingCreation,
}
impl FormattingEntry {
    fn release(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(FormattingElement { creation, .. }) = &mut self.element {
            creation.release(c)?;
        }
        Ok(())
    }
}

enum Bookmark {
    Replace,
    After(ParserNode),
}

impl HeadState<'_> {
    fn format_node(
        &self,
        index: usize,
        c: &mut ParseControl,
    ) -> Result<Option<ParserNode>, HeadError> {
        c.checkpoint()?;
        self.formatting.require_disposal_owner(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        match &self
            .formatting
            .as_slice()
            .get(index)
            .ok_or(HeadError::InvalidFormattingState)?
            .element
        {
            None => Ok(None),
            Some(FormattingElement { node, .. }) => Ok(Some(copy_node(
                node.as_ref().ok_or(HeadError::InvalidFormattingState)?,
                c,
            )?)),
        }
    }
    fn format_index(
        &self,
        target: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<Option<usize>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.formatting.len()).rev() {
            if let Some(node) = self.format_node(index, c)?
                && self.before.kernel.session.same_node(c, &node, target)?
            {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn format_subject(&self, c: &mut ParseControl) -> Result<Option<usize>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.formatting.len()).rev() {
            let Some(node) = self.format_node(index, c)? else {
                break;
            };
            if self.token_matches_node(&node, c)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn format_named(
        &self,
        name: &'static str,
        c: &mut ParseControl,
    ) -> Result<Option<ParserNode>, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.formatting.len()).rev() {
            let Some(node) = self.format_node(index, c)? else {
                break;
            };
            if self.before.kernel.named(&node, name, c)? {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }
    fn format_replace_node(
        &mut self,
        index: usize,
        node: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<(), HeadError> {
        let node = copy_node(node, c)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        let entry = self
            .formatting
            .get_mut(index, c)?
            .ok_or(HeadError::InvalidFormattingState)?;
        let Some(FormattingElement { node: old, .. }) = &mut entry.element else {
            return Err(HeadError::InvalidFormattingState);
        };
        *old = Some(node);
        Ok(())
    }
    fn replay_format(
        &mut self,
        index: usize,
        parent: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<ParserNode, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(FormattingElement { creation, .. }) = &self
            .formatting
            .as_slice()
            .get(index)
            .ok_or(HeadError::InvalidFormattingState)?
            .element
        else {
            return Err(HeadError::InvalidFormattingState);
        };
        Ok(self
            .before
            .kernel
            .session
            .create_formatting_from_creation(c, creation, parent)?)
    }
    /// Stable shifts keep every original recipe in an owned slot if admission
    /// stops midway. No node removal refunds an arena record.
    fn remove_format(&mut self, index: usize, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 2)?;
        if self.formatting_removed.is_some() || index >= self.formatting.len() {
            return Err(HeadError::InvalidFormattingState);
        }
        write(size_of::<Option<FormattingEntry>>(), 1, c)?;
        for shift in index..self.formatting.len() - 1 {
            if !self.formatting.swap_slots(shift, shift + 1, c)? {
                return Err(HeadError::InvalidFormattingState);
            }
        }
        self.formatting_removed = self.formatting.try_pop(c)?;
        Ok(())
    }
    fn dispose_removed_format(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<Option<FormattingEntry>>(), 1, c)?;
        if let Some(entry) = &mut self.formatting_removed {
            entry.release(c)?;
        }
        self.formatting_removed = None;
        Ok(())
    }
    fn erase_format(&mut self, index: usize, c: &mut ParseControl) -> Result<(), HeadError> {
        self.remove_format(index, c)?;
        self.dispose_removed_format(c)
    }
    fn push_format_pending(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<Option<FormattingEntry>>(), 1, c)?;
        let entry = self
            .formatting_pending
            .take()
            .ok_or(HeadError::InvalidFormattingState)?;
        if let Err((stop, entry)) = self.formatting.try_push_recover(c, entry) {
            self.formatting_pending = Some(entry);
            return Err(stop.into());
        }
        Ok(())
    }
    pub(super) fn reconstruct_formatting(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(mut index) = self.formatting.len().checked_sub(1) else {
            return Ok(());
        };
        write(size_of::<usize>(), 1, c)?;
        let Some(last) = self.format_node(index, c)? else {
            return Ok(());
        };
        if self.before.kernel.stack_index(&last, c)?.is_some() {
            return Ok(());
        }
        while index > 0 {
            write(size_of::<usize>(), 1, c)?;
            index -= 1;
            let stop = match self.format_node(index, c)? {
                None => true,
                Some(node) => self.before.kernel.stack_index(&node, c)?.is_some(),
            };
            if stop {
                write(size_of::<usize>(), 1, c)?;
                index += 1;
                break;
            }
        }
        loop {
            let location = self.before.kernel.adjusted_location(None, c)?;
            let node = self.replay_format(index, &location.parent, c)?;
            self.before.kernel.insert_element_at(&node, &location, c)?;
            self.before.kernel.push(&node, c)?;
            self.format_replace_node(index, &node, c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            if index == self.formatting.len() - 1 {
                break;
            }
            write(size_of::<usize>(), 1, c)?;
            index += 1;
        }
        Ok(())
    }
    fn noah(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        write(size_of::<usize>() + size_of::<Option<usize>>(), 2, c)?;
        let mut matches = 0usize;
        let mut earliest = None;
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.formatting.len()).rev() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            let Some(FormattingElement { creation, .. }) =
                &self.formatting.as_slice()[index].element
            else {
                break;
            };
            let Some(FormattingElement {
                creation: pending, ..
            }) = &self
                .formatting_pending
                .as_ref()
                .ok_or(HeadError::InvalidFormattingState)?
                .element
            else {
                return Err(HeadError::InvalidFormattingState);
            };
            if self
                .before
                .kernel
                .session
                .same_formatting_creation(c, creation, pending)?
            {
                write(size_of::<usize>() + size_of::<Option<usize>>(), 2, c)?;
                matches += 1;
                earliest = Some(index);
            }
        }
        if matches >= 3 {
            self.erase_format(earliest.ok_or(HeadError::InvalidFormattingState)?, c)?;
        }
        Ok(())
    }
    pub(super) fn formatting_start(
        &mut self,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        if self.token_in(&["a"], c)?.is_some()
            && let Some(old) = self.format_named("a", c)?
        {
            self.emit(HeadCondition::UnexpectedFormatting, c)?;
            self.adoption(c)?;
            if let Some(index) = self.format_index(&old, c)? {
                self.erase_format(index, c)?;
            }
            if self.before.kernel.stack_index(&old, c)?.is_some() {
                self.before.kernel.remove_stack_identity(&old, c)?;
            }
        }
        self.reconstruct_formatting(c)?;
        if self.token_in(&["nobr"], c)?.is_some()
            && self.before.kernel.scoped_name("nobr", Scope::Default, c)?
        {
            self.emit(HeadCondition::UnexpectedFormatting, c)?;
            self.adoption(c)?;
            self.reconstruct_formatting(c)?;
        }
        c.charge(WorkCategory::RecordVisit, 2)?;
        let pending = self.pending.as_mut().ok_or(HeadError::MissingInput)?;
        let Payload::Tag(input) = &mut pending.payload else {
            return Err(HeadError::MissingInput);
        };
        c.charge(WorkCategory::ProviderCall, 1)?;
        let source = pending.source.clone();
        write(size_of::<Option<FormattingEntry>>(), 4, c)?;
        self.formatting_pending = Some(FormattingEntry {
            element: Some(FormattingElement {
                node: None,
                creation: self
                    .before
                    .kernel
                    .session
                    .retain_formatting_creation(c, input, source)?,
            }),
        });
        let location = self.before.kernel.adjusted_location(None, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let Some(FormattingElement {
            creation,
            node: stored,
        }) = &mut self
            .formatting_pending
            .as_mut()
            .ok_or(HeadError::InvalidFormattingState)?
            .element
        else {
            return Err(HeadError::InvalidFormattingState);
        };
        let node = self.before.kernel.session.create_formatting_from_creation(
            c,
            creation,
            &location.parent,
        )?;
        let retained = copy_node(&node, c)?;
        write(size_of::<Option<ParserNode>>(), 1, c)?;
        *stored = Some(retained);
        self.before.kernel.insert_element_at(&node, &location, c)?;
        self.before.kernel.push(&node, c)?;
        self.noah(c)?;
        self.push_format_pending(c)?;
        if close {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    pub(super) fn marker_start(
        &mut self,
        close: bool,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        self.reconstruct_formatting(c)?;
        self.insert_tag(c)?;
        write(size_of::<Option<FormattingEntry>>(), 2, c)?;
        self.formatting_pending = Some(FormattingEntry { element: None });
        self.push_format_pending(c)?;
        self.frameset(false, c)?;
        if close {
            self.emit(HeadCondition::UnacknowledgedSelfClosing, c)?;
        }
        self.finish(c)
    }
    pub(super) fn marker_end(
        &mut self,
        name: &'static str,
        c: &mut ParseControl,
    ) -> Result<HeadStep, HeadError> {
        if !self.before.kernel.scoped_name(name, Scope::Default, c)? {
            self.emit(HeadCondition::UnexpectedEndTag, c)?;
            return self.discard(c);
        }
        self.before.kernel.implied_ends(None, c)?;
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
        loop {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let index = self
                .formatting
                .len()
                .checked_sub(1)
                .ok_or(HeadError::InvalidFormattingState)?;
            let marker = self.format_node(index, c)?.is_none();
            self.erase_format(index, c)?;
            if marker {
                break;
            }
        }
        self.discard(c)
    }
    pub(super) fn adoption(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        let current = self.before.kernel.current(c)?;
        if self.token_matches_node(&current, c)? && self.format_index(&current, c)?.is_none() {
            self.before.kernel.pop(c)?;
            return Ok(());
        }
        for _ in 0..8 {
            c.charge(WorkCategory::StateStep, 1)?;
            let Some(format_index) = self.format_subject(c)? else {
                return self.generic_end(c);
            };
            let formatting = self
                .format_node(format_index, c)?
                .ok_or(HeadError::InvalidFormattingState)?;
            let Some(open_index) = self.before.kernel.stack_index(&formatting, c)? else {
                self.emit(HeadCondition::UnexpectedFormatting, c)?;
                self.erase_format(format_index, c)?;
                return Ok(());
            };
            if !self
                .before
                .kernel
                .in_scope(&formatting, Scope::Default, c)?
            {
                self.emit(HeadCondition::UnexpectedFormatting, c)?;
                return Ok(());
            }
            let current = self.before.kernel.current(c)?;
            if !self
                .before
                .kernel
                .session
                .same_node(c, &current, &formatting)?
            {
                self.emit(HeadCondition::UnexpectedFormatting, c)?;
            }
            let mut furthest = None;
            write(size_of::<Option<(usize, ParserNode)>>(), 1, c)?;
            c.charge(WorkCategory::RecordVisit, 1)?;
            for index in open_index + 1..self.before.kernel.stack()?.len() {
                let node = self.before.kernel.at(index, c)?;
                if self.before.kernel.node_set(&node, NodeSet::Special, c)? {
                    write(size_of::<Option<(usize, ParserNode)>>(), 2, c)?;
                    furthest = Some((index, node));
                    break;
                }
            }
            let Some((mut cursor, furthest)) = furthest else {
                self.before.kernel.pop_through(&formatting, c)?;
                self.erase_format(format_index, c)?;
                return Ok(());
            };
            let ancestor = self.before.kernel.at(
                open_index
                    .checked_sub(1)
                    .ok_or(HeadError::InvalidFormattingState)?,
                c,
            )?;
            let mut bookmark = Bookmark::Replace;
            let mut last = copy_node(&furthest, c)?;
            let mut inner = 0usize;
            write(
                size_of::<Bookmark>() + size_of::<usize>() * 2 + size_of::<ParserNode>(),
                4,
                c,
            )?;
            loop {
                write(size_of::<usize>() * 2, 2, c)?;
                inner += 1;
                cursor = cursor
                    .checked_sub(1)
                    .ok_or(HeadError::InvalidFormattingState)?;
                let node = self.before.kernel.at(cursor, c)?;
                if self
                    .before
                    .kernel
                    .session
                    .same_node(c, &node, &formatting)?
                {
                    break;
                }
                if inner > 3
                    && let Some(index) = self.format_index(&node, c)?
                {
                    self.erase_format(index, c)?;
                }
                let Some(index) = self.format_index(&node, c)? else {
                    self.before.kernel.remove_stack_identity(&node, c)?;
                    continue;
                };
                let replacement = self.replay_format(index, &ancestor, c)?;
                self.format_replace_node(index, &replacement, c)?;
                self.before.kernel.replace_stack(cursor, &replacement, c)?;
                if self.before.kernel.session.same_node(c, &last, &furthest)? {
                    let after = copy_node(&replacement, c)?;
                    write(size_of::<Bookmark>(), 1, c)?;
                    bookmark = Bookmark::After(after);
                }
                self.before
                    .kernel
                    .session
                    .insert(c, &replacement, &last, None)?;
                write(size_of::<ParserNode>(), 1, c)?;
                last = replacement;
            }
            let location = self.before.kernel.adjusted_location(Some(&ancestor), c)?;
            if self.before.kernel.session.parent(c, &last)?.is_some() {
                self.before.kernel.session.remove(c, &last)?;
            }
            let eligible = self
                .before
                .kernel
                .session
                .element_insertion_ignored(c, &last, &location.parent)?
                .is_none();
            let reference_matches = if !eligible {
                false
            } else if let Some(reference) = &location.before {
                match self.before.kernel.session.parent(c, reference)? {
                    Some(parent) => {
                        self.before
                            .kernel
                            .session
                            .same_node(c, &parent, &location.parent)?
                    }
                    None => false,
                }
            } else {
                true
            };
            if reference_matches {
                self.before.kernel.session.insert(
                    c,
                    &location.parent,
                    &last,
                    location.before.as_ref(),
                )?;
            }
            let index = self
                .format_index(&formatting, c)?
                .ok_or(HeadError::InvalidFormattingState)?;
            let replacement = self.replay_format(index, &furthest, c)?;
            self.before
                .kernel
                .session
                .reparent_children(c, &furthest, &replacement)?;
            self.before
                .kernel
                .session
                .insert(c, &furthest, &replacement, None)?;
            self.remove_format(index, c)?;
            let insertion = match bookmark {
                Bookmark::Replace => index,
                Bookmark::After(after) => {
                    self.format_index(&after, c)?
                        .ok_or(HeadError::InvalidFormattingState)?
                        + 1
                }
            };
            let stored = copy_node(&replacement, c)?;
            write(size_of::<Option<ParserNode>>(), 1, c)?;
            let Some(FormattingElement { node, .. }) = &mut self
                .formatting_removed
                .as_mut()
                .ok_or(HeadError::InvalidFormattingState)?
                .element
            else {
                return Err(HeadError::InvalidFormattingState);
            };
            *node = Some(stored);
            write(size_of::<Option<FormattingEntry>>(), 1, c)?;
            let entry = self
                .formatting_removed
                .take()
                .ok_or(HeadError::InvalidFormattingState)?;
            if let Err((stop, entry)) = self.formatting.try_push_recover(c, entry) {
                self.formatting_removed = Some(entry);
                return Err(stop.into());
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            for shift in (insertion..self.formatting.len() - 1).rev() {
                if !self.formatting.swap_slots(shift, shift + 1, c)? {
                    return Err(HeadError::InvalidFormattingState);
                }
            }
            self.before.kernel.remove_stack_identity(&formatting, c)?;
            let position = self
                .before
                .kernel
                .stack_index(&furthest, c)?
                .ok_or(HeadError::InvalidFormattingState)?;
            self.before
                .kernel
                .insert_stack(position + 1, &replacement, c)?;
        }
        Ok(())
    }
    pub(super) fn release_formatting(&mut self, c: &mut ParseControl) -> Result<(), HeadError> {
        if c.stop().is_none() {
            self.formatting.require_owner(c)?;
            write(size_of::<Option<FormattingEntry>>() * 2, 2, c)?;
        }
        if let Some(entry) = &mut self.formatting_pending {
            entry.release(c)?;
        }
        self.formatting_pending = None;
        if let Some(entry) = &mut self.formatting_removed {
            entry.release(c)?;
        }
        self.formatting_removed = None;
        loop {
            if c.stop().is_none() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self.formatting.len() == 0 {
                    break;
                }
                write(size_of::<Option<FormattingEntry>>() * 2, 2, c)?;
            }
            let entry = if c.stop().is_some() {
                self.formatting.cleanup_pop(c)?
            } else {
                self.formatting.try_pop(c)?
            };
            let Some(entry) = entry else { break };
            self.formatting_removed = Some(entry);
            self.formatting_removed
                .as_mut()
                .expect("retained disposal owner")
                .release(c)?;
            self.formatting_removed = None;
        }
        Ok(())
    }
}

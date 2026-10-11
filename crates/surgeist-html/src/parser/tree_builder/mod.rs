// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from html5ever/src/tree_builder/mod.rs at
// d7232d746d3112f08926d169591576bd12fe2fdf, specifically insertion location
// and stack/scope operations. HTML 877d6146487f15b15bc8dd0e3c625063a37b456a
// governs the fragment substitution and newly-created template state.

//! Concrete tree operations, document prologue and bounded owning head dispatch
//! over one construction arena. Script/template and AfterHead continuations
//! retain unfinished input; complete parsing and report publication are later.

mod doctype;
mod head;
mod prologue;
mod tag_sets;
#[cfg(test)]
mod tests;

use super::{
    control::{ControlIdentity, OwnedVec, ParseControl, ParseStop, WorkCategory},
    tokenizer::token::{Tag, Text},
};
use crate::{
    HtmlParseResource,
    construction::parser_session::{
        ElementInsertionIgnored, ParserArenaSession, ParserNode, SessionError, TextInsertion,
    },
};
use std::mem::size_of;
pub(crate) use tag_sets::Scope;

#[derive(Debug)]
pub(crate) enum KernelError {
    Stop(ParseStop),
    Session(SessionError),
    EmptyStack,
    NotElement,
    FirstOpenElementNotHtml,
    DuplicateOpenElement,
    Consumed,
}
impl From<ParseStop> for KernelError {
    fn from(value: ParseStop) -> Self {
        Self::Stop(value)
    }
}
impl From<SessionError> for KernelError {
    fn from(value: SessionError) -> Self {
        Self::Session(value)
    }
}

/// Identities remain qualified to the one session. A reference child is an
/// ordinary child of the selected parent, never a template association.
pub(crate) struct InsertionLocation {
    parent: ParserNode,
    before: Option<ParserNode>,
}

pub(crate) enum StackPush {
    Push,
    NoPush,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementInsertionOutcome {
    Inserted,
    Ignored(ElementInsertionIgnored),
}

pub(crate) struct CreatedElement {
    pub(crate) node: ParserNode,
    pub(crate) insertion: ElementInsertionOutcome,
}

/// Owns algorithm state and the existing native session, not a second tree.
pub(crate) struct LocationKernel {
    owner: ControlIdentity,
    session: ParserArenaSession,
    open: Option<OwnedVec<ParserNode>>,
    root_target: Option<ParserNode>,
    foster_parenting: bool,
}

fn write(bytes: usize, writes: usize, control: &mut ParseControl) -> Result<(), ParseStop> {
    control.admit_atomic_layout(bytes)?;
    control.charge(WorkCategory::CopyByte, bytes as u64)?;
    control.charge(WorkCategory::RecordWrite, writes as u64)
}

fn copy_node(node: &ParserNode, control: &mut ParseControl) -> Result<ParserNode, ParseStop> {
    control.charge(WorkCategory::ProviderCall, 1)?;
    control.charge(WorkCategory::CopyByte, size_of::<ParserNode>() as u64)?;
    Ok(node.clone())
}

impl LocationKernel {
    /// The source-free native root kind establishes fragment substitution. No
    /// captured host is manufactured as an arena node or guessed from a flag.
    pub(crate) fn new(
        session: &mut Option<ParserArenaSession>,
        control: &mut ParseControl,
    ) -> Result<Self, KernelError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let root_target = session
            .as_ref()
            .ok_or(KernelError::Consumed)?
            .fragment_root(control)?;
        let open = OwnedVec::new(HtmlParseResource::OpenElements)?;
        write(size_of::<Self>(), 5, control)?;
        write(size_of::<Option<ParserArenaSession>>(), 1, control)?;
        control.charge(WorkCategory::ProviderCall, 1)?;
        Ok(Self {
            owner: control.identity(),
            session: session
                .take()
                .expect("verified session remains owned until commit"),
            open: Some(open),
            root_target,
            foster_parenting: false,
        })
    }

    fn require(&self, control: &mut ParseControl) -> Result<(), KernelError> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.open.as_ref().ok_or(KernelError::Consumed)?;
        Ok(())
    }

    fn stack(&self) -> Result<&OwnedVec<ParserNode>, KernelError> {
        self.open.as_ref().ok_or(KernelError::Consumed)
    }

    fn at(&self, index: usize, control: &mut ParseControl) -> Result<ParserNode, KernelError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let node = self
            .stack()?
            .as_slice()
            .get(index)
            .ok_or(KernelError::EmptyStack)?;
        Ok(copy_node(node, control)?)
    }

    fn named(
        &self,
        node: &ParserNode,
        name: &'static str,
        control: &mut ParseControl,
    ) -> Result<bool, KernelError> {
        let Some(qualified) = self.session.element_name(control, node)? else {
            return Ok(false);
        };
        Ok(tag_sets::html_named(qualified, name, control)?)
    }

    pub(crate) fn push(
        &mut self,
        node: &ParserNode,
        control: &mut ParseControl,
    ) -> Result<(), KernelError> {
        self.require(control)?;
        if self.session.element_name(control, node)?.is_none() {
            return Err(KernelError::NotElement);
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.stack()?.len() == 0 && !self.named(node, "html", control)? {
            return Err(KernelError::FirstOpenElementNotHtml);
        }
        for index in 0..self.stack()?.len() {
            control.charge(WorkCategory::StateStep, 1)?;
            let existing = self.at(index, control)?;
            if self.session.same_node(control, &existing, node)? {
                return Err(KernelError::DuplicateOpenElement);
            }
        }
        let node = copy_node(node, control)?;
        self.open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_push(control, node)?;
        Ok(())
    }

    /// Popping a stack slot does not detach or refund the retained arena node.
    pub(crate) fn pop(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<ParserNode>, KernelError> {
        self.require(control)?;
        Ok(self
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_pop(control)?)
    }

    pub(crate) fn set_foster_parenting(
        &mut self,
        enabled: bool,
        control: &mut ParseControl,
    ) -> Result<(), KernelError> {
        self.require(control)?;
        write(size_of::<bool>(), 1, control)?;
        self.foster_parenting = enabled;
        Ok(())
    }

    /// Target matching precedes scope-boundary matching, as §13.2.4.2 requires.
    pub(crate) fn in_scope(
        &self,
        target: &ParserNode,
        scope: Scope,
        control: &mut ParseControl,
    ) -> Result<bool, KernelError> {
        self.require(control)?;
        self.session.validate_node(control, target)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            control.charge(WorkCategory::StateStep, 1)?;
            let node = self.at(index, control)?;
            if self.session.same_node(control, &node, target)? {
                return Ok(true);
            }
            let qualified = self
                .session
                .element_name(control, &node)?
                .ok_or(KernelError::NotElement)?;
            if tag_sets::scope_boundary(qualified, scope, control)? {
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// Actual topmost stack element, used by the narrow InBody-html branch.
    fn first(&self, control: &mut ParseControl) -> Result<ParserNode, KernelError> {
        self.require(control)?;
        self.at(0, control)
    }

    /// Namespace-qualified membership, not ordinary tree ancestry or scope.
    fn contains_html_template(&self, control: &mut ParseControl) -> Result<bool, KernelError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            control.charge(WorkCategory::StateStep, 1)?;
            let node = self.at(index, control)?;
            if self.named(&node, "template", control)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Implements the appropriate-place algorithm followed by root-target
    /// substitution. An existing insertion location contributes only its parent
    /// as overrideTarget; its old reference child is not carried forward.
    pub(crate) fn adjusted_location(
        &self,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<InsertionLocation, KernelError> {
        self.require(control)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let last = self
            .stack()?
            .len()
            .checked_sub(1)
            .ok_or(KernelError::EmptyStack)?;
        let mut parent = match override_target {
            Some(node) => {
                self.session.validate_node(control, node)?;
                copy_node(node, control)?
            }
            None => self.at(last, control)?,
        };
        let mut before = None;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let foster = if self.foster_parenting {
            match self.session.element_name(control, &parent)? {
                Some(name) => tag_sets::foster_target(name, control)?,
                None => false,
            }
        } else {
            false
        };
        if foster {
            let mut selected = false;
            control.charge(WorkCategory::RecordVisit, 1)?;
            for index in (0..self.stack()?.len()).rev() {
                control.charge(WorkCategory::StateStep, 1)?;
                let node = self.at(index, control)?;
                if self.named(&node, "template", control)? {
                    parent = node;
                    selected = true;
                    break;
                }
                if self.named(&node, "table", control)? {
                    if let Some(actual_parent) = self.session.parent(control, &node)? {
                        parent = actual_parent;
                        before = Some(node);
                    } else {
                        parent = self.at(
                            index
                                .checked_sub(1)
                                .ok_or(KernelError::FirstOpenElementNotHtml)?,
                            control,
                        )?;
                    }
                    selected = true;
                    break;
                }
            }
            if !selected {
                parent = self.at(0, control)?;
            }
        }
        if self.named(&parent, "template", control)? {
            // Only actual parser creation establishes the initially-null patch
            // target/markers. Imported template facts remain unavailable.
            parent = self
                .session
                .unpatched_template_contents(control, &parent)?
                .ok_or(KernelError::NotElement)?;
            before = None;
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(root_target) = &self.root_target {
            let first = self.at(0, control)?;
            if self.session.same_node(control, &parent, &first)? {
                parent = copy_node(root_target, control)?;
                before = None;
            }
        }
        write(size_of::<InsertionLocation>(), 2, control)?;
        Ok(InsertionLocation { parent, before })
    }

    /// Inserts an existing element at the selected location. The pinned step-2
    /// guards return an ignored outcome without detaching or moving the element.
    /// Native topology and association ancestry are inspected by their owner.
    pub(crate) fn insert_element(
        &mut self,
        element: &ParserNode,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<ElementInsertionOutcome, KernelError> {
        self.require(control)?;
        if self.session.element_name(control, element)?.is_none() {
            return Err(KernelError::NotElement);
        }
        let location = self.adjusted_location(override_target, control)?;
        self.insert_element_at(element, &location, control)
    }

    fn insert_element_at(
        &mut self,
        element: &ParserNode,
        location: &InsertionLocation,
        control: &mut ParseControl,
    ) -> Result<ElementInsertionOutcome, KernelError> {
        let outcome =
            match self
                .session
                .element_insertion_ignored(control, element, &location.parent)?
            {
                Some(reason) => ElementInsertionOutcome::Ignored(reason),
                None => {
                    self.session.insert(
                        control,
                        &location.parent,
                        element,
                        location.before.as_ref(),
                    )?;
                    ElementInsertionOutcome::Inserted
                }
            };
        write(size_of::<ElementInsertionOutcome>(), 1, control)?;
        Ok(outcome)
    }

    /// Transfers a real lexical HTML start tag and retains its created element
    /// even when insertion is ignored. The requested stack push follows the
    /// insertion operation in both cases (§13.2.6.1 insert-foreign steps 3–5).
    /// Insertion-mode policy and implicit tags are later.
    pub(crate) fn insert_html_tag(
        &mut self,
        input: &mut Option<Tag>,
        push: StackPush,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<CreatedElement, KernelError> {
        let location = self.adjusted_location(override_target, control)?;
        let node = self.session.allocate_html_tag(control, input)?;
        let insertion = self.insert_element_at(&node, &location, control)?;
        if matches!(push, StackPush::Push) {
            self.push(&node, control)?;
        }
        write(size_of::<CreatedElement>(), 2, control)?;
        Ok(CreatedElement { node, insertion })
    }

    pub(crate) fn insert_text(
        &mut self,
        input: &mut Option<Text>,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<Option<TextInsertion>, KernelError> {
        let location = self.adjusted_location(override_target, control)?;
        Ok(self
            .session
            .insert_text(control, &location.parent, location.before.as_ref(), input)?)
    }

    pub(crate) fn release(&mut self, control: &mut ParseControl) -> Result<(), KernelError> {
        // Validate ownership even after a sticky stop, before any consumption.
        // Zero-unit cleanup performs no useful parsing or new budget admission.
        if control.stop().is_some() {
            control.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        } else if let Err(stop) = control.require_identity(&self.owner) {
            control
                .cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)
                .map_err(|_| KernelError::Stop(stop))?;
        }
        self.open.as_ref().ok_or(KernelError::Consumed)?;
        // Session checks aliases without taking its storage on failure, so a
        // borrowed kernel retains all owners for a later valid cleanup retry.
        self.session.release(control)?;
        self.open
            .take()
            .expect("verified active stack")
            .release(control)?;
        self.root_target = None;
        Ok(())
    }
}

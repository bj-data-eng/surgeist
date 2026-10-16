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
//! over one construction arena. The fresh sealed Disabled driver handles actual
//! scripts; a general proofless head stage suspends at script. Ordinary
//! body/block/list/ruby, formatting/reconstruction/adoption, applet/marquee and
//! trailing modes, ordinary form pointer recovery, image and listed non-resettable
//! button/fieldset/object form associations and bounded structural tables are
//! admitted, together with ordinary standalone frameset entry and trailing modes,
//! textarea RCDATA, raw child-text hooks and parser-pop reset, output creation,
//! ordered Hidden-input creation, and ordinary select/option/selectedcontent
//! effects, clones and intrinsic FIFO work. Ordinary non-script MathML/SVG foreign
//! dispatch and qualified cloning are admitted. Fresh ordinary templates retain
//! real inert document bindings, contents, insertion modes and PI-based patching.
//! The sealed ordinary execution owner also handles qualified SVG scripts.
//! Context fragments retain actual captured documents, registries, form pointers,
//! distinct parser/output roots and qualified declarative-shadow construction
//! effects. Inert fragment scripts remain distinct from Disabled preparation.
//! Other resettable controls and unavailable execution/capture domains retain
//! typed refusals. Pending intrinsic work refuses freezing; immutable native
//! completion is not a public parse/report assembly claim.

mod body;
#[cfg(test)]
mod conformance;
mod context;
#[cfg(test)]
pub(crate) use context::ContextFragmentDriver;
#[cfg(test)]
pub(crate) use context::initialized_driver as initialized_fragment_fixture;
mod doctype;
mod end;
pub(crate) mod entry;
mod foreign;
mod formatting;
mod frameset;
pub(crate) mod head;
pub(crate) mod parse;
pub(crate) mod prologue;
#[cfg(test)]
mod provenance_scaling_tests;
pub(crate) mod report;
pub(crate) mod seal;
mod tables;
mod tag_sets;
mod template;
mod terminal;
#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
use super::tokenizer::token::Tag;
use super::{
    control::{
        ControlIdentity, OwnedVec, ParseControl, ParseStop, StandaloneUnsupported, WorkCategory,
    },
    tokenizer::token::Text,
};
use crate::{
    HtmlParseResource,
    construction::{
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
/// The element creation location precedes fragment root redirection. Its
/// identity cannot be passed as an adjusted insertion location accidentally.
pub(crate) struct AppropriatePlace {
    parent: ParserNode,
    before: Option<ParserNode>,
}

#[cfg(test)]
pub(crate) enum StackPush {
    Push,
    NoPush,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementInsertionOutcome {
    Inserted,
    Ignored(ElementInsertionIgnored),
}

#[cfg(test)]
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
    fn stack_index(
        &self,
        target: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<Option<usize>, KernelError> {
        self.require(c)?;
        self.session.validate_node(c, target)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            let node = self.at(index, c)?;
            if self.session.same_node(c, &node, target)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn replace_stack(
        &mut self,
        index: usize,
        target: &ParserNode,
        c: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<(), KernelError> {
        self.require(c)?;
        self.session.validate_node(c, target)?;
        let removed = self.at(index, c)?;
        self.session
            .qualify_stack_removal(c, &removed, capability)?;
        let node = copy_node(target, c)?;
        write(size_of::<ParserNode>(), 1, c)?;
        let slot = self
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .get_mut(index, c)?
            .ok_or(KernelError::EmptyStack)?;
        *slot = node;
        self.session.parser_stack_removed(c, &removed, capability)?;
        Ok(())
    }
    fn insert_stack(
        &mut self,
        index: usize,
        target: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<(), KernelError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if index > self.stack()?.len() {
            return Err(KernelError::EmptyStack);
        }
        self.push(target, c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        for shift in (index..self.stack()?.len() - 1).rev() {
            if !self
                .open
                .as_mut()
                .ok_or(KernelError::Consumed)?
                .swap_slots(shift, shift + 1, c)?
            {
                return Err(KernelError::EmptyStack);
            }
        }
        Ok(())
    }
    fn current(&self, c: &mut ParseControl) -> Result<ParserNode, KernelError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.at(
            self.stack()?
                .len()
                .checked_sub(1)
                .ok_or(KernelError::EmptyStack)?,
            c,
        )
    }
    fn node_set(
        &self,
        node: &ParserNode,
        set: tag_sets::NodeSet,
        c: &mut ParseControl,
    ) -> Result<bool, KernelError> {
        let name = self
            .session
            .element_name(c, node)?
            .ok_or(KernelError::NotElement)?;
        Ok(tag_sets::in_set(name, set, c)?)
    }
    fn scoped_name(
        &self,
        name: &'static str,
        scope: Scope,
        c: &mut ParseControl,
    ) -> Result<bool, KernelError> {
        self.require(c)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            let node = self.at(index, c)?;
            if self.named(&node, name, c)? {
                return Ok(true);
            }
            let q = self
                .session
                .element_name(c, &node)?
                .ok_or(KernelError::NotElement)?;
            if tag_sets::scope_boundary(q, scope, c)? {
                return Ok(false);
            }
        }
        Ok(false)
    }
    fn implied_ends(
        &mut self,
        except: Option<&'static str>,
        c: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<(), KernelError> {
        loop {
            let node = self.current(c)?;
            if let Some(except) = except
                && self.named(&node, except, c)?
            {
                return Ok(());
            }
            if !self.node_set(&node, tag_sets::NodeSet::Implied, c)? {
                return Ok(());
            }
            self.pop(c, capability)?;
        }
    }
    fn pop_through(
        &mut self,
        target: &ParserNode,
        c: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<(), KernelError> {
        loop {
            let node = self.pop(c, capability)?.ok_or(KernelError::EmptyStack)?;
            if self.session.same_node(c, &node, target)? {
                return Ok(());
            }
        }
    }
    fn remove_stack_identity(
        &mut self,
        target: &ParserNode,
        c: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<(), KernelError> {
        self.require(c)?;
        self.session.validate_node(c, target)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let len = self.stack()?.len();
        for index in 0..len {
            let node = self.at(index, c)?;
            if self.session.same_node(c, &node, target)? {
                self.session.qualify_stack_removal(c, &node, capability)?;
                for shift in index..len - 1 {
                    if !self
                        .open
                        .as_mut()
                        .ok_or(KernelError::Consumed)?
                        .swap_slots(shift, shift + 1, c)?
                    {
                        return Err(KernelError::EmptyStack);
                    }
                }
                self.pop(c, capability)?;
                return Ok(());
            }
        }
        Err(KernelError::EmptyStack)
    }
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
        let copied = copy_node(node, control)?;
        self.open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_push(control, copied)?;
        self.session.parser_stack_added(control, node)?;
        Ok(())
    }

    /// Popping a stack slot does not detach or refund the retained arena node.
    pub(crate) fn pop(
        &mut self,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<Option<ParserNode>, KernelError> {
        self.require(control)?;
        if self.stack()?.len() != 0 {
            let current = self.current(control)?;
            self.session
                .qualify_stack_removal(control, &current, capability)?;
        }
        let node = self
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_pop(control)?;
        if let Some(node) = &node {
            self.session
                .parser_stack_removed(control, node, capability)?;
        }
        Ok(node)
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
    fn last_template(&self, control: &mut ParseControl) -> Result<Option<ParserNode>, KernelError> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            let node = self.at(index, control)?;
            if self.named(&node, "template", control)? {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }

    /// §13.2.6.1 appropriate place, including foster/template routing. Fragment
    /// root substitution belongs to the subsequent insertion operation.
    pub(crate) fn appropriate_place(
        &self,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<AppropriatePlace, KernelError> {
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
            let state = self.session.template_state(control, &parent)?;
            if let Some(target) = state.target {
                before = if let Some(end) = state.end {
                    if let Some(actual) = self.session.parent(control, &end)?
                        && self.session.same_node(control, &actual, &target)?
                    {
                        Some(end)
                    } else {
                        None
                    }
                } else {
                    None
                };
                parent = target;
            } else {
                parent = state.contents;
                before = None;
            }
        }
        write(size_of::<AppropriatePlace>(), 2, control)?;
        Ok(AppropriatePlace { parent, before })
    }

    /// An existing source location contributes its parent as overrideTarget;
    /// its old reference child is not carried forward. Root redirection occurs
    /// here, after creation, and canonical insertion performs actual adoption.
    pub(crate) fn adjusted_location(
        &self,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<InsertionLocation, KernelError> {
        let AppropriatePlace {
            mut parent,
            mut before,
        } = self.appropriate_place(override_target, control)?;
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
    #[cfg(test)]
    pub(crate) fn insert_element(
        &mut self,
        element: &ParserNode,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<ElementInsertionOutcome, KernelError> {
        self.require(control)?;
        if self.session.element_name(control, element)?.is_none() {
            return Err(KernelError::NotElement);
        }
        let location = self.adjusted_location(override_target, control)?;
        self.insert_element_at(element, &location, control, capability)
    }

    fn insert_element_at(
        &mut self,
        element: &ParserNode,
        location: &InsertionLocation,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<ElementInsertionOutcome, KernelError> {
        let outcome =
            match self
                .session
                .element_insertion_ignored(control, element, &location.parent)?
            {
                Some(reason) => ElementInsertionOutcome::Ignored(reason),
                None => {
                    if let Some(capability) = capability {
                        self.session.insert_parser(
                            control,
                            &location.parent,
                            element,
                            location.before.as_ref(),
                            capability,
                        )?;
                    } else {
                        self.session.insert(
                            control,
                            &location.parent,
                            element,
                            location.before.as_ref(),
                            None,
                        )?;
                    }
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
    #[cfg(test)]
    pub(crate) fn insert_html_tag(
        &mut self,
        input: &mut Option<Tag>,
        push: StackPush,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<CreatedElement, KernelError> {
        let location = self.adjusted_location(override_target, control)?;
        let node = self.session.allocate_html_tag(control, input)?;
        let insertion = self.insert_element_at(&node, &location, control, capability)?;
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
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<Option<TextInsertion>, KernelError> {
        let location = self.adjusted_location(override_target, control)?;
        Ok(self.session.insert_text(
            control,
            &location.parent,
            location.before.as_ref(),
            input,
            capability,
        )?)
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
        self.release_stack(control)
    }
    /// The completed assembler already transferred the actual session into its
    /// retained FrozenArena. Only the original stack storage remains here.
    pub(super) fn release_frozen_stack(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<(), KernelError> {
        if control.stop().is_some() {
            control.cleanup_release(&self.owner, HtmlParseResource::AuxiliaryEntries, 0)?;
        } else {
            control.require_identity(&self.owner)?;
        }
        self.release_stack(control)
    }
    fn release_stack(&mut self, control: &mut ParseControl) -> Result<(), KernelError> {
        self.open
            .take()
            .expect("verified active stack")
            .release(control)?;
        self.root_target = None;
        Ok(())
    }
}

/// Sealed execution eligibility issued by the real ordinary-document or
/// context-fragment request/bootstrap owner. Source/control and actual arena
/// identity travel together; captured facts remain in the same session.
/// There are no host/realm/settings objects, executable registry/callbacks,
/// observers/listeners, external mutation/feed hooks, loader/speculative parser,
/// live nodes, JS stack, promise recipients, IDB transactions or kept JS objects.
/// Admitted producers are real lexical and implicit HTML construction, attribute
/// merging, ordinary placement, and the supported control/clone/effect owners.
/// Their intrinsic tasks share this arena's controlled FIFO; no external task or
/// execution domain is admitted. Managed template contents have distinct inert
/// node documents; supported SVG scripts retain their own processed state under
/// explicit no-loader, Disabled execution and no-notification-recipient closure.
/// Imported elements remain outside these producers. The ordinary-document
/// constructor cannot attach shadow roots; the qualified fragment profile can
/// construct local roots and retain captured-host attachment effects. Immutable
/// capture facts do not create a live DOM or notification-recipient domain.
/// Structural records and metadata remain retained.
/// No running-script/frame-timing domain exists: checkpoint specialization does
/// not implement an external timing algorithm or assume a normative null return.
pub(crate) struct ConstructionCapability<'source> {
    source: &'source crate::HtmlSource,
    owner: ControlIdentity,
    document: ParserNode,
    profile: ExecutionProfile,
    images: crate::HtmlImageSupport,
}
/// Issued only after the corresponding real request/bootstrap is admitted.
/// The fragment's scripting/attachment facts remain in the sole session owner.
#[derive(Clone, Copy)]
enum ExecutionProfile {
    Ordinary,
    Fragment,
}
impl ConstructionCapability<'_> {
    pub(crate) fn details_notification_domain(
        &self,
        c: &mut ParseControl,
        session: &ParserArenaSession,
    ) -> Result<bool, SessionError> {
        self.validate_session(c, session)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        // Both real issuers admit only this source/controller/arena's closed
        // construction producers. A fragment's temporary parser Document and
        // owned inert/ordinary trees are not its immutable captured target.
        // Registry definitions requiring reactions refuse before construction;
        // captures introduce no live nodes, listeners, task recipients or reentry.
        // Every Details consumer uses canonical attributes/tree/provenance only.
        // Toggle has no intrinsic follow-on consumer in this closed owner, so
        // its task/tracker can be eliminated while retaining exclusivity hooks.
        Ok(true)
    }
    pub(crate) fn validate_session(
        &self,
        c: &mut ParseControl,
        session: &ParserArenaSession,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.owner)?;
        c.charge(WorkCategory::RecordVisit, 3)?;
        c.require_source(self.source)?;
        if matches!(self.profile, ExecutionProfile::Ordinary) {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.images != crate::HtmlImageSupport::Disabled {
                return Err(SessionError::ContextFactUnavailable(
                    crate::HtmlCaptureFact::ImageSupport,
                ));
            }
        }
        match self.profile {
            ExecutionProfile::Ordinary => session.validate_standalone_proof(c, &self.document)?,
            ExecutionProfile::Fragment => session.validate_fragment_proof(c, &self.document)?,
        }
        Ok(())
    }
    pub(crate) fn validate_fragment_session(
        &self,
        c: &mut ParseControl,
        session: &ParserArenaSession,
    ) -> Result<(), SessionError> {
        self.validate_session(c, session)?;
        if !matches!(self.profile, ExecutionProfile::Fragment) {
            return Err(c
                .stop_unsupported_standalone(StandaloneUnsupported::Provenance)
                .into());
        }
        Ok(())
    }
    fn checkpoint(
        &self,
        c: &mut ParseControl,
        session: &mut ParserArenaSession,
    ) -> Result<(), SessionError> {
        self.validate_session(c, session)?;
        c.charge(WorkCategory::StateStep, 1)?;
        session.perform_intrinsic_checkpoint(c, self)
    }
}

pub(crate) enum StandaloneError {
    UnsupportedDocumentInitialization,
    UnsupportedExecutionProfile,
    Stop(ParseStop),
    Prologue(prologue::PrologueError),
    Head(head::HeadError),
    Consumed,
    TokenizerExhausted,
}
impl std::fmt::Debug for StandaloneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedDocumentInitialization => {
                f.write_str("UnsupportedDocumentInitialization")
            }
            Self::UnsupportedExecutionProfile => f.write_str("UnsupportedExecutionProfile"),
            Self::Stop(error) => f.debug_tuple("Stop").field(error).finish(),
            Self::Prologue(error) => f.debug_tuple("Prologue").field(error).finish(),
            Self::Head(error) => f.debug_tuple("Head").field(error).finish(),
            Self::Consumed => f.write_str("Consumed"),
            Self::TokenizerExhausted => f.write_str("TokenizerExhausted"),
        }
    }
}
impl From<ParseStop> for StandaloneError {
    fn from(error: ParseStop) -> Self {
        Self::Stop(error)
    }
}
impl From<prologue::PrologueError> for StandaloneError {
    fn from(error: prologue::PrologueError) -> Self {
        Self::Prologue(error)
    }
}
impl From<head::HeadError> for StandaloneError {
    fn from(error: head::HeadError) -> Self {
        Self::Head(error)
    }
}

/// Fresh ordinary request admission owns the sole controller and actual lexer /
/// prologue/head state. Captured/hosted initialization cannot enter this path.
/// This bounded driver retains unsupported row continuations and qualified
/// completion; full public parse/report publication is unfinished.
pub(crate) struct StandaloneDocumentDriver<'source> {
    source: &'source crate::HtmlSource,
    control: ParseControl,
    prologue: Option<prologue::DocumentPrologue<'source>>,
    tokenizer: Option<super::tokenizer::Tokenizer<'source>>,
    event: Option<super::tokenizer::token::Event>,
    token: Option<super::tokenizer::token::Token>,
    before: Option<prologue::BeforeHead>,
    head: Option<head::HeadStage<'source>>,
    lexical: Option<OwnedVec<super::tokenizer::token::Event>>,
    failed_evidence: Option<crate::construction::evidence::FailureDiagnosticEvidence>,
    failure_usage: Option<super::control::ParseUsage>,
}
impl<'source> StandaloneDocumentDriver<'source> {
    pub(crate) fn step(&mut self) -> Result<head::HeadStep, StandaloneError> {
        let result = self.step_inner();
        self.retain_terminal_failure(&result);
        result
    }
    fn retain_terminal_failure<T>(&mut self, result: &Result<T, StandaloneError>) {
        if let Err(error) = &result {
            // One controller owns terminality. Retained owners support cleanup,
            // never resumption or successful partial continuation publication.
            let reason = match error {
                StandaloneError::Head(head::HeadError::UnsupportedScriptPause) => {
                    StandaloneUnsupported::ScriptPause
                }
                StandaloneError::Head(head::HeadError::UnsupportedPendingScript) => {
                    StandaloneUnsupported::PendingScript
                }
                StandaloneError::Head(head::HeadError::ConstructionCapabilityUnavailable) => {
                    StandaloneUnsupported::Provenance
                }
                StandaloneError::Head(head::HeadError::Kernel(KernelError::Session(
                    SessionError::ScriptMetadataUnavailable
                    | SessionError::SvgScriptMetadataUnavailable,
                ))) => StandaloneUnsupported::ScriptDocument,
                StandaloneError::Head(head::HeadError::IncompleteCompletion)
                | StandaloneError::Head(head::HeadError::Kernel(KernelError::Session(
                    SessionError::CompletionMetadataUnavailable
                    | SessionError::CompletionIncomplete,
                ))) => StandaloneUnsupported::Completion,
                _ => StandaloneUnsupported::InvalidNesting,
            };
            self.control.stop_unsupported_standalone(reason);
        }
    }
    #[cfg(test)]
    pub(crate) fn make_available(&mut self, through: usize) -> Result<(), StandaloneError> {
        let result = self.make_available_inner(through);
        self.retain_terminal_failure(&result);
        result
    }
    #[cfg(test)]
    fn make_available_inner(&mut self, through: usize) -> Result<(), StandaloneError> {
        self.control.require_source(self.source)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(head) = &mut self.head {
            head.require(&mut self.control)?;
            self.control.charge(WorkCategory::RecordVisit, 2)?;
            head.state
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .tokenizer
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .make_available(through, &mut self.control)?;
        } else {
            self.control.charge(WorkCategory::RecordVisit, 1)?;
            self.tokenizer
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .make_available(through, &mut self.control)?;
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn seal_input(&mut self) -> Result<(), StandaloneError> {
        let result = self.seal_input_inner();
        self.retain_terminal_failure(&result);
        result
    }
    #[cfg(test)]
    fn seal_input_inner(&mut self) -> Result<(), StandaloneError> {
        self.control.require_source(self.source)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(head) = &mut self.head {
            head.require(&mut self.control)?;
            self.control.charge(WorkCategory::RecordVisit, 2)?;
            head.state
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .tokenizer
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .seal_input(&mut self.control)?;
        } else {
            self.control.charge(WorkCategory::RecordVisit, 1)?;
            self.tokenizer
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .seal_input(&mut self.control)?;
        }
        Ok(())
    }
    fn step_inner(&mut self) -> Result<head::HeadStep, StandaloneError> {
        use super::tokenizer::token::Event;
        self.control.require_source(self.source)?;
        self.control.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(head) = &mut self.head {
            let step = head.step(&mut self.control)?;
            if step == head::HeadStep::Boundary(head::HeadBoundary::AfterHead) {
                let state = head.state.as_mut().ok_or(StandaloneError::Consumed)?;
                // Consume the actual phase handoff in the sole fresh driver.
                // No new tree, tokenizer, controller or eligibility is minted.
                self.control.charge(WorkCategory::RecordVisit, 1)?;
                write(
                    size_of::<Option<head::HeadBoundary>>(),
                    1,
                    &mut self.control,
                )?;
                state.boundary = None;
                return Ok(head::HeadStep::Consumed);
            }
            return Ok(step);
        }
        let c = &mut self.control;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.before.is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self.token.is_none() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                write(size_of::<Option<Event>>(), 1, c)?;
                let evidence = self
                    .prologue
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .kernel
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .session
                    .evidence(c)
                    .map_err(|e| StandaloneError::Head(e.into()))?;
                let pulled = self
                    .tokenizer
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .next(c, false, evidence)?;
                match pulled {
                    super::tokenizer::TokenizerPull::Event(event) => self.event = Some(event),
                    super::tokenizer::TokenizerPull::NeedInput => {
                        return Ok(head::HeadStep::NeedInput);
                    }
                    super::tokenizer::TokenizerPull::Exhausted => {
                        return Err(StandaloneError::TokenizerExhausted);
                    }
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let event = self
                    .event
                    .as_ref()
                    .ok_or(StandaloneError::TokenizerExhausted)?;
                if let Event::Diagnostic(id) = event {
                    self.prologue
                        .as_mut()
                        .ok_or(StandaloneError::Consumed)?
                        .kernel
                        .as_mut()
                        .ok_or(StandaloneError::Consumed)?
                        .session
                        .evidence(c)
                        .map_err(|e| StandaloneError::Head(e.into()))?
                        .get(c, id)?;
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                if event.token().is_some() {
                    write(
                        size_of::<Option<Event>>()
                            + size_of::<Option<super::tokenizer::token::Token>>(),
                        2,
                        c,
                    )?;
                    let Event::Token(event) =
                        self.event.take().expect("retained actual token event")
                    else {
                        unreachable!("variant inspected")
                    };
                    self.token = Some(event.into_token(c)?);
                } else {
                    self.lexical
                        .as_mut()
                        .ok_or(StandaloneError::Consumed)?
                        .reserve_geometric(c, 1)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    write(size_of::<Option<Event>>(), 1, c)?;
                    let event = self.event.take().expect("retained lexical event");
                    if let Err((stop, event)) = self
                        .lexical
                        .as_mut()
                        .ok_or(StandaloneError::Consumed)?
                        .try_push_recover(c, event)
                    {
                        self.event = Some(event);
                        return Err(stop.into());
                    }
                    return Ok(head::HeadStep::Consumed);
                }
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            if self
                .prologue
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .process(&mut self.token, c)?
                == prologue::PrologueStep::Consumed
            {
                return Ok(head::HeadStep::Consumed);
            }
            c.charge(WorkCategory::RecordVisit, 1)?;
            write(
                size_of::<Option<prologue::BeforeHead>>()
                    + size_of::<Option<prologue::DocumentPrologue>>(),
                2,
                c,
            )?;
            self.before = Some(
                self.prologue
                    .as_mut()
                    .expect("active prologue")
                    .take_before_head(c)?,
            );
            self.prologue = None;
        }
        write(size_of::<Option<head::HeadStage>>(), 1, c)?;
        self.head = Some(head::HeadStage::new(
            self.source,
            &mut self.before,
            &mut self.tokenizer,
            c,
        )?);
        c.charge(WorkCategory::RecordVisit, 2)?;
        let head = self.head.as_mut().expect("actual fresh head transfer");
        write(size_of::<ConstructionCapability>(), 5, c)?;
        c.charge(WorkCategory::ProviderCall, 1)?;
        let document = head
            .state
            .as_ref()
            .expect("actual fresh head")
            .before
            .kernel
            .session
            .root(c)
            .map_err(|error| StandaloneError::Head(error.into()))?;
        let capability = ConstructionCapability {
            source: self.source,
            owner: c.identity(),
            document,
            profile: ExecutionProfile::Ordinary,
            images: crate::HtmlImageSupport::Disabled,
        };
        head.install_standalone(capability, c)?;
        // Swap retained real lexical evidence into the same head owner; leave
        // its original empty vector in this driver for release, without cloning.
        c.charge(WorkCategory::RecordVisit, 2)?;
        write(size_of::<OwnedVec<Event>>() * 2, 2, c)?;
        std::mem::swap(
            &mut head.state.as_mut().expect("active head").lexical,
            self.lexical.as_mut().expect("retained lexical evidence"),
        );
        Ok(head::HeadStep::Consumed)
    }
    #[cfg(test)]
    pub(crate) fn standalone_parts(
        &mut self,
    ) -> Option<(
        &mut ParseControl,
        &mut ParserArenaSession,
        &ConstructionCapability<'source>,
    )> {
        let (session, capability) = self.head.as_mut()?.standalone_parts()?;
        Some((&mut self.control, session, capability))
    }
    #[cfg(test)]
    pub(crate) fn usage(&self) -> &super::control::ParseUsage {
        self.control.usage()
    }
    /// The only fixture issuer moves the real admitted owners after a genuine
    /// fresh prologue/head transition. Production exclusive ownership is intact.
    #[cfg(test)]
    pub(crate) fn take_standalone_fixture(
        input: &mut Option<Self>,
    ) -> Result<
        (
            ParseControl,
            ParserArenaSession,
            ConstructionCapability<'source>,
        ),
        StandaloneError,
    > {
        let driver = input.as_mut().ok_or(StandaloneError::Consumed)?;
        let c = &mut driver.control;
        c.require_source(driver.source)?;
        driver
            .head
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .prepare_standalone_fixture(c)?;
        dispose_token(&mut driver.token, c)?;
        write(size_of::<Option<super::tokenizer::token::Event>>(), 1, c)?;
        if let Some(event) = driver.event.take() {
            event.release(c)?;
        }
        if let Some(lexical) = &mut driver.lexical {
            while let Some(event) = lexical.try_pop(c)? {
                event.release(c)?;
            }
        }
        write(
            size_of::<Option<OwnedVec<super::tokenizer::token::Event>>>(),
            1,
            c,
        )?;
        if let Some(lexical) = driver.lexical.take() {
            lexical.release(c)?;
        }
        write(
            size_of::<Self>()
                + size_of::<Option<Self>>()
                + size_of::<ParseControl>()
                + size_of::<ParserArenaSession>()
                + size_of::<ConstructionCapability>(),
            5,
            c,
        )?;
        let driver = input.take().expect("validated owning real driver");
        let (session, capability) = driver
            .head
            .expect("prepared real head")
            .into_standalone_fixture();
        Ok((driver.control, session, capability))
    }
    #[cfg(test)]
    pub(crate) fn release(&mut self) -> Result<(), StandaloneError> {
        self.reclaim_parse_owners()?;
        if let Some(evidence) = self.failed_evidence.take() {
            evidence.release(&mut self.control)?;
        }
        Ok(())
    }
    fn reclaim_parse_owners(&mut self) -> Result<(), StandaloneError> {
        self.reclaim_owners(true)
    }
    fn reclaim_frozen_execution(&mut self) -> Result<(), StandaloneError> {
        self.reclaim_owners(false)
    }
    fn reclaim_owners(&mut self, active_session: bool) -> Result<(), StandaloneError> {
        let c = &mut self.control;
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(head) = &mut self.head {
            if c.stop().is_none() {
                write(size_of::<Option<head::HeadStage>>(), 1, c)?;
            }
            if active_session {
                head.release(c)?;
            } else {
                head.release_frozen_execution(c)?;
            }
            self.head = None;
        }
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(before) = &mut self.before {
            dispose_token(&mut before.input, c)?;
            loop {
                let text = if c.stop().is_some() {
                    before.whitespace.cleanup_pop(c)?
                } else {
                    before.whitespace.try_pop(c)?
                };
                let Some(text) = text else { break };
                text.release(c)?;
            }
            before
                .kernel
                .release(c)
                .map_err(|error| StandaloneError::Head(error.into()))?;
            if c.stop().is_none() {
                write(size_of::<Option<prologue::BeforeHead>>(), 1, c)?;
            }
            let before = self.before.take().expect("retained BeforeHead");
            before.whitespace.release(c)?;
        }
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(prologue) = &mut self.prologue {
            if c.stop().is_none() {
                write(size_of::<Option<prologue::DocumentPrologue>>(), 1, c)?;
            }
            prologue.release(c)?;
            self.prologue = None;
        }
        dispose_token(&mut self.token, c)?;
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            write(
                size_of::<Option<super::tokenizer::token::Event>>()
                    + size_of::<Option<super::tokenizer::Tokenizer>>(),
                2,
                c,
            )?;
        }
        if let Some(event) = self.event.take() {
            event.release(c)?;
        }
        if let Some(tokenizer) = self.tokenizer.take() {
            tokenizer.release(c)?;
        }
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
        }
        if let Some(lexical) = &mut self.lexical {
            loop {
                let event = if c.stop().is_some() {
                    lexical.cleanup_pop(c)?
                } else {
                    lexical.try_pop(c)?
                };
                let Some(event) = event else { break };
                event.release(c)?;
            }
        }
        if c.stop().is_none() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            write(
                size_of::<Option<OwnedVec<super::tokenizer::token::Event>>>(),
                1,
                c,
            )?;
        }
        if let Some(lexical) = self.lexical.take() {
            lexical.release(c)?;
        }
        Ok(())
    }
}
fn dispose_token(
    input: &mut Option<super::tokenizer::token::Token>,
    c: &mut ParseControl,
) -> Result<(), StandaloneError> {
    if c.stop().is_none() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let slot_bytes = size_of::<Option<super::tokenizer::token::Token>>();
        if input.is_some() {
            write(slot_bytes + size_of::<prologue::PendingToken>(), 3, c)?;
        } else {
            write(slot_bytes, 1, c)?;
        }
    }
    if let Some(token) = input.take() {
        let mut pending = prologue::PendingToken::from_token(token);
        let result = pending.release(c);
        if result.is_err() {
            *input = pending.into_token();
        }
        result?;
    }
    Ok(())
}

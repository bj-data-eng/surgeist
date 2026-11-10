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
#[cfg(test)]
mod outcome_tests;
pub(crate) mod parse;
pub(crate) mod prologue;
#[cfg(test)]
mod provenance_scaling_tests;
pub(crate) mod report;
pub(crate) mod seal;
mod stack_class;
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
        ControlIdentity, HtmlParseInvariant, HtmlParseStop, HtmlParseUnsupportedState,
        HtmlParseWorkCategory, OwnedVec, ParseControl,
    },
    tokenizer::token::Text,
};
use crate::{
    HtmlParseResource,
    construction::{
        ElementInsertionIgnored, ParserArenaSession, ParserNode, SessionError, TextInsertion,
    },
};
use stack_class::{ElementClass, Listing};
use std::mem::size_of;
pub(crate) use tag_sets::Scope;

#[derive(Debug)]
pub(crate) enum KernelError {
    Stop(HtmlParseStop),
    Session(SessionError),
    EmptyStack,
    NotElement,
    FirstOpenElementNotHtml,
    DuplicateOpenElement,
    Consumed,
}
impl From<HtmlParseStop> for KernelError {
    fn from(value: HtmlParseStop) -> Self {
        Self::Stop(value)
    }
}
impl From<SessionError> for KernelError {
    fn from(value: SessionError) -> Self {
        Self::Session(value)
    }
}
impl KernelError {
    /// What ends a parse that this error ends. A stop is already latched. A
    /// session error is classified by its own class. The stack of open
    /// elements being empty, holding a node that is not an element, starting
    /// with a non-HTML element or holding an element twice are shapes input
    /// may reach, which the standalone environment does not continue from.
    pub(super) fn stop_cause(&self) -> StopCause {
        match self {
            Self::Stop(stop) => StopCause::Session(SessionError::Stop(*stop)),
            Self::Session(error) => StopCause::Session(*error),
            Self::EmptyStack
            | Self::NotElement
            | Self::FirstOpenElementNotHtml
            | Self::DuplicateOpenElement => {
                StopCause::Unsupported(HtmlParseUnsupportedState::InvalidNesting)
            }
            Self::Consumed => StopCause::LostOwner,
        }
    }
}

/// What ended an execution, as far as the first stop and the failure's reason
/// are concerned. Every error type of every driver and stage names its cause
/// once (`stop_cause`), exhaustively, and the stop ([`Self::latch`]) and the
/// reason (`parse::cause_reason`) are both derived from it, so they cannot
/// disagree and one cause is one stop whichever driver or stage raised it.
#[derive(Clone, Copy, Debug)]
pub(super) enum StopCause {
    /// An owner the driver's own sequencing keeps is gone: the first stop is
    /// the invariant, or stays the stop that was already first.
    LostOwner,
    /// A session error, which ends the parse as its class says
    /// (`SessionClass`).
    Session(SessionError),
    /// A state the closed standalone environment does not continue from.
    Unsupported(HtmlParseUnsupportedState),
}
impl StopCause {
    pub(super) fn latch(self, control: &mut ParseControl) {
        match self {
            Self::LostOwner => {
                control.fail_invariant(HtmlParseInvariant::TreeBuilderState);
            }
            Self::Session(error) => {
                crate::construction::session_class(&error).latch(control);
            }
            Self::Unsupported(state) => {
                control.stop_unsupported_standalone(state);
            }
        }
    }
    /// Whether the cause is a lost owner of the driver's own sequencing.
    pub(super) fn is_lost_owner(self) -> bool {
        matches!(self, Self::LostOwner)
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
    open: Option<OwnedVec<OpenElement>>,
    /// Exclusive upper bound on the arena position of every node that is or
    /// was on the stack of open elements. Arena positions are never reused, so
    /// a node at or beyond the bound cannot already be on the stack: pushing it
    /// needs no duplicate scan.
    stack_position_bound: usize,
    /// The stack indices of the HTML `template` elements on the stack of open
    /// elements, in ascending order, so the last of them is the last template.
    /// Every operation that adds, removes or replaces a stack entry keeps the
    /// list exact, and an entry moves only within the tail the operation has
    /// already walked, so neither the last template nor the one before it is
    /// ever searched for. It holds nothing, and costs nothing, without a template.
    templates: Option<OwnedVec<usize>>,
    /// As `templates`, for the HTML `p` elements on the stack.
    paragraphs: Option<OwnedVec<usize>>,
    /// As `templates`, for the other boundaries of the default scope: see
    /// [`Listing::ScopeBoundary`].
    scope_boundaries: Option<OwnedVec<usize>>,
    /// As `templates`, for the HTML `button` elements on the stack.
    buttons: Option<OwnedVec<usize>>,
    root_target: Option<ParserNode>,
    foster_parenting: bool,
}

/// One entry of the stack of open elements: the element and what it is to the
/// scope walks, decided when it was pushed.
struct OpenElement {
    node: ParserNode,
    class: ElementClass,
}

/// Binary search over the ascending `listed` for `index`: the position where it
/// is found, else where it would be inserted. Each probe calls `probe` before
/// reading its listing, so a refusal stops the search before that work. The
/// search stops at the first probe that finds `index`.
fn search_listing(
    listed: &[usize],
    index: usize,
    mut probe: impl FnMut() -> Result<(), HtmlParseStop>,
) -> Result<Result<usize, usize>, HtmlParseStop> {
    let (mut low, mut high) = (0_usize, listed.len());
    while low < high {
        probe()?;
        let middle = low.saturating_add(high.saturating_sub(low) / 2);
        let Some(entry) = listed.get(middle) else {
            break;
        };
        match entry.cmp(&index) {
            std::cmp::Ordering::Equal => return Ok(Ok(middle)),
            std::cmp::Ordering::Less => low = middle.saturating_add(1),
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    Ok(Err(low))
}

fn write(bytes: usize, writes: usize, control: &mut ParseControl) -> Result<(), HtmlParseStop> {
    control.admit_atomic_layout(bytes)?;
    control.charge(HtmlParseWorkCategory::CopyByte, bytes as u64)?;
    control.charge(HtmlParseWorkCategory::RecordWrite, writes as u64)
}

fn copy_node(node: &ParserNode, control: &mut ParseControl) -> Result<ParserNode, HtmlParseStop> {
    control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
    control.charge(
        HtmlParseWorkCategory::CopyByte,
        size_of::<ParserNode>() as u64,
    )?;
    Ok(*node)
}

impl LocationKernel {
    fn stack_index(
        &self,
        target: &ParserNode,
        c: &mut ParseControl,
    ) -> Result<Option<usize>, KernelError> {
        self.require(c)?;
        self.session.validate_node(c, target)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        let position = self.session.node_position(c, target)?;
        let removed = self.at(index, c)?;
        self.session
            .qualify_stack_removal(c, &removed, capability)?;
        let old_class = self.class_at(index)?;
        let class = self.classify(target, c)?;
        // Each listing the removed entry leaves is searched once, for where it
        // lies, before anything changes.
        let mut leaving = [None; Listing::ALL.len()];
        for (listing, at) in Listing::ALL.into_iter().zip(&mut leaving) {
            if old_class.listed(listing) && !class.listed(listing) {
                *at = Some(self.listed_position(listing, index, c)?);
            }
        }
        let node = copy_node(target, c)?;
        write(size_of::<OpenElement>(), 1, c)?;
        let slot = self
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .get_mut(index, c)?
            .ok_or(KernelError::EmptyStack)?;
        *slot = OpenElement { node, class };
        for (listing, at) in Listing::ALL.into_iter().zip(leaving) {
            if let Some(at) = at {
                self.unlist(listing, at, c)?;
            } else if class.listed(listing) && !old_class.listed(listing) {
                self.list_at(listing, index, c)?;
            }
        }
        self.stack_position_bound = self.stack_position_bound.max(position.saturating_add(1));
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if index > self.stack()?.len() {
            return Err(KernelError::EmptyStack);
        }
        self.push(target, c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        // The push left the target on top, so the stack is not empty.
        let class = self.class_at(
            self.stack()?
                .len()
                .checked_sub(1)
                .ok_or(KernelError::EmptyStack)?,
        )?;
        let top = self
            .stack()?
            .len()
            .checked_sub(1)
            .ok_or(KernelError::EmptyStack)?;
        for shift in (index..top).rev() {
            if !self
                .open
                .as_mut()
                .ok_or(KernelError::Consumed)?
                .swap_slots(shift, shift.saturating_add(1), c)?
            {
                return Err(KernelError::EmptyStack);
            }
        }
        // The target now sits at `index` and every entry from there up moved
        // one place toward the top. The push listed the target at the top.
        for listing in Listing::ALL {
            if class.listed(listing) {
                self.unlist_last(listing, c)?;
            }
        }
        self.listings_shift(index, true, c)?;
        for listing in Listing::ALL {
            if class.listed(listing) {
                self.list_at(listing, index, c)?;
            }
        }
        Ok(())
    }
    fn current(&self, c: &mut ParseControl) -> Result<ParserNode, KernelError> {
        self.require(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        if name == "p" && matches!(scope, Scope::Button) {
            return self.paragraph_in_button_scope(c);
        }
        self.require(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            let node = self.at(index, c)?;
            if self.named(&node, name, c)? {
                return Ok(true);
            }
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self.class_at(index)?.bounds(scope) {
                return Ok(false);
            }
        }
        Ok(false)
    }
    /// Whether an HTML `p` element is in button scope (§13.2.4.2): the nearest
    /// element from the top of the stack that is a `p` or a boundary of button
    /// scope is a `p`. No element is both, so the answer is whether the last
    /// listed `p` lies above the last listed boundary. This reads the end of
    /// each of the four ascending lists instead of walking the stack, and
    /// charges one record visit for each of them.
    fn paragraph_in_button_scope(&self, c: &mut ParseControl) -> Result<bool, KernelError> {
        self.require(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(paragraph) = self.last_listed(Listing::Paragraph) else {
            return Ok(false);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        let boundary = [
            Listing::Template,
            Listing::ScopeBoundary,
            Listing::ButtonBoundary,
        ]
        .into_iter()
        .filter_map(|listing| self.last_listed(listing))
        .max();
        Ok(boundary.is_none_or(|boundary| paragraph > boundary))
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let len = self.stack()?.len();
        // An empty stack holds no match, which is the same refusal as below.
        let top = len.checked_sub(1).ok_or(KernelError::EmptyStack)?;
        // A node is on the stack at most once, so the search order changes no
        // answer, and the nodes removed are the ones the parser has just opened.
        for index in (0..len).rev() {
            let node = self.at(index, c)?;
            if self.session.same_node(c, &node, target)? {
                self.session.qualify_stack_removal(c, &node, capability)?;
                // The removal moves every entry above `index` one place down.
                let class = self.class_at(index)?;
                for listing in Listing::ALL {
                    if class.listed(listing) {
                        let at = self.listed_position(listing, index, c)?;
                        self.unlist(listing, at, c)?;
                    }
                }
                self.listings_shift(index.saturating_add(1), false, c)?;
                for shift in index..top {
                    if !self
                        .open
                        .as_mut()
                        .ok_or(KernelError::Consumed)?
                        .swap_slots(shift, shift.saturating_add(1), c)?
                    {
                        return Err(KernelError::EmptyStack);
                    }
                }
                self.pop_with_lists(c, capability, false)?;
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let root_target = session
            .as_ref()
            .ok_or(KernelError::Consumed)?
            .fragment_root(control)?;
        let open = OwnedVec::new(HtmlParseResource::OpenElements)?;
        let templates = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let paragraphs = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let scope_boundaries = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let buttons = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        write(size_of::<Self>(), 8, control)?;
        write(size_of::<Option<ParserArenaSession>>(), 1, control)?;
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        // Every fallible step precedes the take, so a refusal leaves the
        // caller's session where it was.
        let Some(session) = session.take() else {
            return Err(KernelError::Consumed);
        };
        Ok(Self {
            owner: control.identity(),
            session,
            open: Some(open),
            stack_position_bound: 0,
            templates: Some(templates),
            paragraphs: Some(paragraphs),
            scope_boundaries: Some(scope_boundaries),
            buttons: Some(buttons),
            root_target,
            foster_parenting: false,
        })
    }

    fn require(&self, control: &mut ParseControl) -> Result<(), KernelError> {
        control.require_identity(&self.owner)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.open.as_ref().ok_or(KernelError::Consumed)?;
        Ok(())
    }

    fn stack(&self) -> Result<&OwnedVec<OpenElement>, KernelError> {
        self.open.as_ref().ok_or(KernelError::Consumed)
    }

    fn at(&self, index: usize, control: &mut ParseControl) -> Result<ParserNode, KernelError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let entry = self
            .stack()?
            .as_slice()
            .get(index)
            .ok_or(KernelError::EmptyStack)?;
        Ok(copy_node(&entry.node, control)?)
    }

    /// The class of the stack entry at `index`. The caller has already charged
    /// the visit that reached the entry.
    fn class_at(&self, index: usize) -> Result<ElementClass, KernelError> {
        self.stack()?
            .as_slice()
            .get(index)
            .map(|entry| entry.class)
            .ok_or(KernelError::EmptyStack)
    }

    /// Classifies an element by its name, which is where its name is compared.
    fn classify(
        &self,
        node: &ParserNode,
        control: &mut ParseControl,
    ) -> Result<ElementClass, KernelError> {
        let name = self
            .session
            .element_name(control, node)?
            .ok_or(KernelError::NotElement)?;
        Ok(ElementClass::classify(name, control)?)
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.stack()?.len() == 0 && !self.named(node, "html", control)? {
            return Err(KernelError::FirstOpenElementNotHtml);
        }
        let position = self.session.node_position(control, node)?;
        if position < self.stack_position_bound {
            for index in 0..self.stack()?.len() {
                control.charge(HtmlParseWorkCategory::StateStep, 1)?;
                let existing = self.at(index, control)?;
                if self.session.same_node(control, &existing, node)? {
                    return Err(KernelError::DuplicateOpenElement);
                }
            }
        }
        let index = self.stack()?.len();
        let class = self.classify(node, control)?;
        let copied = copy_node(node, control)?;
        self.open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_push_geometric(
                control,
                OpenElement {
                    node: copied,
                    class,
                },
            )?;
        for listing in Listing::ALL {
            if class.listed(listing) {
                self.list_mut(listing)?.try_push_geometric(control, index)?;
            }
        }
        self.stack_position_bound = self.stack_position_bound.max(position.saturating_add(1));
        self.session.parser_stack_added(control, node)?;
        Ok(())
    }

    /// Popping a stack slot does not detach or refund the retained arena node.
    pub(crate) fn pop(
        &mut self,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
    ) -> Result<Option<ParserNode>, KernelError> {
        self.pop_with_lists(control, capability, true)
    }
    /// Pops the top entry, unlisting it from each list its class names when `listed`; a caller
    /// that unlisted it already (and moved the entry to the top) passes `false`. The lists
    /// must agree with the class: an entry the class lists must be the last of that list when
    /// `listed` and absent from it otherwise, and an entry it does not list must be absent, or
    /// the stack's own state is inconsistent, which stops the parse as a defect.
    fn pop_with_lists(
        &mut self,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
        listed: bool,
    ) -> Result<Option<ParserNode>, KernelError> {
        self.require(control)?;
        if self.stack()?.len() != 0 {
            let current = self.current(control)?;
            self.session
                .qualify_stack_removal(control, &current, capability)?;
        }
        let top = self.stack()?.len().checked_sub(1);
        let holds_top =
            Listing::ALL.map(|listing| top.is_some() && top == self.last_listed(listing));
        let entry = self
            .open
            .as_mut()
            .ok_or(KernelError::Consumed)?
            .try_pop(control)?;
        if let Some(entry) = &entry {
            for (listing, holds) in Listing::ALL.into_iter().zip(holds_top) {
                let unlist = listed && entry.class.listed(listing);
                if holds != unlist {
                    return Err(control
                        .fail_invariant(HtmlParseInvariant::TreeBuilderState)
                        .into());
                }
                if unlist {
                    self.unlist_last(listing, control)?;
                }
            }
        }
        if let Some(entry) = &entry {
            self.session
                .parser_stack_removed(control, &entry.node, capability)?;
        }
        Ok(entry.map(|entry| entry.node))
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        for index in (0..self.stack()?.len()).rev() {
            control.charge(HtmlParseWorkCategory::StateStep, 1)?;
            let node = self.at(index, control)?;
            if self.session.same_node(control, &node, target)? {
                return Ok(true);
            }
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self.class_at(index)?.bounds(scope) {
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

    /// The topmost stack element as an explicit insertion parent. In the
    /// fragment case its children are the fragment root's, so the parent is
    /// redirected exactly as an adjusted location targeting it would be.
    fn first_as_parent(&self, control: &mut ParseControl) -> Result<ParserNode, KernelError> {
        self.require(control)?;
        let Some(root_target) = &self.root_target else {
            return self.at(0, control);
        };
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(copy_node(root_target, control)?)
    }

    /// Namespace-qualified membership, not ordinary tree ancestry or scope.
    fn contains_html_template(&self, control: &mut ParseControl) -> Result<bool, KernelError> {
        self.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        Ok(self.last_template_index().is_some())
    }
    /// The last template element in the stack of open elements, if any.
    fn last_template(&self, control: &mut ParseControl) -> Result<Option<ParserNode>, KernelError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        self.last_template_index()
            .map(|index| self.at(index, control))
            .transpose()
    }
    /// The ascending stack indices the stack lists under `listing`.
    fn list(&self, listing: Listing) -> Option<&OwnedVec<usize>> {
        match listing {
            Listing::Template => self.templates.as_ref(),
            Listing::Paragraph => self.paragraphs.as_ref(),
            Listing::ScopeBoundary => self.scope_boundaries.as_ref(),
            Listing::ButtonBoundary => self.buttons.as_ref(),
        }
    }
    fn list_mut(&mut self, listing: Listing) -> Result<&mut OwnedVec<usize>, KernelError> {
        match listing {
            Listing::Template => self.templates.as_mut(),
            Listing::Paragraph => self.paragraphs.as_mut(),
            Listing::ScopeBoundary => self.scope_boundaries.as_mut(),
            Listing::ButtonBoundary => self.buttons.as_mut(),
        }
        .ok_or(KernelError::Consumed)
    }
    /// The listing's storage, if the kernel still owns it, for fixtures that
    /// release it themselves.
    #[cfg(test)]
    fn list_mut_for_fixture(&mut self, listing: Listing) -> Option<&mut OwnedVec<usize>> {
        self.list_mut(listing).ok()
    }
    fn listed_indices(&self, listing: Listing) -> &[usize] {
        self.list(listing).map_or(&[], OwnedVec::as_slice)
    }
    #[cfg(test)]
    fn template_indices(&self) -> &[usize] {
        self.listed_indices(Listing::Template)
    }
    fn last_listed(&self, listing: Listing) -> Option<usize> {
        self.listed_indices(listing).last().copied()
    }
    fn last_template_index(&self) -> Option<usize> {
        self.last_listed(Listing::Template)
    }
    /// The position among the `listing` of the stack entry at `index`, which
    /// the entry's class says is listed. The search charges one state step for
    /// each probe it makes, before the probe reads its listing. State steps are
    /// what this kernel charges for each entry a loop over its own state
    /// examines (the duplicate scan, the scope walks); a probe examines one
    /// listing the same way. The probes are counted by the loop in
    /// `search_listing`, not derived from the length, so the charge is the work
    /// whatever the standard library's binary search would do. An entry the
    /// class lists but the list lacks is a defect of the stack's own state.
    fn listed_position(
        &self,
        listing: Listing,
        index: usize,
        c: &mut ParseControl,
    ) -> Result<usize, KernelError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match search_listing(self.listed_indices(listing), index, || {
            c.charge(HtmlParseWorkCategory::StateStep, 1)
        })? {
            Ok(at) => Ok(at),
            Err(_) => Err(c
                .fail_invariant(HtmlParseInvariant::TreeBuilderState)
                .into()),
        }
    }
    /// Drops the last listing: the stack entry it names left the stack.
    fn unlist_last(&mut self, listing: Listing, c: &mut ParseControl) -> Result<(), KernelError> {
        self.list_mut(listing)?.try_pop(c)?;
        Ok(())
    }
    /// Lists the stack entry at `index`, which is not yet listed, keeping the
    /// list ascending. Only the entries above `index` move.
    fn list_at(
        &mut self,
        listing: Listing,
        index: usize,
        c: &mut ParseControl,
    ) -> Result<(), KernelError> {
        let listed = self.list_mut(listing)?;
        listed.try_push_geometric(c, index)?;
        let mut at = listed.len().saturating_sub(1);
        while at > 0
            && listed
                .as_slice()
                .get(at.saturating_sub(1))
                .is_some_and(|below| *below > index)
        {
            if !listed.swap_slots(at.saturating_sub(1), at, c)? {
                return Err(KernelError::EmptyStack);
            }
            at = at.saturating_sub(1);
        }
        Ok(())
    }
    /// Unlists the listing at position `at`. Only the listings above it move.
    fn unlist(
        &mut self,
        listing: Listing,
        at: usize,
        c: &mut ParseControl,
    ) -> Result<(), KernelError> {
        let listed = self.list_mut(listing)?;
        if at >= listed.len() {
            return Err(KernelError::EmptyStack);
        }
        for shift in at..listed.len().saturating_sub(1) {
            if !listed.swap_slots(shift, shift.saturating_add(1), c)? {
                return Err(KernelError::EmptyStack);
            }
        }
        listed.try_pop(c)?;
        Ok(())
    }
    /// Renumbers the listed entries at or above stack index `from`, in every
    /// listing, which moved one place up or down together with their stack
    /// entries.
    fn listings_shift(
        &mut self,
        from: usize,
        up: bool,
        c: &mut ParseControl,
    ) -> Result<(), KernelError> {
        for listing in Listing::ALL {
            let listed = self.list_mut(listing)?;
            let mut at = listed.len();
            while at > 0
                && listed
                    .as_slice()
                    .get(at.saturating_sub(1))
                    .is_some_and(|index| *index >= from)
            {
                at = at.saturating_sub(1);
                let entry = listed.get_mut(at, c)?.ok_or(KernelError::EmptyStack)?;
                let moved = if up {
                    entry.checked_add(1)
                } else {
                    entry.checked_sub(1)
                };
                *entry =
                    moved.ok_or_else(|| c.fail_invariant(HtmlParseInvariant::TreeBuilderState))?;
            }
        }
        Ok(())
    }

    /// §13.2.6.1 appropriate place, including foster/template routing. Fragment
    /// root substitution belongs to the subsequent insertion operation.
    pub(crate) fn appropriate_place(
        &self,
        override_target: Option<&ParserNode>,
        control: &mut ParseControl,
    ) -> Result<AppropriatePlace, KernelError> {
        self.require(control)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            for index in (0..self.stack()?.len()).rev() {
                control.charge(HtmlParseWorkCategory::StateStep, 1)?;
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
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        self.insert_element_at_with_meta_csp(element, location, control, capability, &mut None)
    }
    fn insert_element_at_with_meta_csp(
        &mut self,
        element: &ParserNode,
        location: &InsertionLocation,
        control: &mut ParseControl,
        capability: Option<&ConstructionCapability<'_>>,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<ElementInsertionOutcome, KernelError> {
        let outcome =
            match self
                .session
                .element_insertion_ignored(control, element, &location.parent)?
            {
                Some(reason) => ElementInsertionOutcome::Ignored(reason),
                None => {
                    if let Some(capability) = capability {
                        self.session.insert_parser_with_meta_csp(
                            control,
                            &location.parent,
                            element,
                            location.before.as_ref(),
                            capability,
                            service,
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
        // Every owner is taken and released even when an earlier release fails;
        // the first failure is the one returned.
        let stack = match self.open.take() {
            Some(open) => open.release(control).map_err(KernelError::from),
            None => Err(KernelError::Consumed),
        };
        let mut listings = Ok(());
        for listing in [
            self.templates.take(),
            self.paragraphs.take(),
            self.scope_boundaries.take(),
            self.buttons.take(),
        ] {
            if let Some(listed) = listing
                && let Err(stop) = listed.release(control)
                && listings.is_ok()
            {
                listings = Err(stop);
            }
        }
        self.root_target = None;
        stack?;
        listings?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        c.require_source(self.source)?;
        if matches!(self.profile, ExecutionProfile::Ordinary) {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
                .stop_unsupported_standalone(HtmlParseUnsupportedState::Provenance)
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
        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
        session.perform_intrinsic_checkpoint(c, self)
    }
}

pub(crate) enum StandaloneError {
    UnsupportedDocumentInitialization,
    UnsupportedExecutionProfile,
    /// An input edit reached a bank execution also holds.
    InputBankRefused,
    Stop(HtmlParseStop),
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
            Self::InputBankRefused => f.write_str("InputBankRefused"),
            Self::Stop(error) => f.debug_tuple("Stop").field(error).finish(),
            Self::Prologue(error) => f.debug_tuple("Prologue").field(error).finish(),
            Self::Head(error) => f.debug_tuple("Head").field(error).finish(),
            Self::Consumed => f.write_str("Consumed"),
            Self::TokenizerExhausted => f.write_str("TokenizerExhausted"),
        }
    }
}
impl StandaloneError {
    /// An owner this driver's own sequencing keeps is gone while its controller
    /// is still held: the first stop becomes the invariant, or stays the stop
    /// that was already first.
    pub(super) fn latch_invariant(control: &mut ParseControl) -> Self {
        Self::Stop(control.fail_invariant(HtmlParseInvariant::TreeBuilderState))
    }
    /// The error names a state the driver's own sequencing cannot produce,
    /// rather than a document or a limit.
    pub(super) fn invariant_fault(&self) -> bool {
        self.stop_cause().is_lost_owner()
    }
}
impl StandaloneError {
    /// What ended the execution, for the first stop and the reason it reports.
    /// A document initialization or execution profile the standalone
    /// environment was not given proof of is the provenance boundary.
    pub(super) fn stop_cause(&self) -> StopCause {
        match self {
            Self::Stop(stop) => StopCause::Session(SessionError::Stop(*stop)),
            Self::Prologue(error) => error.stop_cause(),
            Self::Head(error) => error.stop_cause(),
            Self::UnsupportedDocumentInitialization | Self::UnsupportedExecutionProfile => {
                StopCause::Unsupported(HtmlParseUnsupportedState::Provenance)
            }
            Self::InputBankRefused | Self::Consumed | Self::TokenizerExhausted => {
                StopCause::LostOwner
            }
        }
    }
}
impl From<HtmlParseStop> for StandaloneError {
    fn from(error: HtmlParseStop) -> Self {
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
    /// Whether the failure evidence has been taken from the session. Set only
    /// after the take succeeded, so a refused take can be retried.
    evidence_taken: bool,
}
impl<'source> StandaloneDocumentDriver<'source> {
    pub(super) fn enable_meta_csp(&mut self, initial_base: bool) -> Result<(), StandaloneError> {
        let result = self.enable_meta_csp_inner(initial_base);
        if let Err(error) = &result
            && error.invariant_fault()
        {
            // Any other refusal keeps the stop its caller latches for it.
            StandaloneError::latch_invariant(&mut self.control);
        }
        result
    }
    fn enable_meta_csp_inner(&mut self, initial_base: bool) -> Result<(), StandaloneError> {
        let session = &mut self
            .prologue
            .as_mut()
            .ok_or(StandaloneError::Consumed)?
            .kernel()?
            .session;
        session
            .enable_meta_csp(&mut self.control, initial_base)
            .map_err(|e| StandaloneError::Head(e.into()))
    }
    #[cfg(test)]
    pub(crate) fn step(&mut self) -> Result<head::HeadStep, StandaloneError> {
        self.step_with_meta_csp(&mut None)
    }
    pub(crate) fn step_with_meta_csp(
        &mut self,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<head::HeadStep, StandaloneError> {
        let result = self.step_inner(service);
        self.retain_terminal_failure(&result);
        result
    }
    fn retain_terminal_failure<T>(&mut self, result: &Result<T, StandaloneError>) {
        if let Err(error) = &result {
            // One controller owns terminality. Retained owners support cleanup,
            // never resumption or successful partial continuation publication.
            // A stop is already the controller's first, so naming it changes
            // nothing; every other error ends the parse as its cause says, in
            // whichever stage it was met.
            error.stop_cause().latch(&mut self.control);
        }
    }
    fn step_inner(
        &mut self,
        service: &mut Option<&mut dyn crate::meta_csp::Service>,
    ) -> Result<head::HeadStep, StandaloneError> {
        use super::tokenizer::token::Event;
        self.control.require_source(self.source)?;
        self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if let Some(head) = &mut self.head {
            let step = head.step_with_meta_csp(&mut self.control, service)?;
            if step == head::HeadStep::Boundary(head::HtmlParseConstructionBoundary::AfterHead) {
                let state = head.state.as_mut().ok_or(StandaloneError::Consumed)?;
                // Consume the actual phase handoff in the sole fresh driver.
                // No new tree, tokenizer, controller or eligibility is minted.
                self.control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                write(
                    size_of::<Option<head::HtmlParseConstructionBoundary>>(),
                    1,
                    &mut self.control,
                )?;
                state.boundary = None;
                return Ok(head::HeadStep::Consumed);
            }
            return Ok(step);
        }
        let c = &mut self.control;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if self.before.is_none() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self.token.is_none() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
                    super::tokenizer::TokenizerPull::Exhausted => {
                        return Err(StandaloneError::TokenizerExhausted);
                    }
                }
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if matches!(event, Event::Token(_)) {
                    write(
                        const {
                            size_of::<Option<Event>>()
                                + size_of::<Option<super::tokenizer::token::Token>>()
                        },
                        2,
                        c,
                    )?;
                    let event = take_token_event(&mut self.event, c)?;
                    self.token = Some(event.into_token(c)?);
                } else {
                    self.lexical
                        .as_mut()
                        .ok_or(StandaloneError::Consumed)?
                        .reserve_geometric(c, 1)?;
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    write(size_of::<Option<Event>>(), 1, c)?;
                    let event = self
                        .event
                        .take()
                        .ok_or(StandaloneError::TokenizerExhausted)?;
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if self
                .prologue
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .process(&mut self.token, c)?
                == prologue::PrologueStep::Consumed
            {
                return Ok(head::HeadStep::Consumed);
            }
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            write(
                const {
                    size_of::<Option<prologue::BeforeHead>>()
                        + size_of::<Option<prologue::DocumentPrologue>>()
                },
                2,
                c,
            )?;
            self.before = Some(
                self.prologue
                    .as_mut()
                    .ok_or(StandaloneError::Consumed)?
                    .take_before_head(c)?,
            );
            self.prologue = None;
        }
        write(size_of::<Option<head::HeadStage>>(), 1, c)?;
        let head = self.head.insert(head::HeadStage::new(
            self.source,
            &mut self.before,
            &mut self.tokenizer,
            c,
        )?);
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        write(size_of::<ConstructionCapability>(), 5, c)?;
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let document = head
            .state
            .as_ref()
            .ok_or(StandaloneError::Consumed)?
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
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        write(const { size_of::<OwnedVec<Event>>() * 2 }, 2, c)?;
        std::mem::swap(
            &mut head
                .state
                .as_mut()
                .ok_or(StandaloneError::Consumed)?
                .lexical,
            self.lexical.as_mut().ok_or(StandaloneError::Consumed)?,
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
    pub(crate) fn usage(&self) -> super::control::HtmlParseUsage {
        self.control.usage()
    }
    /// The only fixture issuer moves the real admitted owners after a
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
        }
        if let Some(before) = self.before.take() {
            before.whitespace.release(c)?;
        }
        if c.stop().is_none() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            write(
                const {
                    size_of::<Option<super::tokenizer::token::Event>>()
                        + size_of::<Option<super::tokenizer::Tokenizer>>()
                },
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
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
/// Takes the token event out of `slot`. Any other content stays in the slot
/// for the usual release, and the refusal is an invariant stop.
fn take_token_event(
    slot: &mut Option<super::tokenizer::token::Event>,
    c: &mut ParseControl,
) -> Result<super::tokenizer::token::TokenEvent, HtmlParseStop> {
    match slot.take() {
        Some(super::tokenizer::token::Event::Token(event)) => Ok(event),
        other => {
            *slot = other;
            Err(c.fail_invariant(super::control::HtmlParseInvariant::TreeBuilderState))
        }
    }
}
fn dispose_token(
    input: &mut Option<super::tokenizer::token::Token>,
    c: &mut ParseControl,
) -> Result<(), StandaloneError> {
    if c.stop().is_none() {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        if input.is_some() {
            write(
                const {
                    size_of::<Option<super::tokenizer::token::Token>>()
                        + size_of::<prologue::PendingToken>()
                },
                3,
                c,
            )?;
        } else {
            write(
                const { size_of::<Option<super::tokenizer::token::Token>>() },
                1,
                c,
            )?;
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

//! Native Select collection edits through the canonical tree Source.

use super::*;
use crate::internal::{
    names::in_html_namespace_with,
    select::{DomSelectOptionBefore, OptionsEdit},
};
use std::ops::ControlFlow;
use surgeist_html::vocabulary::KnownHtmlElement;

const LENGTH_GROWTH_LIMIT: usize = 100_000;

pub(in crate::internal::store) enum Frame {
    Fill {
        select: Key,
        fragment: Key,
        remaining: usize,
        donor: Option<Key>,
    },
    AfterFill {
        select: Key,
        fragment: Key,
        donor: Option<Key>,
    },
    Shrink {
        suffix: select::ScratchVec<Key>,
        next: usize,
    },
}

impl Frame {
    pub(in crate::internal::store) fn freeze(
        &mut self,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Self::Shrink { suffix, .. } = self {
            suffix.freeze(draft, cancel)?;
        }
        Ok(())
    }

    pub(in crate::internal::store) fn checkpoint(
        &self,
        admission: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        Ok(match self {
            Self::Fill {
                select,
                fragment,
                remaining,
                donor,
            } => Self::Fill {
                select: *select,
                fragment: *fragment,
                remaining: *remaining,
                donor: *donor,
            },
            Self::AfterFill {
                select,
                fragment,
                donor,
            } => Self::AfterFill {
                select: *select,
                fragment: *fragment,
                donor: *donor,
            },
            Self::Shrink { suffix, next } => Self::Shrink {
                suffix: suffix.checkpoint(admission, cancel)?,
                next: *next,
            },
        })
    }
}

impl Draft {
    pub(super) fn edit_select_options(
        &mut self,
        select: Key,
        edit: OptionsEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.element(select)?.select_state().is_none() {
            return Err(DomError::WrongNodeKind);
        }
        match edit {
            OptionsEdit::Length(length) => {
                let length =
                    usize::try_from(length).map_err(|_| DomError::RepresentationExhausted)?;
                self.resize_select_options(select, length, limits, cancel)
            }
            OptionsEdit::Set { index, value } => {
                let donor = value.map(|node| self.resolve(node.into())).transpose()?;
                if let Some(donor) = donor {
                    self.source_step(cancel)?;
                    if self.element(donor)?.option_state().is_none() {
                        return Err(DomError::WrongNodeKind);
                    }
                }
                let index =
                    usize::try_from(index).map_err(|_| DomError::RepresentationExhausted)?;
                let (member, observed) = self.select_option_at(select, index, cancel)?;
                match (member, donor) {
                    (Some(member), Some(donor)) => {
                        self.source_step(cancel)?;
                        let parent = self
                            .node(member)?
                            .parent()
                            .ok_or(DomError::SelectProtocol)?;
                        self.replace_child(donor, parent, member, limits, cancel)
                    }
                    (Some(member), None) => self.remove(member, limits, cancel),
                    (None, Some(donor)) if index > observed => self.fill_select_options(
                        select,
                        index - observed,
                        Some(donor),
                        limits,
                        cancel,
                    ),
                    (None, Some(donor)) => self.pre_insert(donor, select, None, limits, cancel),
                    (None, None) => Ok(()),
                }
            }
            OptionsEdit::Add { element, before } => {
                self.add_select_member(select, element, before, limits, cancel)
            }
            OptionsEdit::Remove(index) => {
                let Ok(index) = usize::try_from(index) else {
                    return Ok(());
                };
                if let Some(member) = self.select_option_at(select, index, cancel)?.0 {
                    self.remove(member, limits, cancel)?;
                }
                Ok(())
            }
        }
    }

    fn visit_draft_options<T>(
        &self,
        select: Key,
        cancel: &AtomicBool,
        mut visit: impl FnMut(usize, Key) -> Result<ControlFlow<T>, DomError>,
    ) -> Result<Option<T>, DomError> {
        self.source_step(cancel)?;
        let group = self.families.options(self.live_key(select));
        let mut step = || self.source_step(cancel);
        let mut next = self.families.first(group, &mut step)?;
        let mut index = 0;
        while let Some(member) = next {
            step()?;
            if let ControlFlow::Break(result) = visit(index, member)? {
                return Ok(Some(result));
            }
            index = index
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            next = self
                .families
                .after(group, member, self.order_view(), &mut step)?;
        }
        Ok(None)
    }

    fn select_option_at(
        &self,
        select: Key,
        index: usize,
        cancel: &AtomicBool,
    ) -> Result<(Option<Key>, usize), DomError> {
        let mut observed = 0;
        let found = self.visit_draft_options(select, cancel, |position, node| {
            observed = position + 1;
            Ok(if position == index {
                ControlFlow::Break(node)
            } else {
                ControlFlow::Continue(())
            })
        })?;
        Ok((found, observed))
    }

    fn resize_select_options(
        &mut self,
        select: Key,
        length: usize,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut suffix = self.effect_vec();
        let mut current = 0;
        self.visit_draft_options::<()>(select, cancel, |index, member| {
            current = index + 1;
            if index >= length {
                self.select_reserve(&mut suffix, 1, cancel)?;
                suffix.push(member);
            }
            Ok(ControlFlow::Continue(()))
        })?;
        if !suffix.is_empty() {
            self.push_effect(
                select::Effect::SelectOptions(Frame::Shrink { suffix, next: 0 }),
                self.effect_scoped,
                cancel,
            )?;
            return self.run_effects(limits, cancel);
        }
        if length > current && length <= LENGTH_GROWTH_LIMIT {
            self.fill_select_options(select, length - current, None, limits, cancel)?;
        }
        Ok(())
    }

    fn add_select_member(
        &mut self,
        select: Key,
        element: NodeId,
        before: DomSelectOptionBefore,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let donor = self.resolve(element.into())?;
        if !matches!(
            KnownHtmlElement::from_name(self.element(donor)?.name()),
            Some(KnownHtmlElement::Option | KnownHtmlElement::Optgroup)
        ) {
            return Err(DomError::WrongNodeKind);
        }

        if self.ordinary_option_ancestor(donor, select, cancel)? {
            return Err(DomError::HierarchyRequest);
        }
        let reference = match before {
            DomSelectOptionBefore::Append => None,
            DomSelectOptionBefore::Index(index) => match usize::try_from(index) {
                Ok(index) => self.select_option_at(select, index, cancel)?.0,
                Err(_) => None,
            },
            DomSelectOptionBefore::Element(reference) => {
                self.source_step(cancel)?;
                let reference = self.resolve(reference.into())?;
                if !in_html_namespace_with(self.element(reference)?.name(), || {
                    self.source_step(cancel)
                })? {
                    return Err(DomError::WrongNodeKind);
                }
                if !self.ordinary_option_ancestor(select, reference, cancel)? {
                    return Err(DomError::NotFound);
                }
                if reference == donor {
                    return Ok(());
                }
                Some(reference)
            }
        };
        let parent = if let Some(reference) = reference {
            self.source_step(cancel)?;
            self.node(reference)?
                .parent()
                .ok_or(DomError::SelectProtocol)?
        } else {
            select
        };
        self.pre_insert(donor, parent, reference, limits, cancel)
    }

    fn ordinary_option_ancestor(
        &self,
        ancestor: Key,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let mut parent = self.node(node)?.parent();
        while let Some(key) = parent {
            self.source_step(cancel)?;
            if self.live_key(key) == self.live_key(ancestor) {
                return Ok(true);
            }
            parent = self.node(key)?.parent();
        }
        Ok(false)
    }

    fn fill_select_options(
        &mut self,
        select: Key,
        remaining: usize,
        donor: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let document = self.node(select)?.document();
        let fragment = self.allocate(NewNode::DocumentFragment, document, limits, cancel)?;
        self.mark_source_private(fragment, cancel)?;
        self.push_effect(
            select::Effect::SelectOptions(Frame::Fill {
                select,
                fragment,
                remaining,
                donor,
            }),
            self.effect_scoped,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }

    pub(in crate::internal::store) fn select_options_effect(
        &mut self,
        frame: Frame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        match frame {
            Frame::Fill {
                select,
                fragment,
                remaining: 0,
                donor,
            } => {
                self.push_effect(
                    select::Effect::SelectOptions(Frame::AfterFill {
                        select,
                        fragment,
                        donor,
                    }),
                    self.effect_scoped,
                    cancel,
                )?;
                self.pre_insert(fragment, select, None, limits, cancel)
            }
            Frame::Fill {
                select,
                fragment,
                remaining,
                donor,
            } => {
                let document = self.node(fragment)?.document();
                let option = self.create_blank_html_option(document, limits, cancel)?;
                self.mark_source_private(option, cancel)?;
                self.push_effect(
                    select::Effect::SelectOptions(Frame::Fill {
                        select,
                        fragment,
                        remaining: remaining - 1,
                        donor,
                    }),
                    self.effect_scoped,
                    cancel,
                )?;
                self.pre_insert(option, fragment, None, limits, cancel)
            }
            Frame::AfterFill {
                select,
                fragment,
                donor,
            } => {
                self.compound_discard_unpublished(fragment, cancel)?;
                if let Some(donor) = donor {
                    self.prepare_compound_failure_slot(cancel)?;
                    if let Err(error) =
                        self.ensure_pre_insert(donor, select, None, None, false, cancel)
                    {
                        return self.compound_error(error);
                    }
                    self.insert_validated(donor, select, None, limits, cancel)?;
                }
                Ok(())
            }
            Frame::Shrink { suffix, next } => {
                let member = suffix.get(next).copied().ok_or(DomError::SelectProtocol)?;
                if next + 1 < suffix.len() {
                    self.push_effect(
                        select::Effect::SelectOptions(Frame::Shrink {
                            suffix,
                            next: next + 1,
                        }),
                        self.effect_scoped,
                        cancel,
                    )?;
                }
                self.remove(member, limits, cancel)
            }
        }
    }
}

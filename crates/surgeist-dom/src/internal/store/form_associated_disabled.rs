//! Effective disabledness on canonical ordinary tree and attribute transitions.

use super::*;
use crate::internal::name::DomQualifiedName;
use std::convert::Infallible;
use surgeist_html::vocabulary::{KnownHtmlAttribute, KnownHtmlElement};
use surgeist_html_semantics::fieldset_disabledness::{
    HtmlFieldsetDisablednessReader, HtmlFieldsetParent, disabled_by_fieldset,
};

/// The first exempt branch before a structural change to a disabled Fieldset.
#[derive(Clone, Copy)]
pub(super) struct LegendFence {
    fieldset: Key,
    first: Option<Key>,
}

impl Draft {
    pub(super) fn refresh_form_associated_custom_disabled(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let Some(previous) = self.form_associated_custom_disabled(node)? else {
            return Ok(());
        };
        let disabled = if self
            .control_attribute(node, KnownHtmlAttribute::Disabled, cancel)?
            .is_some()
        {
            true
        } else {
            match disabled_by_fieldset(
                &mut Reader {
                    draft: self,
                    cancel,
                },
                node,
            )? {
                Ok(disabled) => disabled,
                Err(impossible) => match impossible {},
            }
        };
        if previous != disabled {
            self.set_form_associated_custom_disabled(node, disabled, cancel)?;
        }
        Ok(())
    }

    pub(super) fn form_associated_disabled_attribute_change(
        &mut self,
        node: Key,
        name: &DomQualifiedName,
        presence_changed: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !presence_changed
            || !self.has_form_associated_custom_controls()
            || name.namespace_uri().is_some()
            || !self.source_ascii(
                name.local_name(),
                KnownHtmlAttribute::Disabled.local_name(),
                cancel,
            )?
        {
            return Ok(());
        }
        let kind = Reader {
            draft: self,
            cancel,
        }
        .kind(node)?;
        if kind == Some(KnownHtmlElement::Fieldset) {
            self.refresh_form_associated_disabled_subtree(node, cancel)
        } else {
            self.refresh_form_associated_custom_disabled(node, cancel)
        }
    }

    pub(super) fn capture_form_associated_legend_fence(
        &self,
        child: Key,
        parent: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<LegendFence>, DomError> {
        if !self.has_form_associated_custom_controls() {
            return Ok(None);
        }
        let mut reader = Reader {
            draft: self,
            cancel,
        };
        if reader.html_legend(child)? && reader.disabled_fieldset(parent)? {
            return Ok(Some(LegendFence {
                fieldset: parent,
                first: reader.first_legend(parent)?,
            }));
        }
        Ok(None)
    }

    pub(super) fn refresh_form_associated_disabled_tree_change(
        &mut self,
        root: Key,
        fences: impl IntoIterator<Item = LegendFence>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.has_form_associated_custom_controls() {
            return Ok(());
        }
        self.refresh_form_associated_disabled_subtree(root, cancel)?;
        for fence in fences {
            let current = Reader {
                draft: self,
                cancel,
            }
            .first_legend(fence.fieldset)?;
            if current != fence.first {
                for branch in [fence.first, current].into_iter().flatten() {
                    if branch != root {
                        self.refresh_form_associated_disabled_subtree(branch, cancel)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn refresh_form_associated_disabled_subtree(
        &mut self,
        root: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut current = Some(root);
        while let Some(node) = current {
            self.refresh_form_associated_custom_disabled(node, cancel)?;
            current = crate::internal::slot::next(self, root, node, cancel)?;
        }
        Ok(())
    }
}

struct Reader<'a> {
    draft: &'a Draft,
    cancel: &'a AtomicBool,
}

impl Reader<'_> {
    fn kind(&self, node: Key) -> Result<Option<KnownHtmlElement>, DomError> {
        self.draft.source_step(self.cancel)?;
        Ok(match self.draft.node(node)?.kind() {
            transaction::KindRead::Element(element) => KnownHtmlElement::from_name(element.name()),
            _ => None,
        })
    }

    fn first_legend(&self, fieldset: Key) -> Result<Option<Key>, DomError> {
        self.draft.source_step(self.cancel)?;
        for child in self.draft.node(fieldset)?.children().iter() {
            if self.kind(child)? == Some(KnownHtmlElement::Legend) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }
}

impl HtmlFieldsetDisablednessReader for Reader<'_> {
    type Node = Key;
    type Missing = Infallible;
    type Error = DomError;

    fn ordinary_parent(
        &mut self,
        node: Key,
    ) -> Result<HtmlFieldsetParent<Key, Infallible>, DomError> {
        self.draft.source_step(self.cancel)?;
        Ok(self
            .draft
            .node(node)?
            .parent()
            .map_or(HtmlFieldsetParent::Root, HtmlFieldsetParent::Parent))
    }

    fn disabled_fieldset(&mut self, node: Key) -> Result<bool, DomError> {
        Ok(self.kind(node)? == Some(KnownHtmlElement::Fieldset)
            && self
                .draft
                .control_attribute(node, KnownHtmlAttribute::Disabled, self.cancel)?
                .is_some())
    }

    fn html_legend(&mut self, node: Key) -> Result<bool, DomError> {
        Ok(self.kind(node)? == Some(KnownHtmlElement::Legend))
    }

    fn first_legend_branch(
        &mut self,
        fieldset: Key,
        branch: Key,
    ) -> Result<Result<bool, Infallible>, DomError> {
        Ok(Ok(self.first_legend(fieldset)? == Some(branch)))
    }
}

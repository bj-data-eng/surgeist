// Copyright 2014-2017 The html5ever Project Developers. See the
// COPYRIGHT file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Adapted from tree_builder/rules.rs:257-307,1403-1462 at d7232d746d3112f08926d169591576bd12fe2fdf.
// HTML 877d6146487f syntax3984-4126/5500-5578 governs patching, PI and routing.

use super::{
    KernelError,
    head::{HeadBoundary, HeadCondition, HeadError, HeadState, HeadStep, InsertionMode},
    prologue::Payload,
    tag_sets, write,
};
use crate::{
    construction::CreationContext,
    parser::{
        control::{ParseControl, WorkCategory},
        tokenizer::token::TagKind,
    },
};
use std::mem::size_of;

impl HeadState<'_> {
    pub(super) fn template_start(&mut self, c: &mut ParseControl) -> Result<HeadStep, HeadError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self.script.is_none() {
            return self.stop_at(HeadBoundary::Template, c);
        }
        self.validate_standalone_capability(c)?;
        let old_template = self.before.kernel.last_template(c)?;
        self.push_marker(c)?;
        self.frameset(false, c)?;
        self.set_mode(InsertionMode::InTemplate, c)?;
        self.template_modes.try_push(c, InsertionMode::InTemplate)?;
        let place = self.before.kernel.appropriate_place(None, c)?;
        // Determine the token states without replacing them with final tree
        // spelling. A non-None shadow mode selects its row before for-patching.
        let (patch, shadow) = {
            let Payload::Tag(Some(tag)) = &self
                .pending
                .as_ref()
                .ok_or(HeadError::MissingInput)?
                .payload
            else {
                return Err(HeadError::MissingInput);
            };
            let mut patch = false;
            let mut shadow = false;
            for attr in tag.attributes.as_slice() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if tag_sets::ascii_eq(attr.name.value.text(), "for", c)? {
                    write(size_of::<bool>(), 1, c)?;
                    patch = true;
                }
                if tag_sets::ascii_eq(attr.name.value.text(), "shadowrootmode", c)? {
                    let value = attr.value.value.text();
                    write(size_of::<bool>(), 1, c)?;
                    shadow = super::foreign::insensitive(value, "open", c)?
                        || super::foreign::insensitive(value, "closed", c)?;
                }
            }
            (patch, shadow)
        };
        let Payload::Tag(input) = &mut self
            .pending
            .as_mut()
            .ok_or(HeadError::MissingInput)?
            .payload
        else {
            return Err(HeadError::MissingInput);
        };
        let node = self.before.kernel.session.allocate_html_tag_for_creation(
            c,
            input,
            &place.parent,
            self.current_form
                .as_ref()
                .and_then(super::head::FormPointer::owned),
            old_template
                .as_ref()
                .map_or(CreationContext::Ordinary, CreationContext::Template),
        )?;
        if shadow && self.before.kernel.session.allows_declarative_shadow(c)? {
            enum Host {
                Owned(super::ParserNode),
                Captured(crate::HtmlCaptureElementId),
            }
            // The adjusted current is captured only at the one real root
            // stack entry. The actual owned root is never used as a fake host.
            let host = if self.before.kernel.stack()?.len() == 1 {
                let context = self
                    .before
                    .kernel
                    .session
                    .fragment_context(c)?
                    .ok_or(crate::construction::SessionError::ContextUnavailable)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                write(size_of::<Host>(), 1, c)?;
                Host::Captured(context.context_element().clone())
            } else {
                let current = self.before.kernel.current(c)?;
                Host::Owned(current)
            };
            // Every lookup that can fail precedes the push, so a refusal never
            // leaves the node on the stack of open elements.
            let capability = &self
                .script
                .as_ref()
                .ok_or(HeadError::ConstructionCapabilityUnavailable)?
                .capability;
            let source = &self.pending.as_ref().ok_or(HeadError::MissingInput)?.source;
            self.before.kernel.push(&node, c)?;
            let host = match &host {
                Host::Owned(host) => {
                    crate::construction::context::DeclarativeShadowHost::Owned(host)
                }
                Host::Captured(host) => {
                    crate::construction::context::DeclarativeShadowHost::Captured(host)
                }
            };
            let attached = self
                .before
                .kernel
                .session
                .attach_declarative_shadow(c, &node, host, source, capability)?;
            if !attached {
                let location = self
                    .before
                    .kernel
                    .adjusted_location(Some(&place.parent), c)?;
                self.before
                    .kernel
                    .insert_element_at(&node, &location, c, Some(capability))?;
            }
            return self.finish(c);
        }
        if shadow || !patch {
            let location = self
                .before
                .kernel
                .adjusted_location(Some(&place.parent), c)?;
            self.before.kernel.insert_element_at(
                &node,
                &location,
                c,
                self.script.as_ref().map(|s| &s.capability),
            )?;
            self.before.kernel.push(&node, c)?;
            return self.finish(c);
        }
        let adjusted = self
            .before
            .kernel
            .adjusted_location(Some(&place.parent), c)?;
        let mut scope = super::copy_node(&adjusted.parent, c)?;
        if self.before.kernel.named(&scope, "template", c)? {
            scope = self
                .before
                .kernel
                .session
                .template_state(c, &scope)?
                .contents;
        } else if self.before.kernel.session.is_document_body(c, &scope)? {
            scope = self
                .before
                .kernel
                .session
                .parent(c, &scope)?
                .ok_or(HeadError::MissingInput)?;
        }
        let capability = &self
            .script
            .as_ref()
            .ok_or(HeadError::ConstructionCapabilityUnavailable)?
            .capability;
        self.before.kernel.push(&node, c)?;
        let success = self
            .before
            .kernel
            .session
            .prepare_template_patching(c, &scope, &node, capability)?;
        if !success {
            let location = self
                .before
                .kernel
                .adjusted_location(Some(&place.parent), c)?;
            self.before
                .kernel
                .pop(c, self.script.as_ref().map(|s| &s.capability))?
                .ok_or(KernelError::EmptyStack)?;
            self.before.kernel.insert_element_at(
                &node,
                &location,
                c,
                self.script.as_ref().map(|s| &s.capability),
            )?;
            self.before.kernel.push(&node, c)?;
        }
        self.finish(c)
    }
    pub(super) fn close_template(
        &mut self,
        c: &mut ParseControl,
        explicit: bool,
    ) -> Result<(), HeadError> {
        let template = self
            .before
            .kernel
            .last_template(c)?
            .ok_or(HeadError::MissingInput)?;
        if explicit {
            loop {
                let current = self.before.kernel.current(c)?;
                let mut implied = false;
                for name in [
                    "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp",
                    "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr",
                ] {
                    if self.before.kernel.named(&current, name, c)? {
                        write(size_of::<bool>(), 1, c)?;
                        implied = true;
                        break;
                    }
                }
                if !implied {
                    break;
                }
                self.before
                    .kernel
                    .pop(c, self.script.as_ref().map(|s| &s.capability))?
                    .ok_or(KernelError::EmptyStack)?;
            }
            let current = self.before.kernel.current(c)?;
            if !self.before.kernel.named(&current, "template", c)? {
                self.emit(
                    HeadCondition::UnexpectedOpenElement,
                    crate::construction::evidence::DiagnosticRule {
                        anchor: "parsing-main-inhead",
                        branch: "template-end/current-node-mismatch",
                    },
                    c,
                )?;
            }
            self.before.kernel.session.finish_template_patching(
                c,
                &template,
                self.script.as_ref().map(|s| &s.capability),
            )?;
        }
        loop {
            let node = self
                .before
                .kernel
                .pop(c, self.script.as_ref().map(|s| &s.capability))?
                .ok_or(KernelError::EmptyStack)?;
            if self.before.kernel.session.same_node(c, &node, &template)? {
                break;
            }
        }
        self.clear_marker(c)?;
        self.template_modes
            .try_pop(c)?
            .ok_or(HeadError::MissingInput)?;
        self.reset_table_mode(c)
    }
    pub(super) fn template_eof(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        if !self.before.kernel.contains_html_template(c)? {
            return Ok(Some(self.begin_end(c)?));
        }
        self.emit(
            HeadCondition::UnexpectedOpenElement,
            crate::construction::evidence::DiagnosticRule {
                anchor: "parsing-main-intemplate",
                branch: "template-eof/template-open",
            },
            c,
        )?;
        self.close_template(c, false)?;
        Ok(None)
    }
    pub(super) fn template_rules(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<HeadStep>, HeadError> {
        let payload = &self
            .pending
            .as_ref()
            .ok_or(HeadError::MissingInput)?
            .payload;
        match payload {
            Payload::Eof => return self.template_eof(c),
            Payload::Text(_) | Payload::Comment(_) | Payload::Pi(_) | Payload::Doctype(_) => {
                let source = self
                    .script
                    .as_ref()
                    .ok_or(HeadError::ConstructionCapabilityUnavailable)?
                    .capability
                    .source;
                return self.body_rules(InsertionMode::InBody, source, c);
            }
            _ => {}
        }
        let (kind, close) = self.kind(c)?.ok_or(HeadError::MissingInput)?;
        if kind == TagKind::End {
            if self.token_in(&["template"], c)?.is_some() {
                return Ok(Some(self.end_template(c)?));
            }
            self.emit(
                HeadCondition::UnexpectedEndTag,
                crate::construction::evidence::DiagnosticRule {
                    anchor: "parsing-main-intemplate",
                    branch: "template/disallowed-end-tag",
                },
                c,
            )?;
            return Ok(Some(self.discard(c)?));
        }
        if self
            .token_in(
                &[
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ],
                c,
            )?
            .is_some()
        {
            return Ok(Some(self.head_delegation(false, close, c)?));
        }
        let mode = if self
            .token_in(&["caption", "colgroup", "tbody", "tfoot", "thead"], c)?
            .is_some()
        {
            InsertionMode::InTable
        } else if self.token_in(&["col"], c)?.is_some() {
            InsertionMode::InColumnGroup
        } else if self.token_in(&["tr"], c)?.is_some() {
            InsertionMode::InTableBody
        } else if self.token_in(&["td", "th"], c)?.is_some() {
            InsertionMode::InRow
        } else {
            InsertionMode::InBody
        };
        c.charge(WorkCategory::RecordVisit, 1)?;
        let index = self
            .template_modes
            .len()
            .checked_sub(1)
            .ok_or(HeadError::MissingInput)?;
        write(size_of::<InsertionMode>(), 1, c)?;
        *self
            .template_modes
            .get_mut(index, c)?
            .ok_or(HeadError::MissingInput)? = mode;
        self.set_mode(mode, c)?;
        Ok(None)
    }
}

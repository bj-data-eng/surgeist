//! Intrinsic select metadata and select-specific construction handlers.
//! Governing HTML 877d6146487f: select/option/selectedcontent processing.

use super::effects::{EffectFrame, FragmentStage, IntrinsicTask, TargetStage, WorkListKind};
use super::*;
use crate::{HtmlOptionFlags, HtmlSelectFlags};
use surgeist_html::bounded::{
    HtmlSelectAction, HtmlSelectCursor, HtmlSelectFirstSelectedAction,
    HtmlSelectFirstSelectedCursor, HtmlSelectFirstSelection, HtmlSelectOperation, HtmlSelectReply,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectState {
    pub(crate) flags: HtmlSelectFlags,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OptionState {
    pub(crate) flags: HtmlOptionFlags,
    nearest_select: Option<usize>,
}
impl OptionState {
    /// Fixed canonical cache read; callers qualify the result-local reference
    /// before immutable publication. This does not recompute ancestry.
    pub(super) fn cached_nearest_select(&self) -> Option<usize> {
        self.nearest_select
    }
    pub(super) fn fresh() -> Self {
        Self {
            flags: HtmlOptionFlags::default(),
            nearest_select: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SelectedcontentState {
    pub(crate) disabled: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SelectMetadata {
    Select(SelectState),
    Option(OptionState),
    Selectedcontent(SelectedcontentState),
}
pub(super) fn initial_metadata(
    c: &mut ParseControl,
    tag: &Tag,
) -> Result<Option<SelectMetadata>, SessionError> {
    for (name, state) in [
        (
            "select",
            Some(SelectMetadata::Select(SelectState {
                flags: HtmlSelectFlags::default(),
            })),
        ),
        (
            "option",
            Some(SelectMetadata::Option(OptionState {
                flags: HtmlOptionFlags::default(),
                nearest_select: None,
            })),
        ),
        (
            "selectedcontent",
            Some(SelectMetadata::Selectedcontent(SelectedcontentState {
                disabled: false,
            })),
        ),
        ("optgroup", None),
    ] {
        if exact_ascii(c, Some(tag.name.value.text()), name)? {
            return Ok(state);
        }
    }
    Err(SessionError::WrongToken)
}
impl SessionStorage {
    pub(super) fn step_select_effect(
        &mut self,
        c: &mut ParseControl,
        frame: EffectFrame,
    ) -> Result<(), SessionError> {
        match frame {
            EffectFrame::UpdateSelect {
                select,
                list,
                cursor,
            } => {
                let Some(target) = self.work_list_node(c, list, cursor)? else {
                    self.release_work_list(c, list)?;
                    return self.write_current_effect(c, None);
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::UpdateSelect {
                        select,
                        list,
                        cursor: cursor + 1,
                    }),
                )?;
                let (_, SelectMetadata::Selectedcontent(state)) =
                    self.select_metadata(c, target)?
                else {
                    return Err(SessionError::SelectMetadataUnavailable);
                };
                if surgeist_html::bounded::selectedcontent_update(
                    false,
                    state.disabled,
                    HtmlSelectFirstSelection::Unavailable,
                ) == surgeist_html::bounded::HtmlSelectedcontentUpdate::Skip
                {
                    return Ok(());
                }
                let option = self.first_selected_option(c, select)?;
                self.start_effect(
                    c,
                    EffectFrame::UpdateTarget {
                        option,
                        target,
                        fragment: None,
                        stage: TargetStage::Prepare,
                    },
                )
            }
            EffectFrame::UpdateTarget {
                option,
                target,
                stage: TargetStage::Prepare,
                ..
            } => {
                if let Some(option) = option {
                    self.qualify_clone_subtree(c, option)?;
                }
                let document = option
                    .map(|option| self.node_document_ref(c, option))
                    .transpose()?;
                let fragment = if option.is_some() {
                    Some(
                        self.allocate_in_document_owned(
                            c,
                            HtmlNodeKind::Fragment,
                            false,
                            0,
                            0,
                            document,
                        )?
                        .index,
                    )
                } else {
                    None
                };
                self.write_current_effect(
                    c,
                    Some(EffectFrame::UpdateTarget {
                        option,
                        target,
                        fragment,
                        stage: TargetStage::Replace,
                    }),
                )?;
                if let (Some(option), Some(fragment)) = (option, fragment) {
                    self.start_effect(
                        c,
                        EffectFrame::CloneOptionChildren {
                            option,
                            fragment,
                            cursor: 0,
                        },
                    )?;
                }
                Ok(())
            }
            EffectFrame::UpdateTarget {
                option,
                target,
                fragment,
                stage: TargetStage::Replace,
            } => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if self
                    .construction
                    .node_at(target)
                    .ok_or(SessionError::ForeignNode)?
                    .child_count()
                    == 0
                {
                    if fragment.is_none() {
                        return self.write_current_effect(c, None);
                    }
                    return self.write_current_effect(
                        c,
                        Some(EffectFrame::UpdateTarget {
                            option,
                            target,
                            fragment,
                            stage: TargetStage::Insert,
                        }),
                    );
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self
                    .construction
                    .node_at(target)
                    .ok_or(SessionError::ForeignNode)?
                    .child_at(0)
                    .ok_or(SessionError::WrongSibling)?
                    .position();
                self.remove_index(c, child)
            }
            EffectFrame::UpdateTarget {
                target,
                fragment: Some(fragment),
                stage: TargetStage::Insert,
                ..
            } => {
                // DOM insert snapshots the fragment's top-level nodes, removes
                // all of them, then finishes all sibling insertion hooks before
                // one parent childrenChanged and static post-connection list.
                let list = self.retain_work_list(c, fragment, WorkListKind::FragmentChildren)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::FragmentInsertion {
                        parent: target,
                        list,
                        cursor: 0,
                        stage: FragmentStage::Remove,
                    }),
                )
            }
            EffectFrame::UpdateTarget { fragment: None, .. } => Err(SessionError::WrongToken),
            EffectFrame::CloneOptionChildren {
                option,
                fragment,
                cursor,
            } => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if cursor
                    == self
                        .construction
                        .node_at(option)
                        .ok_or(SessionError::ForeignNode)?
                        .child_count()
                {
                    return self.write_current_effect(c, None);
                }
                c.charge(WorkCategory::RecordVisit, 1)?;
                let child = self
                    .construction
                    .node_at(option)
                    .ok_or(SessionError::ForeignNode)?
                    .child_at(cursor)
                    .ok_or(SessionError::WrongSibling)?
                    .position();
                let document = self.node_document_ref(c, child)?;
                let child_document = document.try_clone(c)?;
                let copy = self.clone_node(c, child, child_document)?;
                self.write_current_effect(
                    c,
                    Some(EffectFrame::CloneOptionChildren {
                        option,
                        fragment,
                        cursor: cursor + 1,
                    }),
                )?;
                self.start_effect(
                    c,
                    EffectFrame::AppendClone {
                        parent: fragment,
                        copy,
                    },
                )?;
                self.start_effect(
                    c,
                    EffectFrame::CloneChildren {
                        source: child,
                        copy,
                        cursor: 0,
                        document,
                    },
                )?;
                self.start_template_clone(c, child, copy)
            }
            _ => Err(SessionError::WrongToken),
        }
    }
    pub(super) fn ancestor_optgroup(
        &self,
        c: &mut ParseControl,
        node: usize,
        select: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        let mut next = self.ordinary_parent(c, node, detached)?;
        for _ in 0..self.construction.storage().len() {
            let Some(ancestor) = next else {
                return Ok(false);
            };
            c.charge(WorkCategory::RecordVisit, 1)?;
            if ancestor == select {
                return Ok(false);
            }
            if self.html_local(c, ancestor, "optgroup")? {
                return Ok(true);
            }
            next = self.ordinary_parent(c, ancestor, detached)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn first_selected_option(
        &mut self,
        c: &mut ParseControl,
        select: usize,
    ) -> Result<Option<usize>, SessionError> {
        let list = self.retain_work_list(c, select, WorkListKind::Options { detached: None })?;
        let count = self.work_list_len(c, list)?;
        payload_write::<HtmlSelectFirstSelectedCursor>(c, 1)?;
        let mut cursor = HtmlSelectFirstSelectedCursor::new(count);
        let selected = loop {
            match cursor.action() {
                HtmlSelectFirstSelectedAction::ReadSelectedness { ordinal } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    payload_write::<HtmlSelectFirstSelectedCursor>(c, 1)?;
                    cursor
                        .reply(Some(state.flags.selectedness()))
                        .map_err(|_| SessionError::WrongToken)?;
                }
                HtmlSelectFirstSelectedAction::Complete(selection) => {
                    match surgeist_html::bounded::selectedcontent_update(false, false, selection) {
                        surgeist_html::bounded::HtmlSelectedcontentUpdate::CloneOptionChildren { ordinal } => {
                            break Some(self.work_list_node(c, list, ordinal)?.ok_or(SessionError::WrongToken)?);
                        }
                        surgeist_html::bounded::HtmlSelectedcontentUpdate::Clear => break None,
                        surgeist_html::bounded::HtmlSelectedcontentUpdate::Unavailable => return Err(SessionError::SelectMetadataUnavailable),
                        surgeist_html::bounded::HtmlSelectedcontentUpdate::Skip => return Err(SessionError::WrongToken),
                    }
                }
            }
        };
        self.release_work_list(c, list)?;
        Ok(selected)
    }
    pub(super) fn update_select(
        &mut self,
        c: &mut ParseControl,
        select: usize,
    ) -> Result<(), SessionError> {
        if surgeist_html::bounded::selectedcontent_update(
            self.form_attribute(c, select, "multiple")?.is_some(),
            false,
            HtmlSelectFirstSelection::Unavailable,
        ) == surgeist_html::bounded::HtmlSelectedcontentUpdate::Skip
        {
            return Ok(());
        }
        let list = self.retain_work_list(c, select, WorkListKind::SelectedTargets)?;
        self.start_effect(
            c,
            EffectFrame::UpdateSelect {
                select,
                list,
                cursor: 0,
            },
        )
    }
    pub(super) fn update_for_option(
        &mut self,
        c: &mut ParseControl,
        option: usize,
    ) -> Result<(), SessionError> {
        let (_, SelectMetadata::Option(state)) = self.select_metadata(c, option)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if state.flags.selectedness()
            && let Some(select) = state.nearest_select
        {
            self.update_select(c, select)?;
        }
        Ok(())
    }
    pub(super) fn intrinsic_select_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "option")? {
            let (slot, SelectMetadata::Option(mut state)) = self.select_metadata(c, node)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            let old = state.nearest_select;
            let new = self.nearest_select(c, node, detached)?;
            state.nearest_select = new;
            self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
            if old != new {
                for select in [old, new].into_iter().flatten() {
                    let changed = self.selectedness_setting(c, select, detached)?;
                    let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    if removal && (changed || state.flags.selectedness()) {
                        let cause = self
                            .evidence
                            .as_ref()
                            .map(|evidence| evidence.current_cause(c))
                            .transpose()?
                            .flatten();
                        self.effects
                            .enqueue(c, IntrinsicTask::UpdateSelect(select), cause)?;
                    }
                }
            }
        } else if !removal && self.html_local(c, node, "selectedcontent")? {
            self.recalculate_selectedcontent(c, node)?;
        }
        Ok(())
    }
    fn recalculate_selectedcontent(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        let (slot, SelectMetadata::Selectedcontent(mut state)) = self.select_metadata(c, node)?
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        state.disabled = false;
        self.set_select_metadata(c, slot, SelectMetadata::Selectedcontent(state))?;
        payload_write::<surgeist_html::bounded::HtmlSelectedcontentAncestors>(c, 1)?;
        let mut decisions = surgeist_html::bounded::HtmlSelectedcontentAncestors::default();
        let mut nearest = None;
        let mut next = self.ordinary_parent(c, node, None)?;
        for _ in 0..self.construction.storage().len() {
            let Some(ancestor) = next else {
                return Ok(nearest);
            };
            let kind = if self.html_local(c, ancestor, "select")? {
                Some(KnownHtmlElement::Select)
            } else if self.html_local(c, ancestor, "option")? {
                Some(KnownHtmlElement::Option)
            } else if self.html_local(c, ancestor, "selectedcontent")? {
                Some(KnownHtmlElement::Selectedcontent)
            } else {
                None
            };
            payload_write::<surgeist_html::bounded::HtmlSelectedcontentAncestors>(c, 1)?;
            match decisions.observe(kind) {
                surgeist_html::bounded::HtmlSelectedcontentAncestorAction::RememberNearestSelect => {
                    payload_write::<Option<usize>>(c, 1)?;
                    nearest = Some(ancestor);
                    next = self.ordinary_parent(c, ancestor, None)?;
                    continue;
                }
                surgeist_html::bounded::HtmlSelectedcontentAncestorAction::StopDisabled => {
                    state.disabled = decisions.disabled();
                    self.set_select_metadata(c, slot, SelectMetadata::Selectedcontent(state))?;
                    return Ok(nearest);
                }
                surgeist_html::bounded::HtmlSelectedcontentAncestorAction::Continue => {}
            }
            next = self.ordinary_parent(c, ancestor, None)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn connect_selectedcontent(
        &mut self,
        c: &mut ParseControl,
        target: usize,
    ) -> Result<(), SessionError> {
        let select = self.recalculate_selectedcontent(c, target)?;
        let (_, SelectMetadata::Selectedcontent(state)) = self.select_metadata(c, target)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        if !state.disabled
            && let Some(select) = select
            && self.form_attribute(c, select, "multiple")?.is_none()
        {
            let option = self.first_selected_option(c, select)?;
            self.start_effect(
                c,
                EffectFrame::UpdateTarget {
                    option,
                    target,
                    fragment: None,
                    stage: TargetStage::Prepare,
                },
            )?;
        }
        Ok(())
    }
    pub(super) fn initial_select_reset(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if !self.html_local(c, node, "select")? {
            return Ok(());
        }
        let (slot, SelectMetadata::Select(mut state)) = self.select_metadata(c, node)? else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        state.flags.reset();
        self.set_select_metadata(c, slot, SelectMetadata::Select(state))?;
        self.selectedness_setting(c, node, None)?;
        // Fresh parser-created select has no children at syntax3537 reset.
        c.charge(WorkCategory::RecordVisit, 1)?;
        if self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .child_count()
            != 0
        {
            return Err(SessionError::WrongToken);
        }
        Ok(())
    }
    pub(super) fn initial_select_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 3)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attribute = element
            .attributes()
            .as_slice()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        let selected = exact_ascii(c, Some(attribute.name().local_name()), "selected")?;
        let disabled = exact_ascii(c, Some(attribute.name().local_name()), "disabled")?;
        let size = exact_ascii(c, Some(attribute.name().local_name()), "size")?;
        let multiple = exact_ascii(c, Some(attribute.name().local_name()), "multiple")?;
        if self.html_local(c, node, "option")? {
            let (slot, SelectMetadata::Option(mut state)) = self.select_metadata(c, node)? else {
                return Err(SessionError::SelectMetadataUnavailable);
            };
            if selected && !state.flags.dirtiness() {
                state.flags.selected_attribute_changed(true);
                self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
            }
            if (selected || disabled)
                && let Some(select) = state.nearest_select
            {
                self.selectedness_setting(c, select, None)?;
            }
        } else if self.html_local(c, node, "select")? && (size || multiple) {
            self.selectedness_setting(c, node, None)?;
        } else if self.html_local(c, node, "optgroup")?
            && disabled
            && let Some(select) = self.nearest_select(c, node, None)?
        {
            self.selectedness_setting(c, select, None)?;
        }
        Ok(())
    }
    pub(super) fn select_metadata(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(usize, SelectMetadata), SessionError> {
        retained_select_metadata(&self.payloads, c, node)
    }
    pub(super) fn set_select_metadata(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        value: SelectMetadata,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        payload_write::<SelectMetadata>(c, 1)?;
        let Some(RetainedPayload::SelectFamily { state, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(SessionError::SelectMetadataUnavailable);
        };
        *state = value;
        Ok(())
    }
    pub(super) fn nearest_select(
        &self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<usize>, SessionError> {
        let mut next = self.ordinary_parent(c, node, detached)?;
        payload_write::<surgeist_html::bounded::HtmlOptionSelectAncestors>(c, 1)?;
        let mut decisions = surgeist_html::bounded::HtmlOptionSelectAncestors::default();
        for _ in 0..self.construction.storage().len() {
            let Some(node) = next else {
                payload_write::<surgeist_html::bounded::HtmlOptionSelectAncestors>(c, 1)?;
                return match decisions.finish_complete() {
                    surgeist_html::bounded::HtmlOptionSelectAncestorAction::NoSelect => Ok(None),
                    _ => Err(SessionError::WrongToken),
                };
            };
            for barrier in [
                KnownHtmlElement::Datalist,
                KnownHtmlElement::Hr,
                KnownHtmlElement::Option,
            ] {
                if self.html_local(c, node, barrier.local_name())? {
                    payload_write::<surgeist_html::bounded::HtmlOptionSelectAncestors>(c, 1)?;
                    return match decisions.observe(Some(barrier)) {
                        surgeist_html::bounded::HtmlOptionSelectAncestorAction::NoSelect => {
                            Ok(None)
                        }
                        _ => Err(SessionError::WrongToken),
                    };
                }
            }
            if self.html_local(c, node, "optgroup")? {
                payload_write::<surgeist_html::bounded::HtmlOptionSelectAncestors>(c, 1)?;
                if decisions.observe(Some(KnownHtmlElement::Optgroup))
                    == surgeist_html::bounded::HtmlOptionSelectAncestorAction::NoSelect
                {
                    return Ok(None);
                }
            }
            if self.html_local(c, node, "select")? {
                payload_write::<surgeist_html::bounded::HtmlOptionSelectAncestors>(c, 1)?;
                return match decisions.observe(Some(KnownHtmlElement::Select)) {
                    surgeist_html::bounded::HtmlOptionSelectAncestorAction::NearestSelect => {
                        Ok(Some(node))
                    }
                    _ => Err(SessionError::WrongToken),
                };
            }
            next = self.ordinary_parent(c, node, detached)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn option_disabled(
        &self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        if self.form_attribute(c, node, "disabled")?.is_some() {
            return Ok(true);
        }
        let mut next = self.ordinary_parent(c, node, detached)?;
        for _ in 0..self.construction.storage().len() {
            let Some(node) = next else { return Ok(false) };
            for kind in [
                KnownHtmlElement::Option,
                KnownHtmlElement::Select,
                KnownHtmlElement::Datalist,
                KnownHtmlElement::Hr,
                KnownHtmlElement::Optgroup,
            ] {
                if self.html_local(c, node, kind.local_name())? {
                    match surgeist_html::bounded::option_disabled_ancestor(Some(kind)) {
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::Enabled => return Ok(false),
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::ReadOptgroupDisabled => {
                            return Ok(self.form_attribute(c, node, "disabled")?.is_some());
                        }
                        surgeist_html::bounded::HtmlOptionDisabledAncestor::Continue => {}
                    }
                }
            }
            next = self.ordinary_parent(c, node, detached)?;
        }
        Err(SessionError::Cycle)
    }
    pub(super) fn select_size(
        &self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<usize, SessionError> {
        let multiple = self.form_attribute(c, node, "multiple")?.is_some();
        let fallback = usize::from(surgeist_html::bounded::select_display_size_default(
            multiple,
        ));
        let Some(value) = self.form_attribute(c, node, "size")? else {
            return Ok(fallback);
        };
        payload_write::<surgeist_html::bounded::IntegerPrefixScanner>(c, 1)?;
        payload_write::<usize>(c, 1)?;
        let mut scanner = surgeist_html::bounded::IntegerPrefixScanner::new(true);
        let mut position = 0;
        while position < value.len() {
            c.charge(WorkCategory::InspectByte, 2)?;
            payload_write::<surgeist_html::bounded::IntegerPrefixScanner>(c, 1)?;
            let unit = value.code_units()[position];
            if !scanner.feed(unit) {
                break;
            }
            payload_write::<usize>(c, 1)?;
            position += 1;
        }
        let Ok(prefix) = scanner.finish() else {
            return Ok(fallback);
        };
        payload_write::<(usize, usize)>(c, 2)?;
        let mut result = 0usize;
        let mut position = prefix.start();
        while position < prefix.end() {
            c.charge(WorkCategory::InspectByte, 2)?;
            let unit = value.code_units()[position];
            if prefix.is_negative() && unit != b'0' as u16 {
                return Ok(fallback);
            }
            payload_write::<usize>(c, 1)?;
            result = (result * 10 + usize::from(unit - b'0' as u16)).min(2);
            payload_write::<usize>(c, 1)?;
            position += 1;
        }
        Ok(result)
    }
    pub(super) fn selectedness_setting(
        &mut self,
        c: &mut ParseControl,
        select: usize,
        detached: Option<usize>,
    ) -> Result<bool, SessionError> {
        if self.form_attribute(c, select, "multiple")?.is_some() {
            return Ok(false);
        }
        let list = self.retain_work_list(c, select, WorkListKind::Options { detached })?;
        let count = self.work_list_len(c, list)?;
        payload_write::<HtmlSelectCursor>(c, 1)?;
        let mut cursor = HtmlSelectCursor::new(HtmlSelectOperation::Normalize, false, count)
            .map_err(|_| SessionError::WrongToken)?;
        let changed = loop {
            let reply = match cursor.action() {
                HtmlSelectAction::ReadSelectedness { ordinal } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    let (_, SelectMetadata::Option(state)) = self.select_metadata(c, node)? else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    HtmlSelectReply::Selectedness(state.flags.selectedness())
                }
                HtmlSelectAction::WriteSelectedness { ordinal, selected } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    let (slot, SelectMetadata::Option(mut state)) =
                        self.select_metadata(c, node)?
                    else {
                        return Err(SessionError::SelectMetadataUnavailable);
                    };
                    state.flags.set_selectedness_for_select(selected);
                    self.set_select_metadata(c, slot, SelectMetadata::Option(state))?;
                    HtmlSelectReply::Applied
                }
                HtmlSelectAction::ReadDisabled { ordinal } => {
                    let node = self
                        .work_list_node(c, list, ordinal)?
                        .ok_or(SessionError::WrongToken)?;
                    HtmlSelectReply::Disabled(self.option_disabled(c, node, detached)?)
                }
                HtmlSelectAction::ReadDisplaySize => {
                    HtmlSelectReply::DisplaySizeIsOne(self.select_size(c, select)? == 1)
                }
                HtmlSelectAction::Complete {
                    normalization_changed,
                } => break normalization_changed,
                _ => return Err(SessionError::WrongToken),
            };
            // The actual constant-state language cursor replaces the old four
            // locals. Admit its real representation before each transition.
            payload_write::<HtmlSelectCursor>(c, 1)?;
            cursor.reply(reply).map_err(|_| SessionError::WrongToken)?;
        };
        self.release_work_list(c, list)?;
        Ok(changed)
    }
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn select_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<SelectState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Select(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn option_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<OptionState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Option(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn selectedcontent_state(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<SelectedcontentState, SessionError> {
        let storage = self.active()?;
        let node = storage.index(c, node)?;
        match storage.select_metadata(c, node)?.1 {
            SelectMetadata::Selectedcontent(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn option_nearest_select(
        &self,
        c: &mut ParseControl,
        node: &ParserNode,
    ) -> Result<Option<ParserNode>, SessionError> {
        self.option_state(c, node)?
            .nearest_select
            .map(|index| self.active()?.node(c, index))
            .transpose()
    }
}

fn retained_select_metadata(
    payloads: &PayloadStorage,
    c: &mut ParseControl,
    node: usize,
) -> Result<(usize, SelectMetadata), SessionError> {
    if let Some(slot) = payloads.joined(c, node, Join::SelectFamily)? {
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(RetainedPayload::SelectFamily {
            node: Some(owner),
            state,
        }) = payloads.values.get(slot)
            && *owner == node
        {
            c.charge(
                WorkCategory::CopyByte,
                mem::size_of::<SelectMetadata>() as u64,
            )?;
            return Ok((slot, *state));
        }
    }
    Err(SessionError::SelectMetadataUnavailable)
}

impl FrozenArena {
    #[cfg(test)]
    fn retained_select_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectMetadata, SessionError> {
        let storage = self.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = &storage.view;
        c.charge(WorkCategory::ProviderCall, 1)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let position = arena
            .node_by_result_id(node)
            .map_err(|_| SessionError::ForeignNode)?
            .position();
        Ok(retained_select_metadata(&storage.payloads, c, position)?.1)
    }
    #[cfg(test)]
    pub(crate) fn select_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Select(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn option_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<OptionState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Option(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn selectedcontent_state(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<SelectedcontentState, SessionError> {
        match self.retained_select_state(c, node)? {
            SelectMetadata::Selectedcontent(state) => Ok(state),
            _ => Err(SessionError::SelectMetadataUnavailable),
        }
    }
    #[cfg(test)]
    pub(crate) fn option_nearest_select(
        &self,
        c: &mut ParseControl,
        node: &HtmlNodeId,
    ) -> Result<Option<HtmlNodeId>, SessionError> {
        let owner = self.option_state(c, node)?.nearest_select;
        self.resolve_form_owner(c, owner)
    }
}
